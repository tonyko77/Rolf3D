//! ROLF3D - a Rust implementation of the WOLF3D raycasting engine :)
//! Main library.

mod assetloader;
mod assets;
mod automap;
mod gameloop;
mod input;
mod livemap;
mod mapcell;
mod picdict;
mod raycaster;
mod scrbuf;
mod sdl_wrapper;
mod status;
mod utils;

pub use assetloader::*;
pub use assets::*;
pub use automap::*;
pub use gameloop::*;
pub use input::*;
pub use livemap::*;
pub use mapcell::*;
pub use picdict::*;
pub use raycaster::*;
pub use scrbuf::*;
pub use sdl_wrapper::*;
pub use status::*;
pub use utils::*;

pub const EPSILON: f64 = 0.001;

// TODO move back to GameLoop ?!
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    // TODO MainMenu,
    // TODO PauseMenu,
    Live,
    Automap,
}
