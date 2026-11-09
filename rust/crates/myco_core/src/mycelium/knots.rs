//! Knots (nodes): adding them, the quick node (right click), spore knots with their
//! smallshrooms, spores themselves, aim start and the prompt reset.

use super::{place, shot, soil, source};
use crate::audio;
use crate::balance::vis;
use crate::events::{NodeReject, ResourceKind, SimEvent};
use crate::gml;
use crate::progress::upgrades::upg_bonus;
use crate::progress::{tasks, tutorial};
use crate::sim::Sim;
use crate::state::{Extra, PointSave};
use serde_json::{json, Value};

/// The smallshroom on a spore knot (`___struct___470`): sheet and frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Smallshroom {
    pub kind: i64,
    pub cell: i64,
}

/// A knot (`___struct___467`, or `___struct___472` for a spore knot with its smallshroom).
#[derive(Clone, Debug, PartialEq)]
pub struct Knot {
    pub x: f64,
    pub y: f64,
    pub shroom: Option<Smallshroom>,
    /// `current_time` (ms, sim clock) the spore knot appeared
    pub born: Option<f64>,
    pub extra: Extra,
}

impl Knot {
    pub fn at(x: f64, y: f64) -> Knot {
        Knot {
            x,
            y,
            shroom: None,
            born: None,
            extra: Extra::new(),
        }
    }

    pub fn from_save(p: &PointSave) -> Knot {
        let mut extra = p.extra.clone();
        let shroom = extra.remove("shroom").and_then(|v| {
            Some(Smallshroom {
                kind: v.get("kind")?.as_f64()? as i64,
                cell: v.get("cell")?.as_f64()? as i64,
            })
        });
        let born = extra.remove("born").and_then(|v| v.as_f64());
        Knot {
            x: p.x,
            y: p.y,
            shroom,
            born,
            extra,
        }
    }

    pub fn to_save(&self) -> PointSave {
        let mut extra = self.extra.clone();
        if let Some(s) = self.shroom {
            extra.insert("shroom".into(), json!({ "kind": s.kind, "cell": s.cell }));
        }
        if let Some(b) = self.born {
            extra.insert("born".into(), Value::from(b));
        }
        PointSave {
            x: self.x,
            y: self.y,
            extra,
        }
    }
}

// port: anon__8392__gml_Object_obj_mycelium_controller_Create_0
/// `myc_clear_prompts`.
pub fn clear_prompts(sim: &mut Sim) {
    let m = &mut sim.mycelium;
    m.spore_prompt = false;
    m.spore_prompt_index = -1;
    m.regen_prompt = false;
    m.mushroom_prompt = false;
    m.build_options.clear();
    m.build_hover = -1;
    m.build_hint.clear();
}

// port: anon__8801__gml_Object_obj_mycelium_controller_Create_0
/// `myc_start_aim_at(ox, oy)`: start dragging a shot from a knot.
pub fn start_aim_at(sim: &mut Sim, ox: f64, oy: f64) {
    tutorial::event(sim, "aim", Some(ox), Some(oy));
    let m = &mut sim.mycelium;
    m.dragging = true;
    m.has_aim = true;
    m.aim_origin_x = ox;
    m.aim_origin_y = oy;
    m.aim_dir = m.default_aim_dir;
    m.aim_len = 0.0;
    m.aim_reach = 0.0;
    m.aim_cost = 0.0;
    m.aim_food = 0.0;
    m.drag_start_x = m.mouse_x;
    m.drag_start_y = m.mouse_y;
    clear_prompts(sim);
}

// port: anon__9236__gml_Object_obj_mycelium_controller_Create_0
pub fn has_knot_near(sim: &Sim, px: f64, py: f64, r: f64) -> bool {
    sim.mycelium
        .knots
        .iter()
        .any(|k| gml::point_distance(px, py, k.x, k.y) <= r)
}

// port: anon__9512__gml_Object_obj_mycelium_controller_Create_0
/// `myc_add_knot(px, py)`: a node, unless one is within 1.6 knot radii or the point shows water.
pub fn add_knot(sim: &mut Sim, px: f64, py: f64) -> bool {
    let r = sim.mycelium.knot_radius * 1.6;
    if has_knot_near(sim, px, py, r) {
        return false;
    }
    if sim.world.water_shown_at(px, py) {
        return false;
    }
    sim.mycelium.knots.push(Knot::at(px, py));
    soil::reach_expand(sim, px, py);
    true
}

