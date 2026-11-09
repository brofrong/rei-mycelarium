//! Growths after the shot: the spatial grid and hit test, scanning as they grow (stone
//! shoves, harvest, length), finishing (budding), link growths to mushrooms, the node burst of
//! the blue cave and the digger burst.

use super::{shot, soil, source, BurstKnot, DiggerJob, Growth, GrowthOpts, GrowthSound, Pt, GRID_CELL};
use crate::audio;
use crate::balance::{vis, wld};
use crate::events::{ResourceKind, SimEvent};
use crate::gml;
use crate::sim::Sim;
use crate::world::{self, caves, relics, stones};

fn cell_of(v: f64) -> i64 {
    (v / GRID_CELL).floor() as i64
}

// port: anon__31845__gml_Object_obj_mycelium_controller_Create_0
/// `myc_grid_add(e)`: file a growth under every grid cell its bounds touch and grow
/// `far_max` (the pulse reach).
pub fn grid_add(sim: &mut Sim, gi: usize) {
    let m = &mut sim.mycelium;
    let g = &m.growths[gi];
    let (bx1, by1, bx2, by2) = (g.bx1, g.by1, g.bx2, g.by2);
    for cx in cell_of(bx1)..=cell_of(bx2) {
        for cy in cell_of(by1)..=cell_of(by2) {
            m.grid.entry((cx, cy)).or_default().push(gi);
        }
    }
    let (x, y) = (m.x, m.y);
    m.far_max = m
        .far_max
        .max(gml::point_distance(x, y, bx1, by1))
        .max(gml::point_distance(x, y, bx2, by1))
        .max(gml::point_distance(x, y, bx1, by2))
        .max(gml::point_distance(x, y, bx2, by2));
}

// port: anon__32773__gml_Object_obj_mycelium_controller_Create_0
/// `myc_grid_rebuild`.
pub fn grid_rebuild(sim: &mut Sim) {
    sim.mycelium.grid.clear();
    sim.mycelium.far_max = 0.0;
    for gi in 0..sim.mycelium.growths.len() {
        grid_add(sim, gi);
    }
}

/// A hit on the mycelium (`___struct___495`); x, y are the query point when there is no hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hit {
    pub hit: bool,
    pub x: f64,
    pub y: f64,
}

// port: anon__32963__gml_Object_obj_mycelium_controller_Create_0
/// `myc_get_mycelium_hit(px, py, r)`: the nearest point within `r` on the grown part of any
/// strand (later candidates win ties), or on a taken cave's wrap.
pub fn get_mycelium_hit(sim: &mut Sim, px: f64, py: f64, r: f64) -> Hit {
    let mut hit = false;
    let mut best = r;
    let (mut hx, mut hy) = (px, py);
    let m = &mut sim.mycelium;
    m.grid_stamp += 1.0;
    let stamp = m.grid_stamp;
    let mut cands = Vec::new();
    for cx in cell_of(px - r)..=cell_of(px + r) {
        for cy in cell_of(py - r)..=cell_of(py + r) {
            if let Some(cell) = m.grid.get(&(cx, cy)) {
                for &gi in cell {
                    let g = &mut m.growths[gi];
                    if g.grid_stamp != stamp {
                        g.grid_stamp = stamp;
                        cands.push(gi);
                    }
                }
            }
        }
    }
    for gi in cands {
        let g = &m.growths[gi];
        for b in &g.branches {
            let pts = &b.points;
            let n = pts.len();
            if n < 2 || px < b.bx1 - r || px > b.bx2 + r || py < b.by1 - r || py > b.by2 + r {
                continue;
            }
            let prog = gml::clamp((g.age - b.start) / g.duration, 0.0, 1.0);
            if prog <= 0.0 {
                continue;
            }
            let lim = prog * (n as f64 - 1.0);
            for k in 0..n - 1 {
                let t = gml::clamp(lim - k as f64, 0.0, 1.0);
                if t <= 0.0 {
                    break;
                }
                let (p0, p1) = (pts[k], pts[k + 1]);
                let dx = (p1.x - p0.x) * t;
                let dy = (p1.y - p0.y) * t;
                let dd = dx * dx + dy * dy;
                if dd > 0.0 {
                    let u = gml::clamp(((px - p0.x) * dx + (py - p0.y) * dy) / dd, 0.0, 1.0);
                    let qx = p0.x + dx * u;
                    let qy = p0.y + dy * u;
                    let d = gml::point_distance(px, py, qx, qy);
                    if d <= best {
                        hit = true;
                        best = d;
                        hx = qx;
                        hy = qy;
                    }
                }
            }
        }
    }
    let w = caves::caves_wrap_hit(sim, px, py, best);
    if w.hit {
        hit = true;
        hx = w.x;
        hy = w.y;
    }
    Hit { hit, x: hx, y: hy }
}

