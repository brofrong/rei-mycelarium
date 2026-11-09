//! Task 17: floating numbers, the right-click tip, right-click routing and the HUD's Begin Step.

use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::ui::cursor_tip::{tip_text, CursorTip, TIP_LIFE};
use myco_app::ui::hud::{step_game, step_pre, HudState, StepIn, UiFonts};
use myco_app::ui::markers::AnimalMarkers;
use myco_app::ui::popups::{Popups, CAP};
use myco_app::ui::LOCALE_EXTRA_CSV;
use myco_core::events::{NodeReject, ResourceKind, SimEvent};
use myco_core::locale::{Lang, Locale};
use myco_core::mushrooms::MushroomKind;
use myco_core::progress::tasks::Block;
use myco_core::sim::{Command, Key, Sim, ViewRect};
use myco_core::state::GameMode;
use std::path::{Path, PathBuf};

fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets")
}

fn assets() -> Option<GameAssets> {
    let root = assets_dir();
    if !root.join("sprites.json").exists() {
        eprintln!("skipped: no build/assets");
        return None;
    }
    Some(
        GameAssets::load(
            &root,
            |_| Handle::default(),
            |_| Handle::default(),
            |_| Handle::default(),
        )
        .unwrap(),
    )
}

fn view() -> ViewRect {
    ViewRect {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1080.0,
    }
}

fn res(kind: ResourceKind, delta: f64, x: f64, y: f64, source: u64) -> SimEvent {
    SimEvent::Resource {
        kind,
        delta,
        x,
        y,
        source,
    }
}

#[test]
fn popups_aggregate_per_source_per_step() {
    let mut p = Popups::default();
    let step = [
        res(ResourceKind::Food, 2.0, 100.0, 100.0, 1),
        res(ResourceKind::Food, 3.0, 110.0, 100.0, 1),
        res(ResourceKind::Food, 1.0, 500.0, 300.0, 2),
        res(ResourceKind::Water, 4.0, 100.0, 100.0, 1),
    ];
    p.add_step(step.iter(), &view());
    assert_eq!(p.list.len(), 3);
    let f1 = &p.list[0];
    assert_eq!(
        (f1.kind, f1.value, f1.x, f1.y),
        (ResourceKind::Food, 5.0, 100.0, 100.0)
    );
    assert_eq!(p.list[1].value, 1.0);
    assert_eq!(
        (p.list[2].kind, p.list[2].value),
        (ResourceKind::Water, 4.0)
    );
    // the next step makes a new number for the same source
    p.add_step(
        [res(ResourceKind::Food, 2.0, 100.0, 100.0, 1)].iter(),
        &view(),
    );
    assert_eq!(p.list.len(), 4);
    // a source whose deltas cancel out makes none
    p.add_step(
        [
            res(ResourceKind::Food, 2.0, 100.0, 100.0, 9),
            res(ResourceKind::Food, -2.0, 100.0, 100.0, 9),
        ]
        .iter(),
        &view(),
    );
    assert_eq!(p.list.len(), 4);
    assert_eq!(p.list[0].label(), "+5");
    p.list[0].value = -3.0;
    assert_eq!(p.list[0].label(), "-3");
}

#[test]
fn popups_cap_64_merges_into_nearest() {
    let mut p = Popups::default();
    // a storm: 100 mushrooms produce in one step
    let storm: Vec<SimEvent> = (0..100)
        .map(|i| res(ResourceKind::Food, 1.0, 10.0 + i as f64 * 10.0, 500.0, i))
        .collect();
    p.add_step(storm.iter(), &view());
    assert_eq!(p.list.len(), CAP);
    let total: f64 = p.list.iter().map(|q| q.value).sum();
    assert_eq!(total, 100.0, "nothing is lost, extra numbers are merged");
    // the 65th source (x = 650) was added to its nearest live number (x = 640)
    let near = p.list.iter().find(|q| q.x == 640.0).unwrap();
    assert!(near.value >= 2.0);
    // full: a new food number merges into the nearest food number
    p.add_step(
        [res(ResourceKind::Food, 7.0, 333.0, 500.0, 500)].iter(),
        &view(),
    );
    assert_eq!(p.list.len(), CAP);
    let at330 = p.list.iter().find(|q| q.x == 330.0).unwrap();
    assert_eq!(at330.value, 8.0);
    // no live water number to merge into: dropped, still 64
    p.add_step(
        [res(ResourceKind::Water, 5.0, 333.0, 500.0, 501)].iter(),
        &view(),
    );
    assert_eq!(p.list.len(), CAP);
    assert!(p.list.iter().all(|q| q.kind == ResourceKind::Food));
}

