//! Worms (scr_life): they crawl through soil near the view, leave a trail, come up in the rain,
//! and a strand that touches one catches it (protein): a tendril wraps it and it is absorbed.

use super::{life_near_zone, squirrel_view, Pt, TendrilOut, TrailDot, Worm};
use crate::balance::{vis, wld};
use crate::events::{AudioCall, ResourceKind, SimEvent};
use crate::gml;
use crate::mycelium::growth::{get_mycelium_hit, Hit};
use crate::mycelium::shot::rasterize_branch;
use crate::mycelium::{source, Cell};
use crate::progress::upgrades::{upg_bonus, upg_is_owned};
use crate::sim::Sim;
use crate::world::Tile;

/// The directions a blocked worm tries, in order (times its side).
const MOVE_OFFSETS: [f64; 9] = [30.0, -30.0, 60.0, -60.0, 90.0, -90.0, 135.0, -135.0, 180.0];

// port: worm_zone_top
pub fn worm_zone_top(sim: &Sim) -> f64 {
    wld("surface_height_px") + sim.bal("worm_depth_min_px")
}

// port: worm_zone_bottom
pub fn worm_zone_bottom(sim: &Sim) -> f64 {
    wld("surface_height_px") + sim.bal("worm_depth_screens") * vis("screen_height_px")
}

// port: worm_cell_ok
/// A soil or hard-soil cell within the worm zone (from `top`, default the zone top).
pub fn worm_cell_ok(sim: &mut Sim, x: f64, y: f64, top: Option<f64>) -> bool {
    let top = top.unwrap_or_else(|| worm_zone_top(sim));
    if y < top || y > worm_zone_bottom(sim) {
        return false;
    }
    matches!(sim.world.tile_at(x, y), Tile::Soil | Tile::HardSoil)
}

// port: worm_segment_px
pub fn worm_segment_px() -> f64 {
    let m = 1f64.max(vis("worm_segments") - 1.0);
    vis("worm_length_px") / m
}

// port: worm_add
/// A worm with its head at (`x`, `y`) and its body trailing behind `dir`; its index.
pub fn worm_add(sim: &mut Sim, x: f64, y: f64, dir: f64) -> usize {
    let n = 2f64.max(vis("worm_segments"));
    let seg = worm_segment_px();
    let mut body = Vec::new();
    let mut i = 0.0;
    while i < n {
        body.push(Pt {
            x: x - gml::lengthdir_x(seg * i, dir),
            y: y - gml::lengthdir_y(seg * i, dir),
        });
        i += 1.0;
    }
    let side = sim.rng.choose(&[-1.0, 1.0]);
    let phase = sim.rng.random(360.0);
    let check = sim.rng.irandom(sim.bal("worm_catch_check_frames")) as f64;
    let id = sim.life.new_id(source::WORM);
    sim.life.worms.push(Worm {
        id,
        body,
        dir,
        side,
        phase,
        check,
        total: 1.0,
        ..Default::default()
    });
    sim.life.worms.len() - 1
}

// port: worm_keep_bottom
/// How deep worms live below a view at `vy` of height `vh` (never below the zone).
pub fn worm_keep_bottom(sim: &Sim, vy: f64, vh: f64) -> f64 {
    let zb = worm_zone_bottom(sim);
    zb.min(vy + vh * (1.0 + sim.bal("worm_spawn_below_screens")))
}

// port: worm_spawn
/// A worm in the near zone off screen (or anywhere in view, `in_view`), in soil, up to 12 tries.
pub fn worm_spawn(sim: &mut Sim, in_view: bool) -> Option<usize> {
    let v = squirrel_view(sim);
    let (vx, vy, vw, vh) = (v.x, v.y, v.w, v.h);
    let top = worm_zone_top(sim) + 8.0;
    let bot = worm_keep_bottom(sim, vy, vh) - 8.0;
    let margin = vis("worm_length_px") + 16.0;
    if !(bot > top || bot.is_nan() || top.is_nan()) {
        return None;
    }
    for _ in 0..12 {
        let zone = life_near_zone(sim);
        let x = if !in_view {
            sim.rng.random_range(zone.x1, zone.x2)
        } else {
            vx + sim.rng.random(vw)
        };
        let y = if !in_view {
            sim.rng.random_range(top, bot)
        } else {
            gml::clamp(vy + sim.rng.random(vh), top, bot)
        };
        let dir = sim.rng.random(360.0);
        let skip = !in_view
            && x > vx - margin
            && x < vx + vw + margin
            && y > vy - margin
            && y < vy + vh + margin;
        if !skip {
            let tx = x - gml::lengthdir_x(vis("worm_length_px"), dir);
            let ty = y - gml::lengthdir_y(vis("worm_length_px"), dir);
            if worm_cell_ok(sim, x, y, None) && worm_cell_ok(sim, tx, ty, None) {
                return Some(worm_add(sim, x, y, dir));
            }
        }
    }
    None
}

