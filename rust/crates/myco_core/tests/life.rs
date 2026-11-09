//! Life (Task 10) against the LÖVE port. Golden topics `life_assets` (the art data myco_core
//! embeds) and `life` (rust/golden/scripts/life.lua): behaviour functions on input grids, the
//! animal timers over 3600 frames, a seeded 1800-frame run of `life_step` with every animal on
//! the colony fixture, and 1500 frames of `game_step`. Plus Rust-only tests: new game → save →
//! load → save, the placement gate (ruling R24), audio and resource events.
mod common;

use common::{f, golden, i};
use myco_core::events::{AudioCall, ResourceKind, SimEvent};
use myco_core::life::{self, boar, crow, hedgehog, leaves, squirrel, worm, TreeRef};
use myco_core::mushrooms::{self, MushroomKind};
use myco_core::rng::Rng;
use myco_core::sim::{Command, Key, Sim, StepInput, ViewRect};
use myco_core::state::{GameMode, GameState, MushroomSave};
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

fn set_view(sim: &mut Sim, g: &Value) {
    let v = &g["view"];
    sim.camera.view_x = f(&v["x"]);
    sim.camera.view_y = f(&v["y"]);
    sim.camera.width = f(&v["w"]);
    sim.camera.height = f(&v["h"]);
}

fn load(name: &str, g: &Value) -> Sim {
    let mut sim = Sim::from_save(fixture_state(name)).unwrap();
    set_view(&mut sim, g);
    sim
}

fn close(path: &str, a: f64, b: f64) {
    if a == b || (a.is_nan() && b.is_nan()) {
        return;
    }
    let scale = 1f64.max(a.abs()).max(b.abs());
    assert!((a - b).abs() <= 1e-9 * scale, "{path}: {a:?} vs {b:?}");
}

fn row(path: &str, got: &[f64], want: &Value) {
    let w = want.as_array().unwrap();
    assert_eq!(got.len(), w.len(), "{path}: length");
    for (k, (a, b)) in got.iter().zip(w).enumerate() {
        close(&format!("{path}[{k}]"), *a, f(b));
    }
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

fn tree(name: &str, x: f64, y: f64, flip: f64) -> TreeRef {
    TreeRef {
        uid: 0,
        sprite: name.to_string(),
        x,
        y,
        flip,
    }
}

#[test]
fn life_assets_match_love() {
    let g = golden("life_assets");
    let a = life::art::art();
    for (name, w) in g["sprites"].as_object().unwrap() {
        let s = a.sprite(name);
        row(&format!("sprites.{name}"), &[s.w, s.h, s.xoff, s.yoff], w);
    }
    for (k, w) in g["hedgehog_back"].as_array().unwrap().iter().enumerate() {
        row(
            &format!("hedgehog_back[{k}]"),
            &[a.hedgehog_back[k][0], a.hedgehog_back[k][1]],
            w,
        );
    }
    for (name, w) in g["trunks"].as_object().unwrap() {
        let t = a.trunk(name);
        row(&format!("{name}.left"), &t.left, &w["left"]);
        row(&format!("{name}.right"), &t.right, &w["right"]);
        close(&format!("{name}.origin_y"), t.origin_y, f(&w["origin_y"]));
        close(&format!("{name}.h"), t.h, f(&w["h"]));
    }
    for k in 0..2 {
        for (n, w) in g["leaf_art"]["faces"][k]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            row(&format!("faces[{k}][{n}]"), &a.leaf_art.faces[k][n], w);
        }
        for (n, w) in g["leaf_art"]["turns"][k]
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
        {
            row(&format!("turns[{k}][{n}]"), &a.leaf_art.turns[k][n], w);
        }
    }
}