// port: anon__57586__gml_Object_obj_mycelium_controller_Create_0
/// `myc_scan_growths`: scan every active, unfinished growth.
pub fn scan_growths(sim: &mut Sim) {
    let mut i = 0;
    while i < sim.mycelium.active.len() {
        let gi = sim.mycelium.active[i];
        if !sim.mycelium.growths[gi].finished {
            scan_growth(sim, gi);
        }
        i += 1;
    }
}

// port: anon__58028__gml_Object_obj_mycelium_controller_Create_0
/// `myc_scan_growth(g)`: for each strand's grown part, shove stones along new segments (and at
/// the tip once full grown), harvest every new point and the partial tip, add the main
/// strand's new length, and finish the growth at `finish_age`.
pub fn scan_growth(sim: &mut Sim, gi: usize) {
    let mut total = 0.0;
    let nb = sim.mycelium.growths[gi].branches.len();
    let gid = sim.mycelium.growths[gi].id;
    let src = source::GROWTH | gid;
    for i in 0..nb {
        let (n, prog, width, instant, no_length) = {
            let g = &sim.mycelium.growths[gi];
            let b = &g.branches[i];
            (
                b.points.len(),
                gml::clamp((g.age - b.start) / g.duration, 0.0, 1.0),
                b.width,
                g.instant,
                g.no_length,
            )
        };
        if n < 2 || prog <= 0.0 {
            continue;
        }
        let pt = |sim: &Sim, j: usize| -> Pt { sim.mycelium.growths[gi].branches[i].points[j] };
        let lim = prog * (n as f64 - 1.0);
        let thick = vis("strand_thickness");
        let power = wld("stone_shove_power") * width / thick.max(0.01);
        let upto = lim.floor().min(n as f64 - 2.0);
        if instant {
            let b = &mut sim.mycelium.growths[gi].branches[i];
            b.swept = b.swept.max(upto);
            b.shoved = true;
        }
        let swept = sim.mycelium.growths[gi].branches[i].swept;
        if upto > swept {
            let mut j = swept;
            while j < upto {
                let (p0, p1) = (pt(sim, j as usize), pt(sim, j as usize + 1));
                stones::stones_shove(
                    sim,
                    (p0.x + p1.x) * 0.5,
                    (p0.y + p1.y) * 0.5,
                    gml::point_direction(p0.x, p0.y, p1.x, p1.y),
                    power,
                );
                j += 1.0;
            }
            sim.mycelium.growths[gi].branches[i].swept = upto;
        }
        if prog >= 1.0 && !sim.mycelium.growths[gi].branches[i].shoved {
            sim.mycelium.growths[gi].branches[i].shoved = true;
            let (a, c) = (pt(sim, n - 1), pt(sim, n - 2));
            stones::stones_shove(sim, a.x, a.y, gml::point_direction(c.x, c.y, a.x, a.y), power);
        }
        let count_len = i == 0 && !no_length;
        let (sd, mut sdl) = {
            let b = &sim.mycelium.growths[gi].branches[i];
            (b.scan_done, b.scan_done_len)
        };
        let hi = lim.floor().min(n as f64 - 1.0);
        let mut j = sd;
        while j < hi {
            let (p0, p1) = (pt(sim, j as usize), pt(sim, j as usize + 1));
            sdl += gml::point_distance(p0.x, p0.y, p1.x, p1.y);
            harvest_at(sim, p1.x, p1.y, src);
            j += 1.0;
        }
        {
            let b = &mut sim.mycelium.growths[gi].branches[i];
            b.scan_done = sd.max(hi);
            b.scan_done_len = sdl;
        }
        let mut partial = 0.0;
        if hi < n as f64 - 1.0 {
            let f = gml::clamp(lim - hi, 0.0, 1.0);
            if f > 0.0 {
                let (p0, p1) = (pt(sim, hi as usize), pt(sim, hi as usize + 1));
                let qx = gml::lerp(p0.x, p1.x, f);
                let qy = gml::lerp(p0.y, p1.y, f);
                partial = gml::point_distance(p0.x, p0.y, qx, qy);
                harvest_at(sim, qx, qy, src);
            }
        }
        if count_len {
            total += sdl + partial;
        }
    }
    let scanned = sim.mycelium.growths[gi].scanned_length;
    soil::mycelium_add_length(sim, total - scanned);
    let g = &mut sim.mycelium.growths[gi];
    g.scanned_length = total;
    if g.age >= g.finish_age {
        finish_growth(sim, gi);
    }
}

