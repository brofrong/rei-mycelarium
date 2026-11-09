//! Tasks panel logic (scr_game `tasks_*`): the first-stage task list with upgrade-point
//! rewards, then the goals stage (migration, magic). Saved state: `state.state.tasks`; the
//! panel fades live in [`TasksRt`]. Drawing, header hit-tests and popups are the app's
//! (Task 17), which also toggles `tasks.open` / `tasks.goal_open` (`tasks_block_toggle`).

use super::goals::{self, Goal};
use crate::mycelium::soil::mycelium_cm;
use super::tutorial::{self, Tip, TipState};
use super::upgrades::upg_is_owned;
use crate::balance::{gml_truthy, vis};
use crate::gml;
use crate::sim::{Sim, StepInput};
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

/// `tasks_block_open(id)` (Task 17 owns the row; the step needs it): the saved open flag of
/// the block (goal blocks default to open).
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
