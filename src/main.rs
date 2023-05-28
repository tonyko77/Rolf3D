//! ROLF3D - a Rust implementation of the WOLF3D raycasting engine :)
//! Main starting point.

// This magic line prevents the opening of a terminal when launching a release build
//#![cfg_attr(not(any(test, debug_assertions)), windows_subsystem = "windows")]

use rolf3d::*;

const SCR_WIDTH: i32 = 640;
const SCR_HEIGHT: i32 = 480;
const PIXEL_SIZE: i32 = 1;
const SLEEP_KIND: SleepKind = SleepKind::SLEEP(1);

fn main() {
    // load and prepare game assets
    let assets = GameAssets::load().expect("ERROR in ROLF3D: failed to load game assets");

    // main game loop
    let sdl_config = SdlConfiguration::new("ROLF3D", SCR_WIDTH, SCR_HEIGHT, PIXEL_SIZE, SLEEP_KIND);
    let mut gameloop = GameLoop::new(SCR_WIDTH, SCR_HEIGHT, PIXEL_SIZE, assets);
    let result = run_game_loop(&sdl_config, &mut gameloop);

    match result {
        Ok(_) => println!("ROLF3D finished OK :)"),
        Err(msg) => println!("ERROR in ROLF3D: {msg}"),
    }
}
