//! Squirrels (scr_life): they come down a tree03 (or run over the ground from one), steal tree
//! mushrooms and consumptors off trunks, and climb away with them.

use super::art::art;
use super::{
    animal_interval, animal_timer_step, animals_rebind, mushroom_delete, mushroom_idx,
    squirrel_view, Squirrel, SquirrelState,
};
use crate::balance::{vis, wld};
use crate::forest::{HostId, TreeRt};
use crate::gml;
use crate::mushrooms::stinky::stinky_guards;
use crate::mushrooms::types::{mushroom_cell_width, sprite_of_code};
use crate::mycelium::source;
use crate::progress::tasks;
use crate::sim::{Sim, ViewRect};
use crate::state::MushroomSave;
use crate::world::perks::perk_has;

/// A big tree as the squirrels hold it (the original keeps the tree struct): its id, sprite and
/// place. Trees never move, so the snapshot stays true; equality in the original is identity,
/// here [`TreeRef::same`] (the uid).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeRef {
    pub uid: u64,
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
}

impl TreeRef {
    pub fn of(t: &TreeRt) -> TreeRef {
        TreeRef {
            uid: t.uid,
            sprite: t.sprite.clone(),
            x: t.x,
            y: t.y,
            flip: t.flip,
        }
    }

    /// The same tree (`t == other` in the original).
    pub fn same(&self, o: &TreeRef) -> bool {
        self.uid == o.uid
    }
}

/// What `squirrel_pick` / `squirrel_summon` hand to `squirrel_spawn` (`___struct___77/79`).
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub mushroom: Option<u64>,
    pub tree: TreeRef,
    pub home: TreeRef,
    pub stop_y: Option<f64>,
}

/// `squirrel_hold_pose(s)` (`___struct___80`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HoldPose {
    pub x: f64,
    pub y: f64,
    pub xscale: f64,
    pub yscale: f64,
    pub angle: f64,
}

// port: squirrel_frame_w
pub fn squirrel_frame_w() -> f64 {
    art().sprite("squirrel").w / 9.0
}

// port: squirrel_frame_h
pub fn squirrel_frame_h() -> f64 {
    art().sprite("squirrel").h
}

// port: squirrel_offset
/// A point of the squirrel sprite (`px`, `py`, frame px) relative to its body origin, mirrored
/// by `face` and rotated by `angle`.
pub fn squirrel_offset(face: f64, angle: f64, px: f64, py: f64) -> (f64, f64) {
    let s = vis("squirrel_scale");
    let dx = (px - vis("squirrel_body_x")) * face * s;
    let dy = (py - vis("squirrel_body_y")) * s;
    let c = gml::dcos(angle);
    let sn = gml::dsin(angle);
    (dx * c + dy * sn, -dx * sn + dy * c)
}

// port: squirrel_point
/// A sprite point of squirrel `sq` in world px.
pub fn squirrel_point(sq: &Squirrel, px: f64, py: f64) -> (f64, f64) {
    let o = squirrel_offset(sq.mirror, sq.angle, px, py);
    (sq.x + o.0, sq.y + o.1)
}

// port: squirrel_edge_x
/// The trunk edge of tree `s` on `side` (<= 0 left) at height `py` (the trunk edges are art
/// data, `squirrel_trunk_edges`).
pub fn squirrel_edge_x(s: &TreeRef, side: f64, py: f64) -> f64 {
    let c = art().trunk(&s.sprite);
    let row = gml::clamp(gml::round(py - s.y + c.origin_y), 0.0, c.h - 1.0) as usize;
    let mut l = c.left[row];
    let mut r = c.right[row];
    if s.flip < 0.0 {
        let t = l;
        l = -r;
        r = -t;
    }
    s.x + if side <= 0.0 { l } else { r }
}

