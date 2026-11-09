//! Tasks panel logic (scr_game `tasks_*`): the first-stage task list with upgrade-point
//! rewards, then the goals stage (migration, magic). Saved state: `state.state.tasks`; the
//! panel fades live in [`TasksRt`]. Drawing and the text-measured header hit-test are the
//! app's (Task 17); a header click arrives as `Command::ToggleTaskBlock` (`tasks_block_toggle`).
//! The popup (`tasks_popup_*`) has no text in its geometry, so its input runs here.

use super::goals::{self, Goal};
use super::mycelium_cm;
use super::tutorial::{self, Tip, TipState};
use super::upgrades::upg_is_owned;
use crate::balance::{gml_truthy, vis};
use crate::gml;
use crate::camera;
use crate::sim::{Command, Key, Sim, StepInput};
use crate::state::{GameMode, GameState, TasksSave};
use serde_json::Value;
use std::collections::BTreeMap;

/// A first-stage task (`tasks_defs()`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskId {
    Absorption,
    ShotRange,
    Water,
    RainPoint,
    Spores,
    Food,
    Build,
    Hedgehog,
    Grow,
    Trees,
    Stinky,
}

const DEFS: [TaskId; 11] = [
    TaskId::Absorption,
    TaskId::ShotRange,
    TaskId::Water,
    TaskId::RainPoint,
    TaskId::Spores,
    TaskId::Food,
    TaskId::Build,
    TaskId::Hedgehog,
    TaskId::Grow,
    TaskId::Trees,
    TaskId::Stinky,
];

impl TaskId {
    /// The key in `tasks.done` and in the `task_<id>` locale key.
    pub fn as_str(self) -> &'static str {
        match self {
            TaskId::Absorption => "absorption",
            TaskId::ShotRange => "shot_range",
            TaskId::Water => "water",
            TaskId::RainPoint => "rain_point",
            TaskId::Spores => "spores",
            TaskId::Food => "food",
            TaskId::Build => "build",
            TaskId::Hedgehog => "hedgehog",
            TaskId::Grow => "grow",
            TaskId::Trees => "trees",
            TaskId::Stinky => "stinky",
        }
    }

    pub fn from_key(s: &str) -> Option<TaskId> {
        DEFS.into_iter().find(|t| t.as_str() == s)
    }
}

/// A block of the tasks panel (`tasks_block_open(id)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Tasks,
    Migration,
    Magic,
}

impl Block {
    pub const ALL: [Block; 3] = [Block::Tasks, Block::Migration, Block::Magic];

    pub fn as_str(self) -> &'static str {
        match self {
            Block::Tasks => "tasks",
            Block::Migration => "migration",
            Block::Magic => "magic",
        }
    }

    pub fn from_key(s: &str) -> Option<Block> {
        Block::ALL.into_iter().find(|b| b.as_str() == s)
    }
}

/// Runtime part of `global.tasks`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TasksRt {
    /// panel fade-in 0..1
    pub alpha: f64,
    /// per block id: fold 0 (closed) .. 1 (open); a missing id counts as 1
    pub fold_k: BTreeMap<String, f64>,
    /// per item key: the check-mark fill the panel animates (`tasks_block_draw`)
    pub fill: BTreeMap<String, f64>,
}

// port: tasks_defs
/// The first-stage tasks in panel order.
pub fn tasks_defs() -> &'static [TaskId] {
    &DEFS
}

// port: tasks_init
/// A fresh tasks state (stats at 0, panel open, nothing done, no popup).
pub fn init(sim: &mut Sim) {
    sim.state.state.tasks = TasksSave::default();
    sim.progress.tasks = TasksRt::default();
}

// port: tasks_stat_add
/// Add `n` to the task counter `key` (also the `ach_*` counters).
pub fn stat_add(sim: &mut Sim, key: &str, n: f64) {
    *sim.state
        .state
        .tasks
        .stats
        .entry(key.to_string())
        .or_insert(0.0) += n;
}

// port: tasks_met
/// Whether the task's condition holds now.
pub fn tasks_met(sim: &Sim, id: TaskId) -> bool {
    let stat = |k: &str| sim.state.state.tasks.stats.get(k).copied().unwrap_or(0.0);
    match id {
        TaskId::Absorption => upg_is_owned(&sim.state, "unlock_absorption"),
        TaskId::ShotRange => upg_is_owned(&sim.state, "shot_range"),
        TaskId::Water => stat("water") >= 1000.0,
        TaskId::RainPoint => stat("rain_points") >= 1.0,
        TaskId::Spores => stat("spores") >= 1.0,
        TaskId::Food => stat("food") >= 30.0,
        TaskId::Build => stat("built") >= 5.0,
        TaskId::Hedgehog => stat("hedgehogs") >= 1.0,
        TaskId::Grow => mycelium_cm(sim) >= 10000.0,
        TaskId::Trees => stat("tree_built") >= 2.0,
        TaskId::Stinky => stat("stinky_built") >= 2.0,
    }
}

