//! Mycelium (Task 8) against the LÖVE port. Golden topic `mycelium`
//! (rust/golden/scripts/mycelium.lua): the Exploration fixture loaded through load_game, then
//! shot parameters, playability probes, five seeded shots, a mouse-driven Step_0 session,
//! quick nodes, node burst, digger burst, spores and placement checks, replayed here in the
//! same order on the same state. Plus Rust-only tests for the right-click quick node
//! (`Command::QuickNode`, rejection reasons and ignore rules) and the resource events.
mod common;

use common::{f, golden, i};
use myco_core::events::{NodeReject, ResourceKind, SimEvent};
use myco_core::mycelium::{
    self, growth, knots, place, shot, soil, BuildOption, Growth, GrowthOpts, Ray,
};
use myco_core::rng::Rng;
use myco_core::sim::{ButtonState, Command, Key, Sim, StepInput};
use myco_core::state::{GameMode, GameState};
use serde_json::Value;

fn fixture_state(name: &str) -> GameState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    let mut st: GameState = serde_json::from_str(text.trim_end_matches('\0')).unwrap();
    st.apply_load_defaults();
    st
}

/// The fixture as load_game leaves it, with the golden script's fixed camera view.
fn fixture_sim(g: &Value) -> Sim {
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let v = &g["view"];
    sim.camera.view_x = f(&v["x"]);
    sim.camera.view_y = f(&v["y"]);
    sim.camera.width = f(&v["w"]);
    sim.camera.height = f(&v["h"]);
    sim
}

fn close(path: &str, a: f64, b: f64) {
    if a == b {
        return;
    }
    let scale = 1f64.max(a.abs()).max(b.abs());
    assert!((a - b).abs() <= 1e-9 * scale, "{path}: {a:?} vs {b:?}");
}

fn num_eq(path: &str, got: &[f64], want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(got.len(), w.len(), "{path}: length");
    for (k, (a, b)) in got.iter().zip(w).enumerate() {
        close(&format!("{path}[{k}]"), *a, f(b));
    }
}

fn b(v: bool) -> f64 {
    v as i64 as f64
}

fn set_upg(sim: &mut Sim, id: &str, lvl: i64) {
    sim.state.state.upgrade_levels.insert(id.to_string(), lvl);
}

fn upg(sim: &Sim, id: &str) -> i64 {
    sim.state.state.upgrade_levels.get(id).copied().unwrap_or(0)
}

fn set_mode(sim: &mut Sim, mode: GameMode) {
    sim.state.state.mode = mode;
    sim.world.mode = mode;
}

fn growth_eq(path: &str, g: &Growth, want: &Value, with_cells: bool) {
    let brs = want["branches"].as_array().unwrap();
    assert_eq!(g.branches.len(), brs.len(), "{path}: branch count");
    for (k, (br, wb)) in g.branches.iter().zip(brs).enumerate() {
        let p = format!("{path}.b{k}");
        let pts: Vec<f64> = br.points.iter().flat_map(|q| [q.x, q.y]).collect();
        let wpts: Vec<f64> = wb["points"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|q| [f(&q[0]), f(&q[1])])
            .collect();
        assert_eq!(pts.len(), wpts.len(), "{p}: point count");
        for (j, (a, w)) in pts.iter().zip(&wpts).enumerate() {
            close(&format!("{p}.pt{j}"), *a, *w);
        }
        if with_cells {
            match (&br.cells, &wb["cells"]) {
                (None, Value::String(s)) => assert_eq!(s, "undefined", "{p}: cells"),
                (Some(cs), Value::Array(w)) => {
                    let got: Vec<f64> = cs.iter().flat_map(|c| [c.x, c.y, c.w]).collect();
                    let want: Vec<f64> = w
                        .iter()
                        .flat_map(|c| [f(&c[0]), f(&c[1]), f(&c[2])])
                        .collect();
                    assert_eq!(got, want, "{p}: cells");
                }
                (c, w) => panic!("{p}: cells {:?} vs {w}", c.as_ref().map(|c| c.len())),
            }
        }
        for (name, v) in [
            ("start", br.start),
            ("width", br.width),
            ("shoved", b(br.shoved)),
            ("swept", br.swept),
            ("bx1", br.bx1),
            ("by1", br.by1),
            ("bx2", br.bx2),
            ("by2", br.by2),
            ("scan_done", br.scan_done),
            ("scan_done_len", br.scan_done_len),
        ] {
            close(&format!("{p}.{name}"), v, f(&wb[name]));
        }
    }
    for (name, v) in [
        ("duration", g.duration),
        ("finish_age", g.finish_age),
        ("finished", b(g.finished)),
        ("budding", b(g.budding)),
        ("no_length", b(g.no_length)),
        ("scanned_length", g.scanned_length),
        ("bx1", g.bx1),
        ("by1", g.by1),
        ("bx2", g.bx2),
        ("by2", g.by2),
        ("sound", b(g.sound.is_some())),
    ] {
        close(&format!("{path}.{name}"), v, f(&want[name]));
    }
}

