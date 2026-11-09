//! Animals and other life (obj_life, scr_life*): hedgehog, squirrel, crow, boar, worm, leaves,
//! predator, drops. Task 10 ports it.

use crate::sim::{Sim, StepInput};
use crate::state::GameState;

/// Runtime life state (`global.animals`, `global.boars`, ...). Task 10 fills it.
#[derive(Clone, Debug, Default)]
pub struct LifeRt {}

/// obj_life Create_0 (`life_init`, Task 10).
pub fn create(_sim: &mut Sim) {}

/// obj_life Step_0 (`intro_step`, `life_step`, Task 10).
pub fn step(_sim: &mut Sim, _input: &StepInput) {}

/// `life_animal_timers_init` (Task 10); called by `game_mode_start`.
pub fn animal_timers_init(_sim: &mut Sim) {}

/// `load_game`'s life part (Task 10): clear animals, restore timers, `life_far_init`.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s life part (Task 10): write animal timers into `out.state.timers`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}

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