#[test]
fn popups_skip_offscreen() {
    let mut p = Popups::default();
    let v = ViewRect {
        x: 1000.0,
        y: 200.0,
        w: 1920.0,
        h: 1080.0,
    };
    p.add_step(
        [
            res(ResourceKind::Food, 1.0, 999.0, 500.0, 1),
            res(ResourceKind::Food, 1.0, 1500.0, 1281.0, 2),
            res(ResourceKind::Water, 1.0, 1500.0, 199.0, 3),
            res(ResourceKind::Water, 2.0, 2920.0, 1280.0, 4),
        ]
        .iter(),
        &v,
    );
    assert_eq!(p.list.len(), 1);
    assert_eq!(p.list[0].value, 2.0);
}

#[test]
fn popup_rises_and_fades() {
    let mut p = Popups::default();
    p.add_step(
        [res(ResourceKind::Food, 3.0, 100.0, 100.0, 1)].iter(),
        &view(),
    );
    assert_eq!((p.list[0].rise(), p.list[0].alpha()), (0.0, 1.0));
    for _ in 0..30 {
        p.tick(1.0 / 60.0);
    }
    let q = &p.list[0];
    assert!((q.rise() - 30.0).abs() < 1e-6, "rise {}", q.rise());
    assert!((q.alpha() - 0.5).abs() <= 0.05, "alpha {}", q.alpha());
    for _ in 0..31 {
        p.tick(1.0 / 60.0);
    }
    assert!(p.list.is_empty(), "gone after 1 s");
}

fn sim() -> Sim {
    let mut s = Sim::new_game(GameMode::Exploration, "Hud", None);
    s.title_open = false;
    s.intro_active = false;
    s
}

fn input(mx: f64, my: f64) -> StepIn {
    StepIn {
        mx,
        my,
        world: Some((mx + 1000.0, my + 50.0)),
        click: false,
        rclick: false,
        lmb_down: false,
        keys: vec![],
        gw: 1920.0,
        gh: 1080.0,
    }
}

#[test]
fn rmb_in_placement_cancels_not_places() {
    let s = sim();
    let mut hud = HudState::new();
    hud.place_type = Some(MushroomKind::Spore);
    hud.build_open = true;
    hud.ui_capture_mouse = true; // placement mode captures the mouse (Begin Step)
    let inp = StepIn {
        rclick: true,
        ..input(900.0, 500.0)
    };
    let out = step_game(&mut hud, &s, &inp);
    assert_eq!(hud.place_type, None);
    assert!(!hud.build_open);
    assert!(hud.right_click_used);
    assert!(
        out.commands.is_empty(),
        "no node, no mushroom: {:?}",
        out.commands
    );
    // only the bar open (no type picked): also just closes
    hud.build_open = true;
    hud.ui_capture_mouse = false;
    let out = step_game(&mut hud, &s, &inp);
    assert!(!hud.build_open);
    assert!(out.commands.is_empty());
}