// port: tasks_block_open
/// `tasks_block_open(id)`: the saved open flag of the block (goal blocks default to open).
pub fn block_open(st: &GameState, id: Block) -> bool {
    let t = &st.state.tasks;
    match id {
        Block::Tasks => t.open,
        _ => t
            .goal_open
            .get(id.as_str())
            .is_none_or(|v| gml_truthy(Some(v))),
    }
}

// port: tasks_step
/// Once the tutorial's final tip appeared: fade the panel in, animate the block folds, mark
/// met tasks done (each pays `task_reward_points`), then switch to the goals stage and raise
/// the goal popups. Not in survival.
pub fn step(sim: &mut Sim, input: &StepInput) {
    if sim.state.state.mode == GameMode::Survival {
        return;
    }
    if sim.title_open || sim.intro_active || sim.pause_open || input.logo_open {
        return;
    }
    if !sim.state.state.tasks.shown && tutorial::tutorial_state(sim, Tip::Final) != TipState::Hidden
    {
        sim.state.state.tasks.shown = true;
    }
    if !sim.state.state.tasks.shown {
        return;
    }
    let rt = &mut sim.progress.tasks;
    rt.alpha = (rt.alpha + 1.0 / vis("tasks_fade_frames").max(1.0)).min(1.0);
    for id in Block::ALL {
        let dir = if block_open(&sim.state, id) {
            1.0
        } else {
            -1.0
        };
        let rt = &mut sim.progress.tasks;
        let k = rt.fold_k.get(id.as_str()).copied().unwrap_or(1.0)
            + dir / vis("tasks_fold_frames").max(1.0);
        rt.fold_k
            .insert(id.as_str().to_string(), gml::clamp(k, 0.0, 1.0));
    }
    if sim.state.state.tasks.stage == 1 {
        for g in Goal::ALL {
            let reached = gml_truthy(sim.state.state.tasks.goals.get(g.as_str()));
            if !reached && goals::goal_met(sim, g) {
                let t = &mut sim.state.state.tasks;
                t.goals.insert(g.as_str().to_string(), Value::Bool(true));
                t.popup = match g {
                    Goal::Migration => "migration_ready".into(),
                    Goal::Magic => "magic".into(),
                };
                break;
            }
        }
    } else {
        let mut all = true;
        for &d in tasks_defs() {
            let key = d.as_str();
            if !gml_truthy(sim.state.state.tasks.done.get(key)) {
                if tasks_met(sim, d) {
                    sim.state
                        .state
                        .tasks
                        .done
                        .insert(key.to_string(), Value::Bool(true));
                    sim.progress.tasks.fill.insert(key.to_string(), 0.0);
                    let reward = sim.bal("task_reward_points");
                    let st = &mut sim.state.state;
                    st.upgrade_points = (st.upgrade_points as f64 + reward) as i64;
                }
                if !gml_truthy(sim.state.state.tasks.done.get(key)) {
                    all = false;
                }
            }
        }
        if all {
            let t = &mut sim.state.state.tasks;
            t.stage = 1;
            t.popup = "tasks".into();
        }
    }
}

// port: tasks_visible
/// The panel is drawn: shown, HUD not hidden, not on the title or in the intro, not survival.
pub fn tasks_visible(sim: &Sim, input: &StepInput) -> bool {
    sim.state.state.tasks.shown
        && !input.hud_hidden
        && !sim.title_open
        && !sim.intro_active
        && sim.state.state.mode != GameMode::Survival
}

// port: tasks_block_toggle
/// `tasks_block_toggle(id)`: flip the saved open flag (`tasks.open` / `tasks.goal_open[id]`).
pub fn block_toggle(sim: &mut Sim, id: Block) {
    if id == Block::Tasks {
        let t = &mut sim.state.state.tasks;
        t.open = !t.open;
        return;
    }
    let open = block_open(&sim.state, id);
    sim.state
        .state
        .tasks
        .goal_open
        .insert(id.as_str().to_string(), Value::Bool(!open));
}

