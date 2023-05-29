//! Contains the ray casting algorithm, isolated and tuned for my implementation.

use crate::{Actor, MapCell, EPSILON};

const FAR_AWAY: f64 = 999.0;
const TEXIDX_DOOR_EDGES: usize = 100;

pub struct RayCaster {
    map_width: i32,
    map_height: i32,
    player_x: f64,
    player_y: f64,
    sin: f64,
    cos: f64,
    map_x: i32,
    map_y: i32,
    map_idx: i32,
    dist_x: f64,
    dist_y: f64,
    scale_x: f64,
    scale_y: f64,
    dir_x: i32,
    dir_y: i32,
    texture_idx: Option<usize>,
}

impl RayCaster {
    /// Set up the ray caster, for casting multiple rays (at different angles) from the same origin.
    pub fn new(player: &Actor, map_width: i32, map_height: i32) -> Self {
        // set the ray starting point a bit behind the player
        // (the rays are supposed to come from behind the screen and intersect it)
        const MAGIC_VIEW_DISTANCE: f64 = -0.375;
        let (player_x, player_y) = translate_point(player.x, player.y, player.angle, MAGIC_VIEW_DISTANCE);

        Self {
            map_width,
            map_height,
            player_x,
            player_y,
            sin: 0.0,
            cos: 0.0,
            map_x: 0,
            map_y: 0,
            map_idx: 0,
            dist_x: 0.0,
            dist_y: 0.0,
            scale_x: 0.0,
            scale_y: 0.0,
            dir_x: 0,
            dir_y: 0,
            texture_idx: None,
        }
    }

    // TODO adjustments for doors and pushed walls !!!
    // TODO then, draw door edges !

    pub fn cast_ray(&mut self, angle: f64, cells: &[MapCell]) -> (f64, usize, f64) {
        self.prepare(angle);

        // keep advancing both rays, until one hits something
        let mut max_steps = Ord::max(self.map_width, self.map_height) << 1;
        while self.texture_idx.is_none() && (self.dist_x < FAR_AWAY || self.dist_y < FAR_AWAY) && max_steps > 0 {
            // check if coming from a door cell
            // (used for painting the door edges correctly)
            let mut from_door_cell = false;
            if self.map_idx >= 0 && (self.map_idx as usize) < cells.len() {
                from_door_cell = cells[self.map_idx as usize].is_door();
            }
            // check and advance the shorter of the 2 rays
            if self.dist_x < self.dist_y {
                self.advance_x_ray(cells, from_door_cell);
            } else {
                self.advance_y_ray(cells, from_door_cell);
            }
            // use a step counter, to avoid overflows when no-clipped out of bounds
            max_steps -= 1;
        }

        // if we got out of bounds => just paint something very far away, from any texture
        if self.texture_idx.is_none() {
            return (FAR_AWAY, 0, 0.0);
        }

        // find the texture relative position and return data
        let texidx = self.texture_idx.unwrap_or(0);
        if self.dist_x < self.dist_y {
            // the hit was on a vertical wall
            let y_spot = self.player_y + self.dist_x * self.sin;
            let texrelofs = if self.dir_x > 0 {
                y_spot - y_spot.floor()
            } else {
                y_spot.floor() + 1.0 - y_spot
            };

            (self.dist_x, texidx, texrelofs)
        } else {
            // the hit was on a horizontal wall
            let x_spot = self.player_x + self.dist_y * self.cos;
            let texrelofs = if self.dir_y < 0 {
                x_spot - x_spot.floor()
            } else {
                x_spot.floor() + 1.0 - x_spot
            };

            (self.dist_y, texidx, texrelofs)
        }
    }

    //----------------