// port: squirrel_climb_x
/// Where the squirrel's body goes so that its grip sits on the trunk edge.
pub fn squirrel_climb_x(flip: f64, angle: f64, s: &TreeRef, side: f64, py: f64) -> f64 {
    let inset = side * vis("squirrel_trunk_inset");
    let edge = squirrel_edge_x(s, side, py);
    let x = edge - inset;
    let off = squirrel_offset(flip, angle, vis("squirrel_grip_x"), vis("squirrel_body_y"));
    x - off.0
}

// port: squirrel_run_y
pub fn squirrel_run_y() -> f64 {
    wld("surface_height_px") + vis("squirrel_run_body_dy")
}

// port: squirrel_tail_drop
pub fn squirrel_tail_drop() -> f64 {
    (vis("squirrel_tail_tip_y") - vis("squirrel_body_y")) * vis("squirrel_scale")
}

/// The trees of surface chunk `c` (generated on first use, as `surface_get_chunk` does).
fn chunk_trees(sim: &mut Sim, c: i64) -> Vec<TreeRef> {
    sim.forest
        .surface_get_chunk(&mut sim.rng, c)
        .trees
        .iter()
        .map(TreeRef::of)
        .collect()
}

// port: squirrel_tree_near
/// The nearest big tree within `range` of `x` (`need_free`: a tree03 without a tree mushroom),
/// other than `exclude` (a tree uid).
pub fn squirrel_tree_near(
    sim: &mut Sim,
    x: f64,
    range: f64,
    need_free: bool,
    exclude: Option<u64>,
) -> Option<TreeRef> {
    let cw = sim.forest.chunk_width;
    let c0 = ((x - range) / cw).floor() as i64;
    let c1 = ((x + range) / cw).floor() as i64;
    let mut best = None;
    let mut best_d = range;
    for c in c0..=c1 {
        for t in chunk_trees(sim, c) {
            if need_free && t.sprite != "tree03" {
                continue;
            }
            if exclude == Some(t.uid) {
                continue;
            }
            if (t.x - x).abs() > best_d {
                continue;
            }
            if need_free && squirrel_tree_has_mushroom(sim, &t) {
                continue;
            }
            best_d = (t.x - x).abs();
            best = Some(t);
        }
    }
    best
}

// port: squirrel_tree_has_mushroom
/// A tree mushroom already grows on tree `t`.
pub fn squirrel_tree_has_mushroom(sim: &Sim, t: &TreeRef) -> bool {
    let off = vis("tree_mushroom_offset_x");
    sim.state
        .mushrooms
        .iter()
        .any(|m| m.kind == 3 && !m.stolen && (m.x - off - t.x).abs() < 2.0)
}

/// The tree a mushroom's `host` is, when it is a big tree.
fn host_tree<'a>(sim: &'a Sim, m: &MushroomSave) -> Option<&'a TreeRt> {
    match m.live.host? {
        HostId::Tree(uid) => sim.forest.tree(uid),
        HostId::Stump(_) => None,
    }
}

// port: squirrel_takes
/// Squirrels take tree mushrooms and consumptors on a tree that is not eaten yet (never with
/// the red perk).
pub fn squirrel_takes(sim: &Sim, mi: usize) -> bool {
    if perk_has(&sim.state, "red") {
        return false;
    }
    let m = &sim.state.mushrooms[mi];
    if m.kind == 3 {
        return true;
    }
    if m.kind == 7 && matches!(m.live.host, Some(HostId::Tree(_))) {
        // a host tree the forest no longer has was eaten (`consumed`)
        return host_tree(sim, m).is_some_and(|t| !t.consumed);
    }
    false
}

// port: squirrel_tree_for
/// The tree a mushroom grows on: a consumptor's host, else the tree under a tree mushroom.
pub fn squirrel_tree_for(sim: &mut Sim, mi: usize) -> Option<TreeRef> {
    let m = &sim.state.mushrooms[mi];
    if m.kind == 7 {
        return host_tree(sim, m).map(TreeRef::of);
    }
    let x = m.x - vis("tree_mushroom_offset_x");
    squirrel_tree_near(sim, x, 80.0, false, None)
}

