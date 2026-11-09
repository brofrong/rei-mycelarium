//! `view_for_window` against `game/gm/display_math.lua`, settings.ini writes, save-dir override.

use myco_app::display::{self, view_for_window, ViewLayout};
use myco_core::state::DisplaySettings;

fn layout(
    view_w: f64,
    view_h: f64,
    surf_w: f64,
    surf_h: f64,
    ui_ox: f64,
    ui_by: f64,
) -> ViewLayout {
    ViewLayout {
        view_w,
        view_h,
        surf_w,
        surf_h,
        ui_ox,
        ui_by,
    }
}

/// Expected values printed by `display_math.view_for_window` under Lua 5.4.
#[test]
fn view_for_window_matches_lua() {
    let cases = [
        (
            (3024.0, 1898.0),
            layout(1920.0, 1205.0, 3024.0, 1898.0, 0.0, 125.0),
        ),
        (
            (1280.0, 720.0),
            layout(1920.0, 1080.0, 1280.0, 720.0, 0.0, 0.0),
        ),
        (
            (1000.0, 1000.0),
            layout(1920.0, 1536.0, 1000.0, 800.0, 0.0, 456.0),
        ),
        (
            (4000.0, 1000.0),
            layout(3840.0, 1080.0, 3556.0, 1000.0, 960.0, 0.0),
        ),
        // tests/test_display.lua
        (
            (3024.0, 1890.0),
            layout(1920.0, 1200.0, 3024.0, 1890.0, 0.0, 120.0),
        ),
        (
            (3840.0, 1600.0),
            layout(2592.0, 1080.0, 3840.0, 1600.0, 336.0, 0.0),
        ),
        (
            (7680.0, 4320.0),
            layout(1920.0, 1080.0, 3840.0, 2160.0, 0.0, 0.0),
        ),
        (
            (5000.0, 1000.0),
            layout(3840.0, 1080.0, 3556.0, 1000.0, 960.0, 0.0),
        ),
    ];
    for ((ww, wh), want) in cases {
        assert_eq!(view_for_window(ww, wh), Some(want), "window {ww}x{wh}");
    }
    assert_eq!(view_for_window(0.0, 900.0), None);
    assert_eq!(view_for_window(900.0, -1.0), None);
}

/// `display_math.fit` on the notch case of tests/test_display.lua.
#[test]
fn fit_fills_safe_area_below_notch() {
    let v = view_for_window(3024.0, 1890.0).unwrap();
    let f = display::fit(
        3024.0, 1964.0, 0.0, 74.0, 3024.0, 1890.0, v.surf_w, v.surf_h,
    );
    assert_eq!((f.x, f.y, f.w, f.h), (0.0, 74.0, 3024.0, 1890.0));
    let degenerate = display::fit(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0);
    assert_eq!(degenerate.scale, 0.0);
}

/// The safe area excludes the notch only when the window covers the whole screen.
#[test]
fn safe_area_excludes_notch_when_window_covers_screen() {
    let screen = display::ScreenInfo {
        width_pt: 1512.0,
        height_pt: 982.0,
        scale: 2.0,
        inset_top_pt: 32.0,
    };
    let s = display::safe_area_in_window(3024.0, 1964.0, Some(&screen));
    assert_eq!((s.x, s.y, s.w, s.h), (0.0, 64.0, 3024.0, 1900.0));
    // native fullscreen already below the notch, or a normal window: the whole window
    let s = display::safe_area_in_window(3024.0, 1898.0, Some(&screen));
    assert_eq!((s.x, s.y, s.w, s.h), (0.0, 0.0, 3024.0, 1898.0));
    let s = display::safe_area_in_window(1280.0, 720.0, None);
    assert_eq!((s.x, s.y, s.w, s.h), (0.0, 0.0, 1280.0, 720.0));
}