fn growths_eq(path: &str, sim: &Sim, want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(sim.mycelium.growths.len(), w.len(), "{path}: growth count");
    for (k, (g, wg)) in sim.mycelium.growths.iter().zip(w).enumerate() {
        growth_eq(&format!("{path}.g{k}"), g, wg, false);
    }
}

fn knots_eq(path: &str, sim: &Sim, want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(sim.mycelium.knots.len(), w.len(), "{path}: knot count");
    for (k, (kn, wk)) in sim.mycelium.knots.iter().zip(w).enumerate() {
        let p = format!("{path}.k{k}");
        close(&format!("{p}.x"), kn.x, f(&wk["x"]));
        close(&format!("{p}.y"), kn.y, f(&wk["y"]));
        match &kn.shroom {
            Some(s) => {
                assert_eq!(s.kind, i(&wk["kind"]), "{p}.kind");
                assert_eq!(s.cell, i(&wk["cell"]), "{p}.cell");
                close(&format!("{p}.born"), kn.born.unwrap(), f(&wk["born"]));
            }
            None => assert!(wk.get("kind").is_none(), "{p}: expected a spore knot"),
        }
    }
}

fn events_eq(path: &str, sim: &Sim, want: &Value) {
    let got: Vec<(String, f64, f64)> = sim
        .state
        .state
        .tutorial
        .events
        .iter()
        .map(|(k, v)| (k.clone(), f(&v["x"]), f(&v["y"])))
        .collect();
    let w = want.as_array().unwrap();
    assert_eq!(got.len(), w.len(), "{path}: {got:?} vs {want}");
    for (k, (g, e)) in got.iter().zip(w).enumerate() {
        assert_eq!(g.0, e[0].as_str().unwrap(), "{path}[{k}]");
        close(&format!("{path}[{k}].x"), g.1, f(&e[1]));
        close(&format!("{path}[{k}].y"), g.2, f(&e[2]));
    }
}

fn unfinished(sim: &Sim) -> f64 {
    sim.mycelium.growths.iter().filter(|g| !g.finished).count() as f64
}

fn digest(sim: &Sim, tag: f64) -> Vec<f64> {
    let m = &sim.mycelium;
    vec![
        tag,
        sim.state.state.food,
        sim.water_total(),
        sim.state.state.length_px,
        m.growths.len() as f64,
        unfinished(sim),
        m.knots.len() as f64,
        b(m.dragging),
        b(m.has_aim),
        m.aim_dir,
        m.aim_len,
        m.aim_reach,
        m.aim_cost,
        m.aim_food,
        m.spores.len() as f64,
    ]
}

/// One scripted frame: {x, y, down, pressed, alt}, exactly like the golden script's Step_0 call.
fn step_input(inp: &Value) -> StepInput {
    let pressed = i(&inp[3]) == 1;
    let mut s = StepInput {
        cursor: Some((f(&inp[0]), f(&inp[1]))),
        lmb: ButtonState {
            down: i(&inp[2]) == 1,
            pressed,
            released: false,
        },
        ..Default::default()
    };
    if i(&inp[4]) == 1 {
        s.keys_down.push(Key::Alt);
    }
    s
}

fn run(sim: &mut Sim, inp: &StepInput) {
    mycelium::step(sim, inp);
    sim.frame += 1;
}

fn idle() -> StepInput {
    step_input(&serde_json::json!([-400, 200, 0, 0, 0]))
}

fn knot_digest(sim: &Sim) -> Vec<f64> {
    sim.mycelium.knots.iter().flat_map(|k| [k.x, k.y]).collect()
}

// ---------------------------------------------------------------------------------------------

#[test]
fn constants_and_loaded_mycelium_match_love() {
    let g = golden("mycelium");
    let sim = fixture_sim(&g);
    let m = &sim.mycelium;
    let c = &g["consts"];
    for (name, v) in [
        ("surface_y", m.surface_y),
        ("tile", m.tile),
        ("sample_step", m.sample_step),
        ("min_growth_len", m.min_growth_len),
        ("growth_duration", m.growth_duration),
        ("food_pickup_radius", m.food_pickup_radius),
        ("knot_radius", m.knot_radius),
        ("pick_radius", m.pick_radius),
    ] {
        close(name, v, f(&c[name]));
    }
    let l = &g["loaded"];
    close("x", m.x, f(&l["x"]));
    close("y", m.y, f(&l["y"]));
    close("far_max", m.far_max, f(&l["far_max"]));
    assert_eq!(m.dirty_growths.len() as i64, i(&l["unbaked"]));
    assert_eq!(m.active.len() as i64, i(&l["active"]));
    let wg = l["growths"].as_array().unwrap();
    for (k, (gr, w)) in m.growths.iter().zip(wg).enumerate() {
        growth_eq(&format!("loaded.g{k}"), gr, w, false);
        close(&format!("loaded.g{k}.age"), gr.age, f(&w["age"]));
    }
    knots_eq("loaded.knots", &sim, &l["knots"]);
    assert_eq!(i(&g["blocking"]), 0);
}

