//! Mycelarium (Rust + Bevy).
//!
//! Environment: `MYCO_ASSETS` (assets folder), `MYCO_SAVE_DIR` (saves and settings.ini; default
//! the LÖVE port's folder), `MYCO_SEED`, `MYCO_INPUT`, `MYCO_DUMP`, `MYCO_SHOT`, `MYCO_FOCUS`
//! (see `autopilot`).

use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::window::ExitCondition;
use myco_app::assets::{assets_root, AssetsPlugin, GameAssets};
use myco_app::autopilot::{seed_from_env, AutopilotPlugin};
use myco_app::camera::{world_to_bevy, CameraPlugin, Cameras, GUI_LAYER};
use myco_app::display::{self, Display, DisplayPlugin, SaveDir};
use myco_app::sim_bridge::{SimBridgePlugin, SimRes};
use myco_core::locale::FontId;
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
    ))
    .add_systems(Startup, spawn_demo_scene)
    .add_systems(Update, follow_demo_scene);
    if let Some(s) = screen {
        app.insert_resource(s);
    }
    app.run();
}

/// Temporary scene until the render tasks: a mushroom sprite at the centre of the game view and
/// the title in a font from `GameAssets`.
#[derive(Component)]
struct DemoMushroom;

fn spawn_demo_scene(mut commands: Commands, assets: Res<GameAssets>, cams: Res<Cameras>) {
    let (cx, cy) = cams.game_view_center();
    let def = assets.try_sprite("sporeshroom").unwrap_or(&assets.sprites[0]);
    let (sprite, anchor) = def.bevy_sprite(0);
    commands.spawn((
        sprite,
        anchor,
        Transform::from_translation(world_to_bevy(cx, cy).extend(0.0)).with_scale(Vec3::splat(4.0)),
        DemoMushroom,
    ));
    let font = assets.font(FontId::CyrGerSpaPort);
    commands.spawn((
        Text2d::new("Mycelarium"),
        TextFont::from_font_size(96.0).with_font(font),
        TextColor(Color::srgb(0.85, 0.95, 0.8)),
        Transform::from_xyz(
            (cams.layout.view_w / 2.0) as f32,
            -((cams.layout.view_h / 2.0) as f32) + 260.0,
            1.0,
        ),
        RenderLayers::layer(GUI_LAYER),
    ));
}

fn follow_demo_scene(cams: Res<Cameras>, mut q: Query<&mut Transform, With<DemoMushroom>>) {
    let (cx, cy) = cams.game_view_center();
    for mut tf in &mut q {
        tf.translation = world_to_bevy(cx, cy).extend(0.0);
    }
}