/// Display settings go through `save::ini_write`, so `[game] language` survives.
#[test]
fn writing_display_settings_keeps_other_sections() {
    let dir = std::env::temp_dir().join(format!("myco_app_settings_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.ini"), "[game]\r\nlanguage=\"ru\"\r\n").unwrap();
    let s = DisplaySettings {
        fullscreen: false,
        window_w: 1440,
        window_h: 810,
    };
    display::write_display_settings(&dir, &s).unwrap();
    let text = std::fs::read_to_string(dir.join("settings.ini")).unwrap();
    assert_eq!(
        myco_core::save::ini_read(&text, "game", "language").as_deref(),
        Some("ru")
    );
    assert_eq!(myco_core::save::read_settings(&dir), s);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn save_dir_honours_env_override() {
    assert_eq!(
        display::save_dir_from(Some("/tmp/myco-x".into())),
        std::path::PathBuf::from("/tmp/myco-x")
    );
    assert_eq!(display::save_dir_from(None), myco_core::save::save_dir());
}

/// Automated runs (MYCO_INPUT / MYCO_DUMP / MYCO_SHOT) without MYCO_SAVE_DIR write into a
/// temp folder, never into the LÖVE port's real save folder.
#[test]
fn automated_runs_default_to_a_temp_save_dir() {
    let real = myco_core::save::save_dir();
    let auto = display::save_dir_for(None, true);
    assert_ne!(auto, real);
    assert!(auto.starts_with(std::env::temp_dir()), "{auto:?}");
    assert_eq!(display::save_dir_for(None, false), real);
    assert_eq!(
        display::save_dir_for(Some("/tmp/myco-y".into()), true),
        std::path::PathBuf::from("/tmp/myco-y")
    );
}

/// Ruling R23: a settings.ini window size is logical points (LÖVE highdpi without DPI scaling),
/// so 1366×768 opens a 1366×768-point window on any scale factor.
#[test]
fn settings_window_size_is_logical_points() {
    let dir = std::env::temp_dir().join(format!("myco-t18-winsize-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("settings.ini"),
        "[display]\r\nfullscreen=\"0.000000\"\r\nwindow_w=\"1366.000000\"\r\nwindow_h=\"768.000000\"\r\n",
    )
    .unwrap();
    let settings = myco_core::save::read_settings(&dir);
    assert_eq!(
        (settings.window_w, settings.window_h, settings.fullscreen),
        (1366, 768, false)
    );
    for scale in [1.0, 2.0, 3.0] {
        let screen = display::ScreenInfo {
            width_pt: 1512.0,
            height_pt: 982.0,
            scale,
            inset_top_pt: 32.0,
        };
        let w = display::primary_window(&settings, Some(&screen));
        assert_eq!(
            (w.resolution.width(), w.resolution.height()),
            (1366.0, 768.0),
            "scale {scale}"
        );
    }
}

/// The original has no fullscreen hotkeys: Cmd+F and Alt+Enter do nothing (Alt+Enter reaches the
/// game only as Enter); Cmd+Esc still quits, as in the LÖVE port.
#[test]
fn no_fullscreen_hotkeys() {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<display::SetFullscreen>()
        .add_message::<AppExit>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_systems(Update, display::display_keys);
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    let count = |app: &mut App| {
        app.world()
            .resource::<Messages<display::SetFullscreen>>()
            .iter_current_update_messages()
            .count()
    };
    for combo in [
        [KeyCode::SuperLeft, KeyCode::KeyF],
        [KeyCode::AltLeft, KeyCode::Enter],
    ] {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.reset_all();
        keys.press(combo[0]);
        keys.press(combo[1]);
        app.update();
        assert_eq!(count(&mut app), 0, "{combo:?} must not toggle fullscreen");
    }
}

/// The game's `display_fit` (core_3.lua:370, the decompiled original) rounds with `gm.round`,
/// GML's banker's rounding: a view width or height of exactly n.5 goes to the even n.
#[test]
fn display_fit_rounds_half_to_even() {
    // 1080 * 4001 / 2160 = 2000.5 -> 2000
    assert_eq!(
        display::view_for_window(4001.0, 2160.0).unwrap().view_w,
        2000.0
    );
    // 1920 * 2401 / 3840 = 1200.5 -> 1200
    assert_eq!(
        display::view_for_window(3840.0, 2401.0).unwrap().view_h,
        1200.0
    );
    // 1080 * 4003 / 2160 = 2001.5 -> 2002
    assert_eq!(
        display::view_for_window(4003.0, 2160.0).unwrap().view_w,
        2002.0
    );
}

/// display_apply / display_step (core_3.lua:413, 460): switching to a window waits 4 steps;
/// at 2 left the window takes the chosen size, at 0 it is centred. Fullscreen needs no wait.
#[test]
fn display_step_resizes_then_centres_a_window() {
    use display::{display_apply_pending, display_step_tick, DisplayAction};
    assert_eq!(display_apply_pending(true), 0);
    let mut pending = display_apply_pending(false);
    assert_eq!(pending, 4);
    let ticks: Vec<_> = (0..6).map(|_| display_step_tick(&mut pending)).collect();
    assert_eq!(
        ticks,
        vec![
            None,
            Some(DisplayAction::SetSize),
            None,
            Some(DisplayAction::Centre),
            None,
            None
        ]
    );
}

/// display_cycle_size goes through display_apply: a window takes the new size two steps later
/// (display_step), not at once, and the choice is saved.
#[test]
fn cycling_the_window_size_uses_the_countdown() {
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use display::{
        apply_cycle_size, display_step, CycleWindowSize, Display, DisplayPending, DisplaySizes,
        SaveDir,
    };
    let dir = std::env::temp_dir().join(format!("myco-t22-cycle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut world = World::new();
    world.init_resource::<Messages<CycleWindowSize>>();
    world.insert_resource(DisplaySizes::new(Some((3024.0, 1964.0)), Some((1280, 720))));
    let settings = myco_core::state::DisplaySettings {
        fullscreen: false,
        window_w: 1280,
        window_h: 720,
    };
    world.insert_resource(Display(settings));
    world.insert_resource(DisplayPending(0));
    world.insert_resource(SaveDir(dir.clone()));
    let mut w = Window::default();
    w.resolution.set(1280.0, 720.0);
    world.spawn((w, PrimaryWindow));
    world.write_message(CycleWindowSize(1));
    world.run_system_once(apply_cycle_size).unwrap();
    assert_eq!(world.resource::<DisplayPending>().0, 4);
    let size = |world: &mut World| {
        let mut q = world.query::<&Window>();
        let w = q.single(world).unwrap();
        (w.resolution.width(), w.resolution.height())
    };
    assert_eq!(size(&mut world), (1280.0, 720.0), "not at once");
    world.run_system_once(display_step).unwrap();
    world.run_system_once(display_step).unwrap();
    assert_eq!(size(&mut world), (1366.0, 768.0), "two steps later");
    assert!(myco_core::save::read_settings(&dir).window_w == 1366);
    let _ = std::fs::remove_dir_all(&dir);
}
