//! Boars (scr_life): they charge across the world and fling the mushrooms in their way; in
//! survival they come in packs. A walking boar keeps its tremor loop going.

use super::art::art;
use super::hedgehog::hedgehog_world_span;
use super::{
    animal_interval, animal_timer_step, animals_rebind, animals_reindex_after_removal, boar_timer,
    mushroom_idx, squirrel_view, Boar, Flung, PackBoar,
};
use crate::balance::{game_mode, vis, wld};
use crate::events::{AudioCall, SimEvent};
use crate::gml;
use crate::mushrooms::types::{
    mushroom_cell_width, mushroom_sprite_now, mushroom_variant, shroom_sprite,
};
use crate::mycelium::source;
use crate::rng::Rng;
use crate::sim::Sim;
use crate::state::{GameMode, MushroomSave};

// port: boar_frame_w
/// Width of one boar frame on screen (6 frames side by side).
pub fn boar_frame_w() -> f64 {
    art().sprite("boar").w / 6.0 * vis("boar_scale")
}

// port: boar_hits_mushroom
/// Boars run past tree mushrooms and consumptors, and what is carried, stolen or targeted.
pub fn boar_hits_mushroom(m: &MushroomSave) -> bool {
    m.kind != 3 && m.kind != 7 && m.carried_by == -1 && !m.stolen && !m.crow_target
}

// port: boar_any_target
pub fn boar_any_target(sim: &Sim) -> bool {
    sim.state.mushrooms.iter().any(boar_hits_mushroom)
}

/// `array_shuffle` (GML; the LÖVE shim draws from the game RNG the same way): Fisher–Yates from
/// the end with `j = floor(random(1) * (i + 1))`.
pub fn array_shuffle<T>(rng: &mut Rng, v: &mut [T]) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng.random(1.0) * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

// port: boar_pick_targets
/// Up to `boar_hits` random mushrooms ahead of `x` in `dir` (ids).
pub fn boar_pick_targets(sim: &mut Sim, x: f64, dir: f64) -> Vec<u64> {
    let mut list: Vec<u64> = sim
        .state
        .mushrooms
        .iter()
        .filter(|m| boar_hits_mushroom(m) && (m.x - x) * dir > 0.0 && !(m.kind == 2 && m.active))
        .map(|m| m.live.id)
        .collect();
    array_shuffle(&mut sim.rng, &mut list);
    let keep = (list.len() as f64).min(sim.bal("boar_hits")).max(0.0) as usize;
    list.truncate(keep);
    list
}

// port: boar_spawn
pub fn boar_spawn(sim: &mut Sim, x: f64, dir: f64) {
    let targets = boar_pick_targets(sim, x, dir);
    let id = sim.life.new_id(source::ANIMAL);
    sim.life.boars.push(Boar {
        id,
        targets,
        x,
        y: wld("surface_height_px"),
        dir,
        frame: 0.0,
        frame_timer: vis("boar_frame_interval"),
        hits: 0.0,
        fleeing: false,
    });
}

// port: boar_spawn_pack
/// A boar; in survival the rest of its pack follows `boar_pack_gap_seconds` apart.
pub fn boar_spawn_pack(sim: &mut Sim, x: f64, dir: f64) {
    boar_spawn(sim, x, dir);
    if game_mode(&sim.state) != GameMode::Survival {
        return;
    }
    let gap = 1f64.max(gml::round(vis("boar_pack_gap_seconds") * sim.fps));
    let mut i = 1.0;
    while i < 1f64.max(vis("boar_pack_size")) {
        sim.life.boar_pack.push(PackBoar {
            x,
            dir,
            wait: gap * i,
        });
        i += 1.0;
    }
}

// port: boar_summon
/// Debug: a boar (pack) just off screen charging toward the mushroom nearest the view centre.
pub fn boar_summon(sim: &mut Sim) -> &'static str {
    let v = squirrel_view(sim);
    let cx = v.x + v.w * 0.5;
    let mut target: Option<f64> = None;
    for m in &sim.state.mushrooms {
        if boar_hits_mushroom(m) && target.is_none_or(|tx| (m.x - cx).abs() < (tx - cx).abs()) {
            target = Some(m.x);
        }
    }
    let dir = match target {
        None => sim.rng.choose(&[-1.0, 1.0]),
        Some(tx) => {
            if tx < cx {
                -1.0
            } else {
                1.0
            }
        }
    };
    let margin = boar_frame_w() + 40.0;
    let x = if dir > 0.0 {
        v.x - margin
    } else {
        v.x + v.w + margin
    };
    boar_spawn_pack(sim, x, dir);
    if target.is_none() {
        "boar (nothing to hit)"
    } else {
        "boar"
    }
}