    fn prepare(&mut self, angle: f64) {
        let plx_fl = self.player_x.floor();
        let ply_fl = self.player_y.floor();

        (self.sin, self.cos) = angle.sin_cos();
        self.map_x = plx_fl as i32;
        self.map_y = ply_fl as i32;
        self.map_idx = self.map_y * self.map_width + self.map_x;
        self.texture_idx = None;

        // compute direction, scale and initial distance along X
        (self.dist_x, self.scale_x, self.dir_x) = if self.cos > EPSILON {
            // looking RIGHT => this ray will hit the WEST face of a wall
            let d = plx_fl + 1.0 - self.player_x;
            (d / self.cos, 1.0 / self.cos, 1)
        } else if self.cos < -EPSILON {
            // looking LEFT => this ray will hit the EAST face of a wall
            let d = plx_fl - self.player_x;
            (d / self.cos, -1.0 / self.cos, -1)
        } else {
            // straight vertical => no hits along the X axis
            (FAR_AWAY, 0.0, 0)
        };

        // compute direction, scale and initial distance along Y
        (self.dist_y, self.scale_y, self.dir_y) = if self.sin > EPSILON {
            // looking DOWN (map is y-flipped) => will hit NORTH
            let d = ply_fl + 1.0 - self.player_y;
            (d / self.sin, 1.0 / self.sin, 1)
        } else if self.sin < -EPSILON {
            // looking UP (map is y-flipped) => will hit SOUTH
            let d = ply_fl - self.player_y;
            (d / self.sin, -1.0 / self.sin, -1)
        } else {
            // straight horizontal => no hits along the Y axis
            (FAR_AWAY, 0.0, 0)
        };
    }

    fn advance_x_ray(&mut self, cells: &[MapCell], from_door_cell: bool) {
        // moving on the X axis
        self.map_x += self.dir_x;
        self.map_idx += self.dir_x;

        // check if we hit a "solid" cell (wall or door)
        let mut got_hit = false;
        if self.map_x >= 0 && self.map_x < self.map_width && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                got_hit = self.check_hit_x_ray(cell, from_door_cell);
            }
        }

        if !got_hit {
            // if not hit, continue along the X axis
            self.dist_x += self.scale_x;
        }
    }

    fn check_hit_x_ray(&mut self, cell: &MapCell, from_door_cell: bool) -> bool {
        if cell.is_wall() {
            // the ray hit a wall OR a door's edge
            let tex = if from_door_cell {
                TEXIDX_DOOR_EDGES
            } else {
                cell.get_texture() + 1 // use the darker texture for E/W walls
            };
            self.texture_idx = Some(tex);
            return true;
        }
        if cell.is_vert_door() {
            // we either hit the door or its edges
            let dist_to_door = self.dist_x + self.scale_x * 0.5;
            if dist_to_door <= self.dist_y {
                // we hit the door
                // TODO (later) take into account if the door is open/opening/closing
                self.dist_x = dist_to_door;
                self.dir_x = 1;
                self.texture_idx = Some(cell.get_texture());
                return true;
            }
        }
        false
    }

    fn advance_y_ray(&mut self, cells: &[MapCell], from_door_cell: bool) {
        // advance on the Y axis
        self.map_y += self.dir_y;
        self.map_idx += self.dir_y * self.map_width;

        // check if we hit a "solid" cell (wall or door)
        let mut got_hit = false;
        if self.map_y >= 0 && self.map_y < self.map_height && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                got_hit = self.check_hit_y_ray(cell, from_door_cell);
            }
        }

        if !got_hit {
            // if not hit, continue along the Y axis
            self.dist_y += self.scale_y;
        }
    }

    fn check_hit_y_ray(&mut self, cell: &MapCell, from_door_cell: bool) -> bool {
        if cell.is_wall() {
            // the ray hit a wall OR a door's edge
            let tex = if from_door_cell {
                TEXIDX_DOOR_EDGES
            } else {
                cell.get_texture()
            };
            self.texture_idx = Some(tex);
            return true;
        }
        if cell.is_horiz_door() {
            // we either hit the door or its edges
            let dist_to_door = self.dist_y + self.scale_y * 0.5;
            if dist_to_door <= self.dist_x {
                // we hit the door
                // TODO (later) take into account if the door is open/opening/closing
                self.dist_y = dist_to_door;
                self.dir_y = -1;
                self.texture_idx = Some(cell.get_texture());
                return true;
            }
        }
        false
    }
}

//--------------------------
// Internal stuff

#[inline]
fn translate_point(x: f64, y: f64, angle: f64, dist: f64) -> (f64, f64) {
    let (sin, cos) = angle.sin_cos();
    let x2 = x + dist * cos;
    let y2 = y + dist * sin;
    (x2, y2)
}
