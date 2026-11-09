//! Mycelarium simulation core. Logic mirrors the LÖVE port (`game/myco/gen/*.lua`), which is the
//! reference; numbers are `f64` like GML doubles.

pub mod balance;
pub mod camera;
pub mod events;
pub mod forest;
pub mod game;
pub mod gml;
pub mod life;
pub mod locale;
pub mod mushrooms;
pub mod mycelium;
pub mod progress;
pub mod rng;
pub mod save;
pub mod sim;
pub mod state;
pub mod weather;
pub mod world;
