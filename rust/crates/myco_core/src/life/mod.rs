//! Animals and other life (obj_life, scr_life): hedgehog, boar, squirrel, crow, worm, leaves,
//! predator, drops, the animal timers and the far-life income.
//!
//! - The original keeps each kind in its own global array (`global.animals` = hedgehogs,
//!   `global.boars`, `global.squirrels`, `global.crows`, ...) and the arrays' order and indices
//!   matter (a hedgehog's `carrying` is a mushroom index, a mushroom's `carried_by` a hedgehog
//!   index), so [`LifeRt`] keeps one `Vec` per kind in the original order. Every animal, leaf and
//!   worm has a stable `id` (`mycelium::source::{ANIMAL, LEAF, WORM} | n`): the HUD's marker
//!   source, the hyphae sound id and the `Resource` source.
//! - Mushrooms are referenced by their stable id (`MushroomLive::id`), where the original holds
//!   the struct; `mushrooms::mushroom_index_of` turns it back into an index. Trees are held as
//!   [`TreeRef`] snapshots (a tree never moves; the original keeps the struct even after the
//!   forest drops it).
//! - The animal timers live in `sim.state.state.timers` (saved), the rest is runtime.
//! - Logic that reads the camera view uses [`squirrel_view`]: the logic camera (`sim.camera`,
//!   the 1× game view the forest step moves), never the app's zoom.
//! - Sounds: the boar tremor loop (`AudioCall::LoopFollow`), the leaf/worm hyphae sounds
//!   (`HyphaeBegin/Step/Stop`); food from leaves and worms emits `Resource{Food}` (R20). The
//!   far-life income has no position (it stands for life off the screen) and emits nothing,
//!   as an off-screen source shows no number.
//! - Art data (hedgehog back line, trunk edges, leaf frames, sprite sizes) comes from
//!   [`art`] (ruling R7).

// The original's `not (a > b)` tests are kept as written: they differ from `a <= b` for NaN.
#![allow(clippy::neg_cmp_op_on_partial_ord)]

pub mod art;
pub mod boar;
pub mod crow;
pub mod drops;
pub mod hedgehog;
pub mod leaves;
pub mod predator;
pub mod squirrel;
pub mod worm;

pub use crate::mycelium::{Cell, Pt};
pub use boar::boar_frame_w;
pub use drops::{drops_step, tree_drop_period, tree_drop_water};
pub use hedgehog::hedgehog_frame_w;
pub use predator::{predator_period, predator_progress, predator_yield};
pub use squirrel::TreeRef;

use crate::balance::wld;
use crate::gml;
use crate::progress::upgrades::upg_is_owned;
use crate::sim::{Sim, StepInput, ViewRect};
use crate::state::GameState;
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// A hedgehog (`global.animals[i]`, `___struct___65/66/106`).
#[derive(Clone, Debug, PartialEq)]
pub struct Hedgehog {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub speed: f64,
    pub frame: f64,
    pub frame_timer: f64,
    /// index into `sim.state.mushrooms` of the mushroom on its back, -1 for none
    pub carrying: i64,
}

impl Default for Hedgehog {
    fn default() -> Self {
        Hedgehog {
            id: 0,
            x: 0.0,
            y: 0.0,
            dir: 1.0,
            speed: 0.0,
            frame: 0.0,
            frame_timer: 0.0,
            carrying: -1,
        }
    }
}

/// A boar (`global.boars[i]`, `___struct___67`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Boar {
    pub id: u64,
    /// ids of the mushrooms it still means to hit (`targets`)
    pub targets: Vec<u64>,
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub frame: f64,
    pub frame_timer: f64,
    pub hits: f64,
    pub fleeing: bool,
}

/// A boar of a survival pack waiting to run in (`global.boar_pack[i]`, `___struct___68`).
#[derive(Clone, Debug, PartialEq)]
pub struct PackBoar {
    pub x: f64,
    pub dir: f64,
    pub wait: f64,
}

/// A mushroom a boar flung into the air (`global.boar_flung[i]`, `___struct___69`).
#[derive(Clone, Debug, PartialEq)]
pub struct Flung {
    /// mushroom type code
    pub kind: i64,
    pub sprite: &'static str,
    pub variant: f64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub angle: f64,
    pub spin: f64,
    pub life: f64,
}

