//! Life logic that reads the camera view sees only the 1× game view: the render zoom never
//! reaches it (spec; Task 10's `worm_keep_zone_uses_game_view`, `squirrel_view_uses_game_view`).

use myco_app::camera::Cameras;
use myco_app::sim_bridge::PendingInput;
use myco_core::life::{self, worm};
use myco_core::rng::Rng;
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::GameMode;

/// Run 240 steps of a fresh game with the app's step input at render `zoom`; the logic's view,
/// the worm keep zone and the worms that live in it.
fn run(zoom: f64) -> (ViewRect, ViewRect, f64, Vec<(f64, f64)>) {
    let cams = Cameras {
        zoom,
        ..Default::default()
    };
    let render = cams.render_view();
    let mut sim = Sim::new_game(GameMode::Exploration, "view", Some(31607));
    sim.intro_active = false;
    // game_init rolls the wind with a clock-seeded RNG: fix it and the RNG
    sim.state.state.wind = [1.0, 2.0, 420.0];
    sim.rng = Rng::new(17);
    let mut pending = PendingInput::default();
    for k in 0..240 {
        let input = pending.take_step(&cams);
        if k == 0 {
            for _ in 0..4 {
                worm::worm_spawn(&mut sim, true);
            }
        }
        sim.step(&input);
    }
    let v = life::squirrel_view(&sim);
    let keep = worm::worm_keep_bottom(&sim, v.y, v.h);
    let worms = sim
        .life
        .worms
        .iter()
        .map(|w| (w.body[0].x, w.body[0].y))
        .collect();
    (render, v, keep, worms)
}

#[test]
fn squirrel_view_uses_game_view() {
    let (r_far, v_far, _, _) = run(0.3);
    let (r_one, v_one, _, _) = run(1.0);
    assert_ne!(r_far, r_one, "the render camera does zoom");
    assert_eq!(v_far, v_one, "the logic view ignores the zoom");
    let game = Cameras::default().game_view;
    assert_eq!((v_one.w, v_one.h), (game.w, game.h), "1× game view size");
}

#[test]
fn worm_keep_zone_uses_game_view() {
    let (_, _, keep_far, worms_far) = run(0.3);
    let (_, _, keep_one, worms_one) = run(1.0);
    assert_eq!(keep_far, keep_one);
    assert!(!worms_one.is_empty(), "worms live in the zone");
    assert_eq!(worms_far, worms_one, "same worms at either zoom");
}
