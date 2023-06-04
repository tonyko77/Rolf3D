//! The contents of a map grid cell from the live map.
//! Also, loads a map from its internal format into a grid of MapCells.

// TODO order this source - it is very messy :(((
// -> I have made a MESS of map cells and their data ://

use std::{collections::HashMap, f64::consts::PI};

use crate::MapData;

// tile constants -> see https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_DEF.H#L61
pub const PUSHABLE_TILE: u16 = 98;
pub const AMBUSH_TILE: u16 = 106;
pub const AREA_TILE: u16 = 107;
pub const ELEVATOR_TILE: u16 = 21;
//pub const EXITTILE: u16 = 99; // at end of castle
//pub const ALTELEVATORTILE: u16 = 107;

const NO_TEXTURE: u16 = 0xFFFF;
const NO_THING: u16 = 0; // TODO use this
const DOOR_TIMEOUT: f64 = 4.0; // TODO tune this (and also push wall timeout)

// see https://github.com/id-Software/wolf3d/blob/05167784ef009d0d0daefe8d012b027f39dc8541/WOLFSRC/WL_AGENT.C#L667
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Collectible {
    None,
    DogFood,        // health += 4
    GoodFood,       // health += 10
    FirstAid,       // health += 25
    Gibs1,          // health += 1 if health <= 10
    Gibs2,          // health += 1 if health <= 10
    AmmoClipSmall,  // ammo += 4, dropped by enemies
    AmmoClipNormal, // ammo += 8
    MachineGun,     // also, ammo += 6
    ChainGun,       // also, ammo += 6
    GoldKey,
    SilverKey,
    TreasureCross,  // score += 100
    TreasureCup,    // score += 500
    TreasureChest,  // score += 1000
    TreasureCrown,  // score += 5000
    TreasureOneUp,  // health += 100 , ammo += 25 , lives += 1
    AmmoBox,        // SOD only; ammo += 25
    SpearOfDestiny, // SOD only
}

impl Collectible {
    fn from_thing_code(thing: u16) -> Collectible {
        match thing {
            29 => Collectible::DogFood,
            43 => Collectible::GoldKey,
            44 => Collectible::SilverKey,
            47 => Collectible::GoodFood,
            48 => Collectible::FirstAid,
            49 => Collectible::AmmoClipNormal,
            50 => Collectible::MachineGun,
            51 => Collectible::ChainGun,
            52 => Collectible::TreasureCross,
            53 => Collectible::TreasureCup,
            54 => Collectible::TreasureChest,
            55 => Collectible::TreasureCrown,
            56 => Collectible::TreasureOneUp,
            57 => Collectible::Gibs1,
            61 => Collectible::Gibs2,
            72 => Collectible::AmmoBox,
            74 => Collectible::SpearOfDestiny,
            _ => Collectible::None,
        }
    }

    fn sprite(&self) -> u16 {
        match self {
            Collectible::DogFood => 8,
            Collectible::GoldKey => 22,
            Collectible::SilverKey => 23,
            Collectible::GoodFood => 26,
            Collectible::FirstAid => 27,
            Collectible::AmmoClipSmall => 28,
            Collectible::AmmoClipNormal => 28,
            Collectible::MachineGun => 29,
            Collectible::ChainGun => 30,
            Collectible::TreasureCross => 31,
            Collectible::TreasureCup => 32,
            Collectible::TreasureChest => 33,
            Collectible::TreasureCrown => 34,
            Collectible::TreasureOneUp => 35,
            Collectible::Gibs1 => 36,
            Collectible::Gibs2 => 40,
            Collectible::AmmoBox => 51,
            Collectible::SpearOfDestiny => 53,
            _ => NO_TEXTURE,
        }
    }
}
//-----------------------
// TODO just do a proper impl for this one :///
pub struct Actor {
    pub thing: u16,
    pub x: f64,
    pub y: f64,
    pub angle: f64,
}

//-----------------------

// TODO clean up
#[derive(Clone, Copy, PartialEq)]
pub enum CellState {
    None,
    Open { timeout: f64 },
    Closed,
    Opening { progress: f64 },
    Closing { progress: f64 },
    Pushing { progress: f64 },
}

