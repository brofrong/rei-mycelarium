//! Forest state (obj_forest_controller): surface chunks with trees, stumps, bushes; generation
//! and queries live here in the core (rulings R5/R10, Task 5); drawing is Task 16's.

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime forest (`obj_forest_controller` variables). Task 5 fills it.
#[derive(Clone, Debug, Default)]
pub struct ForestRt {}

/// obj_forest_controller Create_0 (Task 5).
pub fn create(_sim: &mut Sim) {}

/// obj_forest_controller Step_0 (Task 5): camera movement, `weather::step`, ...
pub fn step(_sim: &mut Sim, _input: &StepInput) {}

/// `load_game`'s forest part (Task 5): rebuild surface chunks / start stump from `sim.state.forest`.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s forest part (Task 5): write runtime chunks and camera into `out.forest`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
