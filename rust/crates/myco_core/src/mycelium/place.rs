//! Where the mycelium may grow (playability), the in-world build menu, surface building and
//! the mushroom placement checks of the HUD's placement mode.
//!
//! Mushroom data and art belong to Task 9 (and squirrel trunk edges to Task 10): the calls
//! into them go through the clearly marked hook section of `crate::mushrooms`.
// `!(a < b)` keeps the original's behaviour for NaN
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use super::{growth, knots, soil};
use crate::balance::vis;
use crate::events::{ResourceKind, SimEvent};
use crate::forest::{stump_parts, stump_slot_at, HostId};
use crate::gml;
use crate::mushrooms::{self, MushroomKind};
use crate::progress::upgrades::{upg_is_owned, upg_level, upg_max_level};
use crate::progress::{tasks, tutorial};
use crate::sim::Sim;
use crate::world::{caves, water, Tile};

// port: anon__1762__gml_Object_obj_mycelium_controller_Create_0
pub fn has_water_source(sim: &Sim) -> bool {
    sim.water_total() > 0.0
}

// port: anon__1897__gml_Object_obj_mycelium_controller_Create_0
/// `myc_surface_block_reason(px, py)`: the hint key why nothing can be built here, or "".
pub fn surface_block_reason(sim: &Sim, px: f64, py: f64) -> &'static str {
    if !has_water_source(sim) {
        return "hint_needs_water";
    }
    if has_mushroom_near(sim, px, py, sim.mycelium.tile * 2.2) {
        return "hint_too_crowded";
    }
    ""
}

// port: anon__2138__gml_Object_obj_mycelium_controller_Create_0
/// Stone blocks growth.
pub fn is_blocked_point(sim: &mut Sim, tx: f64, ty: f64) -> bool {
    sim.world.tile_at(tx, ty) == Tile::Stone
}

// port: anon__2406__gml_Object_obj_mycelium_controller_Create_0
/// Underground, not stone (`cave`: piercing growths pass stone but not cave walls).
pub fn is_playable_point(sim: &mut Sim, px: f64, py: f64, cave: bool) -> bool {
    if !cave && is_blocked_point(sim, px, py) {
        return false;
    }
    if cave && caves::world_cave_solid_any(&mut sim.world, px, py) {
        return false;
    }
    sim.world.tile_at(px, py) != Tile::Air
}

// port: anon__2806__gml_Object_obj_mycelium_controller_Create_0
/// Playable, and (without `cave`) no stone `r` px left, right, above or below.
pub fn is_playable_clear_point(sim: &mut Sim, px: f64, py: f64, r: f64, cave: bool) -> bool {
    if !is_playable_point(sim, px, py, cave) {
        return false;
    }
    if cave {
        return true;
    }
    !is_blocked_point(sim, px - r, py)
        && !is_blocked_point(sim, px + r, py)
        && !is_blocked_point(sim, px, py - r)
        && !is_blocked_point(sim, px, py + r)
}

// port: anon__3225__gml_Object_obj_mycelium_controller_Create_0
/// `myc_segment_is_playable(x1, y1, x2, y2, r, cave)`: clear points every `sample_step` and at
/// the end. (`myc_place_check` passes the GML `true` as `r`, i.e. 1.)
pub fn segment_is_playable(
    sim: &mut Sim,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    r: f64,
    cave: bool,
) -> bool {
    let dist = gml::point_distance(x1, y1, x2, y2);
    if dist <= 0.0 {
        return is_playable_clear_point(sim, x1, y1, r, cave);
    }
    let step = sim.mycelium.sample_step;
    let mut d = 0.0;
    while d < dist {
        let t = d / dist;
        if !is_playable_clear_point(sim, gml::lerp(x1, x2, t), gml::lerp(y1, y2, t), r, cave) {
            return false;
        }
        d += step;
    }
    is_playable_clear_point(sim, x2, y2, r, cave)
}

