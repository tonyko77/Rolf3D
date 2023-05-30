//! Loads a map from its internal format into a grid of "live" MapCells.

// TODO order this source - it is very messy :(((
// -> I have made a MESS of map cells and their data ://

use std::f64::consts::PI;

use crate::MapData;

// tile constants -> see https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61
pub const PUSHABLE_TILE: u16 = 98;
pub const AMBUSH_TILE: u16 = 106;
pub const AREA_TILE: u16 = 107;
pub const ELEVATOR_TILE: u16 = 21;
//pub const EXITTILE: u16 = 99; // at end of castle
//pub const ALTELEVATORTILE: u16 = 107;

const NO_TEXTURE: u16 = 0xFFFF;

// #[derive(Clone, Copy, PartialEq, Eq)]
// pub enum Collectible {
//     None,
//     Treasure(u16),
//     Health(u16),
//     Ammo(u16),
//     Weapon(u16),
//     OneUp,
// }

//-----------------------
// TODO just do a proper impl for this one :///
pub struct Actor {
    pub thing: u16,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub state: i32, // TODO ...
}

//-----------------------

#[derive(Clone)]
pub struct MapCell {
    pub tile: u16,  // TODO temp pub ...
    pub thing: u16, // TODO temp pub ...
    tex_sprt: u16,  // texture or sprite index
    flags: u16,     // state + various flags
    _progress: f64, // for moving walls, opening/closing doors
}

impl MapCell {
    #[inline]
    pub fn is_wall(&self) -> bool {
        (self.flags & FLG_IS_WALL) != 0
    }

    #[inline]
    pub fn is_door(&self) -> bool {
        (self.flags & FLG_IS_DOOR) != 0
    }

    // TODO if door = OPENED => NOT solid !!
    #[inline]
    pub fn is_solid(&self) -> bool {
        (self.flags & (FLG_IS_DOOR | FLG_IS_WALL)) != 0
    }

    #[inline]
    pub fn is_horiz_door(&self) -> bool {
        (self.flags & FLG_IS_HORIZ_DOOR) != 0
    }

    #[inline]
    pub fn is_vert_door(&self) -> bool {
        (self.flags & FLG_IS_VERT_DOOR) != 0
    }

    #[inline]
    pub fn is_actionable(&self) -> bool {
        self.tile == ELEVATOR_TILE || (self.flags & (FLG_IS_DOOR | FLG_IS_PUSH_WALL)) != 0
    }

    // #[inline]
    // pub fn collectible(&self) -> Collectible {
    //     if (self.tile & FLG_IS_WALKABLE) != 0 {
    //         // only walkable cells can contain something collectible
    //         Collectible::None // TODO implement this ...
    //     } else {
    //         Collectible::None
    //     }
    // }

    pub fn get_texture(&self) -> usize {
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
    pub fn get_sprite(&self) -> u16 {
        if self.flags & FLG_IS_SPRITE != 0 {
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
                _progress: 0.0,
            };
            cells.push(c);
        }
    }

    // init cells
    let mut player = None;
    let mut enemies = Vec::with_capacity(100);
    for idx in 0..len {
        // TODO - also extract player, actors, doors from each cell
        let act = init_map_cell(&mut cells, idx, width as usize);
        if let Some(actor) = act {
            let is_player = actor.thing >= 19 && actor.thing <= 22;
            if is_player {
                player = Some(actor);
            } else {
                enemies.push(actor);
            }
        }
    }

    (cells, player.expect("Player not found on map"), enemies)
}

//-------------------
//  Internal stuff
//-------------------

// const GET_STATE_MASK: u16 = 0x000F;
// const SET_STATE_MASK: u16 = 0xFFF0;
// const STATE_DOOR_CLOSED: u16 = 0;
// const STATE_DOOR_OPENING: u16 = 1;
// const STATE_DOOR_OPEN: u16 = 2;
// const STATE_DOOR_CLOSING: u16 = 3;
// const STATE_DOOR_LOCKED_BLUE_KEY: u16 = 4;
// const STATE_DOOR_LOCKED_YELLOW_KEY: u16 = 5;
// const STATE_PUSH_WALL_CLOSED: u16 = 0;
// const STATE_PUSH_WALL_OPENING: u16 = 1;

const FLG_IS_WALL: u16 = 1 << 4;
const FLG_IS_PUSH_WALL: u16 = 1 << 5;
const FLG_IS_SPRITE: u16 = 1 << 6;
const FLG_IS_HORIZ_DOOR: u16 = 1 << 7;
const FLG_IS_VERT_DOOR: u16 = 1 << 8;
const FLG_IS_DOOR: u16 = FLG_IS_HORIZ_DOOR | FLG_IS_VERT_DOOR;
// const FLG_IS_LOCKED_DOOR: u16 = 1 << 9;
// const FLG_HAS_BLOCKER_DECO: u16 = 1 << 10;
// const FLG_HAS_COLLECTIBLE: u16 = 1 << 11;
const FLG_IS_AMBUSH: u16 = 1 << 12;
//const FLG_WAS_SEEN: u16 = 1 << 13;
//const FLG_IS_AREA: u16 = 1 << 14; // TODO is this useful

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
                cell.flags |= FLG_IS_PUSH_WALL;
            }
        }
        90..=101 => {
            // door
            // TODO improve detection of locked doors etc
            if cell.tile & 0x01 == 0 {
                cell.flags |= FLG_IS_VERT_DOOR;
            } else {
                cell.flags |= FLG_IS_HORIZ_DOOR;
            }
            cell.tex_sprt = if cell.tile >= 100 {
                cell.tile - 76
            } else {
                (cell.tile ^ 1) + 8
            };
        }
        106 => {
            // ambush tile
            cell.flags |= FLG_IS_AMBUSH;
            // TODO get area code from a neighbouring tile? is that necessary?
        }
        107.. => {
            // area tile => nothing to do, the tile
            cell.flags = 0;
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
            cell.flags |= FLG_IS_SPRITE;
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