// port: worm_absorb_frames
pub fn worm_absorb_frames(sim: &Sim) -> f64 {
    let gf = 1f64.max(vis("leaf_tendril_grow_frames"));
    let sp = 1.0 + upg_bonus(&sim.state, "absorption_speed");
    (gf + 60.0).max(sim.bal("worm_absorb_time") / sp)
}

// port: worm_move
/// One crawl step of worm `wi`: wander, steer down (or up in the rain), turn aside from what is
/// not soil, leave trail dots, and pull the body after the head.
pub fn worm_move(sim: &mut Sim, wi: usize) {
    let rain = crate::weather::is_raining(sim);
    let top = worm_zone_top(sim);
    let head = sim.life.worms[wi].body[0];
    let floor_y = if !rain {
        top.min(head.y - 2.0)
    } else {
        wld("surface_height_px") + 4.0
    };
    let turn_d = sim.rng.random_range(-0.35, 0.35);
    let w = &mut sim.life.worms[wi];
    w.phase += vis("worm_wave_speed");
    w.turn = w.turn * 0.95 + turn_d;
    w.dir += w.turn;
    if !rain {
        if head.y < top {
            w.dir += gml::angle_difference(270.0, w.dir) * 0.03;
        }
    } else {
        w.dir += gml::angle_difference(90.0, w.dir) * 0.04;
    }
    let look = 6.0;
    let (dir, side) = (w.dir, w.side);
    let ahead = (
        head.x + gml::lengthdir_x(look, dir),
        head.y + gml::lengthdir_y(look, dir),
    );
    if !worm_cell_ok(sim, ahead.0, ahead.1, Some(floor_y)) {
        let mut found = false;
        for off in MOVE_OFFSETS {
            let nd = dir + off * side;
            let (lx, ly) = (
                head.x + gml::lengthdir_x(look, nd),
                head.y + gml::lengthdir_y(look, nd),
            );
            if worm_cell_ok(sim, lx, ly, Some(floor_y)) {
                let w = &mut sim.life.worms[wi];
                w.dir = nd;
                w.turn = 0.0;
                found = true;
                break;
            }
        }
        if !found {
            let w = &mut sim.life.worms[wi];
            w.side = -w.side;
            return;
        }
    }
    let phase = sim.life.worms[wi].phase;
    let mut spd = sim.bal("worm_speed") * (0.3 + 0.7 * 0f64.max(phase.sin()));
    if sim.world.tile_at(head.x, head.y) == Tile::HardSoil {
        spd *= sim.bal("worm_hardsoil_speed");
    }
    if rain {
        spd *= sim.bal("worm_rain_speed");
    }
    let w = &mut sim.life.worms[wi];
    let dir = w.dir;
    w.body[0].x += gml::lengthdir_x(spd, dir);
    w.body[0].y += gml::lengthdir_y(spd, dir);
    w.trail_walk += spd;
    let head = w.body[0];
    if w.trail_walk >= vis("worm_trail_spacing_px")
        && (sim.life.worm_trail.len() as f64) < vis("worm_trail_max")
    {
        sim.life.worms[wi].trail_walk = 0.0;
        let tx = head.x.floor();
        let ty = head.y.floor();
        let r = (1f64.max(vis("worm_thickness_px") - 1.0) / 2.0).trunc() + 1.0;
        let wet = sim.world.water_at(tx - r, ty - r)
            || sim.world.water_at(tx + r, ty - r)
            || sim.world.water_at(tx - r, ty + r)
            || sim.world.water_at(tx + r, ty + r);
        if !wet {
            sim.life.worm_trail.push(TrailDot {
                x: tx,
                y: ty,
                life: vis("worm_trail_frames"),
            });
        }
    }
    let seg = worm_segment_px();
    let st = vis("worm_stretch");
    let w = &mut sim.life.worms[wi];
    for i in 1..w.body.len() {
        let a = w.body[i - 1];
        let b = w.body[i];
        let d = seg * (1.0 + st * (w.phase - i as f64 * 0.5).sin());
        let ang = gml::point_direction(a.x, a.y, b.x, b.y);
        w.body[i] = Pt {
            x: a.x + gml::lengthdir_x(d, ang),
            y: a.y + gml::lengthdir_y(d, ang),
        };
    }
}

