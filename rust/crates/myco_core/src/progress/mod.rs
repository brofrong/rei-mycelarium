//! Upgrades, skill tree, tasks, goals, tutorial (Task 11).

pub mod tutorial;

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime progress state (`global.upgrade_grow`, tasks/tutorial animation). Task 11 fills it.
#[derive(Clone, Debug, Default)]
pub struct ProgressRt {}

/// The logic part of obj_hud Step_1 (Begin Step): `water_counter_step` (Task 5). The HUD
/// itself (layout, menus, Escape handling) is the app's (Task 17).
pub fn hud_begin_step(sim: &mut Sim, _input: &StepInput) {
    crate::world::water_counter_step(sim);
}

/// `tasks_stat_add(key, n)` (Task 11): logic reports task statistics here (water connected,
/// food eaten, ...). Until then: nothing.
pub fn tasks_stat_add(_sim: &mut Sim, _key: &str, _n: f64) {}

/// `load_game`'s progress part (Task 11): `upgrade_grow` from owned upgrades, `tasks_init`
/// then the saved task state, tutorial state.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s progress part (Task 11): upgrades, tutorial and tasks into `out.state`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
