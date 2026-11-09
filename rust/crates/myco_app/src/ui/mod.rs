//! Menus: main menu, slots, naming, settings/pause, credits, the logo generator, and saving on
//! quit. See `menu.rs` for the state machine; this module wires it into the app.
//!
//! - Fixed step, `SimSet::Prepare` after the autopilot: [`menu_step`] reads the latched input,
//!   runs the menus (which may start, load or restart the game), applies their effects, then
//!   fills `PendingInput.{menu,logo,debug}_open/hud_hidden`, `ui_captured` and [`UiBlocking`].
//! - `Update`: [`menu_draw`] builds [`UiDrawList`]; `PostUpdate` renders it on the GUI camera.
//! - `MYCO_LOGO=<text>` opens the logo generator with that text once a game runs (checks).
//! - `Last`: [`save_on_exit`] runs obj_game Other_3 (`Sim::game_end`, a save) once when the app
//!   exits for any reason (Quit/Exit item, Cmd+Esc, closing the window).

pub mod credits;
pub mod draw;
pub mod logo_screen;
pub mod menu;
pub mod name;
pub mod pause;
pub mod settings;
pub mod slots;

use crate::assets::GameAssets;
use crate::audio::{Audio, AudioRequest, VolumeKind};
use crate::autopilot::{run_script, seed_from_env};
use crate::camera::Cameras;
use crate::display::{CycleWindowSize, Display, DisplaySizes, SaveDir, SetFullscreen};
use crate::sim_bridge::{PendingInput, SimRes, SimSet, UiBlocking};
use bevy::prelude::*;
use draw::{UiDrawList, UiFonts};
use menu::{MenuEffect, MenuEnv, MenuInput, Menus, UiFlags};
use myco_core::locale::{self, Locale};
use myco_core::sim::{Key, Sim};

/// The game's strings in the current language (`global.locale_*`).
#[derive(Resource, Clone, Debug)]
pub struct UiLocale(pub Locale);

/// `locale_font_ink_h(Font1, "HEXN")`, the reference for fitting TTF fonts.
#[derive(Resource, Clone, Copy, Debug)]
pub struct Font1Ink(pub Option<f64>);

/// obj_game Other_3 already ran.
#[derive(Resource, Default)]
pub struct ExitSaved(pub bool);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiFlags>()
            .init_resource::<UiDrawList>()
            .init_resource::<draw::DrawnList>()
            .init_resource::<ExitSaved>()
            .add_systems(Startup, init_menus)
            .add_systems(
                FixedUpdate,
                menu_step.in_set(SimSet::Prepare).after(run_script),
            )
            .add_systems(Update, menu_draw)
            .add_systems(
                PostUpdate,
                // new entities must get their transform and visibility in the same frame, or the
                // menu blinks out for a frame whenever it changes
                draw::render_draw_list
                    .before(bevy::transform::TransformSystems::Propagate)
                    .before(bevy::camera::visibility::VisibilitySystems::VisibilityPropagate),
            )
            .add_systems(Last, save_on_exit);
    }
}

/// `LANG=de_DE.UTF-8` → ("de", "DE") for `locale_system`.
fn os_language() -> (String, String) {
    let lang = std::env::var("LANG").unwrap_or_default();
    let base = lang.split('.').next().unwrap_or("");
    let mut it = base.split(['_', '-']);
    (
        it.next().unwrap_or("").to_string(),
        it.next().unwrap_or("").to_string(),
    )
}

