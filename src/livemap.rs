//! LiveMapSimulator - simulates the game world -> player, doors, actors, AI, timings etc

// TODO TEMPORARY !!!
#![allow(unused_variables, dead_code)]
use crate::{input::InputManager, MapData};

pub struct LiveMapSimulator {
    name: String,
    width: u16,
    height: u16,
    cells: Vec<MapCell>,
    // TODO .....
}

impl LiveMapSimulator {
    pub fn new(mapsrc: &MapData) -> Self {
        let name = mapsrc.name.to_string();
        let width = mapsrc.width;
        let height = mapsrc.height;
        // add map cells
        let len = (width as usize) * (height as usize);
        let mut cells = Vec::with_capacity(len);
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let c = MapCell {
                    tile: mapsrc.tile(x, y),
                    thing: mapsrc.thing(x, y),
                    flags: 0,
                };
                cells.push(c);
            }
        }
        // init live map
        let mut livemap = Self {
            name,
            width,
            height,
            cells,
        };
        init_live_map(&mut livemap);
        livemap
    }

    pub fn update_state(&mut self, elapsed_time: f64, inputs: &InputManager) {
        // TODO: update player, doors, actors etc
    }

    // TODO ............
}

//-------------------------------
// One cell from the live map

const FLG_IS_SOLID: u16 = 1 << 0;
const FLG_IS_DOOR: u16 = 1 << 1;
const FLG_WAS_SEEN: u16 = 1 << 2;

pub struct MapCell {
    // TODO should this struct be pub ??
    tile: u16,
    thing: u16,
    flags: u16,
    // TODO ...
}

//-------------------
//  Internal stuff

fn init_live_map(livemap: &mut LiveMapSimulator) {
    // TODO: compute tile flags, extract doors, live things, AMBUSH tiles etc.
    // -> see WOLF3D sources - e.g. https://github.com/id-Software/wolf3d/blob/master/WOLFSRC/WL_GAME.C#L221
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
