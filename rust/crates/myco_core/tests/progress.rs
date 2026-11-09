//! Progress: upgrades, skill tree layout, tasks, goals, tutorial, against
//! `rust/golden/data/progress.json` (tools/golden.sh progress, MYCO_SEED=31607).

mod common;

use common::{assert_close, f, golden, i};
use myco_core::events::SimEvent;
use myco_core::progress::goals::{self, Goal, TextMeasure};
use myco_core::progress::skilltree::{self, UpgKind};
use myco_core::progress::tasks::{self, Block, TaskId};
use myco_core::progress::tutorial::{self, Tip, TipState};
use myco_core::progress::upgrades::{self, BuyReject, UpgState};
use myco_core::progress::{self as progress};
use myco_core::sim::{Command, Sim, StepInput};
use myco_core::state::{FoodSave, GameMode, GameState, LakeSave, MushroomSave, WorldChunkSave};
use serde_json::Value;
use std::collections::BTreeMap;

const TOL: f64 = 1e-9;

fn fresh() -> Sim {
    let mut sim = Sim::new_game(GameMode::Exploration, "Progress", None);
    upgrades::init(&mut sim);
    tutorial::init(&mut sim);
    tasks::init(&mut sim);
    sim
}

/// A Lua value that was `undefined` / absent encodes as `[]`.
fn none(v: &Value) -> bool {
    v.is_null() || v.as_array().is_some_and(|a| a.is_empty())
}

fn s(v: &Value) -> &str {
    v.as_str().unwrap_or_else(|| panic!("not a string: {v}"))
}

fn b(v: &Value) -> bool {
    v.as_bool().unwrap_or_else(|| panic!("not a bool: {v}"))
}

/// Object entries without the `__obj` marker the script adds to keep empty tables objects.
fn entries(v: &Value) -> Vec<(&String, &Value)> {
    v.as_object()
        .unwrap_or_else(|| panic!("not an object: {v}"))
        .iter()
        .filter(|(k, _)| k.as_str() != "__obj")
        .collect()
}

/// A Lua array that went through `obj()`: keys "1".."n".
fn obj_array(v: &Value) -> Vec<&Value> {
    let mut e: Vec<(usize, &Value)> = entries(v)
        .into_iter()
        .map(|(k, v)| (k.parse().unwrap(), v))
        .collect();
    e.sort_by_key(|(k, _)| *k);
    e.into_iter().map(|(_, v)| v).collect()
}

fn set_levels(st: &mut GameState, levels: &[(&str, i64)]) {
    st.state.upgrade_levels = BTreeMap::from([("origin".to_string(), 1)]);
    for (k, v) in levels {
        st.state.upgrade_levels.insert(k.to_string(), *v);
    }
}

fn mode_of(name: &str) -> GameMode {
    match name {
        "exploration" => GameMode::Exploration,
        "survival" => GameMode::Survival,
        _ => panic!("mode {name}"),
    }
}