fn init_menus(mut commands: Commands, assets: Res<GameAssets>, dir: Option<Res<SaveDir>>) {
    let csv = assets.read_text("localization.csv").unwrap_or_default();
    let saved = dir
        .as_ref()
        .map(|d| locale::read_language(&d.0))
        .unwrap_or_default();
    let (os_lang, os_region) = os_language();
    let loc = Locale::init(&csv, &saved, &os_lang, &os_region);
    let ink = draw::gm_font_ink(&assets, "Font1", "HEXN");
    commands.insert_resource(UiFonts::build(&assets, &loc, ink));
    commands.insert_resource(Font1Ink(ink));
    commands.insert_resource(UiLocale(loc));
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
fn menu_step(
    mut sim: ResMut<SimRes>,
    mut menus: ResMut<Menus>,
    mut flags: ResMut<UiFlags>,
    mut pending: ResMut<PendingInput>,
    mut blocking: ResMut<UiBlocking>,
    mut loc: ResMut<UiLocale>,
    cams: Res<Cameras>,
    display: Res<Display>,
    sizes: Option<Res<DisplaySizes>>,
    audio: Option<Res<Audio>>,
    dir: Option<Res<SaveDir>>,
    assets: Res<GameAssets>,
    ink: Res<Font1Ink>,
    mut commands: Commands,
    mut writers: (
        MessageWriter<AppExit>,
        MessageWriter<SetFullscreen>,
        MessageWriter<CycleWindowSize>,
        MessageWriter<AudioRequest>,
    ),
    mut logo_hook: Local<bool>,
) {
    let inp = menu_input(&pending, &cams);
    // MYCO_LOGO=<text> (checks/screenshots): once in game, open the logo generator and grow it
    if !*logo_hook && !sim.0.title_open {
        *logo_hook = true;
        if let Ok(text) = std::env::var("MYCO_LOGO") {
            let m = &mut *menus;
            m.logo.logo_open(&mut m.keyboard_string);
            m.keyboard_string = text;
            m.logo.logo_read_typing(&mut m.keyboard_string);
            m.logo.logo_grow_start();
        }
    }
    let window_size = sizes.map_or(
        (display.0.window_w, display.0.window_h),
        |s| s.display_window_size(),
    );
    let effects = {
        let mut env = MenuEnv {
            locale: &loc.0,
            volumes: volumes(audio.as_deref()),
            fullscreen: display.0.fullscreen,
            window_size,
            flags: &mut flags,
        };
        menus.step(&mut sim.0, &inp, &mut env)
    };
    for fx in effects {
        match fx {
            MenuEffect::Restart => restart_to_title(&mut sim.0, &mut menus, &mut flags),
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
                let lang = loc.0.cycle(d);
                if let Some(dir) = &dir {
                    if let Err(e) = locale::write_language(&dir.0, lang.code()) {
                        warn!("settings.ini: {e}");
                    }
                }
                commands.insert_resource(UiFonts::build(&assets, &loc.0, ink.0));
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
    pending.menu_open = flags.menu_open;
    pending.debug_open = flags.debug_open;
    pending.hud_hidden = flags.hud_hidden;
    pending.logo_open = menus.logo.open;
    let block = menu::ui_blocking(&sim.0, &flags, menus.logo.open);
    blocking.0 = block;
    if block {
        pending.ui_captured = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn menu_draw(
    sim: Res<SimRes>,
    mut menus: ResMut<Menus>,
    mut flags: ResMut<UiFlags>,
    loc: Res<UiLocale>,
    fonts: Option<Res<UiFonts>>,
    cams: Res<Cameras>,
    display: Res<Display>,
    sizes: Option<Res<DisplaySizes>>,
    audio: Option<Res<Audio>>,
    time: Res<Time<Real>>,
    mut list: ResMut<UiDrawList>,
) {
    let Some(fonts) = fonts else { return };
    let window_size = sizes.map_or(
        (display.0.window_w, display.0.window_h),
        |s| s.display_window_size(),
    );
    let env = MenuEnv {
        locale: &loc.0,
        volumes: volumes(audio.as_deref()),
        fullscreen: display.0.fullscreen,
        window_size,
        flags: &mut flags,
    };
    let cmds = menus.draw(
        &sim.0,
        &*fonts,
        &env,
        cams.layout.view_w,
        cams.layout.view_h,
        time.elapsed_secs_f64() * 1000.0,
    );
    if list.0 != cmds {
        list.0 = cmds;
    }
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
