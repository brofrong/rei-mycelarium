//! Upgrades, skill tree, tasks, goals, tutorial (Task 11).
//!
//! The saved part lives in `sim.state.state` (`upgrade_points`, `upgrade_levels`, `tutorial`,
//! `tasks`); [`ProgressRt`] holds what the original keeps only at runtime (upgrade grow
//! animation, tasks panel fades, tutorial hand/camera animation, cheat flash).
//!
//! Entry points the obj_game orchestration (Task 10) calls: `game_init` →
//! [`upgrades::init`], [`tutorial::init`], [`tasks::init`]; `game_step` →
//! [`upgrades::grow_step`], [`tutorial::step`], [`tasks::step`]; obj_game Step_0 →
//! [`cheats_step`]. Other subsystems report through [`tutorial::event`] and
//! [`tasks::stat_add`]. `Command::BuyUpgrade` is handled in [`hud_begin_step`] (the tree screen
//! buys in obj_hud's Begin Step).

pub mod goals;
pub mod skilltree;
pub mod tasks;
pub mod tutorial;
pub mod upgrades;

use crate::balance::gml_truthy;
use crate::sim::{Sim, StepInput};
use crate::state::GameState;
use std::collections::BTreeMap;

/// Runtime-only progress state.
#[derive(Clone, Debug, PartialEq)]
pub struct ProgressRt {
    /// `global.upgrade_grow`: 0..1 growth of each tree node's hypha (tree screen animation)
    pub upgrade_grow: BTreeMap<String, f64>,
    /// Drawn hypha length per node (`tree_hyphae[id].hypha.main_len * total`), filled by the
    /// tree screen (Task 19) once it builds the hyphae; missing ids use
    /// [`skilltree::tree_hypha_length_fallback`], as `tree_hypha_length` does.
    pub tree_hypha_len: BTreeMap<String, f64>,
    pub tutorial: tutorial::TutorialRt,
    pub tasks: tasks::TasksRt,
    /// `global.cheat_text`, `global.cheat_timer` (frames the cheat text stays on screen)
    pub cheat_text: String,
    pub cheat_timer: f64,
}

impl Default for ProgressRt {
    /// The state right after `upgrades_init` / `tutorial_init` / `tasks_init` with a fresh
    /// `upgrade_levels` (`origin` owned).
    fn default() -> Self {
        let mut upgrade_grow = BTreeMap::new();
        for d in skilltree::layout() {
            upgrade_grow.insert(d.id.to_string(), if d.id == "origin" { 1.0 } else { 0.0 });
        }
        ProgressRt {
            upgrade_grow,
            tree_hypha_len: BTreeMap::new(),
            tutorial: tutorial::TutorialRt::default(),
            tasks: tasks::TasksRt::default(),
            cheat_text: String::new(),
            cheat_timer: 0.0,
        }
    }
}

/// The logic part of obj_hud Step_1 (Begin Step), in its order: `water_counter_step`, the
/// tasks headers (`tasks_mouse_step`), a click on an animal marker (`camera_fly_to_marker`),
/// `cine_step`; then the early returns for a blocking cut-scene, the tasks popup, the pause,
/// the debug overlay, the title and the logo screen; only past those do the tree purchases
/// (`upg_buy` from the tree screen, as `Command::BuyUpgrade`), the camera buttons
/// (`camera_jump_*`) and the placement run. The HUD itself (layout, menus, Escape handling) is the
/// app's (Task 17).
pub fn hud_begin_step(sim: &mut Sim, input: &StepInput) {
    use crate::sim::Command;
    // obj_hud Step_1 runs water_counter_step before its input handling
    crate::world::water_counter_step(sim);
    tasks::apply_commands(sim, input);
    for cmd in &input.commands {
        match *cmd {
            Command::CameraFlyTo { x, y } => crate::camera::camera_fly_to_marker(sim, x, y),
            Command::CameraFlyTrack { x, y } => crate::camera::camera_fly_track(sim, x, y),
            _ => {}
        }
    }
    crate::camera::cine_step(sim);
    // the popup state is core's (saved); the app also reports it (`StepInput.tasks_popup_open`)
    let popup = tasks::tasks_popup_open(sim) || input.tasks_popup_open;
    if crate::camera::cine_blocking(sim) && !popup {
        sim.mushrooms.place_preview = None;
        return;
    }
    if popup {
        if tasks::tasks_popup_open(sim) {
            tasks::tasks_popup_step(sim, input);
        }
        sim.mushrooms.place_preview = None;
        return;
    }
    if sim.pause_open || input.debug_open || sim.title_open || input.logo_open {
        sim.mushrooms.place_preview = None;
        return;
    }
    // the tree screen's branch (menu_open) buys upgrades; the game branch has the camera buttons
    upgrades::apply_commands(sim, input);
    if !input.menu_open && !input.hud_hidden && !sim.intro_active {
        for cmd in &input.commands {
            match *cmd {
                Command::CameraJumpEdge { dir } => crate::camera::camera_jump_edge(sim, dir),
                Command::CameraJumpOrigin => crate::camera::camera_jump_origin(sim),
                _ => {}
            }
        }
    }
    // the HUD's placement mode places mushrooms here (obj_hud Step_1 -> myc_place_check /
    // myc_place_mushroom), only where Step_1 gets that far (ruling R24)
    if hud_reaches_placement(sim, input) {
        crate::mycelium::apply_hud_commands(sim, input);
    } else {
        sim.mushrooms.place_preview = None;
    }
}

