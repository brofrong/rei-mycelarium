//! Mushrooms (Task 9) against the LÖVE port. Golden topic `mushrooms`
//! (rust/golden/scripts/mushrooms.lua): sprite metadata and the type catalogue, costs over
//! perks/upgrades/experience counts, `mushroom_add` rolls on the Exploration fixture, then the
//! colony fixture `tests/fixtures/mushrooms.json` (written by the same script) run for 600 frames
//! of `mushrooms_step` + `drops_step` + `repeller_clouds_step` with a hedgehog and a boar walking
//! into repellers; stinky and repeller helpers. Plus Rust-only tests for the resource events,
//! the placement preview (ruling R22) and `MushroomPlaced`.
mod common;

use common::{f, golden, i};
use myco_core::events::{ResourceKind, SimEvent};
use myco_core::life::{self, Boar, Hedgehog};
use myco_core::mushrooms::{self, consumptor, repeller, stinky, types, HitBox, MushroomKind};
use myco_core::rng::Rng;
use myco_core::sim::{Command, Sim, StepInput};
use myco_core::state::{GameState, MushroomSave};
use serde_json::Value;
use std::collections::HashMap;

fn fixture_state(name: &str) -> GameState {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    let mut st: GameState = serde_json::from_str(text.trim_end_matches('\0')).unwrap();
    st.apply_load_defaults();
    st
}

