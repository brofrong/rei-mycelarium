//! Mycelium (`obj_mycelium_controller`): aiming and the LMB shot, hyphae growth and scanning,
//! nodes (knots) and the right-click quick node, spores, node/digger bursts, the in-world
//! build menu and mushroom placement checks, length/cm and the purple glow pulses.
//!
//! Layout (one file per area; every original function carries its `// port:` marker):
//! - [`soil`]: length/cm, `reach_expand`, soil penetration;
//! - [`shot`]: the shot parameters (`myc_shot_range` ...), shot cost/food, the growth itself
//!   (`myc_add_growth_from`, `myc_make_branch`, `myc_build_branch`, `myc_rasterize_branch`);
//! - [`growth`]: the spatial grid, mycelium hit test, scanning (stone shoves, harvest, length),
//!   finishing, link growths, node burst and digger burst;
//! - [`knots`]: knots, the quick node, spore knots and smallshrooms, spores, aim start, prompts;
//! - [`place`]: playability, the build menu, surface building and mushroom placement checks.
//!
//! Rendering is the app's (ruling R9): logic marks growths that need (re)baking in
//! [`MyceliumRt::dirty_growths`] (= `myc_unbaked`) and keeps tile bookkeeping
//! ([`MyceliumRt::tiles`], `myc_tile_add`); Task 14 bakes, merges and draws from those.
//! In-world buttons are hit-tested by the app, which sends `Command::MycBuildOption`,
//! `Command::MycReleaseSpores` and `Command::MycRegenRepeller` on a click (ruling R9b).
//!
//! Time budgets (`get_timer` deadlines in the digger and burst steps) are treated as never
//! running out, like an infinitely fast machine, so the simulation stays deterministic.

pub mod growth;
pub mod knots;
pub mod place;
pub mod shot;
pub mod soil;

pub use growth::Hit;
pub use knots::{Knot, Ray, Smallshroom, Spore};
pub use place::{BuildOption, Plan};
pub use shot::GrowthOpts;

use std::collections::{BTreeMap, HashMap};

use crate::audio::HyphaePlan;
use crate::balance::{vis, wld};
use crate::events::{AudioCall, SimEvent};
use crate::sim::{Command, Key, Sim, StepInput};
use crate::state::{BranchSave, Extra, GameState, GrowthSave, MyceliumSave, PointSave};
use crate::world::perks::perk_has;

/// Grid cell size of the growth lookup (`myc_grid`).
pub const GRID_CELL: f64 = 512.0;
/// Tile size of the baked-mesh bookkeeping (`myc_tiles`).
pub const TILE_SIZE: f64 = 1024.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pt {
    pub x: f64,
    pub y: f64,
}

/// A rasterized strand cell (`myc_rasterize_branch`): integer pixel position and width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub x: f64,
    pub y: f64,
    pub w: f64,
}

/// One strand of a growth (`___struct___498`).
#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    pub points: Vec<Pt>,
    /// rasterized cells; `None` when built without raster (instant / loaded) or once baked
    pub cells: Option<Vec<Cell>>,
    pub shoved: bool,
    pub swept: f64,
    /// age (frames) at which this strand starts growing
    pub start: f64,
    pub width: f64,
    pub bx1: f64,
    pub by1: f64,
    pub bx2: f64,
    pub by2: f64,
    /// segments already harvested and their length (`scan_done`, `scan_done_len`)
    pub scan_done: f64,
    pub scan_done_len: f64,
    /// unknown fields of a loaded branch, kept for the save
    pub extra: Extra,
}

/// The hyphae sound of a growth: the plan rolled by `hyphae_sound_begin` and whether the
/// original's `s.done` is set (see [`crate::audio::HyphaePlan::done_at`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowthSound {
    pub plan: HyphaePlan,
    pub done: bool,
}

