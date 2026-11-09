//! Mycelium (obj_mycelium_controller): shots, nodes, growth, spores. Task 8 ports it.
//! Changed growths are marked dirty here for the app to rebake (ruling R9).

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime mycelium (`obj_mycelium_controller` variables). Task 8 fills it.
#[derive(Clone, Debug, Default)]
pub struct MyceliumRt {}

/// obj_mycelium_controller Create_0 (Task 8).
pub fn create(_sim: &mut Sim) {}

/// obj_mycelium_controller Step_0 (Task 8). Reads its `Command`s (quick node, mushroom
/// placement) from `input.commands`, where the original reads the mouse.
pub fn step(_sim: &mut Sim, _input: &StepInput) {}

/// `load_game`'s mycelium part (Task 8): rebuild growths/knots/grid from `sim.state.mycelium`.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s mycelium part (Task 8): write growths and knots into `out.mycelium`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}

/// `mycelium_extreme_tip(dir)` (Task 8): the growth point furthest in direction `dir` (>= 0
/// right, < 0 left), or None without growths. Neutral (no growths) until Task 8 ports it; the
/// yellow cave (`world::caves::cave_yellow_pick`) asks it.
pub fn mycelium_extreme_tip(_sim: &Sim, _dir: f64) -> Option<(f64, f64)> {
    None
}

/// `myc_glow_set(on)` (Task 8, obj_mycelium_controller Create_0 method): the purple perk's
/// mycelium glow. `world::caves::cave_take` turns it on for the first purple perk; Task 8
/// fills the body.
pub fn myc_glow_set(_sim: &mut Sim, _on: bool) {}
