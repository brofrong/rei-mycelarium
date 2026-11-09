//! Task 17's logic side of the HUD: locale extras, tasks panel toggles and the tasks popup.

use myco_core::locale::{Lang, Locale};
use myco_core::progress::tasks::{self, Block, PopupButton};
use myco_core::progress::{self as progress, tutorial, upgrades};
use myco_core::sim::{ButtonState, Command, Key, Sim, StepInput, ViewRect};
use myco_core::state::GameMode;

fn fresh() -> Sim {
    let mut sim = Sim::new_game(GameMode::Exploration, "Hud", None);
    upgrades::init(&mut sim);
    tutorial::init(&mut sim);
    tasks::init(&mut sim);
    sim
}

fn view() -> ViewRect {
    ViewRect {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1080.0,
    }
}

const BASE: &str = "key,context,en,ru,de\nhud_cm,,cm,см,cm\nhint_no_nodes_in_water,,No nodes in water,Нельзя в воде,Nicht im Wasser\nslot_label,,SLOT,СЛОТ,PLATZ\n";
const EXTRA: &str = "key,en,ru,de\ntip_node_too_close,Too close to a node,Слишком близко к узлу,\nhint_no_nodes_in_water,OVERRIDE,OVERRIDE,OVERRIDE\n";

#[test]
fn locale_extend_adds_missing_keys_with_english_fallback() {
    let mut l = Locale::load(BASE);
    l.extend_from_csv(EXTRA);
    assert_eq!(l.loc("tip_node_too_close"), "Too close to a node");
    // existing keys are not overridden by the extras
    assert_eq!(l.loc("hint_no_nodes_in_water"), "No nodes in water");
    l.set_language(Lang::Ru);
    assert_eq!(l.loc("tip_node_too_close"), "Слишком близко к узлу");
    assert_eq!(l.loc("hint_no_nodes_in_water"), "Нельзя в воде");
    // an empty cell falls back to English
    l.set_language(Lang::De);
    assert_eq!(l.loc("tip_node_too_close"), "Too close to a node");
    // the language list does not change
    assert_eq!(l.langs(), &[Lang::En, Lang::De, Lang::Ru]);
}

#[test]
fn locale_extend_applies_to_the_current_language() {
    let mut l = Locale::load(BASE);
    l.set_language(Lang::Ru);
    l.extend_from_csv(EXTRA);
    assert_eq!(l.loc("tip_node_too_close"), "Слишком близко к узлу");
}

#[test]
fn toggle_task_block_command_flips_saved_state() {
    let mut sim = fresh();
    assert!(tasks::block_open(&sim.state, Block::Tasks));
    let input = StepInput {
        view: view(),
        commands: vec![Command::ToggleTaskBlock {
            block: Block::Tasks,
        }],
        ..Default::default()
    };
    progress::hud_begin_step(&mut sim, &input);
    assert!(!tasks::block_open(&sim.state, Block::Tasks));
    assert!(!sim.state.state.tasks.open);
    let input = StepInput {
        view: view(),
        commands: vec![Command::ToggleTaskBlock {
            block: Block::Magic,
        }],
        ..Default::default()
    };
    progress::hud_begin_step(&mut sim, &input);
    assert!(!tasks::block_open(&sim.state, Block::Magic));
    assert!(tasks::block_open(&sim.state, Block::Migration));
    // the flag is saved
    let (save, _) = sim.to_save();
    assert_eq!(
        save.state.tasks.goal_open.get("magic"),
        Some(&serde_json::Value::Bool(false))
    );
}

#[test]
fn tasks_popup_geometry_matches_lua() {
    let mut sim = fresh();
    sim.state.state.tasks.popup = "tasks".into();
    assert!(tasks::tasks_popup_open(&sim));
    let r = tasks::tasks_popup_rect(&sim, 1920.0, 1080.0);
    // w 1300, h 640: y1 = 540 - 740 / 2
    assert_eq!(
        (r.x1, r.y1, r.x2, r.y2, r.cx),
        (310.0, 170.0, 1610.0, 810.0, 960.0)
    );
    let b = tasks::tasks_popup_buttons(&sim, 1920.0, 1080.0);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0].id, PopupButton::Close);
    assert_eq!(b[0].label_key, "goal_button_grow");
    assert_eq!(
        (b[0].x1, b[0].y1, b[0].x2, b[0].y2),
        (750.0, 838.0, 1170.0, 902.0)
    );
    sim.state.state.tasks.popup = "migration_ready".into();
    let b = tasks::tasks_popup_buttons(&sim, 1920.0, 1080.0);
    assert_eq!(b[0].id, PopupButton::Spores);
    assert_eq!(b[0].label_key, "goal_button_spores");
    sim.state.state.tasks.popup = "migration_end".into();
    let b = tasks::tasks_popup_buttons(&sim, 1920.0, 1080.0);
    assert_eq!(b[0].label_key, "goal_button_continue");
    assert_eq!(b[0].y1, 1080.0 - 64.0 - 72.0);
    // survival has no popups
    sim.state.state.mode = GameMode::Survival;
    assert!(!tasks::tasks_popup_open(&sim));
}

#[test]
fn tasks_popup_closes_on_button_click_in_begin_step() {
    let mut sim = fresh();
    sim.state.state.tasks.popup = "tasks".into();
    // a click outside the button does nothing
    let miss = StepInput {
        view: view(),
        cursor_gui: Some((100.0, 100.0)),
        lmb: ButtonState {
            down: true,
            pressed: true,
            released: false,
        },
        ..Default::default()
    };
    progress::hud_begin_step(&mut sim, &miss);
    assert_eq!(sim.state.state.tasks.popup, "tasks");
    let hit = StepInput {
        cursor_gui: Some((960.0, 870.0)),
        ..miss.clone()
    };
    progress::hud_begin_step(&mut sim, &hit);
    assert_eq!(sim.state.state.tasks.popup, "");
}

#[test]
fn tasks_popup_closes_on_enter_or_escape() {
    let mut sim = fresh();
    for key in [Key::Enter, Key::Escape] {
        sim.state.state.tasks.popup = "magic".into();
        let input = StepInput {
            view: view(),
            keys_pressed: vec![key],
            ..Default::default()
        };
        progress::hud_begin_step(&mut sim, &input);
        assert_eq!(sim.state.state.tasks.popup, "");
    }
}