// port: boar_fling
/// Mushroom `mi` flies off the boar's snout (its look is kept for the drawing).
pub fn boar_fling(sim: &mut Sim, mi: usize, dir: f64) {
    let m = &sim.state.mushrooms[mi];
    let spr = mushroom_sprite_now(m);
    let meta = shroom_sprite(spr).expect("mushroom sprite");
    let cw = mushroom_cell_width(spr);
    let (kind, mx, my) = (m.kind, m.x, m.y);
    let variant = mushroom_variant(m);
    let x = mx - meta.xoff + cw * 0.5;
    let y = my - meta.yoff + meta.h * 0.5;
    let base_vx = sim.bal("boar_speed") * 0.5;
    let rng = &mut sim.rng;
    let vx = dir * (base_vx + rng.random_range(vis("boar_fling_vx_min"), vis("boar_fling_vx_max")));
    let vy = -rng.random_range(vis("boar_fling_vy_min"), vis("boar_fling_vy_max"));
    let spin = -dir * rng.random_range(vis("boar_fling_spin_min"), vis("boar_fling_spin_max"));
    sim.life.boar_flung.push(Flung {
        kind,
        sprite: spr,
        variant,
        x,
        y,
        vx,
        vy,
        angle: 0.0,
        spin,
        life: vis("boar_fling_frames"),
    });
}

// port: boars_step
/// Spawn (while there is something to hit), release waiting pack members, charge, fling the
/// targets the snout passes, leave beyond the world span; flung mushrooms fly and fade.
pub fn boars_step(sim: &mut Sim) {
    if boar_any_target(sim) {
        let step = animal_timer_step(sim);
        sim.state.state.timers.boar = Some(boar_timer(sim) - step);
        if boar_timer(sim) <= 0.0 {
            sim.state.state.timers.boar = Some(animal_interval(sim, "boar_spawn_interval"));
            let (left, right) = hedgehog_world_span(sim);
            let m = boar_frame_w() + 60.0;
            let from_left = sim.rng.choose(&[true, false]);
            let sx = if from_left { left - m } else { right + m };
            boar_spawn_pack(sim, sx, if from_left { 1.0 } else { -1.0 });
        }
    }
    let pack = std::mem::take(&mut sim.life.boar_pack);
    let mut pack_next = Vec::new();
    for mut p in pack {
        p.wait -= 1.0;
        if !(p.wait <= 0.0) {
            pack_next.push(p);
        } else {
            boar_spawn(sim, p.x, p.dir);
        }
    }
    sim.life.boar_pack = pack_next;
    let (left, right) = hedgehog_world_span(sim);
    let m = boar_frame_w() + 60.0;
    let fw = boar_frame_w();
    let snout_dx = vis("boar_snout_dx");
    let speed = sim.bal("boar_speed");
    let reach = sim.bal("boar_reach_px");
    let mut bi = 0;
    while bi < sim.life.boars.len() {
        let (snout0, snout1, id, bx, by) = {
            let b = &mut sim.life.boars[bi];
            let snout0 = b.x + b.dir * fw * snout_dx;
            b.x += b.dir * speed;
            let snout1 = b.x + b.dir * fw * snout_dx;
            (snout0, snout1, b.id, b.x, b.y)
        };
        sim.emit(SimEvent::Audio(AudioCall::LoopFollow {
            id,
            sound: "boar_tremor",
            x: bx,
            y: by,
            volume: vis("boar_tremor_volume"),
            reach: vis("boar_tremor_reach_views"),
            fade_in: vis("boar_tremor_fade_in_frames"),
            fade_out: vis("boar_tremor_fade_out_frames"),
        }));
        let b = &mut sim.life.boars[bi];
        b.frame_timer -= 1.0;
        if b.frame_timer <= 0.0 {
            b.frame_timer = vis("boar_frame_interval");
            b.frame = gml::gmod(b.frame + 1.0, 6.0);
        }
        if !b.fleeing && !b.targets.is_empty() {
            let lo = snout0.min(snout1) - reach;
            let hi = snout0.max(snout1) + reach;
            let dir = b.dir;
            let targets = std::mem::take(&mut b.targets);
            let mut left_t = Vec::new();
            let mut hits = 0.0;
            for t in targets {
                let Some(idx) = mushroom_idx(sim, t) else {
                    continue;
                };
                let tm = &sim.state.mushrooms[idx];
                if !boar_hits_mushroom(tm) {
                    continue;
                }
                if tm.x < lo || tm.x > hi || (tm.kind == 2 && tm.active) {
                    left_t.push(t);
                } else {
                    boar_fling(sim, idx, dir);
                    sim.state.mushrooms.remove(idx);
                    animals_reindex_after_removal(sim, idx as i64);
                    hits += 1.0;
                }
            }
            let b = &mut sim.life.boars[bi];
            b.targets = left_t;
            b.hits += hits;
            animals_rebind(sim);
        }
        let b = &sim.life.boars[bi];
        let gone = (b.dir < 0.0 && b.x < left - m) || (b.dir > 0.0 && b.x > right + m);
        if gone {
            sim.life.boars.remove(bi);
        } else {
            bi += 1;
        }
    }
    let g = vis("boar_fling_gravity");
    sim.life.boar_flung.retain_mut(|f| {
        f.x += f.vx;
        f.y += f.vy;
        f.vy += g;
        f.vx *= 0.985;
        f.angle += f.spin;
        f.life -= 1.0;
        f.life > 0.0
    });
}