// port: anon__4008__gml_Object_obj_mycelium_controller_Create_0
/// `myc_measure_allowed_len_from(px, py, dir, maxlen, cave)`: how far a shot gets: clear
/// points every `sample_step`, paying `travel_cost` per step (clay and deep soil cost more,
/// soil penetration helps) out of a budget of `maxlen`.
pub fn measure_allowed_len_from(
    sim: &mut Sim,
    px: f64,
    py: f64,
    dir: f64,
    maxlen: f64,
    cave: bool,
) -> f64 {
    let pen = soil::soil_penetration(sim);
    let r = 2.0;
    let budget = maxlen.max(0.0);
    let mut best = 0.0;
    let mut used = 0.0;
    if !is_playable_clear_point(sim, px, py, r, cave) {
        return 0.0;
    }
    let step = sim.mycelium.sample_step;
    let mut d = step;
    while used < budget {
        let sx = px + gml::lengthdir_x(d, dir);
        let sy = py + gml::lengthdir_y(d, dir);
        if !is_playable_clear_point(sim, sx, sy, r, cave) {
            break;
        }
        used += step * sim.world.travel_cost(sx, sy, pen);
        best = d;
        d += step;
    }
    best
}

/// An option of the in-world build menu (`___struct___481` node / `___struct___482` shroom).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildOption {
    Node,
    Shroom(MushroomKind),
}

/// A build button's label, formatted by the app with its locale: see [`OptionLabel::text`].
#[derive(Clone, Debug, PartialEq)]
pub struct OptionLabel {
    /// locale key of the name (`button_add_node` or `upg_unlock_*`)
    pub key: String,
    /// shown upper-case (mushroom names)
    pub upper: bool,
    pub cost: f64,
}

impl OptionLabel {
    /// `<name>  <cost> <hud_food>` as the original builds it.
    pub fn text(&self, loc: &dyn Fn(&str) -> String) -> String {
        let name = loc(&self.key);
        let name = if self.upper {
            name.to_uppercase()
        } else {
            name
        };
        format!(
            "{}  {} {}",
            name,
            soil::gml_string(self.cost),
            loc("hud_food")
        )
    }
}

// port: anon__6802__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_option_label(opt)` (the text is assembled by the app, which owns the locale).
pub fn build_option_label(sim: &Sim, opt: &BuildOption) -> OptionLabel {
    match opt {
        BuildOption::Node => OptionLabel {
            key: "button_add_node".into(),
            upper: false,
            cost: super::shot::node_food_cost(sim),
        },
        BuildOption::Shroom(t) => OptionLabel {
            key: mushrooms::mushroom_type_name_key(*t),
            upper: true,
            cost: mushrooms::mushroom_type_cost(&sim.state, *t),
        },
    }
}

/// The origin mushroom's box (`___struct___475`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// `originshroom` bbox (left, top, right, bottom) and origin (sprites.json).
const ORIGINSHROOM_BBOX: [f64; 4] = [10.0, 9.0, 48.0, 40.0];
const ORIGINSHROOM_ORIGIN: [f64; 2] = [39.0, 41.0];

// port: anon__13119__gml_Object_obj_mycelium_controller_Create_0
pub fn origin_box(sim: &Sim) -> Rect {
    let [l, t, r, b] = ORIGINSHROOM_BBOX;
    let x1 = sim.mycelium.x.floor() + l - ORIGINSHROOM_ORIGIN[0];
    let y1 = sim.mycelium.y.floor() + t - ORIGINSHROOM_ORIGIN[1];
    Rect {
        x1,
        y1,
        x2: x1 + r - l,
        y2: y1 + b - t,
    }
}

fn in_rect(px: f64, py: f64, r: &Rect) -> bool {
    px >= r.x1 && px <= r.x2 && py >= r.y1 && py <= r.y2
}

// port: anon__12049__gml_Object_obj_mycelium_controller_Create_0
/// The origin or a mushroom (not tree mushrooms or consumptors) within `r`.
pub fn has_mushroom_near(sim: &Sim, px: f64, py: f64, r: f64) -> bool {
    if gml::point_distance(px, py, sim.mycelium.x, sim.mycelium.y) <= r {
        return true;
    }
    sim.state
        .mushrooms
        .iter()
        .any(|m| !(m.kind == 3 || m.kind == 7) && gml::point_distance(px, py, m.x, m.y) <= r)
}

