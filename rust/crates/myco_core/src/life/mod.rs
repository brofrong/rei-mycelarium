//! Animals and other life (obj_life, scr_life*): hedgehog, squirrel, crow, boar, worm, leaves,
//! predator, drops. Task 10 ports it.
//!
//! Task 9 (mushrooms) needs part of it now: the animal lists the repellers act on, the tree
//! mushrooms' water drops, and a few pure helpers. Those helpers are faithful ports of Task 10's
//! rows without `// port:` markers (Task 10 marks them here, or moves them and keeps the names;
//! like Task 8's precedent): `squirrel_view`, `hedgehog_frame_w`, `boar_frame_w`,
//! `predator_period`, `predator_yield`, `predator_progress`, `tree_drop_period`,
//! `tree_drop_water`, `drops_step`, `animals_reindex_after_removal`, `animals_rebind`. The
//! structs below hold the fields Task 9 reads; Task 10 adds the rest.

use crate::balance::vis;
use crate::events::{ResourceKind, SimEvent};
use crate::gml;
use crate::progress::tasks;
use crate::progress::upgrades::upg_bonus;
use crate::sim::{Sim, StepInput, ViewRect};
use crate::state::GameState;
use crate::world::perks::perk_time_mult;

/// A hedgehog (`global.animals[i]`, `___struct___65/66`).
#[derive(Clone, Debug, PartialEq)]
pub struct Hedgehog {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub speed: f64,
    pub frame: f64,
    pub frame_timer: f64,
    /// index into `sim.state.mushrooms` of the mushroom on its back, -1 for none
    pub carrying: i64,
}

/// A boar (`global.boars[i]`); Task 10 adds its other fields.
#[derive(Clone, Debug, PartialEq)]
pub struct Boar {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub fleeing: bool,
}

/// A water drop falling from a tree mushroom (`global.drops[i]`, `___struct___63`).
#[derive(Clone, Debug, PartialEq)]
pub struct Drop {
    pub x: f64,
    pub y: f64,
    pub target_y: f64,
    pub speed: f64,
    /// `Resource::source` of the mushroom that dropped it (`source::MUSHROOM | id`)
    pub source: u64,
}

/// Runtime life state (`global.animals`, `global.boars`, ...). Task 10 fills it.
#[derive(Clone, Debug, Default)]
pub struct LifeRt {
    /// `global.animals` (hedgehogs)
    pub animals: Vec<Hedgehog>,
    /// `global.boars`
    pub boars: Vec<Boar>,
    /// `global.drops`
    pub drops: Vec<Drop>,
}

/// obj_life Create_0 (`life_init`, Task 10).
pub fn create(_sim: &mut Sim) {}

/// obj_life Step_0 (`intro_step`, `life_step`, Task 10). Until Task 10 ports it whole: nothing
/// while the UI blocks (intro included), else the parts of `life_step` that exist, in order.
pub fn step(sim: &mut Sim, input: &StepInput) {
    if crate::forest::ui_blocking(sim, input) {
        return;
    }
    crate::mushrooms::step(sim);
    drops_step(sim);
    // animals_step, boars_step (Task 10)
    crate::mushrooms::repeller::repeller_clouds_step(sim);
    // squirrels_step, crows_step, leaves_step, worms_step, life_far_step (Task 10)
}

/// `life_animal_timers_init` (Task 10); called by `game_mode_start`.
pub fn animal_timers_init(_sim: &mut Sim) {}

/// `load_game`'s life part (Task 10): clear animals, restore timers, `life_far_init`.
pub fn on_load(sim: &mut Sim) {
    sim.life.animals.clear();
    sim.life.boars.clear();
    sim.life.drops.clear();
}

/// `save_game`'s life part (Task 10): write animal timers into `out.state.timers`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}

/// `squirrel_view()` (Task 10's row): the camera view, world px.
pub fn squirrel_view(sim: &Sim) -> ViewRect {
    let c = &sim.camera;
    ViewRect {
        x: c.view_x,
        y: c.view_y,
        w: c.width,
        h: c.height,
    }
}

/// Width of the `hedgehog` sprite (8 frames side by side).
const HEDGEHOG_SPRITE_W: f64 = 1024.0;
/// Width of the `boar` sprite (6 frames side by side).
const BOAR_SPRITE_W: f64 = 1134.0;

/// `hedgehog_frame_w()` (Task 10's row).
pub fn hedgehog_frame_w() -> f64 {
    HEDGEHOG_SPRITE_W / 8.0 * vis("hedgehog_scale")
}

/// `boar_frame_w()` (Task 10's row).
pub fn boar_frame_w() -> f64 {
    BOAR_SPRITE_W / 6.0 * vis("boar_scale")
}

/// `predator_period()` (Task 10's row): frames between two catches of a predator mushroom.
pub fn predator_period(sim: &Sim) -> f64 {
    let d = 1.0 + upg_bonus(&sim.state, "predator_digestion");
    (sim.bal("predator_catch_interval") / d * perk_time_mult(&sim.state)).max(60.0)
}

