//! Falling leaves (scr_life): they drift down with the wind; one that lands on the mycelium
//! (absorption) is wrapped by a tendril and absorbed into food. Also the leaf art lookups the
//! backdrop leaves use (frame data from [`super::art`]).

use super::art::{art, LeafArtFrame};
use super::{life_near_zone, squirrel_view, Leaf, Pt, TendrilOut};
use crate::balance::{vis, wld};
use crate::events::{AudioCall, ResourceKind, SimEvent};
use crate::forest::{
    leaf_fg_carry, leaf_fg_drop, leaf_fg_frame, leaf_fg_mirror, BackdropLeaf, LeafFgPose,
};
use crate::gml;
use crate::mycelium::growth::get_mycelium_hit;
use crate::mycelium::shot::rasterize_branch;
use crate::mycelium::{source, Cell};
use crate::progress::upgrades::{upg_bonus, upg_is_owned};
use crate::sim::Sim;

// port: leaf_face_for_wind
/// The leaf face that suits the wind direction (`leaf_face_flip` swaps them).
pub fn leaf_face_for_wind(wind: f64) -> f64 {
    let face = if wind >= 0.0 { 1.0 } else { 0.0 };
    if vis("leaf_face_flip") > 0.0 {
        1.0 - face
    } else {
        face
    }
}

// port: leaf_face_sprite
pub fn leaf_face_sprite(face: f64) -> &'static str {
    if face != 0.0 {
        "leaf_2"
    } else {
        "leaf"
    }
}

// port: leaf_change_sprite
pub fn leaf_change_sprite(face: f64) -> &'static str {
    if face != 0.0 {
        "leaf_change_2"
    } else {
        "leaf_change"
    }
}

// port: leaf_art_ready
/// The leaf frame data is embedded, so always ready (the original builds it on first draw).
pub fn leaf_art_ready() -> bool {
    true
}

/// The fields of a (backdrop) leaf `leaf_art_frame` reads; missing ones are `None`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeafArtPose {
    pub face: Option<f64>,
    pub change_from: Option<f64>,
    pub change_frame: f64,
}

impl LeafArtPose {
    pub fn of(l: &BackdropLeaf) -> LeafArtPose {
        LeafArtPose {
            face: Some(l.face),
            change_from: Some(l.change_from),
            change_frame: l.change_frame,
        }
    }
}

// port: leaf_art_frame
/// The leaf frame to draw: a turn frame while changing face, else the rest face.
pub fn leaf_art_frame(l: &LeafArtPose) -> LeafArtFrame {
    let a = &art().leaf_art;
    let cf = l.change_from.unwrap_or(-1.0);
    if cf >= 0.0 {
        return a.turns[cf.floor() as usize][l.change_frame.floor() as usize];
    }
    a.faces[l.face.unwrap_or(0.0).floor() as usize][0]
}

// port: leaf_start_turn
/// Leaves no longer turn in the air (the original returns false).
pub fn leaf_start_turn(_l: &mut Leaf) -> bool {
    false
}

// port: leaves_turn_all
pub fn leaves_turn_all(sim: &mut Sim) {
    for l in &mut sim.life.leaves {
        leaf_start_turn(l);
    }
}

// port: leaf_decay_time
/// Frames to absorb a leaf.
pub fn leaf_decay_time(sim: &Sim) -> f64 {
    let k = 1.0 + upg_bonus(&sim.state, "absorption_speed");
    30f64.max(sim.bal("leaf_absorb_time") / k)
}

// port: leaf_food_value
/// Food per absorbed leaf.
pub fn leaf_food_value(sim: &Sim) -> f64 {
    let b = upg_bonus(&sim.state, "absorption_yield");
    (sim.bal("leaf_food_reward") + b).floor()
}

fn pose(l: &Leaf) -> LeafFgPose {
    LeafFgPose {
        turn: Some(l.turn),
        turned: Some(l.turned),
        angle: Some(l.angle),
        mirror: Some(l.mirror),
    }
}