/// A mushroom under the cursor (`___struct___476/477/478`): `index` -1 is the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MushroomHit {
    pub hit: bool,
    pub index: i64,
    pub x: f64,
    pub y: f64,
}

// port: anon__13668__gml_Object_obj_mycelium_controller_Create_0
/// `myc_get_mushroom_at(px, py)`: the topmost mushroom whose hit box holds the point, else the
/// origin mushroom.
pub fn get_mushroom_at(sim: &Sim, px: f64, py: f64) -> MushroomHit {
    for (i, m) in sim.state.mushrooms.iter().enumerate().rev() {
        if m.stolen {
            continue;
        }
        let b = mushrooms::mushroom_hit_box(sim, i);
        if in_rect(
            px,
            py,
            &Rect {
                x1: b.x1,
                y1: b.y1,
                x2: b.x2,
                y2: b.y2,
            },
        ) {
            return MushroomHit {
                hit: true,
                index: i as i64,
                x: m.x,
                y: m.y,
            };
        }
    }
    if in_rect(px, py, &origin_box(sim)) {
        return MushroomHit {
            hit: true,
            index: -1,
            x: sim.mycelium.x,
            y: sim.mycelium.y,
        };
    }
    MushroomHit {
        hit: false,
        index: -1,
        x: 0.0,
        y: 0.0,
    }
}

// port: anon__14604__gml_Object_obj_mycelium_controller_Create_0
/// `myc_get_surface_build_at(px, py)`: snaps to the surface within the build band.
pub fn get_surface_build_at(sim: &Sim, px: f64, py: f64) -> growth::Hit {
    let sy = sim.mycelium.surface_y;
    if (py - sy).abs() <= sim.bal("surface_build_band_px") {
        return growth::Hit {
            hit: true,
            x: px,
            y: sy,
        };
    }
    growth::Hit {
        hit: false,
        x: px,
        y: py,
    }
}

// port: anon__14901__gml_Object_obj_mycelium_controller_Create_0
/// Water available, not crowded, on the surface and mycelium just below.
pub fn surface_buildable_at(sim: &mut Sim, px: f64, py: f64) -> bool {
    if !has_water_source(sim) {
        return false;
    }
    let tile = sim.mycelium.tile;
    if has_mushroom_near(sim, px, py, tile * 2.2) {
        return false;
    }
    let b = get_surface_build_at(sim, px, py);
    if !b.hit || !(gml::point_distance(b.x, b.y, px, py) <= tile) {
        return false;
    }
    growth::get_mycelium_hit(sim, px, py + tile * 0.85, tile * 1.2).hit
}

// port: anon__15353__gml_Object_obj_mycelium_controller_Create_0
/// The tree a tree mushroom would grow on (uid), within `tree_build_reach_px`.
pub fn find_tree_near(sim: &mut Sim, px: f64) -> Option<u64> {
    let r = sim.bal("tree_build_reach_px");
    sim.forest.find_tree_near(&mut sim.rng, px, r)
}

// port: anon__15583__gml_Object_obj_mycelium_controller_Create_0
/// Unlocked and affordable; a tree mushroom needs a tree without one.
pub fn can_build_type(sim: &mut Sim, t: MushroomKind, px: f64) -> bool {
    if !mushrooms::mushroom_type_unlocked(&sim.state, t)
        || !mushrooms::mushroom_type_affordable(sim, t)
    {
        return false;
    }
    if t == MushroomKind::Tree {
        return match find_tree_near(sim, px) {
            Some(uid) => !mushrooms::squirrel_tree_has_mushroom(sim, uid),
            None => false,
        };
    }
    true
}