/// A growth (`___struct___502` shot, `___struct___489` link, `___struct___196` loaded).
#[derive(Clone, Debug, PartialEq)]
pub struct Growth {
    /// stable id: hyphae sound id and `Resource` source
    pub id: u64,
    pub age: f64,
    pub duration: f64,
    pub finish_age: f64,
    pub branches: Vec<Branch>,
    pub scanned_length: f64,
    pub finished: bool,
    pub budding: bool,
    /// link and loaded growths do not add to the mycelium length
    pub no_length: bool,
    pub sound: Option<GrowthSound>,
    /// a burst growth finished at once (`g.instant`)
    pub instant: bool,
    pub bx1: f64,
    pub by1: f64,
    pub bx2: f64,
    pub by2: f64,
    /// distance from the camera centre when queued for baking after a load (`bake_d`)
    pub bake_d: f64,
    /// `grid_stamp` of the last hit test that visited it
    pub grid_stamp: f64,
    pub extra: Extra,
}

/// A queued digger strand (`___struct___504`); `shared` indexes the burst's "sounded" flag.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DiggerJob {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
    pub shared: usize,
}

/// A knot queued by the node burst (`___struct___506`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BurstKnot {
    pub x: f64,
    pub y: f64,
    pub d: f64,
}

/// A glow pulse ring (`___struct___511`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pulse {
    pub born: f64,
    pub far: f64,
}

/// Baked-mesh tile bookkeeping (`___struct___513` without the GPU buffers, which Task 14 adds).
#[derive(Clone, Debug, PartialEq)]
pub struct MycTile {
    pub key: (i64, i64),
    /// growth indices waiting to be merged into the tile mesh
    pub pending: Vec<usize>,
    pub since: f64,
    pub bx1: f64,
    pub by1: f64,
    pub bx2: f64,
    pub by2: f64,
}

/// `myc_prof` entry.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Prof {
    pub now: f64,
    pub peak: f64,
}

/// Runtime mycelium: the `obj_mycelium_controller` instance variables.
#[derive(Clone, Debug, Default)]
pub struct MyceliumRt {
    // constants (Create_0)
    pub surface_y: f64,
    pub tile: f64,
    pub sample_step: f64,
    pub min_growth_len: f64,
    pub growth_duration: f64,
    pub default_aim_dir: f64,
    pub food_pickup_radius: f64,
    pub knot_radius: f64,
    pub pick_radius: f64,
    pub core_color: f64,
    /// the origin mushroom (`self.x`, `self.y`)
    pub x: f64,
    pub y: f64,
    // view (`myc_sync_view`)
    pub camera_x: f64,
    pub camera_y: f64,
    pub view_w: f64,
    pub view_h: f64,
    /// last cursor position (`mouse_x`, `mouse_y` keep their value when the cursor leaves)
    pub mouse_x: f64,
    pub mouse_y: f64,
    // aim
    pub dragging: bool,
    pub has_aim: bool,
    pub aim_origin_x: f64,
    pub aim_origin_y: f64,
    pub aim_dir: f64,
    pub aim_len: f64,
    pub aim_reach: f64,
    pub aim_cost: f64,
    pub aim_food: f64,
    pub drag_start_x: f64,
    pub drag_start_y: f64,
    // mycelium
    pub growths: Vec<Growth>,
    pub knots: Vec<Knot>,
    /// growths still aging (`myc_active`), indices into `growths`
    pub active: Vec<usize>,
    /// finished growths waiting to be baked (`myc_unbaked`, popped from the end), indices
    /// into `growths`; Task 14 pops and bakes them (ruling R9a)
    pub dirty_growths: Vec<usize>,
    /// set when every baked tile must be dropped (`myc_tiles_clear` on load); Task 14 clears it
    pub tiles_reset: bool,
    pub grid: HashMap<(i64, i64), Vec<usize>>,
    pub grid_stamp: f64,
    pub far_max: f64,
    pub next_id: u64,
    // prompts
    pub spore_prompt: bool,
    pub spore_prompt_hover: bool,
    pub spore_prompt_x: f64,
    pub spore_prompt_y: f64,
    pub spore_prompt_index: i64,
    pub regen_prompt: bool,
    pub regen_prompt_hover: bool,
    pub mushroom_prompt: bool,
    pub build_options: Vec<BuildOption>,
    pub build_hover: i64,
    pub build_node_x: f64,
    pub build_node_y: f64,
    pub build_surface_x: f64,
    pub build_surface_y: f64,
    pub build_surface_ok: bool,
    /// locale key of the build hint ("" for none)
    pub build_hint: String,
    // spores
    pub spores: Vec<Spore>,
    // bursts
    pub digger_queue: Vec<DiggerJob>,
    pub digger_next: usize,
    /// the digger bursts' shared "a strand already sounded" flags
    pub digger_shared: Vec<bool>,
    pub burst_queue: Vec<BurstKnot>,
    pub burst_next: usize,
    pub burst_show: Vec<BurstKnot>,
    pub burst_show_next: usize,
    pub burst_sounded: bool,
    pub burst_growing: Vec<usize>,
    // glow
    pub glow_on: bool,
    pub pulses: Vec<Pulse>,
    pub pulse_timer: f64,
    pub debug_draw: bool,
    // baked tiles (bookkeeping only)
    pub tiles: HashMap<(i64, i64), usize>,
    pub tile_list: Vec<MycTile>,
    /// `myc_frame`, advanced by Task 14's `myc_tiles_step`
    pub frame: f64,
    pub prof: BTreeMap<String, Prof>,
}

