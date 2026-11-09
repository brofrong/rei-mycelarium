//! The shot: its parameters (range, spread, water and node costs, budding chance), the water
//! and food a shot costs, and the growth itself (main strand plus side branches).

use super::{place, source, Branch, Cell, Growth, GrowthSound, Pt};
use crate::audio;
use crate::balance::vis;
use crate::events::{ResourceKind, SimEvent};
use crate::gml;
use crate::progress::tutorial::{self, Tip, TipState};
use crate::progress::upgrades::upg_bonus;
use crate::sim::Sim;
use crate::world::{soil as wsoil, water, Tile};

// port: myc_shot_range
pub fn shot_range(sim: &Sim) -> f64 {
    sim.bal("base_shot_range") * (1.0 + upg_bonus(&sim.state, "shot_range"))
}

// port: myc_shot_range_max
pub fn shot_range_max(sim: &Sim) -> f64 {
    let per = sim.bal("upg_shot_range_per_level");
    let m = 1.0 + sim.bal("upg_shot_range_max_level") * per;
    sim.bal("base_shot_range") * m
}

// port: myc_digger_range
pub fn digger_range(sim: &Sim) -> f64 {
    shot_range_max(sim) * (1.0 + upg_bonus(&sim.state, "digger_length"))
}

// port: myc_shot_spread
pub fn shot_spread(sim: &Sim) -> f64 {
    sim.bal("base_shot_spread") * (1.0 + upg_bonus(&sim.state, "shot_width"))
}

// port: myc_shot_water_cost
/// Water of a full-range shot; free while it rains.
pub fn shot_water_cost(sim: &Sim) -> f64 {
    if crate::weather::is_raining(sim) {
        return 0.0;
    }
    let m = (1.0 - upg_bonus(&sim.state, "water_efficiency")).max(0.25);
    sim.bal("base_shot_water_cost") * m
}

// port: myc_shot_water_rate
/// Water per pixel of shot.
pub fn shot_water_rate(sim: &Sim) -> f64 {
    shot_water_cost(sim) / sim.bal("base_shot_range").max(1.0)
}

// port: myc_node_food_cost
pub fn node_food_cost(sim: &Sim) -> f64 {
    let m = (1.0 - upg_bonus(&sim.state, "node_efficiency")).max(0.2);
    gml::round(sim.bal("base_node_food_cost") * m).max(1.0)
}

// port: myc_budding_chance
/// Chance that a shot ends in a free node.
pub fn budding_chance(sim: &Sim) -> f64 {
    let b = upg_bonus(&sim.state, "budding_chance");
    gml::clamp(sim.bal("base_random_node_chance") + b, 0.0, 1.0)
}

// port: anon__5108__gml_Object_obj_mycelium_controller_Create_0
/// `myc_shot_cost_for(px, py, dir, len)`: water rate × length × the mean soil water factor
/// sampled every `sample_step` along the shot.
pub fn shot_cost_for(sim: &mut Sim, px: f64, py: f64, dir: f64, len: f64) -> f64 {
    if !(len > 0.0) {
        return 0.0;
    }
    let step = sim.mycelium.sample_step;
    let mut samples = 0.0;
    let mut sum = 0.0;
    let mut d = 0.0;
    while d <= len {
        samples += 1.0;
        let sx = px + gml::lengthdir_x(d, dir);
        let sy = py + gml::lengthdir_y(d, dir);
        sum += sim.world.soil_water_factor(sx, sy);
        d += step;
    }
    let avg = sum / f64::max(1.0, samples);
    shot_water_rate(sim) * len * avg
}

// port: anon__5711__gml_Object_obj_mycelium_controller_Create_0
/// `myc_shot_food_for`: the deep-soil food cost when the shot touches deep soil, else 0.
pub fn shot_food_for(sim: &mut Sim, px: f64, py: f64, dir: f64, len: f64) -> f64 {
    let step = sim.mycelium.sample_step;
    let mut d = 0.0;
    while d <= len {
        let sx = px + gml::lengthdir_x(d, dir);
        let sy = py + gml::lengthdir_y(d, dir);
        if sim.world.is_deep_soil(sx, sy) {
            return sim.bal("deepsoil_shot_food_cost");
        }
        d += step;
    }
    0.0
}