// port: leaf_spawn
/// A leaf at `x` (default: random in the near zone, or in view) and `y` (default -40).
pub fn leaf_spawn(sim: &mut Sim, in_view: bool, x: Option<f64>, y: Option<f64>) {
    let y = y.unwrap_or(-40.0);
    let zone = life_near_zone(sim);
    let v = squirrel_view(sim);
    let lx = match x {
        Some(x) => x,
        None => {
            if !in_view {
                sim.rng.random_range(zone.x1, zone.x2)
            } else {
                v.x + sim.rng.random(v.w)
            }
        }
    };
    let phase = sim.rng.random(360.0);
    let mirror = sim.rng.irandom(1.0) as f64;
    let turn_delay =
        sim.rng
            .irandom_range(vis("leaf_turn_delay_min"), vis("leaf_turn_delay_max")) as f64;
    let [wind_dir, wind_speed, _] = sim.state.state.wind;
    let drift = wind_dir * wind_speed * vis("leaf_wind_push");
    let id = sim.life.new_id(source::LEAF);
    sim.life.leaves.push(Leaf {
        id,
        x: lx,
        y,
        phase,
        mirror,
        turn: -1.0,
        turn_delay,
        drift,
        life: vis("leaf_idle_life"),
        ..Default::default()
    });
}

// port: leaf_gust_spawn
/// A gust: a bunch of leaves around one spot of the near zone, high above.
pub fn leaf_gust_spawn(sim: &mut Sim) {
    let zone = life_near_zone(sim);
    let cx = sim.rng.random_range(zone.x1, zone.x2);
    let n = sim
        .rng
        .irandom_range(vis("leaf_gust_min"), vis("leaf_gust_max"));
    let spread = vis("leaf_gust_spread_px");
    for _ in 0..n.max(0) {
        let x = cx + sim.rng.random_range(-spread, spread);
        let y = -40.0 - sim.rng.random(vis("leaf_gust_height_px"));
        leaf_spawn(sim, false, Some(x), Some(y));
    }
}

// port: leaves_step
/// Spawn on the timer; leaves drift and fall; a landing leaf over the mycelium (absorption)
/// starts being absorbed (tendril, hyphae sound); lying leaves fade; absorbed ones pay food
/// (`Resource{Food}` at the leaf) and leave their tendril retracting.
pub fn leaves_step(sim: &mut Sim) {
    sim.state.state.timers.leaf -= 1.0;
    if sim.state.state.timers.leaf <= 0.0 {
        let zone = life_near_zone(sim);
        sim.state.state.timers.leaf =
            sim.bal("leaf_spawn_interval") * zone.w / 1f64.max(zone.x2 - zone.x1);
        leaf_spawn(sim, false, None, None);
    }
    let surf = wld("surface_height_px");
    let leaves = std::mem::take(&mut sim.life.leaves);
    let mut kept = Vec::with_capacity(leaves.len());
    for mut leaf in leaves {
        if !leaf.landed {
            if leaf.turn >= 0.0 {
                leaf.turn_timer -= 1.0;
                if leaf.turn_timer <= 0.0 {
                    leaf.turn_timer = vis("leaf_turn_frame_interval");
                    leaf.turn += 1.0;
                    if leaf.turn >= 14.0 {
                        let (cx, cy) = leaf_fg_carry();
                        leaf.x += cx * if leaf.mirror > 0.0 { -1.0 } else { 1.0 };
                        leaf.y += cy;
                        leaf.turn = -1.0;
                        leaf.turned = true;
                    }
                }
            }
            let [wind_dir, wind_speed, _] = sim.state.state.wind;
            let push = wind_dir * wind_speed * vis("leaf_wind_push");
            leaf.drift += (push - leaf.drift) * gml::clamp(vis("leaf_wind_ease"), 0.0, 1.0);
            leaf.phase += vis("leaf_sway_speed");
            let spin = gml::sin(gml::degtorad(leaf.phase)) * vis("leaf_spin_degrees");
            let target = if leaf.turn >= 0.0 {
                leaf.hold_angle
            } else {
                spin
            };
            leaf.angle += (target - leaf.angle) * gml::clamp(vis("leaf_spin_ease"), 0.0, 1.0);
            leaf.x += gml::lengthdir_x(vis("leaf_sway"), leaf.phase) + leaf.drift;
            leaf.y += vis("leaf_fall_speed");
            let bottom = leaf.y + leaf_fg_drop(&pose(&leaf));
            if bottom >= surf {
                leaf.y -= bottom - surf;
                leaf.landed = true;
                if upg_is_owned(&sim.state, "unlock_absorption") {
                    let r = sim.bal("leaf_absorb_radius");
                    let hit = get_mycelium_hit(sim, leaf.x, surf + wld("reference_tile_size"), r);
                    if hit.hit {
                        leaf.absorbing = true;
                        let gf = 1f64.max(vis("leaf_tendril_grow_frames"));
                        leaf.timer = gf
                            + sim
                                .bal("leaf_min_hang_frames")
                                .max(leaf_decay_time(sim) - gf);
                        leaf.total = Some(leaf.timer);
                        leaf.grow_frames = Some(gf);
                        leaf.tendril = Some(leaf_tendril_make(sim, &leaf, hit.x, hit.y));
                        let d = gml::point_distance(hit.x, hit.y, leaf.x, surf);
                        let (ev, _) = crate::audio::hyphae_begin(
                            &mut sim.rng,
                            leaf.id,
                            d,
                            gf,
                            Some((leaf.x, surf)),
                        );
                        sim.emit(ev);
                        leaf.sound = true;
                    }
                }
            }
            kept.push(leaf);
        } else if !leaf.absorbing {
            leaf.life -= 1.0;
            if leaf.life > 0.0 {
                kept.push(leaf);
            }
        } else {
            leaf.timer -= 1.0;
            let total = leaf.total.unwrap_or(0.0);
            if leaf.sound {
                sim.emit(SimEvent::Audio(AudioCall::HyphaeStep {
                    id: leaf.id,
                    t: total - leaf.timer,
                }));
            }
            if leaf.timer <= 0.0 {
                if leaf.sound {
                    sim.emit(SimEvent::Audio(AudioCall::HyphaeStop { id: leaf.id }));
                }
                let food = leaf_food_value(sim);
                sim.food_add(food);
                sim.emit(SimEvent::Resource {
                    kind: ResourceKind::Food,
                    delta: food,
                    x: leaf.x,
                    y: leaf.y,
                    source: leaf.id,
                });
                if let Some(cells) = leaf.tendril.take() {
                    let rf = 1f64.max(vis("leaf_tendril_retract_frames"));
                    sim.life.leaf_tendrils_out.push(TendrilOut {
                        cells,
                        timer: rf,
                        total: rf,
                    });
                }
            } else {
                kept.push(leaf);
            }
        }
    }
    // leaves spawned during the loop (none today) would follow the kept ones
    kept.append(&mut sim.life.leaves);
    sim.life.leaves = kept;
    sim.life.leaf_tendrils_out.retain_mut(|t| {
        t.timer -= 1.0;
        t.timer > 0.0
    });
}

