//! The migration cut-scene (`cine_*`, scr_game): the camera visits the spore mushrooms, the
//! origin fires a mega burst, then the planet screen (stars, the Earth, mycelium spreading over
//! it) holds on the "migration_end" popup and fades out when it closes.
//!
//! The state is `global.tasks.cine` (not saved: `load_game` re-runs `tasks_init`), kept in
//! [`TasksRt::cine`](crate::progress::tasks::TasksRt). Drawing (`cine_draw`, `cine_ball`) is the
//! app's (ruling R12c); every function here is logic and runs in obj_hud Begin Step.

use crate::balance::vis;
use crate::gml;
use crate::mycelium::knots::{self, Ray};
use crate::progress::{tasks, upgrades::upg_bonus};
use crate::rng::{array_shuffle, Rng};
use crate::sim::Sim;

/// `cine.phase`: "show" (camera tour), "play" (planet), "hold" (popup up), "out" (fade).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CinePhase {
    Show,
    Play,
    Hold,
    Out,
}

/// One camera stop (`___struct___42/43`, plus `rays` / `left` set when its burst fires).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CineStop {
    pub x: f64,
    pub y: f64,
    pub seconds: f64,
    /// mega-burst spores (only the origin's stop has any)
    pub count: f64,
    pub rays: Vec<Ray>,
    pub left: f64,
}

/// A mushroom firing show spores during the tour (`cine_spore_spots` + `next`).
#[derive(Clone, Debug, PartialEq)]
pub struct CineFire {
    pub x: f64,
    pub y: f64,
    pub next: f64,
}

/// A planet-screen star (`___struct___45`), x/y in 0..1 of the GUI.
#[derive(Clone, Debug, PartialEq)]
pub struct CineStar {
    pub x: f64,
    pub y: f64,
    pub s: f64,
    pub a: f64,
    pub ph: f64,
    pub sp: f64,
}

/// A mycelium segment on the planet (`___struct___48`), in ball radii; `time` 0..1.
#[derive(Clone, Debug, PartialEq)]
pub struct CineSeg {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub time: f64,
    pub w: f64,
}

/// `global.tasks.cine` (`___struct___44` plus the fields `cine_step` / `cine_planet_start` add).
#[derive(Clone, Debug, PartialEq)]
pub struct Cine {
    pub phase: CinePhase,
    pub t: f64,
    pub alpha: f64,
    /// current stop index, -1 before the first
    pub stop: i64,
    pub shot: bool,
    pub stops: Vec<CineStop>,
    pub clock: Option<f64>,
    pub firing: Option<Vec<CineFire>>,
    pub white: Option<f64>,
    pub shake_k: Option<f64>,
    pub stars: Vec<CineStar>,
    pub segs: Vec<CineSeg>,
}

// port: cine_active
/// A cut-scene exists (`tasks.cine` is a struct).
pub fn cine_active(sim: &Sim) -> bool {
    sim.progress.tasks.cine.is_some()
}

// port: cine_blocking
/// A cut-scene holds the game (any phase but the fade out).
pub fn cine_blocking(sim: &Sim) -> bool {
    sim.progress
        .tasks
        .cine
        .as_ref()
        .is_some_and(|c| c.phase != CinePhase::Out)
}

// port: cine_showing
/// The camera tour is running (camera input frozen, the shake on).
pub fn cine_showing(sim: &Sim) -> bool {
    sim.progress
        .tasks
        .cine
        .as_ref()
        .is_some_and(|c| c.phase == CinePhase::Show)
}

// port: cine_spore_spots
/// Standing spore mushrooms (type 0, not stolen, not carried), in `global.mushrooms` order.
pub fn cine_spore_spots(sim: &Sim) -> Vec<(f64, f64)> {
    sim.state
        .mushrooms
        .iter()
        .filter(|m| m.kind == 0 && !m.stolen && m.carried_by == -1)
        .map(|m| (m.x, m.y))
        .collect()
}

// port: cine_span
/// `clamp((t - a) / max(0.001, b - a), 0, 1)`.
pub fn cine_span(t: f64, a: f64, b: f64) -> f64 {
    gml::clamp((t - a) / (b - a).max(0.001), 0.0, 1.0)
}

