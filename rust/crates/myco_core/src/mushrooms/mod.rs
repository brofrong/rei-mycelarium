//! Mushrooms (`global.mushrooms`, scr_life): building, per-frame production (predators catch
//! food, tree mushrooms drop water, experience mushrooms give upgrade points, consumptors eat
//! their host), repellers, stinky columns. The live list is `sim.state.mushrooms`; each
//! mushroom's runtime part (stable id, host, stench pool) is `MushroomSave::live`.
//!
//! - Production credits food/water/points and emits `SimEvent::Resource{kind, +n, x, y,
//!   source}` in the same place, `source = mycelium::source::MUSHROOM | id` (one event per
//!   source per frame).
//! - Placement (ruling R22): while `StepInput::placement` is `Some`, obj_hud Step_1's
//!   `myc_place_check` runs every step ([`placement_step`]) and its plan is kept in
//!   [`MushroomsRt::place_preview`] for the app to draw; `Command::PlaceMushroom` goes through
//!   [`build_at`], which emits `SimEvent::MushroomPlaced` on success.
//! - Drawing (`mushroom_draw`, `stinky_draw`, cloud art) is Task 15's.

pub mod consumptor;
pub mod repeller;
pub mod stinky;
pub mod types;

pub use consumptor::{
    consume_info, consumptor_variant, consumptors_on, set_consumptor, ConsumeInfo,
};
pub use types::{
    mushroom_type_affordable, mushroom_type_cost, mushroom_type_cost_base, mushroom_type_count,
    mushroom_type_name_key,
    mushroom_type_unlock_id, mushroom_type_unlocked, mushroom_type_water_cost,
    mushroom_type_water_cost_base, repeller_max_charges, repeller_regen_price,
};

use crate::audio;
use crate::balance::vis;
use crate::events::{ResourceKind, SimEvent};
use crate::forest::{self, HostId};
use crate::life::{self, Drop};
use crate::mycelium::{place, source};
use crate::progress::tasks;
use crate::sim::{Sim, StepInput};
use crate::state::{GameState, MushroomSave};

/// Mushroom types by their GML code (`mushroom_type_unlock_id`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MushroomKind {
    Spore,
    Predator,
    Repeller,
    Tree,
    Digger,
    Stinky,
    Experience,
    Consumptor,
}

impl MushroomKind {
    pub const ALL: [MushroomKind; 8] = [
        MushroomKind::Spore,
        MushroomKind::Predator,
        MushroomKind::Repeller,
        MushroomKind::Tree,
        MushroomKind::Digger,
        MushroomKind::Stinky,
        MushroomKind::Experience,
        MushroomKind::Consumptor,
    ];

    /// The `type` field of a mushroom (0..=7).
    pub fn code(self) -> i64 {
        self as i64
    }

    pub fn from_code(code: i64) -> Option<MushroomKind> {
        usize::try_from(code)
            .ok()
            .and_then(|i| MushroomKind::ALL.get(i).copied())
    }
}

/// The placement preview (`obj_hud.hud_place_at`): the type and `myc_place_check`'s plan.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacePreview {
    pub kind: MushroomKind,
    pub plan: place::Plan,
}

/// Runtime mushroom state outside `GameState`.
#[derive(Clone, Debug, Default)]
pub struct MushroomsRt {
    /// last stable id handed out (ids start at 1)
    pub next_id: u64,
    /// `global.repel_clouds`
    pub repel_clouds: Vec<repeller::RepelCloud>,
    /// `global.repel_kick`: camera kick of the last nearby repeller (Task 21's `boar_shake`
    /// reads and decays it)
    pub repel_kick: f64,
    /// `global.stinky_live_count` / `stinky_live_last`: stinky columns in view this / last step
    pub stinky_live_count: f64,
    pub stinky_live_last: f64,
    /// `stinky_shade_table`'s cache: (hot, deep, 257 colours)
    pub shade: Option<(f64, f64, Vec<f64>)>,
    /// the placement preview while `StepInput::placement` is set (ruling R22)
    pub place_preview: Option<PlacePreview>,
}

/// A mushroom hit box (`___struct___61/62`), inclusive pixel bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitBox {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// Give every mushroom without one a stable id.
fn assign_ids(sim: &mut Sim) {
    for m in &mut sim.state.mushrooms {
        if m.live.id == 0 {
            sim.mushrooms.next_id += 1;
            m.live.id = sim.mushrooms.next_id;
        }
    }
}

/// The stable id of mushroom `i` (0 if none was assigned yet).
pub fn mushroom_id(sim: &Sim, i: usize) -> u64 {
    sim.state.mushrooms[i].live.id
}