/// The options struct of `myc_add_growth_from` (missing fields read as their defaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowthOpts {
    /// no water/food cost, no tutorial events (bursts, digger)
    pub free: bool,
    /// grows through stone and cave walls are checked (digger, cave burst)
    pub pierce: bool,
    pub width_mult: f64,
    pub kinks_per_base: f64,
    /// finished at once, not rasterized (off-screen burst)
    pub instant: bool,
    /// no hyphae sound
    pub silent: bool,
}

impl Default for GrowthOpts {
    fn default() -> Self {
        GrowthOpts {
            free: false,
            pierce: false,
            width_mult: 1.0,
            kinks_per_base: 0.0,
            instant: false,
            silent: false,
        }
    }
}

// port: anon__42245__gml_Object_obj_mycelium_controller_Create_0
/// `myc_add_growth_from(px, py, dir, len, opts)`: pay for the shot (unless free), build the
/// main strand and the side branches, report the tutorial events, roll budding and the hyphae
/// sound and file the growth. Emits `Resource{Water|Food, -n}` at the origin for what it costs.
pub fn add_growth_from(sim: &mut Sim, px: f64, py: f64, dir: f64, len: f64, opts: &GrowthOpts) -> bool {
    if len < sim.mycelium.min_growth_len {
        return false;
    }
    let id = sim.mycelium.next_id + 1;
    if !opts.free {
        let cost = shot_cost_for(sim, px, py, dir, len);
        let food = shot_food_for(sim, px, py, dir, len);
        if sim.water_total() < cost || sim.state.state.food < food {
            return false;
        }
        water::water_spend(sim, cost);
        sim.food_spend(food);
        for (kind, n) in [(ResourceKind::Water, cost), (ResourceKind::Food, food)] {
            if n > 0.0 {
                sim.emit(SimEvent::Resource {
                    kind,
                    delta: -n,
                    x: px,
                    y: py,
                    source: source::GROWTH | id,
                });
            }
        }
    }
    let thick = vis("strand_thickness") * opts.width_mult;
    let spread = shot_spread(sim);
    let range = sim.bal("base_shot_range").max(1.0);
    let mainlen = len.min(range);
    let ratio = (len / range).max(1.0);
    let jitter = mainlen * vis("growth_jitter") * 0.1 * spread;
    let kinks = opts.kinks_per_base * ratio;
    let raster = !opts.instant;
    let pierce = opts.pierce;
    let main = make_branch(
        sim,
        px,
        py,
        dir,
        len,
        gml::round(14.0 * ratio),
        jitter,
        0.0,
        thick,
        pierce,
        kinks,
        raster,
    );
    let mpts = main.points.clone();
    let n = mpts.len();
    let mut branches = vec![main];
    let mut max_start: f64 = 0.0;
    let dur = sim.mycelium.growth_duration;
    if n > 4 {
        let count = gml::round(sim.bal("growth_side_branches") * ratio);
        let step = (n as f64 - 4.0) / count.max(1.0);
        let mut k = 0.0;
        while k < count {
            let r = sim.rng.random(1.0);
            let idx = gml::clamp(2.0 + ((k + r) * step).floor(), 0.0, n as f64 - 2.0);
            let bp = mpts[idx as usize];
            let sang = (sim.rng.random_range(25.0, 55.0) * gml::lerp(1.0, spread, 0.5)).min(80.0);
            let sdir = dir + sim.rng.choose(&[-1.0, 1.0]) * sang;
            let want = sim.rng.random_range(mainlen * 0.17, mainlen * 0.44) * spread;
            let slen = place::measure_allowed_len_from(sim, bp.x, bp.y, sdir, want, pierce);
            if !(slen < sim.mycelium.tile * 0.5) {
                let sstart = 5.0 + (idx / (n as f64 - 1.0).max(1.0)) * dur;
                let sb = make_branch(
                    sim,
                    bp.x,
                    bp.y,
                    sdir,
                    slen,
                    6.0,
                    jitter * 0.69,
                    sstart,
                    thick * 0.66,
                    pierce,
                    0.0,
                    raster,
                );
                if sb.points.len() > 1 {
                    branches.push(sb);
                    max_start = max_start.max(sstart);
                }
            }
            k += 1.0;
        }
    }
    let (mut bx1, mut by1, mut bx2, mut by2) =
        (branches[0].bx1, branches[0].by1, branches[0].bx2, branches[0].by2);
    for b in &branches[1..] {
        bx1 = bx1.min(b.bx1);
        by1 = by1.min(b.by1);
        bx2 = bx2.max(b.bx2);
        by2 = by2.max(b.by2);
    }
    if !opts.free {
        let tip = mpts[n - 1];
        tutorial::event(sim, "shot", Some(tip.x), Some(tip.y));
        if tutorial::tutorial_state(sim, Tip::Clay) == TipState::Hidden {
            for p in mpts.iter().rev() {
                if sim.world.tile_at(p.x, p.y) == Tile::HardSoil
                    && sim.world.locate(p.x, p.y).row as f64 >= wsoil::dense_row()
                {
                    tutorial::event(sim, "clay", Some(p.x), Some(p.y));
                    break;
                }
            }
        }
    }
    let finish = (dur + max_start) + 2.0;
    let bud_chance = budding_chance(sim);
    let budding = sim.rng.random(1.0) < bud_chance;
    let id = sim.mycelium.new_id();
    let sound = if opts.silent {
        None
    } else {
        let (ev, plan) = audio::hyphae_begin(&mut sim.rng, id, len, finish, None);
        sim.emit(ev);
        Some(GrowthSound { plan, done: false })
    };
    let g = Growth {
        id,
        age: 0.0,
        duration: dur,
        finish_age: finish,
        branches,
        scanned_length: 0.0,
        finished: false,
        budding,
        no_length: false,
        sound,
        instant: false,
        bx1,
        by1,
        bx2,
        by2,
        bake_d: 0.0,
        grid_stamp: -1.0,
        extra: Default::default(),
    };
    let m = &mut sim.mycelium;
    m.growths.push(g);
    let gi = m.growths.len() - 1;
    m.active.push(gi);
    super::growth::grid_add(sim, gi);
    true
}

