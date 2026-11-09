//! Camera and cut-scenes (Task 21) against the LÖVE port. Golden topic `camera`
//! (rust/golden/scripts/camera.lua): `boar_shake` over 130 frames (repeller kick, boars near the
//! view, the pause), and the migration cut-scene from `cine_show_start` through every
//! `cine_step` (stops, camera, shake, white, spores, the planet), the hold, `tasks_popup_close`
//! and the fade out. Plus Rust-only tests for the camera jumps, `forest_fit_view` and the
//! fly-to-marker.
mod common;

use common::{f, golden, i};
use myco_core::camera::{self, CameraFly};
use myco_core::cine::{self, CinePhase};
use myco_core::life::Boar;
use myco_core::progress::tasks;
use myco_core::rng::Rng;
use myco_core::sim::Sim;
use myco_core::state::MushroomSave;

/// The title-screen sim with every blocking flag down (as the golden script sets them).
fn open_sim() -> Sim {
    let mut sim = Sim::title_screen(Some(31607));
    sim.title_open = false;
    sim.intro_active = false;
    sim.pause_open = false;
    sim.state.state.tasks.popup.clear();
    sim.progress.tasks.cine = None;
    sim
}

fn close(a: f64, b: f64, what: &str) {
    let scale = 1f64.max(a.abs()).max(b.abs());
    assert!((a - b).abs() <= 1e-9 * scale, "{what}: {a} vs {b}");
}

#[test]
fn boar_shake_matches_love() {
    let g = golden("camera");
    let mut sim = open_sim();
    sim.rng = Rng::new(3);
    sim.mushrooms.repel_kick = 4.0;
    sim.life.boars.clear();
    let boar = |x: f64| Boar {
        x,
        y: 480.0,
        dir: 1.0,
        ..Default::default()
    };
    for row in g["boar_shake"]["rows"].as_array().unwrap() {
        let fr = i(&row[0]);
        if fr == 41 {
            sim.life.boars = vec![boar(3800.0 + 1920.0 + 300.0)];
        }
        if fr == 81 {
            sim.life.boars = vec![boar(4500.0), boar(-9000.0)];
        }
        if fr == 121 {
            sim.mushrooms.repel_kick = 3.0;
            sim.pause_open = true;
        }
        sim.frame = fr as u64;
        let blocking = sim.pause_open;
        let s = camera::boar_shake(&mut sim, 3800.0, 1920.0, blocking);
        close(s[0], f(&row[1]), &format!("frame {fr} dx"));
        close(s[1], f(&row[2]), &format!("frame {fr} dy"));
        close(
            sim.mushrooms.repel_kick,
            f(&row[3]),
            &format!("frame {fr} kick"),
        );
    }
    close(
        sim.rng.random(1.0),
        f(&g["boar_shake"]["rng_after"]),
        "rng after",
    );
}

fn shroom(kind: i64, x: f64, stolen: bool, carried_by: i64) -> MushroomSave {
    MushroomSave {
        kind,
        x,
        y: 480.0,
        stolen,
        carried_by,
        ..Default::default()
    }
}

fn cine_sim() -> Sim {
    let mut sim = open_sim();
    sim.life.boars.clear();
    sim.mushrooms.repel_kick = 0.0;
    sim.state.state.wind[0] = 1.0;
    sim.state.state.wind[1] = 2.0;
    sim.mycelium.x = 5000.0;
    sim.mycelium.y = 480.0;
    sim.mycelium.spores.clear();
    sim.camera.x = 3800.0;
    sim.camera.y = 0.0;
    sim.camera.width = 1920.0;
    sim.camera.height = 1080.0;
    sim.camera.vx = 3.0;
    sim.camera.vy = -1.0;
    sim.state.mushrooms = vec![
        shroom(0, 4000.0, false, -1),
        shroom(0, 5600.0, false, -1),
        shroom(1, 4400.0, false, -1),
        shroom(0, 7000.0, false, -1),
        shroom(0, 6100.0, true, -1),
        shroom(0, 2500.0, false, -1),
        shroom(0, 4800.0, false, 3),
    ];
    sim
}