// port: cine_show_start
/// Start the migration: stops at the spore mushroom nearest the view centre, then up to
/// `migration_spore_stops` others in random order (at least two stops, padded with points beside
/// the origin), then the origin with the mega burst.
pub fn cine_show_start(sim: &mut Sim) {
    let cam = &sim.camera;
    let cam_cx = cam.x + cam.width * 0.5;
    let cam_cy = cam.y + cam.height * 0.5;
    // obj_mycelium_controller always exists in the sim (the `start_x` fallback never applies)
    let (ox, oy) = (sim.mycelium.x, sim.mycelium.y);
    let spots = cine_spore_spots(sim);
    let mut nearest = None;
    let mut best = f64::INFINITY;
    for (k, s) in spots.iter().enumerate() {
        let d = gml::point_distance(cam_cx, cam_cy, s.0, s.1);
        if d < best {
            best = d;
            nearest = Some(k);
        }
    }
    let mut rest: Vec<(f64, f64)> = spots
        .iter()
        .enumerate()
        .filter(|(k, _)| Some(*k) != nearest)
        .map(|(_, s)| *s)
        .collect();
    array_shuffle(&mut sim.rng, &mut rest);
    let mut stops: Vec<(f64, f64)> = nearest.map(|k| spots[k]).into_iter().collect();
    let max_stops = vis("migration_spore_stops");
    for s in rest {
        if (stops.len() as f64) >= max_stops {
            break;
        }
        stops.push(s);
    }
    while stops.len() <= 1 {
        let side = if stops.is_empty() { -1.0 } else { 1.0 };
        let n = stops.len() as f64;
        stops.push((ox + side * (n * 300.0 + 1400.0), oy));
    }
    let count = gml::round(
        sim.bal("spore_base_count")
            * (1.0 + sim.bal("upg_spore_count_per_level") * sim.bal("upg_spore_count_max_level"))
            * vis("migration_origin_spore_mult"),
    );
    let mut list: Vec<CineStop> = stops
        .iter()
        .map(|&(x, y)| CineStop {
            x,
            y,
            seconds: vis("migration_stop_seconds"),
            ..Default::default()
        })
        .collect();
    list.push(CineStop {
        x: ox,
        y: oy,
        seconds: vis("migration_origin_seconds"),
        count,
        ..Default::default()
    });
    sim.progress.tasks.cine = Some(Cine {
        phase: CinePhase::Show,
        t: 0.0,
        alpha: 1.0,
        stop: -1,
        shot: false,
        stops: list,
        clock: None,
        firing: None,
        white: None,
        shake_k: None,
        stars: Vec::new(),
        segs: Vec::new(),
    });
}

/// A growing tip of the planet mycelium (`___struct___47/49`); `seed` indexes its hub.
struct Front {
    x: f64,
    y: f64,
    a: f64,
    w: f64,
    seed: usize,
}

/// A planet hub (`___struct___46`).
struct Hub {
    x: f64,
    y: f64,
    delay: f64,
    pace: f64,
}