#[test]
fn shot_params_match_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let ids = [
        "shot_range",
        "digger_length",
        "shot_width",
        "water_efficiency",
        "node_efficiency",
        "budding_chance",
        "soil_penetration",
        "spore_quality",
    ];
    for row in g["params"].as_array().unwrap() {
        let mode = if row["mode"] == "survival" {
            GameMode::Survival
        } else {
            GameMode::Exploration
        };
        set_mode(&mut sim, mode);
        for id in ids {
            set_upg(&mut sim, id, i(&row["lvl"]));
        }
        sim.weather.intensity = f(&row["rain"]);
        let p = format!("{} lvl {} rain {}", row["mode"], row["lvl"], row["rain"]);
        for (name, v) in [
            ("shot_range", shot::shot_range(&sim)),
            ("shot_range_max", shot::shot_range_max(&sim)),
            ("digger_range", shot::digger_range(&sim)),
            ("shot_spread", shot::shot_spread(&sim)),
            ("water_cost", shot::shot_water_cost(&sim)),
            ("water_rate", shot::shot_water_rate(&sim)),
            ("node_cost", shot::node_food_cost(&sim)),
            ("budding", shot::budding_chance(&sim)),
            ("pen", soil::soil_penetration(&sim)),
            ("root_chance", knots::spore_root_chance(&sim)),
        ] {
            close(&format!("{p} {name}"), v, f(&row[name]));
        }
    }
}

#[test]
fn node_cost_matches_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    for row in g["node_cost"].as_array().unwrap() {
        let mode = if row[0] == "survival" {
            GameMode::Survival
        } else {
            GameMode::Exploration
        };
        set_mode(&mut sim, mode);
        set_upg(&mut sim, "node_efficiency", i(&row[1]));
        close(&format!("{row}"), shot::node_food_cost(&sim), f(&row[2]));
    }
}

#[test]
fn length_cm_matches_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let loc = |k: &str| k.to_string();
    for row in g["cm"].as_array().unwrap() {
        let px = f(&row[0]);
        sim.state.state.length_px = px;
        close("pixels_to_cm", soil::pixels_to_cm(&sim, px), f(&row[1]));
        let cm = soil::mycelium_cm(&sim);
        close("mycelium_cm", cm, f(&row[2]));
        assert_eq!(soil::length_text(false, cm, &loc), row[3].as_str().unwrap());
        assert_eq!(soil::length_text(true, cm, &loc), row[4].as_str().unwrap());
    }
    sim.state.state.length_px = 526.4976212684751;
    let mut got = Vec::new();
    for d in [10.0, -5.0, 0.0, 2.5] {
        soil::mycelium_add_length(&mut sim, d);
        got.push(sim.state.state.length_px);
    }
    num_eq("add_length", &got, &g["add_length"]);
}

#[test]
fn playability_matches_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let p = &g["play"];
    for row in p["points"].as_array().unwrap() {
        let (x, y) = (f(&row[0]), f(&row[1]));
        let got = [
            b(place::is_blocked_point(&mut sim, x, y)),
            b(place::is_playable_point(&mut sim, x, y, false)),
            b(place::is_playable_point(&mut sim, x, y, true)),
            b(place::is_playable_clear_point(&mut sim, x, y, 1.0, false)),
            b(place::is_playable_clear_point(&mut sim, x, y, 2.0, false)),
            b(place::is_playable_clear_point(&mut sim, x, y, 2.0, true)),
        ];
        num_eq(&format!("point {x},{y}"), &got, &Value::from(row.as_array().unwrap()[2..].to_vec()));
    }
    // r = 1 is the GML `true` myc_place_check passes as the radius (the LÖVE crash, baff3b6)
    for row in p["segments"].as_array().unwrap() {
        let v: Vec<f64> = row.as_array().unwrap().iter().map(f).collect();
        let (x1, y1, x2, y2) = (v[0], v[1], v[2], v[3]);
        let got = [
            b(place::segment_is_playable(&mut sim, x1, y1, x2, y2, 1.0, false)),
            b(place::segment_is_playable(&mut sim, x1, y1, x2, y2, 1.0, false)),
            b(place::segment_is_playable(&mut sim, x1, y1, x2, y2, 2.0, false)),
            b(place::segment_is_playable(&mut sim, x1, y1, x2, y2, 2.0, true)),
        ];
        num_eq(&format!("segment {v:?}"), &got, &Value::from(v[4..].to_vec()));
    }
    for row in p["measure"].as_array().unwrap() {
        let v: Vec<f64> = row.as_array().unwrap().iter().map(f).collect();
        let (x, y, dir, maxlen) = (v[0], v[1], v[2], v[3]);
        let l0 = place::measure_allowed_len_from(&mut sim, x, y, dir, maxlen, false);
        let l1 = place::measure_allowed_len_from(&mut sim, x, y, dir, maxlen, true);
        let got = [
            l0,
            l1,
            shot::shot_cost_for(&mut sim, x, y, dir, l0),
            shot::shot_food_for(&mut sim, x, y, dir, l0),
            shot::shot_cost_for(&mut sim, x, y, dir, maxlen),
            shot::shot_food_for(&mut sim, x, y, dir, maxlen),
        ];
        num_eq(&format!("measure {v:?}"), &got, &Value::from(v[4..].to_vec()));
    }
}

