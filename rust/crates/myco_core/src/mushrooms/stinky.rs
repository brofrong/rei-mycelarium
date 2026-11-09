//! Stinky mushrooms: the rising stench column (a particle pool per mushroom, simulated in
//! logic like the original so the RNG stream matches), its funnel shape, the beam test that
//! keeps crows off and the guard that scares squirrels. Drawing (`stinky_draw`,
//! `stinky_art_init`) is Task 15's; it reads `px/py/pix/colour/alpha` of the first `n`
//! particles of [`pool_of`] while `stench_live` is not false.

use super::{types, MushroomKind, MushroomsRt};
use crate::balance::{vis, vis_arr};
use crate::gml;
use crate::progress::upgrades::upg_level;
use crate::sim::{Sim, ViewRect};
use crate::state::MushroomSave;

/// `___struct___89` (`m.stench`): the particle pool, structure of arrays; `n` are live.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StinkyPool {
    pub n: usize,
    pub side: Vec<f64>,
    pub rise: Vec<f64>,
    pub offset: Vec<f64>,
    pub speed: Vec<f64>,
    pub age: Vec<f64>,
    pub phase: Vec<f64>,
    pub size: Vec<f64>,
    pub heat: Vec<f64>,
    pub drift: Vec<f64>,
    pub px: Vec<f64>,
    pub py: Vec<f64>,
    pub pix: Vec<f64>,
    pub colour: Vec<f64>,
    pub alpha: Vec<f64>,
}

/// `a[i] = x` on a GML array (grows it when `i` is the length).
fn put(a: &mut Vec<f64>, i: usize, x: f64) {
    if i < a.len() {
        a[i] = x;
    } else {
        a.resize(i, 0.0);
        a.push(x);
    }
}

/// The stench pool of a mushroom, if it has one yet.
pub fn pool_of(m: &MushroomSave) -> Option<&StinkyPool> {
    m.live.pool.as_deref()
}

// port: stinky_cap_height
/// Height of the cap above the foot (per variant, scaled by the grow-in), minus the emit inset.
pub fn stinky_cap_height(sim: &Sim, m: &MushroomSave) -> f64 {
    const HEIGHTS: [f64; 3] = [31.0, 51.0, 30.0];
    let v = types::mushroom_variant(m);
    let gs = types::shroom_grow_scale(sim, m);
    let idx = gml::clamp(v, 0.0, 2.0).trunc() as usize;
    HEIGHTS[idx] * gs - vis("stinky_emit_into_cap_px")
}

/// `___struct___88`: the funnel at the current `stinky_width` level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StinkyShape {
    pub base: f64,
    pub open: f64,
    pub flare: f64,
    pub top: f64,
}

// port: stinky_shape
pub fn stinky_shape(sim: &Sim) -> StinkyShape {
    let lvl = gml::clamp(upg_level(&sim.state, "stinky_width") as f64, 0.0, 4.0);
    let li = lvl.floor() as usize;
    let base = vis_arr("stinky_base_radius_levels")[li];
    let t = gml::dtan(vis_arr("stinky_cone_deg_levels")[li]);
    let open = vis("stinky_beam_height_px") * t;
    StinkyShape {
        base,
        open,
        flare: vis_arr("stinky_flare_power_levels")[li],
        top: base + open,
    }
}

// port: stinky_top_at
/// Radius at the top of the column for width level `lvl`.
pub fn stinky_top_at(lvl: f64) -> f64 {
    let li = lvl.trunc() as usize;
    let t = gml::dtan(vis_arr("stinky_cone_deg_levels")[li]);
    vis_arr("stinky_base_radius_levels")[li] + vis("stinky_beam_height_px") * t
}

// port: stinky_funnel_radius
/// Column radius `dy` px above the cap.
pub fn stinky_funnel_radius(sh: &StinkyShape, dy: f64) -> f64 {
    let t = gml::clamp(dy / vis("stinky_beam_height_px").max(1.0), 0.0, 1.0);
    sh.base + sh.open * gml::power(t, sh.flare)
}

// port: stinky_pool
/// The mushroom's pool, made empty on first use.
pub fn stinky_pool(m: &mut MushroomSave) -> &mut StinkyPool {
    m.live.pool.get_or_insert_with(Default::default)
}