// port: squirrel_bite_y
/// The height the squirrel's mouth reaches for mushroom `m`.
pub fn squirrel_bite_y(m: &MushroomSave) -> f64 {
    let by = m.y + vis("squirrel_bite_dy");
    if m.kind == 7 {
        return by + vis("squirrel_consumptor_bite_dy");
    }
    by
}

// port: squirrel_shroom_bite
/// Where on the mushroom sprite the squirrel bites (sprite px).
pub fn squirrel_shroom_bite(kind: i64) -> (f64, f64) {
    if kind == 7 {
        return (
            vis("squirrel_consumptor_bite_x"),
            vis("squirrel_consumptor_bite_y"),
        );
    }
    (vis("squirrel_shroom_bite_x"), vis("squirrel_shroom_bite_y"))
}

/// Some squirrel is already after mushroom `id`.
fn taken(sim: &Sim, id: u64) -> bool {
    sim.life.squirrels.iter().any(|s| s.mushroom == Some(id))
}

// port: squirrel_pick
/// A random target: a mushroom it takes (in view, if `in_view`), not taken by another squirrel,
/// with its tree and a tree03 home to come from.
pub fn squirrel_pick(sim: &mut Sim, v: &ViewRect, in_view: bool) -> Option<Pick> {
    let mut picks = Vec::new();
    let mut i = 0;
    while i < sim.state.mushrooms.len() {
        let m = &sim.state.mushrooms[i];
        let (mx, my, id) = (m.x, m.y, m.live.id);
        let skip = !squirrel_takes(sim, i)
            || m.stolen
            || m.crow_target
            || (in_view
                && !(mx >= v.x + 40.0 && mx <= v.x + v.w - 40.0 && my >= v.y && my <= v.y + v.h))
            || taken(sim, id);
        if !skip {
            if let Some(tree) = squirrel_tree_for(sim, i) {
                let home = if tree.sprite == "tree03" {
                    Some(tree.clone())
                } else {
                    squirrel_tree03_find(sim, tree.x, 0.0, 0.0, true, Some(tree.uid))
                };
                if let Some(home) = home {
                    picks.push(Pick {
                        mushroom: Some(id),
                        tree,
                        home,
                        stop_y: None,
                    });
                }
            }
        }
        i += 1;
    }
    if picks.is_empty() {
        return None;
    }
    let k = sim.rng.irandom((picks.len() - 1) as f64) as usize;
    Some(picks.swap_remove(k))
}

// port: squirrel_sky_y
/// Where a squirrel appears above tree `s`: its crown top, or the view top when that is higher
/// and the tree is near the view.
pub fn squirrel_sky_y(s: &TreeRef, v: &ViewRect) -> f64 {
    let top = s.y - art().sprite(&s.sprite).yoff;
    let margin = squirrel_frame_w() + 80.0;
    if s.x > v.x - margin && s.x < v.x + v.w + margin {
        return top.min(v.y);
    }
    top
}

// port: squirrel_any_target
pub fn squirrel_any_target(sim: &Sim) -> bool {
    (0..sim.state.mushrooms.len()).any(|i| {
        let m = &sim.state.mushrooms[i];
        squirrel_takes(sim, i) && !m.stolen && !m.crow_target && !taken(sim, m.live.id)
    })
}

