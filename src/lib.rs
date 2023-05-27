//! ROLF3D - a Rust implementation of the WOLF3D raycasting engine :)
//! Main library.

mod assetloader;
mod assets;
mod automap;
mod gameloop;
mod input;
mod livemap;
mod maploader;
mod scrbuf;
mod sdl_wrapper;
mod utils;

pub use assetloader::*;
pub use assets::*;
pub use automap::*;
pub use gameloop::*;
pub use input::*;
pub use livemap::*;
pub use maploader::*;
pub use scrbuf::*;
pub use sdl_wrapper::*;

// TODO move back to GameLoop ?!
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    // TODO MainMenu,
    // TODO PauseMenu,
    Live,
    Automap,
}