/// `squirrel.state` (0..4 in the original).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SquirrelState {
    /// 0: climbing down its tree
    #[default]
    Down,
    /// 1: jumping off the trunk to the ground
    Jump,
    /// 2: running over the ground
    Run,
    /// 3: jumping onto the target trunk
    Land,
    /// 4: climbing up and away
    Up,
}

impl SquirrelState {
    pub fn code(self) -> i64 {
        match self {
            SquirrelState::Down => 0,
            SquirrelState::Jump => 1,
            SquirrelState::Run => 2,
            SquirrelState::Land => 3,
            SquirrelState::Up => 4,
        }
    }
}

/// A squirrel (`global.squirrels[i]`, `___struct___78`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Squirrel {
    pub id: u64,
    pub tree: TreeRef,
    pub home: TreeRef,
    pub target_tree: TreeRef,
    pub far_tree: TreeRef,
    pub direct: bool,
    pub side: f64,
    pub dir: f64,
    pub mirror: f64,
    pub angle: f64,
    pub x: f64,
    pub y: f64,
    pub stop_y: f64,
    pub run_line: f64,
    pub state: SquirrelState,
    pub turn_t: f64,
    pub turn_from_angle: f64,
    pub turn_to_angle: f64,
    pub turn_from_x: f64,
    pub turn_from_y: f64,
    pub turn_to_x: f64,
    pub turn_to_y: f64,
    pub frame: f64,
    pub frame_timer: f64,
    pub timer: f64,
    /// id of the mushroom it is after (or carries)
    pub mushroom: Option<u64>,
    pub empty_run: bool,
    pub grab_x: f64,
    pub grab_y: f64,
}

/// `crow.state` (0..2 in the original).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CrowState {
    /// 0: diving at the mushroom
    #[default]
    Dive,
    /// 1: holding at the grab
    Grab,
    /// 2: climbing away
    Climb,
}

impl CrowState {
    pub fn code(self) -> i64 {
        match self {
            CrowState::Dive => 0,
            CrowState::Grab => 1,
            CrowState::Climb => 2,
        }
    }
}

/// A crow (`global.crows[i]`, `___struct___81`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Crow {
    pub id: u64,
    pub state: CrowState,
    pub dir: f64,
    pub x: f64,
    pub y: f64,
    pub x0: f64,
    pub y0: f64,
    pub grab_x: f64,
    pub grab_y: f64,
    pub t: f64,
    pub frame: f64,
    pub frame_timer: f64,
    pub timer: f64,
    pub speed: f64,
    pub flee: bool,
    pub mushroom: Option<u64>,
}

/// A crow announced before it arrives (`global.crow_pending[slot]`, `___struct___82`). The crow
/// that comes keeps the warning's id, so the HUD marker follows it (crow_due rebinds the marker).
#[derive(Clone, Debug, PartialEq)]
pub struct CrowPending {
    pub id: u64,
    pub mushroom: u64,
    pub dir: f64,
    pub x: f64,
    pub y: f64,
}

/// A falling or lying leaf (`global.leaves[i]`, `___struct___91`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Leaf {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub phase: f64,
    pub mirror: f64,
    pub turn: f64,
    pub turn_timer: f64,
    pub turned: bool,
    pub turn_delay: f64,
    pub angle: f64,
    pub hold_angle: f64,
    pub drift: f64,
    pub landed: bool,
    pub absorbing: bool,
    pub timer: f64,
    pub life: f64,
    /// set when absorption starts
    pub total: Option<f64>,
    pub grow_frames: Option<f64>,
    pub tendril: Option<Vec<Cell>>,
    /// a hyphae sound runs (id = leaf id)
    pub sound: bool,
}

/// A retracting leaf/worm tendril (`global.leaf_tendrils_out[i]`, `___struct___92/104`).
#[derive(Clone, Debug, PartialEq)]
pub struct TendrilOut {
    pub cells: Vec<Cell>,
    pub timer: f64,
    pub total: f64,
}