// port: anon__36616__gml_Object_obj_mycelium_controller_Create_0
/// `myc_make_branch(x0, y0, dir0, len, segs, wobble, start, width, cave, kink, raster)`: a
/// wobbling strand of `segs` segments that stops (or, with kinks, straightens) where the
/// soil is not playable.
#[allow(clippy::too_many_arguments)]
pub fn make_branch(
    sim: &mut Sim,
    x0: f64,
    y0: f64,
    dir0: f64,
    len: f64,
    segs: f64,
    wobble: f64,
    start: f64,
    width: f64,
    cave: bool,
    kink: f64,
    raster: bool,
) -> Branch {
    let mut pts = vec![Pt { x: x0, y: y0 }];
    let mut perp = dir0 + 90.0;
    let w = (width * 0.5).max(2.0);
    let mut off = 0.0;
    let mut dir = dir0;
    let step = len / segs.max(1.0);
    let kink_chance = kink / segs.max(1.0);
    let mut cx = x0;
    let mut cy = y0;
    let mut i = 1.0;
    while i <= segs {
        let t = i / segs;
        if kink > 0.0 {
            if sim.rng.random(1.0) < kink_chance {
                let kr = sim
                    .rng
                    .random_range(vis("strand_kink_min_deg"), vis("strand_kink_max_deg"));
                let kd = sim.rng.choose(&[-1.0, 1.0]) * kr;
                let cone = vis("strand_kink_cone_deg");
                dir = dir0 + gml::clamp(gml::angle_difference(dir + kd, dir0), -cone, cone);
                perp = dir + 90.0;
            }
            cx += gml::lengthdir_x(step, dir);
            cy += gml::lengthdir_y(step, dir);
        } else {
            cx = x0 + gml::lengthdir_x(len * t, dir0);
            cy = y0 + gml::lengthdir_y(len * t, dir0);
        }
        off = off * 0.55 + sim.rng.random_range(-wobble, wobble) * 0.45;
        let amp = off * (1.0 - (t - 0.5).abs());
        let mut nx = cx + gml::lengthdir_x(amp, perp);
        let mut ny = cy + gml::lengthdir_y(amp, perp);
        let last = *pts.last().unwrap();
        if !place::segment_is_playable(sim, last.x, last.y, nx, ny, w, cave) {
            nx = cx;
            ny = cy;
            if !place::segment_is_playable(sim, last.x, last.y, nx, ny, w, cave) {
                if kink <= 0.0 || dir == dir0 {
                    break;
                }
                dir = dir0;
                perp = dir0 + 90.0;
                cx = last.x + gml::lengthdir_x(step, dir0);
                cy = last.y + gml::lengthdir_y(step, dir0);
                nx = cx;
                ny = cy;
                if !place::segment_is_playable(sim, last.x, last.y, nx, ny, w, cave) {
                    break;
                }
            }
        }
        pts.push(Pt { x: nx, y: ny });
        i += 1.0;
    }
    build_branch(&pts, start, width, raster)
}