#[test]
fn shot_length_matches_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let cases: [(f64, f64, f64, &[(&str, i64)], Option<GrowthOpts>); 5] = [
        (0.0, 480.0, 270.0, &[], None),
        (
            -44.58447299418679,
            596.7521495073507,
            200.0,
            &[("shot_range", 3), ("shot_width", 2)],
            None,
        ),
        (0.0, 480.0, 315.0, &[("shot_range", 6), ("budding_chance", 4)], None),
        (
            150.0,
            1800.0,
            250.0,
            &[("soil_penetration", 2)],
            Some(GrowthOpts {
                free: true,
                pierce: true,
                width_mult: 1.4,
                kinks_per_base: 2.0,
                ..GrowthOpts::default()
            }),
        ),
        (
            -44.58447299418679,
            596.7521495073507,
            120.0,
            &[("shot_width", 5)],
            Some(GrowthOpts {
                free: true,
                silent: true,
                instant: true,
                ..GrowthOpts::default()
            }),
        ),
    ];
    let shots = g["shots"].as_array().unwrap();
    for ((x, y, dir, upgs, opts), want) in cases.into_iter().zip(shots) {
        let saved: Vec<(&str, i64)> = upgs.iter().map(|&(id, _)| (id, upg(&sim, id))).collect();
        for &(id, lvl) in upgs {
            set_upg(&mut sim, id, lvl);
        }
        sim.rng = Rng::new(i(&want["seed"]));
        let opts = opts.unwrap_or_default();
        let range = shot::shot_range(&sim);
        let len = place::measure_allowed_len_from(&mut sim, x, y, dir, range, opts.pierce);
        let p = format!("shot {}", want["seed"]);
        close(&format!("{p} len"), len, f(&want["len"]));
        let n0 = sim.mycelium.growths.len();
        let ok = shot::add_growth_from(&mut sim, x, y, dir, len, &opts);
        assert_eq!(b(ok), f(&want["ok"]), "{p} ok");
        close(&format!("{p} food"), sim.state.state.food, f(&want["food"]));
        close(&format!("{p} water"), sim.water_total(), f(&want["water"]));
        close(&format!("{p} water_free"), sim.state.state.water_free, f(&want["water_free"]));
        assert_eq!(sim.mycelium.growths.len(), n0 + 1);
        growth_eq(&p, sim.mycelium.growths.last().unwrap(), &want["growth"], true);
        close(&format!("{p} probe"), sim.rng.random(1.0), f(&want["probe"]));
        for (id, lvl) in saved {
            set_upg(&mut sim, id, lvl);
        }
    }
}

