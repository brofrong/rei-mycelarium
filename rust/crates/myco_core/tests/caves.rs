//! Caves, relics, perks and bones (Task 6) against the LÖVE port.
//! Golden topics (rust/golden/scripts): caves_art (masks, outlines, sprite sizes), caves_s31607
//! and caves_s7 (layout, chunks, queries, helpers, the yellow cave; captured with only the
//! stone entry points neutralized, ruling R19), caves_touch (rewards on the Exploration
//! fixture). The cave caches fill lazily, so every test replays the golden script's calls in
//! the same order.
mod common;

use common::{f, golden, i};
use myco_core::sim::Sim;
use myco_core::state::{GameMode, GameState, MushroomSave};
use myco_core::world::caves::{self, Cave, CaveHost, CaveKind, HostKey, YellowSpot};
use myco_core::world::{bones, perks, relics, stones, WorldRt};
use serde_json::{json, Value};

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

/// Structural JSON equality: numbers within 1e-9 (relative above 1), the golden "undefined"
/// equals null, object key order ignored, an empty struct written as [] equals {}.
fn json_eq(path: &str, got: &Value, want: &Value) {
    match (got, want) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            let scale = 1f64.max(a.abs()).max(b.abs());
            assert!((a - b).abs() <= 1e-9 * scale, "{path}: {a} vs {b}");
        }
        (Value::Null, Value::String(s)) | (Value::String(s), Value::Null) if s == "undefined" => {}
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: length\n got {got}\nwant {want}");
            for (k, (x, y)) in a.iter().zip(b).enumerate() {
                json_eq(&format!("{path}[{k}]"), x, y);
            }
        }
        (Value::Object(a), Value::Array(b)) | (Value::Array(b), Value::Object(a))
            if a.is_empty() && b.is_empty() => {}
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

fn b(v: bool) -> Value {
    json!(if v { 1 } else { 0 })
}

// ---------------------------------------------------------------------------------------------
// art

#[test]
fn cave_art_matches_love() {
    let g = golden("caves_art");
    let masks: Vec<&Value> = g["caves"]
        .as_array()
        .unwrap()
        .iter()
        .chain(g["relics"].as_array().unwrap())
        .collect();
    for m in masks {
        let id = m["id"].as_str().unwrap();
        let mask = caves::cave_mask(id).unwrap_or_else(|| panic!("no mask {id}"));
        assert_eq!(mask.w as f64, f(&m["w"]), "{id}.w");
        assert_eq!(mask.h as f64, f(&m["h"]), "{id}.h");
        assert_eq!(
            mask.sprite,
            [f(&m["sprite"][0]), f(&m["sprite"][1])],
            "{id}.sprite"
        );
        for (y, row) in m["spans"].as_array().unwrap().iter().enumerate() {
            let want: Vec<f64> = row.as_array().unwrap().iter().map(f).collect();
            let mut got = Vec::new();
            let mut x = 0;
            while x < mask.w {
                if mask.solid(x as i64, y as i64) {
                    let x0 = x;
                    while x + 1 < mask.w && mask.solid(x as i64 + 1, y as i64) {
                        x += 1;
                    }
                    got.push(x0 as f64);
                    got.push(x as f64);
                }
                x += 1;
            }
            assert_eq!(got, want, "{id}.spans[{y}]");
        }
        let pts = |v: &Vec<[f64; 2]>| to_json(v);
        json_eq(&format!("{id}.outline"), &pts(&mask.outline), &m["outline"]);
        let branches: Vec<Value> = mask
            .branches
            .iter()
            .map(|br| json!({"at": br.at, "delay": br.delay, "points": pts(&br.points)}))
            .collect();
        json_eq(
            &format!("{id}.branches"),
            &Value::Array(branches),
            &m["branches"],
        );
        let stems: Vec<Value> = mask.stems.iter().map(pts).collect();
        json_eq(&format!("{id}.stems"), &Value::Array(stems), &m["stems"]);
    }
    for c in g["caves"].as_array().unwrap() {
        let def = caves::cave_def(c["id"].as_str().unwrap());
        assert_eq!(def.kind.id(), c["id"].as_str().unwrap());
        assert_eq!(def.left, f(&c["left"]));
        assert_eq!(def.w, f(&c["def_w"]));
        json_eq("feet", &to_json(&def.feet), &c["feet"]);
    }
    let ids: Vec<&str> = caves::cave_defs().iter().map(|d| d.kind.id()).collect();
    assert_eq!(ids, ["blue", "purple", "red", "green", "yellow"]);
    for (k, want) in bones::bone_kinds()
        .iter()
        .zip(g["bones"].as_array().unwrap())
    {
        assert_eq!(k.id, want["id"].as_str().unwrap());
        assert_eq!(k.weight, f(&want["weight"]));
        assert_eq!(k.near, f(&want["near"]));
        assert_eq!(k.far, f(&want["far"]));
        assert_eq!(k.deep_share, f(&want["deep_share"]));
        assert_eq!(
            k.size(),
            [f(&want["size"][0]), f(&want["size"][1])],
            "{}",
            k.id
        );
    }
    for (id, w) in [("t_rex", 567.0), ("ufo", 620.0)] {
        let d = relics::relic_def(id).unwrap();
        assert_eq!(d.w, w);
        assert_eq!(d.kind.id(), id);
    }
    assert!(relics::relic_def("nope").is_none());
}