#[test]
fn skill_tree_layout_matches_love() {
    let g = golden("progress");
    let defs = g["defs"].as_array().unwrap();
    let layout = skilltree::layout();
    assert_eq!(layout.len(), defs.len());
    for (n, d) in layout.iter().zip(defs) {
        assert_eq!(n.id, s(&d["id"]));
        assert_eq!(n.trunk.as_str(), s(&d["trunk"]), "{}", n.id);
        assert_eq!(n.kind.as_str(), s(&d["kind"]), "{}", n.id);
        assert_eq!(n.parent, s(&d["parent"]), "{}", n.id);
        assert_eq!(n.req, f(&d["req"]), "{}", n.id);
        assert_eq!(n.tier, f(&d["tier"]), "{}", n.id);
        assert_eq!((n.px, n.py), (f(&d["px"]), f(&d["py"])), "{}", n.id);
        match n.via {
            None => assert!(none(&d["bx"]) && none(&d["by"]), "{}", n.id),
            Some((bx, by)) => assert_eq!((bx, by), (f(&d["bx"]), f(&d["by"])), "{}", n.id),
        }
        assert_eq!(n.available, b(&d["available"]), "{}", n.id);
        assert_eq!(n.icon.unwrap_or(""), s(&d["icon"]), "{}", n.id);
        let key = |v: &Value| {
            if none(v) {
                String::new()
            } else {
                s(v).to_string()
            }
        };
        assert_eq!(n.max_key, key(&d["max_key"]), "{}", n.id);
        assert_eq!(n.step_key, key(&d["step_key"]), "{}", n.id);
        assert_eq!(n.cost_key, key(&d["cost_key"]), "{}", n.id);
        let (x, y) = skilltree::upg_tree_pos(n);
        assert_close(x, f(&d["pos"][0]), TOL);
        assert_close(y, f(&d["pos"][1]), TOL);
        match skilltree::upg_tree_via(n) {
            None => assert_eq!(d["via"], "none", "{}", n.id),
            Some((x, y)) => {
                assert_close(x, f(&d["via"][0]), TOL);
                assert_close(y, f(&d["via"][1]), TOL);
            }
        }
        let kids: Vec<&str> = skilltree::upg_children(n.id).iter().map(|c| c.id).collect();
        let want: Vec<&str> = obj_array(&d["children"]).into_iter().map(s).collect();
        assert_eq!(kids, want, "{}", n.id);
        assert_close(
            skilltree::tree_hypha_length_fallback(n.id),
            f(&d["hypha_len"]),
            TOL,
        );
        assert_eq!(skilltree::upg_def(n.id).map(|x| x.id), Some(n.id));
    }
    assert!(skilltree::upg_def("nope").is_none());
    assert_eq!(layout[0].kind, UpgKind::Root);
}

#[test]
fn upg_bonus_matches_love() {
    let g = golden("progress");
    let mut st = GameState::default();
    for (mode, rows) in g["levels"].as_object().unwrap() {
        st.state.mode = mode_of(mode);
        for d in skilltree::layout() {
            let row = &rows[d.id];
            set_levels(&mut st, &[]);
            assert_eq!(
                upgrades::upg_max_level(&st, d.id),
                f(&row["max"]),
                "{mode} {}",
                d.id
            );
            for lv in row["levels"].as_array().unwrap() {
                let l = i(&lv["level"]);
                set_levels(&mut st, &[(d.id, l)]);
                let what = format!("{mode} {} level {l}", d.id);
                assert_close(upgrades::upg_bonus(&st, d.id), f(&lv["bonus"]), TOL);
                assert_eq!(upgrades::upg_cost(&st, d.id), f(&lv["cost"]), "{what}");
                assert_eq!(upgrades::upg_is_owned(&st, d.id), b(&lv["owned"]), "{what}");
                assert_eq!(upgrades::upg_level(&st, d.id), l, "{what}");
            }
        }
    }
    assert_eq!(upgrades::upg_bonus(&st, "nope"), 0.0);
    assert_eq!(upgrades::upg_max_level(&st, "nope"), 0.0);
    assert_eq!(upgrades::upg_cost(&st, "nope"), 0.0);
}

