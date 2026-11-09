//! Repellers: a charged repeller fires a dust cloud at a hedgehog or boar walking into it; the
//! cloud's front turns the animal around. Cloud drawing (`repeller_clouds_draw` and the soft
//! sprites) is Task 15's; it reads [`MushroomsRt::repel_clouds`](super::MushroomsRt).

use super::MushroomKind;
use crate::audio;
use crate::balance::{vis, wld};
use crate::gml;
use crate::life;
use crate::progress::tasks;
use crate::sim::Sim;

/// A dust/cloud puff (`___struct___71` dust, `___struct___72` cloud).
#[derive(Clone, Debug, PartialEq)]
pub struct Puff {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub size: f64,
    pub grow: f64,
    pub life: f64,
    pub total: f64,
    pub colour: f64,
    pub dust: bool,
}

/// A speck (`___struct___73`).
#[derive(Clone, Debug, PartialEq)]
pub struct Speck {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub total: f64,
}

/// A repeller cloud (`___struct___70`, `global.repel_clouds`).
#[derive(Clone, Debug, PartialEq)]
pub struct RepelCloud {
    pub x: f64,
    pub y: f64,
    pub ground: f64,
    pub dir: f64,
    pub flash: f64,
    pub puffs: Vec<Puff>,
    pub specks: Vec<Speck>,
}

// port: repeller_cloud_fire
/// Repeller `idx` fires toward `dir`: spend a charge, flash, an explosion sound, a camera kick
/// when near the view, and a cloud of dust, puffs and specks.
pub fn repeller_cloud_fire(sim: &mut Sim, idx: usize, dir: f64) {
    tasks::stat_add(sim, "ach_repels", 1.0);
    let r = &mut sim.state.mushrooms[idx];
    r.active = false;
    r.flash = vis("repeller_flash_frames");
    r.charges -= 1.0;
    let (rx, ry) = (r.x, r.y);
    let mut c = RepelCloud {
        x: rx,
        y: ry - vis("repeller_cloud_dy"),
        ground: wld("surface_height_px"),
        dir,
        flash: vis("repeller_cloud_flash_frames"),
        puffs: Vec::new(),
        specks: Vec::new(),
    };
    sim.emit(audio::sfx_at(
        "explosion",
        c.x,
        c.y,
        vis("explosion_volume"),
        None,
    ));
    let v = life::squirrel_view(sim);
    let off = 0f64.max(v.x - c.x).max(c.x - (v.x + v.w));
    if off < v.w * 0.5 {
        let k = sim.mushrooms.repel_kick;
        sim.mushrooms.repel_kick = k.max(vis("repeller_cloud_kick_px") * (1.0 - off / (v.w * 0.5)));
    }
    let speed = vis("repeller_cloud_speed");
    let (life_min, life_max) = (
        vis("repeller_cloud_life_min"),
        vis("repeller_cloud_life_max"),
    );
    let (size_min, size_max) = (
        vis("repeller_cloud_size_min"),
        vis("repeller_cloud_size_max"),
    );
    let rng = &mut sim.rng;
    let mut i = 0.0;
    while i < vis("repeller_cloud_dust") {
        let d = if rng.random(1.0) < 0.7 { dir } else { -dir };
        let life = rng.irandom_range(life_min, life_max) as f64;
        let col_t = rng.random(1.0);
        let px = c.x + rng.random_range(-10.0, 10.0);
        let py = c.ground - rng.random_range(2.0, 10.0);
        let pvx = d * speed * rng.random_range(0.2, 0.75);
        let pvy = -rng.random_range(0.0, 1.2);
        let psize = rng.random_range(8.0, 16.0);
        let pgrow = rng.random_range(size_max * 0.5, size_max * 0.9);
        c.puffs.push(Puff {
            x: px,
            y: py,
            vx: pvx,
            vy: pvy,
            size: psize,
            grow: pgrow,
            life,
            total: life,
            colour: gml::merge_colour(14740206.0, 12767446.0, col_t),
            dust: true,
        });
        i += 1.0;
    }
    let spread = vis("repeller_cloud_spread_deg");
    let base = if dir <= 0.0 { 180.0 } else { 0.0 };
    let mut i = 0.0;
    while i < vis("repeller_cloud_puffs") {
        let narrow = if rng.random(1.0) < 0.8 { 0.5 } else { 1.0 };
        let a = base + rng.random_range(-spread, spread) * narrow + dir * 12.0;
        let spd = speed * (0.25 + 0.75 * gml::power(rng.random(1.0), 0.6));
        let life = rng.irandom_range(life_min, life_max) as f64;
        let col_t = rng.random(1.0);
        let px = c.x + rng.random_range(-6.0, 6.0);
        let py = c.y + rng.random_range(-6.0, 6.0);
        let pvx = gml::lengthdir_x(spd, a);
        let pvy = gml::lengthdir_y(spd, a);
        let psize = rng.random_range(size_min, size_min * 1.6);
        let pgrow = rng.random_range(size_max * 0.6, size_max);
        c.puffs.push(Puff {
            x: px,
            y: py,
            vx: pvx,
            vy: pvy,
            size: psize,
            grow: pgrow,
            life,
            total: life,
            colour: gml::merge_colour(16777215.0, 15133932.0, col_t),
            dust: false,
        });
        i += 1.0;
    }
    let mut i = 0.0;
    while i < vis("repeller_cloud_specks") {
        let a = base + rng.random_range(-spread, spread) + dir * 12.0;
        let spd = speed * rng.random_range(0.5, 1.4);
        let life = rng.irandom_range(life_min, life_max) as f64;
        c.specks.push(Speck {
            x: c.x,
            y: c.y,
            vx: gml::lengthdir_x(spd, a),
            vy: gml::lengthdir_y(spd, a),
            life,
            total: life,
        });
        i += 1.0;
    }
    sim.mushrooms.repel_clouds.push(c);
}