// port: anon__54952__gml_Object_obj_mycelium_controller_Create_0
/// `myc_finish_growth(g)`: done; queue it for baking (its live strand chunks are freed by
/// Task 14); a budding growth puts a node at the tip of a random strand.
pub fn finish_growth(sim: &mut Sim, gi: usize) {
    sim.mycelium.growths[gi].finished = true;
    sim.mycelium.dirty_growths.push(gi);
    if !sim.mycelium.growths[gi].budding {
        return;
    }
    let nb = sim.mycelium.growths[gi].branches.len();
    let k = sim.rng.irandom(nb as f64 - 1.0) as usize;
    let p = *sim.mycelium.growths[gi].branches[k].points.last().unwrap();
    super::knots::add_knot(sim, p.x, p.y);
}

// port: anon__55404__gml_Object_obj_mycelium_controller_Create_0
/// `myc_harvest_at(px, py)`: collect food, connect water, touch caves and relics, grow the
/// camera box. Emits `Resource{Food|Water, +n}` at the point with the growth's `source`
/// (ruling R20; infinite ocean water has no number and emits none).
pub fn harvest_at(sim: &mut Sim, px: f64, py: f64, src: u64) {
    let r = sim.mycelium.food_pickup_radius;
    let n = world::collect_food_at(sim, px, py, r);
    sim.state.state.food += n;
    if n > 0.0 {
        sim.emit(SimEvent::Resource {
            kind: ResourceKind::Food,
            delta: n,
            x: px,
            y: py,
            source: src,
        });
    }
    let before = sim.state.water.len();
    if world::connect_water_at(sim, px, py) {
        sim.emit(audio::water_touch(Some((px, py))));
        if sim.state.water.len() > before {
            let w = sim.state.water.last().unwrap().clone();
            let amount = world::water_source_amount(sim, &w);
            if amount > 0.0 {
                sim.emit(SimEvent::Resource {
                    kind: ResourceKind::Water,
                    delta: amount,
                    x: px,
                    y: py,
                    source: src,
                });
            }
        }
    }
    caves::caves_touch_at(sim, px, py);
    relics::relics_touch_at(sim, px, py);
    soil::reach_expand(sim, px, py);
}