#[test]
fn rmb_over_hud_does_nothing() {
    let Some(assets) = assets() else {
        return;
    };
    let s = sim();
    let loc = Locale::load("key,en\n");
    let uf = UiFonts::bitmap();
    let markers = AnimalMarkers::default();
    let mut hud = HudState::new();
    let step = |hud: &mut HudState, inp: StepIn| {
        let mut cmds = step_pre(hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &inp);
        cmds.extend(step_game(hud, &s, &inp).commands);
        cmds
    };
    // over the build button (the round button bottom-left)
    let (bx, by) = (hud.build.cx, hud.build.cy);
    let cmds = step(
        &mut hud,
        StepIn {
            rclick: true,
            ..input(bx, by)
        },
    );
    assert!(hud.ui_capture_mouse);
    assert!(cmds.is_empty(), "{cmds:?}");
    // over the upgrades button
    let r = hud.upgrades_button_rect(&assets);
    let cmds = step(
        &mut hud,
        StepIn {
            rclick: true,
            ..input((r.x1 + r.x2) / 2.0, (r.y1 + r.y2) / 2.0)
        },
    );
    assert!(cmds.is_empty(), "{cmds:?}");
    // over a camera button
    let cx = hud.cam_x(1) + 40.0;
    let cy = hud.camnav.y + 40.0;
    let cmds = step(
        &mut hud,
        StepIn {
            rclick: true,
            ..input(cx, cy)
        },
    );
    assert!(cmds.is_empty(), "{cmds:?}");
    // in the world: a node right under the cursor
    let cmds = step(
        &mut hud,
        StepIn {
            rclick: true,
            ..input(900.0, 500.0)
        },
    );
    assert_eq!(
        cmds,
        vec![Command::QuickNode {
            x: 1900.0,
            y: 550.0
        }]
    );
    // while aiming (left button held): nothing
    let cmds = step(
        &mut hud,
        StepIn {
            rclick: true,
            lmb_down: true,
            ..input(900.0, 500.0)
        },
    );
    assert!(cmds.is_empty());
    // in the pause menu: nothing
    let mut paused = sim();
    paused.pause_open = true;
    let inp = StepIn {
        rclick: true,
        ..input(900.0, 500.0)
    };
    let mut cmds = step_pre(
        &mut hud, &paused, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &inp,
    );
    cmds.extend(step_game(&mut hud, &paused, &inp).commands);
    assert!(cmds.is_empty(), "{cmds:?}");
}

/// One Escape chain (Task 18's `Menus::step`): with the build bar or placement up, an Escape
/// only closes it (through `UiFlags.build_close_requested`); the next one pauses.
#[test]
fn escape_closes_placement_before_pausing() {
    use myco_app::ui::menu::{MenuEnv, MenuInput, Menus, UiFlags};
    let Some(assets) = assets() else {
        return;
    };
    let mut s = sim();
    let loc = Locale::load("key,en\n");
    let uf = UiFonts::bitmap();
    let markers = AnimalMarkers::default();
    let mut hud = HudState::new();
    let mut menus = Menus::new(None);
    let mut flags = UiFlags::default();
    hud.place_type = Some(MushroomKind::Spore);
    let mut escape_step = |s: &mut Sim, hud: &mut HudState, flags: &mut UiFlags| {
        // the HUD reports its state each step, the menus run first, then the HUD
        flags.build_open = hud.build_open || hud.place_type.is_some();
        let inp = MenuInput {
            mx: 900.0,
            my: 500.0,
            click: false,
            rclick: false,
            lmb_down: false,
            escape: true,
            enter: false,
            backspace: 0,
            text: String::new(),
            gw: 1920.0,
            gh: 1080.0,
        };
        let mut env = MenuEnv {
            locale: &loc,
            volumes: [1.0, 1.0],
            fullscreen: false,
            window_size: (1280, 720),
            flags,
        };
        menus.step(s, &inp, &mut env);
        let close = std::mem::take(&mut flags.build_close_requested);
        let esc = StepIn {
            keys: vec![Key::Escape],
            ..input(900.0, 500.0)
        };
        step_pre(hud, s, &assets, &uf, &loc, &markers, 0.0, 0.0, close, &esc)
    };
    let cmds = escape_step(&mut s, &mut hud, &mut flags);
    assert!(cmds.is_empty());
    assert_eq!(hud.place_type, None);
    assert!(!s.pause_open, "the same Escape must not also pause");
    let cmds = escape_step(&mut s, &mut hud, &mut flags);
    assert!(
        cmds.is_empty(),
        "the HUD sends no pause of its own: {cmds:?}"
    );
    assert!(s.pause_open, "the second Escape pauses (menus)");
}

#[test]
fn tasks_header_click_toggles_block() {
    let Some(assets) = assets() else {
        return;
    };
    let mut s = sim();
    s.state.state.tasks.shown = true;
    let loc = Locale::load("key,en\ntasks_title,TASKS\n");
    let uf = UiFonts::bitmap();
    let markers = AnimalMarkers::default();
    let mut hud = HudState::new();
    let x = myco_core::balance::vis("tasks_x") + 4.0;
    let y = myco_core::balance::vis("tasks_y");
    let inp = StepIn {
        click: true,
        ..input(x, y)
    };
    let cmds = step_pre(
        &mut hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &inp,
    );
    assert_eq!(
        cmds,
        vec![Command::ToggleTaskBlock {
            block: Block::Tasks
        }]
    );
    assert!(hud.ui_capture_mouse, "the header takes the mouse");
    // and a right click on it places nothing
    let out = step_game(
        &mut hud,
        &s,
        &StepIn {
            rclick: true,
            ..input(x, y)
        },
    );
    assert!(out.commands.is_empty());
}

