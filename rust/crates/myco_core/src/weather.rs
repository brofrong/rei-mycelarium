//! Weather (scr_weather: rain, wind). Task 12 ports it.

use crate::sim::Sim;
use crate::state::GameState;

/// Runtime weather state. Task 12 fills it.
#[derive(Clone, Debug, Default)]
pub struct WeatherRt {}

/// `weather_init` (Task 12); called by `game_init` and `load_game`.
pub fn init(_sim: &mut Sim) {}

/// `weather_step(vx, vy, vw, vh)` (Task 12); called from obj_forest_controller Step_0.
pub fn step(_sim: &mut Sim, _vx: f64, _vy: f64, _vw: f64, _vh: f64) {}

/// `save_game`'s weather part (Task 12), if it keeps runtime state outside `GameState`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
