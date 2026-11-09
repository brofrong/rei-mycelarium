//! Main menu, slots, naming, settings/pause and save-on-quit (Task 18).

use bevy::prelude::*;
use myco_app::sim_bridge::SimRes;
use myco_app::ui::draw::{DrawCmd, FixedMetrics};
use myco_app::ui::menu::*;
use myco_app::ui::settings::hud_pause_button_rect;
use myco_app::ui::slots::hud_slot_button_rect;
use myco_app::ui::{save_on_exit, ExitSaved};
use myco_core::locale::Locale;
use myco_core::save::{self, SlotStatus};
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use std::path::{Path, PathBuf};

const GW: f64 = 1920.0;
const GH: f64 = 1200.0;

fn locale() -> Locale {
    let p = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../build/assets/localization.csv"
    );
    let csv = std::fs::read_to_string(p).unwrap_or_default();
    Locale::init(&csv, "en", "en", "US")
}

fn temp_dir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("myco-t18-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// A saved game in `slot` named `name`.
fn write_game(dir: &Path, slot: i64, name: &str) {
    let mut sim = Sim::new_game(GameMode::Exploration, name, Some(7));
    sim.save_dir = Some(dir.to_path_buf());
    sim.save_start_new(slot, name);
    assert!(sim.save_game());
}

fn title(dir: &Path) -> Sim {
    let mut sim = Sim::title_screen(Some(7));
    sim.save_dir = Some(dir.to_path_buf());
    sim
}

struct Ctx {
    loc: Locale,
    flags: UiFlags,
}

impl Ctx {
    fn new() -> Ctx {
        Ctx {
            loc: locale(),
            flags: UiFlags::default(),
        }
    }
    fn step(&mut self, m: &mut Menus, sim: &mut Sim, inp: MenuInput) -> Vec<MenuEffect> {
        let mut env = MenuEnv {
            locale: &self.loc,
            volumes: [0.5, 0.5],
            fullscreen: false,
            window_size: (1280, 720),
            flags: &mut self.flags,
        };
        m.step(sim, &inp, &mut env)
    }
    fn draw(&mut self, m: &mut Menus, sim: &Sim) -> Vec<DrawCmd> {
        let env = MenuEnv {
            locale: &self.loc,
            volumes: [0.5, 0.5],
            fullscreen: false,
            window_size: (1280, 720),
            flags: &mut self.flags,
        };
        m.draw(sim, &FixedMetrics { w: 20.0, h: 40.0 }, &env, GW, GH, 0.0)
    }
}

fn click_at(r: R) -> MenuInput {
    MenuInput {
        mx: r.cx(),
        my: r.cy(),
        click: true,
        lmb_down: true,
        gw: GW,
        gh: GH,
        ..Default::default()
    }
}

fn hover_at(r: R) -> MenuInput {
    MenuInput {
        click: false,
        lmb_down: false,
        ..click_at(r)
    }
}

fn idle() -> MenuInput {
    MenuInput {
        mx: -1e9,
        my: -1e9,
        gw: GW,
        gh: GH,
        ..Default::default()
    }
}

fn texts(d: &[DrawCmd]) -> Vec<String> {
    d.iter()
        .filter_map(|c| match c {
            DrawCmd::Text { s, .. } => Some(s.clone()),
            _ => None,
        })
        .collect()
}

fn title_item(sim: &Sim, item: &str) -> R {
    let i = TITLE_ITEMS.iter().position(|t| *t == item).unwrap();
    hud_title_button_rect(GW, sim.camera.view_y, i)
}

/// Review Focus 3: a slot whose save is broken is listed with a mark, and clicking it in the
/// load screen neither loads nor breaks anything; a good slot next to it still loads.
#[test]
fn corrupt_slot_listed_not_loadable() {
    let dir = temp_dir("corrupt");
    write_game(&dir, 0, "Good");
    write_game(&dir, 1, "Broken");
    // truncate slot 2's save (its meta stays readable)
    let p = save::save_path(&dir, 1);
    let text = std::fs::read(&p).unwrap();
    std::fs::write(&p, &text[..text.len() / 2]).unwrap();
    assert!(matches!(
        save::list_slots(&dir, 3)[1],
        SlotStatus::Corrupt(_)
    ));

    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    assert!(m.saves_exist);
    let r = title_item(&sim, "load_game");
    cx.step(&mut m, &mut sim, click_at(r));
    assert_eq!(m.slot_mode, SlotMode::Load);

    let shown = texts(&cx.draw(&mut m, &sim));
    assert!(
        shown
            .iter()
            .any(|s| s.starts_with("Slot 2") && s.contains("Damaged")),
        "{shown:?}"
    );
    assert!(
        shown
            .iter()
            .any(|s| s.starts_with("Slot 1") && s.contains("Good")),
        "{shown:?}"
    );
    assert!(
        shown
            .iter()
            .any(|s| s.starts_with("Slot 3") && s.contains("Empty")),
        "{shown:?}"
    );

    let r1 = hud_slot_button_rect(GW, sim.camera.view_y, 1);
    cx.step(&mut m, &mut sim, hover_at(r1));
    assert_eq!(m.slot_hover, -1, "a damaged slot does not highlight");
    cx.step(&mut m, &mut sim, click_at(r1));
    assert_eq!(m.slot_mode, SlotMode::Load, "still on the load screen");
    assert!(sim.title_open, "nothing was loaded");
    assert_eq!(sim.save.slot, -1);

    let r0 = hud_slot_button_rect(GW, sim.camera.view_y, 0);
    cx.step(&mut m, &mut sim, click_at(r0));
    assert!(!sim.title_open, "the good slot loads");
    assert_eq!((sim.save.slot, sim.save.name.as_str()), (0, "Good"));
    assert_eq!(m.slot_mode, SlotMode::None);
}

/// "Load Game" is disabled without saves; a damaged meta does not count as a save.
#[test]
fn load_disabled_without_readable_saves() {
    let dir = temp_dir("nosaves");
    std::fs::write(save::meta_path(&dir, 0), b"{not json").unwrap();
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    assert!(!m.saves_exist);
    let r = title_item(&sim, "load_game");
    cx.step(&mut m, &mut sim, click_at(r));
    assert_eq!(m.slot_mode, SlotMode::None);
    assert_eq!(m.title_hover, -1);
}

/// Exploration → slot → name → the game starts in that slot with that name; the title fades.
#[test]
fn new_game_flow_binds_slot_and_name() {
    let dir = temp_dir("newgame");
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    let r = title_item(&sim, "survival");
    cx.step(&mut m, &mut sim, click_at(r));
    assert_eq!(
        (m.slot_mode, m.new_mode),
        (SlotMode::New, GameMode::Survival)
    );
    let r2 = hud_slot_button_rect(GW, sim.camera.view_y, 2);
    cx.step(&mut m, &mut sim, click_at(r2));
    assert_eq!(m.slot_naming, 2);
    cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            text: "Spore cap and much longer".into(),
            ..idle()
        },
    );
    assert_eq!(
        m.keyboard_string, "Spore cap and mu",
        "16 characters at most"
    );
    cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            backspace: 1,
            ..idle()
        },
    );
    cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            enter: true,
            ..idle()
        },
    );
    assert!(!sim.title_open);
    assert_eq!(
        (sim.save.slot, sim.save.name.as_str()),
        (2, "Spore cap and m")
    );
    assert_eq!(sim.state.state.mode, GameMode::Survival);
    assert_eq!(m.slot_mode, SlotMode::None);
    for _ in 0..31 {
        cx.step(&mut m, &mut sim, idle());
    }
    assert_eq!(m.title_alpha, 0.0, "menus faded out in about half a second");
}