// ---------------------------------------------------------------------------------------------
// layout for two seeds

fn rect_json(r: &Cave) -> Value {
    json!({"id": r.id, "def": r.kind.id(), "k": r.k, "x1": r.x1, "y1": r.y1, "x2": r.x2, "y2": r.y2})
}

fn rects_json(w: &mut WorldRt) -> Value {
    Value::Array(w.cave_rects().iter().map(rect_json).collect())
}

fn host_json(h: &CaveHost) -> Value {
    let key = match &h.key {
        HostKey::Obstacle(k) => json!(k),
        HostKey::Cluster(s) => json!(s),
    };
    json!({"key": key, "x1": h.x1, "y1": h.y1, "x2": h.x2, "y2": h.y2, "used": b(h.used)})
}

fn blue_state_json(w: &WorldRt) -> Value {
    match &w.caves.blue_state {
        None => json!("undefined"),
        Some(st) => {
            let lane = |l: &caves::BlueLane| json!({"dir": l.dir, "n": l.n, "gap": l.gap, "cursor": l.cursor});
            json!({
                "mazes": st.mazes.iter().map(host_json).collect::<Vec<_>>(),
                "walls": st.walls.iter().map(host_json).collect::<Vec<_>>(),
                "left": lane(&st.left), "right": lane(&st.right),
            })
        }
    }
}

fn boxes_json(list: &[caves::Zone]) -> Value {
    Value::Array(
        list.iter()
            .map(|z| json!([z.id, z.x1, z.y1, z.x2, z.y2]))
            .collect(),
    )
}

fn obstacles_eq(w: &WorldRt, want: &Value, what: &str) {
    let o = &w.params.obstacles;
    for (side, s) in [("left", &o.left), ("right", &o.right)] {
        let ws = &want[side];
        assert_eq!(s.count as f64, f(&ws["count"]), "{what}.{side}.count");
        assert_eq!(s.cursor, f(&ws["cursor"]), "{what}.{side}.cursor");
        let list = ws["list"].as_array().unwrap();
        assert_eq!(s.list.len(), list.len(), "{what}.{side}.list");
        for (k, (a, e)) in s.list.iter().zip(list).enumerate() {
            let p = format!("{what}.{side}.list[{k}]");
            assert_eq!(a.key as f64, f(&e["key"]), "{p}.key");
            assert_eq!(a.kind, e["kind"].as_str().unwrap(), "{p}.kind");
            assert_eq!(a.x1, f(&e["x1"]), "{p}.x1");
            assert_eq!(a.x2, f(&e["x2"]), "{p}.x2");
            assert_eq!(a.cx.map(|v| v as f64), e.get("cx").map(f), "{p}.cx");
            assert_eq!(a.c1, e.get("c1").map(f), "{p}.c1");
            assert_eq!(a.c2, e.get("c2").map(f), "{p}.c2");
        }
    }
}

