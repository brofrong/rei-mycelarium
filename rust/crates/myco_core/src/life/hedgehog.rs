//! Hedgehogs (scr_life): they walk across the world and carry off the first mushroom they reach.

use super::art::art;
use super::{animal_interval, animal_timer_step, animals_rebind, squirrel_view, Hedgehog};
use crate::balance::{vis, wld};
use crate::gml;
use crate::mycelium::source;
use crate::sim::Sim;
use crate::state::MushroomSave;

// port: hedgehog_frame_w
/// Width of one hedgehog frame on screen (8 frames side by side).
pub fn hedgehog_frame_w() -> f64 {
    art().sprite("hedgehog").w / 8.0 * vis("hedgehog_scale")
}

// port: hedgehog_frame_h
pub fn hedgehog_frame_h() -> f64 {
    art().sprite("hedgehog").h * vis("hedgehog_scale")
}

// port: hedgehog_back_y
/// Height of the back above the feet (fallback carry point).
pub fn hedgehog_back_y() -> f64 {
    -hedgehog_frame_h() * 0.68
}

// port: hedgehog_back_point
/// Where a carried mushroom sits on the back for this frame (the back line is the art data
/// `hedgehog_back_init` measures; it is always available here).
pub fn hedgehog_back_point(x: f64, y: f64, dir: f64, frame: f64) -> (f64, f64) {
    let a = art();
    let s = vis("hedgehog_scale");
    let spr = a.sprite("hedgehog");
    let fw = spr.w / 8.0;
    let fh = spr.h;
    let sx = if dir >= 0.0 { -1.0 } else { 1.0 };
    let p = a.hedgehog_back[gml::clamp(frame, 0.0, 7.0).floor() as usize];
    (
        x.floor() + sx * (p[0] + vis("hedgehog_carry_back_px") - fw * 0.5) * s,
        y.floor() - fh * s + (p[1] + vis("hedgehog_carry_sink_px")) * s,
    )
}

// port: hedgehog_world_span
/// The x range the camera can show (`camera_bounds` plus the view width): animals enter and
/// leave beyond it. Returns `(left, right)`.
pub fn hedgehog_world_span(sim: &Sim) -> (f64, f64) {
    let (vw, vh) = (sim.camera.width, sim.camera.height);
    let b = crate::camera::camera_bounds(sim, vw, vh);
    (b.x1, b.x2 + vw)
}

// port: hedgehog_border_margin
pub fn hedgehog_border_margin() -> f64 {
    hedgehog_frame_w() + 60.0
}

fn push(sim: &mut Sim, x: f64, dir: f64) {
    let id = sim.life.new_id(source::ANIMAL);
    sim.life.animals.push(Hedgehog {
        id,
        x,
        y: wld("surface_height_px"),
        dir,
        speed: sim.bal("hedgehog_speed"),
        frame: 0.0,
        frame_timer: vis("hedgehog_frame_interval"),
        carrying: -1,
    });
}

// port: animal_spawn_hedgehog
/// A hedgehog from a random side, beyond the world span.
pub fn animal_spawn_hedgehog(sim: &mut Sim) {
    let (left, right) = hedgehog_world_span(sim);
    let margin = hedgehog_border_margin();
    let from_left = sim.rng.choose(&[true, false]);
    let x = if from_left {
        left - margin
    } else {
        right + margin
    };
    push(sim, x, if from_left { 1.0 } else { -1.0 });
}

