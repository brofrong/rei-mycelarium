//! Mycelarium (Rust + Bevy).
//!
//! Environment: `MYCO_ASSETS` (assets folder), `MYCO_SAVE_DIR` (saves and settings.ini; default
//! the LÖVE port's folder), `MYCO_SEED`, `MYCO_INPUT`, `MYCO_DUMP`, `MYCO_SHOT`, `MYCO_FOCUS`
//! (see `autopilot`), `MYCO_FORCE_RAIN=1` (start the rain at once), `MYCO_CAMERA=x,y`
//! (camera top-left at game start), `MYCO_CAVE_TAKE=<cave id>` (take a cave at game start),
//! `MYCO_BW=1` (BLACK METAL post effect on; the shipped menus cannot turn it on).

use bevy::prelude::*;
use bevy::window::ExitCondition;
use myco_app::assets::{assets_root, AssetsPlugin};
use myco_app::audio::GameAudioPlugin;
use myco_app::autopilot::{seed_from_env, AutopilotPlugin};
use myco_app::camera::CameraPlugin;
use myco_app::cine::CinePlugin;
use myco_app::display::{self, Display, DisplayPlugin, SaveDir};
use myco_app::render::forest::ForestRenderPlugin;
use myco_app::render::life::LifeRenderPlugin;
use myco_app::render::mycelium::MyceliumRenderPlugin;
use myco_app::render::post::PostPlugin;
use myco_app::render::weather::WeatherRenderPlugin;
use myco_app::render::world::WorldRenderPlugin;
use myco_app::sim_bridge::{SimBridgePlugin, SimRes};
use myco_app::ui::{MenuPlugin, UiPlugin};
use myco_core::save;
use myco_core::sim::Sim;

/// MYCO_CAMERA=x,y (camera top-left) and MYCO_CAVE_TAKE=<cave id> (take that cave), applied once
/// when the game starts (the title closes): render comparisons; the LÖVE side does the same
/// through MYCO_PROBE (forest controller camera_x/camera_y, cave_take).
fn comparison_hooks(mut sim: ResMut<SimRes>, mut done: Local<bool>) {
    let sim = &mut sim.0;
    if *done || sim.title_open {
        return;
    }
    *done = true;
    if let Some((x, y)) = std::env::var("MYCO_CAMERA").ok().and_then(|v| {
        let (x, y) = v.split_once(',')?;
        Some((x.trim().parse::<f64>().ok()?, y.trim().parse::<f64>().ok()?))
    }) {
        sim.camera.x = x;
        sim.camera.y = y;
        // camera_bounds is real now (Task 21): grow the camera box so the forest step keeps the
        // requested view (the LÖVE side neutralises camera_bounds through MYCO_PROBE)
        let (w, h) = (sim.camera.width, sim.camera.height);
        let b = &mut sim.state.state.cam_box;
        b[0] = b[0].min(x);
        b[1] = b[1].min(y);
        b[2] = b[2].max(x + w);
        b[3] = b[3].max(y + h);
    }
    if let Ok(id) = std::env::var("MYCO_CAVE_TAKE") {
        let cave = sim.world.cave_rects().iter().find(|c| c.id == id).cloned();
        if let Some(c) = cave {
            myco_core::world::caves::cave_take(sim, &c, None, None);
        }
    }
}

fn main() {
    let save_dir = display::save_dir();
    let settings = save::read_settings(&save_dir);
    // NSScreen must be queried on the main thread, before the event loop starts
    let screen = display::main_screen_info();
    let window = display::primary_window(&settings, screen.as_ref());

    // the game boots into the title menu over a fresh world (obj_game Create_0)
    let mut sim = Sim::title_screen(seed_from_env());
    sim.save_dir = Some(save_dir.clone());
    // MYCO_FORCE_RAIN=1: start the rain cycle now (the original's weather_force_rain debug hook;
    // used for render comparisons)
    if std::env::var("MYCO_FORCE_RAIN").is_ok_and(|v| v == "1") {
        myco_core::weather::force_rain(&mut sim);
    }

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(window),
                exit_condition: ExitCondition::OnPrimaryClosed,
                ..default()
            })
            .set(AssetPlugin {
                file_path: assets_root().to_string_lossy().into_owned(),
                ..default()
            }),
    )
    .insert_resource(ClearColor(Color::BLACK))
    .insert_resource(SaveDir(save_dir))
    .insert_resource(Display(settings))
    .insert_resource(SimRes(sim))
    .add_plugins((
        AssetsPlugin,
        DisplayPlugin,
        CameraPlugin,
        SimBridgePlugin,
        AutopilotPlugin,
        GameAudioPlugin,
        UiPlugin,
        ForestRenderPlugin,
        WeatherRenderPlugin,
        MyceliumRenderPlugin,
        MenuPlugin,
        WorldRenderPlugin,
        LifeRenderPlugin,
        PostPlugin,
        CinePlugin,
    ));
    app.add_systems(Update, comparison_hooks);
    if let Some(s) = screen {
        app.insert_resource(s);
    }
    app.run();
}
