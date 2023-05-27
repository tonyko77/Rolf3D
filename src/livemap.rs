//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

use crate::{input::InputManager, MapData};

// tile constants -> see https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61
pub const AMBUSHTILE: u16 = 106;
pub const AREATILE: u16 = 107; // first of NUMAREAS floor tiles
pub const PUSHABLETILE: u16 = 98;
//pub const EXITTILE: u16 = 99; // at end of castle
//pub const NUMAREAS: u16 = 37;
//pub const ELEVATORTILE: u16 = 21;
//pub const ALTELEVATORTILE: u16 = 107;

pub struct LiveMapSimulator {
    name: String,
    cells: Vec<MapCell>,
    width: u16,
    height: u16,
    episode: u8,
    level: u8,
    total_enemies: u16,
    total_secrets: u16,
    total_treasures: u16,
    cnt_kills: u16,
    cnt_secrets: u16,
    cnt_treasures: u16,
}

impl LiveMapSimulator {
    pub fn new(index: usize, mapsrc: &MapData) -> Self {
        new_live_map(index, mapsrc)
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

    pub fn update_player(&mut self, _elapsed_time: f64, _inputs: &InputManager) {
        // TODO: update doors, secret walls, actors - only if NOT paused
    }

    pub fn update_actors(&mut self, _elapsed_time: f64) {
        // TODO: update player - only if in 3D view and NOT paused
    }

    pub fn automap_description(&self) -> String {
        format!("{} - ep. {}, level {}", self.name, self.episode, self.level)
    }

    pub fn automap_secrets(&self) -> String {
        format!(
            "K: {}/{}   T: {}/{}    S: {}/{}",
            self.cnt_kills,
            self.total_enemies,
            self.cnt_treasures,
            self.total_treasures,
            self.cnt_secrets,
            self.total_secrets
        )
    }
    // TODO ............
}

//------------
// Map Cell
//------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    North,
    West,
    South,
    East,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Walkable,
    Wall,
    Door,
    SolidDeco,
    Elevator,
    SecretElevator,
    EndEpisode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Collectible {
    None,
    Treasure,
    Health(u16),
    Ammo(u16),
    Weapon(u16),
    OneUp,
}

#[derive(Clone)]
pub struct MapCell {
    pub tile: u16,  // TODO temp pub ...
    pub thing: u16, // TODO temp pub ...
    tex_sprt: u16,
    flags: u16,
}

impl MapCell {
    #[inline]
    pub fn cell_type(&self) -> CellType {
        // TODO REPAIR & IMPROVE THIS !!!
        // (at the moment it is INCOMPLETE - see enum above)
        if (self.flags & FLG_IS_WALKABLE) != 0 {
            CellType::Walkable
        } else if (self.flags & FLG_IS_WALL) != 0 {
            CellType::Wall
        } else if (self.flags & FLG_IS_DOOR) != 0 {
            CellType::Door
        } else {
            CellType::SolidDeco
        }
    }

    #[inline]
    pub fn collectible(&self) -> Collectible {
        if (self.tile & FLG_IS_WALKABLE) != 0 {
            // only walkable cells can contain something collectible
            Collectible::None // TODO implement this ...
        } else {
            Collectible::None
        }
    }

    // TODO
    pub fn automap_texture(&self) -> usize {
        match self.tile {
            21 => 41, // elevator switch instead of handle bars
            1..=106 => self.tex_sprt as usize,
            _ => NO_TEXTURE as usize,
        }
    }

    pub fn texture(&self, ori: Orientation) -> usize {
        // check if it has a door in that area
        let door_flag = 1 << (ori as u16);
        if (self.flags & door_flag) != 0 {
            // TODO door "hinge" texture
            return 1;
        }
        // check for regular texture
        (if self.flags & FLG_IS_WALL != 0 {
            self.tex_sprt + ((ori as u16) & 0x01)
        } else if self.flags & FLG_IS_DOOR != 0 {
            self.tex_sprt
        } else {
            NO_TEXTURE
        }) as usize
    }