// port: anon__39420__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_branch(pts, start, width, raster)`: the branch with its bounds and (when
/// `raster`) its rasterized cells.
pub fn build_branch(pts: &[Pt], start: f64, width: f64, raster: bool) -> Branch {
    let p0 = pts[0];
    let (mut minx, mut miny, mut maxx, mut maxy) = (p0.x, p0.y, p0.x, p0.y);
    for p in &pts[1..] {
        minx = minx.min(p.x);
        miny = miny.min(p.y);
        maxx = maxx.max(p.x);
        maxy = maxy.max(p.y);
    }
    Branch {
        points: pts.to_vec(),
        cells: raster.then(|| rasterize_branch(pts, width)),
        shoved: false,
        swept: 0.0,
        start,
        width,
        bx1: minx,
        by1: miny,
        bx2: maxx,
        by2: maxy,
        scan_done: 0.0,
        scan_done_len: 0.0,
        extra: Default::default(),
    }
}

// port: anon__40328__gml_Object_obj_mycelium_controller_Create_0
/// `myc_rasterize_branch(pts, width)`: integer cells along the strand, tapering from `width`
/// to 1, one per new pixel position, plus the last point.
pub fn rasterize_branch(pts: &[Pt], width: f64) -> Vec<Cell> {
    let n = pts.len();
    let mut out = Vec::new();
    if n < 2 {
        return out;
    }
    let mut total = 0.0;
    for i in 1..n {
        total += gml::point_distance(pts[i - 1].x, pts[i - 1].y, pts[i].x, pts[i].y);
    }
    if total <= 0.0 {
        return out;
    }
    let mut acc = 0.0;
    let (mut lx, mut ly) = (-999999.0, -999999.0);
    for i in 1..n {
        let a = pts[i - 1];
        let b = pts[i];
        let seg = gml::point_distance(a.x, a.y, b.x, b.y);
        if seg > 0.0 {
            let mut d = 0.0;
            while d < seg {
                let t = d / seg;
                let u = (acc + d) / total;
                let w = gml::round(gml::lerp(width, 1.0, u)).max(1.0);
                let cx = gml::lerp(a.x, b.x, t).floor();
                let cy = gml::lerp(a.y, b.y, t).floor();
                if cx != lx || cy != ly {
                    out.push(Cell { x: cx, y: cy, w });
                    lx = cx;
                    ly = cy;
                }
                d += (w - 1.0).max(1.0);
            }
            acc += seg;
        }
    }
    let last = pts[n - 1];
    out.push(Cell {
        x: last.x.floor(),
        y: last.y.floor(),
        w: 1.0,
    });
    out
}