/// Whether obj_hud Step_1 gets to its placement code this step (ruling R24): its early returns
/// for the pause menu, the debug overlay, the title (slots, credits, menu), the logo screen,
/// the upgrade tree, a hidden HUD or the intro, a right click in placement mode (cancels it), a
/// camera button click, and a build key for an unlocked type (`hud_build_all_types` = 0..7).
/// (A cut-scene and the tasks popup return earlier, in [`hud_begin_step`].)
pub fn hud_reaches_placement(sim: &Sim, input: &StepInput) -> bool {
    use crate::mushrooms::{mushroom_type_unlocked, MushroomKind};
    use crate::sim::{Command, Key};
    if sim.pause_open || input.debug_open || sim.title_open || input.logo_open || input.menu_open {
        return false;
    }
    if input.hud_hidden || sim.intro_active {
        return false;
    }
    if input.rmb.pressed && input.placement.is_some() {
        return false;
    }
    if input.commands.iter().any(|c| {
        matches!(
            c,
            Command::CameraJumpEdge { .. } | Command::CameraJumpOrigin
        )
    }) {
        return false;
    }
    for (k, t) in MushroomKind::ALL.iter().enumerate().take(8) {
        let digit = char::from(b'1' + k as u8);
        if input.keys_pressed.contains(&Key::Char(digit))
            || input.keys_pressed.contains(&Key::Numpad(1 + k as u8))
        {
            // the first build key pressed: an unlocked type opens the bar and returns
            return !mushroom_type_unlocked(&sim.state, *t);
        }
    }
    true
}

/// `load_game`'s progress part: `upgrade_grow` from owned upgrades, the tutorial tips merged
/// over the fresh ones (done by `GameState::apply_load_defaults`), `tasks_init` then the
/// saved task state with the panel fades settled.
pub fn on_load(sim: &mut Sim) {
    let grow: BTreeMap<String, f64> = skilltree::layout()
        .iter()
        .map(|d| {
            let owned = upgrades::upg_is_owned(&sim.state, d.id);
            (d.id.to_string(), if owned { 1.0 } else { 0.0 })
        })
        .collect();
    sim.progress.upgrade_grow = grow;
    // tasks_init resets the runtime part; the saved part is already in sim.state
    sim.progress.tasks = tasks::TasksRt::default();
    let t = &sim.state.state.tasks;
    let fold = |open: bool| if open { 1.0 } else { 0.0 };
    let goal_open = |k: &str| t.goal_open.get(k).is_none_or(|v| gml_truthy(Some(v)));
    let rt = &mut sim.progress.tasks;
    rt.fold_k.insert("tasks".into(), fold(t.open));
    rt.fold_k
        .insert("migration".into(), fold(goal_open("migration")));
    rt.fold_k.insert("magic".into(), fold(goal_open("magic")));
    rt.alpha = if t.shown { 1.0 } else { 0.0 };
}

/// `save_game`'s progress part: everything saved already lives in `sim.state`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}

// port: cheats_step
/// obj_game Step_0: count the cheat flash down (the cheat keys themselves are compiled out).
pub fn cheats_step(sim: &mut Sim) {
    if sim.progress.cheat_timer > 0.0 {
        sim.progress.cheat_timer -= 1.0;
    }
}

// port: cheat_flash
/// Show `text` as the cheat flash for 150 frames.
pub fn cheat_flash(sim: &mut Sim, text: &str) {
    sim.progress.cheat_text = text.to_string();
    sim.progress.cheat_timer = 150.0;
}