impl MyceliumRt {
    /// A fresh id for a growth (sound id, resource source).
    pub fn new_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }
}

/// `Resource` source ids: a tag in the top byte, then the growth id / knot index / mushroom.
pub mod source {
    pub const GROWTH: u64 = 1 << 56;
    pub const KNOT: u64 = 2 << 56;
    pub const MUSHROOM: u64 = 3 << 56;
    pub const ORIGIN: u64 = 4 << 56;
}

// port: anon__238__gml_Object_obj_mycelium_controller_Create_0
/// `myc_sync_view`: the view of camera 0 (the logic view, 1×).
pub fn sync_view(sim: &mut Sim) {
    let c = &sim.camera;
    let m = &mut sim.mycelium;
    m.camera_x = c.view_x;
    m.camera_y = c.view_y;
    m.view_w = c.width;
    m.view_h = c.height;
}

// port: gml_Object_obj_mycelium_controller_Create_0
/// obj_mycelium_controller Create_0: constants, the origin knot (unless the intro runs) and
/// the camera box around the origin.
pub fn create(sim: &mut Sim) {
    let surface_y = wld("surface_height_px");
    let tile = wld("reference_tile_size");
    let min_growth_len = sim.bal("growth_min_length");
    let x = sim.state.state.start_x;
    let m = &mut sim.mycelium;
    *m = MyceliumRt::default();
    m.surface_y = surface_y;
    m.tile = tile;
    m.view_w = vis("screen_width_px");
    m.view_h = vis("screen_height_px");
    sync_view(sim);
    let m = &mut sim.mycelium;
    m.x = x;
    m.y = surface_y;
    m.sample_step = 5.0;
    m.min_growth_len = min_growth_len;
    m.growth_duration = vis("growth_duration_frames");
    m.default_aim_dir = 270.0;
    m.food_pickup_radius = tile * 0.35;
    m.aim_origin_x = x;
    m.aim_origin_y = surface_y + tile;
    m.aim_dir = 270.0;
    m.knot_radius = 7.0;
    m.pick_radius = 12.0;
    m.spore_prompt_index = -1;
    m.build_hover = -1;
    m.pulse_timer = vis("myc_pulse_interval_seconds") * sim.fps;
    m.core_color = 14873582.0;
    if !sim.intro_active {
        knots::add_knot(sim, x, surface_y);
    }
    soil::reach_expand(sim, x, surface_y);
}

/// `ui_blocking()` with the app's state from `StepInput`.
pub(crate) fn ui_blocking(sim: &Sim, input: &StepInput) -> bool {
    crate::forest::ui_blocking(sim, input)
}