#[test]
fn migration_cutscene_matches_love() {
    let g = golden("camera");
    let gc = &g["cine"];
    let mut sim = cine_sim();
    let stat0 = sim
        .state
        .state
        .tasks
        .stats
        .get("ach_migrated")
        .copied()
        .unwrap_or(0.0);
    sim.rng = Rng::new(21);
    sim.frame = 0;
    cine::cine_show_start(&mut sim);
    {
        let c = sim.progress.tasks.cine.as_ref().unwrap();
        let want = gc["stops"].as_array().unwrap();
        assert_eq!(c.stops.len(), want.len());
        for (s, w) in c.stops.iter().zip(want) {
            assert_eq!(
                [s.x, s.y, s.seconds, s.count],
                [f(&w[0]), f(&w[1]), f(&w[2]), f(&w[3])]
            );
        }
    }
    assert_eq!(cine::cine_blocking(&sim), gc["blocking"].as_bool().unwrap());
    assert_eq!(cine::cine_showing(&sim), gc["showing"].as_bool().unwrap());

    for row in gc["frames"].as_array().unwrap() {
        let fr = i(&row[0]);
        sim.frame = fr as u64;
        cine::cine_step(&mut sim);
        let c = sim.progress.tasks.cine.as_ref().unwrap();
        let what = format!("frame {fr}");
        assert_eq!(
            (c.phase == CinePhase::Show) as i64,
            i(&row[1]),
            "{what} phase"
        );
        assert_eq!(c.stop, i(&row[2]), "{what} stop");
        close(c.t, f(&row[3]), &format!("{what} t"));
        close(sim.camera.x, f(&row[4]), &format!("{what} cam x"));
        close(sim.camera.y, f(&row[5]), &format!("{what} cam y"));
        close(sim.camera.vx, f(&row[6]), &format!("{what} cam vx"));
        close(sim.camera.vy, f(&row[7]), &format!("{what} cam vy"));
        close(
            c.shake_k.unwrap_or(-1.0),
            f(&row[8]),
            &format!("{what} shake_k"),
        );
        close(
            c.white.unwrap_or(-1.0),
            f(&row[9]),
            &format!("{what} white"),
        );
        assert_eq!(
            sim.mycelium.spores.len() as i64,
            i(&row[10]),
            "{what} spores"
        );
        let sh = cine::cine_shake(&sim).unwrap_or([-999.0, -999.0]);
        close(sh[0], f(&row[11]), &format!("{what} shake dx"));
        close(sh[1], f(&row[12]), &format!("{what} shake dy"));
    }
    let (sx, sy) = sim
        .mycelium
        .spores
        .iter()
        .fold((0.0, 0.0), |(a, b), s| (a + s.x + s.vx, b + s.y + s.vy));
    let want = &gc["spores_after_show"];
    assert_eq!(sim.mycelium.spores.len() as i64, i(&want[0]));
    close(sx, f(&want[1]), "spore x sum");
    close(sy, f(&want[2]), "spore y sum");

    // the planet
    let gp = &gc["planet"];
    {
        let c = sim.progress.tasks.cine.as_ref().unwrap();
        assert_eq!(c.phase, CinePhase::Play);
        assert_eq!(gp["phase"].as_str().unwrap(), "play");
        close(c.t, f(&gp["t"]), "planet t");
        close(c.white.unwrap(), f(&gp["white"]), "planet white");
        close(c.shake_k.unwrap(), f(&gp["shake_k"]), "planet shake_k");
        let stars = gp["stars"].as_array().unwrap();
        assert_eq!(c.stars.len(), stars.len());
        for (s, w) in c.stars.iter().zip(stars) {
            assert_eq!(
                [s.x, s.y, s.s, s.a, s.ph, s.sp],
                [f(&w[0]), f(&w[1]), f(&w[2]), f(&w[3]), f(&w[4]), f(&w[5])]
            );
        }
        assert_eq!(c.segs.len() as i64, i(&gp["seg_count"]));
        let tsum: f64 = c.segs.iter().map(|s| s.time).sum();
        let wsum: f64 = c.segs.iter().map(|s| s.w).sum();
        close(tsum, f(&gp["seg_tsum"]), "seg time sum");
        close(wsum, f(&gp["seg_wsum"]), "seg w sum");
        for w in gp["segs"].as_array().unwrap() {
            let s = &c.segs[i(&w[0]) as usize];
            for (k, v) in [s.x1, s.y1, s.x2, s.y2, s.time, s.w].iter().enumerate() {
                close(*v, f(&w[k + 1]), &format!("seg {} field {k}", i(&w[0])));
            }
        }
    }
    close(
        sim.rng.random(1.0),
        f(&gp["rng_probe_state"]),
        "rng after planet",
    );

    // play until the hold
    let mut play = 0;
    while sim.progress.tasks.cine.as_ref().unwrap().phase == CinePhase::Play && play < 2000 {
        play += 1;
        sim.frame += 1;
        cine::cine_step(&mut sim);
    }
    assert_eq!(play, i(&gc["play_frames"]));
    let gh = &gc["hold"];
    {
        let c = sim.progress.tasks.cine.as_ref().unwrap();
        assert_eq!(c.phase, CinePhase::Hold);
        close(c.t, f(&gh["t"]), "hold t");
    }
    assert_eq!(sim.state.state.tasks.popup, gh["popup"].as_str().unwrap());
    let stat = sim.state.state.tasks.stats["ach_migrated"] - stat0;
    assert_eq!(stat, f(&gh["stat"]));
    assert_eq!(cine::cine_blocking(&sim), gh["blocking"].as_bool().unwrap());
    assert_eq!(cine::cine_showing(&sim), gh["showing"].as_bool().unwrap());
    for _ in 0..5 {
        sim.frame += 1;
        cine::cine_step(&mut sim);
    }
    close(
        sim.progress.tasks.cine.as_ref().unwrap().t,
        f(&gh["t_after5"]),
        "hold t after 5",
    );
    tasks::tasks_popup_close(&mut sim);
    let mut fade = Vec::new();
    while sim.progress.tasks.cine.is_some() && fade.len() < 500 {
        sim.frame += 1;
        let out = sim.progress.tasks.cine.as_ref().unwrap().phase == CinePhase::Out;
        cine::cine_step(&mut sim);
        let a = sim.progress.tasks.cine.as_ref().map_or(-1.0, |c| c.alpha);
        fade.push((out, a, cine::cine_blocking(&sim)));
    }
    let want = gc["fade"].as_array().unwrap();
    assert_eq!(fade.len(), want.len());
    for (k, ((out, a, b), w)) in fade.iter().zip(want).enumerate() {
        assert_eq!(*out as i64, i(&w[0]), "fade {k} phase");
        close(*a, f(&w[1]), &format!("fade {k} alpha"));
        assert_eq!(*b as i64, i(&w[2]), "fade {k} blocking");
    }
    close(sim.rng.random(1.0), f(&gc["rng_after"]), "rng after");
}

