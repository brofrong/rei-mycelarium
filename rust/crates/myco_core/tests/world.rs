//! World (Task 5) against the LÖVE port: hashes, art-derived data and soil rows, chunk
//! generation for seeds 31607 and 7, tile/water queries, water and food mutations, the
//! survival layout and the Exploration fixture loaded through `Sim::from_save`.
//! Golden topics: world_hash, world_assets, world_s31607, world_s7, world_survival,
//! world_fixture (captured with the Task 6/7 features neutralized, see
//! rust/golden/scripts/world_lib.lua: their hooks are neutral until those tasks port them).
mod common;

use common::{f, golden, i};
use myco_core::sim::Sim;
use myco_core::state::{GameMode, GameState, WaterSave};
use myco_core::world::{self, food, hash, lakes, soil, Tile, WorldRt};
use serde_json::Value;

/// Decode `[[value, count], ...]`.
fn unrle(v: &Value) -> Vec<f64> {
    let mut out = Vec::new();
    for run in v.as_array().unwrap() {
        let r = run.as_array().unwrap();
        for _ in 0..i(&r[1]) {
            out.push(f(&r[0]));
        }
    }
    out
}

/// Structural JSON equality: numbers compared as f64, the golden "undefined" equals null,
/// object key order ignored.
fn json_eq(path: &str, got: &Value, want: &Value) {
    match (got, want) {
        (Value::Number(a), Value::Number(b)) => {
            assert_eq!(a.as_f64().unwrap(), b.as_f64().unwrap(), "{path}")
        }
        (Value::Null, Value::String(s)) | (Value::String(s), Value::Null) if s == "undefined" => {}
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: length\n got {got}\nwant {want}");
            for (k, (x, y)) in a.iter().zip(b).enumerate() {
                json_eq(&format!("{path}[{k}]"), x, y);
            }
        }
        // LÖVE writes an empty struct as []
        (Value::Object(a), Value::Array(b)) if a.is_empty() && b.is_empty() => {}
        (Value::Object(a), Value::Object(b)) => {
            let mut ka: Vec<_> = a.keys().collect();
            let mut kb: Vec<_> = b.keys().collect();
            ka.sort();
            kb.sort();
            assert_eq!(ka, kb, "{path}: keys");
            for (k, x) in a {
                json_eq(&format!("{path}.{k}"), x, &b[k]);
            }
        }
        _ => assert_eq!(got, want, "{path}"),
    }
}

fn to_json<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}