/// `predator_yield()` (Task 10's row): food per catch.
pub fn predator_yield(sim: &Sim) -> f64 {
    (sim.bal("predator_food_per_fly") + upg_bonus(&sim.state, "predator_yield")).floor()
}

/// `predator_progress(m)` (Task 10's row): hover-bar progress of predator mushroom `i`.
pub fn predator_progress(sim: &Sim, i: usize) -> Option<f64> {
    let m = &sim.state.mushrooms[i];
    if m.carried_by == -1 && !m.stolen {
        return Some(gml::clamp(1.0 - m.timer / predator_period(sim), 0.0, 1.0));
    }
    None
}

/// `tree_drop_period()` (Task 10's row): frames between two drops of a tree mushroom.
pub fn tree_drop_period(sim: &Sim) -> f64 {
    let rate = 1.0 + upg_bonus(&sim.state, "tree_drop_rate");
    (sim.bal("tree_drop_interval") / rate * perk_time_mult(&sim.state)).max(30.0)
}

/// `tree_drop_water()` (Task 10's row): water per drop.
pub fn tree_drop_water(sim: &Sim) -> f64 {
    let y = 1.0 + upg_bonus(&sim.state, "tree_water_yield");
    sim.bal("tree_water_per_drop") * y
}

/// `drops_step()` (Task 10's row): drops fall; a landed drop adds its water (and emits
/// `Resource{Water, +n}` at the landing point with its tree mushroom as the source).
pub fn drops_step(sim: &mut Sim) {
    let drops = std::mem::take(&mut sim.life.drops);
    let mut keep = Vec::with_capacity(drops.len());
    for mut d in drops {
        d.speed = (d.speed + 0.22).min(7.0);
        d.y += d.speed;
        if d.y >= d.target_y {
            let w = tree_drop_water(sim);
            sim.state.state.water_free += w;
            sim.emit(SimEvent::Resource {
                kind: ResourceKind::Water,
                delta: w,
                x: d.x,
                y: d.target_y,
                source: d.source,
            });
            tasks::stat_add(sim, "ach_tree_water", w);
        } else {
            keep.push(d);
        }
    }
    sim.life.drops = keep;
}

/// `animals_reindex_after_removal(removed)` (Task 10's row): mushroom `removed` was deleted.
pub fn animals_reindex_after_removal(sim: &mut Sim, removed: i64) {
    for a in &mut sim.life.animals {
        if a.carrying == removed {
            a.carrying = -1;
        } else if a.carrying > removed {
            a.carrying -= 1;
        }
    }
}

/// `animals_rebind()` (Task 10's row): `carried_by` from the hedgehogs' `carrying`.
pub fn animals_rebind(sim: &mut Sim) {
    for m in &mut sim.state.mushrooms {
        m.carried_by = -1;
    }
    let n = sim.state.mushrooms.len() as i64;
    for (i, a) in sim.life.animals.iter_mut().enumerate() {
        if a.carrying != -1 && a.carrying < n {
            sim.state.mushrooms[a.carrying as usize].carried_by = i as i64;
        } else {
            a.carrying = -1;
        }
    }
}

/// Kind of a live animal the HUD shows an off-screen marker for (`animal_marker_list`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimalKind {
    Hedgehog,
    Boar,
    Squirrel,
    Crow,
    /// `global.crow_pending`: a crow about to arrive (frame from `current_time`)
    CrowPending,
}

/// What the HUD needs of one animal (Task 10 hook output, in `animal_marker_list` order:
/// `animals`, `boars`, `squirrels`, `crows`, `crow_pending`). `src` is a stable id; `dir` is
/// the walking direction (hedgehog/boar/crow), `mirror` the squirrel's own mirror.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimalView {
    pub kind: AnimalKind,
    pub src: u64,
    pub x: f64,
    pub y: f64,
    pub frame: f64,
    pub dir: f64,
    pub mirror: f64,
}

/// The live animals for the HUD markers (Task 10 hook). Until Task 10: none.
pub fn animal_views(_sim: &Sim) -> Vec<AnimalView> {
    Vec::new()
}

/// An absorbing leaf the hover bar can show (`global.leaves` with `absorbing`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LeafHover {
    pub x: f64,
    pub y: f64,
    pub timer: f64,
    pub total: f64,
}

/// A caught worm the hover bar can show (`global.worms` with `caught`).
#[derive(Clone, Debug, PartialEq)]
pub struct WormHover {
    pub body: Vec<(f64, f64)>,
    pub timer: f64,
    pub total: f64,
}

/// Absorbing leaves in `global.leaves` order (Task 10 hook). Until Task 10: none.
pub fn absorbing_leaves(_sim: &Sim) -> Vec<LeafHover> {
    Vec::new()
}

/// Caught worms in `global.worms` order (Task 10 hook). Until Task 10: none.
pub fn caught_worms(_sim: &Sim) -> Vec<WormHover> {
    Vec::new()
}