/// `worm_body_at(w, s)` (`___struct___100`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyAt {
    pub x: f64,
    pub y: f64,
    pub dir: f64,
}

// port: worm_body_at
/// The point `s` segments along the body (clamped), with the segment's direction.
pub fn worm_body_at(w: &Worm, s: f64) -> BodyAt {
    let body = &w.body;
    let last = body.len() as f64 - 1.0;
    let hi = 0f64.max(last - 1.0);
    let i = gml::clamp(s.floor(), 0.0, hi);
    let f = gml::clamp(s - i, 0.0, 1.0);
    let a = body[i as usize];
    let b = body[(i + 1.0).min(last) as usize];
    BodyAt {
        x: gml::lerp(a.x, b.x, f),
        y: gml::lerp(a.y, b.y, f),
        dir: gml::point_direction(a.x, a.y, b.x, b.y),
    }
}

// port: worm_tendril_make
/// The catching tendril from the mycelium hit (`hx`, `hy`) to the nearest body point, then
/// wrapping along the body; sets the worm's `touch_frames`. Returns its cells.
pub fn worm_tendril_make(sim: &mut Sim, wi: usize, hx: f64, hy: f64) -> Vec<Cell> {
    let w = &sim.life.worms[wi];
    let last = w.body.len() - 1;
    let mut best = 0usize;
    let mut bd = f64::INFINITY;
    for (i, p) in w.body.iter().enumerate() {
        let d = gml::point_distance(hx, hy, p.x, p.y);
        if d < bd {
            bd = d;
            best = i;
        }
    }
    let (bx, by) = (w.body[best].x, w.body[best].y);
    let mut path = vec![Pt { x: hx, y: hy }];
    let dir = gml::point_direction(hx, hy, bx, by);
    let n = 3f64.max(gml::round(bd / 10.0));
    let mut wob = 0.0;
    let mut k = 1.0;
    while k <= n {
        let t = k / n;
        wob = wob * 0.55 + sim.rng.random_range(-1.0, 1.0) * bd * 0.06;
        let off = wob * (1.0 - (t - 0.5).abs() * 2.0);
        path.push(Pt {
            x: gml::lerp(hx, bx, t) + gml::lengthdir_x(off, dir + 90.0),
            y: gml::lerp(hy, by, t) + gml::lengthdir_y(off, dir + 90.0),
        });
        k += 1.0;
    }
    let npath = path.len();
    let (lastf, bestf) = (last as f64, best as f64);
    let fwd = !(bestf > lastf * 0.5);
    let rem = if !fwd { lastf - bestf } else { bestf };
    let span = rem + lastf;
    let steps = 16f64.max(gml::round(span * worm_segment_px()));
    let th = vis("worm_thickness_px") * 0.5 + 1.0;
    let wraps = vis("worm_tendril_wraps");
    let w = &sim.life.worms[wi];
    let mut j = 1.0;
    while j <= steps {
        let s = j / steps * span;
        let idx = if fwd {
            if s > rem {
                s - rem
            } else {
                bestf - s
            }
        } else if s > rem {
            lastf - (s - rem)
        } else {
            bestf + s
        };
        let p = worm_body_at(w, idx);
        let r = (s / span * std::f64::consts::PI * 2.0 * wraps).sin() * th;
        path.push(Pt {
            x: p.x + gml::lengthdir_x(r, p.dir + 90.0),
            y: p.y + gml::lengthdir_y(r, p.dir + 90.0),
        });
        j += 1.0;
    }
    let mut total = 0.0;
    let mut acc = 0.0;
    for q in 1..path.len() {
        let seg = gml::point_distance(path[q - 1].x, path[q - 1].y, path[q].x, path[q].y);
        acc += seg;
        if q < npath {
            total += seg;
        }
    }
    sim.life.worms[wi].touch_frames = Some(gml::round(
        1f64.max(vis("leaf_tendril_grow_frames")) * total / 1f64.max(acc),
    ));
    rasterize_branch(&path, vis("strand_thickness"))
}