#[test]
fn funcs_match_love() {
    let g = golden("life");
    let fz = &g["funcs"];
    let mut sim = load("mushrooms.json", &g);
    sim.rng = Rng::new(1);
    let sy = myco_core::balance::wld("surface_height_px");
    row(
        "hedgehog",
        &[
            hedgehog::hedgehog_frame_w(),
            hedgehog::hedgehog_frame_h(),
            hedgehog::hedgehog_back_y(),
            hedgehog::hedgehog_border_margin(),
        ],
        &fz["hedgehog"],
    );
    for (n, w) in fz["back_point"].as_array().unwrap().iter().enumerate() {
        let (x, y, d, fr) = (f(&w[0]), f(&w[1]), f(&w[2]), f(&w[3]));
        let p = hedgehog::hedgehog_back_point(x, y, d, fr);
        row(&format!("back_point[{n}]"), &[x, y, d, fr, p.0, p.1], w);
    }
    let span = hedgehog::hedgehog_world_span(&sim);
    row("span", &[span.0, span.1], &fz["span"]);
    row("boar", &[boar::boar_frame_w()], &fz["boar"]);
    row(
        "squirrel",
        &[
            squirrel::squirrel_frame_w(),
            squirrel::squirrel_frame_h(),
            squirrel::squirrel_run_y(),
            squirrel::squirrel_tail_drop(),
        ],
        &fz["squirrel"],
    );
    for (n, w) in fz["offset"].as_array().unwrap().iter().enumerate() {
        let (face, ang, px, py) = (f(&w[0]), f(&w[1]), f(&w[2]), f(&w[3]));
        let o = squirrel::squirrel_offset(face, ang, px, py);
        let s = life::Squirrel {
            mirror: face,
            angle: ang,
            x: 120.25,
            y: 300.0,
            ..Default::default()
        };
        let p = squirrel::squirrel_point(&s, px, py);
        row(
            &format!("offset[{n}]"),
            &[face, ang, px, py, o.0, o.1, p.0, p.1],
            w,
        );
    }
    for (n, w) in fz["bite"].as_array().unwrap().iter().enumerate() {
        let t = i(&w[0]);
        let bt = squirrel::squirrel_shroom_bite(t);
        let m = MushroomSave {
            kind: t,
            y: 470.5,
            ..Default::default()
        };
        row(
            &format!("bite[{n}]"),
            &[
                t as f64,
                bt.0,
                bt.1,
                squirrel::squirrel_bite_y(&m),
                crow::crow_cap_dy(t),
            ],
            w,
        );
    }
    for (n, w) in fz["edges"].as_array().unwrap().iter().enumerate() {
        let tn = w[0].as_str().unwrap();
        let (flip, side, py) = (f(&w[1]), f(&w[2]), f(&w[3]));
        let t = tree(tn, 333.5, sy, flip);
        let got = [
            squirrel::squirrel_edge_x(&t, side, py),
            squirrel::squirrel_climb_x(-side, 90.0 * side, &t, side, py),
            squirrel::squirrel_climb_x(side, 0.0, &t, side, py),
        ];
        row(
            &format!("edges[{n}]"),
            &got,
            &Value::from(w.as_array().unwrap()[4..].to_vec()),
        );
    }
    for (n, w) in fz["sky"].as_array().unwrap().iter().enumerate() {
        let t = tree(w[0].as_str().unwrap(), f(&w[1]), sy, 1.0);
        let v = ViewRect {
            x: -960.0,
            y: f(&w[2]),
            w: 1920.0,
            h: 1080.0,
        };
        close(
            &format!("sky[{n}]"),
            squirrel::squirrel_sky_y(&t, &v),
            f(&w[3]),
        );
    }
    let mut s = life::Squirrel {
        x: 10.0,
        y: 20.0,
        ..Default::default()
    };
    squirrel::squirrel_turn_start(&mut s, life::SquirrelState::Land, 180.0, 50.0, -40.0);
    for (n, w) in fz["turn"].as_array().unwrap().iter().enumerate() {
        let done = squirrel::squirrel_turn_step(&mut s);
        row(
            &format!("turn[{n}]"),
            &[s.turn_t, s.angle, s.x, s.y, b(done), s.state.code() as f64],
            w,
        );
    }
    for (n, w) in fz["hold"].as_array().unwrap().iter().enumerate() {
        let (mir, ang, t) = (f(&w[0]), f(&w[1]), i(&w[2]));
        let s = life::Squirrel {
            mirror: mir,
            angle: ang,
            x: 50.0,
            y: 250.0,
            ..Default::default()
        };
        let p = squirrel::squirrel_hold_pose(&s, t);
        row(
            &format!("hold[{n}]"),
            &[mir, ang, t as f64, p.x, p.y, p.xscale, p.yscale, p.angle],
            w,
        );
    }
    row(
        "crow",
        &[crow::crow_frame_w(), crow::crow_below_claws()],
        &fz["crow"],
    );
    for (n, w) in fz["claw"].as_array().unwrap().iter().enumerate() {
        let (fr, dir) = (f(&w[0]), f(&w[1]));
        let c = crow::crow_claw(fr);
        let o = crow::crow_claw_offset(fr, dir);
        row(&format!("claw[{n}]"), &[fr, dir, c.0, c.1, o.0, o.1], w);
    }
    for (n, w) in fz["crow_sky"].as_array().unwrap().iter().enumerate() {
        let v = ViewRect {
            x: 0.0,
            y: f(&w[0]),
            w: 1920.0,
            h: 1080.0,
        };
        close(
            &format!("crow_sky[{n}]"),
            crow::crow_sky_y(&v, f(&w[1])),
            f(&w[2]),
        );
    }
    row(
        "worm",
        &[
            worm::worm_zone_top(&sim),
            worm::worm_zone_bottom(&sim),
            worm::worm_segment_px(),
        ],
        &fz["worm"],
    );
    for (n, w) in fz["keep"].as_array().unwrap().iter().enumerate() {
        close(
            &format!("keep[{n}]"),
            worm::worm_keep_bottom(&sim, f(&w[0]), f(&w[1])),
            f(&w[2]),
        );
    }
    for (n, w) in fz["cell"].as_array().unwrap().iter().enumerate() {
        let (x, y) = (f(&w[0]), f(&w[1]));
        let a = worm::worm_cell_ok(&mut sim, x, y, None);
        let c = worm::worm_cell_ok(&mut sim, x, y, Some(sy + 4.0));
        row(&format!("cell[{n}]"), &[x, y, b(a), b(c)], w);
    }
    let body = [(0.0, 0.0), (3.0, 4.0), (6.0, 4.0), (6.0, 10.0)];
    let wm = life::Worm {
        body: body.iter().map(|&(x, y)| life::Pt { x, y }).collect(),
        ..Default::default()
    };
    for (n, w) in fz["body_at"].as_array().unwrap().iter().enumerate() {
        let s = f(&w[0]);
        let p = worm::worm_body_at(&wm, s);
        row(&format!("body_at[{n}]"), &[s, p.x, p.y, p.dir], w);
    }
    for (n, w) in fz["leaf"].as_array().unwrap().iter().enumerate() {
        close(
            &format!("leaf[{n}]"),
            leaves::leaf_face_for_wind(f(&w[0])),
            f(&w[1]),
        );
    }
    for (n, w) in fz["levels"].as_array().unwrap().iter().enumerate() {
        let lvl = i(&w[0]);
        set_upg(&mut sim, "absorption_speed", lvl);
        set_upg(&mut sim, "absorption_yield", lvl);
        row(
            &format!("levels[{n}]"),
            &[
                lvl as f64,
                leaves::leaf_decay_time(&sim),
                leaves::leaf_food_value(&sim),
                worm::worm_absorb_frames(&sim),
            ],
            w,
        );
    }
    set_upg(&mut sim, "absorption_speed", 0);
    set_upg(&mut sim, "absorption_yield", 0);
    let poses = [
        leaves::LeafArtPose::default(),
        leaves::LeafArtPose {
            face: Some(1.0),
            ..Default::default()
        },
        leaves::LeafArtPose {
            face: Some(0.0),
            change_from: Some(0.0),
            change_frame: 3.0,
        },
        leaves::LeafArtPose {
            face: Some(1.0),
            change_from: Some(1.0),
            change_frame: 7.5,
        },
        leaves::LeafArtPose {
            change_from: Some(-1.0),
            ..Default::default()
        },
    ];
    for (n, w) in fz["leaf_art"].as_array().unwrap().iter().enumerate() {
        let a = leaves::leaf_art_frame(&poses[n]);
        row(&format!("leaf_art[{n}]"), &a, w);
    }
    for (mi, wrow) in fz["checks"].as_array().unwrap().iter().enumerate() {
        let mut k = 0;
        for (stolen, crow_t, carried) in [
            (false, false, -1),
            (true, false, -1),
            (false, true, -1),
            (false, false, 0),
        ] {
            {
                let m = &mut sim.state.mushrooms[mi];
                m.stolen = stolen;
                m.crow_target = crow_t;
                m.carried_by = carried;
            }
            for red in [false, true] {
                set_perk(&mut sim, "red", red);
                let m = &sim.state.mushrooms[mi];
                let got = [
                    b(hedgehog::hedgehog_takes(m)),
                    b(boar::boar_hits_mushroom(m)),
                    b(crow::crow_can_take(&sim, mi)),
                    b(squirrel::squirrel_takes(&sim, mi)),
                ];
                row(&format!("checks[{mi}][{k}]"), &got, &wrow[k]);
                k += 1;
            }
        }
        let m = &mut sim.state.mushrooms[mi];
        m.stolen = false;
        m.crow_target = false;
        m.carried_by = -1;
        set_perk(&mut sim, "red", false);
    }
    let saved = std::mem::take(&mut sim.state.mushrooms);
    for (n, w) in fz["counts"].as_array().unwrap().iter().enumerate() {
        let cnt = i(&w[0]);
        sim.state.mushrooms = (1..=cnt)
            .map(|k| MushroomSave {
                kind: 0,
                carried_by: if k % 7 == 0 { 0 } else { -1 },
                stolen: k % 11 == 0,
                ..Default::default()
            })
            .collect();
        row(
            &format!("counts[{n}]"),
            &[
                cnt as f64,
                life::animal_mushroom_count(&sim),
                life::animal_timer_step(&sim),
            ],
            w,
        );
    }
    sim.state.mushrooms = saved;
    let z = life::life_near_zone(&sim);
    row("zone", &[z.x1, z.x2, z.w], &fz["zone"]);
    for (n, w) in fz["outside"].as_array().unwrap().iter().enumerate() {
        let (a, bb) = (f(&w[0]), f(&w[1]));
        close(
            &format!("outside[{n}]"),
            life::life_far_outside(a, bb, &z),
            f(&w[2]),
        );
    }
    row(
        "any",
        &[
            b(boar::boar_any_target(&sim)),
            b(squirrel::squirrel_any_target(&sim)),
        ],
        &fz["any"],
    );
    let mut k = 0;
    let want_has = fz["tree_has"].as_array().unwrap();
    for c in -2..=2 {
        let trees: Vec<TreeRef> = sim
            .forest
            .surface_get_chunk(&mut sim.rng, c)
            .trees
            .iter()
            .map(TreeRef::of)
            .collect();
        for t in trees {
            let w = &want_has[k];
            close(&format!("tree_has[{k}].x"), t.x, f(&w[0]));
            assert_eq!(t.sprite, w[1].as_str().unwrap(), "tree_has[{k}].sprite");
            close(
                &format!("tree_has[{k}].has"),
                b(squirrel::squirrel_tree_has_mushroom(&sim, &t)),
                f(&w[2]),
            );
            k += 1;
        }
    }
    assert_eq!(k, want_has.len());
    let near = fz["near"].as_array().unwrap();
    let tx = |t: Option<TreeRef>| t.map(|t| t.x);
    let mut n = 0;
    for x in [-3000.0, -1200.0, -400.0, 0.0, 350.0, 1500.0, 4200.0] {
        for range in [80.0, 600.0, 2000.0] {
            for free in [false, true] {
                let t = squirrel::squirrel_tree_near(&mut sim, x, range, free, None);
                opt(&format!("near[{n}]"), tx(t), &near[n][3]);
                n += 1;
            }
        }
        for dir in [-1.0, 0.0, 1.0] {
            let t = squirrel::squirrel_tree03_find(&mut sim, x, dir, 200.0, false, None);
            let t2 = squirrel::squirrel_tree03_find(&mut sim, x, dir, 0.0, true, None);
            opt(&format!("find[{n}]"), tx(t), &near[n][2]);
            opt(&format!("find2[{n}]"), tx(t2), &near[n][3]);
            n += 1;
        }
    }
    assert_eq!(n, near.len());
    sim.state.state.mode = GameMode::Survival;
    sim.life.boars.clear();
    sim.life.boar_pack.clear();
    boar::boar_spawn_pack(&mut sim, -3000.0, 1.0);
    let pack = fz["pack"].as_array().unwrap();
    assert_eq!(sim.life.boar_pack.len(), pack.len());
    for (n, (p, w)) in sim.life.boar_pack.iter().zip(pack).enumerate() {
        row(&format!("pack[{n}]"), &[p.x, p.dir, p.wait], w);
    }
    assert_eq!(
        sim.life.boars[0].targets.len() as i64,
        i(&fz["pack_targets"])
    );
}

