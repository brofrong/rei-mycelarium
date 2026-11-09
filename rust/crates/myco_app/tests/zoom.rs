//! The render zoom (Task 21, Review Focus 4): limits (2× in; out to the explored colony plus
//! 256 px, but 1× always), towards the point under the cursor, clamped between the sky top and
//! the world bottom, locked to 1× during the intro and cut-scenes.
use bevy::math::{DVec2, Vec2};
use myco_app::camera::{
    zoom_advance, zoom_limits, zoom_locked, zoom_wheel, Cameras, ZoomState, ZOOM_MAX,
};
use myco_app::display::SafeArea;
use myco_core::cine;
use myco_core::mycelium::knots::Knot;
use myco_core::sim::Sim;
use myco_core::state::GameMode;

fn knot(x: f64, y: f64) -> Knot {
    Knot {
        x,
        y,
        ..Default::default()
    }
}

/// A started game past the intro with only the origin knot.
fn small_colony() -> Sim {
    let mut sim = Sim::new_game(GameMode::Exploration, "zoom", Some(31607));
    sim.intro_active = false;
    sim.mycelium.growths.clear();
    sim.mycelium.knots = vec![knot(sim.mycelium.x, sim.mycelium.y)];
    sim.state.mushrooms.clear();
    sim
}

/// Knots far apart: an explored area much larger than one screen.
fn big_colony() -> Sim {
    let mut sim = small_colony();
    let (x, y) = (sim.mycelium.x, sim.mycelium.y);
    sim.mycelium.knots.push(knot(x - 6000.0, y + 300.0));
    sim.mycelium.knots.push(knot(x + 6000.0, y + 1800.0));
    sim.state.state.cam_box = [x - 20000.0, 0.0, x + 20000.0, 8000.0];
    sim
}

/// Retina ×2 with the picture below a 64 px notch.
fn retina() -> Cameras {
    Cameras::new(
        3024.0,
        1964.0,
        2.0,
        SafeArea {
            x: 0.0,
            y: 64.0,
            w: 3024.0,
            h: 1900.0,
        },
    )
}

fn settle(cams: &mut Cameras, sim: &mut Sim, zs: &mut ZoomState, anchor: Vec2, frames: usize) {
    for _ in 0..frames {
        zoom_advance(cams, sim, zs, 1.0 / 60.0, anchor);
    }
}

#[test]
fn zoom_limits_small_colony_allows_1x() {
    let mut sim = small_colony();
    let mut cams = Cameras::default();
    cams.sync_game_view(&sim.camera);
    let (lo, hi) = zoom_limits(&sim, &cams);
    assert_eq!((lo, hi), (1.0, ZOOM_MAX));
    assert_eq!(ZOOM_MAX, 2.0);
    // zooming out stops at 1×, zooming in at 2×
    let mut zs = ZoomState::default();
    let centre = Vec2::new(960.0, 540.0);
    zoom_wheel(&mut zs, 0.25);
    settle(&mut cams, &mut sim, &mut zs, centre, 120);
    assert_eq!(cams.zoom, 1.0);
    zoom_wheel(&mut zs, 10.0);
    settle(&mut cams, &mut sim, &mut zs, centre, 240);
    assert!((cams.zoom - 2.0).abs() < 1e-9, "{}", cams.zoom);

    // a colony much wider than the screen: out to its bounding box plus 256 px each side
    let sim = big_colony();
    let (lo, _) = zoom_limits(&sim, &cams);
    let (w, h) = (12000.0 + 512.0, 1800.0 - 300.0 + 512.0);
    let fit = (cams.game_view.w / w).min(cams.game_view.h / h);
    assert!((lo - fit).abs() < 1e-12, "{lo} vs {fit}");
    assert!(lo < 1.0);
}

#[test]
fn zoom_keeps_point_under_cursor() {
    let mut sim = big_colony();
    sim.camera.x = sim.mycelium.x - 960.0;
    sim.camera.y = 1500.0;
    sim.camera.view_x = sim.camera.x.floor();
    sim.camera.view_y = sim.camera.y.floor();
    let mut cams = retina();
    cams.sync_game_view(&sim.camera);
    let mut zs = ZoomState::default();
    for (cursor, factor) in [
        (Vec2::new(300.0, 250.0), 1.8),
        (Vec2::new(1200.0, 700.0), 0.4),
        (Vec2::new(800.0, 120.0), 1.3),
    ] {
        let w0: DVec2 = cams.screen_to_world(cursor);
        zoom_wheel(&mut zs, factor);
        for frame in 0..90 {
            zoom_advance(&mut cams, &mut sim, &mut zs, 1.0 / 60.0, cursor);
            let w = cams.screen_to_world(cursor);
            assert!(
                (w - w0).length() <= 0.5,
                "frame {frame} zoom {}: {w0:?} -> {w:?}",
                cams.zoom
            );
        }
        // the logic keeps the 1× view centred on the render camera
        let gv = cams.game_view;
        let rv = cams.render_view();
        assert!((gv.x + gv.w / 2.0 - (rv.x + rv.w / 2.0)).abs() <= 1.0);
        assert!((gv.y + gv.h / 2.0 - (rv.y + rv.h / 2.0)).abs() <= 1.0);
        assert_eq!((sim.camera.view_x, sim.camera.view_y), (gv.x, gv.y));
    }
    assert!(cams.zoom < 1.0);
}

