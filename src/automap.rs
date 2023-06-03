//! AutomapRenderer - renders the automap using the LiveMapSimulator.

// TODO:
//  1. Use LiveMap instead of MapData !!
//  2. game is ALWAYS PAUSED in automap !!

use crate::*;
use sdl2::keyboard::Keycode;
use std::rc::Rc;

// constants for movement and scaling speeds
const DEFAULT_SCALE: f64 = 20.5;
const MIN_SCALE: f64 = 10.5;
const MAX_SCALE: f64 = 40.5;
const MOVE_SPEED: f64 = 12.0;
const SCALE_SPEED: f64 = 8.0;
const MIN_POS: f64 = -14.0;
const MAX_POS: f64 = 60.0;
const DIV_MOUSE: f64 = 12.0;

const TEXIDX_ELEVATOR_SWITCH: usize = 41;

pub struct AutomapRenderer {
    assets: Rc<GameAssets>,
    xpos: f64,
    ypos: f64,
    scale: f64,
}

impl AutomapRenderer {
    pub fn new(assets: Rc<GameAssets>) -> Self {
        Self {
            assets,
            xpos: 0.0,
            ypos: 0.0,
            scale: DEFAULT_SCALE,
        }
    }

    pub fn handle_inputs(&mut self, inputs: &mut InputManager, elapsed_time: f64) -> Option<GameMode> {
        if inputs.consume_key(Keycode::Tab) {
            return Some(GameMode::Live);
        }

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

        if inputs.mouse_btn(sdl2::mouse::MouseButton::Left) {
            let (dx, dy) = inputs.mouse_motion();
            self.xpos = (self.xpos - (dx as f64) / DIV_MOUSE).clamp(MIN_POS, MAX_POS);
            self.ypos = (self.ypos - (dy as f64) / DIV_MOUSE).clamp(MIN_POS, MAX_POS);
        }

        None
    }

    pub fn paint(&self, map: &LiveMap, scrbuf: &mut ScreenBuffer) {
        let sw = scrbuf.scr_width() as i32;
        let sh = scrbuf.scr_height() as i32;
        scrbuf.fill_rect(0, 0, sw, sh, 0);

        let scl = self.scale as i32;

        let pos_x = self.xpos.floor() as i32;
        let pos_y = self.ypos.floor() as i32;
        let frac_x = ((self.xpos - (pos_x as f64)) * self.scale) as i32;
        let frac_y = ((self.ypos - (pos_y as f64)) * self.scale) as i32;

        // TODO temporary - coords of info cell
        let mut tmp_x = 0;
        let mut tmp_y = 0;

        // paint automap
        let mw = map.width() as i32;
        let mh = map.height() as i32;
        for y in 0..mh {
            for x in 0..mw {
                let xx = (x as i32) + pos_x;
                let yy = (y as i32) + pos_y;
                if let Some(cell) = map.cell(xx, yy) {
                    let ix = (x * scl) - frac_x;
                    let iy = (y * scl) - frac_y;
                    // paint texture
                    let tex = cell.get_texture();
                    let tex = if tex == (ELEVATOR_TILE as usize) {
                        TEXIDX_ELEVATOR_SWITCH
                    } else {
                        tex
                    };
                    if tex < 0xF000 {
                        if tex < self.assets.walls.len() {
                            let wall = &self.assets.walls[tex];
                            scrbuf.draw_scaled_pic(ix, iy, scl, wall);
                        } else {
                            // PROBLEM - MISSING texture ?!
                            println!("[WARN] MISSING texture: {tex}");
                            scrbuf.fill_rect(ix, iy, scl, scl, 0xFF);
                        }
                    } else {
                        // gray background for empty areas
                        scrbuf.fill_rect(ix, iy, scl, scl, 30);
                    }
                    // TODO temporary paint thing sprites
                    let thng = cell.thing;
                    if thng > 0 {
                        let spr = cell.get_sprite() as usize;
                        if spr < self.assets.sprites.len() {
                            scrbuf.fill_rect(ix, iy, scl, scl, 30);
                            let sprite = &self.assets.sprites[spr];
                            scrbuf.draw_scaled_pic(ix, iy, scl, sprite);
                        } else {
                            // Special thing
                            // TODO temporary - paint marker for special things
                            if tex >= 0xF000 {
                                scrbuf.fill_rect(ix, iy, scl, scl, 28);
                            }
                            scrbuf.fill_rect(ix + 2, iy + 2, 7, 7, 0);
                            scrbuf.fill_rect(ix + 3, iy + 3, 5, 5, (thng & 0xFF) as u8);
                        }
                    }
                    // TODO temporary - info about selected cell
                    if ix <= 200 && iy <= 200 && 200 < (ix + scl) && 200 < (iy + scl) {
                        tmp_x = xx;
                        tmp_y = yy;
                        scrbuf.fill_rect(ix, iy, 1, scl, 255);
                        scrbuf.fill_rect(ix + scl - 1, iy, 1, scl, 255);
                        scrbuf.fill_rect(ix, iy, scl, 1, 255);
                        scrbuf.fill_rect(ix, iy + scl - 1, scl, 1, 255);
                    }
                }
            }
        }

        // paint messages
        scrbuf.fill_rect(0, 0, sw, 12, 28);
        let description = map.get_description();
        self.assets.font1.draw_text(6, 1, description, 15, scrbuf);
        let secrets = map.get_secrets_msg();
        let scw = self.assets.font1.text_width(&secrets) + 7;
        self.assets.font1.draw_text(sw - scw, 1, &secrets, 14, scrbuf);

        // TODO temporary show info about selected cell
        if let Some(cell) = map.cell(tmp_x, tmp_y) {
            let str = format!(
                "AT ({tmp_x},{tmp_y}) => tile={}, thing={}, tex={}, spr={}",
                cell.tile,
                cell.thing,
                cell.get_texture(),
                cell.get_sprite()
            );
            scrbuf.fill_rect(0, 12, sw, 12, 31);
            self.assets.font1.draw_text(4, 14, &str, 15, scrbuf);
        }
    }
}
