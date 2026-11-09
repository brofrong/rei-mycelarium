//! Crows (scr_life): announced a few seconds ahead, they dive at a mushroom, grab it and fly
//! off with it; experience mushrooms have their own crows. A stinky column scares them away.

use super::art::art;
use super::{
    animal_interval, animal_timer_step, animals_rebind, mushroom_delete, mushroom_idx,
    squirrel_view, Crow, CrowPending, CrowState,
};
use crate::balance::vis;
use crate::gml;
use crate::mushrooms::stinky::stinky_beam_hits;
use crate::mushrooms::types::{shroom_sprite, sprite_of_code};
use crate::mycelium::source;
use crate::progress::tasks;
use crate::sim::{Sim, ViewRect};

/// The claw tip per crow frame (sprite px, `crow_claw` static table).
const CLAWS: [(f64, f64); 9] = [
    (74.0, 83.0),
    (70.0, 82.0),
    (70.0, 80.0),
    (70.0, 77.0),
    (76.0, 95.0),
    (76.0, 81.0),
    (74.0, 79.0),
    (74.0, 81.0),
    (74.0, 83.0),
];

// port: crow_frame_w
/// Width of one crow frame (9 frames, integer division).
pub fn crow_frame_w() -> f64 {
    (art().sprite("crow").w / 9.0).trunc()
}

// port: crow_claw
pub fn crow_claw(frame: f64) -> (f64, f64) {
    CLAWS[gml::clamp(frame, 0.0, 8.0).floor() as usize]
}

// port: crow_below_claws
/// Sprite height below the claws (scaled).
pub fn crow_below_claws() -> f64 {
    let c = crow_claw(0.0);
    (art().sprite("crow").h - c.1) * vis("crow_scale")
}

// port: crow_claw_offset
/// How far the claws of `frame` sit from those of frame 0 (world px, facing `dir`).
pub fn crow_claw_offset(frame: f64, dir: f64) -> (f64, f64) {
    let sc = vis("crow_scale");
    let c0 = crow_claw(0.0);
    let c = crow_claw(frame);
    let ndir = -dir;
    ((c.0 - c0.0) * ndir * sc, (c.1 - c0.1) * sc)
}

// port: crow_cap_dy
/// From a mushroom's foot to where the crow grabs its cap.
pub fn crow_cap_dy(kind: i64) -> f64 {
    let spr = shroom_sprite(sprite_of_code(kind)).expect("mushroom sprite");
    spr.bbox_t - spr.yoff + vis("crow_grab_into_cap_px")
}

// port: crow_can_take
pub fn crow_can_take(sim: &Sim, mi: usize) -> bool {
    let m = &sim.state.mushrooms[mi];
    if m.stolen || m.crow_target || m.carried_by != -1 {
        return false;
    }
    let id = m.live.id;
    !sim.life.squirrels.iter().any(|s| s.mushroom == Some(id))
}

// port: crow_pick
/// A random mushroom a crow can take (of `kind`, if given), preferring those in view (index).
pub fn crow_pick(sim: &mut Sim, v: &ViewRect, kind: Option<i64>) -> Option<usize> {
    let mut in_view = Vec::new();
    let mut any = Vec::new();
    for i in 0..sim.state.mushrooms.len() {
        if !crow_can_take(sim, i) {
            continue;
        }
        let m = &sim.state.mushrooms[i];
        if kind.is_some_and(|k| m.kind != k) {
            continue;
        }
        any.push(i);
        if m.x > v.x + 40.0 && m.x < v.x + v.w - 40.0 && m.y > v.y && m.y < v.y + v.h {
            in_view.push(i);
        }
    }
    let pool = if !in_view.is_empty() { in_view } else { any };
    if pool.is_empty() {
        return None;
    }
    Some(pool[sim.rng.irandom((pool.len() - 1) as f64) as usize])
}

// port: crow_sky_y
/// Above the view (and above the sky top) by `h` and a little.
pub fn crow_sky_y(v: &ViewRect, h: f64) -> f64 {
    0f64.min(v.y) - h - 10.0
}