// port: anon__54631__gml_Object_obj_mycelium_controller_Create_0
/// `myc_prof_add(key, us)`: the debug profile of the step parts.
pub fn prof_add(sim: &mut Sim, key: &str, us: f64) {
    let e = sim.mycelium.prof.entry(key.to_string()).or_default();
    e.now = us;
    e.peak = (e.peak * 0.97).max(us);
}

fn timed(sim: &mut Sim, key: &str, f: impl FnOnce(&mut Sim)) {
    let t0 = std::time::Instant::now();
    f(sim);
    prof_add(sim, key, t0.elapsed().as_micros() as f64);
}

// port: gml_Object_obj_mycelium_controller_Step_0
/// obj_mycelium_controller Step_0. Mouse input comes from `input` (cursor, LMB, Alt); the
/// right-click quick node is `Command::QuickNode` (ignored while the LMB is held, over the HUD
/// and while the game is blocked); the in-world buttons come as `Command::MycBuildOption`,
/// `MycReleaseSpores`, `MycRegenRepeller`, the build menu as `Command::OpenBuildMenu`.
pub fn step(sim: &mut Sim, input: &StepInput) {
    sync_view(sim);
    if let Some((mx, my)) = input.cursor {
        sim.mycelium.mouse_x = mx;
        sim.mycelium.mouse_y = my;
    }
    if let Some((bx, by)) = sim.world.caves.burst_pending.take() {
        growth::node_burst(sim, bx, by);
    }
    if !ui_blocking(sim, input) {
        timed(sim, "burst", growth::node_burst_step);
        timed(sim, "digger", growth::digger_step);
    }
    if !perk_has(&sim.state, "purple") {
        if sim.mycelium.glow_on {
            glow_set(sim, false);
        }
    } else if input.keys_pressed.contains(&Key::Char('F')) && !input.slot_naming {
        let on = !sim.mycelium.glow_on;
        glow_set(sim, on);
    }
    if sim.mycelium.glow_on && !ui_blocking(sim, input) {
        sim.mycelium.pulse_timer -= 1.0;
        if sim.mycelium.pulse_timer <= 0.0 {
            sim.mycelium.pulse_timer = vis("myc_pulse_interval_seconds") * sim.fps;
            pulse_start(sim);
        }
    }
    let blocking = ui_blocking(sim, input);
    if blocking {
        knots::clear_prompts(sim);
        let m = &mut sim.mycelium;
        if m.dragging {
            m.dragging = false;
            m.has_aim = false;
            m.aim_len = 0.0;
        }
    }
    // the app hit-tests the in-world buttons (text-measured rects, ruling R9b) and reports a
    // click on one as a command: that is the original's hover + LMB press on it
    let mut build_click: i64 = -1;
    let mut spore_click = false;
    let mut regen_click = false;
    for c in &input.commands {
        match c {
            Command::MycBuildOption { index } => build_click = *index as i64,
            Command::MycReleaseSpores => spore_click = true,
            Command::MycRegenRepeller => regen_click = true,
            _ => {}
        }
    }
    {
        let m = &mut sim.mycelium;
        m.build_hover = -1;
        if m.mushroom_prompt && build_click >= 0 && (build_click as usize) < m.build_options.len() {
            m.build_hover = build_click;
        }
        m.spore_prompt_hover = m.spore_prompt && spore_click;
        m.regen_prompt_hover = m.regen_prompt && regen_click;
    }
    if !blocking && !input.ui_captured {
        // right click (the original opens the build menu on mb_right; the port's RMB is the
        // quick node, ruling: ignored while aiming)
        for c in &input.commands {
            match *c {
                Command::QuickNode { x, y } if !input.lmb.down => {
                    if let Err(reason) = knots::quick_node(sim, x, y) {
                        sim.emit(SimEvent::NodeRejected { reason, x, y });
                    }
                }
                Command::OpenBuildMenu { x, y } if !place::open_build_menu_at(sim, x, y) => {
                    knots::clear_prompts(sim);
                }
                _ => {}
            }
        }
    }
    let lmb_pressed = input.lmb.pressed && input.cursor.is_some();
    let button = build_click >= 0 || spore_click || regen_click;
    if !blocking && !input.ui_captured && (lmb_pressed || button) {
        lmb_press(sim, input, lmb_pressed);
    }
    let (mx, my) = (sim.mycelium.mouse_x, sim.mycelium.mouse_y);
    if sim.mycelium.dragging && input.lmb.down {
        let m = &sim.mycelium;
        let (ox, oy) = (m.aim_origin_x, m.aim_origin_y);
        let d = crate::gml::point_distance(ox, oy, mx, my);
        if d > 6.0 {
            let dir = crate::gml::point_direction(ox, oy, mx, my);
            let range = shot::shot_range(sim);
            let pull = (d * sim.bal("shot_pull_scale")).min(range);
            let len = place::measure_allowed_len_from(sim, ox, oy, dir, pull, false);
            let reach = place::measure_allowed_len_from(sim, ox, oy, dir, range, false);
            let cost = shot::shot_cost_for(sim, ox, oy, dir, len);
            let food = shot::shot_food_for(sim, ox, oy, dir, len);
            let m = &mut sim.mycelium;
            m.aim_dir = dir;
            m.aim_len = len;
            m.aim_reach = reach;
            m.aim_cost = cost;
            m.aim_food = food;
            m.has_aim = true;
        }
    }
    if sim.mycelium.dragging && !input.lmb.down {
        let m = &sim.mycelium;
        if m.has_aim && m.aim_len >= m.min_growth_len {
            let (ox, oy, dir, len) = (m.aim_origin_x, m.aim_origin_y, m.aim_dir, m.aim_len);
            shot::add_growth_from(sim, ox, oy, dir, len, &GrowthOpts::default());
        }
        let m = &mut sim.mycelium;
        m.dragging = false;
        m.has_aim = false;
        m.aim_len = 0.0;
    }
    // age the active growths; a finished one stays while its sound plays
    let active = std::mem::take(&mut sim.mycelium.active);
    let mut keep = Vec::with_capacity(active.len());
    for gi in active {
        let g = &mut sim.mycelium.growths[gi];
        let snd = g.sound.is_some_and(|s| !s.done);
        if !g.finished || snd {
            g.age += 1.0;
            if snd {
                let (id, t) = (g.id, g.age);
                let s = g.sound.as_mut().unwrap();
                if t >= s.plan.done_at(true) {
                    s.done = true;
                }
                sim.emit(SimEvent::Audio(AudioCall::HyphaeStep { id, t }));
            }
            keep.push(gi);
        }
    }
    sim.mycelium.active = keep;
    timed(sim, "scan", growth::scan_growths);
    // myc_tiles_step (baking) is Task 14's, driven by `dirty_growths`
    knots::update_spores(sim);
}