fn with_view(mut sim: Sim, g: &Value) -> Sim {
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

fn opt(path: &str, a: Option<f64>, want: &Value) {
    match (a, want.as_str()) {
        (None, Some("undefined")) => {}
        (Some(x), None) => close(path, x, f(want)),
        _ => panic!("{path}: {a:?} vs {want}"),
    }
}

fn b(v: bool) -> f64 {
    v as i64 as f64
}

fn set_upg(sim: &mut Sim, id: &str, lvl: i64) {
    sim.state.state.upgrade_levels.insert(id.to_string(), lvl);
}

fn set_perk(sim: &mut Sim, id: &str, on: bool) {
    if on {
        sim.state
            .state
            .cave_perks
            .insert(id.to_string(), true.into());
    } else {
        sim.state.state.cave_perks.remove(id);
    }
}

fn kind(t: i64) -> MushroomKind {
    MushroomKind::from_code(t).unwrap()
}

fn stats_eq(path: &str, sim: &Sim, want: &Value) {
    for w in want.as_array().unwrap() {
        let k = w[0].as_str().unwrap();
        let got = sim.state.state.tasks.stats.get(k).copied().unwrap_or(0.0);
        close(&format!("{path}.{k}"), got, f(&w[1]));
    }
}

#[test]
fn sprites_and_types_match_love() {
    let g = golden("mushrooms");
    for s in g["sprites"].as_array().unwrap() {
        let name = s[0].as_str().unwrap();
        if name == "hedgehog" || name == "boar" {
            continue; // their widths are checked through frame_w below
        }
        let p = types::shroom_sprite(name).unwrap_or_else(|| panic!("no sprite {name}"));
        let got = [
            p.w, p.h, p.xoff, p.yoff, p.bbox_l, p.bbox_t, p.bbox_r, p.bbox_b,
        ];
        for (k, v) in got.iter().enumerate() {
            close(&format!("{name}[{k}]"), *v, f(&s[k + 1]));
        }
        close(
            &format!("{name}.cells"),
            types::shroom_sheet_cells(name),
            f(&s[9]),
        );
        close(
            &format!("{name}.cw"),
            types::mushroom_cell_width(name),
            f(&s[10]),
        );
    }
    close(
        "hedgehog_frame_w",
        life::hedgehog_frame_w(),
        f(&g["frame_w"][0]),
    );
    close("boar_frame_w", life::boar_frame_w(), f(&g["frame_w"][1]));
    for (t, w) in g["types"].as_array().unwrap().iter().enumerate() {
        let k = kind(t as i64);
        let spr = types::mushroom_type_sprite(k);
        assert_eq!(spr, w[0].as_str().unwrap());
        close("variants", types::mushroom_type_variants(k), f(&w[1]));
        assert_eq!(types::mushroom_type_unlock_id(k), w[2].as_str().unwrap());
        assert_eq!(types::mushroom_type_name_key(k), w[3].as_str().unwrap());
        for v in 0..3 {
            close(
                &format!("cell_left {spr} {v}"),
                types::mushroom_cell_left(spr, v as f64),
                f(&w[4][v]),
            );
        }
    }
    close(
        "front",
        repeller::repeller_animal_front(100.0, -1.0, 30.0),
        f(&g["front"]),
    );
}

const LEVELED: [&str; 10] = [
    "digger_length",
    "repeller_regen_cost",
    "repeller_autoregen",
    "predator_digestion",
    "predator_yield",
    "tree_drop_rate",
    "tree_water_yield",
    "parasitism_speed",
    "parasitism_yield",
    "stinky_width",
];
const UNLOCKS: [&str; 8] = [
    "unlock_spore",
    "unlock_predator",
    "unlock_repeller",
    "unlock_tree",
    "unlock_digger",
    "unlock_stinky",
    "unlock_experience",
    "unlock_parasitism",
];

fn cost_rows_eq(path: &str, sim: &Sim, want: &Value) {
    for (t, w) in want.as_array().unwrap().iter().enumerate() {
        let k = kind(t as i64);
        let p = format!("{path}.t{t}");
        close(
            &format!("{p}.base"),
            types::mushroom_type_cost_base(&sim.state, k),
            f(&w[0]),
        );
        close(
            &format!("{p}.wbase"),
            types::mushroom_type_water_cost_base(&sim.state, k),
            f(&w[1]),
        );
        close(
            &format!("{p}.cost"),
            mushrooms::cost(&sim.state, k),
            f(&w[2]),
        );
        close(
            &format!("{p}.water"),
            mushrooms::water_cost(&sim.state, k),
            f(&w[3]),
        );
        close(
            &format!("{p}.afford"),
            b(mushrooms::affordable(sim, k)),
            f(&w[4]),
        );
        close(
            &format!("{p}.unlocked"),
            b(mushrooms::unlocked(&sim.state, k)),
            f(&w[5]),
        );
    }
}

/// The golden's colony build (section 3) on the Exploration fixture; returns the Sim after it.
fn build_colony(g: &Value) -> Sim {
    let mut sim = with_view(
        Sim::from_save(fixture_state("exploration.json")).unwrap(),
        g,
    );
    let maxl: HashMap<String, i64> = g["max_levels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (r[0].as_str().unwrap().to_string(), i(&r[1])))
        .collect();
    for id in maxl.keys() {
        close(
            &format!("max {id}"),
            myco_core::progress::upgrades::upg_max_level(&sim.state, id),
            maxl[id] as f64,
        );
    }
    // costs, locked then unlocked
    let costs = g["costs"].as_array().unwrap();
    let mut row = 0;
    for tag in ["locked", "unlocked"] {
        if tag == "unlocked" {
            for id in UNLOCKS {
                set_upg(&mut sim, id, 1);
            }
        }
        for perk in [false, true] {
            set_perk(&mut sim, "green", perk);
            for lvl in 0..=maxl["digger_length"] {
                set_upg(&mut sim, "digger_length", lvl);
                for food in [0.0, 45.0, 100000.0] {
                    sim.state.state.food = food;
                    let w = &costs[row];
                    assert_eq!(w["tag"].as_str().unwrap(), tag);
                    assert_eq!(i(&w["digger"]), lvl);
                    cost_rows_eq(&format!("costs[{row}]"), &sim, &w["rows"]);
                    row += 1;
                }
            }
        }
        set_perk(&mut sim, "green", false);
        set_upg(&mut sim, "digger_length", 0);
    }
    assert_eq!(row, costs.len());
    // helpers over levels and the yellow perk
    let helpers = g["helpers"].as_array().unwrap();
    let mut row = 0;
    for yellow in [false, true] {
        set_perk(&mut sim, "yellow", yellow);
        for id in LEVELED {
            for lvl in 0..=maxl[id] {
                set_upg(&mut sim, id, lvl);
                let w = &helpers[row];
                assert_eq!(w[1].as_str().unwrap(), id);
                let st = &sim.state;
                let got = [
                    types::repeller_regen_price(st),
                    types::repeller_autoregen_frames(&sim),
                    types::repeller_max_charges(st),
                    types::experience_period(&sim),
                    life::predator_period(&sim),
                    life::predator_yield(&sim),
                    life::tree_drop_period(&sim),
                    life::tree_drop_water(&sim),
                ];
                for (k, v) in got.iter().enumerate() {
                    close(&format!("helpers[{row}] {id} {lvl} [{k}]"), *v, f(&w[3][k]));
                }
                row += 1;
            }
            set_upg(&mut sim, id, 0);
        }
    }
    set_perk(&mut sim, "yellow", false);
    assert_eq!(row, helpers.len());
    // the colony
    for l in g["levels"].as_array().unwrap() {
        set_upg(&mut sim, l[0].as_str().unwrap(), i(&l[1]));
    }
    sim.state.state.food = 100000.0;
    sim.state.state.water_free = 5000.0;
    sim.rng = Rng::new(4242);
    let added = &g["added"];
    for (n, w) in added["list"].as_array().unwrap().iter().enumerate() {
        let idx = mushrooms::mushroom_add(&mut sim, kind(i(&w["type"])), f(&w["x"]), f(&w["y"]));
        assert_eq!(idx as i64, i(&w["idx"]));
        let m = &sim.state.mushrooms[idx];
        let p = format!("added[{n}]");
        assert_eq!(m.variant, i(&w["variant"]), "{p}.variant");
        close(&format!("{p}.timer"), m.timer, f(&w["timer"]));
        close(&format!("{p}.charges"), m.charges, f(&w["charges"]));
        close(&format!("{p}.born"), m.born.unwrap(), f(&w["born"]));
        let flies = w["flies"].as_array().unwrap();
        assert_eq!(m.flies.len(), flies.len(), "{p}.flies");
        for (q, (fl, wf)) in m.flies.iter().zip(flies).enumerate() {
            let got = [fl.angle, fl.radius, fl.speed, fl.bob];
            for (k, v) in got.iter().enumerate() {
                close(&format!("{p}.fly{q}[{k}]"), *v, f(&wf[k]));
            }
        }
    }
    close("added.probe", sim.rng.random(1.0), f(&added["probe"]));
    stats_eq("added.stats", &sim, &added["stats"]);
    for (k, perk) in [false, true].into_iter().enumerate() {
        set_perk(&mut sim, "green", perk);
        cost_rows_eq(&format!("costs_after[{k}]"), &sim, &g["costs_after"][k]);
    }
    set_perk(&mut sim, "green", false);
    sim
}

#[test]
fn costs_match_love() {
    let g = golden("mushrooms");
    let mut sim = build_colony(&g);
    // the build emitted one grow sound per mushroom
    let grows = sim
        .drain_events()
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::Audio(myco_core::events::AudioCall::MushroomGrow { .. })
            )
        })
        .count();
    assert_eq!(grows, g["added"]["list"].as_array().unwrap().len());
}