#[test]
fn zoom_clamped_to_world_bounds() {
    let mut sim = big_colony();
    // a shallow world (the generated one is a million tiles deep) the knots reach the bottom of
    sim.world.params.bottom_y = sim.world.params.surface_y + 6000.0;
    let bottom = sim.world.params.bottom_y;
    let surface = sim.world.params.surface_y;
    let x = sim.mycelium.x;
    sim.mycelium.knots.push(knot(x, bottom - 10.0));
    sim.state.state.cam_box[3] = bottom + 5000.0;
    let mut cams = Cameras::default();
    let mut zs = ZoomState::default();
    // at the top of the bounds (the sky), cursor at the top edge, all the way out
    sim.camera.y = 0.0;
    sim.camera.view_y = 0.0;
    cams.sync_game_view(&sim.camera);
    zoom_wheel(&mut zs, 0.01);
    settle(&mut cams, &mut sim, &mut zs, Vec2::new(960.0, 5.0), 240);
    let (lo, _) = zoom_limits(&sim, &cams);
    assert!((cams.zoom - lo).abs() < 1e-9, "{} vs {lo}", cams.zoom);
    // the sky top, but not above y = 0 where the backdrop starts
    let sky_top = (surface - cams.game_view.h).max(0.0);
    let rv = cams.render_view();
    assert!(
        rv.y >= sky_top - 1e-6,
        "{} above the sky top {sky_top}",
        rv.y
    );
    assert!(
        rv.y + rv.h <= bottom + 1e-6,
        "{} below the bottom {bottom}",
        rv.y + rv.h
    );
    // at the bottom of the world, cursor at the bottom edge
    sim.camera.y = bottom - cams.game_view.h;
    sim.camera.view_y = sim.camera.y.floor();
    cams.sync_game_view(&sim.camera);
    settle(&mut cams, &mut sim, &mut zs, Vec2::new(960.0, 1075.0), 60);
    let rv = cams.render_view();
    assert!(rv.y >= sky_top - 1e-6);
    assert!(
        rv.y + rv.h <= bottom + 1e-6,
        "{} below {bottom}",
        rv.y + rv.h
    );
    // the mouse maps through the clamped camera
    let p = Vec2::new(960.0, 540.0);
    let w = cams.screen_to_world(p);
    assert!((cams.world_to_screen(w) - p.as_dvec2()).length() < 1e-6);
    assert!((w.y - (rv.y + rv.h / 2.0)).abs() < 1e-6);
}

#[test]
fn zoom_locked_during_cine() {
    let mut sim = big_colony();
    let mut cams = Cameras::default();
    cams.sync_game_view(&sim.camera);
    let mut zs = ZoomState::default();
    zoom_wheel(&mut zs, 0.3);
    settle(&mut cams, &mut sim, &mut zs, Vec2::new(960.0, 540.0), 60);
    assert!(cams.zoom < 1.0);
    assert!(!zoom_locked(&sim));
    cine::cine_show_start(&mut sim);
    assert!(zoom_locked(&sim));
    zoom_advance(&mut cams, &mut sim, &mut zs, 1.0 / 60.0, Vec2::ZERO);
    assert_eq!(cams.zoom, 1.0);
    zoom_wheel(&mut zs, 0.3);
    settle(&mut cams, &mut sim, &mut zs, Vec2::ZERO, 30);
    assert_eq!(cams.zoom, 1.0);
    assert_eq!(zs.target, 1.0);
    // and during the intro
    sim.progress.tasks.cine = None;
    sim.intro_active = true;
    assert!(zoom_locked(&sim));
    zoom_wheel(&mut zs, 1.7);
    settle(&mut cams, &mut sim, &mut zs, Vec2::ZERO, 30);
    assert_eq!(cams.zoom, 1.0);
    sim.intro_active = false;
    zoom_wheel(&mut zs, 1.7);
    settle(&mut cams, &mut sim, &mut zs, Vec2::ZERO, 120);
    assert!((cams.zoom - 1.7).abs() < 1e-3, "{}", cams.zoom);
}