fn spend(sim: &mut Sim, kind: ResourceKind, n: f64, x: f64, y: f64) {
    if n > 0.0 {
        sim.emit(SimEvent::Resource {
            kind,
            delta: -n,
            x,
            y,
            source: mushrooms::next_source(sim),
        });
    }
}

// port: anon__15952__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_type_at(t, px, py)`: pay and add the mushroom (a tree mushroom on its tree),
/// with its root knot; a digger starts its strands. Emits `Resource{Food|Water, -n}`.
pub fn build_type_at(sim: &mut Sim, t: MushroomKind, px: f64, py: f64) -> bool {
    if !can_build_type(sim, t, px) {
        return false;
    }
    let cost = mushrooms::mushroom_type_cost(&sim.state, t);
    let wcost = mushrooms::mushroom_type_water_cost(&sim.state, t);
    if t == MushroomKind::Tree {
        let Some(uid) = find_tree_near(sim, px) else {
            return false;
        };
        let tx = sim.forest.tree(uid).map_or(px, |tr| tr.x);
        let mx = tx + vis("tree_mushroom_offset_x");
        let my = sim.mycelium.surface_y - vis("tree_mushroom_height_px");
        sim.food_spend(cost);
        spend(sim, ResourceKind::Food, cost, mx, my);
        water::water_spend(sim, wcost);
        spend(sim, ResourceKind::Water, wcost, mx, my);
        mushrooms::mushroom_add(sim, t, mx, my);
        return true;
    }
    sim.food_spend(cost);
    spend(sim, ResourceKind::Food, cost, px, py);
    if wcost > 0.0 {
        water::water_spend(sim, wcost);
        spend(sim, ResourceKind::Water, wcost, px, py);
    }
    mushrooms::mushroom_add(sim, t, px, py);
    knots::add_root_knot_for_mushroom(sim, px, py);
    if t == MushroomKind::Digger {
        growth::digger_burst(sim, px, py);
        if upg_is_owned(&sim.state, "unlock_digger")
            && upg_level(&sim.state, "digger_length") as f64
                >= upg_max_level(&sim.state, "digger_length")
        {
            tasks::stat_add(sim, "ach_digger_full", 1.0);
        }
    }
    true
}

// port: anon__17079__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_options_at`: the node option unless in water (then the hint) or next to a knot.
/// (The shroom options are compiled out of the original binary: never offered.)
pub fn build_options_at(sim: &mut Sim) -> Vec<BuildOption> {
    let mut options = Vec::new();
    let (x, y) = (sim.mycelium.build_node_x, sim.mycelium.build_node_y);
    if !sim.world.water_shown_at(x, y) {
        let r = sim.mycelium.knot_radius * 2.0;
        if !knots::has_knot_near(sim, x, y, r) {
            options.push(BuildOption::Node);
        }
    } else {
        sim.mycelium.build_hint = "hint_no_nodes_in_water".into();
    }
    options
}

// port: anon__18320__gml_Object_obj_mycelium_controller_Create_0
pub fn can_build_option(sim: &mut Sim, opt: &BuildOption) -> bool {
    match opt {
        BuildOption::Node => sim.state.state.food >= super::shot::node_food_cost(sim),
        BuildOption::Shroom(t) => {
            let x = sim.mycelium.build_surface_x;
            can_build_type(sim, *t, x)
        }
    }
}

// port: anon__18703__gml_Object_obj_mycelium_controller_Create_0
/// `myc_open_build_menu_at(px, py)`: the build menu at the nearest mycelium point.
pub fn open_build_menu_at(sim: &mut Sim, px: f64, py: f64) -> bool {
    let r = sim.mycelium.pick_radius;
    let hit = growth::get_mycelium_hit(sim, px, py, r);
    if !hit.hit {
        return false;
    }
    let sb = get_surface_build_at(sim, hit.x, hit.y);
    knots::clear_prompts(sim);
    let m = &mut sim.mycelium;
    m.build_node_x = hit.x;
    m.build_node_y = hit.y;
    m.build_surface_x = sb.x;
    m.build_surface_y = sb.y;
    sim.mycelium.build_surface_ok = sb.hit && surface_buildable_at(sim, sb.x, sb.y);
    let opts = build_options_at(sim);
    let m = &mut sim.mycelium;
    m.build_options = opts;
    m.mushroom_prompt = !m.build_options.is_empty() || !m.build_hint.is_empty();
    true
}

