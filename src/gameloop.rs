//! Main game loop.
//! Also acts as a facade, to hold and manage all game objects
//! (assets, renderers, other managers etc)

use crate::defs::*;
use crate::*;
use sdl2::mouse::MouseButton;
use sdl2::{event::Event, keyboard::Keycode};
use std::collections::HashSet;
use std::rc::Rc;

pub struct GameLoop {
    scrbuf: ScreenBuffer,
    assets: Rc<GameAssets>,
    inputs: InputManager,
    automap: AutomapRenderer,
    tmp_idx: usize,
    tmp_automap: bool,
}

impl GameLoop {
    pub fn new(width: usize, height: usize, pixel_size: i32, assets: GameAssets) -> Self {
        let is_sod = assets.is_sod();
        let ga = Rc::from(assets);
        Self {
            scrbuf: ScreenBuffer::new(width, height, is_sod),
            assets: Rc::clone(&ga),
            inputs: InputManager::new(pixel_size),
            automap: AutomapRenderer::new(Rc::clone(&ga)),
            tmp_idx: 0,
            tmp_automap: false,
        }
    }
}

impl GraphicsLoop for GameLoop {
    fn handle_event(&mut self, event: &Event) -> bool {
        self.inputs.handle_event(event)
    }

    fn update_state(&mut self, elapsed_time: f64) -> bool {
        // check keys
        if self.inputs.consume_key(Keycode::Tab) {
            self.tmp_automap = !self.tmp_automap;
        }
        if self.inputs.consume_key(Keycode::PageUp) {
            self.tmp_idx = (self.tmp_idx + 999) % 1000;
            self.automap.reset_map(self.tmp_idx % self.assets.maps.len());
        } else if self.inputs.consume_key(Keycode::PageDown) {
            self.tmp_idx = (self.tmp_idx + 1) % 1000;
            self.automap.reset_map(self.tmp_idx % self.assets.maps.len());
        }
        if self.inputs.consume_key(Keycode::Home) {
            self.tmp_idx = 0;
            self.automap.reset_map(0);
        }

        // TODO update game state
        if self.tmp_automap {
            self.automap.handle_inputs(&mut self.inputs, elapsed_time);
            self.automap.paint(&mut self.scrbuf);
            //_temp_paint_map(self);
        } else {
            _temp_paint_gfx(self);
        }

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

//----------------------
//  Internal stuff

// TODO temporary paint gfx
fn _temp_paint_gfx(zelf: &mut GameLoop) {
    _temp_paint_palette(&mut zelf.scrbuf);

    let x0 = (zelf.scrbuf.width() - 100) as i32;
    let y0 = (zelf.scrbuf.height() - 202) as i32;

    // paint wall
    let wallidx = zelf.tmp_idx % zelf.assets.walls.len();
    let wall = &zelf.assets.walls[wallidx];
    _temp_paint_pic(wall, x0, 10, &mut zelf.scrbuf);
    let str = format!("WALL #{wallidx}");
    zelf.assets.font1.draw_text(x0, 80, &str, 14, &mut zelf.scrbuf);

    // paint sprite
    let sprtidx = zelf.tmp_idx % zelf.assets.sprites.len();
    let sprite = &zelf.assets.sprites[sprtidx];
    _temp_paint_pic(sprite, x0, y0, &mut zelf.scrbuf);
    let str = format!("SPRT #{sprtidx}");
    zelf.assets.font1.draw_text(x0, y0 - 16, &str, 14, &mut zelf.scrbuf);

    // paint pics
    let picidx = zelf.tmp_idx % zelf.assets.pics.len();
    let pic = &zelf.assets.pics[picidx];
    _temp_paint_pic(pic, 0, y0, &mut zelf.scrbuf);
    let str = format!("PIC #{picidx}");
    zelf.assets.font1.draw_text(0, y0 - 16, &str, 14, &mut zelf.scrbuf);

    // paint fonts
    let char_idx = zelf.tmp_idx % 100;
    let ch = (char_idx + 33) as u8;
    let str = format!("{} = {}", ch as char, ch);
    zelf.assets.font1.draw_text(170, 10, &str, 11, &mut zelf.scrbuf);
    zelf.assets.font2.draw_text(170, 30, &str, 12, &mut zelf.scrbuf);
}

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

    let sw = scrbuf.width() as i32;
    let sh = scrbuf.height() as i32;
    scrbuf.fill_rect(0, 0, sw, sh, 0);

    // paint the palette
    let mut cidx: i32 = 0;
    for y in 0..16 {
        for x in 0..16 {
            let c = cidx as u8;
            scrbuf.fill_rect(x * SQSIZE, y * SQSIZE, SQSIZE, SQSIZE, c);
            cidx += 1;
        }
    }
}

// TODO TEMP get info about map
fn _temp_map_debug_info(zelf: &GameLoop) {
    let mapidx = zelf.tmp_idx % zelf.assets.maps.len();
    let map = &zelf.assets.maps[mapidx];

    // check for tiles >= AREATILE
    let mut minwall = 9999;
    let mut maxwall = 0;
    let mut non_wall = HashSet::new();
    for x in 0..64 {
        for y in 0..64 {
            let tile = map.tile(x, y);
            if tile < AREATILE {
                // solid wall go from 1 to 106 (AREATILE - 1)
                minwall = Ord::min(minwall, tile);
                maxwall = Ord::max(maxwall, tile);

                // check for missing textures
                let widx = (tile * 2 - 2) as usize;
                if widx >= zelf.assets.walls.len() {
                    if tile >= 90 && tile <= 101 {
                        // it's a door, it is ok
                    } else if tile == AMBUSHTILE {
                        // it's an ambush tile
                    } else {
                        println!(
                            "MISSING wall texture for tile {tile} => widx={widx} >= {}",
                            zelf.assets.walls.len()
                        );
                    }
                }
            } else {
                // seem to be between 108 (AREATILE + 1) and ~143
                // but what do they mean ????
                non_wall.insert(tile);
            }
        }
    }
}
