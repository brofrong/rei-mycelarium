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
    // past the intro (game_init starts it; save_game refuses while it runs)
    sim.intro_active = false;
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
    sim.intro_active = false; // past the intro (save_game refuses while it runs)
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
    sim.intro_active = false; // past the intro (save_game refuses while it runs)
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

/// Every way out reaches obj_game Other_3 through `AppExit` and saves exactly once: Cmd+Q
/// (the app menu's Quit, the Dock's Quit: `applicationShouldTerminate:` -> `request_quit`),
/// Cmd+Q as a key, Cmd+Esc, closing the primary window; two of them in a row still save once.
#[test]
fn every_quit_path_saves_once() {
    use bevy::window::{exit_on_primary_closed, PrimaryWindow};
    use myco_app::display;
    let dir = temp_dir("quit-paths");
    let mut sim = title(&dir);
    sim.start_from_title(1, "Leaving", GameMode::Exploration);
    sim.intro_active = false;
    let mut app = App::new();
    app.add_message::<AppExit>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(SimRes(sim))
        .init_resource::<ExitSaved>()
        .add_systems(
            Update,
            (display::display_quit_requests, display::display_keys),
        )
        .add_systems(PostUpdate, exit_on_primary_closed)
        .add_systems(Last, save_on_exit);
    let win = app
        .world_mut()
        .spawn((Window::default(), PrimaryWindow))
        .id();
    app.update();
    let path = save::save_path(&dir, 1);
    assert!(!path.exists(), "no save before quitting");
    // the app menu's Quit (Cmd+Q): the first request is turned into AppExit, not a terminate
    assert!(!display::request_quit(), "first Quit is not forced");
    app.update();
    assert!(path.exists(), "Cmd+Q saved");
    assert!(app.world().resource::<ExitSaved>().0);
    // later exits (Cmd+Esc, Cmd+Q as a key, closing the window) do not save again
    std::fs::remove_file(&path).unwrap();
    {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.press(KeyCode::SuperLeft);
        keys.press(KeyCode::Escape);
        keys.press(KeyCode::KeyQ);
    }
    app.world_mut().despawn(win);
    app.update();
    let exits = app
        .world()
        .resource::<Messages<AppExit>>()
        .iter_current_update_messages()
        .count();
    assert!(exits >= 2, "Cmd+Esc/Cmd+Q/close each ask to exit ({exits})");
    assert!(!path.exists(), "saved only once");
    // a second Quit while the first is still pending forces the terminate (a stuck app quits)
    assert!(display::request_quit());
}

/// Closing the window and Cmd+Esc each save on their own (fresh app each).
#[test]
fn window_close_and_cmd_esc_save() {
    use bevy::window::{exit_on_primary_closed, PrimaryWindow};
    use myco_app::display;
    for (tag, close) in [("close", true), ("cmdesc", false)] {
        let dir = temp_dir(tag);
        let mut sim = title(&dir);
        sim.start_from_title(1, "Leaving", GameMode::Exploration);
        sim.intro_active = false;
        let mut app = App::new();
        app.add_message::<AppExit>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(SimRes(sim))
            .init_resource::<ExitSaved>()
            .add_systems(Update, display::display_keys)
            .add_systems(PostUpdate, exit_on_primary_closed)
            .add_systems(Last, save_on_exit);
        let win = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.update();
        if close {
            app.world_mut().despawn(win);
        } else {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::SuperRight);
            keys.press(KeyCode::Escape);
        }
        app.update();
        assert!(save::save_path(&dir, 1).exists(), "{tag} saved");
    }
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
    assert!(shown.iter().any(|s| s == VERSION) && shown.iter().any(|s| s == "Exploration Mode"));
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

/// "Main menu" restarts like game_restart (gm/instance.lua): the old game runs the rest of its
/// step, the fresh title replaces it at the end of the step, and the title's first step is the
/// next one (found by the T22 conformance run: Rust stepped the new title in the restart step).
#[test]
fn main_menu_restart_happens_after_the_step() {
    use bevy::time::TimeUpdateStrategy;
    use myco_app::camera::Cameras;
    use myco_app::sim_bridge::{SimBridgePlugin, SimRes, SimSet, StepHistory};
    use myco_app::ui::menu::{Menus, UiFlags};
    use myco_app::ui::tree::TreeScreen;
    use myco_app::ui::{apply_restart, RestartPending};
    use std::time::Duration;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::input::InputPlugin, SimBridgePlugin));
    let mut game =
        myco_core::sim::Sim::new_game(myco_core::state::GameMode::Exploration, "R", Some(7));
    game.title_open = false;
    game.intro_active = false;
    app.insert_resource(SimRes(game))
        .insert_resource(Cameras::default())
        .insert_resource(StepHistory::default())
        .insert_resource(Menus::new(None))
        .insert_resource(UiFlags::default())
        .insert_resource(TreeScreen::default())
        .insert_resource(RestartPending(false))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
        .add_systems(
            FixedUpdate,
            (
                (|mut r: ResMut<RestartPending>| r.0 = true).in_set(SimSet::Prepare),
                apply_restart
                    .in_set(SimSet::Step)
                    .after(myco_app::sim_bridge::step_sim),
            ),
        );
    app.update();
    let old_session = app.world().resource::<SimRes>().0.session;
    let step = app.world().resource::<Time<Fixed>>().timestep();
    app.insert_resource(TimeUpdateStrategy::ManualDuration(step));
    app.update();
    assert_eq!(app.world().resource::<StepHistory>().0.len(), 1, "one step");
    let sim = &app.world().resource::<SimRes>().0;
    assert_ne!(sim.session, old_session, "replaced by a fresh title");
    assert!(sim.title_open);
    assert_eq!(
        sim.frame, 1,
        "the old game took the step; the clock goes on"
    );
    // the new title has not stepped yet: its RNG is where a fresh title's is (draws since
    // game_init's reseed), not past a step's draws
    let draws = |s: &myco_core::sim::Sim| {
        let mut r = myco_core::rng::Rng::new(s.rng.seed());
        (0..100_000).find(|_| {
            let hit = r.state() == s.rng.state();
            r.next01();
            hit
        })
    };
    // a fresh room makes about a dozen draws (game_init, world_init, the start stump); the
    // title's first step makes ~160 more (the backdrop leaves)
    let n = draws(sim).expect("from the title's own seed");
    assert!(n < 60, "the new title has not stepped yet ({n} draws)");
}