/// The colony fixture as load_game leaves it, with the golden's view.
fn colony_sim(g: &Value) -> Sim {
    with_view(Sim::from_save(fixture_state("mushrooms.json")).unwrap(), g)
}

fn box_eq(path: &str, bx: &HitBox, w: &Value, off: usize) {
    let got = [bx.x1, bx.y1, bx.x2, bx.y2];
    for (k, v) in got.iter().enumerate() {
        close(&format!("{path}[{}]", off + k), *v, f(&w[off + k]));
    }
}

fn boxes_eq(path: &str, sim: &Sim, want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(sim.state.mushrooms.len(), w.len(), "{path}: count");
    for (n, wm) in w.iter().enumerate() {
        let p = format!("{path}[{n}]");
        box_eq(&p, &mushrooms::mushroom_hit_box(sim, n), wm, 0);
        box_eq(&p, &mushrooms::mushroom_hit_box_unmirrored(sim, n), wm, 4);
        let m = &sim.state.mushrooms[n];
        close(
            &format!("{p}.variant"),
            types::mushroom_variant(m),
            f(&wm[8]),
        );
        assert_eq!(
            types::mushroom_sprite_now(m),
            wm[9].as_str().unwrap(),
            "{p}.sprite"
        );
        close(
            &format!("{p}.grow"),
            types::shroom_grow_scale(sim, m),
            f(&wm[10]),
        );
        let id = mushrooms::mushroom_id(sim, n);
        close(
            &format!("{p}.index"),
            mushrooms::mushroom_index_of(sim, id) as f64,
            f(&wm[11]),
        );
    }
}

