//! Predator mushroom helpers (scr_life `predator_*`): catch period, yield, hover progress.

use crate::gml;
use crate::progress::upgrades::upg_bonus;
use crate::sim::Sim;
use crate::world::perks::perk_time_mult;

// port: predator_period
/// Frames between two catches of a predator mushroom.
pub fn predator_period(sim: &Sim) -> f64 {
    let d = 1.0 + upg_bonus(&sim.state, "predator_digestion");
    (sim.bal("predator_catch_interval") / d * perk_time_mult(&sim.state)).max(60.0)
}

// port: predator_yield
/// Food per catch.
pub fn predator_yield(sim: &Sim) -> f64 {
    (sim.bal("predator_food_per_fly") + upg_bonus(&sim.state, "predator_yield")).floor()
}

// port: predator_progress
/// Hover-bar progress of predator mushroom `i` (`None` while carried or stolen).
pub fn predator_progress(sim: &Sim, i: usize) -> Option<f64> {
    let m = &sim.state.mushrooms[i];
    if m.carried_by == -1 && !m.stolen {
        return Some(gml::clamp(1.0 - m.timer / predator_period(sim), 0.0, 1.0));
    }
    None
}
