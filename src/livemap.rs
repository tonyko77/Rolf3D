//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

use crate::{raycaster::RayCaster, *};
use sdl2::keyboard::Keycode;
use std::{f64::consts::PI, rc::Rc};

const MOVE_SPEED: f64 = 4.5;
const ROTATE_SPEED: f64 = 2.0;
const WALL_HEIGHT_SCALER: f64 = 1.1;
// Minimum distance between the player and a wall
// (or, it can be considered the "diameter" of the player object in the world)
const MIN_DISTANCE_TO_WALL: f64 = 0.375;

const PI2: f64 = PI * 2.0;
const HALF_PI: f64 = PI / 2.0;
const PI_1_4: f64 = PI / 4.0;
const PI_3_4: f64 = PI * 3.0 / 4.0;
const PI_5_4: f64 = PI * 5.0 / 4.0;
const PI_7_4: f64 = PI * 7.0 / 4.0;

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
    _tmp_clip: bool,
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
            _tmp_clip: true,
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

    #[inline]
    pub fn cell_index(&self, x: i32, y: i32) -> usize {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            (y * w + x) as usize
        } else {
            0xFFFF_FFFF
        }
    }

    #[inline]
    pub fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut MapCell> {
        let idx = self.cell_index(x, y);
        if idx < self.cells.len() {
            self.cells.get_mut(idx)
        } else {
            None
        }
    }

    #[inline]
    pub fn cell(&self, x: i32, y: i32) -> Option<&MapCell> {
        let idx = self.cell_index(x, y);
        if idx < self.cells.len() {
            self.cells.get(idx)
        } else {
            None
        }
    }

    // TODO the return of next game state is kinda hacky => FIX IT
    pub fn handle_inputs(&mut self, inputs: &mut InputManager, elapsed_time: f64) -> Option<GameState> {
        // TODO: update doors, secret walls, actors - only if NOT paused

        if inputs.consume_key(Keycode::Tab) {
            return Some(GameState::Automap);
        }

        if inputs.consume_key(Keycode::E) || inputs.consume_key(Keycode::Space) {
            self.perform_use();
            return None;
        }
        // update player
        let player_angle = self.player.angle;
        if inputs.key(Keycode::W) || inputs.key(Keycode::Up) {
            (self.player.x, self.player.y) = self.translate_actor(&self.player, elapsed_time, player_angle);
        } else if inputs.key(Keycode::S) || inputs.key(Keycode::Down) {
            (self.player.x, self.player.y) = self.translate_actor(&self.player, -elapsed_time, player_angle);
        }

        if inputs.key(Keycode::A) {
            (self.player.x, self.player.y) = self.translate_actor(&self.player, elapsed_time, player_angle - HALF_PI);
        } else if inputs.key(Keycode::D) {
            (self.player.x, self.player.y) = self.translate_actor(&self.player, elapsed_time, player_angle + HALF_PI);
        }

        if inputs.key(Keycode::Left) {
            rotate_actor(&mut self.player, -elapsed_time);
        } else if inputs.key(Keycode::Right) {
            rotate_actor(&mut self.player, elapsed_time);
        }

        // TODO: temporary keys
        if inputs.consume_key(Keycode::F1) {
            self._tmp_clip = !self._tmp_clip;
        }

        // TODO temporary hack, to auto-cycle through graphics
        self._tmp_timer += elapsed_time * 1.8;
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

    // Open door, push wall, trigger elevator
    fn perform_use(&mut self) {
        // figure out the cell to be "used"
        let pa = self.player.angle;
        let (dx, dy) = if pa < PI_1_4 || pa >= PI_7_4 {
            (1, 0) // East
        } else if pa < PI_3_4 {
            (0, 1) // North
        } else if pa < PI_5_4 {
            (-1, 0) // West
        } else {
            (0, -1) // South
        };
        // check the cell
        let cx = (self.player.x as i32) + dx;
        let cy = (self.player.y as i32) + dy;
        if let Some(cell) = self.cell_mut(cx, cy) {
            cell.use_open(dx, dy);
        }
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
        let mut ray_caster = RayCaster::new(&self.player, self.width as i32, self.height as i32);
        let mut x_dists = Vec::with_capacity(width as usize);
        for x in 0..width {
            let angle = scrbuf.screen_x_to_angle(x);
            let (dist, texidx, texrelofs) = ray_caster.cast_ray(angle + pa, &self.cells);
            // remember the distance, for sprite painting
            x_dists.push(dist);
            // rectify ray distance, to avoid fish-eye distortion
            let dist = dist * angle.cos();
            if dist >= 0.004 {
                // adjust outputs
                let texture = &self.assets.walls[texidx];
                let height_scale = WALL_HEIGHT_SCALER / dist;
                texture.render_column(texrelofs, height_scale, x, scrbuf);
            }
        }

        // TODO paint the sprites
        let _visited_cells = ray_caster.into_visited_cells();
        //println!("Visited cells: {}", visited_cells.len());

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

        // TODO show some debug info
        let noclip = if self._tmp_clip { "off" } else { "ON" };
        let str = format!(
            "Player @ ({}, {}, {}), noclip={noclip}",
            self.player.x, self.player.y, self.player.angle
        );
        self.assets.font1.draw_text(0, 0, &str, 15, scrbuf);
    }

    #[inline]
    fn translate_actor(&self, actor: &Actor, ellapsed_time: f64, angle: f64) -> (f64, f64) {
        let distance = ellapsed_time * MOVE_SPEED;
        // transform the polar (distance, angle) into movements along the 2 axis
        let (s, c) = angle.sin_cos();
        let delta_x = distance * c;
        let delta_y = distance * s;

        let mut upd_x = actor.x + delta_x;
        let mut upd_y = actor.y + delta_y;

        // check for bounds
        if self._tmp_clip {
            let ix = upd_x as i32;
            let iy = upd_y as i32;
            let fwd_x = (upd_x + MIN_DISTANCE_TO_WALL * delta_x.signum()) as i32;
            let fwd_y = (upd_y + MIN_DISTANCE_TO_WALL * delta_y.signum()) as i32;
            // check for collisions on each axis
            let mut no_collision = true;
            if self.cell_is_solid(fwd_x, iy) {
                upd_x = actor.x;
                no_collision = false;
            }
            if self.cell_is_solid(ix, fwd_y) {
                upd_y = actor.y;
                no_collision = false;
            }
            // check for corner collision
            if no_collision && self.cell_is_solid(fwd_x, fwd_y) {
                // cancel the smaller movement, to get some wall sliding
                if delta_x.abs() < delta_y.abs() {
                    upd_x = actor.x;
                } else {
                    upd_y = actor.y;
                }
            }
        }
        (upd_x, upd_y)
    }

    #[inline]
    fn cell_is_solid(&self, x: i32, y: i32) -> bool {
        if let Some(cell) = self.cell(x, y) {
            cell.is_solid()
        } else {
            true
        }
    }
}

//--------------------
// Internal stuff
//--------------------

struct MapDetails {
    descr_msg: String,
    total_kills: u16,
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
            total_kills: 0,
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
            self.total_kills,
            self.cnt_treasures,
            self.total_treasures,
            self.cnt_secrets,
            self.total_secrets
        )
    }
}

//-------------------

#[inline]
fn rotate_actor(actor: &mut Actor, ellapsed_time: f64) {
    actor.angle += ellapsed_time * ROTATE_SPEED;
    if actor.angle >= PI2 {
        actor.angle -= PI2;
    } else if actor.angle < 0.0 {
        actor.angle += PI2;
    }
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