#[test]
fn hash_matches_love() {
    let g = golden("world_hash");
    let rows = g["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 10_000);
    for r in rows {
        let (seed, a, b, c) = (i(&r[0]), f(&r[1]), f(&r[2]), f(&r[3]));
        assert_eq!(
            hash::world_hash01(seed, a, b, c),
            f(&r[4]),
            "world_hash01 {r}"
        );
        assert_eq!(hash::cave_hash(a, b, c), f(&r[5]), "cave_hash {r}");
    }
}

#[test]
fn art_data_and_soil_rows_match_love() {
    let g = golden("world_assets");
    // lake masks
    let kinds = lakes::lake_kinds();
    for (k, want) in g["lake_kinds"].as_array().unwrap().iter().enumerate() {
        let kd = &kinds[k];
        assert_eq!(kd.left, f(&want["left"]));
        assert_eq!(kd.width, f(&want["width"]));
        assert_eq!(kd.height, f(&want["height"]));
        assert_eq!(kd.top, f(&want["top"]));
        assert_eq!(kd.bottom, f(&want["bottom"]));
    }
    let mut n = 0;
    for (k, kd) in kinds.iter().enumerate().take(3) {
        let mut y = -2.0;
        while y <= kd.height + 1.0 {
            let mut x = -2.0;
            while x <= kd.width + 1.0 {
                let want = i(&g["solid"][n]) == 1;
                assert_eq!(
                    lakes::lake_solid_at(kd, x, y),
                    want,
                    "solid kind {k} ({x},{y})"
                );
                n += 1;
                x += 5.0;
            }
            y += 3.0;
        }
    }
    assert_eq!(n, g["solid"].as_array().unwrap().len());
    for (k, want) in g["lake_layout"].as_array().unwrap().iter().enumerate() {
        let l = &lakes::lake_layout()[k];
        for (key, got) in [
            ("left", l.left),
            ("width", l.width),
            ("body_x", l.body_x),
            ("body_y", l.body_y),
            ("body_w", l.body_w),
            ("body_h", l.body_h),
        ] {
            assert_eq!(got, f(&want[key]), "layout {k}.{key}");
        }
    }
    for r in g["lake_pick"].as_array().unwrap() {
        assert_eq!(lakes::lake_pick_kind(f(&r[0]), f(&r[1])), i(&r[2]), "{r}");
    }
    for k in 0..3 {
        assert_eq!(lakes::lake_amount(k as i64), f(&g["lake_amount"][k]));
        assert_eq!(
            lakes::lake_flare_count(k as i64),
            f(&g["lake_flare_count"][k])
        );
    }
    assert_eq!(lakes::lake_flare_life_ms(), f(&g["flare_life"]));
    for r in g["flare_alpha"].as_array().unwrap() {
        assert_eq!(lakes::lake_flare_alpha(f(&r[0])), f(&r[1]), "{r}");
    }
    for r in g["cut_row"].as_array().unwrap() {
        let k = i(&r[0]);
        let lake = myco_core::state::LakeSave {
            amount: f(&r[1]),
            full: lakes::lake_amount(k),
            ..Default::default()
        };
        assert_eq!(
            lakes::lake_cut_row(&lake, &kinds[k as usize]),
            f(&r[2]),
            "{r}"
        );
    }
    // food sheets
    for (n, k) in (-1..=3).enumerate() {
        assert_eq!(
            food::food_cells(k as f64),
            f(&g["food_cells"][n]),
            "cells {k}"
        );
        assert_eq!(
            food::food_value(k as f64),
            f(&g["food_value"][n]),
            "value {k}"
        );
    }
    assert_eq!(food::food_kinds(), i(&g["food_kinds"]));
    assert_eq!(world::TURF_ASSET_INDEX, i(&g["turf"]));
    // soil rows
    assert_eq!(soil::dense_row(), f(&g["dense_row"]));
    assert_eq!(soil::deep_row(), f(&g["deep_row"]));
    assert_eq!(soil::ocean_row(), f(&g["ocean_row"]));
    let w = WorldRt::init(0.0, 31607, GameMode::Exploration);
    assert_eq!(w.ocean_y(), f(&g["ocean_y"]));
    assert_eq!(w.deep_y(), f(&g["deep_y"]));
    assert_eq!(w.dense_y(), f(&g["dense_y"]));
    for r in g["is_ocean"].as_array().unwrap() {
        assert_eq!(w.is_ocean_at(f(&r[0])), r[1].as_bool().unwrap(), "{r}");
    }
    for r in g["seam"].as_array().unwrap() {
        assert_eq!(soil::seam_row_at_screens(f(&r[0])), f(&r[1]), "{r}");
    }
    for r in g["rows"].as_array().unwrap() {
        let row = f(&r[0]);
        assert_eq!(soil::soil_band(row), f(&r[1]), "band {r}");
        assert_eq!(soil::soil_tier(row), f(&r[2]), "tier {r}");
        assert_eq!(soil::soil_sprite(row), r[3].as_str().unwrap(), "soil {r}");
        assert_eq!(soil::dirt_sprite(row), r[4].as_str().unwrap(), "dirt {r}");
        assert_eq!(
            soil::soil_edge_sprite(soil::soil_band(row), row),
            r[5].as_str().unwrap()
        );
    }
}

fn params_eq(w: &WorldRt, want: &Value) {
    let p = &w.params;
    assert_eq!(p.tile as f64, f(&want["tile"]));
    assert_eq!(p.surface_y, f(&want["surface_y"]));
    assert_eq!(p.rows as f64, f(&want["rows"]));
    assert_eq!(p.cols as f64, f(&want["cols"]));
    assert_eq!(p.chunk_w as f64, f(&want["chunk_w"]));
    assert_eq!(p.bottom_y, f(&want["bottom_y"]));
    assert_eq!(p.water_row as f64, f(&want["water_row"]));
    assert_eq!(p.hard_row as f64, f(&want["hard_row"]));
    assert_eq!(p.layer_rows as f64, f(&want["layer_rows"]));
    assert_eq!(p.seed as f64, f(&want["seed"]));
    assert_eq!(p.turf as f64, f(&want["turf"]));
    assert_eq!(p.start_x, f(&want["start_x"]));
}

fn chunk_index(w: &WorldRt) -> Value {
    to_json(&w.params.chunk_index)
}

/// foods and lakes of every generated chunk, keyed like the golden ("c<cx>")
fn chunk_lists_eq(w: &WorldRt, want: &Value, what: &str) {
    let want = want.as_object().unwrap();
    assert_eq!(w.chunks.len(), want.len(), "{what}: chunk count");
    for (key, wc) in want {
        let cx: i64 = key[1..].parse().unwrap();
        let ch = w
            .chunk(cx)
            .unwrap_or_else(|| panic!("{what}: missing {key}"));
        json_eq(
            &format!("{what}.{key}.foods"),
            &to_json(&ch.foods),
            &wc["foods"],
        );
        json_eq(
            &format!("{what}.{key}.lakes"),
            &to_json(&ch.lakes),
            &wc["lakes"],
        );
        if let Some(rows) = wc.get("rows") {
            assert_eq!(ch.rows as f64, f(rows), "{what}.{key}.rows");
        }
        if let Some(t) = wc.get("tiles") {
            let tiles: Vec<f64> = if t[0].is_array() {
                unrle(t)
            } else {
                t.as_array().unwrap().iter().map(f).collect()
            };
            let got: Vec<f64> = ch.tiles.iter().map(|&v| v as f64).collect();
            assert_eq!(got, tiles, "{what}.{key}.tiles");
            let amount: Vec<f64> = if wc["amount"][0].is_array() {
                unrle(&wc["amount"])
            } else {
                wc["amount"].as_array().unwrap().iter().map(f).collect()
            };
            assert_eq!(ch.amount, amount, "{what}.{key}.amount");
        }
    }
}

fn tile_grid_eq(w: &mut WorldRt, grids: &Value, what: &str) {
    for g in grids.as_array().unwrap() {
        let cx = f(&g["cx"]);
        let rows = i(&g["rows"]);
        let want = unrle(&g["codes"]);
        let p = w.params.clone();
        let mut got = Vec::new();
        for row in 0..rows {
            let y = p.surface_y + (row * p.tile) as f64 + p.tile as f64 * 0.5;
            for col in 0..p.cols {
                let x = ((cx as i64 * p.cols + col) * p.tile) as f64 + p.tile as f64 * 0.5;
                got.push(w.tile_at(x, y).code() as f64);
            }
        }
        assert_eq!(got, want, "{what}: tile grid of chunk {cx}");
    }
}

fn point(v: &Value) -> (f64, f64) {
    let x = if v[0].is_string() { f64::NAN } else { f(&v[0]) };
    (x, f(&v[1]))
}

fn sources(sim: &Sim) -> Value {
    let mut out = Vec::new();
    for src in &sim.state.water {
        match src {
            WaterSave::Lake { lake_cx, lake, .. } => {
                let l = &sim.world.chunk(*lake_cx).unwrap().lakes[*lake as usize];
                out.push(serde_json::json!(["lake", lake_cx, l.body_x, l.amount]));
            }
            WaterSave::Cell { cx, idx, .. } => {
                out.push(serde_json::json!([
                    "cell",
                    cx,
                    idx,
                    world::water_source_amount(sim, src)
                ]));
            }
        }
    }
    Value::Array(out)
}

fn world_seed_matches(topic: &str, seed: i64) {
    // R19: these goldens were captured with stones neutralized (world_lib.lua); drop this when
    // world_* are re-captured after Tasks 6 and 7 merge
    myco_core::world::stones::set_golden_neutral(true);
    let g = golden(topic);
    let mut sim = Sim::new_game(GameMode::Exploration, "Golden", Some(seed));
    params_eq(&sim.world, &g["params"]);
    assert_eq!(
        chunk_index(&sim.world),
        g["params"]["chunk_index"],
        "chunk_index after init"
    );
    chunk_lists_eq(&sim.world, &g["init_chunks"], "init");

    let layer_rows = sim.world.params.layer_rows;
    for cx in -2..=2 {
        sim.world.get_chunk(cx);
        sim.world.chunk_reach(cx, 41 * layer_rows - 1);
    }
    chunk_lists_eq(&sim.world, &g["chunks"], "reached");
    assert_eq!(chunk_index(&sim.world), g["chunk_index"]);
    tile_grid_eq(&mut sim.world, &g["tile_grid"], topic);

    let pts: Vec<(f64, f64)> = g["points"].as_array().unwrap().iter().map(point).collect();
    for (k, &(x, y)) in pts.iter().enumerate() {
        let q = &g["queries"][k];
        let w = &mut sim.world;
        let got = [
            w.tile_at(x, y).code() as f64,
            w.water_at(x, y) as i64 as f64,
            w.water_shown_at(x, y) as i64 as f64,
            w.is_passable(x, y) as i64 as f64,
            w.is_deep_soil(x, y) as i64 as f64,
            w.soil_water_factor(x, y),
            w.travel_cost(x, y, 0.1),
        ];
        let want: Vec<f64> = q.as_array().unwrap().iter().map(f).collect();
        assert_eq!(got.to_vec(), want, "queries at ({x},{y})");
    }
    for r in g["locate"].as_array().unwrap() {
        let l = sim.world.locate(f(&r[0]), f(&r[1]));
        assert_eq!((l.cx, l.row, l.idx), (i(&r[2]), i(&r[3]), i(&r[4])), "{r}");
    }
    for r in g["food_value_in"].as_array().unwrap() {
        let v = sim
            .world
            .food_value_in(f(&r[0]), f(&r[1]), f(&r[2]), f(&r[3]));
        assert_eq!(v, f(&r[4]), "{r}");
    }

    // mutations
    for r in g["connect"].as_array().unwrap() {
        let got = world::connect_water_at(&mut sim, f(&r[0]), f(&r[1]));
        assert_eq!(got as i64, i(&r[2]), "connect {r}");
    }
    json_eq("sources", &sources(&sim), &g["sources"]);
    assert_eq!(sim.water_total(), f(&g["water_total"]));
    for r in g["spend"].as_array().unwrap() {
        world::water_spend(&mut sim, f(&r[0]));
        assert_eq!(sim.state.state.water_free, f(&r[1]), "water_free after {r}");
        assert_eq!(sim.water_total(), f(&r[2]), "total after spend");
        assert_eq!(world::water_shown(&sim), f(&r[3]), "shown after spend");
        json_eq("spend sources", &sources(&sim), &r[4]);
    }
    for (k, &(x, y)) in pts.iter().enumerate() {
        let want = i(&g["shown_after_spend"][k]) == 1;
        assert_eq!(
            sim.world.water_shown_at(x, y),
            want,
            "shown after spend at ({x},{y})"
        );
    }
    for r in g["collect"].as_array().unwrap() {
        let got = world::collect_food_at(&mut sim, f(&r[0]), f(&r[1]), f(&r[2]));
        assert_eq!(got, f(&r[3]), "collect {r}");
    }
    for (n, cx) in (-2..=2).enumerate() {
        let ch = sim.world.chunk(cx).unwrap();
        json_eq(
            "final foods",
            &to_json(&ch.foods),
            &g["final_chunks"][n]["foods"],
        );
        json_eq(
            "final lakes",
            &to_json(&ch.lakes),
            &g["final_chunks"][n]["lakes"],
        );
    }
}

#[test]
fn chunk_tiles_match_love_seed_31607() {
    world_seed_matches("world_s31607", 31607);
}

#[test]
fn chunk_tiles_match_love_seed_7() {
    world_seed_matches("world_s7", 7);
}

#[test]
fn survival_layout_matches_love() {
    // R19: these goldens were captured with stones neutralized (world_lib.lua); drop this when
    // world_* are re-captured after Tasks 6 and 7 merge
    myco_core::world::stones::set_golden_neutral(true);
    let g = golden("world_survival");
    // exploration: the ocean is infinite water; a spend dips the shown amount and refills
    let mut sim = Sim::new_game(GameMode::Exploration, "Golden", Some(31607));
    let oy = f(&g["ocean"][0]);
    assert_eq!(sim.world.ocean_y(), oy);
    assert_eq!(sim.world.tile_at(10.0, oy + 5.0), Tile::Water);
    assert!(sim.world.water_at(10.0, oy + 5.0));
    assert_eq!(
        world::connect_water_at(&mut sim, 10.0, oy + 5.0) as i64,
        i(&g["infinite"][0])
    );
    assert_eq!(
        world::connect_water_at(&mut sim, 10.0, oy + 5.0) as i64,
        i(&g["infinite"][1])
    );
    assert_eq!(sim.state.state.water_infinite as i64, i(&g["infinite"][2]));
    world::water_spend(&mut sim, 50.0);
    let dips = g["dip"].as_array().unwrap();
    for (k, want) in dips.iter().enumerate() {
        if k + 1 == dips.len() {
            world::water_spend(&mut sim, 7.0);
        } else {
            world::water_counter_step(&mut sim);
        }
        let got = [
            sim.world.water_dip,
            sim.world.water_dip_hold,
            world::water_shown(&sim),
        ];
        let want: Vec<f64> = want.as_array().unwrap().iter().map(f).collect();
        assert_eq!(got.to_vec(), want, "dip step {k}");
    }

    let mut sim = Sim::new_game(GameMode::Survival, "Golden", Some(31607));
    chunk_lists_eq(&sim.world, &g["thinned"], "thinned");
    for cx in [3, -4, 6] {
        sim.world.get_chunk(cx);
    }
    let lr = sim.world.params.layer_rows;
    sim.world.chunk_reach(0, 4 * lr - 1);
    sim.world.chunk_reach(6, 3 * lr - 1);
    chunk_lists_eq(&sim.world, &g["chunks"], "survival");
    assert_eq!(chunk_index(&sim.world), g["chunk_index"]);
    for r in g["full_at"].as_array().unwrap() {
        assert_eq!(
            sim.world.survival_full_at(f(&r[0]), f(&r[1])) as i64,
            i(&r[2]),
            "{r}"
        );
    }
    let r = &g["rock"];
    assert_eq!(sim.world.bottom_is_rock() as i64, i(&r[0]));
    assert_eq!(sim.world.tile_at(10.0, oy + 5.0).code(), i(&r[1]));
    assert_eq!(sim.world.water_at(10.0, oy + 5.0) as i64, i(&r[2]));
    assert_eq!(sim.world.water_shown_at(10.0, oy + 5.0) as i64, i(&r[3]));
    assert_eq!(
        world::connect_water_at(&mut sim, 10.0, oy + 5.0) as i64,
        i(&r[4])
    );
}

pub fn fixture_state(name: &str) -> GameState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(text.trim_end_matches('\0')).unwrap()
}