/// Rows with `demo_upgrades_only = 0` (the shipped balance); the demo rows are checked by the
/// unit test in `progress::upgrades`, which can override the key.
#[test]
fn buy_costs_and_requirements_match_love() {
    let g = golden("progress");
    let mut st = GameState::default();
    let mut checked = 0;
    for (mode, rows) in g["states"].as_object().unwrap() {
        st.state.mode = mode_of(mode);
        for d in skilltree::layout() {
            for r in rows[d.id].as_array().unwrap() {
                if i(&r["demo"]) != 0 {
                    continue;
                }
                let plv = i(&r["parent_level"]);
                let own = i(&r["own"]);
                let mut lv = vec![(d.id, own)];
                if !d.parent.is_empty() && d.parent != "origin" {
                    lv.push((d.parent, plv));
                }
                if d.parent == "origin" {
                    lv.push(("origin", if plv == 0 { 0 } else { 1 }));
                }
                set_levels(&mut st, &lv);
                st.state.upgrade_points = i(&r["points"]);
                let what = format!("{mode} {} {r}", d.id);
                let can = b(&r["can_buy"]);
                assert_eq!(upgrades::upg_can_buy(&st, d.id), can, "{what}");
                assert_eq!(upgrades::can_buy(&st, d.id).is_ok(), can, "{what}");
                assert_eq!(
                    upgrades::upg_state(&st, d.id).as_str(),
                    s(&r["state"]),
                    "{what}"
                );
                assert_eq!(
                    upgrades::upg_demo_locked(&st, d.id),
                    b(&r["demo_locked"]),
                    "{what}"
                );
                assert_eq!(
                    upgrades::upg_requirement_met(&st, d.id),
                    b(&r["req_met"]),
                    "{what}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 400, "{checked}");

    // reasons
    let mut st = GameState::default();
    set_levels(&mut st, &[]);
    st.state.upgrade_points = 0;
    assert_eq!(upgrades::can_buy(&st, "nope"), Err(BuyReject::Unknown));
    assert_eq!(upgrades::can_buy(&st, "origin"), Err(BuyReject::NotBuyable));
    assert_eq!(
        upgrades::can_buy(&st, "spore_count"),
        Err(BuyReject::RequirementNotMet)
    );
    assert_eq!(
        upgrades::can_buy(&st, "unlock_spore"),
        Err(BuyReject::NotEnoughPoints { need: 2.0 })
    );
    set_levels(&mut st, &[("unlock_spore", 1)]);
    assert_eq!(
        upgrades::can_buy(&st, "unlock_spore"),
        Err(BuyReject::MaxLevel)
    );
    assert_eq!(upgrades::upg_state(&st, "unlock_spore"), UpgState::Maxed);

    // the purchase sequence
    let mut sim = fresh();
    sim.state.state.upgrade_points = 40;
    for r in g["buys"].as_array().unwrap() {
        let id = s(&r["id"]);
        assert_eq!(upgrades::buy(&mut sim, id), b(&r["ok"]), "buy {id}");
        assert_eq!(sim.state.state.upgrade_points, i(&r["points"]), "buy {id}");
        let want: BTreeMap<String, i64> = r["levels"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), i(v)))
            .collect();
        assert_eq!(sim.state.state.upgrade_levels, want, "buy {id}");
    }
    // tree purchases show no popup numbers in the world
    assert!(sim.drain_events().is_empty());
}

#[test]
fn buy_upgrade_command_buys_in_hud_begin_step() {
    let mut sim = fresh();
    sim.state.state.upgrade_points = 5;
    let input = StepInput {
        commands: vec![
            Command::BuyUpgrade {
                id: "unlock_spore".into(),
            },
            Command::BuyUpgrade {
                id: "unlock_digger".into(),
            },
        ],
        ..Default::default()
    };
    sim.step(&input);
    assert_eq!(upgrades::upg_level(&sim.state, "unlock_spore"), 1);
    assert_eq!(upgrades::upg_level(&sim.state, "unlock_digger"), 0);
    assert_eq!(sim.state.state.upgrade_points, 3);
    assert!(
        !sim.drain_events()
            .iter()
            .any(|e| matches!(e, SimEvent::Resource { .. })),
        "a purchase shows no popup number"
    );
}

#[test]
fn upgrade_grow_matches_love() {
    let g = golden("progress");
    let mut sim = fresh();
    sim.state
        .state
        .upgrade_levels
        .insert("unlock_spore".into(), 1);
    sim.state
        .state
        .upgrade_levels
        .insert("spore_count".into(), 2);
    let rows = g["grow"].as_array().unwrap();
    let mut k = 0;
    for frame in 1..=90 {
        upgrades::grow_step(&mut sim);
        if frame % 10 == 0 || frame <= 3 {
            let r = &rows[k];
            assert_eq!(i(&r["frame"]), frame);
            assert_close(
                upgrades::upg_grow(&sim, "unlock_spore"),
                f(&r["spore"]),
                TOL,
            );
            assert_close(upgrades::upg_grow(&sim, "spore_count"), f(&r["count"]), TOL);
            assert_close(upgrades::upg_grow(&sim, "origin"), f(&r["origin"]), TOL);
            k += 1;
        }
    }
    sim.state
        .state
        .upgrade_levels
        .insert("spore_count".into(), 0);
    upgrades::grow_step(&mut sim);
    let r = &rows[k];
    assert_close(upgrades::upg_grow(&sim, "spore_count"), f(&r["count"]), TOL);
    assert_close(
        upgrades::upg_grow(&sim, "unlock_spore"),
        f(&r["spore"]),
        TOL,
    );
    assert_eq!(upgrades::upg_grow(&sim, "nope"), 0.0);
}

fn push_spores(sim: &mut Sim, n: usize) {
    let m = |kind: i64, stolen: bool, carried_by: i64| MushroomSave {
        kind,
        stolen,
        carried_by,
        ..Default::default()
    };
    for _ in 0..n {
        sim.state.mushrooms.push(m(0, false, -1));
    }
    sim.state.mushrooms.push(m(0, true, -1));
    sim.state.mushrooms.push(m(0, false, 3));
    sim.state.mushrooms.push(m(1, false, -1));
}

fn toggle_block(sim: &mut Sim, id: &str) {
    // tasks_block_toggle (the app's, Task 17): flips the saved open flag
    let t = &mut sim.state.state.tasks;
    if id == "tasks" {
        t.open = !t.open;
    } else {
        let open = tasks::block_open(&sim.state, Block::from_key(id).unwrap());
        sim.state
            .state
            .tasks
            .goal_open
            .insert(id.into(), Value::Bool(!open));
    }
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().unwrap() > 0.5,
        Some(_) => true,
    }
}

#[test]
fn tasks_progress_matches_love() {
    let g = golden("progress");
    let want_defs: Vec<&str> = g["task_defs"].as_array().unwrap().iter().map(s).collect();
    let have_defs: Vec<&str> = tasks::tasks_defs().iter().map(|t| t.as_str()).collect();
    assert_eq!(have_defs, want_defs);

    let mut sim = fresh();
    sim.state.state.length_px = 0.0;
    sim.state.state.food = 0.0;
    sim.state.state.water_free = 0.0;
    let mut input = StepInput::default();
    for snap in g["tasks"].as_array().unwrap() {
        let label = s(&snap["label"]);
        let a: Vec<&str> = label.split_once(':').unwrap().1.split(' ').collect();
        let num = |k: usize| a[k].parse::<f64>().unwrap();
        match a[0] {
            "step" => tasks::step(&mut sim, &input),
            "final" => {
                sim.state
                    .state
                    .tutorial
                    .tips
                    .get_mut("final")
                    .unwrap()
                    .state = 1
            }
            "stat" => tasks::stat_add(&mut sim, a[1], if a.len() > 2 { num(2) } else { 1.0 }),
            "close" | "open" => {
                if (a[0] == "open") != tasks::block_open(&sim.state, Block::from_key(a[1]).unwrap())
                {
                    toggle_block(&mut sim, a[1]);
                }
            }
            "upg" => {
                sim.state.state.upgrade_levels.insert(a[1].into(), 1);
            }
            "len" => sim.state.state.length_px = num(1),
            "popup_clear" => sim.state.state.tasks.popup.clear(),
            "spores" => push_spores(&mut sim, num(1) as usize),
            "food" => sim.state.state.food = num(1),
            "water" => sim.state.state.water_free += num(1),
            "perks" => {
                for p in &a[1..] {
                    sim.state
                        .state
                        .cave_perks
                        .insert(p.to_string(), Value::Bool(true));
                }
            }
            "hud_hidden" => input.hud_hidden = true,
            "survival" => {
                input.hud_hidden = false;
                sim.state.state.mode = GameMode::Survival;
            }
            k => panic!("action {k}"),
        }
        let t = &sim.state.state.tasks;
        let rt = &sim.progress.tasks;
        for (k, v) in snap["stats"].as_object().unwrap() {
            assert_close(t.stats.get(k).copied().unwrap_or(0.0), f(v), TOL);
        }
        assert_eq!(
            t.stats.len(),
            snap["stats"].as_object().unwrap().len(),
            "{label}"
        );
        let done: Vec<&String> = t
            .done
            .iter()
            .filter(|(_, v)| truthy(Some(v)))
            .map(|(k, _)| k)
            .collect();
        let want_done: Vec<&String> = entries(&snap["done"]).into_iter().map(|(k, _)| k).collect();
        assert_eq!(done, want_done, "{label}");
        assert_eq!(rt.fill.len(), entries(&snap["fill"]).len(), "{label}");
        for (k, v) in entries(&snap["fill"]) {
            assert_close(rt.fill[k], f(v), TOL);
        }
        assert_eq!(rt.fold_k.len(), entries(&snap["fold_k"]).len(), "{label}");
        for (k, v) in entries(&snap["fold_k"]) {
            assert_close(rt.fold_k[k], f(v), TOL);
        }
        for (k, v) in snap["goals"].as_object().unwrap() {
            assert_eq!(truthy(t.goals.get(k)), b(v), "{label} goal {k}");
        }
        assert_eq!(t.stage, i(&snap["stage"]), "{label}");
        assert_eq!(t.popup, s(&snap["popup"]), "{label}");
        assert_eq!(t.shown, b(&snap["shown"]), "{label}");
        assert_close(rt.alpha, f(&snap["alpha"]), TOL);
        assert_eq!(
            sim.state.state.upgrade_points,
            i(&snap["points"]),
            "{label}"
        );
        for (k, v) in snap["met"].as_object().unwrap() {
            let id = TaskId::from_key(k).unwrap();
            assert_eq!(tasks::tasks_met(&sim, id), b(v), "{label} met {k}");
        }
        assert_eq!(
            goals::goal_met(&sim, Goal::Migration),
            b(&snap["goal_migration"]),
            "{label}"
        );
        assert_eq!(
            goals::goal_met(&sim, Goal::Magic),
            b(&snap["goal_magic"]),
            "{label}"
        );
        assert_eq!(
            goals::goal_spore_count(&sim),
            f(&snap["spore_count"]),
            "{label}"
        );
        assert_eq!(
            tasks::tasks_visible(&sim, &input),
            b(&snap["visible"]),
            "{label}"
        );
    }
}

#[test]
fn goal_texts_match_love() {
    let g = golden("progress");
    for r in g["numbers"].as_array().unwrap() {
        assert_eq!(goals::goal_number_text(f(&r[0])), s(&r[1]), "{}", r[0]);
    }

    /// The script's fake font: 10 px per character, 20 px lines, wrapping at `w`.
    struct Mono;
    impl Mono {
        fn w(s: &str) -> f64 {
            s.chars().count() as f64 * 10.0
        }
    }
    impl TextMeasure for Mono {
        fn string_width(&self, s: &str) -> f64 {
            Mono::w(s)
        }
        fn string_height(&self, _s: &str) -> f64 {
            20.0
        }
        fn string_width_ext(&self, s: &str, _sep: f64, w: f64) -> f64 {
            Mono::w(s).min(w)
        }
        fn string_height_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
            let lines = if w <= 0.0 {
                1.0
            } else {
                (Mono::w(s) / w).ceil().max(1.0)
            };
            lines * if sep < 0.0 { 20.0 } else { sep }
        }
    }
    for r in g["fits"].as_array().unwrap() {
        let a = &r["args"];
        let got = goals::goal_fit(&Mono, s(&a[0]), f(&a[1]), f(&a[2]), f(&a[3]), f(&a[4]));
        assert_close(got, f(&r["scale"]), TOL);
    }
    assert_close(
        goals::goal_text_scale(&Mono, 36.0),
        f(&g["text_scale"][0]),
        TOL,
    );
    assert_close(
        goals::goal_text_scale(&Mono, 22.0),
        f(&g["text_scale"][1]),
        TOL,
    );
}