fn timers(sim: &Sim) -> Vec<f64> {
    let t = &sim.state.state.timers;
    vec![
        t.hedgehog,
        t.boar.unwrap(),
        t.squirrel,
        t.crow,
        t.experience_crows[0],
        t.experience_crows[1],
        t.leaf,
        t.worm,
    ]
}

#[test]
fn timers_tick_like_love() {
    let g = golden("life");
    let tg = &g["timers"];
    let mut sim = load("exploration.json", &g);
    sim.rng = Rng::new(7);
    row("start", &timers(&sim), &tg["start"]);
    let at = tg["at"].as_array().unwrap();
    let mut n = 0;
    for k in 0..3600u64 {
        sim.frame = k;
        life::life_step(&mut sim);
        if (k + 1) % 600 == 0 {
            assert_eq!(i(&at[n][0]), k as i64 + 1);
            row(&format!("timers at {}", k + 1), &timers(&sim), &at[n][1]);
            n += 1;
        }
    }
    assert_eq!(n, at.len());
}

fn idx(sim: &Sim, id: Option<u64>) -> f64 {
    id.map_or(-1.0, |id| mushrooms::mushroom_index_of(sim, id) as f64)
}

fn snap_eq(p: &str, sim: &Sim, w: &Value) {
    let l = &sim.life;
    let arr = |k: &str| w[k].as_array().unwrap().clone();
    let wa = arr("animals");
    assert_eq!(l.animals.len(), wa.len(), "{p}.animals");
    for (n, (a, w)) in l.animals.iter().zip(&wa).enumerate() {
        row(
            &format!("{p}.animals[{n}]"),
            &[
                a.x,
                a.y,
                a.dir,
                a.speed,
                a.frame,
                a.frame_timer,
                a.carrying as f64,
            ],
            w,
        );
    }
    let wb = arr("boars");
    assert_eq!(l.boars.len(), wb.len(), "{p}.boars");
    for (n, (bo, w)) in l.boars.iter().zip(&wb).enumerate() {
        row(
            &format!("{p}.boars[{n}]"),
            &[
                bo.x,
                bo.y,
                bo.dir,
                bo.frame,
                bo.frame_timer,
                bo.hits,
                b(bo.fleeing),
            ],
            &Value::from(w.as_array().unwrap()[..7].to_vec()),
        );
        let tg: Vec<f64> = bo.targets.iter().map(|&t| idx(sim, Some(t))).collect();
        row(&format!("{p}.boars[{n}].targets"), &tg, &w[7]);
    }
    let wp = arr("pack");
    assert_eq!(l.boar_pack.len(), wp.len(), "{p}.pack");
    for (n, (pk, w)) in l.boar_pack.iter().zip(&wp).enumerate() {
        row(&format!("{p}.pack[{n}]"), &[pk.x, pk.dir, pk.wait], w);
    }
    let wf = arr("flung");
    assert_eq!(l.boar_flung.len(), wf.len(), "{p}.flung");
    for (n, (fl, w)) in l.boar_flung.iter().zip(&wf).enumerate() {
        assert_eq!(fl.sprite, w[1].as_str().unwrap(), "{p}.flung[{n}].sprite");
        let got = [
            fl.kind as f64,
            fl.variant,
            fl.x,
            fl.y,
            fl.vx,
            fl.vy,
            fl.angle,
            fl.spin,
            fl.life,
        ];
        let mut want = w.as_array().unwrap().clone();
        want.remove(1);
        row(&format!("{p}.flung[{n}]"), &got, &Value::from(want));
    }
    let wq = arr("squirrels");
    assert_eq!(l.squirrels.len(), wq.len(), "{p}.squirrels");
    for (n, (q, w)) in l.squirrels.iter().zip(&wq).enumerate() {
        let got = [
            q.tree.x,
            q.home.x,
            q.target_tree.x,
            q.far_tree.x,
            b(q.direct),
            q.side,
            q.dir,
            q.mirror,
            q.angle,
            q.x,
            q.y,
            q.stop_y,
            q.run_line,
            q.state.code() as f64,
            q.turn_t,
            q.frame,
            q.frame_timer,
            q.timer,
            idx(sim, q.mushroom),
            b(q.empty_run),
            q.grab_x,
            q.grab_y,
        ];
        row(&format!("{p}.squirrels[{n}]"), &got, w);
    }
    let wc = arr("crows");
    assert_eq!(l.crows.len(), wc.len(), "{p}.crows");
    for (n, (c, w)) in l.crows.iter().zip(&wc).enumerate() {
        let got = [
            c.state.code() as f64,
            c.dir,
            c.x,
            c.y,
            c.x0,
            c.y0,
            c.grab_x,
            c.grab_y,
            c.t,
            c.frame,
            c.frame_timer,
            c.timer,
            c.speed,
            b(c.flee),
            idx(sim, c.mushroom),
        ];
        row(&format!("{p}.crows[{n}]"), &got, w);
    }
    for (n, w) in arr("pending").iter().enumerate() {
        match (&l.crow_pending[n], w.as_str()) {
            (None, Some("undefined")) => {}
            (Some(pd), None) => row(
                &format!("{p}.pending[{n}]"),
                &[idx(sim, Some(pd.mushroom)), pd.dir, pd.x, pd.y],
                w,
            ),
            (got, _) => panic!("{p}.pending[{n}]: {got:?} vs {w}"),
        }
    }
    let wl = arr("leaves");
    assert_eq!(l.leaves.len(), wl.len(), "{p}.leaves");
    for (n, (lf, w)) in l.leaves.iter().zip(&wl).enumerate() {
        let got = [
            lf.x,
            lf.y,
            lf.phase,
            lf.mirror,
            lf.turn,
            b(lf.turned),
            lf.turn_delay,
            lf.angle,
            lf.drift,
            b(lf.landed),
            b(lf.absorbing),
            lf.timer,
            lf.life,
        ];
        row(
            &format!("{p}.leaves[{n}]"),
            &got,
            &Value::from(w.as_array().unwrap()[..13].to_vec()),
        );
        opt(&format!("{p}.leaves[{n}].total"), lf.total, &w[13]);
        close(
            &format!("{p}.leaves[{n}].tendril"),
            lf.tendril.as_ref().map_or(-1.0, |t| t.len() as f64),
            f(&w[14]),
        );
    }
    let ww = arr("worms");
    assert_eq!(l.worms.len(), ww.len(), "{p}.worms");
    for (n, (wm, w)) in l.worms.iter().zip(&ww).enumerate() {
        let got = [
            wm.dir,
            wm.turn,
            wm.side,
            wm.phase,
            wm.check,
            wm.trail_walk,
            b(wm.caught),
            wm.timer,
            wm.total,
            wm.struggle,
        ];
        row(
            &format!("{p}.worms[{n}]"),
            &got,
            &Value::from(w.as_array().unwrap()[..10].to_vec()),
        );
        opt(&format!("{p}.worms[{n}].touch"), wm.touch_frames, &w[10]);
        close(
            &format!("{p}.worms[{n}].tendril"),
            wm.tendril.as_ref().map_or(-1.0, |t| t.len() as f64),
            f(&w[11]),
        );
        let body: Vec<f64> = wm.body.iter().flat_map(|q| [q.x, q.y]).collect();
        let wbody: Vec<Value> = w[12]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|q| q.as_array().unwrap().clone())
            .collect();
        row(&format!("{p}.worms[{n}].body"), &body, &Value::from(wbody));
    }
    let wt = arr("trail");
    assert_eq!(l.worm_trail.len(), wt.len(), "{p}.trail");
    for (n, (t, w)) in l.worm_trail.iter().zip(&wt).enumerate() {
        row(&format!("{p}.trail[{n}]"), &[t.x, t.y, t.life], w);
    }
    let wto = arr("tendrils");
    assert_eq!(l.leaf_tendrils_out.len(), wto.len(), "{p}.tendrils");
    for (n, (t, w)) in l.leaf_tendrils_out.iter().zip(&wto).enumerate() {
        row(
            &format!("{p}.tendrils[{n}]"),
            &[t.cells.len() as f64, t.timer, t.total],
            w,
        );
    }
    let wm = arr("mushrooms");
    assert_eq!(sim.state.mushrooms.len(), wm.len(), "{p}.mushrooms");
    for (n, (m, w)) in sim.state.mushrooms.iter().zip(&wm).enumerate() {
        let got = [
            m.kind as f64,
            m.x,
            m.y,
            b(m.stolen),
            b(m.crow_target),
            m.carried_by as f64,
            m.hold_xscale,
            m.hold_yscale,
            m.hold_angle,
        ];
        row(
            &format!("{p}.mushrooms[{n}]"),
            &got,
            &Value::from(w.as_array().unwrap()[..9].to_vec()),
        );
        opt(
            &format!("{p}.mushrooms[{n}].carry_angle"),
            m.extra.get("carry_angle").and_then(Value::as_f64),
            &w[9],
        );
    }
    let far = &l.far;
    row(
        &format!("{p}.far"),
        &[
            far.rate.leaves,
            far.rate.worms,
            far.rate.food,
            far.food_due,
            far.timer,
            far.scan_from as f64,
        ],
        &w["far"],
    );
}