// port: squirrel_spawn
/// A squirrel at the top of its home tree, heading down the side toward its target.
pub fn squirrel_spawn(sim: &mut Sim, p: Pick, v: &ViewRect) {
    let home = p.home.clone();
    let direct = p.tree.same(&home);
    let dirc = gml::sign(p.tree.x - home.x);
    let mut side = if direct {
        sim.rng.choose(&[-1.0, 1.0])
    } else {
        dirc
    };
    let mut stop_y = p.stop_y.unwrap_or(0.0);
    if direct {
        if let Some(mi) = p.mushroom.and_then(|id| mushroom_idx(sim, id)) {
            let m = &sim.state.mushrooms[mi];
            side = if m.x < home.x { -1.0 } else { 1.0 };
            stop_y = squirrel_bite_y(m);
        }
    }
    let y0 = squirrel_sky_y(&home, v)
        - vis("squirrel_scale") * (squirrel_frame_h() - vis("squirrel_body_y"))
        - 8.0;
    let id = sim.life.new_id(source::ANIMAL);
    let mut s = Squirrel {
        id,
        tree: home.clone(),
        home: home.clone(),
        target_tree: p.tree.clone(),
        far_tree: p.tree.clone(),
        direct,
        side,
        dir: dirc,
        mirror: -side,
        y: y0,
        stop_y,
        frame_timer: vis("squirrel_frame_interval"),
        mushroom: p.mushroom,
        empty_run: p.mushroom.is_none(),
        ..Default::default()
    };
    s.x = squirrel_climb_x(s.mirror, 0.0, &home, side, y0);
    sim.life.squirrels.push(s);
}

// port: squirrel_summon
/// Debug: a squirrel for a mushroom in view (else anywhere), else an empty run on a free tree03.
pub fn squirrel_summon(sim: &mut Sim) -> &'static str {
    let v = squirrel_view(sim);
    let mut p = squirrel_pick(sim, &v, true);
    if p.is_none() {
        p = squirrel_pick(sim, &v, false);
    }
    if let Some(p) = p {
        let direct = p.tree.same(&p.home);
        squirrel_spawn(sim, p, &v);
        if direct {
            return "squirrel: tree mushroom on its tree03";
        }
        return "squirrel: over the ground to another tree";
    }
    let Some(t) = squirrel_tree_near(sim, v.x + v.w * 0.5, v.w * 0.5, true, None) else {
        return "squirrel: no free tree03 on screen";
    };
    let stop_y = wld("surface_height_px") - vis("tree_mushroom_height_px");
    squirrel_spawn(
        sim,
        Pick {
            mushroom: None,
            tree: t.clone(),
            home: t,
            stop_y: Some(stop_y),
        },
        &v,
    );
    "squirrel: empty run"
}

// port: squirrel_hold_pose
/// Where a mushroom of type `kind` held in the squirrel's mouth goes, and its scale and angle.
pub fn squirrel_hold_pose(s: &Squirrel, kind: i64) -> HoldPose {
    let cs = vis("squirrel_carry_scale");
    let dir = -s.mirror;
    let xs = dir * cs;
    let ys = cs;
    let ang = gml::angle_difference(vis("squirrel_carry_angle") * dir + s.angle, 0.0);
    let spr = sprite_of_code(kind);
    let fwd = mushroom_cell_width(spr) * cs * vis("squirrel_carry_forward") / vis("squirrel_scale");
    let mouth = squirrel_point(
        s,
        vis("squirrel_mouth_x") + vis("squirrel_hold_into_head_x"),
        vis("squirrel_mouth_y") + vis("squirrel_hold_into_head_y") + fwd,
    );
    let bite = squirrel_shroom_bite(kind);
    let bx = xs * bite.0;
    let by = ys * bite.1;
    let c = gml::dcos(ang);
    let sn = gml::dsin(ang);
    HoldPose {
        x: mouth.0 - (bx * c + by * sn),
        y: mouth.1 - (-bx * sn + by * c),
        xscale: xs,
        yscale: ys,
        angle: ang,
    }
}

// port: squirrel_animate
pub fn squirrel_animate(s: &mut Squirrel) {
    s.frame_timer -= 1.0;
    if s.frame_timer <= 0.0 {
        s.frame_timer = vis("squirrel_frame_interval");
        s.frame = gml::gmod(s.frame + 1.0, 9.0);
    }
}

// port: squirrel_flip_up
pub fn squirrel_flip_up(s: &mut Squirrel) {
    s.state = SquirrelState::Up;
    s.mirror = s.side;
    s.angle = 180.0;
}