#[test]
fn tutorial_wrap_matches_love() {
    let g = golden("progress");
    for r in g["wraps"].as_array().unwrap() {
        assert_eq!(tutorial::tutorial_wrap(s(&r[0])), s(&r[1]));
    }
}

fn load_chunks(sim: &mut Sim, g: &Value) {
    sim.state.world.chunk_w = i(&g["chunk_w"]);
    for c in g["chunks"].as_array().unwrap() {
        let cx = i(&c["cx"]);
        let foods = obj_array(&c["foods"])
            .into_iter()
            .map(|v| FoodSave {
                x: f(&v["x"]),
                y: f(&v["y"]),
                collected: b(&v["collected"]),
                ..Default::default()
            })
            .collect();
        let lakes = obj_array(&c["lakes"])
            .into_iter()
            .map(|v| LakeSave {
                body_x: f(&v["body_x"]),
                body_y: f(&v["body_y"]),
                body_w: f(&v["body_w"]),
                body_h: f(&v["body_h"]),
                connected: b(&v["connected"]),
                ..Default::default()
            })
            .collect();
        let ch = WorldChunkSave {
            cx,
            foods,
            lakes,
            ..Default::default()
        };
        sim.state.world.chunks.insert(format!("c{cx}"), ch);
    }
}

#[test]
fn tutorial_stages_match_love() {
    let g = golden("progress");
    let mut sim = fresh();
    load_chunks(&mut sim, &g);
    sim.state.state.length_px = 0.0;
    sim.state.state.start_x = 0.0;
    let surface = f(&g["surface_height_px"]);
    let (node_x, node_y) = (260.0, surface + 300.0);
    let mut input = StepInput::default();
    let mut raining = false;
    let mut tips_shown = 0;
    for snap in g["tutorial"].as_array().unwrap() {
        let label = s(&snap["label"]);
        // "<n>:<action> x<frames>"
        let body = label.split_once(':').unwrap().1;
        let (k, frames) = body.rsplit_once(" x").unwrap();
        let frames: usize = frames.parse().unwrap();
        match k {
            "nil" => {}
            "aim" | "food" | "water" | "mushroom" => tutorial::event(&mut sim, k, None, None),
            "shot" => tutorial::event(&mut sim, k, Some(100.0), Some(900.0)),
            "node" => tutorial::event(&mut sim, k, Some(node_x), Some(node_y)),
            "clay" => tutorial::event(&mut sim, k, Some(300.0), Some(1200.0)),
            "len" => {
                sim.state.state.length_px = if sim.state.state.length_px == 0.0 {
                    3200.0
                } else {
                    3201.0
                }
            }
            "rain_on" => raining = true,
            "rain_off" => raining = false,
            "menu_on" => input.menu_open = true,
            "menu_off" => input.menu_open = false,
            "upg" => {
                sim.state
                    .state
                    .upgrade_levels
                    .insert("unlock_spore".into(), 1);
            }
            "pause_on" => sim.pause_open = true,
            "pause_off" => sim.pause_open = false,
            "title_on" => {
                tutorial::event(&mut sim, "aim", None, None);
                sim.title_open = true;
            }
            "title_off" => sim.title_open = false,
            k => panic!("action {k}"),
        }
        for _ in 0..frames {
            tutorial::step(&mut sim, &input, raining);
        }
        tips_shown += sim
            .drain_events()
            .iter()
            .filter(|e| matches!(e, SimEvent::TutorialTip { .. }))
            .count();
        let tut = &sim.state.state.tutorial;
        for tip in Tip::ORDER {
            let want = &snap["tips"][tip.as_str()];
            let have = &tut.tips[tip.as_str()];
            let what = format!("{label} {}", tip.as_str());
            assert_eq!(have.state, i(&want["state"]), "{what}");
            assert_eq!(
                tutorial::tutorial_state(&sim, tip),
                TipState::from_code(have.state).unwrap()
            );
            assert_close(have.alpha, f(&want["alpha"]), TOL);
            assert_close(have.timer, f(&want["timer"]), TOL);
            assert_eq!(have.anchor, s(&want["anchor"]), "{what}");
            assert_close(have.x, f(&want["x"]), TOL);
            assert_close(have.y, f(&want["y"]), TOL);
        }
        assert_eq!(tut.ate, b(&snap["ate"]), "{label}");
        assert_eq!(tut.drank, b(&snap["drank"]), "{label}");
        assert_eq!(tut.hand_done, b(&snap["hand_done"]), "{label}");
        let rt = &sim.progress.tutorial;
        assert_close(rt.hand_alpha, f(&snap["hand_alpha"]), TOL);
        assert_close(rt.hand_frame, f(&snap["hand_frame"]), TOL);
        assert_close(rt.camera_alpha, f(&snap["camera_alpha"]), TOL);
        assert_close(rt.camera_frame, f(&snap["camera_frame"]), TOL);
        let pending: Vec<&String> = tut.events.keys().collect();
        let want: Vec<&String> = entries(&snap["pending"])
            .into_iter()
            .map(|(k, _)| k)
            .collect();
        assert_eq!(pending, want, "{label}");
    }
    // every tip that appeared was announced once
    assert_eq!(tips_shown, Tip::ORDER.len());
}