// port: cine_planet_start
/// The planet screen: 260 stars and the mycelium over the Earth, grown as random walks from 7
/// hubs inside the unit disc (at most 9000 segments, branching while fewer than 240 tips), each
/// segment timed by its distance from the hub (normalised to 0..1).
pub fn cine_planet_start(c: &mut Cine, rng: &mut Rng) {
    let mut stars = Vec::with_capacity(260);
    for _ in 0..260 {
        let x = rng.random(1.0);
        let y = rng.random(1.0);
        let s = rng.choose(&[1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 4.0]);
        let a = rng.random_range(0.7, 1.0);
        let ph = rng.random(360.0);
        let sp = rng.random_range(1.0, 4.0);
        stars.push(CineStar { x, y, s, a, ph, sp });
    }
    let mut segs: Vec<CineSeg> = Vec::new();
    let mut front: Vec<Front> = Vec::new();
    let mut hubs: Vec<Hub> = Vec::new();
    let n = 7;
    let mut tmax: f64 = 0.01;
    for i in 0..n {
        let ang = rng.random(360.0);
        let r = if i == 0 {
            rng.random(0.2)
        } else {
            rng.random_range(0.25, 0.8)
        };
        let hx = gml::lengthdir_x(r, ang);
        let hy = gml::lengthdir_y(r, ang);
        let delay = if i == 0 {
            0.0
        } else {
            rng.random_range(0.05, 0.45)
        };
        let pace = rng.random_range(0.6, 1.4);
        hubs.push(Hub {
            x: hx,
            y: hy,
            delay,
            pace,
        });
        let k = 10;
        for j in 0..k {
            let a = j as f64 * 360.0 / k as f64 + rng.random_range(-12.0, 12.0);
            front.push(Front {
                x: hx,
                y: hy,
                a,
                w: 1.0,
                seed: i,
            });
        }
    }
    let spd = 0.03;
    while !front.is_empty() && segs.len() < 9000 {
        let mut nxt: Vec<Front> = Vec::new();
        let flen = front.len();
        for mut f in std::mem::take(&mut front) {
            let s = &hubs[f.seed];
            let target = if f.x == s.x && f.y == s.y {
                f.a
            } else {
                gml::point_direction(s.x, s.y, f.x, f.y)
            };
            let jitter = rng.random_range(-32.0, 32.0);
            f.a += jitter + gml::angle_difference(target, f.a) * 0.2;
            let nx = f.x + gml::lengthdir_x(spd, f.a);
            let ny = f.y + gml::lengthdir_y(spd, f.a);
            if gml::point_distance(0.0, 0.0, nx, ny) <= 0.97 {
                let t = s.delay + gml::point_distance(s.x, s.y, f.x, f.y) / s.pace;
                segs.push(CineSeg {
                    x1: f.x,
                    y1: f.y,
                    x2: nx,
                    y2: ny,
                    time: t,
                    w: f.w,
                });
                tmax = tmax.max(t);
                f.x = nx;
                f.y = ny;
                let (fx, fy, fa, fw, seed) = (f.x, f.y, f.a, f.w, f.seed);
                nxt.push(f);
                if nxt.len() + flen < 240 && rng.random(1.0) < 0.12 && fw > 0.2 {
                    let rr = rng.random_range(25.0, 55.0);
                    let sign = rng.choose(&[-1.0, 1.0]);
                    nxt.push(Front {
                        x: fx,
                        y: fy,
                        a: fa + sign * rr,
                        w: fw * 0.82,
                        seed,
                    });
                }
            }
        }
        front = nxt;
    }
    for s in &mut segs {
        s.time /= tmax;
    }
    c.phase = CinePhase::Play;
    c.t = 0.0;
    c.white = Some(1.0);
    c.shake_k = Some(1.0);
    c.stars = stars;
    c.segs = segs;
}

// port: cine_shake
/// The camera shake of the tour (`None` outside it): `boar_shake_px` × `migration_shake_mult` ×
/// `shake_k`, two sines per axis on `current_time` (the sim clock).
pub fn cine_shake(sim: &Sim) -> Option<[f64; 2]> {
    if !cine_showing(sim) {
        return None;
    }
    let c = sim.progress.tasks.cine.as_ref()?;
    let k = c.shake_k.unwrap_or(1.0);
    let mult = vis("migration_shake_mult");
    let amp = vis("boar_shake_px") * mult * k;
    let t = sim.now_ms();
    let ax = amp * 0.45;
    let dx = ax * ((t * 0.057 + 1.3).sin() * 0.4 + (t * 0.031).sin() * 0.6);
    let dy = amp * ((t * 0.089 + 2.1).sin() * 0.4 + (t * 0.043 + 0.7).sin() * 0.6);
    Some([dx, dy])
}

// port: cine_step
/// One frame of the cut-scene (obj_hud Begin Step, before its popup/pause gates): the tour moves
/// the camera stop to stop, the spore mushrooms near the stop burst every `migration_burst_seconds`,
/// the origin fires its mega burst along random rays while the shake and the white flash build;
/// then the planet plays, holds on the "migration_end" popup and fades out once it closes.
pub fn cine_step(sim: &mut Sim) {
    let Some(mut c) = sim.progress.tasks.cine.take() else {
        return;
    };
    let keep = cine_step_inner(sim, &mut c);
    if keep {
        sim.progress.tasks.cine = Some(c);
    }
}