/// An empty name becomes "Mycelium N"; Escape backs out of naming, then of the slots.
#[test]
fn empty_name_defaults_and_escape_backs_out() {
    let dir = temp_dir("names");
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    let r = title_item(&sim, "exploration");
    cx.step(&mut m, &mut sim, click_at(r));
    let r = hud_slot_button_rect(GW, sim.camera.view_y, 0);
    cx.step(&mut m, &mut sim, click_at(r));
    let esc = MenuInput {
        escape: true,
        ..idle()
    };
    cx.step(&mut m, &mut sim, esc.clone());
    assert_eq!((m.slot_mode, m.slot_naming), (SlotMode::New, -1));
    cx.step(&mut m, &mut sim, esc.clone());
    assert_eq!(m.slot_mode, SlotMode::None);
    cx.step(&mut m, &mut sim, esc);
    assert!(!sim.pause_open, "Escape does not pause on the title");

    let r = title_item(&sim, "exploration");

    cx.step(&mut m, &mut sim, click_at(r));
    let r = hud_slot_button_rect(GW, sim.camera.view_y, 1);
    cx.step(&mut m, &mut sim, click_at(r));
    cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            text: "   ".into(),
            enter: true,
            ..idle()
        },
    );
    assert_eq!(sim.save.name, "Mycelium 2");
}