fn progress_eq(path: &str, sim: &Sim, want: &Value) {
    for (n, w) in want.as_array().unwrap().iter().enumerate() {
        let p = format!("{path}[{n}]");
        opt(
            &format!("{p}.tree"),
            types::tree_shroom_progress(sim, n),
            &w[0],
        );
        opt(
            &format!("{p}.xp"),
            types::experience_progress(sim, n),
            &w[1],
        );
        opt(
            &format!("{p}.cons"),
            consumptor::consumptor_progress(sim, n),
            &w[2],
        );
        opt(
            &format!("{p}.regen"),
            types::repeller_regen_progress(sim, n),
            &w[3],
        );
        opt(&format!("{p}.pred"), life::predator_progress(sim, n), &w[4]);
    }
}

/// One frame of the golden's loop (frame `k`): injections, then the three steps.
fn run_frame(sim: &mut Sim, k: u64) {
    sim.frame = k;
    let sy = myco_core::balance::wld("surface_height_px");
    if k == 100 {
        let x = sim.state.mushrooms[3].x - life::hedgehog_frame_w() * 0.4 - 30.0;
        sim.life.animals.push(Hedgehog {
            x,
            y: sy,
            dir: 1.0,
            speed: 0.0,
            frame: 0.0,
            frame_timer: 4.0,
            carrying: -1,
        });
    }
    if k == 300 {
        let breach = life::boar_frame_w() * myco_core::balance::vis("boar_snout_dx");
        let x = sim.state.mushrooms[4].x + breach + 40.0;
        sim.life.boars.push(Boar {
            x,
            y: sy,
            dir: -1.0,
            fleeing: false,
        });
    }
    mushrooms::step(sim);
    life::drops_step(sim);
    repeller::repeller_clouds_step(sim);
}

fn mushrooms_snap_eq(path: &str, sim: &Sim, want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(sim.state.mushrooms.len(), w.len(), "{path}: count");
    for (n, (m, wm)) in sim.state.mushrooms.iter().zip(w).enumerate() {
        let p = format!("{path}[{n}]");
        assert_eq!(m.kind, i(&wm["type"]), "{p}.type");
        close(&format!("{p}.x"), m.x, f(&wm["x"]));
        close(&format!("{p}.y"), m.y, f(&wm["y"]));
        close(&format!("{p}.timer"), m.timer, f(&wm["timer"]));
        close(&format!("{p}.active"), b(m.active), f(&wm["active"]));
        close(&format!("{p}.charges"), m.charges, f(&wm["charges"]));
        close(&format!("{p}.flash"), m.flash, f(&wm["flash"]));
        opt(&format!("{p}.regen"), m.regen_timer, &wm["regen"]);
        let fly0 = wm["fly0"].as_array().unwrap();
        if let Some(fl) = m.flies.first() {
            close(&format!("{p}.fly.angle"), fl.angle, f(&fly0[0]));
            close(&format!("{p}.fly.bob"), fl.bob, f(&fly0[1]));
        } else {
            assert!(fly0.is_empty(), "{p}: flies");
        }
    }
}

fn pool_eq(path: &str, m: &MushroomSave, w: &Value) {
    close(&format!("{path}.due"), m.stench_due, f(&w["due"]));
    match (m.stench_live, &w["live"]) {
        (None, Value::String(s)) if s == "undefined" => {}
        (Some(l), Value::Bool(x)) => assert_eq!(l, *x, "{path}.live"),
        (a, x) => panic!("{path}.live: {a:?} vs {x}"),
    }
    let Some(pool) = stinky::pool_of(m) else {
        assert_eq!(i(&w["n"]), -1, "{path}: no pool");
        return;
    };
    assert_eq!(pool.n as i64, i(&w["n"]), "{path}.n");
    for (q, wf) in w["first"].as_array().unwrap().iter().enumerate() {
        let got = [
            pool.side[q],
            pool.rise[q],
            pool.offset[q],
            pool.speed[q],
            pool.age[q],
            pool.phase[q],
            pool.size[q],
            pool.heat[q],
            pool.drift[q],
            pool.px[q],
            pool.py[q],
            pool.pix[q],
            pool.colour[q],
            pool.alpha[q],
        ];
        for (k, v) in got.iter().enumerate() {
            close(&format!("{path}.p{q}[{k}]"), *v, f(&wf[k]));
        }
    }
    let n = pool.n;
    let sums = [
        pool.px[..n].iter().sum::<f64>(),
        pool.py[..n].iter().sum::<f64>(),
        pool.colour[..n].iter().sum::<f64>(),
        pool.alpha[..n].iter().sum::<f64>(),
    ];
    for (k, v) in sums.iter().enumerate() {
        let want = f(&w["sums"][k]);
        let scale = 1f64.max(v.abs()).max(want.abs());
        assert!(
            (v - want).abs() <= 1e-7 * scale,
            "{path}.sum{k}: {v} vs {want}"
        );
    }
}

