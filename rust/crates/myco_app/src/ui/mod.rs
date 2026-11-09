//! GUI screens. Task 19: the GameMaker-style GUI painter ([`gm_draw`]) and the upgrade-tree
//! screen ([`tree`]); [`LocaleRes`] holds the language strings and the UI font.
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
pub mod draw;
pub mod gm_draw;
pub mod logo_screen;
pub mod menu;
pub mod name;
pub mod pause;
pub mod settings;
pub mod slots;
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

pub fn init_locale(mut commands: Commands, assets: Res<GameAssets>, dir: Option<Res<SaveDir>>) {
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
                menu_step
                    .in_set(SimSet::Prepare)
                    .after(run_script)
                    .before(tree::tree_step_system),
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
