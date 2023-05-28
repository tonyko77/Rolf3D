//! Contains the ray casting algorithm, isolated and tuned for my implementation.

use crate::{MapCell, Orientation, EPSILON};

const FAR_AWAY: f64 = 999.0;

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
    max_steps: i32,
}

impl RayCaster {
    /// Set up the ray caster, for casting multiple rays (at different angles) from the same origin.
    pub fn new(player_x: f64, player_y: f64, map_width: i32, map_height: i32) -> Self {
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
            max_steps: 0,
        }
    }

    // TODO adjustments for doors and pushed walls !!!
    // TODO then, draw door edges !

    pub fn cast_ray(&mut self, angle: f64, cells: &[MapCell]) -> (f64, usize, f64) {
        self.prepare(angle);

        // keep advancing both rays, until one hits something
        while self.texture_idx.is_none() && (self.dist_x < FAR_AWAY || self.dist_y < FAR_AWAY) && self.max_steps > 0 {
            if self.dist_x < self.dist_y {
                self.advance_x_ray(cells);
            } else {
                self.advance_y_ray(cells);
            }
            self.max_steps -= 1;
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
        self.max_steps = Ord::max(self.map_width, self.map_height) * 3;

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

    fn advance_x_ray(&mut self, cells: &[MapCell]) {
        // moving on the X axis
        self.map_x += self.dir_x;
        self.map_idx += self.dir_x;

        // check if we hit a "solid" cell (wall or door)
        if self.map_x >= 0 && self.map_x < self.map_width && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                if cell.is_solid_textured() {
                    // the ray hit a wall
                    // TODO correct orientation OR use a bool instead of `Orientation` !!
                    self.texture_idx = Some(cell.texture(Orientation::East));
                    return;
                }
            }
        }
        // no hit, continue on the X axis
        self.dist_x += self.scale_x;
    }

    fn advance_y_ray(&mut self, cells: &[MapCell]) {
        // moving on the Y axis
        self.map_y += self.dir_y;
        self.map_idx += self.dir_y * self.map_width;

        // check if we hit a "solid" cell (wall or door)
        if self.map_y >= 0 && self.map_y < self.map_height && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                if cell.is_solid_textured() {
                    // the ray hit a wall
                    // TODO correct orientation OR use a bool instead of `Orientation` !!
                    self.texture_idx = Some(cell.texture(Orientation::North));
                    return;
                }
            }
        }

        // no hit, continue on the Y axis
        self.dist_y += self.scale_y;
    }
}