/// `Command::ToggleTaskBlock` (the app's `tasks_mouse_step` hit a header): toggle in
/// obj_hud's Begin Step, where the original reads the click.
pub fn apply_commands(sim: &mut Sim, input: &StepInput) {
    for cmd in &input.commands {
        if let Command::ToggleTaskBlock { block } = cmd {
            block_toggle(sim, *block);
        }
    }
}

// port: tasks_popup_open
/// A tasks/goal popup is up (`tasks.popup` is set), never in survival.
pub fn tasks_popup_open(sim: &Sim) -> bool {
    !sim.state.state.tasks.popup.is_empty() && sim.state.state.mode != GameMode::Survival
}

/// `tasks_popup_rect()` (`___struct___35`), GUI coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PopupRect {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub cx: f64,
}

// port: tasks_popup_rect
/// The popup panel, centred in the GUI (`gw`×`gh`, view-sized): 1300×640 for the tasks
/// popup, 1100×300 for the goal ones, shifted up by 50 to leave room for the button.
pub fn tasks_popup_rect(sim: &Sim, gw: f64, gh: f64) -> PopupRect {
    let cx = gw * 0.5;
    let cy = gh * 0.5;
    let is_tasks = sim.state.state.tasks.popup == "tasks";
    let w = if is_tasks { 1300.0 } else { 1100.0 };
    let h = if is_tasks { 640.0 } else { 300.0 };
    let y1 = cy - (h + 100.0) * 0.5;
    PopupRect {
        x1: cx - w * 0.5,
        y1,
        x2: cx + w * 0.5,
        y2: y1 + h,
        cx,
    }
}

/// What a popup button does (`id` of `___struct___36..38`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupButton {
    Close,
    Spores,
}

/// One popup button: its id, the locale key of its label (the app translates it) and rect.
#[derive(Clone, Debug, PartialEq)]
pub struct PopupButtonRect {
    pub id: PopupButton,
    pub label_key: &'static str,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

// port: tasks_popup_buttons
/// The popup's single button under the panel (at the bottom of the GUI for `migration_end`).
pub fn tasks_popup_buttons(sim: &Sim, gw: f64, gh: f64) -> Vec<PopupButtonRect> {
    let r = tasks_popup_rect(sim, gw, gh);
    let (w, h) = (420.0, 64.0);
    let popup = sim.state.state.tasks.popup.as_str();
    let y = if popup == "migration_end" {
        gh - h - 72.0
    } else {
        r.y2 + 28.0
    };
    let (id, label_key) = match popup {
        "tasks" => (PopupButton::Close, "goal_button_grow"),
        "migration_ready" => (PopupButton::Spores, "goal_button_spores"),
        _ => (PopupButton::Close, "goal_button_continue"),
    };
    vec![PopupButtonRect {
        id,
        label_key,
        x1: r.cx - w * 0.5,
        y1: y,
        x2: r.cx + w * 0.5,
        y2: y + h,
    }]
}

// port: tasks_popup_close
/// Close the popup; a migration cut-scene holding on it moves on (`cine.phase` hold → out).
pub fn tasks_popup_close(sim: &mut Sim) {
    sim.state.state.tasks.popup.clear();
    camera::cine_release_hold(sim);
}

// port: tasks_popup_step
/// obj_hud Begin Step while a popup is up (it takes all input): Escape/Enter or a click on the
/// button. "Spores" (migration ready) starts the spore cut-scene, everything else closes.
/// The GUI is view-sized, so the button rects use `input.view` w/h and `input.cursor_gui`.
pub fn tasks_popup_step(sim: &mut Sim, input: &StepInput) {
    let pressed = |k: Key| input.keys_pressed.contains(&k);
    if pressed(Key::Escape) || pressed(Key::Enter) {
        if sim.state.state.tasks.popup == "migration_ready" {
            sim.state.state.tasks.popup.clear();
            camera::cine_show_start(sim);
        } else {
            tasks_popup_close(sim);
        }
        return;
    }
    if !input.lmb.pressed {
        return;
    }
    let Some((mx, my)) = input.cursor_gui else {
        return;
    };
    for b in tasks_popup_buttons(sim, input.view.w, input.view.h) {
        if gml::point_in_rectangle(mx, my, b.x1, b.y1, b.x2, b.y2) {
            if b.id == PopupButton::Spores {
                sim.state.state.tasks.popup.clear();
                camera::cine_show_start(sim);
                return;
            }
            // the original's "exit" button id never occurs (no constructor makes it)
            tasks_popup_close(sim);
            return;
        }
    }
}
