//! Mycelarium (Rust + Bevy).
//!
//! Environment: `MYCO_ASSETS` (assets folder), `MYCO_SAVE_DIR` (saves and settings.ini; default
//! the LÖVE port's folder), `MYCO_SEED`, `MYCO_INPUT`, `MYCO_DUMP`, `MYCO_SHOT`, `MYCO_FOCUS`
//! (see `autopilot`), `MYCO_FORCE_RAIN=1` (start the rain at once).

use bevy::prelude::*;
use bevy::window::ExitCondition;
use myco_app::assets::{assets_root, AssetsPlugin};
use myco_app::audio::GameAudioPlugin;
use myco_app::autopilot::{seed_from_env, AutopilotPlugin};
use myco_app::camera::CameraPlugin;
use myco_app::display::{self, Display, DisplayPlugin, SaveDir};
use myco_app::render::forest::ForestRenderPlugin;
use myco_app::render::mycelium::MyceliumRenderPlugin;
use myco_app::render::weather::WeatherRenderPlugin;
use myco_app::sim_bridge::{SimBridgePlugin, SimRes};
use myco_app::ui::{MenuPlugin, UiPlugin};
use myco_core::save;
use myco_core::sim::Sim;

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
    ));
    if let Some(s) = screen {
        app.insert_resource(s);
    }
    app.run();
}