/// The whole golden session in order: two aimed LMB shots through `mycelium::step`, quick
/// nodes, a long shot through food, node burst, digger burst, spores, placement checks.
#[test]
fn session_matches_love() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let saved_shot_range = upg(&sim, "shot_range");

    // session 1: aim from the origin knot, release, let it grow; then from the second knot
    sim.rng = Rng::new(4242);
    let digests = g["session"]["digests"].as_array().unwrap();
    for (k, inp) in g["session_inputs"].as_array().unwrap().iter().enumerate() {
        run(&mut sim, &step_input(inp));
        num_eq(&format!("session frame {}", k + 1), &digest(&sim, (k + 1) as f64), &digests[k]);
    }
    let s = &g["session"];
    growths_eq("session", &sim, &s["growths"]);
    knots_eq("session.knots", &sim, &s["knots"]);
    num_eq("session.cam_box", &sim.state.state.cam_box, &s["cam_box"]);
    close("session.far_max", sim.mycelium.far_max, f(&s["far_max"]));
    events_eq("session.events", &sim, &s["events"]);
    close("session.probe", sim.rng.random(1.0), f(&s["probe"]));

    // quick nodes (myc_node_quick = the right-click quick node)
    sim.rng = Rng::new(777);
    let rows = g["quick"]["rows"].as_array().unwrap();
    for row in &rows[..rows.len() - 1] {
        let (x, y) = (f(&row[0]), f(&row[1]));
        let r = sim.mycelium.pick_radius;
        let hit = growth::get_mycelium_hit(&mut sim, x, y, r);
        let ok = knots::quick_node(&mut sim, x, y).is_ok();
        let got = [
            x,
            y,
            b(ok),
            b(hit.hit),
            hit.x,
            hit.y,
            sim.state.state.food,
            sim.mycelium.knots.len() as f64,
        ];
        num_eq(&format!("quick {x},{y}"), &got, row);
    }
    sim.state.state.food = 3.0;
    let last = rows.last().unwrap();
    let r = knots::quick_node(&mut sim, 110.0, 746.0);
    assert_eq!(b(r.is_ok()), f(&last[2]));
    close("quick poor food", sim.state.state.food, f(&last[6]));
    sim.state.state.food = 500.0;
    knots_eq("quick.knots", &sim, &g["quick"]["knots"]);
    events_eq("quick.events", &sim, &g["quick"]["events"]);

    // session 2: a long shot through food, an alt+click quick node
    set_upg(&mut sim, "shot_range", 4);
    sim.rng = Rng::new(4343);
    let s2 = &g["session2"];
    close("s2 food0", sim.state.state.food, f(&s2["food0"]));
    close("s2 water0", sim.water_total(), f(&s2["water0"]));
    let digests = s2["digests"].as_array().unwrap();
    for (k, inp) in g["session2_inputs"].as_array().unwrap().iter().enumerate() {
        run(&mut sim, &step_input(inp));
        num_eq(&format!("session2 frame {}", k + 1), &digest(&sim, (k + 1) as f64), &digests[k]);
    }
    growths_eq("session2", &sim, &s2["growths"]);
    knots_eq("session2.knots", &sim, &s2["knots"]);
    num_eq("session2.cam_box", &sim.state.state.cam_box, &s2["cam_box"]);
    close("session2.water_free", sim.state.state.water_free, f(&s2["water_free"]));
    assert_eq!(sim.state.water.len() as i64, i(&s2["sources"]));
    events_eq("session2.events", &sim, &s2["events"]);
    close("session2.probe", sim.rng.random(1.0), f(&s2["probe"]));
    set_upg(&mut sim, "shot_range", saved_shot_range);

    // node cost table changes the mode on the LÖVE side and restores it: nothing to replay

    // node burst
    sim.rng = Rng::new(9001);
    growth::node_burst(&mut sim, 40.0, 900.0);
    let bw = &g["burst"];
    assert_eq!(sim.mycelium.burst_queue.len() as i64, i(&bw["queue"]));
    for (k, want) in bw["digests"].as_array().unwrap().iter().enumerate() {
        run(&mut sim, &idle());
        num_eq(&format!("burst frame {}", k + 1), &digest(&sim, (k + 1) as f64), want);
    }
    growths_eq("burst", &sim, &bw["growths"]);
    knots_eq("burst.knots", &sim, &bw["knots"]);
    close("burst.probe", sim.rng.random(1.0), f(&bw["probe"]));

    // digger burst
    sim.rng = Rng::new(31337);
    set_upg(&mut sim, "digger_length", 2);
    growth::digger_burst(&mut sim, 120.0, 480.0);
    let gw = &g["digger"];
    assert_eq!(sim.mycelium.digger_queue.len() as i64, i(&gw["queue"]));
    for (k, want) in gw["digests"].as_array().unwrap().iter().enumerate() {
        run(&mut sim, &idle());
        num_eq(&format!("digger frame {}", k + 1), &digest(&sim, (k + 1) as f64), want);
    }
    growths_eq("digger", &sim, &gw["growths"]);
    close("digger.probe", sim.rng.random(1.0), f(&gw["probe"]));

    // spores
    sim.rng = Rng::new(2468);
    sim.state.state.food = 500.0;
    set_upg(&mut sim, "spore_count", 2);
    set_upg(&mut sim, "spore_quality", 3);
    let sw = &g["spores"];
    let ok = knots::spawn_spores_from(&mut sim, 0.0, 480.0);
    assert_eq!(b(ok), f(&sw["ok"]));
    knots::spawn_spores_show(&mut sim, 300.0, 420.0, 5.0, 1.3);
    let rays = [
        Ray { a: 80.0, power: 2.0 },
        Ray { a: 100.0, power: 1.5 },
        Ray { a: 60.0, power: 3.0 },
    ];
    knots::spawn_spores_rays(&mut sim, -300.0, 400.0, 7.0, &rays, 1.7);
    let spores = |sim: &Sim| -> Vec<f64> {
        sim.mycelium
            .spores
            .iter()
            .flat_map(|s| {
                [s.x, s.y, s.vx, s.vy, s.weight, s.age, s.life, s.phase, b(s.show), b(s.ray)]
            })
            .collect()
    };
    let flat = |v: &Value| -> Value {
        Value::from(
            v.as_array()
                .unwrap()
                .iter()
                .flat_map(|r| r.as_array().unwrap().clone())
                .collect::<Vec<_>>(),
        )
    };
    num_eq("spores.start", &spores(&sim), &flat(&sw["start"]));
    close("spores.food", sim.state.state.food, f(&sw["food"]));
    let snaps = sw["snaps"].as_array().unwrap();
    let mut si = 0;
    for k in 1..=1000 {
        run(&mut sim, &idle());
        if k % 100 == 0 {
            let snap = &snaps[si];
            si += 1;
            num_eq(&format!("spores@{k}"), &spores(&sim), &flat(&snap[1]));
            knots_eq(&format!("spores@{k}.knots"), &sim, &snap[2]);
        }
    }
    let stats: Vec<(String, f64)> = sim
        .state
        .state
        .tasks
        .stats
        .iter()
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    for (k, row) in sw["stats"].as_array().unwrap().iter().enumerate() {
        assert_eq!(stats[k].0, row[0].as_str().unwrap());
        close(&format!("stat {}", stats[k].0), stats[k].1, f(&row[1]));
    }
    num_eq("spores.cam_box", &sim.state.state.cam_box, &sw["cam_box"]);
    close("spores.probe", sim.rng.random(1.0), f(&sw["probe"]));

    // placement checks and the build menu
    for id in [
        "unlock_spore",
        "unlock_predator",
        "unlock_repeller",
        "unlock_tree",
        "unlock_digger",
        "unlock_stinky",
        "unlock_experience",
    ] {
        set_upg(&mut sim, id, 1);
    }
    sim.state.state.food = 1000.0;
    sim.rng = Rng::new(1357);
    let mut rows = g["place"].as_array().unwrap().iter();
    for t in 0..=6i64 {
        let kind = myco_core::mushrooms::MushroomKind::from_code(t).unwrap();
        let mut x = -900.0;
        while x <= 900.0 {
            let p = place::place_check(&mut sim, kind, x, Some(470.0));
            let can = place::can_build_type(&mut sim, kind, x);
            let got = [
                t as f64,
                x,
                b(p.ok),
                p.x,
                p.y,
                p.face,
                p.root_x,
                p.from_x,
                p.from_y,
                b(can),
            ];
            num_eq(&format!("place {t} {x}"), &got, rows.next().unwrap());
            x += 75.0;
        }
    }
    close("place.probe", sim.rng.random(1.0), f(&g["place_probe"]));
    for row in g["surface"].as_array().unwrap() {
        let (x, y) = (f(&row[0]), f(&row[1]));
        let sb = place::get_surface_build_at(&sim, x, y);
        let got = [
            x,
            y,
            b(sb.hit),
            sb.x,
            sb.y,
            b(place::surface_buildable_at(&mut sim, x, y)),
            b(place::has_mushroom_near(&sim, x, y, 50.0)),
        ];
        num_eq(&format!("surface {x},{y}"), &got, &Value::from(row.as_array().unwrap()[..7].to_vec()));
        assert_eq!(place::surface_block_reason(&sim, x, y), row[7].as_str().unwrap());
    }
    for row in g["menu"].as_array().unwrap() {
        let (x, y) = (f(&row[0]), f(&row[1]));
        let ok = place::open_build_menu_at(&mut sim, x, y);
        let m = &sim.mycelium;
        let got = [
            x,
            y,
            b(ok),
            m.build_node_x,
            m.build_node_y,
            m.build_surface_x,
            m.build_surface_y,
            b(m.build_surface_ok),
            b(m.mushroom_prompt),
        ];
        num_eq(&format!("menu {x},{y}"), &got, &Value::from(row.as_array().unwrap()[..9].to_vec()));
        assert_eq!(m.build_hint, row[9].as_str().unwrap());
        let opts = row[10].as_array().unwrap();
        assert_eq!(m.build_options.len(), opts.len());
        let options = m.build_options.clone();
        for (o, w) in options.iter().zip(opts) {
            assert_eq!(*o, BuildOption::Node);
            assert_eq!(w[0], "node");
            assert_eq!(b(place::can_build_option(&mut sim, o)), f(&w[2]));
            let label = place::build_option_label(&sim, o);
            let loc = |k: &str| k.to_string();
            assert_eq!(label.text(&loc), w[3].as_str().unwrap());
        }
    }

    // pure helpers
    let ob = place::origin_box(&sim);
    num_eq("origin_box", &[ob.x1, ob.y1, ob.x2, ob.y2], &g["origin_box"]);
    for row in g["smallshroom"].as_array().unwrap() {
        let r = knots::smallshroom_rect(&knots::Smallshroom {
            kind: i(&row[0]),
            cell: i(&row[1]),
        });
        num_eq("smallshroom", &[f(&row[0]), f(&row[1]), r.left, r.top, r.width, r.height], row);
    }
    for row in g["tips"].as_array().unwrap() {
        let t = mycelium::mycelium_extreme_tip(&sim, f(&row[0])).unwrap();
        num_eq("tip", &[f(&row[0]), t.0, t.1], row);
    }

    // the save view
    let (st, _) = sim.to_save();
    let sv = &g["saved"];
    let wg = sv["growths"].as_array().unwrap();
    assert_eq!(st.mycelium.growths.len(), wg.len());
    for (k, (gs, w)) in st.mycelium.growths.iter().zip(wg).enumerate() {
        let w = w.as_array().unwrap();
        assert_eq!(gs.branches.len(), w.len(), "saved g{k}");
        for (q, (bs, wb)) in gs.branches.iter().zip(w).enumerate() {
            let pts: Vec<f64> = bs.points.iter().flat_map(|p| [p.x, p.y]).collect();
            let wp = flat(&wb["points"]);
            num_eq(&format!("saved g{k} b{q}"), &pts, &wp);
            close("start", bs.start, f(&wb["start"]));
            close("width", bs.width, f(&wb["width"]));
            assert_eq!(b(bs.shoved), f(&wb["shoved"]));
            close("swept", bs.swept, f(&wb["swept"]));
        }
    }
    let wk = sv["knots"].as_array().unwrap();
    assert_eq!(st.mycelium.knots.len(), wk.len());
    close("saved length", st.state.length_px, f(&sv["length_px"]));
    // and it loads back to the same mycelium
    let sim2 = Sim::from_save(st).unwrap();
    assert_eq!(knot_digest(&sim2), knot_digest(&sim));
    assert_eq!(sim2.mycelium.growths.len(), sim.mycelium.growths.len());
    assert!(sim2.mycelium.growths.iter().all(|g| g.finished && g.no_length));
}