#[test]
fn cheat_flash_counts_down() {
    let g = golden("progress");
    let mut sim = fresh();
    progress::cheat_flash(&mut sim, "CHEAT");
    assert_eq!(sim.progress.cheat_text, s(&g["cheat"][0]));
    assert_eq!(sim.progress.cheat_timer, f(&g["cheat"][1]));
    for _ in 0..10 {
        progress::cheats_step(&mut sim);
    }
    assert_eq!(sim.progress.cheat_timer, f(&g["cheat"][2]));
}

#[test]
fn progress_survives_save_and_load() {
    let mut sim = fresh();
    sim.state.state.upgrade_points = 7;
    sim.state
        .state
        .upgrade_levels
        .insert("unlock_spore".into(), 1);
    sim.state.state.tasks.shown = true;
    sim.state.state.tasks.open = false;
    sim.state
        .state
        .tasks
        .goal_open
        .insert("magic".into(), Value::Bool(false));
    sim.state
        .state
        .tasks
        .done
        .insert("water".into(), Value::Bool(true));
    sim.state
        .state
        .tutorial
        .tips
        .get_mut("shoot")
        .unwrap()
        .state = 2;
    sim.state.state.tutorial.ate = true;
    let (save, _) = sim.to_save();
    let text = serde_json::to_string(&save).unwrap();
    let back: GameState = serde_json::from_str(&text).unwrap();
    let mut sim2 = Sim::from_save(back).unwrap();
    assert_eq!(sim2.state.state.upgrade_points, 7);
    assert_eq!(upgrades::upg_grow(&sim2, "unlock_spore"), 1.0);
    assert_eq!(upgrades::upg_grow(&sim2, "origin"), 1.0);
    assert_eq!(upgrades::upg_grow(&sim2, "spore_count"), 0.0);
    let rt = &sim2.progress.tasks;
    assert_eq!(rt.alpha, 1.0);
    assert_eq!(rt.fold_k["tasks"], 0.0);
    assert_eq!(rt.fold_k["migration"], 1.0);
    assert_eq!(rt.fold_k["magic"], 0.0);
    assert!(rt.fill.is_empty());
    assert_eq!(tutorial::tutorial_state(&sim2, Tip::Shoot), TipState::Done);
    assert!(sim2.state.state.tutorial.ate);
    // tasks keep counting after a load
    tasks::stat_add(&mut sim2, "water", 5.0);
    assert_eq!(sim2.state.state.tasks.stats["water"], 5.0);
}
