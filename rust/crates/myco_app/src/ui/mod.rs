//! GUI screens. Task 19: the GameMaker-style GUI painter ([`gm_draw`]) and the upgrade-tree
//! screen ([`tree`]); [`LocaleRes`] holds the language strings and the UI font.

pub mod gm_draw;
pub mod tree;

use crate::assets::GameAssets;
use crate::display::SaveDir;
use bevy::prelude::*;
use gm_draw::{TtfFont, UiFont};
use myco_core::locale::{self, Locale};

/// The game's strings (`loc`) and `ui_font_small()` as the painter uses it: `fnt_gm_20` for
/// `Font1` languages, else the language TTF fitted like `locale_font_fit(file, -1)`.
#[derive(Resource)]
pub struct LocaleRes {
    pub locale: Locale,
    pub font_small: UiFont,
}

/// `AppleLocale` (or `LANG`) as the LÖVE port reads it: `(language, region)`.
pub fn os_locale() -> (String, String) {
    let s = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("LANG").ok())
        .unwrap_or_else(|| "en_US".into());
    let lang = s
        .get(0..2)
        .filter(|l| l.chars().all(|c| c.is_ascii_alphabetic()));
    let region = s
        .get(3..5)
        .filter(|r| r.chars().all(|c| c.is_ascii_alphabetic()));
    (
        lang.unwrap_or("en").to_lowercase(),
        region.unwrap_or("US").to_uppercase(),
    )
}

impl LocaleRes {
    pub fn load(assets: &GameAssets, save_dir: &std::path::Path) -> LocaleRes {
        let csv = assets.read_text("localization.csv").unwrap_or_default();
        let (lang, region) = os_locale();
        let loc = Locale::init(&csv, &locale::read_language(save_dir), &lang, &region);
        let font_small = loc
            .font_small_for()
            .and_then(|id| {
                let bytes = std::fs::read(assets.root.join(id.file()?)).ok()?;
                let ref_h = assets
                    .gm_font("fnt_gm_20")
                    .map(|f| f.glyphs.values().map(|g| g.h as f64).fold(0.0, f64::max))
                    .unwrap_or(31.0);
                TtfFont::fit(assets.font(id), bytes, ref_h, 12.0)
            })
            .map(UiFont::Ttf)
            .unwrap_or_else(|| UiFont::Gm("fnt_gm_20".into()));
        LocaleRes {
            locale: loc,
            font_small,
        }
    }
}

fn init_locale(mut commands: Commands, assets: Res<GameAssets>, dir: Option<Res<SaveDir>>) {
    let dir = dir
        .map(|d| d.0.clone())
        .unwrap_or_else(crate::display::save_dir);
    commands.insert_resource(LocaleRes::load(&assets, &dir));
}

/// The painter, the locale and the tree screen.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gm_draw::GuiDrawPlugin, tree::TreePlugin))
            .add_systems(Startup, init_locale);
    }
}