// port: anon__25115__gml_Object_obj_mycelium_controller_Create_0
/// `myc_add_link_growth(x1, y1, x2, y2)`: a thin strand from the mycelium to a new mushroom's
/// root (ends exactly at (x2, y2), adds no length, always sounds).
pub fn add_link_growth(sim: &mut Sim, x1: f64, y1: f64, x2: f64, y2: f64) {
    let dist = gml::point_distance(x1, y1, x2, y2);
    if dist < 2.0 {
        return;
    }
    let thick = vis("strand_thickness") * 0.8;
    let br = shot::make_branch(
        sim,
        x1,
        y1,
        gml::point_direction(x1, y1, x2, y2),
        dist,
        gml::round(dist / 24.0).max(4.0),
        dist * vis("growth_jitter") * 0.1,
        0.0,
        thick,
        false,
        0.0,
        true,
    );
    let mut pts = br.points;
    if pts.len() > 1 {
        *pts.last_mut().unwrap() = Pt { x: x2, y: y2 };
    } else {
        pts.push(Pt { x: x2, y: y2 });
    }
    let vb = shot::build_branch(&pts, 0.0, thick, true);
    let (bx1, by1, bx2, by2) = (vb.bx1, vb.by1, vb.bx2, vb.by2);
    let dur = sim.mycelium.growth_duration;
    let id = sim.mycelium.new_id();
    let (ev, plan) = audio::hyphae_begin(&mut sim.rng, id, dist, dur + 2.0, None);
    sim.emit(ev);
    let m = &mut sim.mycelium;
    m.growths.push(Growth {
        id,
        age: 0.0,
        duration: dur,
        finish_age: dur + 2.0,
        branches: vec![vb],
        scanned_length: 0.0,
        finished: false,
        budding: false,
        no_length: true,
        sound: Some(GrowthSound { plan, done: false }),
        instant: false,
        bx1,
        by1,
        bx2,
        by2,
        bake_d: 0.0,
        grid_stamp: -1.0,
        extra: Default::default(),
    });
    let gi = m.growths.len() - 1;
    m.active.push(gi);
    grid_add(sim, gi);
}

// port: anon__47204__gml_Object_obj_mycelium_controller_Create_0
/// `myc_digger_burst(px, py)`: queue a fan of digger strands below a digger mushroom (they
/// share one "sounded" flag: only the first strand that grows makes a sound).
pub fn digger_burst(sim: &mut Sim, px: f64, py: f64) {
    let count = sim.bal("digger_strand_count").max(1.0);
    let fan_from = vis("digger_fan_from");
    let fan_to = vis("digger_fan_to");
    let ry = py + vis("digger_root_depth");
    sim.mycelium.digger_shared.push(false);
    let shared = sim.mycelium.digger_shared.len() - 1;
    let mut i = 0.0;
    while i < count {
        let t = if count == 1.0 { 0.5 } else { i / (count - 1.0) };
        let jit = vis("digger_fan_jitter");
        let jr = sim.rng.random_range(-1.0, 1.0) * jit;
        sim.mycelium.digger_queue.push(DiggerJob {
            x: px,
            y: ry,
            dir: gml::lerp(fan_from, fan_to, t) + jr,
            shared,
        });
        i += 1.0;
    }
}

// port: anon__48225__gml_Object_obj_mycelium_controller_Create_0
/// `myc_digger_step`: grow the queued digger strands (free, piercing, kinked); the original's
/// time budget never runs out here, so the whole queue grows in one step.
pub fn digger_step(sim: &mut Sim) {
    if sim.mycelium.digger_queue.is_empty() {
        return;
    }
    let range = shot::digger_range(sim);
    while sim.mycelium.digger_next < sim.mycelium.digger_queue.len() {
        let q = sim.mycelium.digger_queue[sim.mycelium.digger_next];
        sim.mycelium.digger_next += 1;
        let len = super::place::measure_allowed_len_from(sim, q.x, q.y, q.dir, range, true);
        let opts = GrowthOpts {
            free: true,
            pierce: true,
            width_mult: vis("digger_strand_width_mult"),
            kinks_per_base: vis("digger_kinks_per_base"),
            instant: false,
            silent: sim.mycelium.digger_shared[q.shared],
        };
        if shot::add_growth_from(sim, q.x, q.y, q.dir, len, &opts) {
            sim.mycelium.digger_shared[q.shared] = true;
        }
    }
    if sim.mycelium.digger_next >= sim.mycelium.digger_queue.len() {
        sim.mycelium.digger_queue.clear();
        sim.mycelium.digger_next = 0;
        sim.mycelium.digger_shared.clear();
    }
}

// port: anon__49789__anon__49515__gml_Object_obj_mycelium_controller_Create_0
// port: anon__49515__gml_Object_obj_mycelium_controller_Create_0
/// `myc_node_burst(px, py)`: queue every knot, nearest to (px, py) first, for a burst growth
/// (the blue cave's reward).
pub fn node_burst(sim: &mut Sim, px: f64, py: f64) {
    let m = &mut sim.mycelium;
    let mut list: Vec<BurstKnot> = m
        .knots
        .iter()
        .map(|k| BurstKnot {
            x: k.x,
            y: k.y,
            d: gml::point_distance(px, py, k.x, k.y),
        })
        .collect();
    // comparator a.d - b.d (stable)
    list.sort_by(|a, b| a.d.partial_cmp(&b.d).unwrap());
    if m.burst_next > 0 {
        m.burst_queue.drain(0..m.burst_next);
        m.burst_next = 0;
    }
    m.burst_queue.extend(list);
}

