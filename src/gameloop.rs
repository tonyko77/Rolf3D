//! Main game loop.
//! Also acts as a facade, to hold and manage all game objects
//! (assets, renderers, other managers etc)

use crate::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::rc::Rc;

pub struct GameLoop {
    scrbuf: ScreenBuffer,
    _assets: Rc<GameAssets>,
    mode: GameMode,
    livemap: LiveMap,
    automap: AutomapRenderer,
    inputs: InputManager,
    status_bar_enabled: bool, // TODO move to some GameConfig struct
}

impl GameLoop {
    // TODO temporary hack
    pub fn new(width: i32, height: i32, pixel_size: i32, assets: GameAssets) -> Self {
        let ga = Rc::from(assets);
        let livemap = LiveMap::new(Rc::clone(&ga), 0);

        let mut zelf = Self {
            scrbuf: ScreenBuffer::new(width, height, ga.is_sod),
            _assets: Rc::clone(&ga),
            mode: GameMode::Live,
            livemap,
            automap: AutomapRenderer::new(Rc::clone(&ga)),
            inputs: InputManager::new(pixel_size),
            status_bar_enabled: false,
        };

        zelf.enable_status_bar(true);
        zelf
    }

    fn enable_status_bar(&mut self, enabled: bool) {
        self.status_bar_enabled = enabled;
        self.scrbuf.enable_status_bar(enabled);
    }
}

impl GraphicsLoop for GameLoop {
    fn on_start_loop(&mut self) {
        self.inputs.reset_mouse_movement();
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        self.inputs.handle_event(event);

        // TODO TEMP - quick exit via Esc or F12
        if self.inputs.consume_key(Keycode::F12) {
            //|| self.inputs.consume_key(Keycode::Escape) {
            return false;
        }

        // TODO temp hack, to scroll between maps
        if self.inputs.consume_key(Keycode::Insert) {
            self.livemap.go_to_next_floor();
        }
        true
    }

    fn update_state(&mut self, elapsed_time: f64) -> bool {
        // show/hide status bar
        if self.inputs.consume_key(Keycode::Minus) {
            self.enable_status_bar(true);
        } else if self.inputs.consume_key(Keycode::Equals) {
            self.enable_status_bar(false);
        }

        if self.inputs.consume_key(Keycode::Tab) {
            match self.mode {
                GameMode::Live => self.mode = GameMode::Automap,
                GameMode::Automap => self.mode = GameMode::Live,
            }
        }

        // TODO temporary: manual loop through pics
        if self.inputs.consume_key(Keycode::F9) {
            _temp_advance_back();
        } else if self.inputs.consume_key(Keycode::F10) {
            _temp_advance_fwd();
        }

        // update depending on game state
        match self.mode {
            GameMode::Live => {
                self.livemap.handle_inputs(&mut self.inputs, elapsed_time);
                self.livemap.paint_3d(&mut self.scrbuf);
            }
            GameMode::Automap => {
                self.automap.handle_inputs(&mut self.inputs, elapsed_time);
                self.automap.paint(&self.livemap, &mut self.scrbuf);
            }
        }

        if self.status_bar_enabled {
            self.livemap.paint_status_bar(&mut self.scrbuf);
        }

        true
    }

    fn paint(&self, painter: &mut dyn Painter) {
        self.scrbuf.paint(painter);
    }
}