fn clouds_eq(path: &str, sim: &Sim, want: &Value) {
    let w = want.as_array().unwrap();
    let cl = &sim.mushrooms.repel_clouds;
    assert_eq!(cl.len(), w.len(), "{path}: count");
    for (n, (c, wc)) in cl.iter().zip(w).enumerate() {
        let p = format!("{path}[{n}]");
        for (k, v) in [
            ("x", c.x),
            ("y", c.y),
            ("ground", c.ground),
            ("dir", c.dir),
            ("flash", c.flash),
        ] {
            close(&format!("{p}.{k}"), v, f(&wc[k]));
        }
        assert_eq!(c.puffs.len() as i64, i(&wc["puffs"]), "{p}.puffs");
        assert_eq!(c.specks.len() as i64, i(&wc["specks"]), "{p}.specks");
        let sums = [
            c.puffs.iter().map(|q| q.x).sum::<f64>(),
            c.puffs.iter().map(|q| q.y).sum::<f64>(),
            c.puffs.iter().map(|q| q.size).sum::<f64>(),
            c.specks.iter().map(|q| q.x).sum::<f64>(),
            c.specks.iter().map(|q| q.y).sum::<f64>(),
        ];
        for (k, v) in sums.iter().enumerate() {
            let want = f(&wc["sums"][k]);
            let scale = 1f64.max(v.abs()).max(want.abs());
            assert!(
                (v - want).abs() <= 1e-7 * scale,
                "{p}.sum{k}: {v} vs {want}"
            );
        }
        if let Some(q) = c.puffs.first() {
            let got = [
                q.x,
                q.y,
                q.vx,
                q.vy,
                q.size,
                q.grow,
                q.life,
                q.total,
                q.colour,
                b(q.dust),
            ];
            for (k, v) in got.iter().enumerate() {
                close(&format!("{p}.p0[{k}]"), *v, f(&wc["p0"][k]));
            }
        }
    }
}

fn life_eq(path: &str, sim: &Sim, s: &Value) {
    let an = s["animals"]["animals"].as_array().unwrap();
    assert_eq!(sim.life.animals.len(), an.len(), "{path}.animals");
    for (a, w) in sim.life.animals.iter().zip(an) {
        close(&format!("{path}.animal.x"), a.x, f(&w[0]));
        close(&format!("{path}.animal.dir"), a.dir, f(&w[1]));
    }
    let bo = s["animals"]["boars"].as_array().unwrap();
    assert_eq!(sim.life.boars.len(), bo.len(), "{path}.boars");
    for (a, w) in sim.life.boars.iter().zip(bo) {
        close(&format!("{path}.boar.x"), a.x, f(&w[0]));
        close(&format!("{path}.boar.dir"), a.dir, f(&w[1]));
        close(&format!("{path}.boar.flee"), b(a.fleeing), f(&w[2]));
    }
    let dr = s["drops"].as_array().unwrap();
    assert_eq!(sim.life.drops.len(), dr.len(), "{path}.drops");
    for (d, w) in sim.life.drops.iter().zip(dr) {
        let got = [d.x, d.y, d.target_y, d.speed];
        for (k, v) in got.iter().enumerate() {
            close(&format!("{path}.drop[{k}]"), *v, f(&w[k]));
        }
    }
    opt(
        &format!("{path}.kick"),
        Some(sim.mushrooms.repel_kick),
        &s["kick"],
    );
}

fn hosts_eq(sim: &Sim, want: &Value) {
    let mut keys: Vec<i64> = sim.forest.chunks.keys().copied().collect();
    keys.sort_by_key(|cx| format!("s{cx}"));
    let mut got = Vec::new();
    for cx in keys {
        let ch = &sim.forest.chunks[&cx];
        for t in &ch.trees {
            got.push(("t", t.x, t.consume, t.consumed));
        }
        for s in &ch.stumps {
            got.push(("s", s.x, s.consume, s.consumed));
        }
    }
    let w = want.as_array().unwrap();
    assert_eq!(got.len(), w.len(), "hosts: count {got:?}");
    for (n, (g, wh)) in got.iter().zip(w).enumerate() {
        assert_eq!(g.0, wh[0].as_str().unwrap(), "host[{n}]");
        close(&format!("host[{n}].x"), g.1, f(&wh[1]));
        close(&format!("host[{n}].consume"), g.2, f(&wh[2]));
        close(&format!("host[{n}].consumed"), b(g.3), f(&wh[3]));
    }
}