fn chunk_eq(w: &WorldRt, key: &str, wc: &Value, what: &str) {
    let cx: i64 = key[1..].parse().unwrap();
    let ch = w
        .chunk(cx)
        .unwrap_or_else(|| panic!("{what}: missing {key}"));
    let p = format!("{what}.{key}");
    assert_eq!(ch.rows as f64, f(&wc["rows"]), "{p}.rows");
    let got: Vec<f64> = ch.tiles.iter().map(|&v| v as f64).collect();
    assert_eq!(got, unrle(&wc["tiles"]), "{p}.tiles");
    assert_eq!(ch.amount, unrle(&wc["amount"]), "{p}.amount");
    json_eq(&format!("{p}.foods"), &to_json(&ch.foods), &wc["foods"]);
    json_eq(&format!("{p}.lakes"), &to_json(&ch.lakes), &wc["lakes"]);
    json_eq(&format!("{p}.bones"), &to_json(&ch.bones), &wc["bones"]);
}

fn chunks_eq(w: &WorldRt, want: &Value, what: &str) {
    let want = want.as_object().unwrap();
    assert_eq!(w.chunks.len(), want.len(), "{what}: chunk count");
    for (key, wc) in want {
        chunk_eq(w, key, wc, what);
    }
}

fn run_plan(w: &mut WorldRt, plan: &Value) {
    for p in plan.as_array().unwrap() {
        w.chunk_reach(i(&p[0]), i(&p[1]));
    }
}

fn probe_eq(w: &mut WorldRt, want: &Value, what: &str) {
    for (k, q) in want.as_array().unwrap().iter().enumerate() {
        let (x, y) = (f(&q[0]), f(&q[1]));
        let got = json!([
            x,
            y,
            w.tile_at(x, y).code(),
            b(caves::world_cave_solid_any(w, x, y)),
            b(caves::world_cave_hits(w, x, y, x, y, None)),
            b(caves::world_cave_hits(
                w,
                x - 30.0,
                y - 30.0,
                x + 30.0,
                y + 30.0,
                Some(10.0)
            )),
        ]);
        json_eq(&format!("{what}[{k}]"), &got, q);
    }
}

fn spot_json(s: &Option<YellowSpot>) -> Value {
    match s {
        None => json!("undefined"),
        Some(s) => json!([s.x, s.y, s.top, s.bottom, s.cluster]),
    }
}