// port: anon__50255__gml_Object_obj_mycelium_controller_Create_0
/// `myc_burst_in_view(p)`: within one shot range of the view.
pub fn burst_in_view(sim: &Sim, p: &BurstKnot) -> bool {
    let m = shot::shot_range(sim);
    let c = &sim.mycelium;
    p.x > c.camera_x - m
        && p.x < (c.camera_x + c.view_w) + m
        && p.y > c.camera_y - m
        && p.y < (c.camera_y + c.view_h) + m
}

// port: anon__50824__gml_Object_obj_mycelium_controller_Create_0
/// `myc_burst_fire(k, instant)`: a free growth in a random direction from the knot; instant
/// ones finish (and harvest) at once and are queued for baking. Returns the growth index.
pub fn burst_fire(sim: &mut Sim, k: &BurstKnot, instant: bool) -> Option<usize> {
    let dir = sim.rng.random(360.0);
    let range = shot::shot_range(sim);
    let len = super::place::measure_allowed_len_from(sim, k.x, k.y, dir, range, false);
    if len < sim.mycelium.min_growth_len {
        return None;
    }
    let silent = instant || sim.mycelium.burst_sounded;
    let opts = GrowthOpts {
        free: true,
        silent,
        instant,
        ..GrowthOpts::default()
    };
    if !shot::add_growth_from(sim, k.x, k.y, dir, len, &opts) {
        return None;
    }
    let gi = sim.mycelium.growths.len() - 1;
    if instant {
        let g = &mut sim.mycelium.growths[gi];
        g.instant = true;
        g.age = g.finish_age;
        scan_growth(sim, gi);
        // myc_growth_bake(g): the finish above queued it in dirty_growths for Task 14
    } else {
        sim.mycelium.burst_sounded = true;
    }
    Some(gi)
}

// port: anon__52233__gml_Object_obj_mycelium_controller_Create_0
/// `myc_node_burst_step`: off-view knots fire instantly, knots in view are shown growing, a
/// few per frame and at most `cave_burst_max_growing` at once (no time budget limit here).
pub fn node_burst_step(sim: &mut Sim) {
    let m = &mut sim.mycelium;
    if m.burst_next >= m.burst_queue.len() && m.burst_show_next >= m.burst_show.len() {
        if !m.burst_queue.is_empty() || !m.burst_show.is_empty() {
            m.burst_queue.clear();
            m.burst_next = 0;
            m.burst_show.clear();
            m.burst_show_next = 0;
        }
        m.burst_growing.clear();
        m.burst_sounded = false;
        return;
    }
    while sim.mycelium.burst_next < sim.mycelium.burst_queue.len() {
        let k = sim.mycelium.burst_queue[sim.mycelium.burst_next];
        sim.mycelium.burst_next += 1;
        if burst_in_view(sim, &k) {
            sim.mycelium.burst_show.push(k);
        } else {
            burst_fire(sim, &k, true);
        }
    }
    let m = &mut sim.mycelium;
    let growths = &m.growths;
    m.burst_growing.retain(|&gi| !growths[gi].finished);
    let mut started = 0.0;
    let per_frame = vis("cave_burst_per_frame").max(1.0);
    let limit = per_frame.min(vis("cave_burst_max_growing") - m.burst_growing.len() as f64);
    while sim.mycelium.burst_show_next < sim.mycelium.burst_show.len() {
        let k = sim.mycelium.burst_show[sim.mycelium.burst_show_next];
        if !burst_in_view(sim, &k) {
            sim.mycelium.burst_show_next += 1;
            burst_fire(sim, &k, true);
        } else {
            if started >= limit {
                return;
            }
            sim.mycelium.burst_show_next += 1;
            started += 1.0;
            if let Some(gi) = burst_fire(sim, &k, false) {
                sim.mycelium.burst_growing.push(gi);
            }
        }
    }
}