// port: squirrel_flip_down
pub fn squirrel_flip_down(s: &mut Squirrel) {
    s.state = SquirrelState::Down;
    s.mirror = -s.side;
    s.angle = 0.0;
}

// port: squirrel_tree03_find
/// The nearest tree03 from `x` (`dir` 0: either side, else ahead and at least `min_dist` away),
/// searching chunk rings outward; `need_free`: without a tree mushroom; never `exclude`.
pub fn squirrel_tree03_find(
    sim: &mut Sim,
    x: f64,
    dir: f64,
    min_dist: f64,
    need_free: bool,
    exclude: Option<u64>,
) -> Option<TreeRef> {
    let cw = sim.forest.chunk_width;
    let c0 = (x / cw).floor() as i64;
    let mut best = None;
    let mut best_d = f64::INFINITY;
    let mut ring = 0i64;
    while ring <= 256 {
        if (ring - 1) as f64 * cw > best_d {
            break;
        }
        let cs: Vec<i64> = if ring == 0 {
            vec![c0]
        } else if dir == 0.0 {
            vec![c0 - ring, c0 + ring]
        } else {
            vec![c0 + dir as i64 * ring]
        };
        for c in cs {
            for t in chunk_trees(sim, c) {
                let d = if dir == 0.0 {
                    (t.x - x).abs()
                } else {
                    (t.x - x) * dir
                };
                if t.sprite != "tree03" {
                    continue;
                }
                if exclude == Some(t.uid) || d >= best_d {
                    continue;
                }
                if dir != 0.0 && d < min_dist {
                    continue;
                }
                if need_free && squirrel_tree_has_mushroom(sim, &t) {
                    continue;
                }
                best_d = d;
                best = Some(t);
            }
        }
        ring += 1;
    }
    best
}

// port: squirrel_check_repeller
/// An active repeller just ahead of the running squirrel fires: it drops its target, turns
/// round and runs for the next tree03 behind it.
pub fn squirrel_check_repeller(sim: &mut Sim, s: &mut Squirrel) -> bool {
    let reach = sim.bal("squirrel_repeller_reach_px");
    for i in 0..sim.state.mushrooms.len() {
        let m = &sim.state.mushrooms[i];
        if !(m.kind == 2 && m.active && m.carried_by == -1) {
            continue;
        }
        let mouth = squirrel_point(s, vis("squirrel_mouth_x"), vis("squirrel_mouth_y"));
        let d = (m.x - mouth.0) * s.dir;
        if d >= 0.0 && !(d > reach) {
            crate::mushrooms::repeller::repeller_cloud_fire(sim, i, -s.dir);
            if let Some(id) = s.mushroom {
                // a mushroom gone from the list was stolen (crow or squirrel), as it is kept
                let stolen = mushroom_idx(sim, id).is_none_or(|k| sim.state.mushrooms[k].stolen);
                if !stolen {
                    s.mushroom = None;
                    s.empty_run = true;
                }
            }
            s.dir = -s.dir;
            s.mirror = -s.mirror;
            s.angle = -s.angle;
            let t =
                squirrel_tree03_find(sim, s.x, s.dir, vis("squirrel_jump_dx") + 40.0, false, None);
            s.target_tree = match t {
                Some(t) => t,
                None => {
                    if s.target_tree.same(&s.home) {
                        s.far_tree.clone()
                    } else {
                        s.home.clone()
                    }
                }
            };
            return true;
        }
    }
    false
}

// port: squirrel_turn_start
pub fn squirrel_turn_start(
    s: &mut Squirrel,
    state: SquirrelState,
    to_angle: f64,
    to_x: f64,
    to_y: f64,
) {
    s.state = state;
    s.turn_t = 0.0;
    s.turn_from_angle = s.angle;
    s.turn_to_angle = to_angle;
    s.turn_from_x = s.x;
    s.turn_from_y = s.y;
    s.turn_to_x = to_x;
    s.turn_to_y = to_y;
}

