//! Forest state (Task 5, rulings R5/R10) against the LÖVE port: obj_forest_controller Create_0
//! and surface chunk generation on a seeded RNG, the pickers, backdrop leaves, leaf_fg_drop,
//! host queries on the loaded Exploration fixture, and the camera step.
//! Golden topics: forest, world_fixture.
mod common;

use common::{f, golden, i};
use myco_core::forest::{self, ForestChunk, HostId, LeafFgPose, StumpRt};
use myco_core::rng::Rng;
use myco_core::sim::{Key, Sim, StepInput};
use myco_core::state::{GameMode, GameState};
use serde_json::{json, Value};

fn num_eq(path: &str, got: f64, want: &Value) {
    assert_eq!(got, f(want), "{path}");
}

fn roots_json(r: &Option<myco_core::state::RootsRect>) -> Value {
    match r {
        None => json!("undefined"),
        Some(r) => json!({"ax": r.ax, "h": r.h, "left": r.left, "top": r.top, "w": r.w}),
    }
}

fn same(path: &str, got: &Value, want: &Value) {
    match (got, want) {
        (Value::Number(a), Value::Number(b)) => assert_eq!(a.as_f64(), b.as_f64(), "{path}"),
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: keys {got} vs {want}");
            for (k, v) in a {
                same(&format!("{path}.{k}"), v, &b[k]);
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: len\n got {got}\nwant {want}");
            for (k, (x, y)) in a.iter().zip(b).enumerate() {
                same(&format!("{path}[{k}]"), x, y);
            }
        }
        _ => assert_eq!(got, want, "{path}"),
    }
}

fn stump_json(s: &StumpRt) -> Value {
    let roots = match &s.roots {
        None => json!("undefined"),
        Some(r) => json!({"flip": r.flip, "x": r.x, "y": r.y, "roots": roots_json(&r.roots)}),
    };
    json!({"x": s.x, "y": s.y, "flip": s.flip, "roots": roots, "part": forest::stump_parts()[s.part].left})
}

fn chunk_json(ch: &ForestChunk) -> Value {
    let trees: Vec<Value> = ch
        .trees
        .iter()
        .map(|t| json!({"x": t.x, "y": t.y, "flip": t.flip, "roots": roots_json(&t.roots), "sprite": t.sprite}))
        .collect();
    let small: Vec<Value> = ch
        .small_trees
        .iter()
        .map(|t| json!({"x": t.x, "y": t.y, "flip": t.flip, "roots": roots_json(&t.roots), "sprite": t.sprite}))
        .collect();
    let parts = |v: &Vec<forest::PartRt>| -> Value {
        Value::Array(
            v.iter()
                .map(|p| json!({"part": p.part, "x": p.x, "y": p.y, "flip": p.flip}))
                .collect(),
        )
    };
    json!({
        "trees": trees, "small_trees": small, "bushes": parts(&ch.bushes), "plants": parts(&ch.plants),
        "decor_trees": parts(&ch.decor_trees),
        "stumps": Value::Array(ch.stumps.iter().map(stump_json).collect()),
    })
}

fn fresh_sim() -> Sim {
    Sim::new_game(GameMode::Exploration, "Forest", Some(31607))
}