// port: worm_catch
/// The mycelium touched worm `wi` at `src`: it is caught, wrapped and starts being absorbed.
pub fn worm_catch(sim: &mut Sim, wi: usize, src: Hit) {
    let gf = 1f64.max(vis("leaf_tendril_grow_frames"));
    let frames = worm_absorb_frames(sim);
    let w = &mut sim.life.worms[wi];
    let mid = w.body[w.body.len() / 2];
    w.caught = true;
    w.timer = frames;
    w.total = frames;
    w.grow_frames = Some(gf);
    w.struggle = vis("worm_struggle_frames");
    let tendril = worm_tendril_make(sim, wi, src.x, src.y);
    sim.life.worms[wi].tendril = Some(tendril);
    let d = gml::point_distance(src.x, src.y, mid.x, mid.y);
    let id = sim.life.worms[wi].id;
    let (ev, _) = crate::audio::hyphae_begin(&mut sim.rng, id, d, gf, Some((mid.x, mid.y)));
    sim.emit(ev);
    sim.life.worms[wi].sound = true;
}

// port: worms_step
/// Spawn on the timer (up to `worm_max_count`); caught worms are absorbed (food); free worms in
/// the near zone crawl and, with protein, are checked against the mycelium now and then; worms
/// out of the zone are dropped; trail dots fade.
pub fn worms_step(sim: &mut Sim) {
    sim.state.state.timers.worm -= 1.0;
    if !(sim.state.state.timers.worm > 0.0) {
        sim.state.state.timers.worm = sim.bal("worm_spawn_interval");
        if (sim.life.worms.len() as f64) < sim.bal("worm_max_count") {
            worm_spawn(sim, false);
        }
    }
    let v = squirrel_view(sim);
    let keep_y = worm_keep_bottom(sim, v.y, v.h) + v.h * 0.5;
    let zone = life_near_zone(sim);
    let can_catch = upg_is_owned(&sim.state, "unlock_protein");
    let catch_radius = sim.bal("worm_catch_radius");
    let mut wi = 0;
    while wi < sim.life.worms.len() {
        let mut keep = true;
        if sim.life.worms[wi].caught {
            let w = &mut sim.life.worms[wi];
            w.timer -= 1.0;
            if w.total - w.timer >= w.touch_frames.unwrap_or(f64::NAN) {
                w.struggle = 0f64.max(w.struggle - 1.0);
            }
            let (id, t, timer, sound) = (w.id, w.total - w.timer, w.timer, w.sound);
            if sound {
                sim.emit(SimEvent::Audio(AudioCall::HyphaeStep { id, t }));
            }
            if timer > 0.0 || timer.is_nan() {
                // kept
            } else {
                if sound {
                    sim.emit(SimEvent::Audio(AudioCall::HyphaeStop { id }));
                }
                let food = sim.bal("worm_food_reward");
                sim.food_add(food);
                let w = &sim.life.worms[wi];
                let mid = w.body[w.body.len() / 2];
                sim.emit(SimEvent::Resource {
                    kind: ResourceKind::Food,
                    delta: food,
                    x: mid.x,
                    y: mid.y,
                    source: id,
                });
                let rf = 1f64.max(vis("leaf_tendril_retract_frames"));
                let cells = sim.life.worms[wi].tendril.take().unwrap_or_default();
                sim.life.leaf_tendrils_out.push(TendrilOut {
                    cells,
                    timer: rf,
                    total: rf,
                });
                keep = false;
            }
        } else {
            let head = sim.life.worms[wi].body[0];
            if !(head.x < zone.x1 || head.x > zone.x2 || head.y > keep_y) {
                worm_move(sim, wi);
                if can_catch {
                    let check_frames = sim.bal("worm_catch_check_frames");
                    let w = &mut sim.life.worms[wi];
                    w.check -= 1.0;
                    if w.check <= 0.0 {
                        w.check = check_frames;
                        let mid = w.body[w.body.len() / 2];
                        let half = vis("worm_length_px") * 0.5;
                        let r = catch_radius + half;
                        let hit = get_mycelium_hit(sim, mid.x, mid.y, r);
                        if hit.hit {
                            worm_catch(sim, wi, hit);
                        }
                    }
                }
            } else {
                keep = false;
            }
        }
        if keep {
            wi += 1;
        } else {
            sim.life.worms.remove(wi);
        }
    }
    sim.life.worm_trail.retain_mut(|t| {
        t.life -= 1.0;
        t.life > 0.0
    });
}