// port: leaf_tendril_make
/// The absorbing tendril from the mycelium hit (`hx`, `hy`) up to the landed leaf, wrapping
/// round it. Returns its cells.
pub fn leaf_tendril_make(sim: &mut Sim, leaf: &Leaf, hx: f64, hy: f64) -> Vec<Cell> {
    let surf = wld("surface_height_px");
    let p = pose(leaf);
    let mir = leaf_fg_mirror(&p);
    let bx = leaf_fg_frame(&p);
    let lx1 = (bx.x1 * mir).min(bx.x2 * mir);
    let lx2 = (bx.x1 * mir).max(bx.x2 * mir);
    let cx = leaf.x + (lx1 + lx2) * 0.5;
    let cy = leaf.y + (bx.y1 + bx.y2) * 0.5;
    let hw = (lx2 - lx1) * 0.5 + 1.0;
    let hh = (bx.y2 - bx.y1) * 0.5 + 1.0;
    let drop = leaf_fg_drop(&p);
    let bottom = (leaf.y + drop).floor();
    let mut path = vec![Pt { x: hx, y: hy }];
    let tx = cx;
    let ty = bottom.max(surf) + 2.0;
    let d = gml::point_distance(hx, hy, tx, ty);
    let dir = gml::point_direction(hx, hy, tx, ty);
    let n = 3f64.max(gml::round(d / 14.0));
    let mut wob = 0.0;
    let mut k = 1.0;
    while k <= n {
        let t = k / n;
        wob = wob * 0.55 + sim.rng.random_range(-1.0, 1.0) * d * 0.06;
        let off = wob * (1.0 - (t - 0.5).abs() * 2.0);
        path.push(Pt {
            x: gml::lerp(hx, tx, t) + gml::lengthdir_x(off, dir + 90.0),
            y: gml::lerp(hy, ty, t) + gml::lengthdir_y(off, dir + 90.0),
        });
        k += 1.0;
    }
    let turns = vis("leaf_tendril_wrap_turns");
    let m = 8f64.max(gml::round(20.0 * turns));
    let side = sim.rng.choose(&[-1.0, 1.0]);
    let mut j = 0.0;
    while j <= m {
        let t = j / m;
        let a = 270.0 + side * 360.0 * turns * t;
        let rr = gml::lerp(1.0, 0.45, t);
        path.push(Pt {
            x: cx + gml::lengthdir_x(hw * rr, a),
            y: cy + gml::lengthdir_y(hh * rr, a),
        });
        j += 1.0;
    }
    rasterize_branch(&path, vis("strand_thickness"))
}
