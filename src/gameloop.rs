//! Main game loop.
//! Also acts as a facade, to hold and manage all game objects
//! (assets, renderers, other managers etc)

use crate::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::rc::Rc;

pub struct GameLoop {
    scrbuf: ScreenBuffer,
    assets: Rc<GameAssets>,
    mode: GameMode,
    mapidx: usize,
    livemap: LiveMap,
    automap: AutomapRenderer,
    inputs: InputManager,
    status: GameStatus,
    status_bar_enabled: bool, // TODO move to some GameConfig struct
}

impl GameLoop {
    pub fn new(width: i32, height: i32, pixel_size: i32, assets: GameAssets) -> Self {
        let ga = Rc::from(assets);
        let livemap = LiveMap::new(Rc::clone(&ga), 0, &ga.maps[0]);

        let mut zelf = Self {
            scrbuf: ScreenBuffer::new(width, height, ga.is_sod),
            assets: Rc::clone(&ga),
            mode: GameMode::Live,
            mapidx: 0,
            livemap,
            automap: AutomapRenderer::new(Rc::clone(&ga)),
            inputs: InputManager::new(pixel_size),
            status: GameStatus::new(0), // TODO get episode from user selection
            status_bar_enabled: false,
        };

        zelf.enable_status_bar(true);
        zelf
    }

    pub fn start_map(&mut self, mapidx: usize) {
        self.mapidx = mapidx;
        let map = &self.assets.maps[mapidx];
        self.livemap = LiveMap::new(Rc::clone(&self.assets), mapidx, map);
    }

    fn enable_status_bar(&mut self, enabled: bool) {
        self.status_bar_enabled = enabled;
        self.scrbuf.enable_status_bar(enabled);
    }
}

impl GraphicsLoop for GameLoop {
    fn handle_event(&mut self, event: &Event) -> bool {
        self.inputs.handle_event(event);

        // TODO TEMP - quick exit via Esc or F12
        if self.inputs.consume_key(Keycode::Escape) || self.inputs.consume_key(Keycode::F12) {
            return false;
        }

        // TODO temp hack, to scroll between maps
        let ml = self.assets.maps.len();
        if self.inputs.consume_key(Keycode::Insert) {
            let idx = (self.mapidx + ml - 1) % ml;
            self.start_map(idx);
        } else if self.inputs.consume_key(Keycode::Delete) {
            let idx = (self.mapidx + 1) % ml;
            self.start_map(idx);
        }

        true
    }

    fn update_state(&mut self, elapsed_time: f64) -> bool {
        // handle status bar
        // TODO: statusbar enable/disable keys only during 32 or Automap !?
        if self.inputs.consume_key(Keycode::Minus) {
            self.enable_status_bar(true);
        } else if self.inputs.consume_key(Keycode::Equals) {
            self.enable_status_bar(false);
        }

        if self.status_bar_enabled {
            self.status.paint_status_bar(&mut self.scrbuf, &self.assets);
        }

        // update depending on game state
        let new_state;
        match self.mode {
            GameMode::Live => {
                new_state = self.livemap.handle_inputs(&mut self.inputs, elapsed_time);
                self.livemap.paint_3d(&mut self.scrbuf);
            }
            GameMode::Automap => {
                new_state = self.automap.handle_inputs(&mut self.inputs, elapsed_time);
                self.automap.paint(&self.livemap, &mut self.scrbuf);
            }
        }
        if let Some(state_update) = new_state {
            self.mode = state_update;
        }

        true
    }

    fn paint(&self, painter: &mut dyn Painter) {
        self.scrbuf.paint(painter);
    }
}
