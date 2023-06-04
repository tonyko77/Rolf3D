//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

use crate::{raycaster::RayCaster, *};
use sdl2::keyboard::Keycode;
use std::{f64::consts::PI, rc::Rc};

const MOVE_SPEED: f64 = 4.5;
const ROTATE_SPEED: f64 = 2.0;
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
    description: String,
    episode: u8,
    floor: u8,
    assets: Rc<GameAssets>,
    cells: Vec<MapCell>,
    actors: Vec<Actor>,
    width: u16,
    height: u16,
    status: GameStatus,
    clipping_enabled: bool,
    secret_floor_return: u8,
    player_map_x: i32,
    player_map_y: i32,
}

impl LiveMap {
    pub fn new(assets: Rc<GameAssets>, episode: u8) -> Self {
        let mut livemap = Self {
            description: String::new(),
            episode,
            floor: 0,
            assets,
            cells: vec![],
            actors: vec![],
            width: 0,
            height: 0,
            status: GameStatus::new(0),
            clipping_enabled: true,
            secret_floor_return: 0,
            player_map_x: -1,
            player_map_y: -1,
        };
        livemap.floor_has_changed();
        livemap
    }

    pub fn go_to_next_floor(&mut self) {
        if self.floor >= 9 {
            self.floor = self.secret_floor_return;
        } else {
            self.floor += 1;
        }
        self.floor_has_changed();
    }