// port: anon__20040__gml_Object_obj_mycelium_controller_Create_0
/// `myc_apply_build_option(opt)` at the build menu's position.
pub fn apply_build_option(sim: &mut Sim, opt: &BuildOption) -> bool {
    match opt {
        BuildOption::Node => {
            let (x, y) = (sim.mycelium.build_node_x, sim.mycelium.build_node_y);
            knots::apply_node_at(sim, x, y).is_ok()
        }
        BuildOption::Shroom(t) => {
            let (x, y) = (sim.mycelium.build_surface_x, sim.mycelium.build_surface_y);
            let ok = build_type_at(sim, *t, x, y);
            if ok {
                tutorial::event(sim, "mushroom", None, None);
            }
            ok
        }
    }
}

/// Where a consumptor would sit (`___struct___484/485`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsumptorSpot {
    pub host: HostId,
    pub x: f64,
    pub y: f64,
    pub face: f64,
    pub free: bool,
}

/// `sprite_get_yoffset` of the tree sprites (sprites.json: origin y of tree01..03).
const TREE_YOFFSET: f64 = 534.0;

// port: anon__21041__gml_Object_obj_mycelium_controller_Create_0
/// `myc_consumptor_spot(px, py)`: the nearest free slot on a stump (or the nearest slot when
/// none is free), or a spot on a tree trunk's side at the cursor height.
pub fn consumptor_spot(sim: &mut Sim, px: f64, py: f64) -> Option<ConsumptorSpot> {
    let r = sim.bal("tree_build_reach_px");
    let host = sim.forest.find_host_near(&mut sim.rng, px, r)?;
    let list = mushrooms::consumptors_on(sim, host);
    let has_room = (list.len() as f64) < mushrooms::consume_info(sim, host).slots;
    if let HostId::Stump(uid) = host {
        let stump = sim.forest.stump(uid)?.clone();
        let mut best = None;
        let mut best_d = f64::INFINITY;
        let mut best_free = false;
        for i in 0..stump_parts()[stump.part].slots.len() {
            let s = stump_slot_at(&stump, i);
            let mut taken = false;
            for &j in &list {
                let m = &sim.state.mushrooms[j];
                taken = taken || gml::point_distance(m.x, m.y, s.x, s.y) < 2.0;
            }
            let d = gml::point_distance(px, py, s.x, s.y);
            if (!taken && !best_free) || ((!taken) == best_free && d < best_d) {
                best = Some(s);
                best_d = d;
                best_free = !taken;
            }
        }
        let b = best?;
        return Some(ConsumptorSpot {
            host,
            x: b.x,
            y: b.y,
            face: b.face,
            free: best_free && has_room,
        });
    }
    let HostId::Tree(uid) = host else {
        unreachable!()
    };
    let (hx, hy) = sim.forest.tree(uid).map(|t| (t.x, t.y))?;
    let side = if px >= hx { 1.0 } else { -1.0 };
    let top = hy - TREE_YOFFSET + sim.bal("consumptor_tree_top_row");
    let foot = hy - vis("consumptor_tree_foot_gap_px");
    let cy = gml::clamp(gml::round(py), top, foot);
    let mut ok = has_room;
    let gap = sim.bal("consumptor_min_gap_px");
    for &j in &list {
        let m = &sim.state.mushrooms[j];
        if m.side == side && (m.y - cy).abs() < gap {
            ok = false;
        }
    }
    let ex = mushrooms::squirrel_edge_x(sim, uid, side, cy);
    Some(ConsumptorSpot {
        host,
        x: gml::round(ex),
        y: cy,
        face: side,
        free: ok,
    })
}