#[test]
fn production_over_600_frames_matches_love() {
    let g = golden("mushrooms");
    let mut sim = colony_sim(&g);
    // loaded colony: hosts resolved, boxes, progress, consumption
    for (n, w) in g["hosts0"].as_array().unwrap().iter().enumerate() {
        opt(&format!("hosts0[{n}]"), consumptor::host_x(&sim, n), w);
    }
    boxes_eq("boxes0", &sim, &g["boxes0"]);
    progress_eq("progress0", &sim, &g["progress0"]);
    // the golden's hosts: the eaten tree, the kept tree, the stump (consumptors 14, 16, 15)
    for (n, w) in g["consume"].as_array().unwrap().iter().enumerate() {
        let host = consumptor::mushroom_host(&sim, [14, 16, 15][n]).unwrap();
        close(
            &format!("consume[{n}].frames"),
            consumptor::consume_frames(&sim, host),
            f(&w[0]),
        );
        close(
            &format!("consume[{n}].food"),
            consumptor::consume_food(&sim, host),
            f(&w[1]),
        );
        assert_eq!(
            consumptor::consumptors_on(&sim, host).len() as i64,
            i(&w[2])
        );
    }
    sim.rng = Rng::new(99);
    let frames = g["frames"].as_array().unwrap();
    let snaps = g["snaps"].as_array().unwrap();
    let mut s = 0;
    for k in 0..600u64 {
        run_frame(&mut sim, k);
        let w = &frames[k as usize];
        let st = &sim.state.state;
        let got = [
            st.food,
            st.water_free,
            st.upgrade_points as f64,
            sim.state.mushrooms.len() as f64,
            sim.life.drops.len() as f64,
            sim.mushrooms.repel_clouds.len() as f64,
            sim.mushrooms.stinky_live_count,
            sim.mushrooms.stinky_live_last,
        ];
        for (q, v) in got.iter().enumerate() {
            close(&format!("frame {k} [{q}]"), *v, f(&w[q]));
        }
        if s < snaps.len() && i(&snaps[s]["frame"]) == k as i64 {
            let p = format!("snap {k}");
            mushrooms_snap_eq(&format!("{p}.mushrooms"), &sim, &snaps[s]["mushrooms"]);
            life_eq(&p, &sim, &snaps[s]);
            s += 1;
        }
    }
    assert_eq!(s, snaps.len());
    let fin = &g["final"];
    hosts_eq(&sim, &fin["hosts"]);
    stats_eq("final.stats", &sim, &fin["stats"]);
    boxes_eq("final.boxes", &sim, &fin["boxes"]);
    progress_eq("final.progress", &sim, &fin["progress"]);
    close("final.probe", sim.rng.random(1.0), f(&fin["probe"]));
}

#[test]
fn production_emits_one_event_per_source_per_frame() {
    let g = golden("mushrooms");
    let mut sim = colony_sim(&g);
    sim.rng = Rng::new(99);
    sim.drain_events();
    let frames = g["frames"].as_array().unwrap();
    let mut prev = [100000.0, 5000.0, 0.0];
    let mut produced = [0usize; 3];
    for k in 0..600u64 {
        run_frame(&mut sim, k);
        let mut seen = HashMap::new();
        let mut sums = [0.0; 3];
        for e in sim.drain_events() {
            if let SimEvent::Resource {
                kind,
                delta,
                source,
                ..
            } = e
            {
                let q = match kind {
                    ResourceKind::Food => 0,
                    ResourceKind::Water => 1,
                    ResourceKind::UpgradePoints => 2,
                };
                assert!(delta > 0.0, "frame {k}: production is positive");
                *seen.entry((q, source)).or_insert(0) += 1;
                sums[q] += delta;
                produced[q] += 1;
            }
        }
        for ((q, src), n) in &seen {
            assert_eq!(
                *n, 1,
                "frame {k}: kind {q} source {src:#x} emitted {n} events"
            );
        }
        let w = &frames[k as usize];
        for q in 0..3 {
            let now = f(&w[q]);
            close(&format!("frame {k} delta[{q}]"), sums[q], now - prev[q]);
            prev[q] = now;
        }
    }
    // predators, consumption (food); tree drops (water); experience (points)
    assert!(produced.iter().all(|n| *n >= 2), "{produced:?}");
}

