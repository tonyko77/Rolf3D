//! SDL2 wrapper, to simplify using SDL2

use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

//-----------------

/// Structure for an RGB color.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    #[inline]
    pub fn from(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }

    // grayscale = 0.3 * R + 0.59 * G + 0.11 * B
    #[inline]
    pub fn grayscale(&self) -> u8 {
        let gray = 30 * (self.r as u32) + 59 * (self.g as u32) + 11 * (self.b as u32);
        (gray / 100) as u8
    }
}

//-----------------

/// Enum for if/how to slep during each game loop execution.
#[derive(PartialEq, Eq)]
pub enum SleepKind {
    NONE,
    YIELD,
    SLEEP(u32),
}

//-----------------

/// Painter interface, to be passed to client code so it can perform painting.
/// *This is not meant to be implemented by client code.*
pub trait Painter {
    /// Get the width of the screen, in pixels.
    fn get_screen_width(&self) -> i32;

    /// Get the height of the screen, in pixels.
    fn get_screen_height(&self) -> i32;

    /// Draw a single pixel.
    fn draw_pixel(&mut self, x: i32, y: i32, color: RGB);
}

//-----------------

/// The configuration to be used for initializing SDL.
pub struct SdlConfiguration {
    title: String,
    scr_width: i32,
    scr_height: i32,
    pixel_size: i32,
    sleep_kind: SleepKind,
}

impl SdlConfiguration {
    pub fn new(title: &str, scr_width: i32, scr_height: i32, pixel_size: i32, sleep_kind: SleepKind) -> Self {
        assert!(scr_width > 0);
        assert!(scr_height > 0);
        assert!(pixel_size > 0);
        SdlConfiguration {
            title: String::from(title),
            scr_width,
            scr_height,
            pixel_size,
            sleep_kind,
        }
    }

    #[inline]
    pub fn scr_width(&self) -> i32 {
        self.scr_width
    }

    #[inline]
    pub fn scr_height(&self) -> i32 {
        self.scr_height
    }
}

/// Trait to be implemented by clients of `run_sdl_loop`.
/// Its methods will be called periodically, during the main game loop.
pub trait GraphicsLoop {
    /// Handle/capture events (e.g. keys, mouse etc).
    fn handle_event(&mut self, event: &Event) -> bool;

    /// Update the internal state.
    fn update_state(&mut self, elapsed_time: f64) -> bool;

    /// Paint the world, based on the updated internal state.
    fn paint(&self, painter: &mut dyn Painter);

    /// Called at the beginning of every graphic loop.
    /// Can be used to e.g. reset some data, like mouse relative motion.
    fn on_start_loop(&mut self) {}
}

/// Main function to run the continuous game loop.
pub fn run_game_loop(cfg: &SdlConfiguration, gfx_loop: &mut dyn GraphicsLoop) -> Result<(), String> {
    assert!(cfg.scr_width > 0);
    assert!(cfg.scr_height > 0);
    assert!(cfg.pixel_size > 0);

    let win_width = (cfg.scr_width * cfg.pixel_size) as u32;
    let win_height = (cfg.scr_height * cfg.pixel_size) as u32;
    let scr_width = cfg.scr_width as u32;
    let scr_height = cfg.scr_height as u32;

    // create window
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window(&cfg.title, win_width, win_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    // create texture, to paint on
    let texture_creator = canvas.texture_creator();
    let mut screen_buffer = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, scr_width, scr_height)
        .map_err(|e| e.to_string())?;

    let mut timer = FpsAndElapsedCounter::new();
    let mut last_fps = 42;
    let mut event_pump = sdl_context.event_pump()?;

    // Main game loop
    'running: loop {
        gfx_loop.on_start_loop();

        // consume the event loop
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event, .. } => {
                    // TODO - steal mouse only when game is RUNNING ?!
                    if win_event == WindowEvent::FocusGained {
                        // sdl_context.mouse().capture(true);
                        // sdl_context.mouse().show_cursor(false);
                    } else if win_event == WindowEvent::FocusLost {
                        sdl_context.mouse().capture(false);
                        sdl_context.mouse().show_cursor(true);
                    }
                }
                _ => {
                    if !gfx_loop.handle_event(&event) {
                        break 'running;
                    }
                }
            }
        }

        // compute time
        let elapsed_time = timer.update_and_get_ellapsed_time();
        if last_fps != timer.fps {
            last_fps = timer.fps;
            let title_with_fps = format!("{} - FPS: {}", cfg.title, last_fps);
            canvas
                .window_mut()
                .set_title(&title_with_fps)
                .map_err(|e| e.to_string())?;
        }

        // update the internal state
        if !gfx_loop.update_state(elapsed_time) {
            break 'running;
        }

        // paint the screen, using a SDL2 streaming texture
        // - see: https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/renderer-texture.rs
        // - see: https://www.reddit.com/r/cpp_questions/comments/eqwsao/sdl_rendering_way_too_slow/
        screen_buffer.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            // all painting must be done in this closure
            let mut painter = InternalTexturePainter {
                buffer,
                pitch,
                scr_width: cfg.scr_width,
                scr_height: cfg.scr_height,
            };
            gfx_loop.paint(&mut painter);
        })?;

        // paint texture on screen
        canvas.copy(&screen_buffer, None, None)?;
        canvas.present();

        // sleep a bit, so we don't hog the CPU
        match cfg.sleep_kind {
            SleepKind::SLEEP(nanos) => {
                std::thread::sleep(Duration::new(0, nanos));
            }
            SleepKind::YIELD => {
                std::thread::yield_now();
            }
            _ => {}
        }
    }

    Ok(())
}

//--------------------------------
// Internal details

struct InternalTexturePainter<'a> {
    buffer: &'a mut [u8],
    pitch: usize,
    scr_width: i32,
    scr_height: i32,
}

impl<'a> Painter for InternalTexturePainter<'a> {
    fn get_screen_width(&self) -> i32 {
        self.scr_width
    }

    fn get_screen_height(&self) -> i32 {
        self.scr_height
    }

    fn draw_pixel(&mut self, x: i32, y: i32, color: RGB) {
        if x >= 0 && y >= 0 && x < self.scr_width && y < self.scr_height {
            let offset = (y as usize) * self.pitch + (x as usize) * 3;
            self.buffer[offset + 0] = color.r;
            self.buffer[offset + 1] = color.g;
            self.buffer[offset + 2] = color.b;
        }
    }
}

//--------------------------

struct FpsAndElapsedCounter {
    time_sum: f64,
    time_cnt: u32,
    fps: u32,
    last_moment: Instant,
}

impl FpsAndElapsedCounter {
    fn new() -> Self {
        FpsAndElapsedCounter {
            time_cnt: 0,
            time_sum: 0.0,
            last_moment: Instant::now(),
            fps: 0,
        }
    }

    fn update_and_get_ellapsed_time(&mut self) -> f64 {
        // compute time
        let next_moment = Instant::now();
        let elapsed_time = next_moment.duration_since(self.last_moment).as_secs_f64();
        self.last_moment = next_moment;

        // compute FPS
        self.time_sum += elapsed_time;
        self.time_cnt += 1;
        if self.time_sum >= 1.0 {
            let avg = self.time_sum / (self.time_cnt as f64);
            self.fps = if avg <= 0.0 { 999999 } else { (1.0 / avg) as u32 };
            self.time_cnt = 0;
            self.time_sum = 0.0;
        }

        elapsed_time
    }
}
