//! AutomapRenderer - renders the automap using the LiveMapSimulator.

// TODO:
//  1. Use LiveMap instead of MapData !!
//  2. game is ALWAYS PAUSED in automap !!

use crate::{defs::*, input::InputManager, GameAssets, ScreenBuffer};
use sdl2::keyboard::Keycode;
use std::rc::Rc;

// constants for movement and scaling speeds
const DEFAULT_SCALE: f64 = 16.5;
const MIN_SCALE: f64 = 10.5;
const MAX_SCALE: f64 = 40.5;
const MOVE_SPEED: f64 = 10.0;
const SCALE_SPEED: f64 = 6.0;
const MIN_POS: f64 = -4.0;
const MAX_POS: f64 = 54.0;
const DIV_MOUSE: f64 = 12.0;

pub struct AutomapRenderer {
    assets: Rc<GameAssets>,
    mapidx: usize,
    xpos: f64,
    ypos: f64,
    scale: f64,
}

impl AutomapRenderer {
    pub fn new(assets: Rc<GameAssets>) -> Self {
        Self {
            assets,
            mapidx: 0,
            xpos: 0.0,
            ypos: 0.0,
            scale: DEFAULT_SCALE,
        }
    }

    pub fn reset_map(&mut self, mapidx: usize) {
        assert!(mapidx < self.assets.maps.len());
        self.mapidx = mapidx;
        self.xpos = 0.0;
        self.ypos = 0.0;
    }

    pub fn handle_inputs(&mut self, inputs: &mut InputManager, elapsed_time: f64) {
        if inputs.key(Keycode::W) || inputs.key(Keycode::Up) {
            self.ypos = (self.ypos - MOVE_SPEED * elapsed_time).clamp(MIN_POS, MAX_POS);
        } else if inputs.key(Keycode::S) || inputs.key(Keycode::Down) {
            self.ypos = (self.ypos + MOVE_SPEED * elapsed_time).clamp(MIN_POS, MAX_POS);
        }

        if inputs.key(Keycode::A) || inputs.key(Keycode::Left) {
            self.xpos = (self.xpos - MOVE_SPEED * elapsed_time).clamp(MIN_POS, MAX_POS);
        } else if inputs.key(Keycode::D) || inputs.key(Keycode::Right) {
            self.xpos = (self.xpos + MOVE_SPEED * elapsed_time).clamp(MIN_POS, MAX_POS);
        }

        if inputs.key(Keycode::KpMinus) {
            self.scale = (self.scale - SCALE_SPEED * elapsed_time).clamp(MIN_SCALE, MAX_SCALE);
        } else if inputs.key(Keycode::KpPlus) {
            self.scale = (self.scale + SCALE_SPEED * elapsed_time).clamp(MIN_SCALE, MAX_SCALE);
        }

        let (dx, dy) = inputs.consume_mouse_motion();
        self.xpos = (self.xpos - (dx as f64) / DIV_MOUSE).clamp(MIN_POS, MAX_POS);
        self.ypos = (self.ypos - (dy as f64) / DIV_MOUSE).clamp(MIN_POS, MAX_POS);
    }

    pub fn paint(&self, scrbuf: &mut ScreenBuffer) {
        let sw = scrbuf.width() as i32;
        let sh = scrbuf.height() as i32;
        scrbuf.fill_rect(0, 0, sw, sh, 0);

        let scl = self.scale as i32;
        let pos_x = self.xpos.floor() as i32;
        let pos_y = self.ypos.floor() as i32;
        let frac_x = ((self.xpos - (pos_x as f64)) * self.scale) as i32;
        let frac_y = ((self.ypos - (pos_y as f64)) * self.scale) as i32;

        let map = &self.assets.maps[self.mapidx];
        let mw = map.width as i32;
        let mh = map.height as i32;

        for y in 0..mh {
            for x in 0..mw {
                let xx = (x as i32) + pos_x;
                let yy = (y as i32) + pos_y;
                let tile = map.tile(xx, yy);
                let thng = map.thing(xx, yy);
                let ix = (x * scl) - frac_x;
                let iy = (y * scl) - frac_y;

                if tile == 0 {
                    // 0 tiles are MY CREATION -> out of bounds tile :/
                    continue;
                }

                if tile >= 90 && tile <= 101 {
                    // => door, vertical if even, lock = (tile - 90|91)/2
                    let widx = if tile >= 100 { 24 } else { (tile + 8) as usize };

                    if widx < self.assets.walls.len() {
                        let wall = &self.assets.walls[widx];
                        wall.draw_scaled(ix, iy, scl, scrbuf);
                    } else {
                        let wcol = (tile - 89) as u8;
                        scrbuf.fill_rect(ix, iy, scl, scl, 14);
                        scrbuf.fill_rect(ix + 1, iy + 1, scl - 2, scl - 2, wcol);
                    }
                } else if tile == AMBUSHTILE {
                    // ambush tile - has special meaning
                    scrbuf.fill_rect(ix, iy, scl, scl, 31);
                    scrbuf.fill_rect(ix + 1, iy + 1, scl - 2, scl - 2, 6);
                } else if tile < AREATILE {
                    // solid wall => draw wall rect

                    // WHICH WALL TEXTURE corresponds to each solid tile:
                    // * wall textures are "in pairs" - alternating light and dark versions
                    // => (tile * 2) selects light/dark version, then -2 makes it 0-based
                    // !!ALSO!! LIGHT walls are used for N/S walls, and DARK for E/W walls
                    let widx = if tile == 21 { 41 } else { ((tile - 1) * 2) as usize };

                    if widx < self.assets.walls.len() {
                        let wall = &self.assets.walls[widx];
                        wall.draw_scaled(ix, iy, scl, scrbuf);
                    } else {
                        let wcol = (tile & 0xFF) as u8;
                        scrbuf.fill_rect(ix, iy, scl, scl, 15);
                        scrbuf.fill_rect(ix + 1, iy + 1, scl - 2, scl - 2, wcol);
                    }
                } else {
                    // empty area
                    let cl = (tile - AREATILE) as u8;
                    scrbuf.fill_rect(ix + 1, iy + 1, scl - 2, scl - 2, cl);

                    // => TODO: what is the hidden meaning behind various empty area codes
                    // they seem to be between 108 (AREATILE + 1) and ~143
                    // OBSERVATION: all empty tiles in one room have THE SAME VALUE
                    // => maybe a way to alert enemies from the same area ?!?
                }

                // draw thing
                if thng > 0 {
                    scrbuf.fill_rect(ix + 2, iy + 2, 4, 4, 0);
                    scrbuf.fill_rect(ix + 3, iy + 3, 2, 2, (thng & 0xFF) as u8);
                }
            }
        }
    }
}