#[test]
fn stinky_and_repeller_match_love() {
    let g = golden("mushrooms");
    let mut sim = colony_sim(&g);
    sim.rng = Rng::new(99);
    let snaps = g["snaps"].as_array().unwrap();
    let stinky_idx: Vec<usize> = (0..sim.state.mushrooms.len())
        .filter(|&n| sim.state.mushrooms[n].kind == 5)
        .collect();
    let ids: Vec<u64> = stinky_idx
        .iter()
        .map(|&n| mushrooms::mushroom_id(&sim, n))
        .collect();
    let mut s = 0;
    for k in 0..600u64 {
        run_frame(&mut sim, k);
        if s < snaps.len() && i(&snaps[s]["frame"]) == k as i64 {
            for (q, id) in ids.iter().enumerate() {
                let n = mushrooms::mushroom_index_of(&sim, *id);
                let m = &sim.state.mushrooms[n as usize];
                pool_eq(&format!("snap {k} pool {q}"), m, &snaps[s]["pools"][q]);
            }
            clouds_eq(&format!("snap {k} clouds"), &sim, &snaps[s]["clouds"]);
            s += 1;
        }
    }
    // shapes, funnel, beam hits, guards per stinky_width level (on the final colony)
    for (lvl, w) in g["stinky_shapes"].as_array().unwrap().iter().enumerate() {
        set_upg(&mut sim, "stinky_width", lvl as i64);
        let p = format!("shape {lvl}");
        let sh = stinky::stinky_shape(&sim);
        close(&format!("{p}.base"), sh.base, f(&w["base"]));
        close(&format!("{p}.open"), sh.open, f(&w["open"]));
        close(&format!("{p}.flare"), sh.flare, f(&w["flare"]));
        close(&format!("{p}.top"), sh.top, f(&w["top"]));
        close(
            &format!("{p}.top_at"),
            stinky::stinky_top_at(lvl as f64),
            f(&w["top_at"]),
        );
        for (q, dy) in [-10.0, 0.0, 33.5, 120.0, 400.0, 2000.0].iter().enumerate() {
            close(
                &format!("{p}.funnel{q}"),
                stinky::stinky_funnel_radius(&sh, *dy),
                f(&w["funnel"][q]),
            );
        }
        let sy = myco_core::balance::wld("surface_height_px");
        let mut q = 0;
        for px in [
            -160.0, -120.0, -100.0, -80.0, -40.0, 0.0, 290.0, 300.0, 330.0, 2990.0,
        ] {
            for py in [sy + 20.0, sy - 20.0, sy - 60.0, sy - 200.0, sy - 600.0] {
                close(
                    &format!("{p}.hit({px},{py})"),
                    b(stinky::stinky_beam_hits(&sim, px, py, 6.0)),
                    f(&w["hits"][q]),
                );
                q += 1;
            }
        }
        close(
            &format!("{p}.reach"),
            stinky::stinky_squirrel_reach(&sim),
            f(&w["reach"]),
        );
        for (n, wg) in w["guards"].as_array().unwrap().iter().enumerate() {
            close(
                &format!("{p}.guard{n}"),
                b(stinky::stinky_guards(&sim, n)),
                f(wg),
            );
        }
    }
    set_upg(&mut sim, "stinky_width", 2);
    // cap heights and grow scale (current_time of the last frame)
    for w in g["caps"].as_array().unwrap() {
        let mut m = MushroomSave {
            kind: 5,
            variant: i(&w[0]),
            ..MushroomSave::default()
        };
        m.born = w[1].as_f64().map(|d| sim.now_ms() + d);
        close("cap", stinky::stinky_cap_height(&sim, &m), f(&w[2]));
        close("grow", types::shroom_grow_scale(&sim, &m), f(&w[3]));
    }
    let sh = &g["shade"];
    let (hot, deep) = stinky::stinky_colours();
    close("hot", hot, f(&sh["hot"]));
    close("deep", deep, f(&sh["deep"]));
    let tab = stinky::stinky_shade_table(&mut sim.mushrooms, hot, deep).to_vec();
    for (k, w) in sh["table"].as_array().unwrap().iter().enumerate() {
        close(&format!("shade[{k}]"), tab[k], f(w));
    }
    close(
        "wind_target",
        stinky::stinky_wind_target(&sim),
        f(&g["wind"][0]),
    );
    close(
        "wind_shift50",
        stinky::stinky_wind_shift(&sim, 50.0),
        f(&g["wind"][1]),
    );
    close(
        "wind_shift0",
        stinky::stinky_wind_shift(&sim, 0.0),
        f(&g["wind"][2]),
    );
}