    #[inline]
    pub fn sprite(&self) -> u16 {
        let test = self.flags & (FLG_HAS_DECO_SPRITE | FLG_HAS_COLLECTIBLE | FLG_HAS_TREASURE);
        if test != 0 {
            self.tex_sprt
        } else {
            NO_TEXTURE
        }
    }
}

//-------------------
//  Internal stuff
//-------------------

const FLG_HAS_DOOR_N: u16 = 1 << 0;
const FLG_HAS_DOOR_W: u16 = 1 << 1;
const FLG_HAS_DOOR_S: u16 = 1 << 2;
const FLG_HAS_DOOR_E: u16 = 1 << 3;
const FLG_IS_WALKABLE: u16 = 1 << 4;
const FLG_IS_WALL: u16 = 1 << 5;
const FLG_IS_PUSH_WALL: u16 = 1 << 6;
const FLG_IS_DOOR: u16 = 1 << 7;
const FLG_IS_AMBUSH: u16 = 1 << 8;
const FLG_HAS_DECO_SPRITE: u16 = 1 << 9;
const FLG_HAS_COLLECTIBLE: u16 = 1 << 10;
const FLG_HAS_TREASURE: u16 = 1 << 11;
// TODO const FLG_WAS_SEEN: u16 = 1 << 12;

const NO_TEXTURE: u16 = 0xFF00;

fn new_live_map(index: usize, mapsrc: &MapData) -> LiveMapSimulator {
    // init data
    let name = mapsrc.name.to_string();
    let episode = (index / 10 + 1) as u8;
    let level = (index % 10 + 1) as u8;

    let width = mapsrc.width;
    let height = mapsrc.height;

    // TODO compute these from each cell ...
    let /*mut*/ total_enemies = 0;
    let /*mut*/ total_secrets = 0;
    let /*mut*/ total_treasures = 0;

    // add map cells
    let len = (width as usize) * (height as usize);
    let mut cells = Vec::with_capacity(len);
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let c = MapCell {
                tile: mapsrc.tile(x, y),
                thing: mapsrc.thing(x, y),
                tex_sprt: NO_TEXTURE,
                flags: 0,
            };
            cells.push(c);
        }
    }
    // init cells
    for idx in 0..len {
        // TODO - also extract player, actors, doors from each cell
        init_map_cell(&mut cells, idx, width as usize);
    }

    // init live map
    // TODO: compute tile flags, extract doors, live things, AMBUSH tiles etc.
    // -> see WOLF3D sources - e.g. https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
    LiveMapSimulator {
        name,
        episode,
        level,
        width,
        height,
        cells,
        total_enemies,
        total_secrets,
        total_treasures,
        cnt_kills: 0,
        cnt_secrets: 0,
        cnt_treasures: 0,
    }
}

fn init_map_cell(cells: &mut Vec<MapCell>, idx: usize, width: usize) {
    let cell = cells.get_mut(idx).unwrap();
    let mut is_horiz_door = false;
    let mut is_vert_door = false;

    // check doors and solid walls
    match cell.tile {
        1..=89 => {
            // wall
            cell.flags |= FLG_IS_WALL;
            // wall textures are "in pairs" - alternating light and dark versions:
            // LIGHT(+0) textures are used for N/S, and DARK(+1) ones for E/W
            cell.tex_sprt = (cell.tile - 1) * 2;
            if cell.thing == PUSHABLETILE {
                // TODO check if this is correct
                cell.flags |= FLG_IS_PUSH_WALL;
            }
        }
        90..=101 => {
            // door
            cell.flags |= FLG_IS_DOOR;
            // doing 1 ^ because even door codes are vertical (facing E/W),
            // but they correspond to light versions of the textures :/
            cell.tex_sprt = 1 ^ if cell.tile >= 100 {
                cell.tile - 76
            } else {
                cell.tile + 8
            };
            if cell.tile & 1 == 0 {
                is_vert_door = true;
            } else {
                is_horiz_door = true;
            }
        }
        106 => {
            // ambush tile
            cell.flags |= FLG_IS_AMBUSH | FLG_IS_WALKABLE;
            // TODO set area code into tile ...
            cell.tex_sprt = NO_TEXTURE;
        }
        107.. => {
            // empty tile
            cell.flags |= FLG_IS_WALKABLE;
            // TODO check for things - collectibles, actors, decos etc
            cell.tex_sprt = NO_TEXTURE;
        }
        _ => {
            panic!("Unknown tile code: {}", cell.tile);
        }
    }

    // for easier drawing, mark cells that neighbour a door
    if is_horiz_door {
        let cell_left = cells.get_mut(idx - 1).unwrap();
        assert!(cell_left.tile <= 89);
        cell_left.flags |= FLG_HAS_DOOR_W;
        let cell_right = cells.get_mut(idx + 1).unwrap();
        assert!(cell_right.tile <= 89);
        cell_right.flags |= FLG_HAS_DOOR_E;
    } else if is_vert_door {
        let cell_up = cells.get_mut(idx - width).unwrap();
        assert!(cell_up.tile <= 89);
        cell_up.flags |= FLG_HAS_DOOR_S;
        let cell_dn = cells.get_mut(idx + width).unwrap();
        assert!(cell_dn.tile <= 89);
        cell_dn.flags |= FLG_HAS_DOOR_N;
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
