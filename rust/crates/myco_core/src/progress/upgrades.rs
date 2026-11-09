//! Upgrades (scr_upgrades): levels, bonuses, prices, purchases, the tree's grow animation.
//! Levels and points live in `state.state.upgrade_levels` / `upgrade_points`.

use super::skilltree::{self, upg_def, UpgKind};
use crate::balance::{bal, bal_get, vis};
use crate::sim::{Command, Sim, StepInput};
use crate::state::GameState;
use std::collections::BTreeMap;

/// Why an upgrade cannot be bought (`upg_can_buy` returning false).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuyReject {
    /// locked in the demo build (`demo_upgrades_only`)
    DemoLocked,
    /// no definition with that id
    Unknown,
    /// not available, the root, or a ghost node
    NotBuyable,
    /// the parent is below `req`
    RequirementNotMet,
    /// already at the max level
    MaxLevel,
    NotEnoughPoints {
        need: f64,
    },
}

/// `upg_state(id)`: how the tree screen shows a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UpgState {
    Locked,
    Root,
    Demo,
    Ghost,
    Maxed,
    /// affordable now
    Ready,
    /// owned, next level not affordable
    Owned,
    /// not owned, not affordable
    Short,
}

impl UpgState {
    pub fn as_str(self) -> &'static str {
        match self {
            UpgState::Locked => "locked",
            UpgState::Root => "root",
            UpgState::Demo => "demo",
            UpgState::Ghost => "ghost",
            UpgState::Maxed => "maxed",
            UpgState::Ready => "ready",
            UpgState::Owned => "owned",
            UpgState::Short => "short",
        }
    }
}

fn b(st: &GameState, key: &str) -> f64 {
    bal(st.state.mode, key)
}

// port: upgrades_init
/// No points, only the root owned, the grow animation settled on that.
pub fn init(sim: &mut Sim) {
    sim.state.state.upgrade_points = 0;
    sim.state.state.upgrade_levels = BTreeMap::from([("origin".to_string(), 1)]);
    // global.upgrade_trunks = [], global.upgrade_defs = skilltree::layout()
    let grow = skilltree::layout()
        .iter()
        .map(|d| {
            let owned = upg_is_owned(&sim.state, d.id);
            (d.id.to_string(), if owned { 1.0 } else { 0.0 })
        })
        .collect();
    sim.progress.upgrade_grow = grow;
}

// port: upg_level
pub fn upg_level(st: &GameState, id: &str) -> i64 {
    st.state.upgrade_levels.get(id).copied().unwrap_or(0)
}

// port: upg_max_level
/// `bal(max_key)` for level upgrades, 1 for the others, 0 for an unknown id.
pub fn upg_max_level(st: &GameState, id: &str) -> f64 {
    let Some(def) = upg_def(id) else {
        return 0.0;
    };
    if def.kind == UpgKind::Level {
        return b(st, def.max_key);
    }
    1.0
}

// port: upg_bonus
/// Level times `bal(step_key)` for level upgrades, else 0.
pub fn upg_bonus(st: &GameState, id: &str) -> f64 {
    match upg_def(id) {
        Some(def) if def.kind == UpgKind::Level => {
            let step = b(st, def.step_key);
            upg_level(st, id) as f64 * step
        }
        _ => 0.0,
    }
}

// port: upg_is_owned
pub fn upg_is_owned(st: &GameState, id: &str) -> bool {
    upg_level(st, id) > 0
}

// port: upg_requirement_met
/// The parent is at least at `req` (always for the root, never for an unknown id).
pub fn upg_requirement_met(st: &GameState, id: &str) -> bool {
    let Some(def) = upg_def(id) else {
        return false;
    };
    if def.parent.is_empty() {
        return true;
    }
    upg_level(st, def.parent) as f64 >= def.req
}

