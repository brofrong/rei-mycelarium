//! Balance tables against the LÖVE port (`rust/golden/data/balance.json`, from
//! `rust/golden/scripts/balance.lua`).

mod common;

use common::{f, golden};
use myco_core::balance::{
    self, bal, bal_get, experience_price_mult, game_mode, perk_cost_mult, vis, vis_arr, wld,
    wld_arr, TableVal,
};
use myco_core::state::{GameMode, GameState, MushroomSave};
use serde_json::Value;

fn num_or_bool(v: &Value) -> Option<f64> {
    match v {
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Number(_) => Some(f(v)),
        _ => None,
    }
}

fn arr(v: &Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(f).collect()
}

#[test]
fn balance_matches_love() {
    let g = golden("balance");
    let defaults = g["defaults"].as_object().unwrap();
    // raw balance_defaults(): same keys, same values
    let ours = balance::balance_defaults();
    assert_eq!(ours.len(), defaults.len(), "balance key count");
    for (k, v) in ours {
        assert_eq!(*v, f(&defaults[*k]), "balance_defaults.{k}");
    }
    // global.balance after game_mode_apply, both modes, every key (bit-exact)
    for (mode, name) in [
        (GameMode::Exploration, "exploration"),
        (GameMode::Survival, "survival"),
    ] {
        for (k, v) in g[name].as_object().unwrap() {
            assert_eq!(bal(mode, k), f(v), "{name} bal.{k}");
        }
    }
    // every literal key the game passes to bal/vis/wld resolves to LÖVE's value
    for k in g["used"]["bal"].as_array().unwrap() {
        let k = k.as_str().unwrap();
        for (mode, name) in [
            (GameMode::Exploration, "exploration"),
            (GameMode::Survival, "survival"),
        ] {
            assert_eq!(
                bal_get(mode, k),
                Some(f(&g[name][k])),
                "used bal.{k} ({name})"
            );
        }
    }
    for (table, getter_num, getter_arr, name) in [
        (
            "visual",
            vis as fn(&str) -> f64,
            vis_arr as fn(&str) -> &'static [f64],
            "vis",
        ),
        ("worldgen", wld, wld_arr, "wld"),
    ] {
        let t = g[table].as_object().unwrap();
        for (k, v) in t {
            match num_or_bool(v) {
                Some(x) => assert_eq!(getter_num(k), x, "{name}.{k}"),
                None => assert_eq!(getter_arr(k), arr(v).as_slice(), "{name}.{k}"),
            }
        }
        for k in g["used"][name].as_array().unwrap() {
            assert!(
                t.contains_key(k.as_str().unwrap()),
                "used {name}.{k} missing in LÖVE"
            );
        }
    }
    assert_eq!(
        balance::vis_table_len(),
        g["visual"].as_object().unwrap().len()
    );
    assert_eq!(
        balance::wld_table_len(),
        g["worldgen"].as_object().unwrap().len()
    );
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "unknown balance key")]
fn unknown_bal_key_panics_in_debug() {
    bal(GameMode::Exploration, "no_such_balance_key");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "unknown visual key")]
fn unknown_vis_key_panics_in_debug() {
    vis("no_such_visual_key");
}

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "array")]
fn array_key_read_as_number_panics_in_debug() {
    wld("caves_blue_first_x");
}

#[test]
fn optional_lookup_does_not_panic() {
    assert_eq!(
        bal_get(GameMode::Exploration, "upg_nothing_cost_growth"),
        None
    );
    assert!(matches!(
        balance::vis_get("stinky_cone_deg_levels"),
        Some(TableVal::Arr(_))
    ));
    assert_eq!(balance::wld_get("nope"), None);
}

fn mushroom(kind: i64, stolen: bool) -> MushroomSave {
    MushroomSave {
        kind,
        stolen,
        ..MushroomSave::default()
    }
}

#[test]
fn game_mode_perk_and_experience_price() {
    let mut s = GameState::default();
    assert_eq!(game_mode(&s), GameMode::Exploration);
    assert_eq!(perk_cost_mult(&s), 1.0);
    // green perk: number above 0.5 or true counts, like gm.bool
    s.state.cave_perks.insert("green".into(), Value::from(1));
    assert_eq!(
        perk_cost_mult(&s),
        1.0 - bal(GameMode::Exploration, "perk_green_discount")
    );
    s.state.cave_perks.insert("green".into(), Value::from(0.5));
    assert_eq!(perk_cost_mult(&s), 1.0);
    s.state.cave_perks.insert("green".into(), Value::Bool(true));
    assert_eq!(perk_cost_mult(&s), 0.75);

    // experience mushrooms (type 6) not stolen raise the price
    assert_eq!(experience_price_mult(&s), 1.0);
    s.state.mode = GameMode::Survival;
    s.mushrooms = vec![
        mushroom(6, false),
        mushroom(6, true),
        mushroom(6, false),
        mushroom(2, false),
    ];
    assert_eq!(game_mode(&s), GameMode::Survival);
    let g = bal(GameMode::Survival, "experience_cost_growth");
    assert_eq!(experience_price_mult(&s), g.powf(2.0));
}