#[test]
fn cine_span_clamps() {
    assert_eq!(cine::cine_span(-1.0, 0.0, 2.0), 0.0);
    assert_eq!(cine::cine_span(1.0, 0.0, 2.0), 0.5);
    assert_eq!(cine::cine_span(5.0, 0.0, 2.0), 1.0);
    // b - a below 0.001 counts as 0.001
    assert_eq!(cine::cine_span(1.0, 1.0, 1.0), 0.0);
}

#[test]
fn camera_jump_origin_centres_on_the_colony() {
    let mut sim = open_sim();
    sim.state.state.cam_box = [-4000.0, 0.0, 4000.0, 3000.0];
    sim.mycelium.x = 1000.0;
    sim.camera.y = 900.0;
    sim.camera.vx = 5.0;
    sim.camera.vy = 5.0;
    camera::camera_jump_origin(&mut sim);
    assert_eq!(sim.camera.x, 1000.0 - sim.camera.width * 0.5);
    assert_eq!(sim.camera.y, 0.0);
    assert_eq!((sim.camera.vx, sim.camera.vy), (0.0, 0.0));
    // clamped by camera_bounds
    sim.mycelium.x = 3900.0;
    camera::camera_jump_origin(&mut sim);
    assert_eq!(sim.camera.x, 4000.0 - sim.camera.width);
}