/// A worm (`global.worms[i]`, `___struct___98`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Worm {
    pub id: u64,
    pub body: Vec<Pt>,
    pub dir: f64,
    pub turn: f64,
    pub side: f64,
    pub phase: f64,
    pub check: f64,
    pub trail_walk: f64,
    pub caught: bool,
    pub timer: f64,
    pub total: f64,
    pub struggle: f64,
    pub grow_frames: Option<f64>,
    pub touch_frames: Option<f64>,
    pub tendril: Option<Vec<Cell>>,
    /// a hyphae sound runs (id = worm id)
    pub sound: bool,
}

/// A dot of a worm's trail (`global.worm_trail[i]`, `___struct___99`).
#[derive(Clone, Debug, PartialEq)]
pub struct TrailDot {
    pub x: f64,
    pub y: f64,
    pub life: f64,
}

/// `global.far_rate` (`___struct___85`): per-minute leaves, worms and food of the far life.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FarRate {
    pub leaves: f64,
    pub worms: f64,
    pub food: f64,
}

/// The far-life globals (`life_far_init`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FarRt {
    /// `global.far_leaf_cells`: 16 px surface cells within the leaf absorb radius of a strand
    pub leaf_cells: BTreeSet<i64>,
    /// `global.far_worm_columns`: strand length per 1024 px column inside the worm zone, keyed
    /// as the original's struct names (sorted as strings, like `variable_struct_get_names`)
    pub worm_columns: BTreeMap<String, f64>,
    pub scan_from: usize,
    pub timer: f64,
    pub food_due: f64,
    pub rate: FarRate,
    /// growths already measured (`g.far_measured`), by growth id
    pub measured: HashSet<u64>,
}

/// Runtime life state (`global.animals`, `global.boars`, ...).
#[derive(Clone, Debug, Default)]
pub struct LifeRt {
    /// `global.animals` (hedgehogs)
    pub animals: Vec<Hedgehog>,
    pub boars: Vec<Boar>,
    pub boar_pack: Vec<PackBoar>,
    pub boar_flung: Vec<Flung>,
    pub squirrels: Vec<Squirrel>,
    pub crows: Vec<Crow>,
    pub crow_pending: [Option<CrowPending>; 3],
    pub leaves: Vec<Leaf>,
    pub leaf_tendrils_out: Vec<TendrilOut>,
    pub drops: Vec<drops::Drop>,
    pub worms: Vec<Worm>,
    pub worm_trail: Vec<TrailDot>,
    pub far: FarRt,
    /// the intro hedgehog and its dropped origin mushroom (`global.intro`, Task 21's intro)
    pub intro: crate::camera::Intro,
    next_id: u64,
}

pub use drops::Drop;

impl LifeRt {
    /// A fresh stable id under `ns` (a `mycelium::source` namespace).
    pub fn new_id(&mut self, ns: u64) -> u64 {
        self.next_id += 1;
        ns | self.next_id
    }
}

// port: gml_Object_obj_life_Create_0
/// obj_life Create_0: `life_init`.
pub fn create(sim: &mut Sim) {
    life_init(sim);
}

// port: life_init
/// `life_init`: empty animal lists, the animal, leaf and worm timers, far life, the intro.
pub fn life_init(sim: &mut Sim) {
    sim.fps = sim.fps.max(1.0);
    sim.state.mushrooms.clear();
    let next_id = sim.life.next_id;
    sim.life = LifeRt {
        next_id,
        ..Default::default()
    };
    sim.mushrooms.repel_clouds.clear();
    sim.mushrooms.repel_kick = 0.0;
    life_animal_timers_init(sim);
    sim.state.state.timers.leaf = sim.bal("leaf_spawn_interval");
    sim.state.state.timers.worm = 60.0;
    sim.mushrooms.stinky_live_count = 0.0;
    sim.mushrooms.stinky_live_last = 0.0;
    life_far_init(sim);
    crate::camera::intro_init(sim);
}

// port: life_animal_timers_init
/// First spawn delays of every animal (rolled), as a new game or a new mode starts them.
pub fn life_animal_timers_init(sim: &mut Sim) {
    let dmax = sim.bal("hedgehog_first_delay_max");
    let dmin = sim.bal("hedgehog_first_delay_min");
    let r = sim.rng.random_range(dmin, dmax);
    let fps = sim.fps;
    sim.state.state.timers.hedgehog = (sim.bal("hedgehog_spawn_interval") + r) * fps;
    sim.state.state.timers.squirrel = animal_interval(sim, "squirrel_spawn_interval");
    sim.state.state.timers.boar = Some(sim.bal("boar_first_delay") * fps);
    sim.state.state.timers.crow = animal_interval(sim, "crow_spawn_interval");
    let t0 = sim.bal("experience_crow_interval") * fps;
    let off = sim.bal("experience_crow_offset");
    let t1 = (sim.bal("experience_crow_interval") + off) * fps;
    sim.state.state.timers.experience_crows = vec![t0, t1];
}