/// `Resource::source` of the next mushroom `mushroom_add` makes (for its build costs).
pub fn next_source(sim: &Sim) -> u64 {
    source::MUSHROOM | (sim.mushrooms.next_id + 1)
}

fn emit_resource(sim: &mut Sim, kind: ResourceKind, delta: f64, i: usize) {
    let m = &sim.state.mushrooms[i];
    let e = SimEvent::Resource {
        kind,
        delta,
        x: m.x,
        y: m.y,
        source: source::MUSHROOM | m.live.id,
    };
    sim.emit(e);
}

// port: mushroom_index_of
/// Index of the mushroom with stable id `id` in `sim.state.mushrooms`, -1 when it is gone.
pub fn mushroom_index_of(sim: &Sim, id: u64) -> i64 {
    sim.state
        .mushrooms
        .iter()
        .position(|m| m.live.id == id)
        .map_or(-1, |i| i as i64)
}

// port: mushroom_add
/// Add a mushroom of type `kind` at (x, y): a random variant, a predator's flies, the first
/// timer, the grow sound and the build stats. Returns its index.
pub fn mushroom_add(sim: &mut Sim, kind: MushroomKind, x: f64, y: f64) -> usize {
    let born = sim.now_ms();
    let variant = sim.rng.irandom(types::mushroom_type_variants(kind) - 1.0);
    let mut m = MushroomSave {
        kind: kind.code(),
        variant,
        x,
        y,
        charges: repeller_max_charges(&sim.state),
        born: Some(born),
        ..MushroomSave::default()
    };
    if kind == MushroomKind::Predator {
        let n = sim.bal("predator_fly_orbit_count");
        let mut i = 0.0;
        while i < n {
            let angle = sim.rng.random(360.0);
            let radius = sim.rng.random_range(26.0, 46.0);
            let speed = sim.rng.random_range(1.4, 2.6);
            let bob = sim.rng.random(360.0);
            m.flies.push(crate::state::FlySave {
                angle,
                radius,
                speed,
                bob,
                extra: Default::default(),
            });
            i += 1.0;
        }
        m.timer = life::predator_period(sim);
    }
    if kind == MushroomKind::Tree {
        m.timer = life::tree_drop_period(sim);
    }
    if kind == MushroomKind::Experience {
        m.timer = types::experience_period(sim);
    }
    sim.state.mushrooms.push(m);
    assign_ids(sim);
    let e = audio::mushroom_grow(&mut sim.rng, None, None);
    sim.emit(e);
    tasks::stat_add(sim, "built", 1.0);
    if kind == MushroomKind::Tree || kind == MushroomKind::Consumptor {
        tasks::stat_add(sim, "tree_built", 1.0);
    }
    if kind == MushroomKind::Stinky {
        tasks::stat_add(sim, "stinky_built", 1.0);
    }
    if kind == MushroomKind::Predator {
        tasks::stat_add(sim, "ach_predators", 1.0);
    }
    sim.state.mushrooms.len() - 1
}

// port: mushroom_hit_box
/// Hit box of mushroom `i`, mirrored around its foot when it faces left (`m.face`).
pub fn mushroom_hit_box(sim: &Sim, i: usize) -> HitBox {
    let mut b = mushroom_hit_box_unmirrored(sim, i);
    let m = &sim.state.mushrooms[i];
    let face = m.extra.get("face").and_then(|v| v.as_f64()).unwrap_or(1.0);
    if face < 0.0 {
        let x1 = b.x1;
        b.x1 = 2.0 * m.x - b.x2;
        b.x2 = 2.0 * m.x - x1;
    }
    b
}

// port: mushroom_hit_box_unmirrored
/// Hit box of mushroom `i` from its sheet: one variant cell wide for multi-cell sheets, else the
/// sprite's bounding box.
pub fn mushroom_hit_box_unmirrored(sim: &Sim, i: usize) -> HitBox {
    let m = &sim.state.mushrooms[i];
    let name = types::sprite_of_code(m.kind);
    let s = types::shroom_sprite(name).expect("mushroom sprite");
    let top = m.y + s.bbox_t - s.yoff;
    let bot = top + (s.bbox_b - s.bbox_t);
    if types::shroom_sheet_cells(name) > 1.0 {
        let left = m.x - s.xoff;
        return HitBox {
            x1: left,
            y1: top,
            x2: left + types::mushroom_cell_width(name) - 1.0,
            y2: bot,
        };
    }
    let left = m.x + s.bbox_l - s.xoff;
    HitBox {
        x1: left,
        y1: top,
        x2: left + (s.bbox_r - s.bbox_l),
        y2: bot,
    }
}

