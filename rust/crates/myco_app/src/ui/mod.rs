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
//!
//! Task 18: the menus (main menu, slots, naming, settings/pause, credits, the logo generator)
//! and saving on quit. `menu.rs` is the state machine; [`MenuPlugin`] wires it:
//! - fixed step, `SimSet::Prepare`, after the autopilot and before the tree: [`menu_step`] reads
//!   the latched input, runs the menus (which may start, load or restart the game), applies their
//!   effects, then sets `PendingInput.{debug,logo}_open/hud_hidden`, [`UiCapture::menus`] and
//!   [`UiBlockers::menus`];
//! - `Update`: [`menu_draw`] paints into [`GuiDraw`] under [`MENU_DRAW_ORDER`] (over the tree);
//! - `MYCO_LOGO=<text>` opens the logo generator with that text once a game runs (checks);
//! - `Last`: [`save_on_exit`] runs obj_game Other_3 (`Sim::game_end`, a save) once when the app
//!   exits for any reason (Quit/Exit item, Cmd+Esc, closing the window).

pub mod credits;
pub mod cursor_tip;
pub mod draw;
pub mod fonts;
pub mod gm_draw;
pub mod hud;
pub mod logo_screen;
pub mod markers;
pub mod menu;
pub mod name;
pub mod pause;
pub mod placement;
pub mod popups;
pub mod settings;
pub mod slots;
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