#[test]
fn create_and_surface_chunks_match_love() {
    let g = golden("forest");
    for run in g["runs"].as_array().unwrap() {
        let mut sim = fresh_sim();
        assert_eq!(sim.state.state.start_x, f(&g["start_x"]));
        assert_eq!(sim.world.params.chunk_w as f64, f(&g["chunk_w"]));
        sim.rng = Rng::new(i(&run["seed"]));
        sim.forest = Default::default();
        forest::create(&mut sim);
        same(
            "start_stump",
            &stump_json(&sim.forest.start_stump),
            &run["start_stump"],
        );
        num_eq("camera x", sim.camera.x, &run["camera"][0]);
        num_eq("camera y", sim.camera.y, &run["camera"][1]);
        for c in run["chunks"].as_array().unwrap() {
            let cx = i(&c["cx"]);
            let ch = sim.forest.surface_get_chunk(&mut sim.rng, cx).clone();
            same(&format!("chunk {cx}"), &chunk_json(&ch), &c["data"]);
        }
        let fr = &mut sim.forest;
        for (k, p) in run["picks"].as_array().unwrap().iter().enumerate() {
            let k = k + 1;
            assert_eq!(fr.bush_pick(&mut sim.rng) as i64, i(&p[0]), "bush_pick");
            assert_eq!(
                forest::part_pick(&mut sim.rng, forest::plant_parts()) as i64,
                i(&p[1])
            );
            assert_eq!(
                forest::part_pick(&mut sim.rng, forest::decor_tree_parts()) as i64,
                i(&p[2])
            );
            let big = forest::TREE_SPRITES[k % 3];
            let small = forest::SMALL_TREE_SPRITES[k % 3];
            same(
                "roots big",
                &roots_json(&forest::tree_roots_pick(&mut sim.rng, big)),
                &p[3],
            );
            same(
                "roots small",
                &roots_json(&forest::tree_roots_pick(&mut sim.rng, small)),
                &p[4],
            );
        }
        for (q, want) in run["stump_make"].as_array().unwrap().iter().enumerate() {
            let s = sim
                .forest
                .stump_make(&mut sim.rng, q, 100.0 * q as f64 - 150.0);
            let mut got = stump_json(&s);
            got.as_object_mut().unwrap().remove("part");
            same("stump_make", &got, want);
        }
    }
}

#[test]
fn backdrop_leaves_match_love() {
    let g = golden("forest");
    let mut sim = fresh_sim();
    sim.rng = Rng::new(99);
    sim.forest.backdrop_leaves = None;
    sim.state.state.wind[0] = 1.0;
    sim.state.state.wind[1] = 2.5;
    let snaps = g["leaves"].as_array().unwrap();
    let mut s = 0;
    let mut cam = -960.0;
    for step in 1..=400 {
        if step == 150 {
            sim.state.state.wind[0] = -1.0;
        }
        if step == 260 {
            sim.state.state.wind[1] = myco_core::balance::vis("wind_max_strength");
        }
        cam += if step > 200 { 7.0 } else { -3.0 };
        forest::backdrop_leaves_step(&mut sim, cam, 1920.0);
        if step % 50 == 0 || step == 1 {
            let want = &snaps[s];
            assert_eq!(i(&want["step"]), step);
            let bands = sim.forest.backdrop_leaves.as_ref().unwrap();
            for b in 0..2 {
                let got: Vec<Value> = bands[b]
                    .iter()
                    .map(|l| serde_json::to_value(l).unwrap())
                    .collect();
                same(
                    &format!("step {step} band {b}"),
                    &Value::Array(got),
                    &want["bands"][b],
                );
            }
            s += 1;
        }
    }
    assert_eq!(s, snaps.len());
}

#[test]
fn leaf_fg_drop_matches_love() {
    let g = golden("forest");
    assert!(forest::leaf_fg_art_ready());
    for r in g["drop"].as_array().unwrap() {
        let opt = |v: &Value| if v.is_string() { None } else { Some(f(v)) };
        let pose = LeafFgPose {
            turn: opt(&r[0]),
            turned: opt(&r[1]).map(|v| v > 0.5),
            angle: opt(&r[2]),
            mirror: opt(&r[3]),
        };
        assert_eq!(forest::leaf_fg_drop(&pose), f(&r[4]), "drop {r}");
        assert_eq!(forest::leaf_fg_mirror(&pose), f(&r[5]), "mirror {r}");
    }
    let a = golden("world_assets");
    let fr = forest::leaf_fg_frame(&LeafFgPose::default());
    num_eq("rest x1", fr.x1, &a["leaf_fg"]["rest"][0]);
    num_eq(
        "carry_x",
        forest::leaf_fg_carry().0,
        &a["leaf_fg"]["carry_x"],
    );
    num_eq(
        "carry_y",
        forest::leaf_fg_carry().1,
        &a["leaf_fg"]["carry_y"],
    );
}

fn fixture_state(name: &str) -> GameState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(text.trim_end_matches('\0')).unwrap()
}

