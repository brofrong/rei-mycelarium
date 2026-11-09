//! Consumptors (parasitism): mushrooms that eat a tree or stump (`m.host`) and pay out food
//! when it is gone.

use super::{mushroom_id, MushroomKind};
use crate::events::{ResourceKind, SimEvent};
use crate::forest::{stump_parts, HostId};
use crate::gml;
use crate::mycelium::source;
use crate::progress::tasks;
use crate::progress::upgrades::upg_bonus;
use crate::sim::Sim;
use crate::world::perks::perk_time_mult;

/// `consume_info(host)` (`___struct___86/87`): slots, minutes and food of a host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsumeInfo {
    pub slots: f64,
    pub minutes: f64,
    pub food: f64,
}

// port: consume_info
/// Slots, minutes and food of a host: a stump by its part, any tree alike.
pub fn consume_info(sim: &Sim, host: HostId) -> ConsumeInfo {
    if let HostId::Stump(uid) = host {
        if let Some(s) = sim.forest.stump(uid) {
            let part = &stump_parts()[s.part];
            return ConsumeInfo {
                slots: part.slots.len() as f64,
                minutes: sim.bal(&format!("consumptor_{}_minutes", part.consume)),
                food: sim.bal(&format!("consumptor_{}_food", part.consume)),
            };
        }
    }
    ConsumeInfo {
        slots: sim.bal("consumptor_tree_slots"),
        minutes: sim.bal("consumptor_tree_minutes"),
        food: sim.bal("consumptor_tree_food"),
    }
}

// port: consume_frames
/// Frames one consumptor needs to eat the host alone.
pub fn consume_frames(sim: &Sim, host: HostId) -> f64 {
    let minutes = consume_info(sim, host).minutes;
    let d = 1.0 + upg_bonus(&sim.state, "parasitism_speed");
    minutes * 60.0 * sim.fps / d * perk_time_mult(&sim.state)
}

// port: consume_food
/// Food paid out when the host is eaten.
pub fn consume_food(sim: &Sim, host: HostId) -> f64 {
    let food = consume_info(sim, host).food;
    let k = 1.0 + upg_bonus(&sim.state, "parasitism_yield");
    gml::round(food * k)
}

// port: consumptors_on
/// Indices of the live (not stolen) consumptors on a host.
pub fn consumptors_on(sim: &Sim, host: HostId) -> Vec<usize> {
    sim.state
        .mushrooms
        .iter()
        .enumerate()
        .filter(|(_, m)| {
            m.kind == MushroomKind::Consumptor.code() && m.live.host == Some(host) && !m.stolen
        })
        .map(|(i, _)| i)
        .collect()
}

// port: consumptor_variant
/// The sheet variant facing `side`.
pub fn consumptor_variant(side: f64) -> i64 {
    if side < 0.0 {
        0
    } else {
        1
    }
}

/// The host of mushroom `i` (`m.host`).
pub fn mushroom_host(sim: &Sim, i: usize) -> Option<HostId> {
    sim.state.mushrooms[i].live.host
}

/// x of mushroom `i`'s host while it stands (`m.host.x`).
pub fn host_x(sim: &Sim, i: usize) -> Option<f64> {
    mushroom_host(sim, i).and_then(|h| sim.forest.host_x(h))
}

/// The host's consumption progress (`host.consume ?? 0`) and whether it is eaten.
pub fn host_consume(sim: &Sim, host: HostId) -> Option<(f64, bool)> {
    match host {
        HostId::Tree(u) => sim.forest.tree(u).map(|t| (t.consume, t.consumed)),
        HostId::Stump(u) => sim.forest.stump(u).map(|s| (s.consume, s.consumed)),
    }
}

/// Set a host's consumption progress and eaten flag.
pub(crate) fn set_host_consume(sim: &mut Sim, host: HostId, consume: f64, consumed: bool) {
    match host {
        HostId::Tree(u) => {
            if let Some(t) = sim.forest.tree_mut(u) {
                t.consume = consume;
                t.consumed = consumed;
            }
        }
        HostId::Stump(u) => {
            if let Some(s) = sim.forest.stump_mut(u) {
                s.consume = consume;
                s.consumed = consumed;
            }
        }
    }
}

/// Set a new consumptor's host, side and variant (`myc_place_mushroom`).
pub fn set_consumptor(sim: &mut Sim, index: usize, host: Option<HostId>, side: f64, variant: i64) {
    let m = &mut sim.state.mushrooms[index];
    m.live.host = host;
    m.side = side;
    m.variant = variant;
}

// port: consumptor_progress
/// Hover-bar progress of a consumptor: its host's consumption (`None` when stolen or hostless).
pub fn consumptor_progress(sim: &Sim, i: usize) -> Option<f64> {
    let m = &sim.state.mushrooms[i];
    if m.stolen {
        return None;
    }
    let host = m.live.host?;
    let c = host_consume(sim, host).map_or(0.0, |(c, _)| c);
    Some(gml::clamp(c, 0.0, 1.0))
}

// port: consume_finish
/// The host is eaten: pay out its food (one `Resource{Food, +n}` at the first consumptor on
/// it), remove its consumptors (and, for a tree, its tree mushroom) and the host itself.
pub fn consume_finish(sim: &mut Sim, host: HostId) {
    let food = consume_food(sim, host);
    let first = consumptors_on(sim, host).first().copied();
    let (ex, ey, src) = match first {
        Some(i) => {
            let m = &sim.state.mushrooms[i];
            (m.x, m.y, source::MUSHROOM | mushroom_id(sim, i))
        }
        None => (
            sim.forest.host_x(host).unwrap_or(0.0),
            sim.forest.surface_height,
            source::MUSHROOM,
        ),
    };
    sim.food_add(food);
    sim.emit(SimEvent::Resource {
        kind: ResourceKind::Food,
        delta: food,
        x: ex,
        y: ey,
        source: src,
    });
    tasks::stat_add(sim, "food", food);
    let is_tree = matches!(host, HostId::Tree(_));
    if is_tree {
        tasks::stat_add(sim, "ach_trees_eaten", 1.0);
    }
    let tx = sim.forest.host_x(host).unwrap_or(0.0);
    let off = crate::balance::vis("tree_mushroom_offset_x");
    let mut i = sim.state.mushrooms.len();
    while i > 0 {
        i -= 1;
        let m = &sim.state.mushrooms[i];
        let kill = if m.kind == MushroomKind::Consumptor.code()
            && m.live.host == Some(host)
            && !m.stolen
        {
            true
        } else {
            is_tree
                && m.kind == MushroomKind::Tree.code()
                && !m.stolen
                && (m.x - off - tx).abs() < 2.0
        };
        if kill {
            sim.state.mushrooms.remove(i);
            crate::life::animals_reindex_after_removal(sim, i as i64);
        }
    }
    crate::life::animals_rebind(sim);
    sim.forest.remove_host(&mut sim.rng, host);
}