pub fn init_locale(mut commands: Commands, assets: Res<GameAssets>, dir: Option<Res<SaveDir>>) {
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

/// Set the logical px per GUI unit (normally kept by `track_gui_scale`; tests).
pub fn set_px_per_unit(k: f64) {
    PX_PER_UNIT.store(k.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

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
                    hud::hud_tasks_fill,
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

// ---- Task 18: menus --------------------------------------------------------------------------

use crate::audio::{Audio, AudioRequest, VolumeKind};
use crate::autopilot::{run_script, seed_from_env};
use crate::camera::Cameras;
use crate::display::{CycleWindowSize, Display, DisplaySizes, SetFullscreen};
use crate::sim_bridge::{PendingInput, SimRes, SimSet, UiBlockers, UiCapture};
use draw::{GmMetrics, MenuFonts};
use gm_draw::GuiDraw;
use menu::{MenuEffect, MenuEnv, MenuInput, Menus, UiFlags};
use myco_core::sim::{Key, Sim};
use tree::TreeScreen;

/// [`GuiDraw`] key of the menus: above the tree screen (the logo generator opens over it).
pub const MENU_DRAW_ORDER: i32 = 600;

/// obj_game Other_3 already ran.
#[derive(Resource, Default)]
pub struct ExitSaved(pub bool);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFlags>()
            .init_resource::<ExitSaved>()
            .add_systems(Startup, init_menus.after(init_locale))
            .add_systems(
                FixedUpdate,
                // obj_hud Step_1: the menus and the single Escape chain come first, then the
                // HUD's stages (Task 17) and the tree (in `HudStepSet::Screens`)
                menu_step
                    .in_set(SimSet::Prepare)
                    .after(run_script)
                    .before(HudStepSet::Pre),
            )
            .add_systems(Update, menu_draw)
            .add_systems(Last, save_on_exit);
    }
}

fn init_menus(
    mut commands: Commands,
    assets: Res<GameAssets>,
    dir: Option<Res<SaveDir>>,
    loc: Res<LocaleRes>,
) {
    commands.insert_resource(MenuFonts::new(&assets, &loc.locale, loc.font_small.clone()));
    commands.insert_resource(Menus::new(dir.map(|d| d.0.clone())));
}

/// The menu input of this fixed step, from the latched pending input.
pub fn menu_input(pending: &PendingInput, cams: &Cameras) -> MenuInput {
    let screen = pending.cursor_override.or(pending.cursor);
    let g = screen.map(|p| cams.screen_to_gui(p)).unwrap_or_default();
    MenuInput {
        mx: if screen.is_some() { g.x } else { -1e9 },
        my: if screen.is_some() { g.y } else { -1e9 },
        click: pending.lmb.pressed,
        rclick: pending.rmb.pressed,
        lmb_down: pending.lmb.down,
        escape: pending.keys_pressed.contains(&Key::Escape),
        enter: pending.keys_pressed.contains(&Key::Enter),
        backspace: pending.keys_pressed.contains(&Key::Backspace) as u32,
        text: pending.text.clone(),
        gw: cams.layout.view_w,
        gh: cams.layout.view_h,
    }
}

fn volumes(audio: Option<&Audio>) -> [f64; 2] {
    audio.map_or([1.0, 1.0], |a| {
        [
            a.audio_get_volume(VolumeKind::Sfx),
            a.audio_get_volume(VolumeKind::Music),
        ]
    })
}

/// The title as `game_restart` brings it back: a fresh world, obj_hud recreated, game_init's
/// flags reset (`units_imperial` survives, as a global that game_init only creates once).
pub fn restart_to_title(sim: &mut Sim, menus: &mut Menus, flags: &mut UiFlags) {
    let mut fresh = Sim::title_screen(seed_from_env());
    fresh.save_dir = menus.save_dir.clone();
    *sim = fresh;
    *menus = Menus::new(menus.save_dir.clone());
    *flags = UiFlags {
        units_imperial: flags.units_imperial,
        ..UiFlags::default()
    };
}

#[allow(clippy::too_many_arguments)]
pub fn menu_step(
    mut sim: ResMut<SimRes>,
    mut menus: ResMut<Menus>,
    mut flags: ResMut<UiFlags>,
    mut tree: ResMut<TreeScreen>,
    mut pending: ResMut<PendingInput>,
    mut ui: (ResMut<UiBlockers>, ResMut<UiCapture>),
    mut loc: ResMut<LocaleRes>,
    cams: Res<Cameras>,
    display: Res<Display>,
    sizes: Option<Res<DisplaySizes>>,
    audio: Option<Res<Audio>>,
    dir: Option<Res<SaveDir>>,
    assets: Res<GameAssets>,
    mut commands: Commands,
    mut writers: (
        MessageWriter<AppExit>,
        MessageWriter<SetFullscreen>,
        MessageWriter<CycleWindowSize>,
        MessageWriter<AudioRequest>,
    ),
    mut logo_hook: Local<bool>,
) {
    let m = &mut *menus;
    // the tree's LOGO button (Task 19): logo_open()
    if tree.logo_requested {
        tree.logo_requested = false;
        m.logo.logo_open(&mut m.keyboard_string);
    }
    // MYCO_LOGO=<text> (checks/screenshots): once in game, open the logo generator and grow it
    if !*logo_hook && !sim.0.title_open {
        *logo_hook = true;
        if let Ok(text) = std::env::var("MYCO_LOGO") {
            m.logo.logo_open(&mut m.keyboard_string);
            m.keyboard_string = text;
            m.logo.logo_read_typing(&mut m.keyboard_string);
            m.logo.logo_grow_start();
        }
    }
    let inp = menu_input(&pending, &cams);
    let window_size = sizes.map_or((display.0.window_w, display.0.window_h), |s| {
        s.display_window_size()
    });
    flags.menu_open = tree.open;
    let effects = {
        let mut env = MenuEnv {
            locale: &loc.locale,
            volumes: volumes(audio.as_deref()),
            fullscreen: display.0.fullscreen,
            window_size,
            flags: &mut flags,
        };
        m.step(&mut sim.0, &inp, &mut env)
    };
    if !flags.menu_open && tree.open {
        tree.close();
    }
    for fx in effects {
        match fx {
            MenuEffect::Restart => {
                restart_to_title(&mut sim.0, &mut menus, &mut flags);
                tree.close();
            }
            MenuEffect::Quit => {
                writers.0.write(AppExit::Success);
            }
            MenuEffect::SetFullscreen(on) => {
                writers.1.write(SetFullscreen(on));
            }
            MenuEffect::CycleWindowSize(d) => {
                writers.2.write(CycleWindowSize(d));
            }
            MenuEffect::CycleLanguage(d) => {
                // locale_cycle: switch, persist [game] language, reload the fonts
                let lang = loc.locale.cycle(d);
                if let Some(dir) = &dir {
                    if let Err(e) = locale::write_language(&dir.0, lang.code()) {
                        warn!("settings.ini: {e}");
                    }
                    *loc = LocaleRes::load(&assets, &dir.0);
                }
                commands.insert_resource(MenuFonts::new(
                    &assets,
                    &loc.locale,
                    loc.font_small.clone(),
                ));
            }
            MenuEffect::SetVolume(i, v) => {
                let kind = if i == 0 {
                    VolumeKind::Sfx
                } else {
                    VolumeKind::Music
                };
                writers.3.write(AudioRequest::SetVolume(kind, v));
            }
            MenuEffect::AudioSave => {
                writers.3.write(AudioRequest::Save);
            }
        }
    }
    pending.debug_open = flags.debug_open;
    pending.hud_hidden = flags.hud_hidden;
    pending.logo_open = menus.logo.open;
    pending.slot_naming = menus.slot_naming >= 0;
    // the tree adds its own flag and screen after this (it runs next)
    let menus_up = sim.0.title_open || sim.0.pause_open || menus.logo.open || flags.debug_open;
    // the tree screen writes its own blocker (`UiBlockers::tree`)
    ui.0.menus = menu::ui_blocking(&sim.0, &flags, menus.logo.open);
    ui.1.menus = menus_up || sim.0.intro_active;
}

#[allow(clippy::too_many_arguments)]
fn menu_draw(
    sim: Res<SimRes>,
    mut menus: ResMut<Menus>,
    mut flags: ResMut<UiFlags>,
    loc: Option<Res<LocaleRes>>,
    fonts: Option<Res<MenuFonts>>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    display: Res<Display>,
    sizes: Option<Res<DisplaySizes>>,
    audio: Option<Res<Audio>>,
    time: Res<Time<Real>>,
    mut draw: ResMut<GuiDraw>,
) {
    let (Some(fonts), Some(loc)) = (fonts, loc) else {
        return;
    };
    let window_size = sizes.map_or((display.0.window_w, display.0.window_h), |s| {
        s.display_window_size()
    });
    let env = MenuEnv {
        locale: &loc.locale,
        volumes: volumes(audio.as_deref()),
        fullscreen: display.0.fullscreen,
        window_size,
        flags: &mut flags,
    };
    let metrics = GmMetrics {
        assets: &assets,
        fonts: &fonts,
    };
    let cmds = menus.draw(
        &sim.0,
        &metrics,
        &env,
        cams.layout.view_w,
        cams.layout.view_h,
        time.elapsed_secs_f64() * 1000.0,
    );
    let mut gm = gm_draw::Gm::new(&assets);
    draw::paint(&cmds, &mut gm, &fonts);
    draw.lists.insert(MENU_DRAW_ORDER, gm.items);
}

/// obj_game Other_3 (Game End): save the bound slot once, whatever ends the app.
pub fn save_on_exit(
    mut exits: MessageReader<AppExit>,
    mut sim: ResMut<SimRes>,
    mut done: ResMut<ExitSaved>,
) {
    if exits.read().count() == 0 || done.0 {
        return;
    }
    done.0 = true;
    sim.0.game_end();
}