#[test]
fn seeded_run_matches_love() {
    let g = golden("life");
    let r = &g["run"];
    let mut sim = load("mushrooms.json", &g);
    sim.rng = Rng::new(99);
    set_upg(&mut sim, "unlock_absorption", 1);
    set_upg(&mut sim, "unlock_protein", 1);
    {
        let t = &mut sim.state.state.timers;
        t.hedgehog = 240.0;
        t.squirrel = 420.0;
        t.crow = 330.0;
        t.experience_crows[0] = 520.0;
        t.experience_crows[1] = 700.0;
        t.boar = Some(610.0);
        t.leaf = 90.0;
        t.worm = 30.0;
    }
    let frames = r["frames"].as_array().unwrap();
    let snaps = r["snaps"].as_array().unwrap();
    let mut s = 0;
    for k in 0..1800u64 {
        sim.frame = k;
        let mut note = String::new();
        match k {
            5 => note = hedgehog::hedgehog_summon(&mut sim).to_string(),
            15 | 900 => note = squirrel::squirrel_summon(&mut sim).to_string(),
            25 | 950 => note = crow::crow_summon(&mut sim).to_string(),
            35 => note = boar::boar_summon(&mut sim).to_string(),
            45 => leaves::leaf_gust_spawn(&mut sim),
            55..=57 => {
                worm::worm_spawn(&mut sim, true);
            }
            65 | 300 | 301 => leaves::leaf_spawn(&mut sim, true, None, None),
            _ => {}
        }
        life::life_step(&mut sim);
        let w = &frames[k as usize];
        let st = &sim.state.state;
        let l = &sim.life;
        let mut got = vec![
            st.food,
            st.water_free,
            sim.state.mushrooms.len() as f64,
            l.animals.len() as f64,
            l.boars.len() as f64,
            l.boar_flung.len() as f64,
            l.squirrels.len() as f64,
            l.crows.len() as f64,
            l.leaves.len() as f64,
            l.worms.len() as f64,
            l.worm_trail.len() as f64,
            l.leaf_tendrils_out.len() as f64,
        ];
        got.extend(timers(&sim));
        let mut want = w.as_array().unwrap().clone();
        let wnote = want.pop().unwrap();
        row(&format!("frame {k}"), &got, &Value::from(want));
        assert_eq!(note, wnote.as_str().unwrap(), "frame {k} note");
        if s < snaps.len() && i(&snaps[s]["frame"]) == k as i64 {
            snap_eq(&format!("snap {k}"), &sim, &snaps[s]);
            s += 1;
        }
    }
    assert_eq!(s, snaps.len());
    for w in r["stats"].as_array().unwrap() {
        let key = w[0].as_str().unwrap();
        let got = sim.state.state.tasks.stats.get(key).copied().unwrap_or(0.0);
        close(&format!("stats.{key}"), got, f(&w[1]));
    }
    close("probe", sim.rng.random(1.0), f(&r["probe"]));
}