// port: hedgehog_summon
/// Debug: a hedgehog just off screen walking toward the mushroom nearest the view centre.
pub fn hedgehog_summon(sim: &mut Sim) -> &'static str {
    let sv = squirrel_view(sim);
    let cx = sv.x + sv.w * 0.5;
    let mut best: Option<f64> = None;
    for m in &sim.state.mushrooms {
        if hedgehog_takes(m)
            && m.carried_by == -1
            && !m.stolen
            && best.is_none_or(|bx| (m.x - cx).abs() < (bx - cx).abs())
        {
            best = Some(m.x);
        }
    }
    let dir = match best {
        Some(bx) => {
            if bx >= cx {
                1.0
            } else {
                -1.0
            }
        }
        None => sim.rng.choose(&[-1.0, 1.0]),
    };
    let margin = hedgehog_frame_w() + 40.0;
    let x = if !(dir > 0.0) {
        sv.x + sv.w + margin
    } else {
        sv.x - margin
    };
    push(sim, x, dir);
    if best.is_some() {
        "hedgehog"
    } else {
        "hedgehog (nothing to carry)"
    }
}

// port: hedgehog_takes
/// Hedgehogs leave tree mushrooms, consumptors and diggers.
pub fn hedgehog_takes(m: &MushroomSave) -> bool {
    m.kind != 3 && m.kind != 7 && m.kind != 4
}

// port: animals_step
/// Spawn on the timer; walk, animate, pick up a mushroom within reach, carry it on the back;
/// a hedgehog past the world span leaves, with its mushroom.
pub fn animals_step(sim: &mut Sim) {
    let step = animal_timer_step(sim);
    sim.state.state.timers.hedgehog -= step;
    if sim.state.state.timers.hedgehog <= 0.0 {
        sim.state.state.timers.hedgehog = animal_interval(sim, "hedgehog_spawn_interval");
        animal_spawn_hedgehog(sim);
    }
    let (left, right) = hedgehog_world_span(sim);
    let margin = hedgehog_border_margin();
    let reach = sim.bal("hedgehog_reach_px");
    let animals = std::mem::take(&mut sim.life.animals);
    let mut keep = Vec::with_capacity(animals.len());
    // the reindexing in the loop sees the live array: the kept ones plus the rest
    let mut rest: std::collections::VecDeque<Hedgehog> = animals.into();
    let mut i = 0i64;
    while let Some(mut a) = rest.pop_front() {
        a.x += a.dir * a.speed;
        a.frame_timer -= 1.0;
        if a.frame_timer <= 0.0 {
            a.frame_timer = vis("hedgehog_frame_interval");
            a.frame = gml::gmod(a.frame + 1.0, 6.0);
        }
        for (j, m) in sim.state.mushrooms.iter_mut().enumerate() {
            if m.kind != 3
                && m.carried_by == -1
                && (m.x - a.x).abs() <= reach
                && !(m.kind == 2 && m.active)
                && hedgehog_takes(m)
                && a.carrying == -1
                && !m.crow_target
                && !m.stolen
            {
                m.carried_by = i;
                a.carrying = j as i64;
            }
        }
        let n = sim.state.mushrooms.len() as i64;
        if a.carrying != -1 && a.carrying < n {
            let c = &mut sim.state.mushrooms[a.carrying as usize];
            let p = hedgehog_back_point(a.x, a.y, a.dir, a.frame);
            c.x = p.0;
            c.y = p.1;
            c.extra.insert(
                "carry_angle".into(),
                (-a.dir * vis("hedgehog_carry_angle")).into(),
            );
        }
        let gone = if a.dir < 0.0 && a.x < left - margin {
            true
        } else if a.dir > 0.0 {
            a.x > right + margin
        } else {
            false
        };
        if gone {
            if a.carrying != -1 && a.carrying < n {
                let removed = a.carrying;
                sim.state.mushrooms.remove(removed as usize);
                // `animals_reindex_after_removal` runs over `global.animals`, which still holds
                // every hedgehog of this step (kept, this one and the rest)
                for h in keep
                    .iter_mut()
                    .chain(std::iter::once(&mut a))
                    .chain(rest.iter_mut())
                {
                    if h.carrying == removed {
                        h.carrying = -1;
                    } else if h.carrying > removed {
                        h.carrying -= 1;
                    }
                }
            }
        } else {
            keep.push(a);
        }
        i += 1;
    }
    sim.life.animals = keep;
    animals_rebind(sim);
}
