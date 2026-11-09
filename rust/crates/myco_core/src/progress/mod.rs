//! Upgrades, skill tree, tasks, goals, tutorial (Task 11).

pub mod tutorial;

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime progress state (`global.upgrade_grow`, tasks/tutorial animation). Task 11 fills it.
#[derive(Clone, Debug, Default)]
pub struct ProgressRt {}

/// The logic part of obj_hud Step_1 (Begin Step), e.g. `water_counter_step` (Task 5). The
/// HUD itself (layout, menus, Escape handling) is the app's (Task 17).
pub fn hud_begin_step(_sim: &mut Sim, _input: &StepInput) {}

/// `load_game`'s progress part (Task 11): `upgrade_grow` from owned upgrades, `tasks_init`
/// then the saved task state, tutorial state.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s progress part (Task 11): upgrades, tutorial and tasks into `out.state`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
