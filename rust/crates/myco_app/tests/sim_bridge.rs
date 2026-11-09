//! Input latching between `Update` and the 60 Hz `FixedUpdate` simulation step.

use bevy::input::mouse::MouseButtonInput;
use bevy::input::{ButtonState as BevyButton, InputPlugin};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use myco_app::camera::Cameras;
use myco_app::sim_bridge::{SimBridgePlugin, SimRes, StepHistory};
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use std::time::Duration;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin, SimBridgePlugin));
    app.insert_resource(SimRes(Sim::new_game(
        GameMode::Exploration,
        "test",
        Some(7),
    )));
    app.insert_resource(Cameras::default());
    app.insert_resource(StepHistory::default());
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
    app.update(); // startup; time starts
    app
}

fn click(app: &mut App, state: BevyButton) {
    app.world_mut().write_message(MouseButtonInput {
        button: MouseButton::Left,
        state,
        window: Entity::PLACEHOLDER,
    });
}

#[test]
fn fixed_timestep_is_60_hz() {
    let app = app();
    let step = app.world().resource::<Time<Fixed>>().timestep();
    assert_eq!(step, Duration::from_secs_f64(1.0 / 60.0));
}

#[test]
fn latched_click_survives_frame_without_fixed_step() {
    let mut app = app();
    let step = app.world().resource::<Time<Fixed>>().timestep();
    // two frames with no fixed step: the click (press and release) lands in the first
    click(&mut app, BevyButton::Pressed);
    app.update();
    click(&mut app, BevyButton::Released);
    app.update();
    assert!(
        app.world().resource::<StepHistory>().0.is_empty(),
        "no fixed step yet"
    );

    app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    app.update();
    app.update();
    let hist = &app.world().resource::<StepHistory>().0;
    assert_eq!(hist.len(), 2, "one fixed step per 1/60 s frame");
    let pressed: Vec<bool> = hist.iter().map(|i| i.lmb.pressed).collect();
    assert_eq!(
        pressed,
        vec![true, false],
        "exactly one pressed, in the next step"
    );
    assert!(hist[0].lmb.released && !hist[0].lmb.down);
    assert!(!hist[1].lmb.released);
}

#[test]
fn held_button_stays_down_across_steps() {
    let mut app = app();
    let step = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    click(&mut app, BevyButton::Pressed);
    app.update();
    app.update();
    let hist = &app.world().resource::<StepHistory>().0;
    assert_eq!(hist.len(), 2);
    assert!(hist[0].lmb.pressed && hist[0].lmb.down);
    assert!(!hist[1].lmb.pressed && hist[1].lmb.down);
}

/// LÖVE caps the step accumulator at 5 steps (game/gm/init.lua `acc = min(acc + dt, step * 5)`):
/// after a long hitch at most 5 steps run in one frame.
#[test]
fn catch_up_is_capped_at_five_steps() {
    let mut app = app();
    let max = app.world().resource::<Time<Virtual>>().max_delta();
    let step = app.world().resource::<Time<Fixed>>().timestep();
    assert_eq!(max, step * 5);
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs(2)));
    app.update();
    assert_eq!(app.world().resource::<StepHistory>().0.len(), 5);
}
