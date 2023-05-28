//! Loads a map from its internal format into a grid of "live" MapCells.

// TODO order this source - it is very messy :(((
// -> I have made a MESS of map cells and their data ://

use std::f64::consts::PI;

use crate::MapData;

// tile constants -> see https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61
//pub const EXITTILE: u16 = 99; // at end of castle
//pub const NUMAREAS: u16 = 37;
pub const PUSHABLE_TILE: u16 = 98;
pub const AMBUSH_TILE: u16 = 106;
pub const AREA_TILE: u16 = 107; // first of NUMAREAS floor tiles
pub const ELEVATOR_TILE: u16 = 21;
//pub const ALTELEVATORTILE: u16 = 107;

const _TEXIDX_DARK_ELEVATOR: usize = 25;
const _TEXIDX_LIGHT_ELEVATOR: usize = 24;
const TEXIDX_ELEVATOR_SWITCH: usize = 41;

/// Enum for map cell types
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

//-----------------------
// TODO just do a proper impl for this one :///
pub struct Actor {
    pub kind: ActorType,
    pub thing: u16,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub state: i32, // TODO ...
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActorType {
    Player,
    Enemy,
    Door,
    PushWall,
}
//-----------------------

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
    pub fn is_solid_textured(&self) -> bool {
        (self.flags & (FLG_IS_WALL | FLG_IS_DOOR)) != 0
    }

    #[inline]
    pub fn is_wall(&self) -> bool {
        (self.flags & FLG_IS_WALL) != 0
    }

    #[inline]
    pub fn is_door(&self) -> bool {
        (self.flags & FLG_IS_DOOR) != 0
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
            ELEVATOR_TILE => TEXIDX_ELEVATOR_SWITCH, // elevator switch instead of handle bars
            1..=106 => self.tex_sprt as usize,
            _ => NO_TEXTURE as usize,
        }
    }

    pub fn texture(&self) -> usize {
        // check for regular texture
        (if self.flags & FLG_IS_WALL != 0 {
            self.tex_sprt
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

pub fn load_map_to_cells(mapsrc: &MapData) -> (Vec<MapCell>, Actor, Vec<Actor>) {
    let width = mapsrc.width;
    let height = mapsrc.height;
    let len = (width as usize) * (height as usize);
    let mut cells = Vec::with_capacity(len);

    // create cells
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
    let mut player = None;
    let mut other_actors = Vec::with_capacity(100);
    for idx in 0..len {
        // TODO - also extract player, actors, doors from each cell
        let act = init_map_cell(&mut cells, idx, width as usize);
        if let Some(actor) = act {
            if actor.kind == ActorType::Player {
                player = Some(actor);
            } else {
                other_actors.push(actor);
            }
        }
    }

    (cells, player.expect("Actor not found on map"), other_actors)
}

//-------------------
//  Internal stuff
//-------------------

const FLG_IS_WALKABLE: u16 = 1 << 0;
const FLG_IS_WALL: u16 = 1 << 1;
const FLG_IS_PUSH_WALL: u16 = 1 << 2;
const FLG_IS_DOOR: u16 = 1 << 3;
const FLG_IS_AMBUSH: u16 = 1 << 4;
const FLG_HAS_DECO_SPRITE: u16 = 1 << 5;
const FLG_HAS_COLLECTIBLE: u16 = 1 << 6;
const FLG_HAS_TREASURE: u16 = 1 << 7;
const _FLG_WAS_SEEN: u16 = 1 << 8;

const NO_TEXTURE: u16 = 0xFF00;

fn init_map_cell(cells: &mut Vec<MapCell>, idx: usize, width: usize) -> Option<Actor> {
    let cell = cells.get_mut(idx).unwrap();

    // check tiles
    match cell.tile {
        1..=89 => {
            // wall
            cell.flags |= FLG_IS_WALL;
            // wall textures are "in pairs" - alternating light and dark versions:
            // LIGHT(+0) textures are used for N/S, and DARK(+1) ones for E/W
            cell.tex_sprt = (cell.tile - 1) * 2;
            if cell.thing == PUSHABLE_TILE {
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
        }
        106 => {
            // ambush tile
            cell.flags |= FLG_IS_AMBUSH | FLG_IS_WALKABLE;
            // TODO set area code into tile ...
        }
        107.. => {
            // empty tile
            cell.flags |= FLG_IS_WALKABLE;
            // TODO check for things - collectibles, actors, decos etc
        }
        _ => {
            panic!("Unknown tile code: {}", cell.tile);
        }
    }

    // check things
    let mut actor = None;
    let x = idx % width;
    let y = idx / width;
    // TODO make sure it is CORRECT !! - at least PLAYER START POS
    // -> https://github.com/id-Software/wolf3d/blob/05167784ef009d0d0daefe8d012b027f39dc8541/WOLFSRC/WL_GAME.C#L214
    match cell.thing {
        19..=22 => {
            // player start position
            actor = Some(Actor {
                kind: ActorType::Player,
                thing: cell.thing,
                x: (x as f64) + 0.5,
                y: (y as f64) + 0.5,
                angle: orientation_to_angle(cell.thing - 19),
                state: 0,
            });
        }
        23..=74 => {
            // Static decorations
            // TODO probably also collectibles, solid + non-solid deco-s etc
            cell.flags |= FLG_HAS_DECO_SPRITE;
            cell.tex_sprt = cell.thing - 21;
        }
        // TODO enemies, etc
        _ => {}
    }

    actor
}

fn orientation_to_angle(x: u16) -> f64 {
    match x & 0x03 {
        0 => PI / 2.0,       // North
        1 => 0.0,            // East
        2 => PI * 3.0 / 2.0, // South
        3 => PI,             // West
        _ => panic!("x & 0x03 should be between 0 and 3 ?!?"),
    }
}