/// [`cine_step`] on the taken-out state; false when the cut-scene ends.
fn cine_step_inner(sim: &mut Sim, c: &mut Cine) -> bool {
    let dt = 1.0 / sim.fps.max(1.0);
    c.t += dt;
    if c.phase != CinePhase::Show {
        if c.phase == CinePhase::Play && c.t >= vis("migration_cine_seconds") {
            c.phase = CinePhase::Hold;
            sim.state.state.tasks.popup = "migration_end".into();
            tasks::stat_add(sim, "ach_migrated", 1.0);
        }
        if c.phase == CinePhase::Out {
            c.alpha -= dt / vis("migration_cine_fade_out_seconds").max(0.05);
            if c.alpha <= 0.0 {
                return false;
            }
        }
        return true;
    }
    if c.stop < 0 || c.t >= c.stops[c.stop as usize].seconds {
        c.stop += 1;
        c.t = 0.0;
        c.shot = false;
        if c.stop as usize >= c.stops.len() {
            cine_planet_start(c, &mut sim.rng);
            return true;
        }
    }
    let si = c.stop as usize;
    let (sx, sy) = (c.stops[si].x, c.stops[si].y);
    let cam = &mut sim.camera;
    cam.x = sx - cam.width * 0.5;
    cam.y = sy - cam.height * vis("migration_camera_ground_share");
    cam.vx = 0.0;
    cam.vy = 0.0;
    let delay = vis("migration_shot_delay_seconds");
    c.clock = Some(c.clock.unwrap_or(0.0) + dt);
    let clock = c.clock.unwrap_or(0.0);
    if c.firing.is_none() {
        let mut firing: Vec<CineFire> = cine_spore_spots(sim)
            .into_iter()
            .map(|(x, y)| CineFire { x, y, next: 0.0 })
            .collect();
        if firing.is_empty() {
            for s in &c.stops {
                if s.count <= 0.0 {
                    firing.push(CineFire {
                        x: s.x,
                        y: s.y,
                        next: 0.0,
                    });
                }
            }
        }
        for f in &mut firing {
            f.next = sim.rng.random(vis("migration_burst_seconds"));
        }
        c.firing = Some(firing);
    }
    let view_w = sim.camera.width;
    let mut firing = c.firing.take().unwrap_or_default();
    for f in &mut firing {
        if clock >= f.next {
            f.next += vis("migration_burst_seconds");
            if (f.x - sx).abs() <= view_w {
                let n = gml::round(
                    sim.bal("spore_base_count") * (1.0 + upg_bonus(&sim.state, "spore_count")),
                );
                knots::spawn_spores_show(sim, f.x, f.y - 52.0, n, 1.0);
            }
        }
    }
    c.firing = Some(firing);
    let s = &mut c.stops[si];
    if s.count <= 0.0 {
        return true;
    }
    if !c.shot && c.t >= delay {
        c.shot = true;
        s.rays = Vec::new();
        s.left = s.count;
        let rays = vis("migration_mega_rays");
        let mut j = 0.0;
        while j < rays {
            let a = -12.0 + (j + sim.rng.random_range(0.15, 0.85)) * 204.0 / rays;
            let power = sim.rng.random_range(0.65, 1.0);
            s.rays.push(Ray { a, power });
            j += 1.0;
        }
    }
    if c.shot && s.left > 0.0 {
        let emit = vis("migration_mega_emit_seconds");
        let n = s.left.min((s.count * dt / emit.max(0.05)).ceil());
        s.left -= n;
        let rays = s.rays.clone();
        knots::spawn_spores_rays(sim, sx, sy - 22.0, n, &rays, vis("migration_mega_speed"));
    }
    let seconds = c.stops[si].seconds;
    let since = c.t - delay;
    c.shake_k = Some(
        1.0 + (vis("migration_mega_shake_max") - 1.0) * cine_span(since, 0.0, seconds - delay),
    );
    c.white = Some(
        cine_span(since, vis("migration_white_from"), seconds - delay).powf(1.6),
    );
    true
}

/// The cut-scene part of `tasks_popup_close`: a cut-scene holding on the popup fades out.
pub fn cine_release_hold(sim: &mut Sim) {
    if let Some(c) = sim.progress.tasks.cine.as_mut() {
        if c.phase == CinePhase::Hold {
            c.phase = CinePhase::Out;
            c.t = 0.0;
        }
    }
}