// port: anon__19580__gml_Object_obj_mycelium_controller_Create_0
/// `myc_node_quick(px, py)`, the right-click quick node: snap to the mycelium within the pick
/// radius and buy a node there. Rejection reasons in the original's check order. On success
/// emits `Resource{Food, -cost}` at the node and the tutorial event "node".
pub fn quick_node(sim: &mut Sim, px: f64, py: f64) -> Result<(), NodeReject> {
    let r = sim.mycelium.pick_radius;
    let hit = super::growth::get_mycelium_hit(sim, px, py, r);
    if !hit.hit {
        return Err(NodeReject::NotOnMycelium);
    }
    if sim.world.water_shown_at(hit.x, hit.y) {
        return Err(NodeReject::InWater);
    }
    let r2 = sim.mycelium.knot_radius * 2.0;
    if has_knot_near(sim, hit.x, hit.y, r2) {
        return Err(NodeReject::TooCloseToNode);
    }
    clear_prompts(sim);
    apply_node_at(sim, hit.x, hit.y)
}

/// The node branch of `myc_apply_build_option` at (x, y) (the build node position): pay and
/// add the knot. `NotEnoughFood` / `Blocked` (add_knot refused) on failure.
pub fn apply_node_at(sim: &mut Sim, x: f64, y: f64) -> Result<(), NodeReject> {
    sim.mycelium.build_node_x = x;
    sim.mycelium.build_node_y = y;
    let cost = shot::node_food_cost(sim);
    if sim.state.state.food < cost {
        return Err(NodeReject::NotEnoughFood { need: cost as i64 });
    }
    if !add_knot(sim, x, y) {
        return Err(NodeReject::Blocked);
    }
    sim.food_spend(cost);
    let k = (sim.mycelium.knots.len() - 1) as u64;
    sim.emit(SimEvent::Resource {
        kind: ResourceKind::Food,
        delta: -cost,
        x,
        y,
        source: source::KNOT | k,
    });
    tutorial::event(sim, "node", Some(x), Some(y));
    Ok(())
}

/// A smallshroom sheet (`___struct___468/469`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmallshroomSheet {
    pub sprite: &'static str,
    pub cols: f64,
    pub rows: f64,
    /// sprite size (sprites.json)
    pub w: f64,
    pub h: f64,
}

// port: anon__10006__gml_Object_obj_mycelium_controller_Create_0
pub fn smallshroom_sheets() -> &'static [SmallshroomSheet; 2] {
    const S: [SmallshroomSheet; 2] = [
        SmallshroomSheet {
            sprite: "smallshroom1",
            cols: 9.0,
            rows: 1.0,
            w: 144.0,
            h: 48.0,
        },
        SmallshroomSheet {
            sprite: "smallshroom3",
            cols: 9.0,
            rows: 1.0,
            w: 144.0,
            h: 48.0,
        },
    ];
    &S
}

// port: anon__10257__gml_Object_obj_mycelium_controller_Create_0
pub fn smallshroom_new(sim: &mut Sim) -> Smallshroom {
    let sheets = smallshroom_sheets();
    let kind = sim.rng.irandom(sheets.len() as f64 - 1.0);
    let s = sheets[kind as usize];
    let cell = sim.rng.irandom(s.cols * s.rows - 1.0);
    Smallshroom { kind, cell }
}

/// The frame of a smallshroom in its sheet (`___struct___471`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SheetRect {
    pub sprite: &'static str,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

// port: anon__10512__gml_Object_obj_mycelium_controller_Create_0
pub fn smallshroom_rect(v: &Smallshroom) -> SheetRect {
    let sheet = smallshroom_sheets()[v.kind as usize];
    let fw = (sheet.w / sheet.cols).trunc();
    let fh = (sheet.h / sheet.rows).trunc();
    let cell = v.cell as f64;
    SheetRect {
        sprite: sheet.sprite,
        left: gml::gmod(cell, sheet.cols) * fw,
        top: (cell / sheet.cols).trunc() * fh,
        width: fw,
        height: fh,
    }
}

// port: anon__10910__gml_Object_obj_mycelium_controller_Create_0
pub fn has_smallshroom_near(sim: &Sim, px: f64, gap: f64) -> bool {
    sim.mycelium
        .knots
        .iter()
        .any(|k| k.shroom.is_some() && (k.x - px).abs() <= gap)
}