// port: upg_cost
/// Price of the next level: `bal(cost_key)` for unlocks; for levels
/// `ceil(base * tier_mult^(tier-1) * growth^level)` with an optional per-upgrade growth
/// (`upg_<id>_cost_growth`); 0 for the root, ghosts and unknown ids.
pub fn upg_cost(st: &GameState, id: &str) -> f64 {
    let Some(def) = upg_def(id) else {
        return 0.0;
    };
    match def.kind {
        UpgKind::Root | UpgKind::Ghost => 0.0,
        UpgKind::Unlock => b(st, def.cost_key),
        UpgKind::Level => {
            let tier_mult =
                crate::gml::power(b(st, "upgrade_cost_tier_mult"), (def.tier - 1.0).max(0.0));
            let growth = bal_get(st.state.mode, &format!("upg_{id}_cost_growth"))
                .unwrap_or_else(|| b(st, "upgrade_cost_growth"));
            let lvl = upg_level(st, id) as f64;
            (b(st, "upgrade_cost_base") * tier_mult * crate::gml::power(growth, lvl)).ceil()
        }
    }
}

/// Upgrades open in the demo build (`upg_demo_locked`'s static list).
const DEMO_OPEN: [&str; 6] = [
    "shot_range",
    "budding_chance",
    "unlock_spore",
    "unlock_predator",
    "unlock_absorption",
    "absorption_yield",
];

fn demo_locked_with(demo_only: f64, id: &str) -> bool {
    if demo_only <= 0.0 {
        return false;
    }
    match upg_def(id) {
        None => false,
        Some(def) if def.kind == UpgKind::Root => false,
        Some(_) => !DEMO_OPEN.contains(&id),
    }
}

// port: upg_demo_locked
/// With `demo_upgrades_only` set, everything but [`DEMO_OPEN`] and the root is locked.
pub fn upg_demo_locked(st: &GameState, id: &str) -> bool {
    demo_locked_with(b(st, "demo_upgrades_only"), id)
}

fn can_buy_with(st: &GameState, id: &str, demo_only: f64) -> Result<(), BuyReject> {
    if demo_locked_with(demo_only, id) {
        return Err(BuyReject::DemoLocked);
    }
    let def = upg_def(id).ok_or(BuyReject::Unknown)?;
    if !def.available || def.kind == UpgKind::Root || def.kind == UpgKind::Ghost {
        return Err(BuyReject::NotBuyable);
    }
    if !upg_requirement_met(st, id) {
        return Err(BuyReject::RequirementNotMet);
    }
    if upg_level(st, id) as f64 >= upg_max_level(st, id) {
        return Err(BuyReject::MaxLevel);
    }
    let need = upg_cost(st, id);
    if (st.state.upgrade_points as f64) >= need {
        Ok(())
    } else {
        Err(BuyReject::NotEnoughPoints { need })
    }
}

// port: upg_can_buy
/// Whether `id` can be bought now, and why not.
pub fn can_buy(st: &GameState, id: &str) -> Result<(), BuyReject> {
    can_buy_with(st, id, b(st, "demo_upgrades_only"))
}

/// `upg_can_buy(id)` as the original's bool.
pub fn upg_can_buy(st: &GameState, id: &str) -> bool {
    can_buy(st, id).is_ok()
}

// port: upg_buy
/// Buy the next level of `id`: pay the price, raise the level. No popup number: the tree
/// screen shows the price itself.
pub fn buy(sim: &mut Sim, id: &str) -> bool {
    if !upg_can_buy(&sim.state, id) {
        return false;
    }
    let cost = upg_cost(&sim.state, id);
    let st = &mut sim.state.state;
    st.upgrade_points = (st.upgrade_points as f64 - cost) as i64;
    let lvl = upg_level(&sim.state, id) + 1;
    sim.state.state.upgrade_levels.insert(id.to_string(), lvl);
    true
}

fn state_with(st: &GameState, id: &str, demo_only: f64) -> UpgState {
    let Some(def) = upg_def(id) else {
        return UpgState::Locked;
    };
    if def.kind == UpgKind::Root {
        return UpgState::Root;
    }
    if demo_locked_with(demo_only, id) {
        return UpgState::Demo;
    }
    if def.kind == UpgKind::Ghost || !def.available {
        return UpgState::Ghost;
    }
    if !upg_requirement_met(st, id) {
        return UpgState::Locked;
    }
    if upg_level(st, id) as f64 >= upg_max_level(st, id) {
        return UpgState::Maxed;
    }
    if st.state.upgrade_points as f64 >= upg_cost(st, id) {
        return UpgState::Ready;
    }
    if upg_is_owned(st, id) {
        return UpgState::Owned;
    }
    UpgState::Short
}

