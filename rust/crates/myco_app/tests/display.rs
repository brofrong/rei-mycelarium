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