/// `life_animal_timers_init` (called by `game_mode_start`).
pub fn animal_timers_init(sim: &mut Sim) {
    life_animal_timers_init(sim);
}

// port: animal_interval
/// `bal(key)` seconds in frames, jittered by `animal_interval_jitter`.
pub fn animal_interval(sim: &mut Sim, key: &str) -> f64 {
    let j = gml::clamp(sim.bal("animal_interval_jitter"), 0.0, 0.95);
    let (lo, hi) = (1.0 - j, 1.0 + j);
    sim.bal(key) * sim.fps * sim.rng.random_range(lo, hi)
}

// port: animal_mushroom_count
/// Mushrooms standing on their own (not carried, not stolen).
pub fn animal_mushroom_count(sim: &Sim) -> f64 {
    sim.state
        .mushrooms
        .iter()
        .filter(|m| m.carried_by == -1 && !m.stolen)
        .count() as f64
}

// port: animal_timer_step
/// How fast the animal timers run: more mushrooms, sooner animals.
pub fn animal_timer_step(sim: &Sim) -> f64 {
    let max_cut = sim.bal("animal_interval_max_cut");
    let per = sim.bal("animal_interval_per_mushroom");
    let cut = max_cut.min(animal_mushroom_count(sim) * per);
    1.0 / (0.05f64).max(1.0 - cut)
}

/// `global.boar_timer` (always set once `life_init` or `load_game` ran).
pub(crate) fn boar_timer(sim: &Sim) -> f64 {
    sim.state.state.timers.boar.unwrap_or(0.0)
}

// port: gml_Object_obj_life_Step_0
/// obj_life Step_0: the intro runs alone; nothing while the UI blocks; else `life_step`.
pub fn step(sim: &mut Sim, input: &StepInput) {
    if crate::camera::intro_step(sim) {
        return;
    }
    if crate::forest::ui_blocking(sim, input) {
        return;
    }
    life_step(sim);
}

// port: life_step
/// Every life system, in order.
pub fn life_step(sim: &mut Sim) {
    crate::mushrooms::step(sim);
    drops_step(sim);
    hedgehog::animals_step(sim);
    boar::boars_step(sim);
    crate::mushrooms::repeller::repeller_clouds_step(sim);
    squirrel::squirrels_step(sim);
    crow::crows_step(sim);
    leaves::leaves_step(sim);
    worm::worms_step(sim);
    life_far_step(sim);
}

/// `load_game`'s life part: every animal list emptied, the timers are the saved ones (in
/// `sim.state`; the boar fallback is `GameState::apply_load_defaults`), `life_far_init`.
pub fn on_load(sim: &mut Sim) {
    let next_id = sim.life.next_id;
    let intro = std::mem::take(&mut sim.life.intro);
    sim.life = LifeRt {
        next_id,
        intro,
        ..Default::default()
    };
    if sim.state.state.timers.boar.is_none() {
        sim.state.state.timers.boar = Some(sim.bal("boar_spawn_interval") * sim.fps);
    }
    life_far_init(sim);
}

/// `save_game`'s life part: the timers already live in `sim.state.state.timers`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}

// port: squirrel_view
/// The camera view (`view_camera[0]`): the logic camera, the 1× game view, world px.
pub fn squirrel_view(sim: &Sim) -> ViewRect {
    let c = &sim.camera;
    ViewRect {
        x: c.view_x,
        y: c.view_y,
        w: c.width,
        h: c.height,
    }
}

/// `life_near_zone()` (`___struct___90`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearZone {
    pub x1: f64,
    pub x2: f64,
    pub w: f64,
}

// port: life_near_zone
/// The view widened by `life_near_screens` on both sides: where leaves and worms live.
pub fn life_near_zone(sim: &Sim) -> NearZone {
    let v = squirrel_view(sim);
    let m = v.w * sim.bal("life_near_screens");
    NearZone {
        x1: v.x - m,
        x2: v.x + v.w + m,
        w: v.w,
    }
}