#[test]
fn game_step_matches_love() {
    let g = golden("life");
    let gg = &g["game"];
    let mut sim = load("exploration.json", &g);
    sim.rng = Rng::new(5);
    let input = StepInput {
        view: ViewRect {
            x: -960.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        },
        ..Default::default()
    };
    let frames = gg["frames"].as_array().unwrap();
    let mut n = 0;
    for k in 0..1500u64 {
        sim.frame = k;
        myco_core::game::step(&mut sim, &input);
        if k % 10 == 9 {
            let st = &sim.state.state;
            row(
                &format!("game frame {k}"),
                &[
                    k as f64,
                    st.wind[0],
                    st.wind[1],
                    st.wind[2],
                    st.upgrade_points as f64,
                    st.points_awarded as f64,
                    st.play_frames as f64,
                    sim.life.leaves.len() as f64,
                ],
                &frames[n],
            );
            n += 1;
        }
    }
    close("game probe", sim.rng.random(1.0), f(&gg["probe"]));
}

/// A new game's save includes every animal timer (life_init sets them, `boar` too), so a save
/// written, loaded and written again is byte-identical.
#[test]
fn new_game_save_load_save_is_identical() {
    let sim = Sim::new_game(GameMode::Exploration, "t10", Some(31607));
    let (st1, _) = sim.to_save();
    assert!(
        st1.state.timers.boar.is_some(),
        "new game saves timers.boar"
    );
    let j1 = serde_json::to_string(&st1).unwrap();
    let sim2 = Sim::from_save(serde_json::from_str(&j1).unwrap()).unwrap();
    let (st2, _) = sim2.to_save();
    let j2 = serde_json::to_string(&st2).unwrap();
    assert_eq!(j1, j2);
    // survival likewise (the boar timer follows the mode)
    let sim = Sim::new_game(GameMode::Survival, "t10s", Some(7));
    let (st1, _) = sim.to_save();
    let j1 = serde_json::to_string(&st1).unwrap();
    let sim2 = Sim::from_save(serde_json::from_str(&j1).unwrap()).unwrap();
    assert_eq!(j1, serde_json::to_string(&sim2.to_save().0).unwrap());
}

