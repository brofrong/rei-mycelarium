//! GUI screens. Task 19: the GameMaker-style GUI painter ([`gm_draw`]) and the upgrade-tree
//! screen ([`tree`]); [`LocaleRes`] holds the language strings and the UI font.
//!
//! Task 17: the in-game HUD ([`hud`]: obj_hud Create/Begin Step/Draw GUI and the `hud_*`
//! helpers), the tasks panel and popups and tutorial tips ([`tasks`]), animal markers and hover
//! bars ([`markers`]), the language fonts ([`fonts`]) and the Rust port's floating resource
//! numbers ([`popups`]), red right-click tip ([`cursor_tip`]) and right-click routing
//! ([`placement`]).
//!
//! Schedules (Task 17):
//! - `FixedUpdate` / `SimSet::Prepare`: [`HudStepSet`] runs obj_hud's Begin Step on the input
//!   latched for this step, before the simulation consumes it; it sets
//!   its `UiCapture.hud` flag and pushes `Command`s. The tree screen's step (Task 19) is in
//!   [`HudStepSet::Screens`]; menus (Task 18) go there too.
//! - `FixedUpdate` / `SimSet::After`: floating numbers and tips read this step's events.
//! - `Update`: the HUD paints into [`gm_draw::GuiDraw`] under the [`draw_order`] keys (Draw_75
//!   order); Task 18's title screens use `draw_order::TITLE`, its pause/logo `draw_order::MODAL`.

pub mod cursor_tip;
pub mod fonts;
pub mod gm_draw;
pub mod hud;
pub mod markers;
pub mod placement;
pub mod popups;
pub mod tasks;
pub mod tree;

use crate::assets::GameAssets;
use crate::display::SaveDir;
use bevy::prelude::*;
use gm_draw::UiFont;
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
        let mut loc = Locale::init(&csv, &locale::read_language(save_dir), &lang, &region);
        // the Rust port's strings (ruling R4)
        loc.extend_from_csv(LOCALE_EXTRA_CSV);
        // ui_font_small(): locale_font_fit(file, -1) (Task 17's full port)
        let font_small = fonts::UiFonts::load(assets, &loc).small;
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

/// The Rust port's extra strings (12 languages, English fallback), compiled in.
pub const LOCALE_EXTRA_CSV: &str = include_str!("../../locale_extra.csv");

/// The painter, the locale, the HUD and the tree screen.
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gm_draw::GuiDrawPlugin, tree::TreePlugin, HudPlugin))
            .add_systems(Startup, init_locale);
    }
}

/// [`gm_draw::GuiDraw`] keys, in obj_hud Draw_75 order.
pub mod draw_order {
    /// world-anchored overlays under the HUD: floating numbers, hover bars
    pub const WORLD: i32 = 100;
    /// top bar (tabs, length, food, water)
    pub const TOP: i32 = 200;
    /// `cave_perk_icons_draw` (Task 13)
    pub const CAVES: i32 = 250;
    /// tasks panel, cave popups (Task 13: 310), markers, points, wind, build bar, buttons,
    /// cheat text, tutorial
    pub const HUD: i32 = 300;
    /// title vignette, slots, credits, title menu (Task 18)
    pub const TITLE: i32 = 400;
    /// hover tooltip, `cine_draw` (Task 21: 460), tasks popup
    pub const TOOLTIP: i32 = 450;
    /// the upgrade tree (Task 19), = `tree::TREE_DRAW_ORDER`
    pub const TREE: i32 = 500;
    /// pause, logo (Task 18)
    pub const MODAL: i32 = 550;
    /// the red right-click tip
    pub const CURSOR: i32 = 900;
}

/// obj_hud Begin Step stages, in `FixedUpdate` inside `SimSet::Prepare` (after the autopilot).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum HudStepSet {
    /// layout, Escape chain, hovers, tasks headers, markers, mouse capture
    Pre,
    /// modal screens (tree: Task 19; pause, title, slots, credits, logo: Task 18)
    Screens,
    /// the in-game HUD: upgrade button, build bar, placement, camera buttons, right click
    Game,
}

/// Logical screen px per GUI unit (`f64` bits), kept by [`track_gui_scale`] for the text
/// minimum ([`min_text_scale`]).
static PX_PER_UNIT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0x3FF0_0000_0000_0000);

/// The owner's minimum text size on screen (logical px).
pub const MIN_TEXT_PX: f64 = 14.0;