fn caves_seed_match(topic: &str, seed: i64) {
    let g = golden(topic);
    let mut sim = Sim::new_game(GameMode::Exploration, "Golden", Some(seed));
    let w = &mut sim.world;
    assert_eq!(w.params.seed, seed);
    chunks_eq(w, &g["init_chunks"], "init_chunks");
    obstacles_eq(w, &g["obstacles"], "obstacles");
    json_eq("rects_init", &rects_json(w), &g["rects_init"]);
    json_eq("blue_state", &blue_state_json(w), &g["blue_state"]);
    let clusters: Vec<Value> = stones::world_rock_clusters(w)
        .iter()
        .map(|c| {
            json!({"id": c.id, "tier": c.tier.id(), "key": c.key, "col": c.col, "row": c.row,
                   "r": c.r, "c1": c.c1, "c2": c.c2, "x1": c.x1, "y1": c.y1, "x2": c.x2, "y2": c.y2})
        })
        .collect();
    json_eq("clusters", &Value::Array(clusters), &g["clusters"]);
    let bb: Vec<caves::Zone> = bones::world_big_bones(w).iter().map(|b| b.zone()).collect();
    json_eq("big_bones", &boxes_json(&bb), &g["big_bones"]);
    let rl: Vec<Value> = relics::world_relics(w)
        .iter()
        .map(|r| {
            json!({"id": r.id, "relic": r.relic.id(), "def": r.relic.id(), "k": r.k,
                   "x1": r.x1, "y1": r.y1, "x2": r.x2, "y2": r.y2})
        })
        .collect();
    json_eq("relics", &Value::Array(rl), &g["relics"]);
    let zones = caves::world_clear_zones(w).to_vec();
    json_eq("zones", &boxes_json(&zones), &g["zones"]);

    run_plan(w, &g["plan"]);
    chunks_eq(w, &g["chunks"], "chunks");
    json_eq(
        "chunk_index",
        &to_json(&w.params.chunk_index),
        &g["chunk_index"],
    );
    probe_eq(w, &g["probe"], "probe");

    // pure helpers
    for q in g["cave_range"].as_array().unwrap() {
        let r = myco_core::balance::wld_arr("caves_purple_x");
        assert_eq!(caves::cave_range(r, f(&q[0])), f(&q[1]), "cave_range");
    }
    let kinds = bones::bone_kinds();
    for q in g["bone_weight"].as_array().unwrap() {
        let got = bones::bone_kind_weight(&kinds[i(&q[0]) as usize], f(&q[1]));
        json_eq("bone_weight", &json!(got), &q[2]);
    }
    for q in g["bone_pick"].as_array().unwrap() {
        let p = bones::bone_kind_pick(f(&q[0]), f(&q[1]));
        json_eq(
            "bone_pick",
            &json!([f(&q[0]), f(&q[1]), p.kind, p.share]),
            q,
        );
    }
    let args = [
        ([10.0, 20.0], 0.0, 18.0, 3.0),
        ([100.5, 40.25], 135.0, 55.0, 77.0),
        ([0.0, 0.0], 359.0, 4.0, 9.0),
    ];
    for (k, (s, d, l, seed)) in args.iter().enumerate() {
        let pts = caves::cave_branch_points(*s, *d, *l, *seed);
        json_eq("branch_points", &to_json(&pts), &g["branch_points"][k]);
    }
    let host = |key: i64, x1: f64, x2: f64, used: bool| CaveHost {
        key: HostKey::Obstacle(key),
        x1,
        y1: 480.0,
        x2,
        y2: 3000.0,
        used,
        cx: 0.0,
        cy: 0.0,
        r: 0.0,
    };
    let hs = [
        host(1, 3000.0, 6000.0, false),
        host(2, -9000.0, -5000.0, false),
        host(3, 12000.0, 15000.0, true),
        host(4, 20000.0, 26000.0, false),
        host(5, -2000.0, -1000.0, false),
    ];
    for q in g["host_pick"].as_array().unwrap() {
        let h = caves::cave_host_pick(&hs, w.params.start_x, f(&q[0]), f(&q[1]));
        let key = h.map(|k| match &hs[k].key {
            HostKey::Obstacle(n) => *n,
            HostKey::Cluster(_) => -2,
        });
        assert_eq!(key.unwrap_or(-1) as f64, f(&q[2]), "host_pick {q}");
    }
    let mk = |r: [f64; 4]| Cave {
        id: "x".into(),
        kind: CaveKind::Blue,
        k: 1.0,
        x1: r[0],
        y1: r[1],
        x2: r[2],
        y2: r[3],
    };
    type Place = (CaveKind, f64, f64, f64, f64, Vec<[f64; 4]>, Vec<[f64; 4]>);
    let places: [Place; 5] = [
        (
            CaveKind::Blue,
            5000.0,
            1500.0,
            600.0,
            3000.0,
            vec![],
            vec![],
        ),
        (
            CaveKind::Purple,
            5000.5,
            100.0,
            600.0,
            3000.0,
            vec![[4700.0, 1000.0, 5300.0, 1800.0]],
            vec![],
        ),
        (
            CaveKind::Red,
            5000.0,
            2900.0,
            600.0,
            3000.0,
            vec![[4700.0, 600.0, 5300.0, 3000.0]],
            vec![[4000.0, 600.0, 4800.0, 3000.0]],
        ),
        (
            CaveKind::Green,
            0.0,
            1500.0,
            600.0,
            700.0,
            vec![],
            vec![[-5000.0, 0.0, 5000.0, 4000.0]],
        ),
        (
            CaveKind::Yellow,
            100.0,
            1500.0,
            600.0,
            3000.0,
            vec![[-10000.0, 0.0, 10000.0, 4000.0]],
            vec![],
        ),
    ];
    for (k, (kind, cx, cy, lo, hi, list, extra)) in places.into_iter().enumerate() {
        let mut l: Vec<Cave> = list.into_iter().map(mk).collect();
        let p = caves::cave_place(&mut l, kind, "t", cx, cy, lo, hi, &extra);
        let got = match p {
            None => json!("undefined"),
            Some(p) => json!([p.x1, p.y1, p.x2, p.y2, p.k, l.len()]),
        };
        json_eq(&format!("place[{k}]"), &got, &g["place"][k]);
    }

    // the yellow cave
    let untouched = [
        [0.0, 600.0, 100.0, 700.0, 0.0],
        [0.0, 50000.0, 100.0, 50100.0, 80.0],
        [9000.0, 3000.0, 9500.0, 3500.0, 152.0],
        [-40000.0, 9000.0, -39000.0, 9500.0, 152.0],
        [30000.0, 600.0, 31000.0, 900.0, 0.0],
    ];
    for (k, a) in untouched.iter().enumerate() {
        let got = caves::world_area_untouched(w, a[0], a[1], a[2], a[3], a[4]);
        json_eq("untouched", &json!([b(got)]), &g["untouched"][k]);
    }
    let tips: [[Option<f64>; 2]; 5] = [
        [None, None],
        [None, Some(3000.0)],
        [Some(-5000.0), None],
        [Some(-40000.0), Some(40000.0)],
        [Some(-1e6), Some(1e6)],
    ];
    for (k, t) in tips.iter().enumerate() {
        let tip = |dir: f64| (if dir < 0.0 { t[0] } else { t[1] }).map(|x| (x, 600.0));
        let s = caves::cave_yellow_pick_with(w, tip);
        json_eq(
            &format!("yellow_pick[{k}]"),
            &spot_json(&s),
            &g["yellow_pick"][k],
        );
    }
    sim.title_open = true;
    sim.intro_active = false;
    sim.pause_open = false;
    let need = myco_core::balance::wld("caves_yellow_after_minutes") * 60.0 * 60.0;
    sim.state.state.play_frames = need as i64 - 3;
    let steps = g["yellow_step"].as_array().unwrap();
    for (k, want) in steps.iter().enumerate() {
        if k == 1 {
            sim.title_open = false;
        }
        caves::cave_yellow_step(&mut sim);
        let got = json!([
            sim.state.state.play_frames,
            spot_json(&sim.world.caves.yellow)
        ]);
        json_eq(&format!("yellow_step[{k}]"), &got, want);
    }
    // the saved global follows the live spot
    let saved = sim
        .state
        .state
        .cave_yellow
        .clone()
        .expect("cave_yellow saved");
    assert_eq!(f(&saved["x"]), sim.world.caves.yellow.as_ref().unwrap().x);
    let w = &mut sim.world;
    json_eq("rects_yellow", &rects_json(w), &g["rects_yellow"]);
    let bb: Vec<caves::Zone> = bones::world_big_bones(w).iter().map(|b| b.zone()).collect();
    json_eq("big_bones_yellow", &boxes_json(&bb), &g["big_bones_yellow"]);
    let zones = caves::world_clear_zones(w).to_vec();
    json_eq("zones_yellow", &boxes_json(&zones), &g["zones_yellow"]);
    json_eq(
        "blue_state_yellow",
        &blue_state_json(w),
        &g["blue_state_yellow"],
    );
    run_plan(w, &g["plan_yellow"]);
    for (key, wc) in g["chunks_yellow"].as_object().unwrap() {
        chunk_eq(w, key, wc, "chunks_yellow");
    }
    probe_eq(w, &g["probe_yellow"], "probe_yellow");

    // fresh caches (a new process) with the yellow spot already set
    let spot = w.caves.yellow.clone();
    w.caves = caves::CaveRt::default();
    w.caves.yellow = spot;
    json_eq("rects_reload", &rects_json(w), &g["rects_reload"]);
    let bb: Vec<caves::Zone> = bones::world_big_bones(w).iter().map(|b| b.zone()).collect();
    json_eq("big_bones_reload", &boxes_json(&bb), &g["big_bones_reload"]);
    let zones = caves::world_clear_zones(w).to_vec();
    json_eq("zones_reload", &boxes_json(&zones), &g["zones_reload"]);
    let n = relics::world_relics(w).len();
    assert_eq!(n, g["relics_reload"].as_array().unwrap().len());

    // a synthetic yellow spot on the T-rex
    let tb = &g["big_bones"][0];
    let sy = (f(&tb[2]) + f(&tb[4])) * 0.5;
    w.caves = caves::CaveRt::default();
    w.caves.yellow = Some(YellowSpot {
        x: (f(&tb[1]) + f(&tb[3])) * 0.5,
        y: sy,
        top: sy - 1500.0,
        bottom: sy + 1500.0,
        cluster: "synthetic".into(),
    });
    json_eq("rects_on_bones", &rects_json(w), &g["rects_on_bones"]);
    let bb: Vec<caves::Zone> = bones::world_big_bones(w).iter().map(|b| b.zone()).collect();
    json_eq(
        "big_bones_on_bones",
        &boxes_json(&bb),
        &g["big_bones_on_bones"],
    );
    let zones = caves::world_clear_zones(w).to_vec();
    json_eq("zones_on_bones", &boxes_json(&zones), &g["zones_on_bones"]);
}

