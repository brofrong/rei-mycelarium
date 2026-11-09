//! Mycelarium (Rust + Bevy).
//!
//! Environment: `MYCO_ASSETS` (assets folder), `MYCO_SAVE_DIR` (saves and settings.ini; default
//! the LÖVE port's folder), `MYCO_SEED`, `MYCO_INPUT`, `MYCO_DUMP`, `MYCO_SHOT`, `MYCO_FOCUS`
//! (see `autopilot`), `MYCO_FORCE_RAIN=1` (start the rain at once), `MYCO_CAMERA=x,y` (camera top-left at start),
//! `MYCO_CAVE_TAKE=<cave id>` (take a cave at start).

use bevy::prelude::*;
use bevy::window::ExitCondition;
use myco_app::assets::{assets_root, AssetsPlugin};
use myco_app::audio::GameAudioPlugin;
use myco_app::autopilot::{seed_from_env, AutopilotPlugin};
use myco_app::camera::CameraPlugin;
use myco_app::display::{self, Display, DisplayPlugin, SaveDir};
use myco_app::render::forest::ForestRenderPlugin;
use myco_app::render::weather::WeatherRenderPlugin;
use myco_app::render::world::WorldRenderPlugin;
use myco_app::sim_bridge::{SimBridgePlugin, SimRes};
use myco_app::ui::UiPlugin;
use myco_core::save;
use myco_core::sim::Sim;
use myco_core::state::GameMode;

fn main() {
    let save_dir = display::save_dir();
    let settings = save::read_settings(&save_dir);
    // NSScreen must be queried on the main thread, before the event loop starts
    let screen = display::main_screen_info();
    let window = display::primary_window(&settings, screen.as_ref());

    let mut sim = Sim::new_game(GameMode::Exploration, "", seed_from_env());
    sim.save_dir = Some(save_dir.clone());
    // MYCO_FORCE_RAIN=1: start the rain cycle now (the original's weather_force_rain debug hook;
    // used for render comparisons)
    if std::env::var("MYCO_FORCE_RAIN").is_ok_and(|v| v == "1") {
        myco_core::weather::force_rain(&mut sim);
    }

    // MYCO_CAMERA=x,y: put the camera's top-left there at start (render comparisons; the LÖVE
    // side sets the forest controller's camera_x/camera_y through MYCO_PROBE)
    if let Some((x, y)) = std::env::var("MYCO_CAMERA").ok().and_then(|v| {
        let (x, y) = v.split_once(',')?;
        Some((x.trim().parse::<f64>().ok()?, y.trim().parse::<f64>().ok()?))
    }) {
        sim.camera.x = x;
        sim.camera.y = y;
    }

    // MYCO_CAVE_TAKE=<cave id>: take that cave at start (render comparisons; LÖVE: cave_take
    // through MYCO_PROBE)
    if let Ok(id) = std::env::var("MYCO_CAVE_TAKE") {
        let cave = sim.world.cave_rects().iter().find(|c| c.id == id).cloned();
        if let Some(c) = cave {
            myco_core::world::caves::cave_take(&mut sim, &c, None, None);
        }
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
        WorldRenderPlugin,
    ));
    if let Some(s) = screen {
        app.insert_resource(s);
    }
    app.run();
}
