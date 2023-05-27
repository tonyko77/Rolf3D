//! ThreeDRenderer - renders the 3D world.

use crate::*;
use sdl2::keyboard::Keycode;
use std::rc::Rc;

pub struct ThreeDRenderer {
    assets: Rc<GameAssets>,
    // TODO ................
    _tmp_idx: usize,
}

impl ThreeDRenderer {
    pub fn new(assets: Rc<GameAssets>) -> Self {
        Self { assets, _tmp_idx: 0 }
    }

    pub fn handle_inputs(&mut self, inputs: &mut InputManager, _elapsed_time: f64) -> Option<GameState> {
        if inputs.consume_key(Keycode::Tab) {
            return Some(GameState::Automap);
        }

        // TODO temporary hack, to cycle through graphics
        if inputs.consume_key(Keycode::PageUp) {
            self._tmp_idx = (self._tmp_idx + 999) % 1000;
        } else if inputs.consume_key(Keycode::PageDown) {
            self._tmp_idx = (self._tmp_idx + 1) % 1000;
        } else if inputs.consume_key(Keycode::Home) {
            self._tmp_idx = 0;
        }

        // TODO temp
        None
    }

    pub fn paint(&self, _map: &LiveMapSimulator, scrbuf: &mut ScreenBuffer) {
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
        _temp_paint_palette(scrbuf);
        let x0 = (scrbuf.width() - 100) as i32;
        let y0 = (scrbuf.height() - 202) as i32;

        // paint wall
        let wallidx = self._tmp_idx % self.assets.walls.len();
        let wall = &self.assets.walls[wallidx];
        _temp_paint_pic(wall, x0, 10, scrbuf);
        let str = format!("WALL #{wallidx}");
        self.assets.font1.draw_text(x0, 80, &str, 14, scrbuf);

        // paint sprite
        let sprtidx = self._tmp_idx % self.assets.sprites.len();
        let sprite = &self.assets.sprites[sprtidx];
        _temp_paint_pic(sprite, x0, y0, scrbuf);
        let str = format!("SPRT #{sprtidx}");
        self.assets.font1.draw_text(x0, y0 - 16, &str, 14, scrbuf);

        // paint pics
        let picidx = self._tmp_idx % self.assets.pics.len();
        let pic = &self.assets.pics[picidx];
        _temp_paint_pic(pic, 0, y0, scrbuf);
        let str = format!("PIC #{picidx}");
        self.assets.font1.draw_text(0, y0 - 16, &str, 14, scrbuf);

        // paint fonts
        let char_idx = self._tmp_idx % 100;
        let ch = (char_idx + 33) as u8;
        let str = format!("{} = {}", ch as char, ch);
        self.assets.font1.draw_text(170, 10, &str, 11, scrbuf);
        self.assets.font2.draw_text(170, 30, &str, 12, scrbuf);
    }
}

//----------------------
//  Internal stuff

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

// TODO temporary paint palette
fn _temp_paint_palette(scrbuf: &mut ScreenBuffer) {
    const SQSIZE: i32 = 8;
    let mut cidx: i32 = 0;
    for y in 0..16 {
        for x in 0..16 {
            let c = cidx as u8;
            scrbuf.fill_rect(x * SQSIZE, y * SQSIZE, SQSIZE, SQSIZE, c);
            cidx += 1;
        }
    }
}