// port: life_far_init
pub fn life_far_init(sim: &mut Sim) {
    sim.life.far = FarRt::default();
}

// port: life_far_measure
/// Record growth `gi`'s strands: surface cells within the leaf radius, strand length per worm
/// column inside the worm zone.
pub fn life_far_measure(sim: &mut Sim, gi: usize) {
    let sy = wld("surface_height_px") + wld("reference_tile_size");
    let r = sim.bal("leaf_absorb_radius");
    let wt = worm::worm_zone_top(sim);
    let wb = worm::worm_zone_bottom(sim);
    let far = &mut sim.life.far;
    for br in &sim.mycelium.growths[gi].branches {
        let pts = &br.points;
        for (j, p) in pts.iter().enumerate() {
            let dy = p.y - sy;
            if dy.abs() <= r {
                let hw = (r * r - dy * dy).sqrt();
                let c1 = ((p.x + hw) / 16.0).floor();
                let mut c = ((p.x - hw) / 16.0).floor();
                while c <= c1 {
                    far.leaf_cells.insert(c as i64);
                    c += 1.0;
                }
            }
            if j != 0 {
                let q = pts[j - 1];
                let my = (p.y + q.y) * 0.5;
                if !(my < wt) && !(my > wb) {
                    let key = format!("{}", ((p.x + q.x) * 0.5 / 1024.0).floor() as i64);
                    let cur = far.worm_columns.entry(key).or_insert(0.0);
                    *cur += gml::point_distance(p.x, p.y, q.x, q.y);
                }
            }
        }
    }
}

// port: life_far_outside
/// Share of `[a, b]` outside the near zone.
pub fn life_far_outside(a: f64, b: f64, v: &NearZone) -> f64 {
    let ov = 0f64.max(b.min(v.x2) - a.max(v.x1));
    1.0 - ov / 1f64.max(b - a)
}

// port: life_far_step
/// Measure newly finished growths, refresh the far rates every `life_far_interval_frames`, and
/// pay the far food out in whole units. (The original stops measuring after 1 ms of a frame;
/// the port measures every pending growth, as an infinitely fast machine would.)
pub fn life_far_step(sim: &mut Sim) {
    sim.life.far.timer -= 1.0;
    let mut i = sim.life.far.scan_from;
    while i < sim.mycelium.growths.len() {
        let g = &sim.mycelium.growths[i];
        if g.finished && !sim.life.far.measured.contains(&g.id) {
            let id = g.id;
            life_far_measure(sim, i);
            sim.life.far.measured.insert(id);
        }
        i += 1;
    }
    while sim.life.far.scan_from < sim.mycelium.growths.len() {
        let id = sim.mycelium.growths[sim.life.far.scan_from].id;
        if !sim.life.far.measured.contains(&id) {
            break;
        }
        sim.life.far.scan_from += 1;
    }
    if sim.life.far.timer <= 0.0 {
        sim.life.far.timer = sim.bal("life_far_interval_frames");
        let z = life_near_zone(sim);
        let per_min = 60.0 * sim.fps;
        let mut leaves = 0.0;
        let mut worms = 0.0;
        if upg_is_owned(&sim.state, "unlock_absorption") {
            let cnt = sim
                .life
                .far
                .leaf_cells
                .iter()
                .filter(|&&c| {
                    let cx = c as f64 * 16.0 + 8.0;
                    cx < z.x1 || cx > z.x2
                })
                .count() as f64;
            let interval = sim.bal("leaf_spawn_interval");
            leaves = cnt * 16.0 / 1f64.max(z.w) * per_min / 1f64.max(interval);
        }
        if upg_is_owned(&sim.state, "unlock_protein") {
            let wcap = sim.bal("worm_far_cap_per_screen_min") * 1024.0 / 1f64.max(z.w);
            let per = sim.bal("worm_far_catch_per_1000px_min");
            for (name, len) in &sim.life.far.worm_columns {
                let c0 = name.parse::<f64>().unwrap_or(0.0) * 1024.0;
                let out = life_far_outside(c0, c0 + 1024.0, &z);
                if !(out <= 0.0) {
                    worms += wcap.min(per * len / 1000.0) * out;
                }
            }
        }
        let wf = worms * sim.bal("worm_food_reward");
        let food = leaves * leaves::leaf_food_value(sim) + wf;
        sim.life.far.rate = FarRate {
            leaves,
            worms,
            food,
        };
    }
    let far = &mut sim.life.far;
    far.food_due += far.rate.food / (60.0 * sim.fps);
    if far.food_due >= 1.0 {
        let f = far.food_due.floor();
        far.food_due -= f;
        sim.food_add(f);
    }
}