// port: stinky_particle_add
/// Add a particle at height `rise` (pre-filled columns start mid-air, aged and drifting).
pub fn stinky_particle_add(sim: &mut Sim, idx: usize, rise: f64) {
    let side = sim.rng.random_range(-1.0, 1.0);
    let off = if rise > 0.0 {
        stinky_wind_shift(sim, rise)
    } else {
        0.0
    };
    let rmax = vis("stinky_rise_max");
    let rmin = vis("stinky_rise_min");
    let speed = sim.rng.random_range(rmin, rmax);
    let age = if rise > 0.0 { 999.0 } else { 0.0 };
    let phase = sim.rng.random(360.0);
    let size = sim.rng.choose(&[1.0, 1.0, 2.0, 2.0, 2.0, 3.0]);
    let heat = sim.rng.random(1.0);
    let drift = if rise > 0.0 {
        stinky_wind_target(sim)
    } else {
        0.0
    };
    let pool = stinky_pool(&mut sim.state.mushrooms[idx]);
    let n = pool.n;
    put(&mut pool.side, n, side);
    put(&mut pool.rise, n, rise);
    put(&mut pool.offset, n, off);
    put(&mut pool.speed, n, speed);
    put(&mut pool.age, n, age);
    put(&mut pool.phase, n, phase);
    put(&mut pool.size, n, size);
    put(&mut pool.heat, n, heat);
    put(&mut pool.drift, n, drift);
    pool.n = n + 1;
}

// port: stinky_wind_target
/// Horizontal push of the wind on the stench.
pub fn stinky_wind_target(sim: &Sim) -> f64 {
    let [wind_dir, wind_speed, _] = sim.state.state.wind;
    wind_dir * wind_speed * vis("stinky_wind_push")
}

// port: stinky_wind_shift
/// How far the wind has carried a particle that rose `rise` px.
pub fn stinky_wind_shift(sim: &Sim, rise: f64) -> f64 {
    let rmax = vis("stinky_rise_max");
    let avg = ((vis("stinky_rise_min") + rmax) * 0.5).max(0.05);
    stinky_wind_target(sim) * rise / avg
}

/// The hot and deep stench colours (`make_colour_rgb` of the `stinky_hot_*` / `_deep_*` keys).
pub fn stinky_colours() -> (f64, f64) {
    let hot = gml::make_colour_rgb(
        vis("stinky_hot_r"),
        vis("stinky_hot_g"),
        vis("stinky_hot_b"),
    );
    let deep = gml::make_colour_rgb(
        vis("stinky_deep_r"),
        vis("stinky_deep_g"),
        vis("stinky_deep_b"),
    );
    (hot, deep)
}

// port: stinky_step
/// One frame of a stinky mushroom's column (`v` = the view): off-view columns pause; visible
/// ones emit (fewer when many columns are live), rise, sway with the wind and get colour and
/// alpha for drawing.
pub fn stinky_step(sim: &mut Sim, idx: usize, v: &ViewRect) {
    let beam = vis("stinky_beam_height_px");
    let sh = stinky_shape(sim);
    let reach = sh.top + stinky_wind_shift(sim, beam).abs() + 120.0;
    let (mx0, my0) = (sim.state.mushrooms[idx].x, sim.state.mushrooms[idx].y);
    let mut visible = false;
    // `not (m.x < v.x - reach) and ...`, as written
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    let in_x = !(mx0 < v.x - reach) && mx0 <= v.x + v.w + reach;
    if in_x {
        let vb = v.y + v.h;
        let cap = stinky_cap_height(sim, &sim.state.mushrooms[idx]);
        if my0 - cap - beam <= vb && my0 >= v.y - 200.0 {
            visible = true;
        }
    }
    if !visible {
        sim.state.mushrooms[idx].stench_live = Some(false);
        return;
    }
    sim.state.mushrooms[idx].stench_live = Some(true);
    sim.mushrooms.stinky_live_count += 1.0;
    stinky_pool(&mut sim.state.mushrooms[idx]);
    let top2 = stinky_top_at(2.0);
    let gs = types::shroom_grow_scale(sim, &sim.state.mushrooms[idx]);
    let mut emit = vis("stinky_emit_per_frame") * gs * sh.top / top2.max(1.0);
    let lastc = sim.mushrooms.stinky_live_last.max(1.0);
    emit *= (vis("stinky_full_columns") / lastc).min(1.0);
    let pool_n = pool_of(&sim.state.mushrooms[idx]).map_or(0, |p| p.n);
    if pool_n == 0 && gs >= 1.0 {
        let rmax = vis("stinky_rise_max");
        let avg = (vis("stinky_rise_min") + rmax) * 0.5;
        let cnt = gml::round(emit * beam / avg);
        let mut k = 0.0;
        while k < cnt {
            let r = sim.rng.random(beam);
            stinky_particle_add(sim, idx, r);
            k += 1.0;
        }
    }
    sim.state.mushrooms[idx].stench_due += emit;
    while sim.state.mushrooms[idx].stench_due >= 1.0 {
        sim.state.mushrooms[idx].stench_due -= 1.0;
        stinky_particle_add(sim, idx, 0.0);
    }
    let lift = vis("stinky_lift");
    let sway = vis("stinky_sway_px");
    let wt = stinky_wind_target(sim);
    let ease = vis("stinky_wind_ease");
    let (base, open, flare) = (sh.base, sh.open, sh.flare);
    let inv_beam = 1.0 / beam.max(1.0);
    let top_fade = 1.0 / vis("stinky_top_fade_px").max(1.0);
    let fade_in = 1.0 / vis("stinky_fade_in_frames").max(1.0);
    let alpha_k = vis("stinky_alpha");
    let (hot, deep) = stinky_colours();
    let shade = stinky_shade_table(&mut sim.mushrooms, hot, deep).to_vec();
    let mx = mx0;
    let my = my0 - stinky_cap_height(sim, &sim.state.mushrooms[idx]);
    let p = stinky_pool(&mut sim.state.mushrooms[idx]);
    let mut n = p.n;
    let mut i = 0;
    while i < n {
        let sp = p.speed[i] + lift;
        let r = p.rise[i] + sp;
        if r >= beam {
            n -= 1;
            if i < n {
                p.side[i] = p.side[n];
                p.rise[i] = p.rise[n];
                p.offset[i] = p.offset[n];
                p.speed[i] = p.speed[n];
                p.age[i] = p.age[n];
                p.phase[i] = p.phase[n];
                p.size[i] = p.size[n];
                p.heat[i] = p.heat[n];
                p.drift[i] = p.drift[n];
            }
        } else {
            let a = p.age[i] + 1.0;
            let ph = p.phase[i] + 3.0;
            let mut d = p.drift[i];
            d += (wt - d) * ease;
            let off = p.offset[i] + gml::dsin(ph) * sway + d;
            let t = r * inv_beam;
            let xoff = p.side[i] * (base + open * gml::power(t, flare)) + off;
            p.speed[i] = sp;
            p.rise[i] = r;
            p.age[i] = a;
            p.phase[i] = ph;
            p.drift[i] = d;
            p.offset[i] = off;
            put(&mut p.px, i, (mx + xoff).floor());
            put(&mut p.py, i, (my - r).floor());
            put(
                &mut p.pix,
                i,
                gml::round(p.size[i] * (1.0 - t * 0.45)).max(1.0),
            );
            let si = gml::round(gml::clamp(t * 0.8 + p.heat[i] * 0.35, 0.0, 1.0) * 256.0);
            put(&mut p.colour, i, shade[si as usize]);
            let alpha = alpha_k
                * (a * fade_in).min(1.0)
                * gml::clamp((beam - r) * top_fade, 0.0, 1.0)
                * (0.75 + 0.25 * gml::dsin(ph * 2.3));
            put(&mut p.alpha, i, alpha);
            i += 1;
        }
    }
    p.n = n;
}