/// The LMB-press part of Step_0: Alt quick node, the prompt buttons, a knot (aim), a mushroom
/// (its prompt), else clear the prompts.
fn lmb_press(sim: &mut Sim, input: &StepInput, pressed: bool) {
    let (mx, my) = (sim.mycelium.mouse_x, sim.mycelium.mouse_y);
    let mut handled = false;
    if pressed && input.keys_down.contains(&Key::Alt) && knots::quick_node(sim, mx, my).is_ok() {
        handled = true;
    }
    if sim.mycelium.mushroom_prompt && sim.mycelium.build_hover >= 0 {
        let opt = sim.mycelium.build_options[sim.mycelium.build_hover as usize];
        if place::apply_build_option(sim, &opt) {
            knots::clear_prompts(sim);
        }
        handled = true;
    }
    if !handled && sim.mycelium.spore_prompt && sim.mycelium.spore_prompt_hover {
        if sim.state.state.food >= sim.bal("spore_release_cost") {
            let m = &sim.mycelium;
            let (mut sx, mut sy) = (m.spore_prompt_x, m.spore_prompt_y);
            let mut carried = false;
            let idx = m.spore_prompt_index;
            if idx >= 0 && (idx as usize) < sim.state.mushrooms.len() {
                let mu = &sim.state.mushrooms[idx as usize];
                sx = mu.x;
                sy = mu.y;
                carried = mu.carried_by != -1;
            }
            if knots::spawn_spores_from(sim, sx, sy) && carried {
                crate::progress::tasks::stat_add(sim, "ach_hedgehog_spores", 1.0);
            }
            sim.mycelium.spore_prompt = false;
        }
        handled = true;
    }
    if !handled && sim.mycelium.regen_prompt && sim.mycelium.regen_prompt_hover {
        let idx = sim.mycelium.spore_prompt_index;
        if idx >= 0 && (idx as usize) < sim.state.mushrooms.len() {
            let i = idx as usize;
            if !sim.state.mushrooms[i].active {
                let price = crate::mushrooms::repeller_regen_price(&sim.state);
                if sim.food_spend(price) {
                    let (x, y) = (sim.state.mushrooms[i].x, sim.state.mushrooms[i].y);
                    sim.emit(SimEvent::Resource {
                        kind: crate::events::ResourceKind::Food,
                        delta: -price,
                        x,
                        y,
                        source: source::MUSHROOM | i as u64,
                    });
                    let charges = crate::mushrooms::repeller_max_charges(&sim.state);
                    let m = &mut sim.state.mushrooms[i];
                    m.active = true;
                    m.charges = charges;
                }
            }
        }
        knots::clear_prompts(sim);
        handled = true;
    }
    if !handled && pressed {
        let r = sim.mycelium.knot_radius + 6.0;
        if let Some((kx, ky)) = knots::get_knot_at(sim, mx, my, r) {
            knots::start_aim_at(sim, kx, ky);
            handled = true;
        }
    }
    if !handled && pressed {
        let h = place::get_mushroom_at(sim, mx, my);
        if h.hit {
            knots::clear_prompts(sim);
            let m = &mut sim.mycelium;
            m.spore_prompt_index = h.index;
            m.spore_prompt_x = h.x;
            m.spore_prompt_y = h.y;
            let mt = if h.index >= 0 {
                sim.state.mushrooms[h.index as usize].kind
            } else {
                0
            };
            if mt == 2 {
                sim.mycelium.regen_prompt = !sim.state.mushrooms[h.index as usize].active;
            } else if mt == 0 {
                sim.mycelium.spore_prompt =
                    crate::progress::upgrades::upg_is_owned(&sim.state, "unlock_spore");
            }
            handled = true;
        }
    }
    if !handled {
        knots::clear_prompts(sim);
    }
}