// port: repeller_animal_front
/// The front of an animal at `x` walking toward `dir` (`reach` ahead of its origin).
pub fn repeller_animal_front(x: f64, dir: f64, reach: f64) -> f64 {
    x + dir * reach
}

// port: repeller_clouds_step
/// Fire charged repellers at hedgehogs and boars walking into them, then move the clouds;
/// a cloud's front turns animals walking into it (boars flee).
pub fn repeller_clouds_step(sim: &mut Sim) {
    let radius = sim.bal("repeller_trigger_radius");
    let hreach = life::hedgehog_frame_w() * 0.4;
    let breach = life::boar_frame_w() * vis("boar_snout_dx");
    let mut i = 0;
    while i < sim.state.mushrooms.len() {
        let m = &sim.state.mushrooms[i];
        if m.kind == MushroomKind::Repeller.code() && m.active && m.carried_by == -1 && !m.stolen {
            let mx = m.x;
            let mut fired = false;
            let mut j = 0;
            while j < sim.life.animals.len() && !fired {
                let a = &sim.life.animals[j];
                let adir = a.dir;
                let gap = (mx - repeller_animal_front(a.x, adir, hreach)) * adir;
                if gap > 0.0 && gap <= radius {
                    repeller_cloud_fire(sim, i, -adir);
                    fired = true;
                }
                j += 1;
            }
            let mut j = 0;
            while j < sim.life.boars.len() && !fired {
                let b = &sim.life.boars[j];
                let bdir = b.dir;
                let gap = (mx - repeller_animal_front(b.x, bdir, breach)) * bdir;
                if !b.fleeing && gap > 0.0 && gap <= radius {
                    repeller_cloud_fire(sim, i, -bdir);
                    fired = true;
                }
                j += 1;
            }
        }
        i += 1;
    }
    let drag = vis("repeller_cloud_drag");
    let clouds = std::mem::take(&mut sim.mushrooms.repel_clouds);
    let mut keep = Vec::with_capacity(clouds.len());
    for mut c in clouds {
        let mut front: f64 = 0.0;
        let mut puffs = Vec::with_capacity(c.puffs.len());
        for mut p in std::mem::take(&mut c.puffs) {
            p.x += p.vx;
            p.y += p.vy;
            p.vx *= drag;
            p.vy = p.vy * drag - 0.03;
            p.size = gml::lerp(p.size, p.grow, 0.08);
            p.life -= 1.0;
            let floor_y = c.ground - p.size * 0.3;
            if p.y > floor_y {
                p.y = floor_y;
                let sx = if p.vx == 0.0 { c.dir } else { p.vx };
                p.vx += gml::sign(sx) * p.vy.abs() * 0.5;
                p.vy = p.vy.abs() * -0.15;
            }
            if p.life > 0.0 {
                if !p.dust && p.life > p.total * 0.35 {
                    front = front.max((p.x - c.x) * c.dir + p.size * 0.5);
                }
                puffs.push(p);
            }
        }
        c.puffs = puffs;
        let mut specks = Vec::with_capacity(c.specks.len());
        for mut s in std::mem::take(&mut c.specks) {
            s.x += s.vx;
            s.y += s.vy;
            s.vx *= 0.95;
            s.vy = s.vy * 0.95 + 0.04;
            s.life -= 1.0;
            if s.y > c.ground - 1.0 {
                s.y = c.ground - 1.0;
                s.vx *= 0.5;
                s.vy = 0.0;
            }
            if s.life > 0.0 {
                specks.push(s);
            }
        }
        c.specks = specks;
        c.flash = (c.flash - 1.0).max(0.0);
        if front > 0.0 {
            let x0 = c.x;
            let x1 = c.x + c.dir * front;
            let lo = x0.min(x1);
            let hi = x0.max(x1);
            let mut turned = 0;
            for a in &mut sim.life.animals {
                let f = repeller_animal_front(a.x, a.dir, hreach);
                if a.dir == -c.dir && f >= lo && f <= hi {
                    a.dir = -a.dir;
                    turned += 1;
                }
            }
            for _ in 0..turned {
                tasks::stat_add(sim, "hedgehogs", 1.0);
            }
            for b in &mut sim.life.boars {
                let f = repeller_animal_front(b.x, b.dir, breach);
                if !b.fleeing && b.dir == -c.dir && f >= lo && f <= hi {
                    b.dir = -b.dir;
                    b.fleeing = true;
                }
            }
        }
        if !c.puffs.is_empty() || !c.specks.is_empty() {
            keep.push(c);
        }
    }
    sim.mushrooms.repel_clouds = keep;
}