/// The title's Settings is the pause screen without "main menu"; the pause in game has it, and
/// "main menu" saves the game and asks for a restart; Quit asks to end the game.
#[test]
fn settings_and_pause_items() {
    let dir = temp_dir("pause");
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    let r = title_item(&sim, "settings");
    cx.step(&mut m, &mut sim, click_at(r));
    assert!(sim.pause_open);
    let shown = texts(&cx.draw(&mut m, &sim));
    assert!(shown.iter().any(|s| s == "LANGUAGE: ENGLISH"), "{shown:?}");
    assert!(shown.iter().any(|s| s == "WINDOW: 1280x720"), "{shown:?}");
    assert!(!shown.iter().any(|s| s == "MAIN MENU"));
    let lang = PAUSE_ITEMS_TITLE
        .iter()
        .position(|i| *i == "language")
        .unwrap();
    let fx = cx.step(
        &mut m,
        &mut sim,
        click_at(hud_pause_button_rect(GW, GH, lang)),
    );
    assert_eq!(fx, vec![MenuEffect::CycleLanguage(1)]);
    let units = PAUSE_ITEMS_TITLE
        .iter()
        .position(|i| *i == "units")
        .unwrap();
    cx.step(
        &mut m,
        &mut sim,
        click_at(hud_pause_button_rect(GW, GH, units)),
    );
    assert!(cx.flags.units_imperial);
    cx.step(&mut m, &mut sim, click_at(hud_pause_button_rect(GW, GH, 0)));
    assert!(!sim.pause_open, "resume");

    // in game: Escape pauses, "main menu" saves and restarts
    sim.start_from_title(0, "Run", GameMode::Exploration);
    cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            escape: true,
            ..idle()
        },
    );
    assert!(sim.pause_open);
    let mm = PAUSE_ITEMS_GAME
        .iter()
        .position(|i| *i == "main_menu")
        .unwrap();
    let fx = cx.step(
        &mut m,
        &mut sim,
        click_at(hud_pause_button_rect(GW, GH, mm)),
    );
    assert_eq!(fx, vec![MenuEffect::AudioSave, MenuEffect::Restart]);
    assert!(matches!(&save::list_slots(&dir, 3)[0], SlotStatus::Ok(meta) if meta.name == "Run"));
    let quit = PAUSE_ITEMS_GAME.iter().position(|i| *i == "quit").unwrap();
    let fx = cx.step(
        &mut m,
        &mut sim,
        click_at(hud_pause_button_rect(GW, GH, quit)),
    );
    assert_eq!(fx, vec![MenuEffect::Quit]);

    // volume slider: drag sets, release saves
    let r = myco_app::ui::settings::hud_volume_slider_rect(GW, GH, 1);
    let fx = cx.step(
        &mut m,
        &mut sim,
        MenuInput {
            mx: r.x1 + (r.x2 - r.x1) * 0.25,
            ..click_at(r)
        },
    );
    assert_eq!(fx, vec![MenuEffect::SetVolume(1, 0.25)]);
    let fx = cx.step(&mut m, &mut sim, idle());
    assert_eq!(fx, vec![MenuEffect::AudioSave]);
}

