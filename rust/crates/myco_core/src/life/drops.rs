//! Water drops falling from tree mushrooms (scr_life `drops_step`, `tree_drop_*`).

use crate::events::{ResourceKind, SimEvent};
use crate::progress::tasks;
use crate::progress::upgrades::upg_bonus;
use crate::sim::Sim;
use crate::world::perks::perk_time_mult;

/// A water drop falling from a tree mushroom (`global.drops[i]`, `___struct___63`).
#[derive(Clone, Debug, PartialEq)]
pub struct Drop {
    pub x: f64,
    pub y: f64,
    pub target_y: f64,
    pub speed: f64,
    /// `Resource::source` of the mushroom that dropped it (`source::MUSHROOM | id`)
    pub source: u64,
}

// port: tree_drop_period
/// Frames between two drops of a tree mushroom.
pub fn tree_drop_period(sim: &Sim) -> f64 {
    let rate = 1.0 + upg_bonus(&sim.state, "tree_drop_rate");
    (sim.bal("tree_drop_interval") / rate * perk_time_mult(&sim.state)).max(30.0)
}

// port: tree_drop_water
/// Water per drop.
pub fn tree_drop_water(sim: &Sim) -> f64 {
    let y = 1.0 + upg_bonus(&sim.state, "tree_water_yield");
    sim.bal("tree_water_per_drop") * y
}

// port: drops_step
/// Drops fall; a landed drop adds its water (and emits `Resource{Water, +n}` at the landing
/// point with its tree mushroom as the source).
pub fn drops_step(sim: &mut Sim) {
    let drops = std::mem::take(&mut sim.life.drops);
    let mut keep = Vec::with_capacity(drops.len());
    for mut d in drops {
        d.speed = (d.speed + 0.22).min(7.0);
        d.y += d.speed;
        if d.y >= d.target_y {
            let w = tree_drop_water(sim);
            sim.state.state.water_free += w;
            sim.emit(SimEvent::Resource {
                kind: ResourceKind::Water,
                delta: w,
                x: d.x,
                y: d.target_y,
                source: d.source,
            });
            tasks::stat_add(sim, "ach_tree_water", w);
        } else {
            keep.push(d);
        }
    }
    sim.life.drops = keep;
}