/// `menu::menus_block` is the menus' share of `ui_blocking()`; with the HUD's share
/// (`hud::hud_blocks`: tasks popup, blocking cut-scene) it is the whole `hud::ui_blocking`.
#[test]
fn menus_block_plus_hud_blocks_is_ui_blocking() {
    use myco_app::ui::hud::{hud_blocks, ui_blocking, HudState};
    let dir = temp_dir("blocking");
    let mut sim = title(&dir);
    sim.start_from_title(1, "Blocks", GameMode::Exploration);
    for bits in 0u32..128 {
        let b = |i: u32| bits & (1 << i) != 0;
        sim.intro_active = b(0);
        sim.title_open = b(1);
        sim.pause_open = b(2);
        sim.state.state.tasks.popup = if b(3) { "t".into() } else { String::new() };
        let flags = UiFlags {
            menu_open: b(4),
            debug_open: b(5),
            ..Default::default()
        };
        let logo = b(6);
        let mut hud = HudState::new();
        hud.menu_open = flags.menu_open;
        hud.debug_open = flags.debug_open;
        hud.logo_open = logo;
        assert_eq!(
            menus_block(&sim, &flags, logo) || hud_blocks(&sim),
            ui_blocking(&sim, &hud),
            "bits {bits:07b}"
        );
    }
}

/// Rust only: with `continue_enabled` the title starts with "Continue", which loads the slot
/// saved last; the start-up load does the same once, with the title already gone.
#[test]
fn continue_loads_the_latest_save() {
    let dir = temp_dir("continue");
    write_game(&dir, 0, "Old");
    write_game(&dir, 2, "New");
    // slot 1 ("Old") was saved long ago
    let mut meta = save::read_meta(&dir, 0).unwrap();
    meta.saved = 1.0;
    let mut bytes = serde_json::to_vec(&meta).unwrap();
    bytes.push(0);
    std::fs::write(save::meta_path(&dir, 0), bytes).unwrap();
    assert_eq!(latest_slot(&dir), Some(2));

    // off (tests, automated runs): the original's items
    let m = Menus::new(Some(dir.clone()));
    assert_eq!(m.title_items(), TITLE_ITEMS.to_vec());

    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    m.continue_enabled = true;
    let mut cx = Ctx::new();
    let items = m.title_items();
    assert_eq!(items[0], "continue");
    assert_eq!(&items[1..], &TITLE_ITEMS[..]);
    let shown = texts(&cx.draw(&mut m, &sim));
    // (this test's locale is the game's file alone; the app adds locale_extra.csv's "Continue")
    assert_eq!(shown[0], "title_continue", "{shown:?}");
    let r = hud_title_button_rect(GW, sim.camera.view_y, 0);
    cx.step(&mut m, &mut sim, click_at(r));
    assert!(!sim.title_open);
    assert_eq!((sim.save.slot, sim.save.name.as_str()), (2, "New"));

    // the start-up load
    let mut sim = title(&dir);
    let mut m = Menus::new(Some(dir.clone()));
    m.continue_enabled = true;
    m.autoload_pending = true;
    cx.step(&mut m, &mut sim, idle());
    assert!(!sim.title_open);
    assert_eq!(sim.save.slot, 2);
    assert_eq!(m.title_alpha, 0.0);
    assert!(!m.autoload_pending);

    // no saves: no item, the start-up load does nothing
    let empty = temp_dir("continue-empty");
    let mut sim = title(&empty);
    let mut m = Menus::new(Some(empty));
    m.continue_enabled = true;
    m.autoload_pending = true;
    assert_eq!(m.title_items(), TITLE_ITEMS.to_vec());
    cx.step(&mut m, &mut sim, idle());
    assert!(sim.title_open);
}

#[test]
fn version_is_the_rust_build() {
    assert_eq!(VERSION, "v1.0.3-RUST");
}