#[test]
fn camera_jump_edge_goes_to_the_extreme_tip_or_the_origin() {
    let mut sim = open_sim();
    sim.state.state.cam_box = [-8000.0, 0.0, 8000.0, 4000.0];
    sim.mycelium.x = 200.0;
    sim.mycelium.growths.clear();
    // no growths: the origin
    camera::camera_jump_edge(&mut sim, 1.0);
    assert_eq!(sim.camera.x, 200.0 - sim.camera.width * 0.5);
    assert_eq!(sim.camera.y, 0.0);
    // camera_jump_to clamps the centred view to the bounds
    camera::camera_jump_to(&mut sim, 3000.0, 2000.0);
    assert_eq!(sim.camera.x, 3000.0 - sim.camera.width * 0.5);
    assert_eq!(sim.camera.y, 2000.0 - sim.camera.height * 0.5);
    camera::camera_jump_to(&mut sim, 3000.0, 99999.0);
    let b = camera::camera_bounds(&sim, sim.camera.width, sim.camera.height);
    assert_eq!(sim.camera.y, b.y2);
}

#[test]
fn forest_fit_view_recentres_on_the_new_size() {
    let mut sim = open_sim();
    sim.camera.x = 100.0;
    sim.camera.y = 50.0;
    sim.camera.width = 1920.0;
    sim.camera.height = 1080.0;
    camera::forest_fit_view(&mut sim, 2400.0, 1200.0);
    assert_eq!((sim.camera.x, sim.camera.y), (100.0 - 240.0, 50.0 - 60.0));
    assert_eq!((sim.camera.width, sim.camera.height), (2400.0, 1200.0));
}

#[test]
fn camera_fly_to_marker_starts_a_fly_and_follows_the_marker() {
    let mut sim = open_sim();
    camera::camera_fly_to_marker(&mut sim, 500.0, 600.0);
    assert_eq!(
        sim.camera.fly,
        Some(CameraFly {
            t: 0.0,
            x: 500.0,
            y: 600.0
        })
    );
    sim.camera.fly.as_mut().unwrap().t = 0.5;
    camera::camera_fly_track(&mut sim, 700.0, 650.0);
    assert_eq!(
        sim.camera.fly,
        Some(CameraFly {
            t: 0.5,
            x: 700.0,
            y: 650.0
        })
    );
    sim.camera.fly = None;
    camera::camera_fly_track(&mut sim, 1.0, 2.0);
    assert_eq!(sim.camera.fly, None);
}

/// obj_hud Step_1 (ui_4.lua:1052+): the camera buttons and tree purchases come after the
/// cut-scene / popup / pause / debug / title / logo returns; the marker fly comes before them.
#[test]
fn hud_camera_commands_wait_for_the_gates() {
    use myco_core::progress;
    use myco_core::sim::{Command, StepInput};
    let mut sim = open_sim();
    sim.state.state.cam_box = [-8000.0, 0.0, 8000.0, 4000.0];
    sim.mycelium.growths.clear();
    sim.mycelium.x = 3000.0;
    let input = StepInput {
        commands: vec![
            Command::CameraJumpOrigin,
            Command::CameraFlyTo { x: 1.0, y: 2.0 },
        ],
        ..Default::default()
    };
    sim.camera.x = 0.0;
    sim.pause_open = true;
    progress::hud_begin_step(&mut sim, &input);
    assert_eq!(
        sim.camera.x, 0.0,
        "the pause returns before the camera buttons"
    );
    assert!(
        sim.camera.fly.is_some(),
        "the marker click runs before the gates"
    );
    sim.pause_open = false;
    cine::cine_show_start(&mut sim);
    sim.camera.fly = None;
    progress::hud_begin_step(&mut sim, &input);
    // the cut-scene moved the camera to its first stop; the origin button did not run after it
    let c = sim.progress.tasks.cine.as_ref().unwrap();
    let stop = &c.stops[c.stop as usize];
    assert_eq!(sim.camera.x, stop.x - sim.camera.width * 0.5);
    sim.progress.tasks.cine = None;
    progress::hud_begin_step(&mut sim, &input);
    assert_eq!(sim.camera.x, 3000.0 - sim.camera.width * 0.5);
}
