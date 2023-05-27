//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

use crate::*;
use sdl2::keyboard::Keycode;
use std::rc::Rc;

pub struct LiveMap {
    assets: Rc<GameAssets>,
    cells: Vec<MapCell>,
    width: u16,
    height: u16,
    details: MapDetails,
    _tmp_idx: usize,
    _tmp_timer: f64,
}

impl LiveMap {
    pub fn new(assets: Rc<GameAssets>, index: usize, mapsrc: &MapData) -> Self {
        let width = mapsrc.width;
        let height = mapsrc.height;
        let cells = maploader::load_map_to_cells(mapsrc);
        let details = MapDetails::new(index, mapsrc);

        // TODO: compute tile flags, extract doors, live things, AMBUSH tiles, count enemies/treasures/secrets
        // -> see WOLF3D sources - e.g. https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
        Self {
            assets,
            cells,
            width,
            height,
            details,
            _tmp_idx: 0,
            _tmp_timer: 0.0,
        }
    }

    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn cell(&self, x: i32, y: i32) -> Option<&MapCell> {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            let idx = (y * w + x) as usize;
            self.cells.get(idx)
        } else {
            None
        }
    }

    pub fn handle_inputs(&mut self, inputs: &mut InputManager, elapsed_time: f64) -> Option<GameState> {
        // TODO: update doors, secret walls, actors - only if NOT paused

        if inputs.consume_key(Keycode::Tab) {
            return Some(GameState::Automap);
        }

        // TODO temporary hack, to auto-cycle through graphics
        self._tmp_timer += elapsed_time;
        let i = self._tmp_timer.floor().clamp(0.0, 10.0) as usize;
        self._tmp_timer -= i as f64;
        self._tmp_idx = (self._tmp_idx + i) % 1000;

        // TODO temp
        None
    }

    pub fn update_actors(&mut self, _elapsed_time: f64) {
        // TODO: update player - only if in 3D view and NOT paused
    }

    #[inline]
    pub fn automap_description(&self) -> &str {
        &self.details.descr_msg
    }

    #[inline]
    pub fn automap_secrets(&self) -> String {
        self.details.secrets_msg()
    }

    pub fn paint_3d(&self, scrbuf: &mut ScreenBuffer) {
        // TODO (later) use correct sky color per game and level
        // (0x1D, 0xBF, 0x4E and 0x8D)
        const SKY_COLOR: u8 = 0x1D;
        const FLOOR_COLOR: u8 = 0x19;

        let w = scrbuf.width() as i32;
        let h = scrbuf.height() as i32;

        // paint sky and floor first
        scrbuf.fill_rect(0, 0, w, h / 2, SKY_COLOR);
        scrbuf.fill_rect(0, h / 2, w, h / 2, FLOOR_COLOR);

        // TODO implement 3D view !!!!!!!

        // TODO temporary paint gfx
        let x0 = w - 80;
        let y0 = (scrbuf.height() - 80) as i32;

        // paint wall
        let wallidx = self._tmp_idx % self.assets.walls.len();
        let wall = &self.assets.walls[wallidx];
        _temp_paint_pic(wall, x0, 5, scrbuf);
        let str = format!("WALL #{wallidx}");
        self.assets.font1.draw_text(x0, 72, &str, 14, scrbuf);

        // paint sprite
        let sprtidx = self._tmp_idx % self.assets.sprites.len();
        let sprite = &self.assets.sprites[sprtidx];
        _temp_paint_pic(sprite, x0, y0, scrbuf);
        let str = format!("SPRT #{sprtidx}");
        self.assets.font1.draw_text(x0, y0 + 67, &str, 14, scrbuf);
    }
    // TODO ............
}

//--------------------
// Internal stuff
//--------------------

struct MapDetails {
    //name: String,
    descr_msg: String,
    //episode: u8,
    //level: u8,
    total_enemies: u16,
    total_secrets: u16,
    total_treasures: u16,
    cnt_kills: u16,
    cnt_secrets: u16,
    cnt_treasures: u16,
}

impl MapDetails {
    fn new(index: usize, mapsrc: &MapData) -> Self {
        let name = mapsrc.name.to_string();
        let episode = (index / 10 + 1) as u8;
        let level = (index % 10 + 1) as u8;
        let descr_msg = format!("{} - ep. {}, level {}", name, episode, level);
        Self {
            descr_msg,
            total_enemies: 0,
            total_secrets: 0,
            total_treasures: 0,
            cnt_kills: 0,
            cnt_secrets: 0,
            cnt_treasures: 0,
        }
    }

    fn secrets_msg(&self) -> String {
        format!(
            "K: {}/{}   T: {}/{}   S: {}/{}",
            self.cnt_kills,
            self.total_enemies,
            self.cnt_treasures,
            self.total_treasures,
            self.cnt_secrets,
            self.total_secrets
        )
    }
}

//-------------------

// TODO move "live" item structs to a separate mod ?!?

// struct Door {
//     // TODO ..
// }

// struct Player {
//     // TODO ..
// }

// struct Enemy {
//     // TODO ..
// }

// TODO temporary paint a graphic
fn _temp_paint_pic(gfx: &GfxData, x0: i32, y0: i32, scrbuf: &mut ScreenBuffer) {
    const BG: u8 = 31;
    let (pw, ph) = gfx.size();
    if pw == 0 || ph == 0 {
        // empty pic !!
        scrbuf.fill_rect(x0, y0, 8, 8, BG);
    } else {
        scrbuf.fill_rect(x0, y0, pw as i32, ph as i32, BG);
        gfx.draw(x0, y0, scrbuf);
    }
}

// // TODO temporary paint palette
// fn _temp_paint_palette(scrbuf: &mut ScreenBuffer) {
//     const SQSIZE: i32 = 8;
//     let mut cidx: i32 = 0;
//     for y in 0..16 {
//         for x in 0..16 {
//             let c = cidx as u8;
//             scrbuf.fill_rect(x * SQSIZE, y * SQSIZE, SQSIZE, SQSIZE, c);
//             cidx += 1;
//         }
//     }
// }