// port: squirrel_turn_step
/// Ease the jump/turn on; true when done.
pub fn squirrel_turn_step(s: &mut Squirrel) -> bool {
    let t = s.turn_t;
    let frames = 1f64.max(vis("squirrel_turn_frames"));
    s.turn_t = 1f64.min(t + 1.0 / frames);
    let u = 1.0 - s.turn_t;
    let e = 1.0 - u * u;
    s.angle = gml::lerp(s.turn_from_angle, s.turn_to_angle, e);
    s.x = gml::lerp(s.turn_from_x, s.turn_to_x, e);
    s.y = gml::lerp(s.turn_from_y, s.turn_to_y, e);
    s.turn_t >= 1.0
}

// port: squirrel_grab
/// The squirrel bites its mushroom: guarded by a stinky (or the red perk) it gives up, else the
/// mushroom is stolen and pulled into its mouth.
pub fn squirrel_grab(sim: &mut Sim, s: &mut Squirrel) {
    let Some(mi) = s.mushroom.and_then(|id| mushroom_idx(sim, id)) else {
        return;
    };
    let guarded = stinky_guards(sim, mi);
    if guarded || perk_has(&sim.state, "red") {
        if guarded {
            tasks::stat_add(sim, "ach_stinky_scares", 1.0);
        }
        s.mushroom = None;
        s.empty_run = true;
        return;
    }
    s.timer = vis("squirrel_pull_frames");
    let m = &mut sim.state.mushrooms[mi];
    s.grab_x = m.x;
    s.grab_y = m.y;
    m.stolen = true;
}

/// Mushroom `id` still exists and is stolen; its index.
fn stolen_idx(sim: &Sim, id: Option<u64>) -> Option<usize> {
    let i = mushroom_idx(sim, id?)?;
    sim.state.mushrooms[i].stolen.then_some(i)
}