// port: upg_state
pub fn upg_state(st: &GameState, id: &str) -> UpgState {
    state_with(st, id, b(st, "demo_upgrades_only"))
}

/// `tree_hypha_length(id)`: the drawn length when the tree screen has built the hyphae,
/// else the layout fallback.
fn tree_hypha_length(sim: &Sim, id: &str) -> f64 {
    match sim.progress.tree_hypha_len.get(id) {
        Some(&len) => len,
        None => skilltree::tree_hypha_length_fallback(id),
    }
}

// port: upgrades_grow_step
/// Grow each owned node's hypha towards 1 at `tree_grow_speed` px per frame along its
/// length; unowned nodes drop to 0 at once.
pub fn grow_step(sim: &mut Sim) {
    for d in skilltree::layout() {
        let target = if upg_is_owned(&sim.state, d.id) {
            1.0
        } else {
            0.0
        };
        let len = tree_hypha_length(sim, d.id).max(1.0);
        let rate = vis("tree_grow_speed") / len;
        let v = match sim.progress.upgrade_grow.get(d.id) {
            Some(&cur) if target > cur => (cur + rate).min(target),
            _ => target,
        };
        sim.progress.upgrade_grow.insert(d.id.to_string(), v);
    }
}

// port: upg_grow
pub fn upg_grow(sim: &Sim, id: &str) -> f64 {
    sim.progress.upgrade_grow.get(id).copied().unwrap_or(0.0)
}

/// `Command::BuyUpgrade` from the tree screen (obj_hud Begin Step's `upg_buy`).
pub fn apply_commands(sim: &mut Sim, input: &StepInput) {
    for cmd in &input.commands {
        if let Command::BuyUpgrade { id } = cmd {
            buy(sim, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameMode;

    /// The golden rows with `demo_upgrades_only = 1` (the shipped balance has 0, so only this
    /// test can reach the demo branch).
    #[test]
    fn demo_rows_match_love() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../golden/data/progress.json"
        );
        let g: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut st = GameState::default();
        let mut n = 0;
        for (mode, rows) in g["states"].as_object().unwrap() {
            st.state.mode = if mode == "survival" {
                GameMode::Survival
            } else {
                GameMode::Exploration
            };
            for d in skilltree::layout() {
                for r in rows[d.id].as_array().unwrap() {
                    if r["demo"].as_f64().unwrap() == 0.0 {
                        continue;
                    }
                    let plv = r["parent_level"].as_f64().unwrap() as i64;
                    let own = r["own"].as_f64().unwrap() as i64;
                    st.state.upgrade_levels = BTreeMap::from([("origin".to_string(), 1)]);
                    st.state.upgrade_levels.insert(d.id.into(), own);
                    if !d.parent.is_empty() && d.parent != "origin" {
                        st.state.upgrade_levels.insert(d.parent.into(), plv);
                    }
                    if d.parent == "origin" {
                        st.state
                            .upgrade_levels
                            .insert("origin".into(), if plv == 0 { 0 } else { 1 });
                    }
                    st.state.upgrade_points = r["points"].as_f64().unwrap() as i64;
                    assert_eq!(
                        can_buy_with(&st, d.id, 1.0).is_ok(),
                        r["can_buy"].as_bool().unwrap(),
                        "{r}"
                    );
                    assert_eq!(
                        state_with(&st, d.id, 1.0).as_str(),
                        r["state"].as_str().unwrap(),
                        "{r}"
                    );
                    assert_eq!(
                        demo_locked_with(1.0, d.id),
                        r["demo_locked"].as_bool().unwrap(),
                        "{r}"
                    );
                    n += 1;
                }
            }
        }
        assert!(n > 400, "{n}");
        assert_eq!(
            can_buy_with(&st, "unlock_tree", 1.0),
            Err(BuyReject::DemoLocked)
        );
    }
}