// ---------------------------------------------------------------------------------------------
// right-click quick node (Command::QuickNode) — Rust-only

fn rmb(x: f64, y: f64) -> StepInput {
    StepInput {
        cursor: Some((x, y)),
        rmb: ButtonState {
            down: true,
            pressed: true,
            released: false,
        },
        commands: vec![Command::QuickNode { x, y }],
        ..Default::default()
    }
}

fn rejections(sim: &mut Sim) -> Vec<NodeReject> {
    sim.drain_events()
        .into_iter()
        .filter_map(|e| match e {
            SimEvent::NodeRejected { reason, .. } => Some(reason),
            _ => None,
        })
        .collect()
}

/// A point on the fixture's first growth, well away from both knots.
const ON_HYPHA: (f64, f64) = (-50.0, 659.0);

#[test]
fn quick_node_rejections() {
    let g = golden("mycelium");
    // NotOnMycelium
    let mut sim = fixture_sim(&g);
    assert_eq!(knots::quick_node(&mut sim, 500.0, 300.0), Err(NodeReject::NotOnMycelium));

    // success: one knot, food spent, Resource{Food, -cost} at the knot, tutorial "node"
    let mut sim = fixture_sim(&g);
    let (x, y) = ON_HYPHA;
    let n0 = sim.mycelium.knots.len();
    let food0 = sim.state.state.food;
    let cost = shot::node_food_cost(&sim);
    sim.drain_events();
    run(&mut sim, &rmb(x, y));
    assert_eq!(sim.mycelium.knots.len(), n0 + 1);
    assert_eq!(sim.state.state.food, food0 - cost);
    let ev = sim.drain_events();
    let k = sim.mycelium.knots.last().unwrap().clone();
    assert!(
        ev.iter().any(|e| matches!(e, SimEvent::Resource { kind: ResourceKind::Food, delta, x, y, .. }
            if *delta == -cost && *x == k.x && *y == k.y)),
        "{ev:?}"
    );
    assert!(sim.state.state.tutorial.events.contains_key("node"));
    // TooCloseToNode: the same point again, many times; still exactly one new knot
    for _ in 0..5 {
        run(&mut sim, &rmb(x, y));
        assert_eq!(rejections(&mut sim), vec![NodeReject::TooCloseToNode]);
    }
    assert_eq!(sim.mycelium.knots.len(), n0 + 1);
    assert_eq!(sim.state.state.food, food0 - cost);

    // NotEnoughFood { need }
    let mut sim = fixture_sim(&g);
    sim.state.state.food = cost - 1.0;
    sim.drain_events();
    run(&mut sim, &rmb(x, y));
    assert_eq!(
        rejections(&mut sim),
        vec![NodeReject::NotEnoughFood { need: cost as i64 }]
    );
    assert_eq!(sim.mycelium.knots.len(), n0);

    // InWater: a free growth straight down through the fixture's lake at (150, 1000); a right
    // click on its strand inside the lake
    let mut sim = fixture_sim(&g);
    let mut wet = None;
    'find: for dy in 0..120 {
        for dx in 0..200 {
            let (px, py) = (150.0 + dx as f64, 1000.0 + dy as f64);
            if sim.world.water_shown_at(px, py) {
                wet = Some((px + 40.0, py + 20.0));
                break 'find;
            }
        }
    }
    let (wx, wy) = wet.expect("the fixture lake shows water");
    assert!(sim.world.water_shown_at(wx, wy), "inside the lake");
    sim.rng = Rng::new(5);
    let free = GrowthOpts {
        free: true,
        ..GrowthOpts::default()
    };
    assert!(shot::add_growth_from(&mut sim, wx, wy - 60.0, 270.0, 120.0, &free));
    for _ in 0..200 {
        run(&mut sim, &idle());
    }
    let pts = sim.mycelium.growths.last().unwrap().branches[0].points.clone();
    let p = pts
        .iter()
        .find(|p| sim.world.water_shown_at(p.x, p.y))
        .copied()
        .expect("the strand crosses the lake");
    assert_eq!(knots::quick_node(&mut sim, p.x, p.y), Err(NodeReject::InWater));

    // Blocked: myc_add_knot refusing although the earlier checks pass (a knot just inside its
    // 1.6 r ring but outside the 2 r check cannot exist, so force it through the knot list)
    let mut sim = fixture_sim(&g);
    let hit = growth::get_mycelium_hit(&mut sim, x, y, 12.0);
    let r = sim.mycelium.knot_radius;
    assert_eq!(
        knots::apply_node_at(&mut sim, hit.x, hit.y),
        Ok(()),
        "a fresh spot takes a node"
    );
    // a second application at the same spot skips quick_node's own ring check: add_knot refuses
    assert_eq!(knots::apply_node_at(&mut sim, hit.x + r, hit.y), Err(NodeReject::Blocked));
}