/// The HUD's mushroom placement (obj_hud Step_1 calls `myc_place_mushroom` on a click in
/// placement mode): `Command::PlaceMushroom` is handled here, from `progress::hud_begin_step`.
/// Task 17 sees success as a new entry in `sim.state.mushrooms`.
pub fn apply_hud_commands(sim: &mut Sim, input: &StepInput) {
    for c in &input.commands {
        if let Command::PlaceMushroom { kind, x, y } = *c {
            place::place_mushroom(sim, kind, x, Some(y));
        }
    }
}

// port: anon__71233__gml_Object_obj_mycelium_controller_Create_0
/// `myc_glow_set(on)`: the purple perk's glow; turning it on starts a pulse right away.
pub fn myc_glow_set(sim: &mut Sim, on: bool) {
    glow_set(sim, on)
}

/// `myc_glow_set` (same as [`myc_glow_set`]).
pub fn glow_set(sim: &mut Sim, on: bool) {
    sim.mycelium.glow_on = on;
    if on {
        pulse_start(sim);
        sim.mycelium.pulse_timer = vis("myc_pulse_interval_seconds") * sim.fps;
    } else {
        sim.mycelium.pulses.clear();
    }
}

// port: anon__71572__gml_Object_obj_mycelium_controller_Create_0
/// `myc_pulse_start`: a ring that runs out past the furthest growth.
pub fn pulse_start(sim: &mut Sim) {
    let born = sim.now_ms();
    let far = sim.mycelium.far_max + vis("myc_pulse_band_px") * 2.0;
    sim.mycelium.pulses.push(Pulse { born, far });
}