#[test]
fn upgrade_button_opens_the_tree() {
    let Some(assets) = assets() else {
        return;
    };
    let s = sim();
    let loc = Locale::load("key,en\n");
    let uf = UiFonts::bitmap();
    let markers = AnimalMarkers::default();
    let mut hud = HudState::new();
    hud.build_open = true;
    let r = hud.upgrades_button_rect(&assets);
    let inp = StepIn {
        click: true,
        ..input((r.x1 + r.x2) / 2.0, (r.y1 + r.y2) / 2.0)
    };
    step_pre(
        &mut hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &inp,
    );
    let out = step_game(&mut hud, &s, &inp);
    assert!(out.opened_tree);
    assert!(!hud.build_open);
    assert_eq!(hud.upgrade_btn_press, hud.ui.press_frames);
}

#[test]
fn layout_follows_ui_offset() {
    let mut hud = HudState::new();
    let (lx, cx, by) = (hud.ui.length_x, hud.camnav.left_x, hud.build.cy);
    hud.layout_apply(100.0, 40.0);
    assert_eq!(hud.ui.length_x, lx + 100.0);
    assert_eq!(
        hud.camnav.left_x,
        cx + 200.0,
        "camera buttons move by twice the offset"
    );
    assert_eq!(hud.build.cy, by + 40.0);
    hud.layout_apply(100.0, 40.0);
    assert_eq!(hud.ui.length_x, lx + 100.0);
}

#[test]
fn cursor_tip_shows_reason_for_1_2_seconds() {
    let csv = "key,en,ru\nhint_no_nodes_in_water,NO NODES IN WATER,НЕЛЬЗЯ\nx,a,b\n";
    let mut loc = Locale::load(csv);
    loc.extend_from_csv(LOCALE_EXTRA_CSV);
    let mut tip = CursorTip::default();
    tip.add_step(
        [SimEvent::NodeRejected {
            reason: NodeReject::NotEnoughFood { need: 12 },
            x: 0.0,
            y: 0.0,
        }]
        .iter(),
    );
    let r = tip.reason.clone().unwrap();
    assert_eq!(tip_text(&loc, &r), "NEED 12 FOOD");
    assert_eq!(tip.alpha(), 1.0);
    assert_eq!(tip_text(&loc, &NodeReject::InWater), "NO NODES IN WATER");
    loc.set_language(Lang::Ru);
    assert_eq!(tip_text(&loc, &r), "НУЖНО 12 ЕДЫ");
    tip.tick(1.0);
    assert!(tip.reason.is_some() && tip.alpha() > 0.0 && tip.alpha() < 1.0);
    tip.tick(TIP_LIFE - 1.0);
    assert!(tip.reason.is_none());
}

#[test]
fn locale_extra_covers_12_languages() {
    let Some(assets) = assets() else {
        return;
    };
    let csv = assets.read_text("localization.csv").unwrap();
    let mut loc = Locale::load(&csv);
    loc.extend_from_csv(LOCALE_EXTRA_CSV);
    let keys = [
        "tip_node_need_food",
        "tip_node_not_on_mycelium",
        "tip_node_too_close",
        "tip_node_blocked",
        "hint_no_nodes_in_water",
    ];
    assert_eq!(loc.langs().len(), 12);
    let en: Vec<String> = keys.iter().map(|k| loc.loc(k).to_string()).collect();
    for lang in Lang::ALL {
        loc.set_language(lang);
        for (i, k) in keys.iter().enumerate() {
            let s = loc.loc(k);
            assert_ne!(s, *k, "{k} missing for {lang:?}");
            if lang != Lang::En {
                assert_ne!(s, en[i], "{k} not translated for {lang:?}");
            }
        }
        assert!(loc.loc("tip_node_need_food").contains("{n}"));
    }
}