fn host_json(sim: &Sim, h: Option<HostId>) -> Value {
    match h {
        None => json!("none"),
        Some(HostId::Stump(uid)) => {
            let s = sim.forest.stump(uid).unwrap();
            json!(["stump", s.x, forest::stump_parts()[s.part].left])
        }
        Some(HostId::Tree(uid)) => {
            let t = sim.forest.tree(uid).unwrap();
            json!(["tree", t.x, t.sprite])
        }
    }
}

#[test]
fn fixture_forest_queries_match_love() {
    let g = golden("world_fixture");
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let fg = &g["forest"];
    num_eq("camera x", sim.camera.x, &fg["camera"][0]);
    num_eq("camera y", sim.camera.y, &fg["camera"][1]);
    num_eq("start x", sim.forest.start_stump.x, &fg["start_stump"][0]);
    num_eq(
        "start part",
        forest::stump_parts()[sim.forest.start_stump.part].left,
        &fg["start_stump"][1],
    );
    let mut keys: Vec<String> = sim
        .forest
        .chunks
        .keys()
        .map(|cx| format!("s{cx}"))
        .collect();
    keys.sort();
    same("chunk keys", &json!(keys), &fg["chunk_keys"]);
    for r in g["host_near"].as_array().unwrap() {
        let h = sim.forest.find_host_near(&mut sim.rng, f(&r[0]), f(&r[1]));
        same(&format!("host_near {r}"), &host_json(&sim, h), &r[2]);
    }
    for r in g["tree_near"].as_array().unwrap() {
        let h = sim
            .forest
            .find_tree_near(&mut sim.rng, f(&r[0]), f(&r[1]))
            .map(HostId::Tree);
        same(&format!("tree_near {r}"), &host_json(&sim, h), &r[2]);
    }
    let start = sim.forest.start_stump.clone();
    for (k, want) in g["slots"].as_array().unwrap().iter().enumerate() {
        let s = forest::stump_slot_at(&start, k);
        same("slot", &json!([s.x, s.y, s.face]), want);
    }
    for r in g["clearing"].as_array().unwrap() {
        assert_eq!(
            sim.forest.surface_in_start_clearing(f(&r[0])) as i64,
            i(&r[1]),
            "{r}"
        );
    }
    let starts = sim.forest.start_stumps();
    for r in g["near_stump"].as_array().unwrap() {
        let got = forest::surface_near_stump(&starts, f(&r[0]), f(&r[1]));
        assert_eq!(got as i64, i(&r[2]), "{r}");
    }
    // the save ids save_game hands out: trees then stumps, chunk keys sorted
    let ids = forest::save_ids(&sim.forest);
    let start_uid = sim.forest.start_stump.uid;
    assert_eq!(
        ids[&start_uid], 1,
        "fixture: tree 0 and the start stump 1 in s-1"
    );
}

#[test]
fn remove_host_takes_it_out_of_its_chunk() {
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let h = sim
        .forest
        .find_host_near(&mut sim.rng, -933.0, 5.0)
        .expect("tree at -933");
    sim.forest.remove_host(&mut sim.rng, h);
    assert_ne!(
        sim.forest.find_host_near(&mut sim.rng, -933.0, 5.0),
        Some(h)
    );
}

#[test]
fn camera_step_follows_keys_with_acceleration() {
    let mut sim = fresh_sim();
    sim.title_open = false;
    sim.intro_active = false;
    let x0 = sim.camera.x;
    let input = StepInput {
        keys_down: vec![Key::Right],
        ..Default::default()
    };
    forest::step(&mut sim, &input);
    let speed = myco_core::balance::vis("camera_speed");
    let accel = myco_core::gml::clamp(myco_core::balance::vis("camera_accel"), 0.01, 1.0);
    let vx = if speed >= 0.25 { accel * speed } else { speed };
    assert_eq!(sim.camera.vx, vx);
    assert_eq!(sim.camera.x, x0 + vx);
    assert_eq!(sim.camera.view_x, (x0 + vx).floor());
    // the title screen freezes the camera input
    sim.title_open = true;
    let v1 = sim.camera.vx;
    forest::step(&mut sim, &input);
    assert_eq!(
        sim.camera.vx,
        if v1.abs() >= 0.25 {
            v1 + (0.0 - v1) * accel
        } else {
            0.0
        }
    );
}
