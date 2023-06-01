//! Contains the ray casting algorithm, isolated and tuned for my implementation.

use crate::{Actor, MapCell, EPSILON};

const TEXIDX_DOOR_EDGES: usize = 100;

#[derive(Clone)]
pub struct TraversedCell {
    pub idx: usize,
    pub dist: f64,
    pub angle: f64,
}

pub struct RayCaster {
    map_width: i32,
    map_height: i32,
    player_x: f64,
    player_y: f64,
    player_angle: f64,
    sin: f64,
    cos: f64,
    map_x: i32,
    map_y: i32,
    map_idx: i32,
    ray_x: Ray,
    ray_y: Ray,
    texture_idx: Option<usize>,
    traversed_cells: Vec<TraversedCell>,
    door_prog: f64,
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
            player_angle: player.angle,
            sin: 0.0,
            cos: 0.0,
            map_x: 0,
            map_y: 0,
            map_idx: 0,
            ray_x: Default::default(),
            ray_y: Default::default(),
            texture_idx: None,
            traversed_cells: Vec::with_capacity(512),
            door_prog: 0.0,
        }
    }

    // TODO adjustments for doors and pushed walls !!!

    pub fn cast_ray(&mut self, angle: f64, cells: &[MapCell]) -> (f64, usize, f64) {
        self.prepare(angle);

        // keep advancing both rays, until one hits something
        let mut max_steps = Ord::max(self.map_width, self.map_height) << 1;
        while self.texture_idx.is_none() && (self.ray_x.not_far_away() || self.ray_y.not_far_away()) && max_steps > 0 {
            // check if coming from a door cell
            // (used for painting the door edges correctly)
            let mut from_door_cell = false;
            if self.map_idx >= 0 && (self.map_idx as usize) < cells.len() {
                from_door_cell = cells[self.map_idx as usize].is_door();
            }
            // check and advance the shorter of the 2 rays
            if self.ray_x.dist < self.ray_y.dist {
                self.advance_x_ray(cells, from_door_cell);
            } else {
                self.advance_y_ray(cells, from_door_cell);
            }
            // use a step counter, to avoid overflows when no-clipped out of bounds
            max_steps -= 1;
        }

        // if we got out of bounds => just paint something very far away, from any texture
        if self.texture_idx.is_none() {
            return (1e6, 0, 0.0);
        }

        // find the texture relative position and return data
        let texidx = self.texture_idx.unwrap_or(0);
        if self.ray_x.dist < self.ray_y.dist {
            // the hit was on a vertical wall
            let y_spot = self.player_y + self.ray_x.dist * self.sin;
            let texrelofs = if self.ray_x.dir > 0 {
                y_spot - y_spot.floor()
            } else {
                y_spot.floor() + 1.0 - y_spot
            };

            (self.ray_x.dist, texidx, texrelofs - self.door_prog)
        } else {
            // the hit was on a horizontal wall
            let x_spot = self.player_x + self.ray_y.dist * self.cos;
            let texrelofs = if self.ray_y.dir < 0 {
                x_spot - x_spot.floor()
            } else {
                x_spot.floor() + 1.0 - x_spot
            };

            (self.ray_y.dist, texidx, texrelofs - self.door_prog)
        }
    }

    pub fn into_visited_cells(mut self) -> Vec<TraversedCell> {
        self.traversed_cells
            .sort_unstable_by(|a, b| b.dist.partial_cmp(&a.dist).unwrap());
        self.traversed_cells
    }

    //----------------

    fn prepare(&mut self, angle: f64) {
        (self.sin, self.cos) = angle.sin_cos();
        self.map_x = self.player_x.floor() as i32;
        self.map_y = self.player_y.floor() as i32;
        self.map_idx = self.map_y * self.map_width + self.map_x;
        self.texture_idx = None;
        self.door_prog = 0.0;

        self.ray_x = Ray::init_x(self);
        self.ray_y = Ray::init_y(self);
    }

    fn advance_x_ray(&mut self, cells: &[MapCell], from_door_cell: bool) {
        // advance on the X axis
        self.map_x += self.ray_x.dir;
        self.map_idx += self.ray_x.dir;
        self.add_visited_cell();

        // check if we hit a "solid" cell (wall or door)
        let mut got_hit = false;
        if self.map_x >= 0 && self.map_x < self.map_width && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                got_hit = self.check_hit_x_ray(cell, from_door_cell);
            }
        }

        if !got_hit {
            // if not hit, continue along the X axis
            self.ray_x.dist += self.ray_x.scale;
        }
    }

    fn check_hit_x_ray(&mut self, cell: &MapCell, from_door_cell: bool) -> bool {
        if cell.is_push_wall() {
            // we MAY have hit the the push wall
            let prog = cell.get_progress();
            if prog < 1.0 {
                let (dist_to_push_wall, _, _) = self.ray_x.intersection(self, 1.0 - prog);
                if dist_to_push_wall <= self.ray_y.dist {
                    self.ray_x.dist = dist_to_push_wall;
                    self.texture_idx = Some(cell.get_texture() + 1);
                    return true;
                } else {
                    return false;
                }
            }
        }
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
            let (dist_to_door, _, iy) = self.ray_x.intersection(self, 0.5);
            if dist_to_door <= self.ray_y.dist {
                // we MAY HAVE hit the door
                let prog = cell.get_progress();
                let dy = iy - iy.floor();
                if dy >= prog {
                    self.ray_x.dist = dist_to_door;
                    self.ray_x.dir = 1;
                    self.texture_idx = Some(cell.get_texture());
                    self.door_prog = prog;
                    return true;
                }
            }
        }
        false
    }

    fn advance_y_ray(&mut self, cells: &[MapCell], from_door_cell: bool) {
        // advance on the Y axis
        self.map_y += self.ray_y.dir;
        self.map_idx += self.ray_y.dir * self.map_width;
        self.add_visited_cell();

        // check if we hit a "solid" cell (wall or door)
        let mut got_hit = false;
        if self.map_y >= 0 && self.map_y < self.map_height && self.map_idx >= 0 {
            if let Some(cell) = cells.get(self.map_idx as usize) {
                got_hit = self.check_hit_y_ray(cell, from_door_cell);
            }
        }

        if !got_hit {
            // if not hit, continue along the Y axis
            self.ray_y.dist += self.ray_y.scale;
        }
    }

    fn check_hit_y_ray(&mut self, cell: &MapCell, from_door_cell: bool) -> bool {
        if cell.is_push_wall() {
            // we MAY have hit the the push wall
            let prog = cell.get_progress();
            if prog < 1.0 {
                let (dist_to_push_wall, _, _) = self.ray_y.intersection(self, 1.0 - prog);
                if dist_to_push_wall <= self.ray_x.dist {
                    self.ray_y.dist = dist_to_push_wall;
                    self.texture_idx = Some(cell.get_texture());
                    return true;
                } else {
                    return false;
                }
            }
        }
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
            let (dist_to_door, ix, _) = self.ray_y.intersection(self, 0.5);
            if dist_to_door <= self.ray_x.dist {
                // we MAY HAVE hit the door
                let prog = cell.get_progress();
                let dx = ix - ix.floor();
                if dx >= prog {
                    self.ray_y.dist = dist_to_door;
                    self.ray_y.dir = -1;
                    self.texture_idx = Some(cell.get_texture());
                    self.door_prog = prog;
                    return true;
                }
            }
        }
        false
    }

    fn add_visited_cell(&mut self) {
        if self.map_x >= 0 && self.map_y >= 0 && self.map_x < self.map_width && self.map_y < self.map_height {
            let idx = self.map_idx as usize;
            let already_added = self.traversed_cells.iter().any(|x| x.idx == idx);
            if !already_added {
                // distances on both axis
                let dx = (self.map_x as f64) + 0.5 - self.player_x;
                let dy = (self.map_y as f64) + 0.5 - self.player_y;
                // angle between player's direction and cell's center
                let angle = dy.atan2(dx) - self.player_angle;
                // distance to cell's center - adjusted for fisheye
                let dist = (dx * dx + dy * dy).sqrt() * angle.cos();
                // add to vector
                let tc = TraversedCell { idx, dist, angle };
                self.traversed_cells.push(tc);
            }
        }
    }
}