// port: anon__11304__gml_Object_obj_mycelium_controller_Create_0
/// `myc_add_spore_knot(px)`: a rooted spore becomes a knot with a smallshroom on the surface.
pub fn add_spore_knot(sim: &mut Sim, px: f64) -> bool {
    let sy = sim.mycelium.surface_y;
    if !sim.world.is_passable(px, sy + 4.0) {
        return false;
    }
    if has_smallshroom_near(sim, px, sim.bal("spore_shroom_gap_px")) {
        return false;
    }
    if has_knot_near(sim, px, sy, sim.mycelium.knot_radius * 1.6) {
        return false;
    }
    let now = sim.now_ms();
    let shroom = smallshroom_new(sim);
    sim.mycelium.knots.push(Knot {
        x: px,
        y: sy,
        shroom: Some(shroom),
        born: Some(now),
        extra: Extra::new(),
    });
    let ev = audio::mushroom_grow(&mut sim.rng, Some((px, sy)), Some(vis("smallshroom_grow_volume")));
    sim.emit(ev);
    soil::reach_expand(sim, px, sy);
    true
}

// port: anon__11972__gml_Object_obj_mycelium_controller_Create_0
pub fn add_root_knot_for_mushroom(sim: &mut Sim, px: f64, py: f64) {
    add_knot(sim, px, py);
}

// port: anon__12632__gml_Object_obj_mycelium_controller_Create_0
/// `myc_get_knot_at(px, py, r)`: the first knot within `r`.
pub fn get_knot_at(sim: &Sim, px: f64, py: f64, r: f64) -> Option<(f64, f64)> {
    sim.mycelium
        .knots
        .iter()
        .find(|k| gml::point_distance(px, py, k.x, k.y) <= r)
        .map(|k| (k.x, k.y))
}

// port: anon__27346__gml_Object_obj_mycelium_controller_Create_0
/// Chance that a landing spore roots.
pub fn spore_root_chance(sim: &Sim) -> f64 {
    let bonus = upg_bonus(&sim.state, "spore_quality");
    gml::clamp(sim.bal("spore_base_node_chance") + bonus, 0.0, 1.0)
}

/// A spore (`___struct___490/491/492`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spore {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub weight: f64,
    pub age: f64,
    pub life: f64,
    pub phase: f64,
    /// only for show (migration): never roots
    pub show: bool,
    /// a migration ray particle (no gravity, drifts)
    pub ray: bool,
}

/// A migration ray (`a` direction, `power`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub a: f64,
    pub power: f64,
}

fn wind_push(sim: &Sim) -> f64 {
    let w = &sim.state.state.wind;
    w[0] * w[1]
}

// port: anon__27471__gml_Object_obj_mycelium_controller_Create_0
/// `myc_spawn_spores_from(px, py)`: pay the release cost (emits `Resource{Food, -n}`) and
/// release spores above the mushroom.
pub fn spawn_spores_from(sim: &mut Sim, px: f64, py: f64) -> bool {
    let cost = sim.bal("spore_release_cost");
    if !sim.food_spend(cost) {
        return false;
    }
    sim.emit(SimEvent::Resource {
        kind: ResourceKind::Food,
        delta: -cost,
        x: px,
        y: py,
        source: source::ORIGIN | ((px.to_bits() ^ py.to_bits()) & ((1 << 56) - 1)),
    });
    tasks::stat_add(sim, "spores", 1.0);
    let count_bonus = upg_bonus(&sim.state, "spore_count");
    let count = gml::round(sim.bal("spore_base_count") * (1.0 + count_bonus));
    let light = (1.0 - upg_bonus(&sim.state, "spore_lightness")).max(0.15);
    let weight = sim.bal("spore_base_weight") * light;
    let wind = wind_push(sim);
    let mut i = 0.0;
    while i < count {
        let r = &mut sim.rng;
        let sx = px + r.random_range(-9.0, 9.0);
        let sy = py - r.random_range(42.0, 64.0);
        let svx = r.random_range(-0.42, 0.42) + wind * 0.09;
        let svy = r.random_range(-1.55, -0.35);
        let life = r.irandom_range(620.0, 920.0) as f64;
        let phase = r.random(360.0);
        sim.mycelium.spores.push(Spore {
            x: sx,
            y: sy,
            vx: svx,
            vy: svy,
            weight,
            age: 0.0,
            life,
            phase,
            show: false,
            ray: false,
        });
        i += 1.0;
    }
    true
}