// port: anon__71848__gml_Object_obj_mycelium_controller_Create_0
/// `myc_pulse_radius(p)`: the ring radius now, or -1 once it ran past `far`.
pub fn pulse_radius(sim: &Sim, p: &Pulse) -> f64 {
    let d = (sim.now_ms() - p.born) / 1000.0 * vis("myc_pulse_speed_px");
    if d <= p.far {
        d
    } else {
        -1.0
    }
}

// port: anon__81793__gml_Object_obj_mycelium_controller_Create_0
/// `myc_tile_add(e)`: file a baked growth under its 1024 px tile (Task 14's bake calls this;
/// the mesh merge is Task 14's `myc_tile_merge`).
pub fn tile_add(m: &mut MyceliumRt, gi: usize) {
    let g = &m.growths[gi];
    let (bx1, by1, bx2, by2) = (g.bx1, g.by1, g.bx2, g.by2);
    let tx = ((bx1 + bx2) * 0.5 / TILE_SIZE).floor() as i64;
    let ty = ((by1 + by2) * 0.5 / TILE_SIZE).floor() as i64;
    let frame = m.frame;
    let ti = match m.tiles.get(&(tx, ty)) {
        Some(&ti) => {
            let t = &mut m.tile_list[ti];
            t.bx1 = t.bx1.min(bx1);
            t.by1 = t.by1.min(by1);
            t.bx2 = t.bx2.max(bx2);
            t.by2 = t.by2.max(by2);
            ti
        }
        None => {
            m.tile_list.push(MycTile {
                key: (tx, ty),
                pending: Vec::new(),
                since: frame,
                bx1,
                by1,
                bx2,
                by2,
            });
            m.tiles.insert((tx, ty), m.tile_list.len() - 1);
            m.tile_list.len() - 1
        }
    };
    let t = &mut m.tile_list[ti];
    if t.pending.is_empty() {
        t.since = frame;
    }
    t.pending.push(gi);
}

// port: anon__86877__anon__86458__gml_Object_obj_mycelium_controller_Create_0
// port: anon__86458__gml_Object_obj_mycelium_controller_Create_0
/// `myc_bake_queue_all(cx, cy)`: queue every growth for baking, nearest to (cx, cy) last (so
/// popped first); nothing stays active.
pub fn bake_queue_all(m: &mut MyceliumRt, cx: f64, cy: f64) {
    m.dirty_growths.clear();
    m.active.clear();
    for (gi, g) in m.growths.iter_mut().enumerate() {
        g.bake_d = crate::gml::point_distance(cx, cy, (g.bx1 + g.bx2) * 0.5, (g.by1 + g.by2) * 0.5);
        m.dirty_growths.push(gi);
    }
    // the comparator b.bake_d - a.bake_d: farthest first (stable)
    let growths = &m.growths;
    m.dirty_growths
        .sort_by(|&a, &b| growths[b].bake_d.partial_cmp(&growths[a].bake_d).unwrap());
}

