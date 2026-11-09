//! Simulation skeleton: frame counter, order of the per-frame subsystem calls, resource
//! accounting, game mode start, save/load plumbing.

use myco_core::balance::{bal, wld};
use myco_core::events::SimEvent;
use myco_core::sim::{Command, LoadError, Sim, StepInput};
use myco_core::state::{GameMode, GameState, MushroomSave};
use serde_json::Value;

/// One frame in Room1 order: Begin Step of all instances (only obj_hud has one), then Step of
/// obj_game (game_step, save_step), obj_forest_controller, obj_mycelium_controller, obj_life;
/// no instance has an End Step.
const FRAME: [&str; 6] = [
    "hud.begin_step",
    "game.step",
    "save.step",
    "forest.step",
    "mycelium.step",
    "life.step",
];

#[test]
fn step_advances_frame_and_calls_in_order() {
    let mut sim = Sim::new_game(GameMode::Exploration, "Test", Some(31607));
    assert_eq!(sim.frame, 0);
    sim.trace = Some(Vec::new());
    let input = StepInput::default();
    sim.step(&input);
    sim.step(&input);
    sim.step(&input);
    assert_eq!(sim.frame, 3);
    let log = sim.trace.take().unwrap();
    let want: Vec<&str> = FRAME
        .iter()
        .cycle()
        .take(FRAME.len() * 3)
        .copied()
        .collect();
    assert_eq!(log, want);
}

#[test]
fn new_game_starts_the_mode() {
    let e = Sim::new_game(GameMode::Exploration, "Ex", Some(5));
    assert_eq!(e.state.state.mode, GameMode::Exploration);
    assert_eq!(e.state.state.food, 50.0);
    assert_eq!(e.state.state.water_free, 80.0);
    assert_eq!(e.world_seed, Some(5));
    assert_eq!(e.save.name, "Ex");
    assert_eq!(e.save.slot, -1);
    assert!(!e.title_open);
    let s = Sim::new_game(GameMode::Survival, "Su", None);
    assert_eq!(s.state.state.mode, GameMode::Survival);
    assert_eq!(s.state.state.food, bal(GameMode::Survival, "start_food"));
    assert_eq!(s.state.state.food, 70.0);
    assert_eq!(s.state.state.water_free, 100.0);
    assert_eq!(s.bal("base_node_food_cost"), 8.0);
}

#[test]
fn food_and_water_accounting() {
    let mut sim = Sim::new_game(GameMode::Exploration, "R", Some(1));
    sim.food_add(5.0);
    assert_eq!(sim.state.state.food, 55.0);
    assert!(!sim.food_spend(56.0));
    assert_eq!(sim.state.state.food, 55.0);
    assert!(sim.food_spend(55.0));
    assert_eq!(sim.state.state.food, 0.0);
    sim.food_add(-3.0);
    assert_eq!(sim.state.state.food, -3.0);
    assert_eq!(sim.water_total(), 80.0);
    sim.state.state.water_infinite = true;
    assert_eq!(sim.water_total(), 1_000_000.0);
}

#[test]
fn events_are_drained_once() {
    let mut sim = Sim::new_game(GameMode::Exploration, "E", Some(1));
    sim.emit(SimEvent::GameOver);
    sim.emit(SimEvent::TutorialTip {
        key: "tut_node".into(),
    });
    assert_eq!(
        sim.drain_events(),
        vec![
            SimEvent::GameOver,
            SimEvent::TutorialTip {
                key: "tut_node".into()
            }
        ]
    );
    assert!(sim.drain_events().is_empty());
}