/// A spore-mushroom spot the placement check accepts on the Exploration fixture.
fn placeable(sim: &mut Sim) -> (f64, f64) {
    let sy = sim.mycelium.surface_y;
    for x in (-400..=400).step_by(10) {
        let x = x as f64;
        let plan = myco_core::mycelium::place::place_check(sim, MushroomKind::Spore, x, Some(sy));
        if plan.ok {
            return (x, sy);
        }
    }
    panic!("no placeable spot");
}

#[test]
fn placement_preview_and_mushroom_placed() {
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    set_upg(&mut sim, "unlock_spore", 1);
    sim.state.state.food = 1000.0;
    sim.state.state.water_free = 1000.0;
    let (x, y) = placeable(&mut sim);
    let knots = sim.mycelium.knots.len();
    // placement mode: the preview is stored, the right click (QuickNode) is ignored
    let input = StepInput {
        cursor: Some((x, y)),
        placement: Some(MushroomKind::Spore),
        commands: vec![Command::QuickNode { x, y: y + 40.0 }],
        ..StepInput::default()
    };
    sim.step(&input);
    let pv = sim.mushrooms.place_preview.expect("preview");
    assert_eq!(pv.kind, MushroomKind::Spore);
    assert!(pv.plan.ok);
    assert_eq!(pv.plan.x, x);
    assert_eq!(
        sim.mycelium.knots.len(),
        knots,
        "QuickNode ignored in placement mode"
    );
    assert!(sim
        .drain_events()
        .iter()
        .all(|e| !matches!(e, SimEvent::NodeRejected { .. })));
    // a click places the mushroom: MushroomPlaced
    let input = StepInput {
        cursor: Some((x, y)),
        placement: Some(MushroomKind::Spore),
        commands: vec![Command::PlaceMushroom {
            kind: MushroomKind::Spore,
            x,
            y,
        }],
        ..StepInput::default()
    };
    let n = sim.state.mushrooms.len();
    sim.step(&input);
    assert_eq!(sim.state.mushrooms.len(), n + 1);
    let ev = sim.drain_events();
    assert!(ev.iter().any(|e| matches!(
        e,
        SimEvent::MushroomPlaced { kind: MushroomKind::Spore, x: px, .. } if *px == x
    )));
    // out of placement mode: no preview
    sim.step(&StepInput::default());
    assert!(sim.mushrooms.place_preview.is_none());
    // can_build_at reports why
    assert_eq!(
        mushrooms::can_build_at(&mut sim, MushroomKind::Stinky, x, y),
        Err(mushrooms::BuildReject::Locked)
    );
    sim.state.state.food = 0.0;
    assert_eq!(
        mushrooms::can_build_at(&mut sim, MushroomKind::Spore, x + 300.0, y),
        Err(mushrooms::BuildReject::NotAffordable)
    );
}

#[test]
fn colony_survives_save_and_load() {
    let g = golden("mushrooms");
    let mut sim = colony_sim(&g);
    sim.rng = Rng::new(99);
    for k in 0..120 {
        run_frame(&mut sim, k);
    }
    let (state, _) = sim.to_save();
    let text = serde_json::to_string(&state).unwrap();
    let back: GameState = serde_json::from_str(&text).unwrap();
    let sim2 = Sim::from_save(back).unwrap();
    assert_eq!(sim2.state.mushrooms.len(), sim.state.mushrooms.len());
    for n in 0..sim.state.mushrooms.len() {
        let (a, b2) = (&sim.state.mushrooms[n], &sim2.state.mushrooms[n]);
        assert_eq!(a.kind, b2.kind);
        assert_eq!(a.timer, b2.timer);
        assert_eq!(a.regen_timer, b2.regen_timer);
        assert_eq!(a.stench_live, b2.stench_live);
        assert_eq!(
            consumptor::host_x(&sim, n),
            consumptor::host_x(&sim2, n),
            "host of {n}"
        );
    }
}
