//! Main game loop.
//! Also acts as a facade, to hold and manage all game objects
//! (assets, renderers, other managers etc)

use crate::*;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use std::rc::Rc;

pub struct GameLoop {
    scrbuf: ScreenBuffer,
    assets: Rc<GameAssets>,
    state: GameState,
    mapidx: usize,
    livemap: LiveMapSimulator,
    renderer: ThreeDRenderer,
    automap: AutomapRenderer,
    inputs: InputManager,
}

impl GameLoop {
    pub fn new(width: usize, height: usize, pixel_size: i32, assets: GameAssets) -> Self {
        let is_sod = assets.is_sod();
        let livemap = LiveMapSimulator::new(0, assets.maps.get(0).unwrap());
        let ga = Rc::from(assets);
        Self {
            scrbuf: ScreenBuffer::new(width, height, is_sod),
            assets: Rc::clone(&ga),
            state: GameState::Live,
            mapidx: 0,
            livemap,
            renderer: ThreeDRenderer::new(Rc::clone(&ga)),
            automap: AutomapRenderer::new(Rc::clone(&ga)),
            inputs: InputManager::new(pixel_size),
        }
    }

    pub fn start_map(&mut self, mapidx: usize) {
        self.mapidx = mapidx;
        let map = &self.assets.maps[mapidx];
        self.livemap = LiveMapSimulator::new(mapidx, map);
    }
}

impl GraphicsLoop for GameLoop {
    fn handle_event(&mut self, event: &Event) -> bool {
        self.inputs.handle_event(event);

        // TODO temp hack, to scroll between maps
        let ml = self.assets.maps.len();
        if self.inputs.consume_key(sdl2::keyboard::Keycode::Insert) {
            let idx = (self.mapidx + ml - 1) % ml;
            self.start_map(idx);
        } else if self.inputs.consume_key(sdl2::keyboard::Keycode::Delete) {
            let idx = (self.mapidx + 1) % ml;
            self.start_map(idx);
        }

        true
    }

    fn update_state(&mut self, elapsed_time: f64) -> bool {
        // update depending on game state
        let new_state;
        match self.state {
            GameState::Live => {
                new_state = self.renderer.handle_inputs(&mut self.inputs, elapsed_time);
                self.renderer.paint(&self.livemap, &mut self.scrbuf);
            }
            GameState::Automap => {
                new_state = self.automap.handle_inputs(&mut self.inputs, elapsed_time);
                self.automap.paint(&self.livemap, &mut self.scrbuf);
            }
        }
        if let Some(state_update) = new_state {
            self.state = state_update;
        }

        // TODO temporary hack, to show mouse position
        if self.inputs.mouse_btn(MouseButton::Left) {
            let (x, y) = self.inputs.mouse_pos();
            self.scrbuf.fill_rect(x - 1, y - 1, 3, 3, 15);
        }

        true
    }

    fn paint(&self, painter: &mut dyn Painter) {
        self.scrbuf.paint(painter);
    }
}