#[test]
fn tasks_fill_advances_once_per_step() {
    use myco_app::ui::tasks::tasks_fill_step;
    let mut s = sim();
    s.state.state.tasks.shown = true;
    s.state
        .state
        .tasks
        .done
        .insert("absorption".into(), serde_json::Value::Bool(true));
    s.progress.tasks.fill.insert("absorption".into(), 0.0);
    let hud = HudState::new();
    let loc = Locale::load("key,en\n");
    let step = 1.0 / myco_core::balance::vis("tasks_fill_frames").max(1.0);
    tasks_fill_step(&mut s, &hud, &loc);
    tasks_fill_step(&mut s, &hud, &loc);
    let f = s.progress.tasks.fill["absorption"];
    assert!((f - 2.0 * step).abs() < 1e-12, "fill {f}");
    // hidden HUD: tasks_draw does not run, the fill stays
    let hidden = HudState {
        hud_hidden: true,
        ..HudState::new()
    };
    tasks_fill_step(&mut s, &hidden, &loc);
    assert!((s.progress.tasks.fill["absorption"] - f).abs() < 1e-12);
}

/// Ruling R22: in placement mode the HUD's step fills `StepInput.placement`; the core runs the
/// placement check from it and stores the preview the Draw GUI shows.
#[test]
fn placement_mode_fills_step_input_and_core_preview() {
    use myco_app::camera::Cameras;
    use myco_app::sim_bridge::{PendingInput, SimRes, UiCapture};
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(SimRes(sim()))
        .insert_resource(HudState::default())
        .init_resource::<Cameras>()
        .init_resource::<PendingInput>()
        .init_resource::<UiCapture>()
        .add_systems(Update, myco_app::ui::hud::hud_step_game);
    app.update();
    assert_eq!(app.world().resource::<PendingInput>().placement, None);
    app.world_mut().resource_mut::<HudState>().place_type = Some(MushroomKind::Spore);
    app.update();
    let cams = Cameras::default();
    let input = {
        let mut p = app.world_mut().resource_mut::<PendingInput>();
        p.cursor = Some(Vec2::new(900.0, 500.0));
        p.take_step(&cams)
    };
    assert_eq!(input.placement, Some(MushroomKind::Spore));
    let mut s = app.world_mut().resource_mut::<SimRes>();
    s.0.step(&input);
    let pv = s.0.mushrooms.place_preview.as_ref().expect("core preview");
    assert_eq!(pv.kind, MushroomKind::Spore);
}

/// `camera_fly_to_marker` keeps the marker itself: the fly's target follows the marker while
/// the camera flies (Task 21).
#[test]
fn camera_fly_follows_the_clicked_marker() {
    use myco_app::ui::markers::Marker;
    use myco_core::camera::CameraFly;
    let Some(assets) = assets() else {
        return;
    };
    let mut s = sim();
    let loc = Locale::load("key,en\n");
    let uf = UiFonts::bitmap();
    let mut markers = AnimalMarkers::default();
    markers.list.push(Marker {
        src: 77,
        sprite: "boar",
        frames: 6.0,
        frame: 0.0,
        mirror: 1.0,
        x: 3000.0,
        y: 500.0,
        w: 50.0,
        h: 40.0,
        warn: false,
        alpha: 1.0,
        vx: 0.0,
        vy: 0.0,
        gone: false,
        seen: true,
        follow: 0.0,
        ui: Some((1800.0, 540.0)),
    });
    let mut hud = HudState::new();
    let click = StepIn {
        click: true,
        ..input(1800.0, 540.0)
    };
    let cmds = step_pre(
        &mut hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &click,
    );
    assert!(
        cmds.contains(&Command::CameraFlyTo {
            x: 3000.0,
            y: 500.0
        }),
        "{cmds:?}"
    );
    // the core started the fly; the boar walks on
    s.camera.fly = Some(CameraFly {
        t: 0.0,
        x: 3000.0,
        y: 500.0,
    });
    markers.list[0].x = 3100.0;
    let idle = input(600.0, 540.0);
    let cmds = step_pre(
        &mut hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &idle,
    );
    assert!(
        cmds.contains(&Command::CameraFlyTrack {
            x: 3100.0,
            y: 500.0
        }),
        "{cmds:?}"
    );
    // the fly ended: no more tracking
    s.camera.fly = None;
    let cmds = step_pre(
        &mut hud, &s, &assets, &uf, &loc, &markers, 0.0, 0.0, false, &idle,
    );
    assert!(!cmds
        .iter()
        .any(|c| matches!(c, Command::CameraFlyTrack { .. })));
    assert_eq!(hud.fly_marker, None);
}