// port: mushrooms_step
/// One frame of every mushroom: flashes, repeller autoregen, predators (food), stinky columns,
/// consumptors (eating the host; finished hosts pay out after the loop), experience mushrooms
/// (upgrade points) and tree mushrooms (water drops).
pub fn step(sim: &mut Sim) {
    sim.mushrooms.stinky_live_last = sim.mushrooms.stinky_live_count;
    sim.mushrooms.stinky_live_count = 0.0;
    assign_ids(sim);
    let sv = life::squirrel_view(sim);
    let mut finished: Vec<HostId> = Vec::new();
    let mut i = 0;
    while i < sim.state.mushrooms.len() {
        if sim.state.mushrooms[i].flash > 0.0 {
            sim.state.mushrooms[i].flash -= 1.0;
        }
        let kind = MushroomKind::from_code(sim.state.mushrooms[i].kind);
        if kind == Some(MushroomKind::Repeller) {
            let regen = types::repeller_autoregen_frames(sim);
            let max_charges = repeller_max_charges(&sim.state);
            let m = &mut sim.state.mushrooms[i];
            if m.active || regen < 0.0 {
                m.regen_timer = Some(0.0);
            } else {
                let rt = m.regen_timer.unwrap_or(0.0) + 1.0;
                m.regen_timer = Some(rt);
                if rt >= regen {
                    m.regen_timer = Some(0.0);
                    m.active = true;
                    m.charges = max_charges;
                }
            }
        }
        match kind {
            Some(MushroomKind::Predator) => {
                let period = life::predator_period(sim);
                let m = &mut sim.state.mushrooms[i];
                for f in &mut m.flies {
                    f.angle += f.speed;
                    f.bob += 3.0;
                }
                let mut caught = false;
                if m.carried_by == -1 && !m.stolen {
                    m.timer -= 1.0;
                    if m.timer <= 0.0 {
                        m.timer = period;
                        caught = true;
                    }
                } else {
                    m.timer = period;
                }
                if caught {
                    let y = life::predator_yield(sim);
                    sim.food_add(y);
                    emit_resource(sim, ResourceKind::Food, y, i);
                    tasks::stat_add(sim, "food", y);
                }
            }
            Some(MushroomKind::Stinky) => stinky::stinky_step(sim, i, &sv),
            Some(MushroomKind::Consumptor) => {
                let m = &sim.state.mushrooms[i];
                if let (false, Some(h)) = (m.stolen, m.live.host) {
                    if let Some((c, false)) = consumptor::host_consume(sim, h) {
                        let c = c + 1.0 / consumptor::consume_frames(sim, h).max(1.0);
                        let done = c >= 1.0;
                        consumptor::set_host_consume(sim, h, c, done);
                        if done {
                            finished.push(h);
                        }
                    }
                }
            }
            Some(MushroomKind::Experience) => {
                let m = &sim.state.mushrooms[i];
                if !m.stolen && m.carried_by == -1 {
                    let period = types::experience_period(sim);
                    let m = &mut sim.state.mushrooms[i];
                    m.timer -= 1.0;
                    if m.timer <= 0.0 {
                        m.timer = period;
                        sim.state.state.upgrade_points += 1;
                        emit_resource(sim, ResourceKind::UpgradePoints, 1.0, i);
                        if crate::weather::is_raining(sim) {
                            tasks::stat_add(sim, "rain_points", 1.0);
                        }
                        sim.state.mushrooms[i].flash = vis("experience_flash_frames");
                    }
                }
            }
            Some(MushroomKind::Tree) if !sim.state.mushrooms[i].stolen => {
                let period = life::tree_drop_period(sim);
                let m = &mut sim.state.mushrooms[i];
                m.timer -= 1.0;
                // `not (timer > 0)`, as written (a NaN timer drops too)
                #[allow(clippy::neg_cmp_op_on_partial_ord)]
                let due = !(m.timer > 0.0);
                if due {
                    m.timer = period;
                    let d = Drop {
                        x: m.x + 18.0,
                        y: m.y - 150.0,
                        target_y: m.y - 4.0,
                        speed: 0.0,
                        source: source::MUSHROOM | m.live.id,
                    };
                    sim.life.drops.push(d);
                }
            }
            _ => {}
        }
        i += 1;
    }
    for h in finished {
        consumptor::consume_finish(sim, h);
    }
}