/// `load_game`'s mycelium part: stop the old sounds, rebuild the saved growths as finished
/// (unrasterized) strands, queue them for baking around the camera centre, take the knots,
/// reset aim, prompts, spores, bursts and rebuild the grid.
pub fn on_load(sim: &mut Sim) {
    // myc_tiles_clear + free every growth's GPU data (Task 14 drops them on tiles_reset)
    let old = std::mem::take(&mut sim.mycelium.growths);
    for g in &old {
        if g.sound.is_some() {
            sim.emit(SimEvent::Audio(AudioCall::HyphaeStop { id: g.id }));
        }
    }
    let save = std::mem::take(&mut sim.state.mycelium);
    let dur = sim.mycelium.growth_duration;
    let mut growths = Vec::new();
    for gs in &save.growths {
        let mut nb = Vec::new();
        let mut mx: f64 = 0.0;
        for b in &gs.branches {
            let pts: Vec<Pt> = b.points.iter().map(|p| Pt { x: p.x, y: p.y }).collect();
            let mut nbr = shot::build_branch(&pts, b.start, b.width, false);
            nbr.shoved = b.shoved;
            nbr.swept = b.swept;
            nbr.extra = b.extra.clone();
            mx = mx.max(b.start);
            nb.push(nbr);
        }
        if nb.is_empty() {
            continue;
        }
        let (mut bx1, mut by1, mut bx2, mut by2) = (nb[0].bx1, nb[0].by1, nb[0].bx2, nb[0].by2);
        for b in &nb[1..] {
            bx1 = bx1.min(b.bx1);
            by1 = by1.min(b.by1);
            bx2 = bx2.max(b.bx2);
            by2 = by2.max(b.by2);
        }
        let age = dur + mx + 2.0;
        let id = sim.mycelium.new_id();
        growths.push(Growth {
            id,
            age,
            duration: dur,
            finish_age: age,
            branches: nb,
            scanned_length: 0.0,
            finished: true,
            budding: false,
            no_length: true,
            sound: None,
            instant: false,
            bx1,
            by1,
            bx2,
            by2,
            bake_d: 0.0,
            grid_stamp: -1.0,
            extra: gs.extra.clone(),
        });
    }
    let m = &mut sim.mycelium;
    m.tiles.clear();
    m.tile_list.clear();
    m.tiles_reset = true;
    m.growths = growths;
    let cx = sim.camera.x + sim.camera.width * 0.5;
    let cy = sim.camera.y + sim.camera.height * 0.5;
    bake_queue_all(&mut sim.mycelium, cx, cy);
    sim.mycelium.knots = save.knots.iter().map(Knot::from_save).collect();
    let start_x = sim.state.state.start_x;
    let m = &mut sim.mycelium;
    m.spores.clear();
    m.dragging = false;
    m.has_aim = false;
    knots::clear_prompts(sim);
    let m = &mut sim.mycelium;
    m.x = start_x;
    growth::grid_rebuild(sim);
    let m = &mut sim.mycelium;
    m.burst_queue.clear();
    m.burst_next = 0;
    m.burst_show.clear();
    m.burst_show_next = 0;
    m.burst_growing.clear();
    m.digger_queue.clear();
    m.digger_next = 0;
    m.digger_shared.clear();
    // unknown top-level fields of the saved mycelium survive the next save
    sim.state.mycelium.extra = save.extra;
}

/// `save_game`'s mycelium part: every growth's branches (points, start, width, shoved, swept)
/// and the knots.
pub fn before_save(sim: &Sim, out: &mut GameState) {
    let m = &sim.mycelium;
    out.mycelium = MyceliumSave {
        growths: m
            .growths
            .iter()
            .map(|g| GrowthSave {
                branches: g
                    .branches
                    .iter()
                    .map(|b| BranchSave {
                        points: b
                            .points
                            .iter()
                            .map(|p| PointSave {
                                x: p.x,
                                y: p.y,
                                extra: Extra::new(),
                            })
                            .collect(),
                        start: b.start,
                        width: b.width,
                        shoved: b.shoved,
                        swept: b.swept,
                        extra: b.extra.clone(),
                    })
                    .collect(),
                extra: g.extra.clone(),
            })
            .collect(),
        knots: m.knots.iter().map(Knot::to_save).collect(),
        extra: sim.state.mycelium.extra.clone(),
    };
}

// port: mycelium_extreme_tip
/// `mycelium_extreme_tip(dir)`: the growth point furthest in direction `dir` (>= 0 right,
/// < 0 left), or None without growths. The yellow cave (`world::caves::cave_yellow_pick`) and
/// the HUD's camera-jump buttons ask it.
pub fn mycelium_extreme_tip(sim: &Sim, dir: f64) -> Option<(f64, f64)> {
    let mut best: Option<(f64, f64)> = None;
    for g in &sim.mycelium.growths {
        for b in &g.branches {
            let edge = if dir >= 0.0 { b.bx2 } else { b.bx1 };
            if best.is_none_or(|(bx, _)| (edge - bx) * dir > 0.0) {
                for p in &b.points {
                    if best.is_none_or(|(bx, _)| (p.x - bx) * dir > 0.0) {
                        best = Some((p.x, p.y));
                    }
                }
            }
        }
    }
    best
}