#[test]
fn rmb_ignored_while_aiming() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    let (x, y) = ON_HYPHA;
    let n0 = sim.mycelium.knots.len();
    let food0 = sim.state.state.food;
    sim.drain_events();
    // LMB held (aiming): the right click does nothing, not even a rejection
    let mut inp = rmb(x, y);
    inp.lmb.down = true;
    run(&mut sim, &inp);
    // over the HUD: nothing either
    let mut inp = rmb(x, y);
    inp.ui_captured = true;
    run(&mut sim, &inp);
    // while the game is blocked (pause): nothing
    sim.pause_open = true;
    run(&mut sim, &rmb(x, y));
    sim.pause_open = false;
    assert_eq!(sim.mycelium.knots.len(), n0);
    assert_eq!(sim.state.state.food, food0);
    assert!(sim
        .drain_events()
        .iter()
        .all(|e| !matches!(e, SimEvent::NodeRejected { .. } | SimEvent::Resource { .. })));
    // a real aim in progress is not disturbed by the ignored right click
    let k0 = sim.mycelium.knots[0].clone();
    let press = StepInput {
        cursor: Some((k0.x + 1.0, k0.y + 1.0)),
        lmb: ButtonState {
            down: true,
            pressed: true,
            released: false,
        },
        ..Default::default()
    };
    run(&mut sim, &press);
    assert!(sim.mycelium.dragging);
    let mut inp = rmb(x, y);
    inp.lmb.down = true;
    run(&mut sim, &inp);
    assert!(sim.mycelium.dragging);
    assert_eq!(sim.mycelium.knots.len(), n0);
}