/// `load_game`'s mushroom part: hosts from their save ids, fresh runtime state.
pub fn on_load(sim: &mut Sim) {
    sim.mushrooms.repel_clouds.clear();
    sim.mushrooms.place_preview = None;
    for m in &mut sim.state.mushrooms {
        m.live = Default::default();
        if m.host >= 0 {
            m.live.host = sim.forest.loaded_ids.get(&m.host).copied();
        }
    }
    assign_ids(sim);
}

/// `save_game`'s mushroom part: each host as its save id (-1 for none).
pub fn before_save(sim: &Sim, out: &mut GameState) {
    let ids = forest::save_ids(&sim.forest);
    for m in &mut out.mushrooms {
        m.host = m
            .live
            .host
            .and_then(|h| match h {
                HostId::Tree(u) | HostId::Stump(u) => ids.get(&u).copied(),
            })
            .unwrap_or(-1);
        m.live = Default::default();
    }
}

// ---------------------------------------------------------------------------------------------
// The build API the app uses.

/// `mushroom_type_cost` (food) of a type.
pub fn cost(st: &GameState, kind: MushroomKind) -> f64 {
    mushroom_type_cost(st, kind)
}

/// `mushroom_type_water_cost` of a type.
pub fn water_cost(st: &GameState, kind: MushroomKind) -> f64 {
    mushroom_type_water_cost(st, kind)
}

/// `mushroom_type_affordable`.
pub fn affordable(sim: &Sim, kind: MushroomKind) -> bool {
    mushroom_type_affordable(sim, kind)
}

/// `mushroom_type_unlocked`.
pub fn unlocked(st: &GameState, kind: MushroomKind) -> bool {
    mushroom_type_unlocked(st, kind)
}

/// Why a mushroom cannot be built at a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildReject {
    /// the type's unlock upgrade is not owned
    Locked,
    /// not enough food or water
    NotAffordable,
    /// no water at all (`myc_has_water_source`)
    NoWaterSource,
    /// no host or tree, too close to another mushroom, or no mycelium within reach
    NoSpot,
}

/// Whether a mushroom of `kind` can be built for a cursor at (x, y): `myc_place_check`'s
/// verdict, with the reason when it says no.
pub fn can_build_at(sim: &mut Sim, kind: MushroomKind, x: f64, y: f64) -> Result<(), BuildReject> {
    if !unlocked(&sim.state, kind) {
        return Err(BuildReject::Locked);
    }
    if !affordable(sim, kind) {
        return Err(BuildReject::NotAffordable);
    }
    if !place::has_water_source(sim) {
        return Err(BuildReject::NoWaterSource);
    }
    if place::place_check(sim, kind, x, Some(y)).ok {
        Ok(())
    } else {
        Err(BuildReject::NoSpot)
    }
}

/// `Command::PlaceMushroom`: `myc_place_mushroom(kind, x, y)`; on success emits
/// `SimEvent::MushroomPlaced` at the new mushroom (the HUD then closes the build bar, like
/// `hud_build_close` after a successful place).
pub fn build_at(sim: &mut Sim, kind: MushroomKind, x: f64, y: f64) -> bool {
    let n = sim.state.mushrooms.len();
    if !place::place_mushroom(sim, kind, x, Some(y)) {
        return false;
    }
    let (mx, my) = sim.state.mushrooms.get(n).map_or((x, y), |m| (m.x, m.y));
    sim.emit(SimEvent::MushroomPlaced { kind, x: mx, y: my });
    true
}

/// obj_hud Step_1's placement preview (ruling R22): `myc_place_check` for the cursor every step
/// while the HUD is in placement mode, kept for the app to draw.
pub fn placement_step(sim: &mut Sim, input: &StepInput) {
    sim.mushrooms.place_preview = match (input.placement, input.cursor) {
        (Some(kind), Some((cx, cy))) => Some(PlacePreview {
            kind,
            plan: place::place_check(sim, kind, cx, Some(cy)),
        }),
        _ => None,
    };
}

/// The hover-bar progress of mushroom `i` by type, as `hover_bar_step_draw` picks it
/// (`predator_progress`, `tree_shroom_progress`, `experience_progress`, `consumptor_progress`,
/// `repeller_regen_progress`); `None`: no bar.
pub fn hover_progress(sim: &Sim, i: usize) -> Option<f64> {
    match MushroomKind::from_code(sim.state.mushrooms[i].kind)? {
        MushroomKind::Predator => life::predator_progress(sim, i),
        MushroomKind::Tree => types::tree_shroom_progress(sim, i),
        MushroomKind::Experience => types::experience_progress(sim, i),
        MushroomKind::Consumptor => consumptor::consumptor_progress(sim, i),
        MushroomKind::Repeller => types::repeller_regen_progress(sim, i),
        _ => None,
    }
}