pub fn px_per_unit() -> f64 {
    f64::from_bits(PX_PER_UNIT.load(std::sync::atomic::Ordering::Relaxed))
}

/// The scale to draw text of line height `line_h` (GUI units) at, so a line is at least
/// `min_px` logical px on screen.
pub fn min_text_scale(sc: f64, line_h: f64, min_px: f64) -> f64 {
    let px = line_h * sc.abs() * px_per_unit();
    if px > 0.0 && px < min_px {
        sc * min_px / px
    } else {
        sc
    }
}

fn track_gui_scale(cams: Res<crate::camera::Cameras>) {
    let vw = cams.layout.view_w;
    if vw > 0.0 && cams.scale_factor > 0.0 && cams.picture.w > 0.0 {
        let k = cams.picture.w / cams.scale_factor / vw;
        PX_PER_UNIT.store(k.to_bits(), std::sync::atomic::Ordering::Relaxed);
    }
}

/// The HUD (Task 17).
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        use crate::sim_bridge::SimSet;
        app.configure_sets(
            FixedUpdate,
            (HudStepSet::Pre, HudStepSet::Screens, HudStepSet::Game)
                .chain()
                .in_set(SimSet::Prepare)
                .after(crate::autopilot::run_script),
        )
        .init_resource::<popups::Popups>()
        .init_resource::<cursor_tip::CursorTip>()
        .init_resource::<markers::AnimalMarkers>()
        .init_resource::<hud::HudState>()
        .insert_resource(fonts::UiFonts::bitmap())
        .add_systems(Startup, init_fonts.after(init_locale))
        .add_systems(
            FixedUpdate,
            (
                hud::hud_step_pre.in_set(HudStepSet::Pre),
                hud::hud_step_game.in_set(HudStepSet::Game),
                ui_demo.in_set(SimSet::After),
                (
                    popups::popups_collect,
                    cursor_tip::cursor_tip_collect,
                    hud::hud_events,
                )
                    .in_set(SimSet::After)
                    .after(ui_demo),
            ),
        )
        .add_systems(
            Update,
            (
                track_gui_scale,
                popups::popups_tick,
                cursor_tip::cursor_tip_tick,
                popups::world_draw,
                hud::hud_draw_top,
                hud::hud_draw_main,
                hud::hud_draw_tooltip,
                cursor_tip::cursor_tip_draw,
            )
                .chain(),
        );
    }
}

fn init_fonts(mut commands: Commands, assets: Res<GameAssets>, locale: Option<Res<LocaleRes>>) {
    if let Some(l) = locale {
        commands.insert_resource(fonts::UiFonts::load(&assets, &l.locale));
    }
}

/// `MYCO_UI_DEMO=1` (dev hook for screenshots): every 0.5 s synthetic resource events around the
/// view centre and a refused node, as the logic tasks (T5/T8/T9) will emit them.
fn ui_demo(
    sim: Res<crate::sim_bridge::SimRes>,
    cams: Res<crate::camera::Cameras>,
    mut out: MessageWriter<crate::sim_bridge::SimEventMsg>,
    mut on: Local<Option<bool>>,
) {
    use myco_core::events::{NodeReject, ResourceKind, SimEvent};
    let on = *on.get_or_insert_with(|| std::env::var("MYCO_UI_DEMO").is_ok_and(|v| v == "1"));
    if !on || !sim.0.frame.is_multiple_of(30) {
        return;
    }
    let (cx, cy) = cams.game_view_center();
    let k = (sim.0.frame / 30) as f64;
    let ev = |kind, delta, dx: f64, dy: f64, source| SimEvent::Resource {
        kind,
        delta,
        x: cx + dx,
        y: cy + dy,
        source,
    };
    for e in [
        ev(ResourceKind::Food, 5.0, -300.0 + 20.0 * (k % 5.0), 80.0, 1),
        ev(ResourceKind::Food, 3.0, -300.0, 80.0, 1),
        ev(ResourceKind::Water, 12.0, 200.0, 140.0, 2),
        ev(ResourceKind::Food, -15.0, -40.0, 220.0, 3),
        ev(ResourceKind::UpgradePoints, 1.0, 420.0, 40.0, 4),
        SimEvent::NodeRejected {
            reason: NodeReject::NotEnoughFood { need: 12 },
            x: cx,
            y: cy,
        },
    ] {
        out.write(crate::sim_bridge::SimEventMsg(e));
    }
}