#[derive(Clone)]
pub struct MapCell {
    pub tile: u16,  // TODO temp pub ...
    pub thing: u16, // TODO temp pub ...
    tex_sprt: u16,  // texture or sprite index
    flags: u8,      // state + various flags
    coll: Collectible,
    pub state: CellState,
}

impl MapCell {
    #[inline]
    pub fn is_wall(&self) -> bool {
        self.has_flag(FLG_IS_WALL)
    }

    #[inline]
    pub fn is_push_wall(&self) -> bool {
        self.has_flag(FLG_IS_PUSH_WALL)
    }

    #[inline]
    pub fn is_door(&self) -> bool {
        self.tile >= 90 && self.tile <= 101
    }

    #[inline]
    pub fn is_horiz_door(&self) -> bool {
        self.tile >= 90 && self.tile <= 101 && (self.tile & 0x01) != 0
    }

    #[inline]
    pub fn is_vert_door(&self) -> bool {
        self.tile >= 90 && self.tile <= 101 && (self.tile & 0x01) == 0
    }

    /// For locked doors, retuns a key type (1 = gold, 2 = silver etc)
    /// Otherwise, returns 0.
    #[inline]
    pub fn get_door_key_type(&self) -> u8 {
        if self.tile >= 92 && self.tile <= 99 {
            ((self.tile - 90) / 2) as u8
        } else {
            0
        }
    }

    #[inline]
    pub fn actor_entered(&mut self) {
        self.flags |= FLG_HAS_ACTOR;
    }

    #[inline]
    pub fn actor_left(&mut self) {
        self.flags &= !FLG_HAS_ACTOR;
    }

    #[inline]
    pub fn has_actor(&self) -> bool {
        self.has_flag(FLG_HAS_ACTOR)
    }

    #[inline]
    pub fn set_seen(&mut self) {
        self.flags |= FLG_WAS_SEEN;
    }

    #[inline]
    pub fn was_seen(&self) -> bool {
        self.has_flag(FLG_WAS_SEEN)
    }

    /// Solid cells cannot be walked into by actors.
    /// This is used for collision detection.
    #[inline]
    pub fn is_solid(&self) -> bool {
        self.is_wall()
            || self.has_flag(FLG_IS_SOLID_SPRITE)
            || self.has_actor()
            || (self.is_door() && !matches!(self.state, CellState::Open { timeout: _ }))
    }

    #[inline]
    pub fn is_actionable(&self) -> bool {
        self.tile == ELEVATOR_TILE || self.is_door() || self.is_push_wall()
    }

    #[inline]
    pub fn get_area(&self) -> u16 {
        if self.tile >= AREA_TILE {
            self.tile
        } else {
            0
        }
    }

    pub fn update_state(&mut self, elapsed_time: f64) {
        if self.is_door() {
            // update door state
            match self.state {
                CellState::Opening { progress } => {
                    let p = progress + elapsed_time;
                    self.state = if p >= 1.0 {
                        CellState::Open { timeout: DOOR_TIMEOUT }
                    } else {
                        CellState::Opening { progress: p }
                    };
                }
                CellState::Closing { progress } => {
                    let p = progress - elapsed_time;
                    self.state = if p <= 0.0 {
                        CellState::Closed
                    } else {
                        CellState::Closing { progress: p }
                    };
                }
                CellState::Open { timeout } => {
                    // only count down if no actor is blocking the door
                    if self.flags & FLG_HAS_ACTOR == 0 {
                        let t = timeout - elapsed_time;
                        self.state = if t <= 0.0 {
                            CellState::Closing { progress: 1.0 }
                        } else {
                            CellState::Open { timeout: t }
                        };
                    }
                }
                _ => {}
            }
        } else if self.flags & FLG_IS_PUSH_WALL != 0 {
            // update push wall state
            if let CellState::Pushing { progress } = self.state {
                let upd_prg = progress - elapsed_time;
                if upd_prg > 0.0 {
                    self.state = CellState::Pushing { progress: upd_prg }
                } else {
                    // finished pushing wall
                    self.state = CellState::None;
                    self.flags = 0;
                    self.tex_sprt = NO_TEXTURE;
                    self.tile = self.thing;
                    self.thing = NO_THING;
                    // TODO set area code = neighbouring area !!
                }
            }
        }
    }

