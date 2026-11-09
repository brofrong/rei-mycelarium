//! Balance (`bal`), visual (`vis`) and worldgen (`wld`) tables, game mode, price multipliers.
//!
//! The default tables are generated from the LÖVE sources by `tools/gen_balance.py` into
//! `balance_tables.rs`. `global.balance` depends on the game mode (`game_mode_apply` rescales
//! costs and animal intervals), so `bal` takes the mode; both mode tables are built once.
//! `vis`/`wld` never change at run time. An unknown key panics in debug builds (GML would
//! yield `undefined`); release builds return 0.

use crate::state::{GameMode, GameState};
use serde_json::Value;
use std::sync::OnceLock;

#[path = "balance_tables.rs"]
#[rustfmt::skip]
mod tables;

/// A `vis`/`wld` entry: a number (`true` is 1) or an array (level tables, `[min, max]` ranges).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TableVal {
    Num(f64),
    Arr(&'static [f64]),
}

fn index_of<T>(table: &[(&str, T)], key: &str) -> Option<usize> {
    table.binary_search_by(|(k, _)| (*k).cmp(key)).ok()
}

// ---------------------------------------------------------------------------------------------
// bal

// port: balance_defaults
/// `balance_defaults()`: the raw balance struct before any game mode is applied, sorted by key.
pub fn balance_defaults() -> &'static [(&'static str, f64)] {
    tables::BAL_DEFAULTS
}

/// One `global.balance`: values aligned with [`balance_defaults`] keys.
#[derive(Clone, Debug, PartialEq)]
pub struct Balance {
    vals: Vec<f64>,
}

impl Balance {
    // port: balance_init
    /// `balance_init`: `global.balance = balance_defaults()`.
    pub fn defaults() -> Balance {
        Balance {
            vals: tables::BAL_DEFAULTS.iter().map(|(_, v)| *v).collect(),
        }
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        index_of(tables::BAL_DEFAULTS, key).map(|i| self.vals[i])
    }

    fn set(&mut self, key: &str, v: f64) {
        let i = index_of(tables::BAL_DEFAULTS, key)
            .unwrap_or_else(|| panic!("unknown balance key {key:?}"));
        self.vals[i] = v;
    }

    fn at(&self, key: &str) -> f64 {
        self.get(key)
            .unwrap_or_else(|| panic!("unknown balance key {key:?}"))
    }

    // port: game_mode_apply
    /// `game_mode_apply(mode)`: fresh defaults rescaled for the mode. Survival raises every
    /// `*_mushroom_cost` / `*_mushroom_water_cost` (except the digger's) by 1.7 rounded up,
    /// the digger's costs ×3 and a few start values; exploration doubles the digger's costs
    /// and stretches animal spawn intervals ×1.7.
    pub fn for_mode(mode: GameMode) -> Balance {
        let mut b = Balance::defaults();
        if mode == GameMode::Survival {
            let k = 1.7;
            for (i, (n, _)) in tables::BAL_DEFAULTS.iter().enumerate() {
                // string_pos("digger", n) == 0, then the suffix tests on character lengths
                if n.contains("digger") {
                    continue;
                }
                let len = n.chars().count();
                let hit = (len > 14 && n.ends_with("_mushroom_cost"))
                    || (len > 20 && n.ends_with("_mushroom_water_cost"));
                if hit {
                    b.vals[i] = (b.vals[i] * k).ceil();
                }
            }
            b.set(
                "digger_mushroom_cost",
                (b.at("digger_mushroom_cost") * 3.0).ceil(),
            );
            b.set(
                "digger_water_cost_min",
                (b.at("digger_water_cost_min") * 3.0).ceil(),
            );
            b.set(
                "digger_water_cost_max",
                (b.at("digger_water_cost_max") * 3.0).ceil(),
            );
            b.set("base_shot_water_cost", b.at("base_shot_water_cost") * k);
            b.set(
                "deepsoil_shot_food_cost",
                (b.at("deepsoil_shot_food_cost") * k).ceil(),
            );
            b.set("base_node_food_cost", 8.0);
            b.set("start_food", b.at("start_food") + 20.0);
            b.set("start_water", b.at("start_water") + 20.0);
            b.set("upgrade_cost_growth", 1.7);
            return b;
        }
        b.set(
            "digger_mushroom_cost",
            (b.at("digger_mushroom_cost") * 2.0).ceil(),
        );
        b.set(
            "digger_water_cost_min",
            (b.at("digger_water_cost_min") * 2.0).ceil(),
        );
        b.set(
            "digger_water_cost_max",
            (b.at("digger_water_cost_max") * 2.0).ceil(),
        );
        let k2 = 1.7;
        for key in [
            "hedgehog_spawn_interval",
            "hedgehog_first_delay_min",
            "hedgehog_first_delay_max",
            "boar_spawn_interval",
            "boar_first_delay",
            "squirrel_spawn_interval",
            "crow_spawn_interval",
            "experience_crow_interval",
            "experience_crow_offset",
        ] {
            b.set(key, b.at(key) * k2);
        }
        b
    }
}