#[test]
fn pause_command_and_save_timer() {
    let mut sim = Sim::new_game(GameMode::Exploration, "P", Some(1));
    sim.save_start_new(0, "P");
    assert_eq!(sim.save.timer, 120.0);
    let mut input = StepInput::default();
    input.commands.push(Command::SetPaused(true));
    sim.step(&input);
    assert!(sim.pause_open);
    // paused: save_step does not count down
    assert_eq!(sim.save.timer, 120.0);
    input.commands = vec![Command::SetPaused(false)];
    sim.step(&input);
    assert!(!sim.pause_open);
    assert_eq!(sim.save.timer, 119.0);
}

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("myco_sim_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn autosave_writes_the_slot_when_the_timer_runs_out() {
    let dir = temp_dir("autosave");
    let mut sim = Sim::new_game(GameMode::Survival, "Auto", Some(1));
    sim.save_dir = Some(dir.clone());
    sim.save_start_new(1, "Auto");
    sim.intro_active = false;
    sim.save.timer = 1.0;
    sim.step(&StepInput::default());
    assert!(sim.drain_events().contains(&SimEvent::Saved));
    assert_eq!(
        sim.save.timer,
        bal(GameMode::Survival, "save_autosave_seconds") * 60.0
    );
    let st = myco_core::save::read_slot(&dir, 1).unwrap();
    assert_eq!(st.name, "Auto");
    assert_eq!(st.state.mode, GameMode::Survival);
    let meta = myco_core::save::read_meta(&dir, 1).unwrap();
    assert_eq!(meta.name, "Auto");
    assert!(meta.saved > 40000.0);
    let loaded = Sim::load_game(&dir, 1).unwrap();
    assert_eq!(loaded.save.slot, 1);
    assert_eq!(loaded.save.name, "Auto");
    assert_eq!(loaded.state.state.mode, GameMode::Survival);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn save_game_respects_guards() {
    let dir = temp_dir("guards");
    let mut sim = Sim::new_game(GameMode::Exploration, "G", Some(1));
    sim.save_dir = Some(dir.clone());
    assert!(!sim.save_game(), "no slot yet");
    sim.save_start_new(0, "G");
    sim.intro_active = true;
    assert!(!sim.save_game(), "intro running");
    sim.intro_active = false;
    assert!(sim.save_game());
    // Game End (obj_game Other_3) saves too
    sim.game_end();
    assert!(dir.join("save_1.json").exists());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn to_save_applies_save_game_transforms() {
    let mut sim = Sim::new_game(GameMode::Exploration, "T", Some(1));
    sim.state.state.length_px = 3200.0;
    let carried = MushroomSave {
        kind: 0,
        carried_by: 3,
        y: 900.0,
        stench: vec![Value::from(1)],
        crow_target: true,
        ..MushroomSave::default()
    };
    let stolen = MushroomSave {
        kind: 1,
        stolen: true,
        ..MushroomSave::default()
    };
    sim.state.mushrooms = vec![carried, stolen];
    let (gs, meta) = sim.to_save();
    assert_eq!(gs.name, "T");
    assert_eq!(gs.mushrooms.len(), 1);
    let m = &gs.mushrooms[0];
    assert_eq!(
        (m.carried_by, m.y, m.crow_target, m.stench.len()),
        (-1, wld("surface_height_px"), false, 0)
    );
    assert_eq!(meta.length_cm, 1000.0);
    assert_eq!(meta.mode, GameMode::Exploration);
    assert_eq!(meta.name, "T");
}

#[test]
fn from_save_applies_load_game_fixups() {
    let mut gs = GameState {
        name: "L".into(),
        ..GameState::default()
    };
    gs.state.mode = GameMode::Survival;
    gs.state.tasks.popup = "migration".into();
    gs.mushrooms = vec![MushroomSave {
        stolen: true,
        carried_by: 2,
        stench_due: 9.0,
        born: Some(3.0),
        ..MushroomSave::default()
    }];
    let sim = Sim::from_save(gs.clone()).unwrap();
    assert_eq!(sim.state.state.tasks.popup, "migration_ready");
    let m = &sim.state.mushrooms[0];
    assert_eq!(
        (m.stolen, m.carried_by, m.stench_due, m.born),
        (false, -1, 0.0, None)
    );
    assert!(!sim.title_open && !sim.intro_active);
    assert_eq!(sim.save.name, "L");
    assert_eq!(
        sim.save.timer,
        bal(GameMode::Survival, "save_autosave_seconds") * 60.0
    );
    gs.state.tasks.popup = "migration_end".into();
    assert_eq!(
        Sim::from_save(gs.clone()).unwrap().state.state.tasks.popup,
        ""
    );
    gs.version = 2;
    assert!(matches!(Sim::from_save(gs), Err(LoadError::Version(2))));
}

const CREATE: [&str; 4] = [
    "game.init",
    "forest.create",
    "mycelium.create",
    "life.create",
];

#[test]
fn new_game_runs_the_create_sequence_first() {
    let sim = Sim::new_game(GameMode::Exploration, "C", Some(1));
    let mut want = CREATE.to_vec();
    want.extend(["save_start_new", "game_mode_start"]);
    assert_eq!(sim.build_log, want);
}

#[test]
fn from_save_runs_create_hooks_before_on_load() {
    let sim = Sim::from_save(GameState::default()).unwrap();
    // load_game runs on a live room: the same Create sequence as a new game, then the loads
    let mut want = CREATE.to_vec();
    want.extend([
        "world.on_load",
        "progress.on_load",
        "forest.on_load",
        "mycelium.on_load",
        "mushrooms.on_load",
        "life.on_load",
        "weather.init",
    ]);
    assert_eq!(sim.build_log, want);
}

#[test]
fn missing_boar_timer_falls_back_per_mode() {
    let parse = |mode: &str, timers: &str| -> GameState {
        let text = format!(
            r#"{{"version":1,"world":{{}},"state":{{"mode":"{mode}","timers":{timers}}}}}"#
        );
        serde_json::from_str(&text).unwrap()
    };
    let t = r#"{"hedgehog":1,"squirrel":2,"crow":3,"experience_crows":[4,5],"leaf":6,"worm":7}"#;
    let ex = Sim::from_save(parse("exploration", t)).unwrap();
    assert_eq!(ex.state.state.timers.boar, Some(30600.0));
    let su = Sim::from_save(parse("survival", t)).unwrap();
    assert_eq!(su.state.state.timers.boar, Some(18000.0));
    assert_eq!(
        su.state.state.timers.boar,
        Some(bal(GameMode::Survival, "boar_spawn_interval") * 60.0)
    );
    // a saved value is kept and round-trips; an unset one is not written
    let kept = parse("survival", r#"{"boar":123.5}"#);
    assert_eq!(
        Sim::from_save(kept.clone())
            .unwrap()
            .state
            .state
            .timers
            .boar,
        Some(123.5)
    );
    assert!(serde_json::to_string(&kept.state.timers)
        .unwrap()
        .contains("\"boar\":123.5"));
    let unset = parse("survival", t);
    assert!(!serde_json::to_string(&unset.state.timers)
        .unwrap()
        .contains("boar"));
}