    /// Returns true if started a push wall
    pub fn activate_door_or_elevator(&mut self, _dx: i32, _dy: i32) -> bool {
        // open/close door
        if self.is_door() {
            self.state = match self.state {
                CellState::Open { timeout: _ } => CellState::Closing { progress: 1.0 },
                CellState::Opening { progress } => CellState::Closing { progress },
                CellState::Closed => CellState::Opening { progress: 0.0 },
                CellState::Closing { progress } => CellState::Opening { progress },
                _ => CellState::Closed,
            };
        }

        // TODO elevator
        // (push wall is handled separately)
        false
    }

    // TODO Not Needed - always push secret walls only 2 tiles
    #[inline]
    pub fn can_push_wall_into(&self) -> bool {
        !(self.is_door() || self.is_wall() || self.has_actor() || self.has_flag(FLG_IS_SOLID_SPRITE))
    }

    // TODO push direction - is it needed ???
    pub fn start_push_wall(&mut self, area_code: u16, wall_texture: u16, progress: f64) {
        self.flags = FLG_IS_WALL | FLG_IS_PUSH_WALL;
        self.state = CellState::Pushing { progress };
        self.tex_sprt = wall_texture;
        // temporarily store the area code into the thing
        self.thing = area_code;
    }

    pub fn end_push_wall(&mut self, wall_texture: u16) {
        self.flags = FLG_IS_WALL;
        self.state = CellState::None;
        self.tex_sprt = wall_texture;
        self.thing = NO_THING;
    }

    pub fn get_progress(&self) -> f64 {
        match self.state {
            CellState::Opening { progress } => progress,
            CellState::Closing { progress } => progress,
            CellState::Open { timeout: _ } => 1.0,
            CellState::Pushing { progress } => progress,
            CellState::Closed => 0.0,
            _ => 1.0,
        }
    }

    #[inline]
    pub fn collectible(&self) -> Collectible {
        self.coll
    }

    #[inline]
    pub fn remove_collectible(&mut self) {
        self.coll = Collectible::None;
    }

    pub fn get_texture(&self) -> usize {
        // check for regular texture
        (if self.is_wall() {
            self.tex_sprt
        } else if self.is_door() {
            self.tex_sprt
        } else {
            NO_TEXTURE
        }) as usize
    }

    #[inline]
    pub fn get_sprite(&self) -> u16 {
        if self.coll != Collectible::None {
            self.coll.sprite()
        } else if self.flags & FLG_IS_SPRITE != 0 {
            self.tex_sprt
        } else {
            NO_TEXTURE
        }
    }

    #[inline(always)]
    fn has_flag(&self, flag: u8) -> bool {
        self.flags & flag != 0
    }
}

pub fn load_map_to_cells(mapsrc: &MapData, is_sod: bool) -> (Vec<MapCell>, Vec<Actor>) {
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
                coll: Collectible::None,
                state: CellState::None,
            };
            cells.push(c);
        }
    }

    // init cells
    let mut actors = Vec::with_capacity(100);
    for idx in 0..len {
        // TODO - also extract player, actors, doors from each cell
        let maybe_actor = init_map_cell(&mut cells, idx, width as usize, is_sod);
        if let Some(actor) = maybe_actor {
            let is_player = actor.thing >= 19 && actor.thing <= 22;
            let actor_cnt = actors.len();
            actors.push(actor);
            if is_player && actor_cnt > 0 {
                actors.swap(0, actor_cnt);
            }
        }
    }

    _temp_map_statistics(&cells);
    (cells, actors)
}

//-------------------
//  Internal stuff
//-------------------

const FLG_IS_WALL: u8 = 1 << 0;
const FLG_IS_PUSH_WALL: u8 = 1 << 1;
const FLG_IS_SPRITE: u8 = 1 << 2;
const FLG_IS_SOLID_SPRITE: u8 = 1 << 3;
const FLG_HAS_ACTOR: u8 = 1 << 4;
const FLG_IS_AMBUSH: u8 = 1 << 5;
const FLG_WAS_SEEN: u8 = 1 << 6;