// port: anon__28384__gml_Object_obj_mycelium_controller_Create_0
/// `myc_spawn_spores_show(px, py, count, power)`: show-only spores (the migration).
pub fn spawn_spores_show(sim: &mut Sim, px: f64, py: f64, count: f64, power: f64) {
    let weight =
        sim.bal("spore_base_weight") * (1.0 - upg_bonus(&sim.state, "spore_lightness")).max(0.15);
    let wind = wind_push(sim);
    let mut i = 0.0;
    while i < count {
        let r = &mut sim.rng;
        let x = px + r.random_range(-9.0, 9.0) * power;
        let y = py + r.random_range(-6.0, 6.0);
        let vx = r.random_range(-0.42, 0.42) * power * 1.6 + wind * 0.09;
        let vy = r.random_range(-1.55, -0.35) * power;
        let life = r.irandom_range(620.0, 920.0) as f64;
        let phase = r.random(360.0);
        sim.mycelium.spores.push(Spore {
            x,
            y,
            vx,
            vy,
            weight,
            age: 0.0,
            life,
            phase,
            show: true,
            ray: false,
        });
        i += 1.0;
    }
}

// port: anon__29233__gml_Object_obj_mycelium_controller_Create_0
/// `myc_spawn_spores_rays(px, py, count, rays, mult)`: ray particles along random rays.
pub fn spawn_spores_rays(sim: &mut Sim, px: f64, py: f64, count: f64, rays: &[Ray], mult: f64) {
    let mut i = 0.0;
    while i < count {
        let r = &mut sim.rng;
        let ray = rays[r.irandom(rays.len() as f64 - 1.0) as usize];
        let dir = ray.a + r.random_range(-2.5, 2.5);
        let spd = mult * ray.power * r.random_range(0.15, 1.0);
        let x = px + r.random_range(-4.0, 4.0);
        let y = py + r.random_range(-4.0, 4.0);
        let life = r.irandom_range(420.0, 700.0) as f64;
        let phase = r.random(360.0);
        sim.mycelium.spores.push(Spore {
            x,
            y,
            vx: gml::lengthdir_x(spd, dir),
            vy: gml::lengthdir_y(spd, dir),
            weight: 1.0,
            age: 0.0,
            life,
            phase,
            show: true,
            ray: true,
        });
        i += 1.0;
    }
}

// port: anon__29888__gml_Object_obj_mycelium_controller_Create_0
/// `myc_update_spores`: move the spores; a real spore crossing the surface may root as a
/// spore knot; dead or landed ones go.
pub fn update_spores(sim: &mut Sim) {
    let root_chance = spore_root_chance(sim);
    let wind = wind_push(sim);
    let sy = sim.mycelium.surface_y;
    let tile = sim.mycelium.tile;
    let spores = std::mem::take(&mut sim.mycelium.spores);
    let mut keep = Vec::with_capacity(spores.len());
    for mut s in spores {
        let oy = s.y;
        s.age += 1.0;
        s.life -= 1.0;
        if s.ray {
            s.vx *= 0.972;
            s.vy = s.vy * 0.972 + 0.004;
            s.x += s.vx + gml::lengthdir_x(0.2, s.phase + s.age * 5.0);
            s.y += s.vy;
            if s.life > 0.0 && s.y < sy {
                keep.push(s);
            }
        } else {
            s.vx += wind * 0.009;
            s.vx = gml::clamp(s.vx, -2.2, 2.2);
            s.vy += 0.014 * s.weight;
            s.vy = s.vy.min(1.05 * s.weight);
            s.x += s.vx + gml::lengthdir_x(0.36, s.phase + s.age * 5.0);
            s.y += s.vy;
            if oy < sy && s.y >= sy {
                if !s.show && sim.rng.random(1.0) < root_chance {
                    add_spore_knot(sim, s.x);
                }
            } else if s.life > 0.0 && s.y < sy + tile {
                keep.push(s);
            }
        }
    }
    // add_spore_knot cannot add spores, so the list is simply replaced
    sim.mycelium.spores = keep;
}

/// Re-exported for the Step's build prompt.
pub use place::apply_build_option;
