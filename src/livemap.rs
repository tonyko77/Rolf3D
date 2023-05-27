//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

use crate::*;
use sdl2::keyboard::Keycode;
use std::{f64::consts::PI, rc::Rc};

const PI2: f64 = PI * 2.0;
const HALF_PI: f64 = PI / 2.0;
const EPSILON: f64 = 0.001;

// TODO tune these !!
const MOVE_SPEED: f64 = 4.0;
const ROTATE_SPEED: f64 = 1.6;
const WALL_HEIGHT_SCALER: f64 = 0.9;
//const MIN_DISTANCE_TO_WALL: f64 = 0.25;

/// The "live" map, whre the player moves, actor act, things are "live" etc.
/// Can also render the 3D view.
pub struct LiveMap {
    assets: Rc<GameAssets>,
    cells: Vec<MapCell>,
    player: Actor,
    _actors: Vec<Actor>,
    width: u16,
    height: u16,
    details: MapDetails,
    // TODO remove these when no longer needed
    _tmp_idx: usize,
    _tmp_timer: f64,
}

impl LiveMap {
    pub fn new(assets: Rc<GameAssets>, index: usize, mapsrc: &MapData) -> Self {
        let width = mapsrc.width;
        let height = mapsrc.height;
        let (cells, player, actors) = maploader::load_map_to_cells(mapsrc);
        let details = MapDetails::new(index, mapsrc);

        // TODO: compute tile flags, extract doors, live things, AMBUSH tiles, count enemies/treasures/secrets
        // -> see WOLF3D sources - e.g. https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
        Self {
            assets,
            cells,
            player,
            _actors: actors,
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

        // update player
        let player_angle = self.player.angle;
        if inputs.key(Keycode::W) || inputs.key(Keycode::Up) {
            translate_actor(&mut self.player, elapsed_time, player_angle);
        } else if inputs.key(Keycode::S) || inputs.key(Keycode::Down) {
            translate_actor(&mut self.player, -elapsed_time, player_angle);
        }

        if inputs.key(Keycode::A) {
            translate_actor(&mut self.player, elapsed_time, player_angle - HALF_PI);
        } else if inputs.key(Keycode::D) {
            translate_actor(&mut self.player, elapsed_time, player_angle + HALF_PI);
        }

        if inputs.key(Keycode::Left) {
            rotate_actor(&mut self.player, -elapsed_time);
        } else if inputs.key(Keycode::Right) {
            rotate_actor(&mut self.player, elapsed_time);
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

        let halfh = scrbuf.get_vert_center();
        let width = scrbuf.scr_width();

        // paint sky and floor first
        scrbuf.fill_rect(0, 0, width, halfh, SKY_COLOR);
        scrbuf.fill_rect(0, halfh, width, halfh, FLOOR_COLOR);

        // cast rays to draw the walls
        let pa = self.player.angle;
        for x in 0..width {
            let angle = scrbuf.screen_x_to_angle(x);
            let (dist, texidx, texrelofs) = self.cast_one_ray(angle + pa);
            // rectify ray distance, to avoid fish-eye distortion
            let dist = dist * angle.cos();
            if dist >= EPSILON {
                // adjust outputs
                let texture = &self.assets.walls[texidx];
                let height_scale = WALL_HEIGHT_SCALER / dist;
                texture.render_column(texrelofs, height_scale, x, scrbuf);
            }
        }

        // TODO temporary paint gfx
        let x0 = width - 80;
        let y0 = (scrbuf.scr_height() - 80) as i32;
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

    /// Cast one ray into the world, at an angle.
    /// Takes into account the texturing and activity (e.g. door opening) of the hit wall/door.
    /// Returns: (ray length, texture index, texture's relative x position(0..1)).
    /// Thanks to [javidx9 a.k.a. olc](https://www.youtube.com/watch?v=NbSee-XM7WA)
    fn cast_one_ray(&self, angle: f64) -> (f64, usize, f64) {
        let (sin, cos) = angle.sin_cos();

        let map_w = self.width as i32;
        let map_h = self.height as i32;
        let plx = self.player.x;
        let ply = self.player.y;
        let plx_fl = plx.floor();
        let ply_fl = ply.floor();
        let mut map_x = plx_fl as i32;
        let mut map_y = ply_fl as i32;
        let mut map_idx = map_y * map_w + map_x;

        let (mut dist_x, scale_x, dir_x, orient_x) = if cos > EPSILON {
            // looking RIGHT
            let d = plx_fl + 1.0 - plx;
            (d / cos, 1.0 / cos, 1, Orientation::West)
        } else if cos < -EPSILON {
            // looking LEFT
            let d = plx_fl - plx;
            (d / cos, -1.0 / cos, -1, Orientation::East)
        } else {
            // straight vertical => no hits on the X axis
            (f64::MAX, 0.0, 0, Orientation::North)
        };

        let (mut dist_y, scale_y, dir_y, orient_y) = if sin > EPSILON {
            // looking DOWN (map is y-flipped)
            let d = ply_fl + 1.0 - ply;
            (d / sin, 1.0 / sin, 1, Orientation::North)
        } else if sin < -EPSILON {
            // looking UP (map is y-flipped)
            let d = ply_fl - ply;
            (d / sin, -1.0 / sin, -1, Orientation::South)
        } else {
            // straight horizontal => no hits on the Y axis
            (f64::MAX, 0.0, 0, Orientation::West)
        };

        // TODO adjustments for doors and pushed walls !!!
        // TODO then, draw door edges !

        // find a hit on the X or Y axis and get the texture index
        let out_of_bounds = self.cells.len() as i32;
        let tex = loop {
            if dist_x < dist_y {
                // moving on the X axis
                map_x += dir_x;
                map_idx += dir_x;
                if map_x < 0 || map_x >= map_w || map_idx < 0 || map_idx >= out_of_bounds {
                    break 0;
                }
                let cell = &self.cells[map_idx as usize];
                if cell.is_solid_textured() {
                    // got a hit
                    let tex = cell.texture(orient_x);
                    break tex;
                }
                // continue on the X axis
                dist_x += scale_x;
            } else {
                // moving on the Y axis
                map_y += dir_y;
                map_idx += dir_y * map_w;
                if map_y < 0 || map_y >= map_h || map_idx < 0 || map_idx >= out_of_bounds {
                    break 0;
                }
                let cell = &self.cells[map_idx as usize];
                if cell.is_solid_textured() {
                    // got a hit
                    let tex = cell.texture(orient_y);
                    break tex;
                }
                // continue on the Y axis
                dist_y += scale_y;
            }
        };

        // find the distance and texture relative position
        if dist_x < dist_y {
            // the hit was on a vertical wall
            let y_spot = ply + dist_x * sin;
            let relofs = y_spot - y_spot.floor();
            let okofs = if dir_x > 0 { relofs } else { 1.0 - relofs };
            (dist_x, tex, okofs)
        } else {
            // the hit was on a horizontal wall
            let x_spot = plx + dist_y * cos;
            let relofs = x_spot - x_spot.floor();
            let okofs = if dir_y < 0 { relofs } else { 1.0 - relofs };
            (dist_y, tex, okofs)
        }
    }
}

//--------------------
// Internal stuff
//--------------------

struct MapDetails {
    descr_msg: String,
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

#[inline]
fn translate_actor(actor: &mut Actor, ellapsed_time: f64, angle: f64) {
    let (dx, dy) = float_polar_translate(ellapsed_time * MOVE_SPEED, angle);
    actor.x += dx;
    actor.y += dy;
}

#[inline]
fn rotate_actor(actor: &mut Actor, ellapsed_time: f64) {
    actor.angle += ellapsed_time * ROTATE_SPEED;
    if actor.angle >= PI2 {
        actor.angle -= PI2;
    } else if actor.angle < 0.0 {
        actor.angle += PI2;
    }
}

#[inline]
fn float_polar_translate(distance: f64, angle: f64) -> (f64, f64) {
    let (s, c) = angle.sin_cos();
    (distance * c, distance * s)
}

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