/// obj_game Other_3: whatever ends the app (Quit, Cmd+Esc, closing the window) saves the bound
/// slot once.
#[test]
fn quit_writes_slot() {
    let dir = temp_dir("quit");
    let mut sim = title(&dir);
    sim.start_from_title(1, "Leaving", GameMode::Exploration);
    let mut app = App::new();
    app.add_message::<AppExit>()
        .insert_resource(SimRes(sim))
        .init_resource::<ExitSaved>()
        .add_systems(Last, save_on_exit);
    app.update();
    assert!(
        !save::save_path(&dir, 1).exists(),
        "no save before quitting"
    );
    app.world_mut().write_message(AppExit::Success);
    app.update();
    let slots = save::list_slots(&dir, 3);
    assert!(
        matches!(&slots[1], SlotStatus::Ok(m) if m.name == "Leaving"),
        "{slots:?}"
    );
    assert!(app.world().resource::<ExitSaved>().0);
    // on the title nothing is bound: quitting writes nothing
    let dir2 = temp_dir("quit-title");
    let mut app = App::new();
    app.add_message::<AppExit>()
        .insert_resource(SimRes(title(&dir2)))
        .init_resource::<ExitSaved>()
        .add_systems(Last, save_on_exit);
    app.world_mut().write_message(AppExit::Success);
    app.update();
    assert!(std::fs::read_dir(&dir2).unwrap().next().is_none());
}

/// Credits open from the title and any click closes them.
#[test]
fn credits_open_and_close() {
    let dir = temp_dir("credits");
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    let mut cx = Ctx::new();
    let r = title_item(&sim, "credits");
    cx.step(&mut m, &mut sim, click_at(r));
    assert!(m.credits_open);
    let shown = texts(&cx.draw(&mut m, &sim));
    assert!(shown.iter().any(|s| s == "Eder Muniz"));
    cx.step(
        &mut m,
        &mut sim,
        click_at(R {
            x1: 5.0,
            y1: 5.0,
            x2: 6.0,
            y2: 6.0,
        }),
    );
    assert!(!m.credits_open);
    let shown = texts(&cx.draw(&mut m, &sim));
    assert!(shown.iter().any(|s| s == "v1.02") && shown.iter().any(|s| s == "Exploration Mode"));
}

/// `display_init` / `display_cycle_size` as LÖVE reports a 14" MacBook (1512×982 points): the
/// settings offer 1280×720 and 1366×768; no saved size selects the largest; cycling wraps.
#[test]
fn display_sizes_follow_love() {
    use myco_app::display::DisplaySizes;
    let mut s = DisplaySizes::new(Some((1512.0, 982.0)), None);
    assert_eq!(s.sizes, vec![(1280, 720), (1366, 768)]);
    assert_eq!(s.display_window_size(), (1366, 768));
    assert_eq!(s.display_cycle_size(1), (1280, 720));
    assert_eq!(s.display_cycle_size(-1), (1366, 768));
    let s = DisplaySizes::new(Some((1512.0, 982.0)), Some((1280, 720)));
    assert_eq!(s.display_window_size(), (1280, 720));
    let s = DisplaySizes::new(Some((800.0, 600.0)), Some((0, 0)));
    assert_eq!(s.sizes, vec![(1280, 720)]);
}