#[test]
fn caves_match_love_seed_31607() {
    caves_seed_match("caves_s31607", 31607);
}

#[test]
fn caves_match_love_seed_7() {
    caves_seed_match("caves_s7", 7);
}

// ---------------------------------------------------------------------------------------------
// rewards on the fixture

fn fixture_state(name: &str) -> GameState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    let mut st: GameState = serde_json::from_str(text.trim_end_matches('\0')).unwrap();
    st.apply_load_defaults();
    st
}

fn keys(m: &std::collections::BTreeMap<String, Value>) -> Value {
    json!(m.keys().collect::<Vec<_>>())
}

fn state_json(sim: &Sim) -> Value {
    let c = &sim.world.caves;
    let st = &sim.state.state;
    json!({
        "taken": keys(&st.cave_taken),
        "perks": keys(&st.cave_perks),
        "relics": keys(&st.relic_taken),
        "flash": c.flash.keys().collect::<Vec<_>>(),
        "wraps": c.wrap.iter().map(|(k, w)| json!([k, w.from_x, w.from_y])).collect::<Vec<_>>(),
        "popups": c.popups.iter().map(|p| json!([p.text, p.x, p.y])).collect::<Vec<_>>(),
        "timers": sim.state.mushrooms.iter().map(|m| m.timer).collect::<Vec<_>>(),
        "burst": match c.burst_pending { None => json!("false"), Some((x, y)) => json!([x, y]) },
        "time_mult": perks::perk_time_mult(&sim.state),
        "cost_mult": perks::perk_cost_mult(&sim.state),
    })
}