    pub fn go_to_secret_floor(&mut self) {
        assert!(self.floor < 8);
        self.secret_floor_return = self.floor + 1;
        self.floor = 9;
        self.floor_has_changed();
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
    pub fn cell(&self, x: i32, y: i32) -> Option<&MapCell> {
        if let Some(idx) = self.cell_index(x, y) {
            self.cells.get(idx)
        } else {
            None
        }
    }

    // TODO the return of next game state is kinda hacky => FIX IT !!
    pub fn handle_inputs(&mut self, inputs: &mut InputManager, elapsed_time: f64) {
        // TODO: update doors, secret walls, actors - only if NOT paused

        // weapons
        if inputs.consume_key(Keycode::Num1) {
            self.status.try_select_weapon(0);
        } else if inputs.consume_key(Keycode::Num2) {
            self.status.try_select_weapon(1);
        } else if inputs.consume_key(Keycode::Num3) {
            self.status.try_select_weapon(2);
        } else if inputs.consume_key(Keycode::Num4) {
            self.status.try_select_weapon(3);
        }

        if inputs.consume_key(Keycode::E) || inputs.consume_key(Keycode::Space) {
            self.perform_use();
        }

        // TODO temporary - "fake" shooting
        // (later, do NOT consume key - machine gun and chain gun are automatic)
        if inputs.consume_key(Keycode::LCtrl) || inputs.consume_key(Keycode::RCtrl) {
            if self.status.get_selected_weapon() != 0 {
                self.status.consume_ammo();
            }
        }

        // update player
        let player_angle = self.actors[0].angle;
        if inputs.key(Keycode::W) || inputs.key(Keycode::Up) {
            self.translate_actor(0, elapsed_time, player_angle);
        } else if inputs.key(Keycode::S) || inputs.key(Keycode::Down) {
            self.translate_actor(0, -elapsed_time, player_angle);
        }

        if inputs.key(Keycode::A) {
            self.translate_actor(0, elapsed_time, player_angle - HALF_PI);
        } else if inputs.key(Keycode::D) {
            self.translate_actor(0, elapsed_time, player_angle + HALF_PI);
        }

        if inputs.key(Keycode::Left) {
            self.rotate_actor(0, -elapsed_time);
        } else if inputs.key(Keycode::Right) {
            self.rotate_actor(0, elapsed_time);
        }

        // update doors and push walls
        // TODO keep wall indexes in an internal map of indexes
        for cell in self.cells.iter_mut() {
            cell.update_state(elapsed_time);
        }

        // update player
        self.update_player();

        // TODO update actors ...

        // TODO: temporary keys
        if inputs.consume_key(Keycode::F1) {
            self.clipping_enabled = !self.clipping_enabled;
        }
        if inputs.consume_key(Keycode::F2) {
            self.status._tmp_give_stuff();
        }
        if inputs.consume_key(Keycode::F3) {
            self.status.damage_health(10);
        }
    }

    #[inline]
    pub fn get_description(&self) -> &str {
        &self.description
    }

    #[inline]
    pub fn get_secrets_msg(&self) -> String {
        self.status.get_secrets_msg()
    }

    pub fn paint_3d(&self, scrbuf: &mut ScreenBuffer) {
        // TODO (later) use correct sky color per game and level
        // (0x1D, 0xBF, 0x4E and 0x8D)
        const SKY_COLOR: u8 = 0x1D;
        const FLOOR_COLOR: u8 = 0x19;

        let halfh = scrbuf.view_height() / 2;
        let width = scrbuf.scr_width();

        // paint sky and floor first
        scrbuf.fill_rect(0, 0, width, halfh, SKY_COLOR);
        scrbuf.fill_rect(0, halfh, width, halfh, FLOOR_COLOR);

        // cast rays to draw the walls
        let pa = self.actors[0].angle;
        let mut ray_caster = RayCaster::new(&self.actors[0], self.width as i32, self.height as i32);
        for x in 0..width {
            let angle = scrbuf.screen_x_to_angle(x);
            let (dist, texidx, texrelofs) = ray_caster.cast_ray(angle + pa, &self.cells);
            // rectify ray distance, to avoid fish-eye distortion
            let dist = dist * angle.cos();
            if dist >= 0.004 {
                // adjust outputs
                let texture = &self.assets.walls[texidx];
                scrbuf.render_texture_column(x, dist, texrelofs, texture);
            }
        }

        // paint the sprites
        let visited_cells = ray_caster.into_visited_cells();
        for tc in visited_cells.into_iter() {
            let cell = &self.cells[tc.idx];
            let spridx = cell.get_sprite() as usize;
            if spridx < self.assets.sprites.len() {
                let sprite = &self.assets.sprites[spridx];
                scrbuf.render_sprite(tc.angle, tc.dist, sprite);
            }
        }

        // TODO temporary debug info
        _temp_debug_info(self, scrbuf);
    }

    #[inline]
    pub fn paint_status_bar(&self, scrbuf: &mut ScreenBuffer) {
        self.status.paint_status_bar(scrbuf, &self.assets);
    }

    //----------------

    // TODO: compute tile flags, extract doors, live things, AMBUSH tiles, count enemies/treasures/secrets
    // -> see WOLF3D sources - e.g. https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
    fn floor_has_changed(&mut self) {
        let idx = (self.episode as usize) * 10 + (self.floor as usize);
        let mapsrc = &self.assets.maps[idx];
        let floor_name = match self.floor {
            9 => "Secret floor".to_string(),
            8 => "Final floor".to_string(),
            _ => format!("floor {}", self.floor + 1),
        };
        self.description = format!("{} - ep. {}, {}", mapsrc.name, self.episode + 1, floor_name);
        // load map
        self.width = mapsrc.width;
        self.height = mapsrc.height;
        (self.cells, self.actors) = mapcell::load_map_to_cells(mapsrc, self.assets.is_sod);
        // update status
        self.status.set_floor(self.floor as i32, (self.actors.len() - 1) as i32);
        self.cells.iter().for_each(|cell| self.status.read_floor_cell(cell));
        self.player_map_x = -1;
        self.player_map_y = -1;
    }

    fn update_player(&mut self) {
        let new_x = self.actors[0].x as i32;
        let new_y = self.actors[0].y as i32;
        if new_x != self.player_map_x || new_y != self.player_map_y {
            if new_x >= 0 && new_y >= 0 && new_x < (self.width as i32) && new_y < (self.height as i32) {
                self.player_map_x = new_x;
                self.player_map_y = new_y;
                let idx = (new_y * (self.width as i32) + new_x) as usize;
                let consumable = self.cells[idx].collectible();
                if self.status.try_consume(consumable) {
                    self.cells[idx].remove_collectible();
                }
            }
        }
    }

    // Open door, push wall, trigger elevator
    fn perform_use(&mut self) {
        // figure out the cell to be "used"
        let pa = self.actors[0].angle;
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
        let cx = (self.actors[0].x as i32) + dx;
        let cy = (self.actors[0].y as i32) + dy;
        if let Some(cell_idx) = self.cell_index(cx, cy) {
            let mut idx = cell_idx as i32;
            let idx_delta = dx + dy * (self.width as i32);
            // only trigger wall pushing if it can be pushed further
            let can_push_wall =
                self.cells[cell_idx].is_push_wall() && self.cells[(idx + idx_delta) as usize].can_push_wall_into();
            if can_push_wall {
                // push walls need special handling - multiple cells need to be set up
                let actor_area = self.cells[(idx - idx_delta) as usize].get_area();
                let wall_texture = self.cells[cell_idx].get_texture() as u16;
                let mut progress = 1.0;
                while self.cells[(idx + idx_delta) as usize].can_push_wall_into() {
                    self.cells[idx as usize].start_push_wall(actor_area, wall_texture, progress);
                    progress += 1.0;
                    idx += idx_delta;
                }
                self.cells[idx as usize].end_push_wall(wall_texture);
                // also increase the secret count !!
                self.status.found_secret();
            } else {
                //TODO check if I have the key
                let door_key = self.cells[cell_idx].get_door_key_type();
                if self.status.has_key(door_key) {
                    self.cells[cell_idx].activate_door_or_elevator(dx, dy);
                }
            }
        }
    }

    #[inline]
    fn translate_actor(&mut self, actor_idx: usize, ellapsed_time: f64, angle: f64) {
        let distance = ellapsed_time * MOVE_SPEED;

        // transform the polar (distance, angle) into movements along the 2 axis
        let (s, c) = angle.sin_cos();
        let delta_x = distance * c;
        let delta_y = distance * s;

        let old_x = self.actors[actor_idx].x;
        let old_y = self.actors[actor_idx].y;
        let mut upd_x = old_x + delta_x;
        let mut upd_y = old_y + delta_y;

        // remove the actor first (otherwise, the bounds check below breaks)
        if let Some(cell) = self.cell_mut(old_x as i32, old_y as i32) {
            cell.actor_left();
        }

        // check for bounds
        if self.clipping_enabled {
            let ix = upd_x as i32;
            let iy = upd_y as i32;
            let fwd_x = (upd_x + MIN_DISTANCE_TO_WALL * delta_x.signum()) as i32;
            let fwd_y = (upd_y + MIN_DISTANCE_TO_WALL * delta_y.signum()) as i32;
            // check for collisions on each axis
            let mut no_collision = true;
            if self.cell_is_solid(fwd_x, iy) {
                upd_x = old_x;
                no_collision = false;
            }
            if self.cell_is_solid(ix, fwd_y) {
                upd_y = old_y;
                no_collision = false;
            }
            // check for corner collision
            if no_collision && self.cell_is_solid(fwd_x, fwd_y) {
                // cancel the smaller movement, to get some wall sliding
                if delta_x.abs() < delta_y.abs() {
                    upd_x = old_x;
                } else {
                    upd_y = old_y;
                }
            }
        }

        // update the actor
        if let Some(cell) = self.cell_mut(upd_x as i32, upd_y as i32) {
            cell.actor_entered();
        }
        self.actors[actor_idx].x = upd_x;
        self.actors[actor_idx].y = upd_y;
    }

    #[inline]
    fn rotate_actor(&mut self, actor_idx: usize, ellapsed_time: f64) {
        let actor = self.actors.get_mut(actor_idx).unwrap();
        actor.angle += ellapsed_time * ROTATE_SPEED;
        if actor.angle >= PI2 {
            actor.angle -= PI2;
        } else if actor.angle < 0.0 {
            actor.angle += PI2;
        }
    }

    #[inline]
    fn cell_index(&self, x: i32, y: i32) -> Option<usize> {
        let w = self.width as i32;
        let h = self.height as i32;
        if x >= 0 && x < w && y >= 0 && y < h {
            Some((y * w + x) as usize)
        } else {
            None
        }
    }

    #[inline]
    fn cell_mut(&mut self, x: i32, y: i32) -> Option<&mut MapCell> {
        if let Some(idx) = self.cell_index(x, y) {
            self.cells.get_mut(idx)
        } else {
            None
        }
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

// TODO temporary show some debug info
fn _temp_debug_info(zelf: &LiveMap, scrbuf: &mut ScreenBuffer) {
    let noclip = if zelf.clipping_enabled { "off" } else { "ON" };
    let str = format!(
        "X={:.2}  Y={:.2}  Angle={:.2})  NoClip={noclip}",
        zelf.actors[0].x,
        zelf.actors[0].y,
        zelf.actors[0].angle * 180.0 / PI
    );

    let y = scrbuf.scr_height() - 16;
    zelf.assets.font1.draw_text(5, y, &str, 15, scrbuf);
}