// port: animals_reindex_after_removal
/// Mushroom `removed` was deleted: the hedgehogs' `carrying` indices follow.
pub fn animals_reindex_after_removal(sim: &mut Sim, removed: i64) {
    for a in &mut sim.life.animals {
        if a.carrying == removed {
            a.carrying = -1;
        } else if a.carrying > removed {
            a.carrying -= 1;
        }
    }
}

// port: animals_rebind
/// `carried_by` from the hedgehogs' `carrying`.
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

/// Delete mushroom `idx` the way the animals do (`array_delete` + reindex).
pub(crate) fn mushroom_delete(sim: &mut Sim, idx: usize) {
    sim.state.mushrooms.remove(idx);
    animals_reindex_after_removal(sim, idx as i64);
}

/// The index of mushroom `id`, if it still exists (`mushroom_index_of(m) >= 0`).
pub(crate) fn mushroom_idx(sim: &Sim, id: u64) -> Option<usize> {
    let i = crate::mushrooms::mushroom_index_of(sim, id);
    (i >= 0).then_some(i as usize)
}

// ---------------------------------------------------------------------------------------------
// What the HUD reads (markers and hover bars; Task 17 draws them)

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

/// What the HUD needs of one animal (in `animal_marker_list` order: `animals`, `boars`,
/// `squirrels`, `crows`, `crow_pending`). `src` is the animal's stable id (a pending crow and
/// the crow it becomes share it); `dir` is the walking direction (hedgehog/boar/crow), `mirror`
/// the squirrel's own mirror.
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

/// The live animals for the HUD markers.
pub fn animal_views(sim: &Sim) -> Vec<AnimalView> {
    let l = &sim.life;
    let mut out = Vec::new();
    let v = |kind, src, x, y, frame, dir, mirror| AnimalView {
        kind,
        src,
        x,
        y,
        frame,
        dir,
        mirror,
    };
    for a in &l.animals {
        out.push(v(AnimalKind::Hedgehog, a.id, a.x, a.y, a.frame, a.dir, 1.0));
    }
    for b in &l.boars {
        out.push(v(AnimalKind::Boar, b.id, b.x, b.y, b.frame, b.dir, 1.0));
    }
    for s in &l.squirrels {
        out.push(v(
            AnimalKind::Squirrel,
            s.id,
            s.x,
            s.y,
            s.frame,
            s.dir,
            s.mirror,
        ));
    }
    for c in &l.crows {
        out.push(v(AnimalKind::Crow, c.id, c.x, c.y, c.frame, c.dir, 1.0));
    }
    for p in l.crow_pending.iter().flatten() {
        out.push(v(AnimalKind::CrowPending, p.id, p.x, p.y, 0.0, p.dir, 1.0));
    }
    out
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

/// Absorbing leaves in `global.leaves` order.
pub fn absorbing_leaves(sim: &Sim) -> Vec<LeafHover> {
    sim.life
        .leaves
        .iter()
        .filter(|l| l.absorbing)
        .map(|l| LeafHover {
            x: l.x,
            y: l.y,
            timer: l.timer,
            total: l.total.unwrap_or(0.0),
        })
        .collect()
}

/// Caught worms in `global.worms` order.
pub fn caught_worms(sim: &Sim) -> Vec<WormHover> {
    sim.life
        .worms
        .iter()
        .filter(|w| w.caught)
        .map(|w| WormHover {
            body: w.body.iter().map(|p| (p.x, p.y)).collect(),
            timer: w.timer,
            total: w.total,
        })
        .collect()
}

/// Id namespaces of life's sources (re-exported for the app).
pub mod ids {
    pub use crate::mycelium::source::{ANIMAL, LEAF, WORM};
}