#[test]
fn fixture_world_loads_identically() {
    // R19: these goldens were captured with stones neutralized (world_lib.lua); drop this when
    // world_* are re-captured after Tasks 6 and 7 merge
    myco_core::world::stones::set_golden_neutral(true);
    let g = golden("world_fixture");
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    params_eq(&sim.world, &g["params"]);
    let mut saved: Vec<i64> = sim.world.chunks.keys().copied().collect();
    saved.sort_by_key(|cx| format!("c{cx}"));
    let want: Vec<i64> = g["saved_cx"].as_array().unwrap().iter().map(i).collect();
    assert_eq!(saved, want);
    tile_grid_eq(&mut sim.world, &g["tile_grid_saved"], "saved rows");
    let lr = sim.world.params.layer_rows;
    for &cx in &saved {
        sim.world.chunk_reach(cx, 3 * lr - 1);
    }
    sim.world.get_chunk(3);
    chunk_lists_eq(&sim.world, &g["chunks"], "fixture");
    assert_eq!(chunk_index(&sim.world), g["chunk_index"]);
    tile_grid_eq(&mut sim.world, &g["tile_grid"], "fixture rows");

    // what save_game writes back: world, water and forest exactly as LÖVE writes them
    let want: Value =
        serde_json::from_str(g["save_json"].as_str().unwrap().trim_end_matches('\0')).unwrap();
    let (out, _) = sim.to_save();
    let got = to_json(&out);
    json_eq("save.world", &got["world"], &want["world"]);
    json_eq("save.water", &got["water"], &want["water"]);
    json_eq("save.forest", &got["forest"], &want["forest"]);
}

#[test]
fn world_chunks_marked_dirty() {
    let mut sim = Sim::new_game(GameMode::Exploration, "Dirty", Some(7));
    assert!(
        sim.world.dirty_chunks.contains(&0),
        "generation marks the chunk dirty"
    );
    sim.world.dirty_chunks.clear();
    let lr = sim.world.params.layer_rows;
    sim.world.chunk_reach(1, 2 * lr - 1);
    assert!(
        sim.world.dirty_chunks.contains(&1),
        "reaching new rows marks the chunk dirty"
    );
}