// port: squirrels_step
/// Spawn on the timer (while there is a target); every squirrel climbs, jumps, runs, steals and
/// climbs away; one that left the top of its tree is gone, with what it carries.
pub fn squirrels_step(sim: &mut Sim) {
    let view = squirrel_view(sim);
    if squirrel_any_target(sim) {
        let step = animal_timer_step(sim);
        sim.state.state.timers.squirrel -= step;
        if sim.state.state.timers.squirrel <= 0.0 {
            sim.state.state.timers.squirrel = animal_interval(sim, "squirrel_spawn_interval");
            if let Some(pick) = squirrel_pick(sim, &view, false) {
                squirrel_spawn(sim, pick, &view);
            }
        }
    }
    let spd = sim.bal("squirrel_speed");
    let run_spd = sim.bal("squirrel_run_speed");
    let squirrels = std::mem::take(&mut sim.life.squirrels);
    // the others stay visible to the per-squirrel code as `global.squirrels` would be
    sim.life.squirrels = squirrels.clone();
    let mut keep = Vec::with_capacity(squirrels.len());
    for (si, mut s) in squirrels.into_iter().enumerate() {
        let mut m = s.mushroom;
        if !s.empty_run && m.and_then(|id| mushroom_idx(sim, id)).is_none() {
            s.mushroom = None;
            m = None;
            s.empty_run = true;
            if s.state == SquirrelState::Down {
                squirrel_flip_up(&mut s);
            }
        }
        match s.state {
            SquirrelState::Down => {
                squirrel_animate(&mut s);
                s.y += spd;
                s.x = squirrel_climb_x(s.mirror, s.angle, &s.tree, s.side, s.y);
                let mouth = squirrel_point(&s, vis("squirrel_mouth_x"), vis("squirrel_mouth_y"));
                if s.direct {
                    if let Some(mi) = m.and_then(|id| mushroom_idx(sim, id)) {
                        s.stop_y = squirrel_bite_y(&sim.state.mushrooms[mi]);
                    }
                    if mouth.1 >= s.stop_y {
                        if m.is_some() {
                            squirrel_grab(sim, &mut s);
                        }
                        squirrel_flip_up(&mut s);
                    }
                } else {
                    let turn = 90.0 * s.dir;
                    let gap_y = squirrel_run_y() - vis("squirrel_jump_gap");
                    if s.y >= gap_y {
                        s.y = gap_y;
                        let jx = s.x + s.dir * vis("squirrel_jump_dx");
                        squirrel_turn_start(
                            &mut s,
                            SquirrelState::Jump,
                            turn,
                            jx,
                            squirrel_run_y(),
                        );
                    }
                }
            }
            SquirrelState::Jump => {
                squirrel_animate(&mut s);
                if squirrel_turn_step(&mut s) {
                    s.state = SquirrelState::Run;
                    s.run_line = s.y;
                }
            }
            SquirrelState::Run => {
                squirrel_animate(&mut s);
                squirrel_check_repeller(sim, &mut s);
                m = s.mushroom;
                s.x += s.dir * run_spd;
                s.y = s.run_line;
                let turn = 180.0 * s.dir;
                let nside = -s.dir;
                let top_y = s.y - vis("squirrel_jump_gap");
                let tx = squirrel_climb_x(s.mirror, turn, &s.target_tree, nside, top_y);
                if (s.x - (tx - s.dir * vis("squirrel_jump_dx"))) * s.dir >= 0.0 {
                    s.tree = s.target_tree.clone();
                    s.side = nside;
                    squirrel_turn_start(&mut s, SquirrelState::Land, turn, tx, top_y);
                }
            }
            SquirrelState::Land => {
                squirrel_animate(&mut s);
                if squirrel_turn_step(&mut s) {
                    s.state = SquirrelState::Up;
                }
            }
            SquirrelState::Up => {
                squirrel_animate(&mut s);
                s.y -= spd;
                s.x = squirrel_climb_x(s.mirror, s.angle, &s.tree, s.side, s.y);
                if let Some(mi) = m.and_then(|id| mushroom_idx(sim, id)) {
                    if !sim.state.mushrooms[mi].stolen {
                        let mouth =
                            squirrel_point(&s, vis("squirrel_mouth_x"), vis("squirrel_mouth_y"));
                        let by = squirrel_bite_y(&sim.state.mushrooms[mi]);
                        if mouth.1 <= by {
                            squirrel_grab(sim, &mut s);
                            squirrel_flip_down(&mut s);
                            s.dir = -s.dir;
                            s.target_tree = s.home.clone();
                        }
                    }
                }
            }
        }
        s.timer = 0f64.max(s.timer - 1.0);
        if let Some(mi) = stolen_idx(sim, m) {
            let kind = sim.state.mushrooms[mi].kind;
            let pose = squirrel_hold_pose(&s, kind);
            let mut t = 1.0 - s.timer / 1f64.max(vis("squirrel_pull_frames"));
            t = 1.0 - (1.0 - t) * (1.0 - t);
            let mm = &mut sim.state.mushrooms[mi];
            mm.x = gml::lerp(s.grab_x, pose.x, t);
            mm.y = gml::lerp(s.grab_y, pose.y, t);
            mm.hold_xscale = gml::lerp(1.0, pose.xscale, t);
            mm.hold_yscale = gml::lerp(1.0, pose.yscale, t);
            mm.hold_angle = pose.angle * t;
        }
        let mut removed = false;
        if s.state == SquirrelState::Up {
            let drop = squirrel_tail_drop();
            if s.y + drop + 8.0 < squirrel_sky_y(&s.tree, &view) {
                removed = true;
                if let Some(mi) = stolen_idx(sim, m) {
                    mushroom_delete(sim, mi);
                    animals_rebind(sim);
                }
            }
        }
        sim.life.squirrels[si] = s.clone();
        if !removed {
            keep.push(s);
        }
    }
    sim.life.squirrels = keep;
}