#[test]
fn shots_and_harvest_emit_resource_events() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    sim.rng = Rng::new(4242);
    sim.drain_events();
    let water0 = sim.water_total();
    // the session's first shot
    for inp in g["session_inputs"].as_array().unwrap().iter().take(17) {
        run(&mut sim, &step_input(inp));
    }
    let spent = water0 - sim.water_total();
    assert!(spent > 0.0);
    let ev = sim.drain_events();
    let water: Vec<_> = ev
        .iter()
        .filter_map(|e| match e {
            SimEvent::Resource {
                kind: ResourceKind::Water,
                delta,
                x,
                y,
                ..
            } => Some((*delta, *x, *y)),
            _ => None,
        })
        .collect();
    assert_eq!(water.len(), 1, "{ev:?}");
    assert_eq!(water[0].0, -spent);
    assert_eq!((water[0].1, water[0].2), (0.0, 480.0), "at the shot origin");
    // a growth through food: +n food at the pickup point, one source per growth
    let mut sim = fixture_sim(&g);
    sim.rng = Rng::new(1);
    sim.drain_events();
    let food0 = sim.state.state.food;
    let opts = GrowthOpts {
        free: true,
        ..GrowthOpts::default()
    };
    // straight into the food cluster around (180, 970)..(232, 1051) from (150, 900)
    let dir = myco_core::gml::point_direction(150.0, 900.0, 200.0, 1040.0);
    assert!(shot::add_growth_from(&mut sim, 150.0, 900.0, dir, 160.0, &opts));
    for _ in 0..200 {
        run(&mut sim, &idle());
    }
    let gained = sim.state.state.food - food0;
    let ev = sim.drain_events();
    let food: Vec<_> = ev
        .iter()
        .filter_map(|e| match e {
            SimEvent::Resource {
                kind: ResourceKind::Food,
                delta,
                source,
                ..
            } => Some((*delta, *source)),
            _ => None,
        })
        .collect();
    assert!(gained > 0.0, "the growth collects food");
    assert_eq!(food.iter().map(|e| e.0).sum::<f64>(), gained);
    let src = food[0].1;
    assert!(food.iter().all(|e| e.1 == src), "one stable source per growth");
}

#[test]
fn hyphae_sounds_follow_the_growth() {
    let g = golden("mycelium");
    let mut sim = fixture_sim(&g);
    sim.rng = Rng::new(3);
    sim.drain_events();
    let opts = GrowthOpts::default();
    assert!(shot::add_growth_from(&mut sim, 0.0, 480.0, 270.0, 150.0, &opts));
    let id = sim.mycelium.growths.last().unwrap().id;
    let plan = sim.mycelium.growths.last().unwrap().sound.unwrap().plan;
    let mut steps = 0;
    for _ in 0..400 {
        run(&mut sim, &idle());
        for e in sim.drain_events() {
            if let SimEvent::Audio(myco_core::events::AudioCall::HyphaeStep { id: sid, t }) = e {
                assert_eq!(sid, id);
                steps += 1;
                assert_eq!(t, steps as f64);
            }
        }
    }
    assert_eq!(steps as f64, plan.done_at(true));
    assert!(sim.mycelium.active.is_empty());
}
