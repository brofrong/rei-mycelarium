//! The settings items shared by the title's "Settings" and the in-game pause menu: sound and
//! music sliders, screen mode, window size, units, language (obj_hud `hud_pause_*`,
//! `hud_volume_*`). The screen around them is in `pause.rs`.

use super::menu::*;
use myco_core::locale::Locale;

/// `hud_volume_labels` (for `hud_volume_channels` sfx, music)
pub const VOLUME_LABELS: [&str; 2] = ["volume_sfx", "volume_music"];

// port: anon__11820__gml_Object_obj_hud_Create_0
/// `hud_pause_cy()`: vertical anchor of the settings screen.
pub fn hud_pause_cy(gh: f64) -> f64 {
    gh * 0.5 - 200.0
}

// port: anon__16043__gml_Object_obj_hud_Create_0
/// `hud_volume_slider_rect(i)`: 420×26 sliders above the buttons.
pub fn hud_volume_slider_rect(gw: f64, gh: f64, i: usize) -> R {
    let w = 420.0;
    let x = (gw - w) * 0.5;
    let y = hud_pause_cy(gh) - 112.0 + i as f64 * 62.0;
    R {
        x1: x,
        y1: y,
        x2: x + w,
        y2: y + 26.0,
    }
}

// port: anon__16388__gml_Object_obj_hud_Create_0
/// `hud_pause_button_rect(i)`: 420×64 buttons, 12 px apart.
pub fn hud_pause_button_rect(gw: f64, gh: f64, i: usize) -> R {
    let (w, h) = (420.0, 64.0);
    let x = (gw - w) * 0.5;
    let y = hud_pause_cy(gh) + 48.0 + i as f64 * (h + 12.0);
    R {
        x1: x,
        y1: y,
        x2: x + w,
        y2: y + h,
    }
}

// port: anon__11912__gml_Object_obj_hud_Create_0
/// `hud_pause_item_disabled(item)`: the window size cannot change in fullscreen.
pub fn hud_pause_item_disabled(item: &str, fullscreen: bool) -> bool {
    item == "window_size" && fullscreen
}

/// A settings button's label with its current value ("LANGUAGE: English", "WINDOW: 1280x720").
pub fn pause_item_label(item: &str, loc: &Locale, env: &MenuEnv) -> String {
    let label = loc.loc(&format!("button_{item}")).to_string();
    match item {
        "language" => format!("{label}: {}", loc.loc("language_name")),
        "units" => format!(
            "{label}: {}",
            loc.loc(if env.flags.units_imperial {
                "units_imperial"
            } else {
                "units_metric"
            })
        ),
        "display_mode" => format!(
            "{label}: {}",
            loc.loc(if env.fullscreen {
                "display_fullscreen"
            } else {
                "display_windowed"
            })
        ),
        "window_size" => format!("{label}: {}x{}", env.window_size.0, env.window_size.1),
        _ => label,
    }
}