/// A fresh game runs the intro first (game_init sets `intro_active`), then the hedgehog drops
/// the origin mushroom and the colony starts.
#[test]
fn new_game_plays_the_intro_then_starts() {
    let mut sim = Sim::new_game(GameMode::Exploration, "t10", Some(31607));
    assert!(sim.intro_active);
    let input = StepInput {
        view: ViewRect {
            x: -960.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        },
        ..Default::default()
    };
    let mut n = 0;
    while sim.intro_active && n < 20_000 {
        sim.step(&input);
        n += 1;
    }
    assert!(!sim.intro_active, "intro ended");
    assert_eq!(sim.life.animals.len(), 1, "the intro hedgehog walks on");
    assert!(!sim.mycelium.knots.is_empty(), "origin knot placed");
}

/// Ruling R24: the placement preview runs only where obj_hud Step_1 reaches it.
#[test]
fn placement_preview_gated_like_hud() {
    let g = golden("life");
    let base = || {
        let mut sim = load("mushrooms.json", &g);
        sim.rng = Rng::new(3);
        sim
    };
    let input = |f: &dyn Fn(&mut StepInput)| {
        let mut i = StepInput {
            view: ViewRect {
                x: -960.0,
                y: 0.0,
                w: 1920.0,
                h: 1080.0,
            },
            cursor: Some((100.0, 470.0)),
            placement: Some(MushroomKind::Spore),
            ..Default::default()
        };
        f(&mut i);
        i
    };
    // open: preview computed
    let mut sim = base();
    sim.step(&input(&|_| {}));
    assert!(
        sim.mushrooms.place_preview.is_some(),
        "preview while placing"
    );
    type Case = (&'static str, Box<dyn Fn(&mut Sim, &mut StepInput)>);
    let cases: Vec<Case> = vec![
        ("tasks popup", Box::new(|_, i| i.tasks_popup_open = true)),
        ("pause", Box::new(|s, _| s.pause_open = true)),
        ("debug", Box::new(|_, i| i.debug_open = true)),
        ("title", Box::new(|s, _| s.title_open = true)),
        ("logo", Box::new(|_, i| i.logo_open = true)),
        ("menu", Box::new(|_, i| i.menu_open = true)),
        ("hud hidden", Box::new(|_, i| i.hud_hidden = true)),
        ("intro", Box::new(|s, _| s.intro_active = true)),
        (
            "right click",
            Box::new(|_, i| {
                i.rmb.pressed = true;
                i.rmb.down = true
            }),
        ),
        (
            "number key",
            Box::new(|_, i| i.keys_pressed.push(Key::Char('1'))),
        ),
    ];
    for (name, set) in cases {
        let mut sim = base();
        sim.mushrooms.place_preview = None;
        let mut inp = input(&|_| {});
        set(&mut sim, &mut inp);
        // a click in the same blocked step does not place either
        let n0 = sim.state.mushrooms.len();
        inp.commands.push(Command::PlaceMushroom {
            kind: MushroomKind::Spore,
            x: 100.0,
            y: 470.0,
        });
        sim.step(&inp);
        assert!(sim.mushrooms.place_preview.is_none(), "{name}: no preview");
        assert_eq!(sim.state.mushrooms.len(), n0, "{name}: no placement");
    }
}

/// Animals and pickups talk to the app: a walking boar keeps its tremor loop, absorbed leaves
/// and caught worms run a hyphae sound and credit food with a `Resource` event (R20).
#[test]
fn life_emits_audio_and_resources() {
    let g = golden("life");
    let mut sim = load("mushrooms.json", &g);
    sim.rng = Rng::new(99);
    set_upg(&mut sim, "unlock_absorption", 1);
    set_upg(&mut sim, "unlock_protein", 1);
    boar::boar_summon(&mut sim);
    let bid = sim.life.boars[0].id;
    let mut loops = 0;
    let mut food_events = 0.0;
    let mut begins = 0;
    let mut stops = 0;
    for _ in 0..200 {
        leaves::leaf_spawn(&mut sim, true, None, None);
        worm::worm_spawn(&mut sim, true);
    }
    for k in 0..2400u64 {
        sim.frame = k;
        let food0 = sim.state.state.food;
        life::life_step(&mut sim);
        let mut credited = 0.0;
        for e in sim.drain_events() {
            match e {
                SimEvent::Audio(AudioCall::LoopFollow { id, sound, .. }) => {
                    assert_eq!(sound, "boar_tremor");
                    if id == bid {
                        loops += 1;
                    }
                }
                SimEvent::Audio(AudioCall::HyphaeBegin { at, .. }) => {
                    assert!(at.is_some(), "leaf/worm hyphae sounds are placed");
                    begins += 1;
                }
                SimEvent::Audio(AudioCall::HyphaeStop { .. }) => stops += 1,
                SimEvent::Resource {
                    kind: ResourceKind::Food,
                    delta,
                    ..
                } => credited += delta,
                _ => {}
            }
        }
        // every food change of a frame is announced (far life, which has no position, earns
        // nothing on this small colony)
        close(
            &format!("frame {k} food events"),
            credited,
            sim.state.state.food - food0,
        );
        food_events += credited;
    }
    assert!(loops > 10, "boar tremor followed every frame ({loops})");
    assert!(begins > 0 && stops > 0, "hyphae sounds ({begins}/{stops})");
    assert!(food_events > 0.0, "food credited");
}