fn mushrooms(timers: &[f64]) -> Vec<MushroomSave> {
    timers
        .iter()
        .map(|&t| MushroomSave {
            timer: t,
            ..Default::default()
        })
        .collect()
}

#[test]
fn cave_touch_rewards_match_love() {
    let g = golden("caves_touch");
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    sim.title_open = false;
    sim.intro_active = false;
    sim.pause_open = false;
    sim.state.mushrooms = mushrooms(&[100.0, 37.5, 0.0]);
    json_eq("rects", &rects_json(&mut sim.world), &g["rects"]);
    obstacles_eq(&sim.world, &g["obstacles"], "obstacles");
    let spot = caves::cave_yellow_pick(&mut sim);
    caves::set_cave_yellow(&mut sim, spot);
    json_eq(
        "rects_yellow",
        &rects_json(&mut sim.world),
        &g["rects_yellow"],
    );

    let mut touches = Vec::new();
    caves::caves_touch_at(&mut sim, 123456.0, 7890.0);
    relics::relics_touch_at(&mut sim, 123456.0, 7890.0);
    let sweep = |sim: &mut Sim, r: [f64; 4], cave: bool, touches: &mut Vec<Value>| {
        let y = (r[1] + r[3]) * 0.5 + 0.5;
        let mut x = r[0] - 60.0;
        while x <= r[2] + 60.0 {
            if cave {
                caves::caves_touch_at(sim, x, y);
            } else {
                relics::relics_touch_at(sim, x, y);
            }
            touches.push(json!([
                x,
                y,
                sim.state.state.cave_taken.len(),
                sim.state.state.relic_taken.len()
            ]));
            x += 11.0;
        }
    };
    let rects: Vec<[f64; 4]> = sim
        .world
        .cave_rects()
        .iter()
        .map(|r| [r.x1, r.y1, r.x2, r.y2])
        .collect();
    for r in &rects {
        sweep(&mut sim, *r, true, &mut touches);
    }
    json_eq("after_caves", &state_json(&sim), &g["after_caves"]);
    let rl: Vec<(String, [f64; 4])> = relics::world_relics(&mut sim.world)
        .iter()
        .map(|r| (r.id.clone(), [r.x1, r.y1, r.x2, r.y2]))
        .collect();
    for (k, (id, r)) in rl.iter().enumerate() {
        json_eq(
            "relic_rects",
            &json!([id, r[0], r[1], r[2], r[3]]),
            &g["relic_rects"][k],
        );
        sweep(&mut sim, *r, false, &mut touches);
    }
    json_eq("touches", &Value::Array(touches), &g["touches"]);
    json_eq("after_relics", &state_json(&sim), &g["after_relics"]);
    let r0 = rects[0];
    caves::caves_touch_at(&mut sim, (r0[0] + r0[2]) * 0.5, (r0[1] + r0[3]) * 0.5);
    json_eq("after_again", &state_json(&sim), &g["after_again"]);
    // nothing taken gives a sound; every take one water_touch
    let sounds = sim
        .drain_events()
        .into_iter()
        .filter(|e| matches!(e, myco_core::events::SimEvent::Sound(s) if s.name == "water_touch"))
        .count();
    assert_eq!(sounds, 7, "water_touch sounds: 5 caves + 2 relics");

    let mut wrapped = Vec::new();
    let ids = ["relic_t_rex", "relic_ufo", "t_rex", "ufo", "nope"];
    for id in ids {
        wrapped.push(json!([id, b(relics::relic_wrapped(&sim, id))]));
    }
    for w in sim.world.caves.wrap.values_mut() {
        w.born = -1e9;
    }
    for id in ids {
        wrapped.push(json!([id, b(relics::relic_wrapped(&sim, id))]));
    }
    json_eq("relic_wrapped", &Value::Array(wrapped), &g["relic_wrapped"]);

    let mut k = 0;
    for r in &rects {
        let mut y = r[1] - 70.0;
        while y <= r[3] + 70.0 {
            let mut x = r[0] - 70.0;
            while x <= r[2] + 70.0 {
                let (px, py) = (x + 0.37, y + 0.61);
                let h = caves::caves_wrap_hit(&mut sim, px, py, 40.0);
                json_eq(
                    &format!("wrap_hit[{k}]"),
                    &json!([px, py, b(h.hit), h.x, h.y, h.dist]),
                    &g["wrap_hit"][k],
                );
                k += 1;
                x += 41.0;
            }
            y += 37.0;
        }
    }
    assert_eq!(k, g["wrap_hit"].as_array().unwrap().len());

    for (k, q) in g["hover"].as_array().unwrap().iter().enumerate() {
        let (vx, vy) = (r0[0] - 300.0, r0[1] - 200.0);
        let tip = caves::cave_hover_tip(&mut sim, f(&q[0]), f(&q[1]), 24.0, 64.0, vx, vy);
        json_eq(&format!("hover[{k}]"), &json!([f(&q[0]), f(&q[1]), tip]), q);
    }

    sim.state.state.cave_perks.clear();
    sim.state.state.cave_taken.clear();
    sim.world.caves.popups.clear();
    sim.state.mushrooms = mushrooms(&[90.0]);
    let dn: Vec<String> = (0..8).map(|_| caves::perk_debug_next(&mut sim)).collect();
    json_eq("debug_next", &to_json(&dn), &g["debug_next"]);
    json_eq("after_debug", &state_json(&sim), &g["after_debug"]);

    // re-taking caves whose perk is held
    sim.state.state.cave_taken.clear();
    sim.state.mushrooms = mushrooms(&[80.0]);
    let all = sim.world.cave_rects().to_vec();
    for r in all
        .iter()
        .filter(|r| matches!(r.kind, CaveKind::Yellow | CaveKind::Purple))
    {
        caves::caves_touch_at(
            &mut sim,
            (r.x1 + r.x2) * 0.5 + 0.25,
            (r.y1 + r.y2) * 0.5 + 0.5,
        );
    }
    json_eq("after_retake", &state_json(&sim), &g["after_retake"]);

    // where a touch starts to count
    let mut probe = Vec::new();
    let mut probe_at = |sim: &mut Sim, cave: bool, x: f64, y: f64| {
        sim.state.state.cave_taken.clear();
        sim.state.state.relic_taken.clear();
        if cave {
            caves::caves_touch_at(sim, x, y);
        } else {
            relics::relics_touch_at(sim, x, y);
        }
        let n = sim.state.state.cave_taken.len() + sim.state.state.relic_taken.len();
        probe.push(json!([x, y, n]));
    };
    let boxes: Vec<(bool, [f64; 4])> = all
        .iter()
        .map(|r| (true, [r.x1, r.y1, r.x2, r.y2]))
        .chain(rl.iter().map(|(_, r)| (false, *r)))
        .collect();
    for (cave, r) in boxes {
        let (mx, my) = ((r[0] + r[2]) * 0.5 + 0.25, (r[1] + r[3]) * 0.5 + 0.5);
        let mut d = -35.0;
        while d <= 70.0 {
            probe_at(&mut sim, cave, mx, r[1] + d);
            probe_at(&mut sim, cave, r[0] + d, my);
            probe_at(&mut sim, cave, mx + 37.0, r[3] - d);
            d += 1.5;
        }
    }
    json_eq("touch_probe", &Value::Array(probe), &g["touch_probe"]);
}