fn init_map_cell(cells: &mut Vec<MapCell>, idx: usize, width: usize, is_sod: bool) -> Option<Actor> {
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
            cell.state = CellState::Closed;
            cell.tex_sprt = if cell.tile >= 100 {
                // elevator doors
                103 - (cell.tile & 0x01)
            } else if cell.tile < 92 {
                // regular doors
                99 - (cell.tile & 0x01)
            } else {
                // locked doors
                105 - (cell.tile & 0x01)
            }
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
    // also pick up collectible

    // TODO make sure it is CORRECT !! - at least PLAYER START POS
    // -> https://github.com/id-Software/wolf3d/blob/05167784ef009d0d0daefe8d012b027f39dc8541/WOLFSRC/WL_GAME.C#L214
    match cell.thing {
        19..=22 => {
            // player start position
            cell.flags |= FLG_HAS_ACTOR;
            actor = Some(Actor {
                thing: cell.thing,
                x: (x as f64) + 0.5,
                y: (y as f64) + 0.5,
                angle: orientation_to_angle(cell.thing - 19),
            });
        }
        23..=74 => {
            // Static decorations + collectibles
            cell.coll = Collectible::from_thing_code(cell.thing);
            if cell.coll == Collectible::None {
                // not a collectible => it is a decoration sprite
                cell.flags |= FLG_IS_SPRITE;
                cell.tex_sprt = cell.thing - 21;
                if is_solid_decoration(cell.thing as u8, is_sod) {
                    cell.flags |= FLG_IS_SOLID_SPRITE;
                }
            }
        }
        // TODO enemies, etc
        _ => {}
    }

    actor
}

fn orientation_to_angle(x: u16) -> f64 {
    // original code: player->angle = (1-dir)*90;
    match x & 0x03 {
        0 => PI * 3.0 / 2.0, // North (but my unit circle is flipped)
        1 => 0.0,            // East
        2 => PI / 2.0,       // South (but my unit circle is flipped)
        3 => PI,             // West
        _ => panic!("x & 0x03 should be between 0 and 3 ?!?"),
    }
}

/// Check if a given static thing code is solid or not.
/// Some things are solid depending on game (Wolf3D or SOD).
fn is_solid_decoration(thing: u8, is_sod: bool) -> bool {
    const COMMON_SOLIDS: &[u8] = &[
        24, 25, 26, 28, 30, 31, 33, 34, 35, 36, 39, 40, 41, 45, 58, 59, 60, 62, 68, 69, 71, 73,
    ];

    match thing {
        38 => is_sod,  // Gibs, solid only in SOD
        67 => is_sod,  // Gibs, solid only in SOD
        63 => !is_sod, // "Call Apogee", solid only in Wolf3D
        _ => COMMON_SOLIDS.binary_search(&thing).is_ok(),
    }
}

//---------------------

// TODO temporary - print statistics of map
fn _temp_map_statistics(cells: &Vec<MapCell>) {
    // collect data
    let mut tiles: HashMap<u16, i32> = HashMap::new();
    let mut things: HashMap<u16, i32> = HashMap::new();
    for mc in cells {
        let cnt = tiles.get(&mc.tile).cloned().unwrap_or(0) + 1;
        tiles.insert(mc.tile, cnt);
        let cnt = tiles.get(&mc.thing).cloned().unwrap_or(0) + 1;
        things.insert(mc.thing, cnt);
    }

    // 96..99 are UNUSED door types
    // 102..105 are UNUSED wall types
    for t in 96..=105 {
        let cnt = tiles.get(&t).cloned().unwrap_or(0);
        if cnt > 0 && t != 100 && t != 101 {
            println!(" -> UNUSED Tile #{t} => {cnt} times");
        }
    }
    // println!("Map things:");
    // for t in 1..255 {
    // //for t in 43..45 {
    //     let cnt = things.get(&t).cloned().unwrap_or(0);
    //     if cnt > 0 {
    //         println!(" -> Thing #{t} => {cnt} times");
    //     }
    // }
}