// port: stinky_shade_table
/// 257 colours from `hot` to `deep` (cached until the colours change).
pub fn stinky_shade_table(rt: &mut MushroomsRt, hot: f64, deep: f64) -> &[f64] {
    let fresh = matches!(&rt.shade, Some((h, d, _)) if *h == hot && *d == deep);
    if !fresh {
        let t = (0..=256)
            .map(|i| gml::merge_colour(hot, deep, i as f64 / 256.0))
            .collect();
        rt.shade = Some((hot, deep, t));
    }
    &rt.shade.as_ref().unwrap().2
}

// port: stinky_beam_hits
/// Whether a circle of radius `r` at (px, py) is inside any stinky column.
pub fn stinky_beam_hits(sim: &Sim, px: f64, py: f64, r: f64) -> bool {
    let bh = vis("stinky_beam_height_px");
    let sh = stinky_shape(sim);
    for m in &sim.state.mushrooms {
        if m.kind == MushroomKind::Stinky.code() && !m.stolen {
            let d = m.y - stinky_cap_height(sim, m) - py;
            if !(d < -r || d > bh + r)
                && (px - (m.x + stinky_wind_shift(sim, d.max(0.0)))).abs()
                    <= stinky_funnel_radius(&sh, d) + r
            {
                return true;
            }
        }
    }
    false
}

// port: stinky_squirrel_reach
/// How far a stinky mushroom scares squirrels (grows with the column width).
pub fn stinky_squirrel_reach(sim: &Sim) -> f64 {
    let top = stinky_shape(sim).top;
    let a0 = stinky_top_at(0.0);
    let span = stinky_top_at(4.0) - a0;
    let k = (top - stinky_top_at(0.0)) / span.max(1.0);
    gml::lerp(
        sim.bal("stinky_squirrel_reach_px"),
        sim.bal("stinky_squirrel_reach_max_px"),
        gml::clamp(k, 0.0, 1.0),
    )
}

// port: stinky_guards
/// Whether another stinky mushroom guards mushroom `idx` from squirrels.
pub fn stinky_guards(sim: &Sim, idx: usize) -> bool {
    let reach = stinky_squirrel_reach(sim);
    let sx = sim.state.mushrooms[idx].x;
    sim.state.mushrooms.iter().enumerate().any(|(i, m)| {
        i != idx && m.kind == MushroomKind::Stinky.code() && !m.stolen && (m.x - sx).abs() <= reach
    })
}