/// `global.balance` for a game mode (built once per mode).
pub fn balance(mode: GameMode) -> &'static Balance {
    static TABLES: OnceLock<[Balance; 2]> = OnceLock::new();
    let t = TABLES.get_or_init(|| {
        [
            Balance::for_mode(GameMode::Exploration),
            Balance::for_mode(GameMode::Survival),
        ]
    });
    match mode {
        GameMode::Exploration => &t[0],
        GameMode::Survival => &t[1],
    }
}

// port: bal
/// `bal(key)` under the given game mode. Unknown key: panic in debug, 0 in release.
pub fn bal(mode: GameMode, key: &str) -> f64 {
    match balance(mode).get(key) {
        Some(v) => v,
        None => missing("balance", key),
    }
}

/// `variable_struct_get(global.balance, key)` where the game tolerates `undefined`
/// (for example the optional `upg_<id>_cost_growth` overrides).
pub fn bal_get(mode: GameMode, key: &str) -> Option<f64> {
    balance(mode).get(key)
}

fn missing(table: &str, key: &str) -> f64 {
    if cfg!(debug_assertions) {
        panic!("unknown {table} key {key:?}");
    }
    0.0
}

// ---------------------------------------------------------------------------------------------
// vis / wld

fn num(table_name: &str, t: &'static [(&'static str, TableVal)], key: &str) -> f64 {
    match index_of(t, key).map(|i| t[i].1) {
        Some(TableVal::Num(v)) => v,
        Some(TableVal::Arr(_)) => {
            if cfg!(debug_assertions) {
                panic!("{table_name} key {key:?} is an array: use the _arr accessor");
            }
            0.0
        }
        None => missing(table_name, key),
    }
}

fn arr(table_name: &str, t: &'static [(&'static str, TableVal)], key: &str) -> &'static [f64] {
    match index_of(t, key).map(|i| t[i].1) {
        Some(TableVal::Arr(a)) => a,
        Some(TableVal::Num(_)) => {
            if cfg!(debug_assertions) {
                panic!("{table_name} key {key:?} is a number, not an array");
            }
            &[]
        }
        None => {
            missing(table_name, key);
            &[]
        }
    }
}

// port: visual_defaults
// port: visual_init
/// `global.visual = visual_defaults()`: never modified at run time, so it is the generated table.
fn visual() -> &'static [(&'static str, TableVal)] {
    tables::VIS_DEFAULTS
}

// port: worldgen_defaults
// port: worldgen_init
/// `global.worldgen = worldgen_defaults()`: never modified at run time.
fn worldgen() -> &'static [(&'static str, TableVal)] {
    tables::WLD_DEFAULTS
}

// port: vis
/// `vis(key)` for a numeric entry (`true` reads as 1).
pub fn vis(key: &str) -> f64 {
    num("visual", visual(), key)
}

/// `vis(key)` for an array entry (`stinky_*_levels`).
pub fn vis_arr(key: &str) -> &'static [f64] {
    arr("visual", visual(), key)
}

pub fn vis_get(key: &str) -> Option<TableVal> {
    index_of(visual(), key).map(|i| visual()[i].1)
}

pub fn vis_table_len() -> usize {
    visual().len()
}

// port: wld
/// `wld(key)` for a numeric entry.
pub fn wld(key: &str) -> f64 {
    num("worldgen", worldgen(), key)
}

/// `wld(key)` for an array entry (`caves_*_x` / `caves_*_depth` / `bones_big_*` ranges).
pub fn wld_arr(key: &str) -> &'static [f64] {
    arr("worldgen", worldgen(), key)
}

pub fn wld_get(key: &str) -> Option<TableVal> {
    index_of(worldgen(), key).map(|i| worldgen()[i].1)
}

pub fn wld_table_len() -> usize {
    worldgen().len()
}

// ---------------------------------------------------------------------------------------------
// game mode and price multipliers

// port: game_mode
/// `game_mode()`: the mode of the running game (`global.game_mode`, saved as `state.mode`).
pub fn game_mode(state: &GameState) -> GameMode {
    state.state.mode
}

/// `gm.bool`: numbers are true above 0.5; `undefined` (missing) is false.
pub(crate) fn gml_truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|x| x > 0.5),
        Some(_) => true,
    }
}

// port: perk_cost_mult
/// `perk_cost_mult()`: the green cave perk discounts mushroom costs.
pub fn perk_cost_mult(state: &GameState) -> f64 {
    // perk_has("green") (Task 6 ports perk_has; it reads global.cave_perks the same way)
    if gml_truthy(state.state.cave_perks.get("green")) {
        return 1.0 - bal(game_mode(state), "perk_green_discount");
    }
    1.0
}

// port: experience_price_mult
/// `experience_price_mult()`: `experience_cost_growth` to the power of the experience
/// mushrooms (type 6) in play that were not stolen.
pub fn experience_price_mult(state: &GameState) -> f64 {
    let n = state
        .mushrooms
        .iter()
        .filter(|m| m.kind == 6 && !m.stolen)
        .count();
    bal(game_mode(state), "experience_cost_growth").powf(n as f64)
}