/// A placement plan (`___struct___486`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub ok: bool,
    pub x: f64,
    pub y: f64,
    pub face: f64,
    pub host: Option<HostId>,
    pub root_x: f64,
    pub from_x: f64,
    pub from_y: f64,
}

// port: anon__23310__gml_Object_obj_mycelium_controller_Create_0
/// `myc_place_check(t, px, py)`: where a mushroom of type `t` would go for a cursor at
/// (px, py) and whether it can (buildable, water, room, mycelium within reach with a playable
/// link to its root). `py: None` = the surface.
pub fn place_check(sim: &mut Sim, t: MushroomKind, px: f64, py: Option<f64>) -> Plan {
    let sy = sim.mycelium.surface_y;
    let py = py.unwrap_or(sy);
    let mut plan = Plan {
        ok: false,
        x: px,
        y: sy,
        face: 1.0,
        host: None,
        root_x: px,
        from_x: px,
        from_y: sy,
    };
    let mut root_x = px;
    if t == MushroomKind::Consumptor {
        let Some(spot) = consumptor_spot(sim, px, py) else {
            return plan;
        };
        plan.x = spot.x;
        plan.y = spot.y;
        plan.face = spot.face;
        plan.host = Some(spot.host);
        root_x = sim.forest.host_x(spot.host).unwrap_or(spot.x);
        if !spot.free {
            plan.root_x = root_x;
            return plan;
        }
    }
    if t == MushroomKind::Tree {
        let Some(uid) = find_tree_near(sim, px) else {
            return plan;
        };
        let tx = sim.forest.tree(uid).map_or(px, |tr| tr.x);
        plan.x = tx + vis("tree_mushroom_offset_x");
        plan.y = sy - vis("tree_mushroom_height_px");
        root_x = tx;
    }
    plan.root_x = root_x;
    if can_build_type(sim, t, px) && has_water_source(sim) {
        if t != MushroomKind::Tree
            && t != MushroomKind::Consumptor
            && has_mushroom_near(sim, plan.x, sy, sim.bal("mushroom_min_gap_px"))
        {
            return plan;
        }
        let reach = sim.bal("mushroom_build_reach_px");
        let hit = growth::get_mycelium_hit(sim, root_x, sy, reach);
        // r = 1: the original passes the GML value `true` as the radius (LÖVE fix baff3b6)
        if hit.hit && segment_is_playable(sim, hit.x, hit.y, root_x, sy + 2.0, 1.0, false) {
            plan.ok = true;
            plan.from_x = hit.x;
            plan.from_y = hit.y;
        }
    }
    plan
}

// port: anon__26361__gml_Object_obj_mycelium_controller_Create_0
/// `myc_place_mushroom(t, px, py)`: place by the plan (a consumptor on its host, others
/// through `myc_build_type_at`), then grow the link strand to its root.
pub fn place_mushroom(sim: &mut Sim, t: MushroomKind, px: f64, py: Option<f64>) -> bool {
    let sy = sim.mycelium.surface_y;
    let plan = place_check(sim, t, px, py);
    if !plan.ok {
        return false;
    }
    if t == MushroomKind::Consumptor {
        let cost = mushrooms::mushroom_type_cost(&sim.state, t);
        let wcost = mushrooms::mushroom_type_water_cost(&sim.state, t);
        sim.food_spend(cost);
        spend(sim, ResourceKind::Food, cost, plan.x, plan.y);
        water::water_spend(sim, wcost);
        spend(sim, ResourceKind::Water, wcost, plan.x, plan.y);
        let idx = mushrooms::mushroom_add(sim, t, plan.x, plan.y);
        let variant = mushrooms::consumptor_variant(plan.face);
        mushrooms::set_consumptor(sim, idx, plan.host, plan.face, variant);
    } else {
        let bx = if t == MushroomKind::Tree { px } else { plan.x };
        if !build_type_at(sim, t, bx, sy) {
            return false;
        }
    }
    growth::add_link_growth(sim, plan.from_x, plan.from_y, plan.root_x, sy);
    tutorial::event(sim, "mushroom", None, None);
    true
}