// port: crow_spawn
/// A crow starting high to one side of mushroom `mi`, which becomes its target. `id`: the id
/// of the warning it fulfils (a fresh one otherwise).
pub fn crow_spawn(sim: &mut Sim, mi: usize, v: &ViewRect, dir: Option<f64>, id: Option<u64>) {
    let dir = match dir {
        Some(d) => d,
        None => sim.rng.choose(&[-1.0, 1.0]),
    };
    let below = crow_below_claws();
    let y0 = crow_sky_y(v, below);
    let m = &mut sim.state.mushrooms[mi];
    let x0 = m.x - dir * vis("crow_dive_dx_px");
    m.crow_target = true;
    let grab_y = m.y + crow_cap_dy(m.kind);
    let (gx, mid) = (m.x, m.live.id);
    let id = id.unwrap_or_else(|| sim.life.new_id(source::ANIMAL));
    sim.life.crows.push(Crow {
        id,
        state: CrowState::Dive,
        dir,
        x: x0,
        y: y0,
        x0,
        y0,
        grab_x: gx,
        grab_y,
        frame_timer: vis("crow_frame_interval"),
        mushroom: Some(mid),
        ..Default::default()
    });
}

// port: crow_warn_check
/// `crow_warning_seconds` before slot `slot`'s crow is due, pick its mushroom and announce it.
pub fn crow_warn_check(sim: &mut Sim, slot: usize, timer: f64, v: &ViewRect, kind: Option<i64>) {
    if sim.life.crow_pending[slot].is_some() {
        return;
    }
    if timer / animal_timer_step(sim) > vis("crow_warning_seconds") * sim.fps {
        return;
    }
    let Some(mi) = crow_pick(sim, v, kind) else {
        return;
    };
    let dir = sim.rng.choose(&[-1.0, 1.0]);
    let m = &sim.state.mushrooms[mi];
    let (mid, mx) = (m.live.id, m.x);
    let id = sim.life.new_id(source::ANIMAL);
    sim.life.crow_pending[slot] = Some(CrowPending {
        id,
        mushroom: mid,
        dir,
        x: mx - dir * vis("crow_dive_dx_px"),
        y: crow_sky_y(v, crow_below_claws()),
    });
}

// port: crow_due
/// Slot `slot`'s crow comes: for the announced mushroom if it can still be taken (the crow keeps
/// the warning's id, so its marker carries on), else for a fresh pick.
pub fn crow_due(sim: &mut Sim, slot: usize, v: &ViewRect, kind: Option<i64>) {
    let p = sim.life.crow_pending[slot].take();
    if let Some(p) = p {
        if let Some(mi) = mushroom_idx(sim, p.mushroom) {
            if crow_can_take(sim, mi) {
                crow_spawn(sim, mi, v, Some(p.dir), Some(p.id));
                return;
            }
        }
    }
    if let Some(mi) = crow_pick(sim, v, kind) {
        crow_spawn(sim, mi, v, None, None);
    }
}

// port: crow_summon
/// Debug: a crow for a random mushroom.
pub fn crow_summon(sim: &mut Sim) -> &'static str {
    let v = squirrel_view(sim);
    let Some(mi) = crow_pick(sim, &v, None) else {
        return "crow: no free mushroom";
    };
    crow_spawn(sim, mi, &v, None, None);
    "crow"
}