//--------------------------
// Internal stuff
#[derive(Default)]
struct Ray {
    dist: f64,
    scale: f64,
    dir: i32,
}

impl Ray {
    // compute direction, scale and initial distance along X
    fn init_x(rc: &RayCaster) -> Self {
        let plx = rc.player_x;
        let plx_fl = plx.floor();
        if rc.cos > EPSILON {
            // looking RIGHT => this ray will hit the WEST face of a wall
            let dx = plx_fl + 1.0 - plx;
            Self {
                dist: dx / rc.cos,
                scale: 1.0 / rc.cos,
                dir: 1,
            }
        } else if rc.cos < -EPSILON {
            // looking LEFT => this ray will hit the EAST face of a wall
            let dx = plx_fl - plx;
            Self {
                dist: dx / rc.cos,
                scale: -1.0 / rc.cos,
                dir: -1,
            }
        } else {
            // straight vertical => no hits along the X axis
            Self {
                dist: 1e9,
                scale: 0.0,
                dir: 0,
            }
        }
    }

    // compute direction, scale and initial distance along Y
    fn init_y(rc: &RayCaster) -> Self {
        let ply = rc.player_y;
        let ply_fl = ply.floor();
        if rc.sin > EPSILON {
            // looking DOWN (map is y-flipped) => will hit NORTH
            let dy = ply_fl + 1.0 - ply;
            Self {
                dist: dy / rc.sin,
                scale: 1.0 / rc.sin,
                dir: 1,
            }
        } else if rc.sin < -EPSILON {
            // looking UP (map is y-flipped) => will hit SOUTH
            let dy = ply_fl - ply;
            Self {
                dist: dy / rc.sin,
                scale: -1.0 / rc.sin,
                dir: -1,
            }
        } else {
            // straight horizontal => no hits along the Y axis
            Self {
                dist: 1e9,
                scale: 0.0,
                dir: 0,
            }
        }
    }

    // Returns (new_dist, intersection X, intersection Y)
    #[inline]
    fn intersection(&self, rc: &RayCaster, advance: f64) -> (f64, f64, f64) {
        let adist = self.dist + self.scale * advance;
        (adist, rc.player_x + adist * rc.cos, rc.player_y + adist * rc.sin)
    }

    #[inline]
    fn not_far_away(&self) -> bool {
        self.dist < 1e6
    }
}

#[inline]
fn translate_point(x: f64, y: f64, angle: f64, dist: f64) -> (f64, f64) {
    let (sin, cos) = angle.sin_cos();
    let x2 = x + dist * cos;
    let y2 = y + dist * sin;
    (x2, y2)
}
