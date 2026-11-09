//! Animals and other life (obj_life, scr_life*): hedgehog, squirrel, crow, boar, worm, leaves,
//! predator, drops. Task 10 ports it.

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime life state (`global.animals`, `global.boars`, ...). Task 10 fills it.
#[derive(Clone, Debug, Default)]
pub struct LifeRt {}

/// obj_life Create_0 (`life_init`, Task 10).
pub fn create(_sim: &mut Sim) {}

/// obj_life Step_0 (`intro_step`, `life_step`, Task 10).
pub fn step(_sim: &mut Sim, _input: &StepInput) {}

/// `life_animal_timers_init` (Task 10); called by `game_mode_start`.
pub fn animal_timers_init(_sim: &mut Sim) {}

/// `load_game`'s life part (Task 10): clear animals, restore timers, `life_far_init`.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s life part (Task 10): write animal timers into `out.state.timers`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