// port: crows_step
/// The crow timers (general and experience) with their warnings; every crow dives, grabs and
/// climbs off; one above the sky takes its mushroom away.
pub fn crows_step(sim: &mut Sim) {
    let v = squirrel_view(sim);
    let any = (0..sim.state.mushrooms.len()).any(|i| crow_can_take(sim, i));
    if any {
        let step = animal_timer_step(sim);
        sim.state.state.timers.crow -= step;
        let t = sim.state.state.timers.crow;
        crow_warn_check(sim, 0, t, &v, None);
        if sim.state.state.timers.crow <= 0.0 {
            sim.state.state.timers.crow = animal_interval(sim, "crow_spawn_interval");
            crow_due(sim, 0, &v, None);
        }
    }
    let any_exp = (0..sim.state.mushrooms.len())
        .any(|i| sim.state.mushrooms[i].kind == 6 && crow_can_take(sim, i));
    if any_exp {
        let mut k = 0;
        while k < sim.state.state.timers.experience_crows.len() {
            let step = animal_timer_step(sim);
            sim.state.state.timers.experience_crows[k] -= step;
            let t = sim.state.state.timers.experience_crows[k];
            crow_warn_check(sim, 1 + k, t, &v, Some(6));
            if sim.state.state.timers.experience_crows[k] <= 0.0 {
                sim.state.state.timers.experience_crows[k] =
                    animal_interval(sim, "experience_crow_interval");
                crow_due(sim, 1 + k, &v, Some(6));
            }
            k += 1;
        }
    }
    let crows = std::mem::take(&mut sim.life.crows);
    let mut keep = Vec::with_capacity(crows.len());
    for mut c in crows {
        let mut m = c.mushroom.and_then(|id| mushroom_idx(sim, id));
        if c.mushroom.is_some() && m.is_none() {
            c.mushroom = None;
        }
        match c.state {
            CrowState::Dive => {
                if let Some(mi) = m {
                    let mm = &sim.state.mushrooms[mi];
                    c.grab_x = mm.x;
                    c.grab_y = mm.y + crow_cap_dy(mm.kind);
                }
                let sc_up = vis("crow_body_up_px") * vis("crow_scale");
                let sc_pad = vis("crow_body_pad_px") * vis("crow_scale");
                if !stinky_beam_hits(sim, c.x, c.y - sc_up, sc_pad) {
                    let co = crow_claw_offset(4.0, c.dir);
                    let tx = c.grab_x - co.0;
                    let ty = c.grab_y - co.1;
                    let dist = 1f64.max(gml::point_distance(c.x0, c.y0, tx, ty));
                    c.t = 1f64.min(c.t + sim.bal("crow_dive_speed") / dist);
                    let e = 1.0 - (1.0 - c.t) * (1.0 - c.t);
                    c.x = gml::lerp(c.x0, tx, c.t);
                    c.y = gml::lerp(c.y0, ty, e);
                    c.frame = 3f64.min((c.t * 4.0).floor());
                    if c.t >= 1.0 {
                        c.state = CrowState::Grab;
                        c.frame = 4.0;
                        c.timer = vis("crow_grab_frames");
                        if let Some(mi) = m {
                            sim.state.mushrooms[mi].stolen = true;
                        }
                    }
                } else {
                    tasks::stat_add(sim, "ach_stinky_scares", 1.0);
                    if let Some(mi) = m {
                        sim.state.mushrooms[mi].crow_target = false;
                    }
                    c.mushroom = None;
                    m = None;
                    c.state = CrowState::Climb;
                    c.frame = 5.0;
                    c.frame_timer = vis("crow_frame_interval");
                    c.speed = sim.bal("crow_climb_speed") * vis("crow_flee_start_speed");
                    c.flee = true;
                }
            }
            CrowState::Grab => {
                c.timer -= 1.0;
                if c.timer <= 0.0 {
                    c.state = CrowState::Climb;
                    c.frame = 5.0;
                    c.frame_timer = vis("crow_frame_interval");
                }
            }
            CrowState::Climb => {
                c.frame_timer -= 1.0;
                if c.frame_timer <= 0.0 {
                    c.frame_timer = vis("crow_frame_interval");
                    c.frame = if c.frame >= 8.0 { 5.0 } else { c.frame + 1.0 };
                }
                let climb = sim.bal("crow_climb_speed");
                c.speed = climb.min(c.speed + climb / 1f64.max(vis("crow_climb_ease_frames")));
                let fwd = if !c.flee {
                    vis("crow_climb_forward")
                } else {
                    vis("crow_flee_forward")
                };
                c.x += c.dir * c.speed * fwd;
                c.y -= c.speed;
            }
        }
        if let Some(mi) = m {
            if sim.state.mushrooms[mi].stolen {
                let co = crow_claw_offset(c.frame, c.dir);
                let mm = &mut sim.state.mushrooms[mi];
                let cap = crow_cap_dy(mm.kind);
                mm.x = c.x + co.0;
                mm.y = c.y + co.1 - cap;
            }
        }
        let mut below = crow_below_claws();
        if let Some(mi) = m {
            let h = shroom_sprite(sprite_of_code(sim.state.mushrooms[mi].kind))
                .expect("mushroom sprite")
                .h;
            below = below.max(h);
        }
        if c.state == CrowState::Climb && c.y < crow_sky_y(&v, below) {
            if let Some(mi) = m {
                mushroom_delete(sim, mi);
                animals_rebind(sim);
            }
        } else {
            keep.push(c);
        }
    }
    sim.life.crows = keep;
}