#[test]
fn perk_costs_match_love() {
    let g = golden("caves_touch");
    let mut st = GameState::default();
    for (id, v) in [
        ("a", json!(true)),
        ("b", json!(false)),
        ("c", json!(1)),
        ("d", json!(0)),
        ("e", json!(0.7)),
        ("f", json!(0.3)),
    ] {
        st.state.cave_perks.insert(id.into(), v);
    }
    for q in g["perk_has"].as_array().unwrap() {
        let id = q[0].as_str().unwrap();
        json_eq("perk_has", &json!([id, b(perks::perk_has(&st, id))]), q);
    }
    for q in g["mult"].as_array().unwrap() {
        st.state.mode = if q[0] == "survival" {
            GameMode::Survival
        } else {
            GameMode::Exploration
        };
        st.state.cave_perks.clear();
        for id in q[1].as_str().unwrap().split('+').filter(|s| !s.is_empty()) {
            st.state.cave_perks.insert(id.into(), json!(true));
        }
        let got = json!([
            q[0],
            q[1],
            perks::perk_time_mult(&st),
            perks::perk_cost_mult(&st)
        ]);
        json_eq("mult", &got, q);
        assert_eq!(
            perks::perk_cost_mult(&st),
            myco_core::balance::perk_cost_mult(&st)
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Rust-only checks

#[test]
fn neutralized_caves_leave_the_world_alone() {
    caves::neutralize_caves(true);
    let mut sim = Sim::new_game(GameMode::Exploration, "N", Some(31607));
    assert!(sim.world.caves().is_empty());
    assert!(!caves::world_cave_hits(
        &mut sim.world,
        -1e9,
        -1e9,
        1e9,
        1e9,
        None
    ));
    caves::neutralize_caves(false);
    let mut sim = Sim::new_game(GameMode::Exploration, "N", Some(31607));
    assert_eq!(sim.world.cave_rects().len(), 4);
}

#[test]
fn cave_take_on_load_starts_fresh_runtime_state() {
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let r = sim.world.cave_rects()[0].clone();
    let taken = caves::caves_touch_at(&mut sim, (r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5 + 0.5);
    assert_eq!(taken, vec![r.id.clone()]);
    assert!(caves::cave_taken(&sim.state, &r.id));
    let (state, _) = sim.to_save();
    let sim2 = Sim::from_save(state).unwrap();
    assert!(caves::cave_taken(&sim2.state, &r.id));
    assert!(sim2.world.caves.wrap.is_empty() && sim2.world.caves.popups.is_empty());
}

#[test]
fn wrap_hits_wait_for_the_wrap_animation() {
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let r = sim.world.cave_rects()[0].clone();
    let (x, y) = ((r.x1 + r.x2) * 0.5, r.y1 - 5.0);
    assert!(
        !caves::caves_wrap_hit(&mut sim, x, y, 60.0).hit,
        "untaken cave"
    );
    caves::cave_take(&mut sim, &r, None, None);
    assert!(
        !caves::caves_wrap_hit(&mut sim, x, y, 60.0).hit,
        "wrap still growing"
    );
    // cave_wrap_ms = 2600 ms on the 60 Hz sim clock
    sim.frame += 156;
    assert_eq!(
        caves::now_ms(&sim) - sim.world.caves.wrap[&r.id].born,
        2600.0
    );
    let h = caves::caves_wrap_hit(&mut sim, x, y, 60.0);
    assert!(h.hit && h.dist <= 60.0, "{h:?}");
}
