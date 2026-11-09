//! The upgrade-tree screen (`global.menu_open` in the original): scr_skilltree `tree_*` (hypha
//! links, strands, icons, glow, level rings, padlock, leaf wrap) plus the tree part of obj_hud
//! (`hud_tree_*` view math, the tree branch of Step_1 and of Draw_75).
//!
//! * Logic of the view (fit, pick, zoom, drag) is [`TreeView`], a pure struct tested against the
//!   LÖVE golden. A click on a node becomes `Command::BuyUpgrade` (ruling R9); the purchase and
//!   its rejection happen in the simulation (`progress::upgrades`), and the hover tooltip shows
//!   the reason a node cannot be bought (requires / no points / max / demo / coming later).
//! * [`TreeScreen`] is the Bevy resource: `open` is `global.menu_open`.
//!   **How the HUD (Task 17) opens and closes it:** call [`TreeScreen::open`] (the upgrade
//!   button, `hud_upgrade_btn_hover` + click in Step_1) and [`TreeScreen::close`]; read `open`
//!   for `ui_blocking`. The screen itself closes on its close button and on Escape.
//!   `logo_requested` is set by the logo button (`logo_open`; Task 18 consumes and clears it).
//!   `MYCO_TREE_OPEN=1` opens it at start (screenshots, autopilot).
//! * Every built hypha's length goes to `ProgressRt::tree_hypha_len` (ruling R16), which the grow
//!   animation (`upgrades_grow_step`) reads like `tree_hypha_length`.

use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::sim_bridge::{PendingInput, SimRes, UiBlockers, UiCapture};
use crate::ui::gm_draw::{Gm, GuiDraw, FA_CENTER, FA_LEFT, FA_MIDDLE, FA_RIGHT, FA_TOP};
use crate::ui::LocaleRes;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use myco_core::balance::{bal, vis};
use myco_core::gml::{self, lengthdir_x, lengthdir_y, point_direction, point_distance};
use myco_core::mushrooms::types::{sheet_cell_width, shroom_sheet_cells};
use myco_core::progress::skilltree::{
    layout, upg_children, upg_def, upg_tree_pos, upg_tree_via, TreeNode, UpgKind,
};
use myco_core::progress::upgrades::{
    upg_cost, upg_grow, upg_is_owned, upg_level, upg_max_level, upg_requirement_met, upg_state,
    UpgState,
};
use myco_core::sim::{Command, Key, Sim};
use myco_core::state::GameMode;
use std::collections::BTreeMap;

// obj_hud colours (Create_0)
pub const COLOR_INK: f64 = 14873582.0;
pub const COLOR_DIM: f64 = 8689804.0;
pub const COLOR_DEEP: f64 = 4609098.0;
pub const COLOR_GOOD: f64 = 9098922.0;
pub const COLOR_BAD: f64 = 6846166.0;
pub const COLOR_GOLD: f64 = 7914718.0;
pub const COLOR_PANEL: f64 = 1185298.0;

/// Draw-list key of the tree screen in [`GuiDraw`] (above the HUD).
pub const TREE_DRAW_ORDER: i32 = 500;

/// `gm.string(v)`: integers without decimals, otherwise two decimals.
fn gml_string(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

// ---- scr_skilltree: pure geometry -----------------------------------------------------------

// port: tree_seed
/// A per-id seed: a 31-polynomial hash of the bytes mod 99991, times 0.013.
pub fn tree_seed(s: &str) -> f64 {
    let mut h = 0.0;
    for ch in s.chars() {
        h = gml::gmod(h * 31.0 + ch as u32 as f64, 99991.0);
    }
    h * 0.013
}

// port: tree_strand_point
/// Point `t` on the strand from (x0, y0) to (x1, y1): a cubic bezier with a sideways wave.
#[allow(clippy::too_many_arguments)]
pub fn tree_strand_point(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    seed: f64,
    amp: f64,
    t: f64,
) -> (f64, f64) {
    let span = y1 - y0;
    let cy1 = y0 + span * 0.58;
    let cy2 = y1 - span * 0.48;
    let u = 1.0 - t;
    let mut px = u * u * u * x0 + 3.0 * u * u * t * x0;
    px += 3.0 * u * t * t * x1;
    px += t * t * t * x1;
    let mut py = u * u * u * y0 + 3.0 * u * u * t * cy1;
    py += 3.0 * u * t * t * cy2;
    py += t * t * t * y1;
    let a1 = t * std::f64::consts::PI * 2.3 + seed * 0.7;
    let a2 = t * std::f64::consts::PI * 5.7 + seed * 1.9;
    let wave = a2.sin() * 0.35 + a1.sin() * 0.65;
    let env = (t * std::f64::consts::PI).sin();
    (px + wave * env * amp, py)
}

// port: tree_noise
/// Hash noise in [-1, 1).
pub fn tree_noise(a: f64, b: f64) -> f64 {
    let n = (a * 12.9898 + b * 78.233).sin() * 43758.5453;
    (n - n.floor()) * 2.0 - 1.0
}

// port: tree_strand_path
/// Up to 31 jittered points `(x, y, t)` along the strand, stopping at `tmax`.
#[allow(clippy::too_many_arguments)]
pub fn tree_strand_path(
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    seed: f64,
    amp: f64,
    wob: f64,
    tmax: f64,
) -> Vec<(f64, f64, f64)> {
    let steps = 30;
    let mut pts = Vec::new();
    let mut drift = 0.0;
    for i in 0..=steps {
        let t = (i as f64 / steps as f64).min(tmax);
        let p = tree_strand_point(x0, y0, x1, y1, seed, amp, t);
        let q = tree_strand_point(x0, y0, x1, y1, seed, amp, (t + 0.02).min(1.0));
        let dir = point_direction(p.0, p.1, q.0, q.1) + 90.0;
        let nz = tree_noise(seed, i as f64);
        drift = drift * 0.6 + nz * 0.4;
        let wmax = (wob * 1.4).max(2.0);
        let env = (t * std::f64::consts::PI).sin();
        let off = drift * wmax * env;
        pts.push((p.0 + lengthdir_x(off, dir), p.1 + lengthdir_y(off, dir), t));
        if t >= tmax {
            break;
        }
    }
    pts
}

// port: tree_bend_point
/// Point `t` on the quadratic bezier (x0, y0) → control (cx, cy) → (x1, y1).
#[allow(clippy::too_many_arguments)]
pub fn tree_bend_point(x0: f64, y0: f64, cx: f64, cy: f64, x1: f64, y1: f64, t: f64) -> (f64, f64) {
    let u = 1.0 - t;
    let mut px = u * u * x0 + 2.0 * u * t * cx;
    px += t * t * x1;
    let mut py = u * u * y0 + 2.0 * u * t * cy;
    py += t * t * y1;
    (px, py)
}

// port: tree_hypha_walk
/// A wandering polyline: towards (tx, ty) (bent through `ctrl` when given), or `len` along `dir`.
#[allow(clippy::too_many_arguments)]
pub fn tree_hypha_walk(
    x0: f64,
    y0: f64,
    mut dir: f64,
    mut len: f64,
    steps: f64,
    amp: f64,
    seed: f64,
    target: Option<(f64, f64)>,
    ctrl: Option<(f64, f64)>,
) -> Vec<(f64, f64)> {
    let mut pts = vec![(x0, y0)];
    let mut drift = 0.0;
    if let Some((tx, ty)) = target {
        dir = point_direction(x0, y0, tx, ty);
        len = point_distance(x0, y0, tx, ty);
    }
    let mut i = 1.0;
    while i <= steps {
        let t = i / steps;
        let nz = tree_noise(seed, i * 1.37);
        drift = drift * 0.55 + nz * amp * 0.45;
        let env = if target.is_some() {
            (t * std::f64::consts::PI).sin()
        } else {
            1.0 - (t - 0.5).abs()
        };
        let off = drift * env;
        match (target, ctrl) {
            (Some((tx, ty)), Some((cx, cy))) => {
                let p = tree_bend_point(x0, y0, cx, cy, tx, ty, t);
                let pn = tree_bend_point(x0, y0, cx, cy, tx, ty, (t + 0.01).min(1.0));
                let pp = tree_bend_point(x0, y0, cx, cy, tx, ty, (t - 0.01).max(0.0));
                let tang = point_direction(pp.0, pp.1, pn.0, pn.1);
                pts.push((
                    p.0 + lengthdir_x(off, tang + 90.0),
                    p.1 + lengthdir_y(off, tang + 90.0),
                ));
            }
            _ => {
                let dist_t = len * t;
                let bx = x0 + lengthdir_x(dist_t, dir);
                let by = y0 + lengthdir_y(dist_t, dir);
                pts.push((
                    bx + lengthdir_x(off, dir + 90.0),
                    by + lengthdir_y(off, dir + 90.0),
                ));
            }
        }
        i += 1.0;
    }
    pts
}

/// One square cell of a tree hypha: integer centre, width, and position along it (0..1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HyphaCell {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub d: f64,
}

// port: tree_hypha_cells
/// Pixel cells along a polyline, tapering from `w0` to 1.
pub fn tree_hypha_cells(pts: &[(f64, f64)], w0: f64) -> Vec<HyphaCell> {
    let mut cells = Vec::new();
    let total: f64 = pts
        .windows(2)
        .map(|p| point_distance(p[0].0, p[0].1, p[1].0, p[1].1))
        .sum();
    if total <= 0.0 {
        return cells;
    }
    let mut acc = 0.0;
    let (mut lx, mut ly) = (-999999.0, -999999.0);
    for p in pts.windows(2) {
        let (a, b) = (p[0], p[1]);
        let seg = point_distance(a.0, a.1, b.0, b.1);
        if seg > 0.0 {
            let mut d = 0.0;
            while d < seg {
                let t = d / seg;
                let w = gml::round(gml::lerp(w0, 1.0, (acc + d) / total)).max(1.0);
                let cx = gml::lerp(a.0, b.0, t).floor();
                let cy = gml::lerp(a.1, b.1, t).floor();
                if !(cx == lx && cy == ly) {
                    cells.push(HyphaCell {
                        x: cx,
                        y: cy,
                        w,
                        d: (acc + d) / total,
                    });
                    lx = cx;
                    ly = cy;
                }
                d += (w - 1.0).max(1.0);
            }
            acc += seg;
        }
    }
    cells
}

#[derive(Clone, Debug, PartialEq)]
pub struct HyphaBranch {
    pub cells: Vec<HyphaCell>,
    /// where along the main hypha (0..1) this branch starts growing
    pub start: f64,
    pub span: f64,
}

/// A built tree hypha (`tree_hypha_build` result, world/tree coordinates).
#[derive(Clone, Debug, PartialEq)]
pub struct Hypha {
    pub branches: Vec<HyphaBranch>,
    pub total: f64,
    pub main_len: f64,
}

// port: tree_hypha_build
/// The main hypha from (x0, y0) to (x1, y1) plus `growth_side_branches` side branches.
#[allow(clippy::too_many_arguments)]
pub fn tree_hypha_build(
    mode: GameMode,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    seed: f64,
    ctrl: Option<(f64, f64)>,
) -> Hypha {
    let dist = point_distance(x0, y0, x1, y1);
    let hw = vis("tree_hypha_width");
    let wander = (dist * vis("growth_jitter") * 0.1).min(vis("tree_hypha_wander_max"));
    let steps = 14.0;
    let main = tree_hypha_walk(x0, y0, 0.0, 0.0, steps, wander, seed, Some((x1, y1)), ctrl);
    let mut branches = vec![HyphaBranch {
        cells: tree_hypha_cells(&main, hw),
        start: 0.0,
        span: 1.0,
    }];
    let nside = bal(mode, "growth_side_branches");
    let mut cum = vec![0.0];
    let mut main_len = 0.0;
    for p in main.windows(2) {
        main_len += point_distance(p[0].0, p[0].1, p[1].0, p[1].1);
        cum.push(main_len);
    }
    let mut total: f64 = 1.0;
    let mut b = 0.0;
    while b < nside {
        let n_idx = tree_noise(seed, 20.0 + b);
        let idx = 2 + (n_idx.abs() * (steps - 3.0)).floor() as usize;
        let (p, q) = (main[idx], main[idx + 1]);
        let side = if tree_noise(seed, 40.0 + b) > 0.0 {
            1.0
        } else {
            -1.0
        };
        let n_ang = tree_noise(seed, 60.0 + b);
        let bdir = point_direction(p.0, p.1, q.0, q.1) + side * (n_ang.abs() * 30.0 + 25.0);
        let n_len = tree_noise(seed, 80.0 + b);
        let blen = (dist * (n_len.abs() * 0.27 + 0.17)).min(vis("tree_hypha_side_max"));
        let bseed = seed + (b + 1.0) * 11.3;
        let bpts = tree_hypha_walk(p.0, p.1, bdir, blen, 6.0, wander * 0.69, bseed, None, None);
        let start = cum[idx] / main_len.max(1.0);
        let span = blen / main_len.max(1.0);
        branches.push(HyphaBranch {
            cells: tree_hypha_cells(&bpts, hw * 0.66),
            start,
            span,
        });
        total = total.max(start + span);
        b += 1.0;
    }
    Hypha {
        branches,
        total,
        main_len,
    }
}

/// `global.tree_hyphae`: built hyphae per node id, rebuilt when the endpoints change.
#[derive(Clone, Debug, Default)]
pub struct HyphaCache {
    pub mode: GameMode,
    entries: BTreeMap<String, (String, Hypha)>,
}

impl HyphaCache {
    // port: tree_hypha_get
    /// The hypha of node `id` between the given tree points, built (and cached) on demand.
    pub fn get(
        &mut self,
        id: &str,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        ctrl: Option<(f64, f64)>,
    ) -> &Hypha {
        let ctrl_key = ctrl
            .map(|(cx, cy)| format!(":{}:{}", gml_string(cx), gml_string(cy)))
            .unwrap_or_default();
        let key = format!(
            "{}:{}:{}:{}:{}{}:v5",
            gml_string(x0),
            gml_string(y0),
            gml_string(x1),
            gml_string(y1),
            gml_string(vis("tree_hypha_width")),
            ctrl_key
        );
        let fresh = !matches!(self.entries.get(id), Some((k, _)) if *k == key);
        if fresh {
            let h = tree_hypha_build(self.mode, x0, y0, x1, y1, tree_seed(id), ctrl);
            self.entries.insert(id.to_string(), (key, h));
        }
        &self.entries[id].1
    }

    // port: tree_hypha_length
    /// `main_len * total` of the built hypha, else 1.3 × the parent distance (1 without parent).
    pub fn length(&self, id: &str) -> f64 {
        match self.entries.get(id) {
            Some((_, h)) => h.main_len * h.total,
            None => myco_core::progress::skilltree::tree_hypha_length_fallback(id),
        }
    }

    /// Lengths of every built hypha (for `ProgressRt::tree_hypha_len`).
    pub fn lengths(&self) -> impl Iterator<Item = (&str, f64)> {
        self.entries
            .iter()
            .map(|(id, (_, h))| (id.as_str(), h.main_len * h.total))
    }
}

// ---- scr_skilltree: drawing -------------------------------------------------------------------

// port: tree_draw_hair
/// A short tapering hair of 4 segments, optionally with one sub-hair.
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_hair(
    gm: &mut Gm,
    x: f64,
    y: f64,
    mut dir: f64,
    len: f64,
    w: f64,
    seed: f64,
    branch: bool,
) {
    let segs = 4.0;
    let (mut cx, mut cy) = (x, y);
    let mut i = 1.0;
    while i <= segs {
        dir += tree_noise(seed, i * 3.1) * 28.0;
        let nx = cx + lengthdir_x(len / segs, dir);
        let ny = cy + lengthdir_y(len / segs, dir);
        gm.line_width(cx, cy, nx, ny, (w * (1.0 - i / (segs + 1.0))).max(1.0));
        if branch && i == 2.0 {
            let side = if tree_noise(seed, 9.0) > 0.0 {
                38.0
            } else {
                -38.0
            };
            tree_draw_hair(
                gm,
                nx,
                ny,
                dir + side,
                len * 0.5,
                w * 0.6,
                seed + 5.3,
                false,
            );
        }
        cx = nx;
        cy = ny;
        i += 1.0;
    }
}

// port: tree_draw_strand
/// A strand drawn up to `tmax`, with up to 7 hairs of length `hairs`.
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_strand(
    gm: &mut Gm,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    seed: f64,
    amp: f64,
    w: f64,
    col: f64,
    alpha: f64,
    tmax: f64,
    hairs: f64,
) {
    if tmax <= 0.0 {
        return;
    }
    let pts = tree_strand_path(x0, y0, x1, y1, seed, amp, w, tmax);
    let n = pts.len();
    gm.set_colour(col);
    gm.set_alpha(alpha);
    for i in 1..n {
        let (a, b) = (pts[i - 1], pts[i]);
        let ww = (w * gml::lerp(1.0, 0.4, b.2)).max(1.0);
        gm.line_width(a.0, a.1, b.0, b.1, ww);
        if ww > 3.0 {
            gm.circle(b.0, b.1, ww * 0.5, false);
        }
    }
    if hairs > 0.0 {
        let count = 7.0;
        let mut j = 0.0;
        while j < count {
            let ht = 0.12 + j * (0.78 / (count - 1.0)) + tree_noise(seed, j + 40.0) * 0.04;
            let idx = gml::clamp(gml::round(ht * 30.0), 0.0, 29.0) as usize;
            if idx + 1 >= n {
                break;
            }
            let (a, b) = (pts[idx], pts[idx + 1]);
            let side = if tree_noise(seed, j + 60.0) > 0.0 {
                1.0
            } else {
                -1.0
            };
            let hdir = point_direction(a.0, a.1, b.0, b.1)
                + side * (55.0 + tree_noise(seed, j + 80.0) * 20.0);
            let hlen = hairs * ((tree_noise(seed, j + 100.0).abs() + 1.0) * 0.5);
            let branch = tree_noise(seed, j + 120.0) > 0.2;
            tree_draw_hair(gm, a.0, a.1, hdir, hlen, w * 0.35, seed + j * 1.7, branch);
            j += 1.0;
        }
    }
    gm.set_alpha(1.0);
}

// port: tree_draw_glow
/// Three stacked translucent discs.
pub fn tree_draw_glow(gm: &mut Gm, x: f64, y: f64, r: f64, col: f64, alpha: f64) {
    gm.set_colour(col);
    for k in (1..=3).rev() {
        let k = k as f64;
        gm.set_alpha(alpha * (0.12 * k));
        gm.circle(x, y, r * (1.0 + k * 0.22), false);
    }
    gm.set_alpha(1.0);
}

// port: tree_draw_level_ring
/// A 260° ring of `maxlvl` dashes, the first `lvl` lit.
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_level_ring(
    gm: &mut Gm,
    x: f64,
    y: f64,
    r: f64,
    lvl: f64,
    maxlvl: f64,
    col_on: f64,
    col_off: f64,
    w: f64,
) {
    if maxlvl <= 1.0 {
        return;
    }
    let arc = 260.0;
    let start = 90.0 + arc * 0.5;
    let step = arc / maxlvl;
    let mut i = 0.0;
    while i < maxlvl {
        let a0 = start - i * step - step * 0.12;
        let a1 = start - (i + 1.0) * step + step * 0.12;
        let off = i >= lvl;
        gm.set_colour(if off { col_off } else { col_on });
        gm.set_alpha(if off { 0.35 } else { 1.0 });
        gm.line_width(
            x + lengthdir_x(r, a0),
            y + lengthdir_y(r, a0),
            x + lengthdir_x(r, a1),
            y + lengthdir_y(r, a1),
            w.max(1.0),
        );
        i += 1.0;
    }
    gm.set_alpha(1.0);
}

// port: tree_icon_box
/// Hand-picked crop box `[x1, y1, x2, y2]` of a mushroom sprite's first cell.
pub fn tree_icon_box(spr: &str) -> Option<[f64; 4]> {
    Some(match spr {
        "sporeshroom" => [4.0, 15.0, 41.0, 63.0],
        "predatorshroom" => [19.0, 8.0, 43.0, 59.0],
        "explosiveshroom" => [10.0, 18.0, 38.0, 47.0],
        "diggershroom" => [2.0, 11.0, 60.0, 63.0],
        "stinkyshroom" => [10.0, 26.0, 37.0, 56.0],
        "experienceshroom" => [4.0, 25.0, 40.0, 63.0],
        "consumptorshroom" => [1.0, 3.0, 46.0, 43.0],
        _ => return None,
    })
}

// port: tree_icon_scale
pub fn tree_icon_scale(spr: &str) -> f64 {
    if spr == "diggershroom" {
        vis("tree_digger_icon_scale")
    } else {
        1.0
    }
}

// port: tree_draw_icon
/// A sprite centred at (x, y) fitted into `size`; `cropped` uses [`tree_icon_box`].
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_icon(
    gm: &mut Gm,
    spr: Option<&str>,
    x: f64,
    y: f64,
    size: f64,
    alpha: f64,
    cropped: bool,
    fit: bool,
) {
    let Some(spr) = spr else { return };
    let Some(def) = gm.assets.try_sprite(spr) else {
        return;
    };
    let (sw, sh) = (def.size.x as f64, def.size.y as f64);
    let (ox, oy) = (def.origin.x as f64, def.origin.y as f64);
    let bx = if cropped { tree_icon_box(spr) } else { None };
    let Some(b) = bx else {
        let cells = shroom_sheet_cells(spr);
        let w = (sw / cells).trunc();
        let h = sh;
        let sc = size / w.max(h).max(1.0);
        if cells <= 1.0 {
            let dx = x + (ox - w * 0.5) * sc;
            let dy = y + (oy - h * 0.5) * sc;
            gm.sprite_ext(spr, 0, dx, dy, sc, sc, 16777215.0, alpha);
            return;
        }
        gm.sprite_part_ext(
            spr,
            0,
            0.0,
            0.0,
            w,
            h,
            x - w * 0.5 * sc,
            y - h * 0.5 * sc,
            sc,
            sc,
            16777215.0,
            alpha,
        );
        return;
    };
    let bw = b[2] - b[0] + 1.0;
    let bh = b[3] - b[1] + 1.0;
    let sc = if fit {
        size * tree_icon_scale(spr) / bw.max(bh)
    } else {
        let cell_w = sheet_cell_width(spr, sw);
        size / cell_w.max(sh).max(1.0)
    };
    gm.sprite_part_ext(
        spr,
        0,
        b[0],
        b[1],
        bw,
        bh,
        x - bw * 0.5 * sc,
        y - bh * 0.5 * sc,
        sc,
        sc,
        16777215.0,
        alpha,
    );
}

// port: tree_draw_guide
/// The faint guide line of a not-yet-grown link (strand curve or bend through `ctrl`).
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_guide(
    gm: &mut Gm,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    w: f64,
    col: f64,
    alpha: f64,
    ctrl: Option<(f64, f64)>,
) {
    if alpha <= 0.0 {
        return;
    }
    let segs = 24.0;
    let mut prev = tree_strand_point(x0, y0, x1, y1, 0.0, 0.0, 0.0);
    gm.set_colour(col);
    gm.set_alpha(alpha);
    let mut i = 1.0;
    while i <= segs {
        let p = match ctrl {
            None => tree_strand_point(x0, y0, x1, y1, 0.0, 0.0, i / segs),
            Some((cx, cy)) => tree_bend_point(x0, y0, cx, cy, x1, y1, i / segs),
        };
        gm.line_width(prev.0, prev.1, p.0, p.1, w);
        prev = p;
        i += 1.0;
    }
    gm.set_alpha(1.0);
}

// port: tree_hypha_cell_quad
/// One cell as two triangles.
pub fn tree_hypha_cell_quad(gm: &mut Gm, c: &HyphaCell, col: f64) {
    let h = (c.w / 2.0).trunc();
    let (x0, y0) = (c.x - h, c.y - h);
    let (x1, y1) = (x0 + c.w, y0 + c.w);
    gm.tri([(x0, y0, col, 1.0), (x1, y0, col, 1.0), (x0, y1, col, 1.0)]);
    gm.tri([(x1, y0, col, 1.0), (x1, y1, col, 1.0), (x0, y1, col, 1.0)]);
}

// port: tree_hypha_draw
/// Draw a hypha translated to (x, y) and scaled, grown to `prog` (0..1).
#[allow(clippy::too_many_arguments)]
pub fn tree_hypha_draw(gm: &mut Gm, h: &Hypha, x: f64, y: f64, scale: f64, col: f64, prog: f64) {
    gm.set_matrix(Some((x, y, scale)));
    if prog >= 1.0 {
        // the frozen vertex buffer: every cell of every branch
        for br in &h.branches {
            for c in &br.cells {
                tree_hypha_cell_quad(gm, c, col);
            }
        }
    } else {
        let lim = prog * h.total;
        for br in &h.branches {
            let frac = gml::clamp((lim - br.start) / br.span.max(0.001), 0.0, 1.0);
            if frac > 0.0 {
                for c in br.cells.iter().take_while(|c| c.d <= frac) {
                    tree_hypha_cell_quad(gm, c, col);
                }
            }
        }
    }
    gm.set_matrix(None);
}

// port: tree_draw_leaf_wrap
/// Two spiral strokes around the absorption node.
#[allow(clippy::too_many_arguments)]
pub fn tree_draw_leaf_wrap(gm: &mut Gm, x: f64, y: f64, r: f64, col: f64, alpha: f64, w: f64) {
    gm.set_colour(col);
    gm.set_alpha(alpha);
    for k in 0..2 {
        let segs = 28.0;
        let turns = 1.35;
        let dirn = if k == 0 { 1.0 } else { -1.0 };
        let a0 = if k == 0 { 250.0 } else { 70.0 };
        let mut prev: Option<(f64, f64)> = None;
        let mut i = 0.0;
        while i <= segs {
            let t = i / segs;
            let ang = a0 + dirn * 360.0 * turns * t;
            let rr = gml::lerp(0.95, 0.4, t);
            let p = (
                x + lengthdir_x(r * 0.62 * rr, ang),
                y + lengthdir_y(r * 0.42 * rr, ang),
            );
            if let Some(q) = prev {
                let lw = (2.4 * w * (1.0 - t * 0.6)).max(1.0);
                gm.line_width(q.0, q.1, p.0, p.1, lw);
            }
            prev = Some(p);
            i += 1.0;
        }
    }
    gm.set_alpha(1.0);
}

// port: tree_draw_padlock
/// A padlock of size `s` (demo-locked nodes).
pub fn tree_draw_padlock(gm: &mut Gm, x: f64, y: f64, s: f64, col: f64, col_hole: f64) {
    let bw = s * 0.9;
    let bh = s * 0.68;
    let by = y - bh * 0.2;
    let sr = bw * 0.3;
    let lw = (s * 0.13).max(2.0);
    let n = 12.0;
    gm.set_colour(col);
    let mut i = 0.0;
    while i < n {
        let a0 = 180.0 * i / n;
        let a1 = 180.0 * (i + 1.0) / n;
        gm.line_width(
            x + lengthdir_x(sr, a0),
            by - bh * 0.28 + lengthdir_y(sr, a0),
            x + lengthdir_x(sr, a1),
            by - bh * 0.28 + lengthdir_y(sr, a1),
            lw,
        );
        i += 1.0;
    }
    gm.line_width(x - sr, by - bh * 0.28, x - sr, by, lw);
    gm.line_width(x + sr, by - bh * 0.28, x + sr, by, lw);
    gm.rectangle(x - bw * 0.5, by, x + bw * 0.5, by + bh, false);
    gm.set_colour(col_hole);
    gm.circle(x, by + bh * 0.4, (s * 0.09).max(1.0), false);
    let k = (s * 0.035).max(1.0);
    gm.rectangle(x - k, by + bh * 0.4, x + k, by + bh * 0.72, false);
}

// ---- obj_hud: tree view ----------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect4 {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Rect4 {
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }
}

/// The input the tree branch of obj_hud Step_1 reads (GUI coordinates).
#[derive(Clone, Copy, Debug, Default)]
pub struct TreeInput {
    pub mx: f64,
    pub my: f64,
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
    /// `mouse_wheel_up() - mouse_wheel_down()`
    pub wheel: f64,
    /// `keyboard_check_pressed(ord("R"))`
    pub recentre_key: bool,
}

/// What a tree step asks the app to do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeOutcome {
    /// the close button: `global.menu_open = false`
    pub close: bool,
    /// the logo button: `logo_open()`
    pub logo: bool,
    /// a click on a node: `upg_buy(id)` → `Command::BuyUpgrade`
    pub buy: Option<String>,
}

/// obj_hud's tree-view fields (`hud_tree_*`, `hud_hover_id`) and the GUI size.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeView {
    pub zoom: f64,
    pub cx: f64,
    pub cy: f64,
    pub fitted: bool,
    pub fit_zoom: f64,
    pub zoom_lo: f64,
    pub fit_w: f64,
    pub fit_h: f64,
    pub bounds: Rect4,
    pub drag: bool,
    pub dragged: bool,
    pub drag_x: f64,
    pub drag_y: f64,
    pub press_id: String,
    pub hover_id: String,
    pub header_h: f64,
    pub footer_h: f64,
    /// `display_get_gui_width/height()`
    pub gui_w: f64,
    pub gui_h: f64,
}

impl Default for TreeView {
    /// obj_hud Create_0 values.
    fn default() -> Self {
        TreeView {
            zoom: 1.0,
            cx: 0.0,
            cy: 0.0,
            fitted: false,
            fit_zoom: 1.0,
            zoom_lo: 0.25,
            fit_w: 0.0,
            fit_h: 0.0,
            bounds: Rect4::default(),
            drag: false,
            dragged: false,
            drag_x: 0.0,
            drag_y: 0.0,
            press_id: String::new(),
            hover_id: String::new(),
            header_h: 104.0,
            footer_h: 72.0,
            gui_w: 1920.0,
            gui_h: 1080.0,
        }
    }
}

impl TreeView {
    pub fn set_gui_size(&mut self, w: f64, h: f64) {
        self.gui_w = w;
        self.gui_h = h;
    }

    // port: hud_tree_canvas
    /// The tree area between the header and the footer.
    pub fn canvas(&self) -> Rect4 {
        Rect4 {
            x1: 0.0,
            y1: self.header_h,
            x2: self.gui_w,
            y2: self.gui_h - self.footer_h,
        }
    }

    // port: hud_tree_world
    pub fn world(&self, d: &TreeNode) -> (f64, f64) {
        upg_tree_pos(d)
    }

    // port: hud_tree_screen
    /// Node position in GUI coordinates.
    pub fn screen(&self, d: &TreeNode) -> (f64, f64) {
        let c = self.canvas();
        let w = self.world(d);
        (
            (c.x1 + c.x2) * 0.5 + (w.0 - self.cx) * self.zoom,
            (c.y1 + c.y2) * 0.5 + (w.1 - self.cy) * self.zoom,
        )
    }

    /// A tree point (e.g. a link's bend) in GUI coordinates.
    fn to_screen(&self, x: f64, y: f64) -> (f64, f64) {
        let c = self.canvas();
        (
            (c.x1 + c.x2) * 0.5 + (x - self.cx) * self.zoom,
            (c.y1 + c.y2) * 0.5 + (y - self.cy) * self.zoom,
        )
    }

    // port: hud_tree_radius
    /// Node radius in tree units.
    pub fn radius(&self, d: &TreeNode) -> f64 {
        let r = vis("tree_node_radius");
        match d.kind {
            UpgKind::Root => r * vis("tree_root_radius_mult"),
            UpgKind::Unlock => r * vis("tree_unlock_radius_mult"),
            _ => r,
        }
    }

    // port: hud_tree_fit
    /// Fit the whole tree into the canvas (also the "recentre" button / R key).
    pub fn fit(&mut self) {
        let c = self.canvas();
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for (i, d) in layout().iter().enumerate() {
            let w = self.world(d);
            let r = self.radius(d);
            if i == 0 {
                min_x = w.0 - r;
                max_x = w.0 + r;
                min_y = w.1 - r;
                max_y = w.1 + r;
            } else {
                min_x = min_x.min(w.0 - r);
                max_x = max_x.max(w.0 + r);
                min_y = min_y.min(w.1 - r);
                max_y = max_y.max(w.1 + r);
            }
        }
        min_y -= 80.0;
        max_y += 90.0;
        self.bounds = Rect4 {
            x1: min_x,
            y1: min_y,
            x2: max_x,
            y2: max_y,
        };
        let margin = 90.0;
        let zx = (c.x2 - c.x1 - margin * 2.0) / (max_x - min_x).max(1.0);
        let zy = (c.y2 - c.y1 - margin * 2.0) / (max_y - min_y).max(1.0);
        self.fit_zoom = zx.min(zy).min(vis("tree_zoom_max"));
        self.zoom_lo = vis("tree_zoom_min").min(self.fit_zoom * vis("tree_zoom_out_past_fit"));
        self.zoom = self.fit_zoom;
        self.cx = (min_x + max_x) * 0.5;
        self.cy = (min_y + max_y) * 0.5;
        self.fit_w = self.gui_w;
        self.fit_h = self.gui_h;
        self.fitted = true;
    }

    // port: hud_tree_text_scale
    pub fn text_scale(&self, size: f64, min_size: f64) -> f64 {
        min_size.max(size * self.zoom)
    }

    // port: hud_tree_clamp_view
    pub fn clamp_view(&mut self) {
        let pad = 260.0;
        self.cx = gml::clamp(self.cx, self.bounds.x1 - pad, self.bounds.x2 + pad);
        self.cy = gml::clamp(self.cy, self.bounds.y1 - pad, self.bounds.y2 + pad);
    }

    // port: hud_tree_pick
    /// The node under (mx, my) (last defined wins), `""` for none.
    pub fn pick(&self, mx: f64, my: f64) -> String {
        if !self.canvas().contains(mx, my) {
            return String::new();
        }
        for d in layout().iter().rev() {
            let p = self.screen(d);
            let r = self.radius(d) * self.zoom;
            if point_distance(mx, my, p.0, p.1) <= r {
                return d.id.to_string();
            }
        }
        String::new()
    }

    /// `hud_close_button_rect()` ([`crate::ui::menu::hud_close_button_rect`]).
    pub fn close_button_rect(&self) -> Rect4 {
        let r = crate::ui::menu::hud_close_button_rect(self.gui_w);
        Rect4 {
            x1: r.x1,
            y1: r.y1,
            x2: r.x2,
            y2: r.y2,
        }
    }

    // port: hud_recentre_button_rect
    pub fn recentre_button_rect(&self) -> Rect4 {
        let c = self.close_button_rect();
        Rect4 {
            x1: c.x1 - 208.0,
            y1: c.y1,
            x2: c.x1 - 20.0,
            y2: c.y2,
        }
    }

    // port: hud_logo_button_rect
    pub fn logo_button_rect(&self) -> Rect4 {
        let c = self.recentre_button_rect();
        Rect4 {
            x1: c.x1 - 580.0,
            y1: c.y1,
            x2: c.x1 - 20.0,
            y2: c.y2,
        }
    }

    /// Step_1 while the tree is closed: `hud_tree_fitted = false`, `hud_tree_drag = false`.
    pub fn closed_step(&mut self) {
        self.fitted = false;
        self.drag = false;
    }

    /// The tree branch of obj_hud Step_1 (`global.menu_open`).
    pub fn step(&mut self, i: &TreeInput) -> TreeOutcome {
        let mut out = TreeOutcome::default();
        let (mx, my) = (i.mx, i.my);
        self.hover_id.clear();
        if !self.fitted || self.fit_w != self.gui_w || self.fit_h != self.gui_h {
            self.fit();
        }
        if i.pressed && self.close_button_rect().contains(mx, my) {
            out.close = true;
            return out;
        }
        if i.pressed && self.logo_button_rect().contains(mx, my) {
            out.logo = true;
            return out;
        }
        if i.recentre_key || (i.pressed && self.recentre_button_rect().contains(mx, my)) {
            self.fit();
            return out;
        }
        self.hover_id = self.pick(mx, my);
        let c = self.canvas();
        if i.wheel != 0.0 && c.contains(mx, my) {
            let ccx = (c.x1 + c.x2) * 0.5;
            let ccy = (c.y1 + c.y2) * 0.5;
            let wx = self.cx + (mx - ccx) / self.zoom;
            let wy = self.cy + (my - ccy) / self.zoom;
            self.zoom = gml::clamp(
                self.zoom * (1.0 + i.wheel * vis("tree_zoom_step")),
                self.zoom_lo,
                vis("tree_zoom_max"),
            );
            self.cx = wx - (mx - ccx) / self.zoom;
            self.cy = wy - (my - ccy) / self.zoom;
            self.clamp_view();
        }
        if i.pressed && c.contains(mx, my) {
            self.drag = true;
            self.dragged = false;
            self.drag_x = mx;
            self.drag_y = my;
            self.press_id = self.hover_id.clone();
        }
        if self.drag && i.down {
            let dx = mx - self.drag_x;
            let dy = my - self.drag_y;
            if dy.abs() + dx.abs() > 6.0 {
                self.dragged = true;
            }
            if self.dragged {
                self.cx -= dx / self.zoom;
                self.cy -= dy / self.zoom;
                self.clamp_view();
            }
            self.drag_x = mx;
            self.drag_y = my;
        }
        if i.released {
            if self.drag
                && !self.dragged
                && !self.press_id.is_empty()
                && self.press_id == self.hover_id
            {
                out.buy = Some(self.press_id.clone());
            }
            self.drag = false;
            self.press_id.clear();
        }
        out
    }

    // port: hud_tree_label_back
    /// `hud_tree_label_back`: a rounded translucent plate behind a label.
    #[allow(clippy::too_many_arguments)]
    fn label_back(&self, gm: &mut Gm, cx: f64, y: f64, w: f64, h: f64, scale: f64, alpha: f64) {
        if alpha <= 0.0 {
            return;
        }
        let padx = 8.0 * scale + 2.0;
        let pady = 3.0 * scale + 1.0;
        gm.set_colour(COLOR_PANEL);
        gm.set_alpha(vis("tree_label_back_alpha") * alpha.clamp(0.0, 1.0));
        gm.roundrect_fill(
            cx - w * 0.5 - padx,
            y - pady,
            cx + w * 0.5 + padx,
            y + h + pady,
            12.0 * scale,
            12.0 * scale,
        );
        gm.set_alpha(1.0);
    }
}

/// `hud_fit` ([`crate::ui::draw::hud_fit_size`]) in the painter's current font.
pub fn hud_fit(gm: &Gm, s: &str, sc: f64, maxw: f64, maxh: f64) -> f64 {
    crate::ui::draw::hud_fit_size(gm.string_width(s), gm.string_height(s), sc, maxw, maxh)
}

/// `hud_text_box` ([`crate::ui::draw::hud_text_box_place`]) on the painter.
fn hud_text_box(gm: &mut Gm, r: Rect4, s: &str, sc: f64, pad: f64) {
    let (ha, va) = (gm.halign, gm.valign);
    let (x, y, k) = crate::ui::draw::hud_text_box_place(
        gm.string_width(s),
        gm.string_height(s),
        (r.x1, r.y1, r.x2, r.y2),
        sc,
        pad,
    );
    gm.halign = FA_LEFT;
    gm.valign = FA_MIDDLE;
    gm.text_transformed(x, y, s, k, k);
    gm.halign = ha;
    gm.valign = va;
}

/// Everything the tree screen draw needs besides the view.
pub struct TreeDrawCtx<'a> {
    pub sim: &'a Sim,
    pub loc: &'a dyn Fn(&str) -> String,
    /// `current_time`, ms
    pub time_ms: f64,
    /// mouse in GUI coordinates (`device_mouse_x/y_to_gui`)
    pub mouse: (f64, f64),
}

/// The tree branch of obj_hud Draw_75 (`global.menu_open`).
pub fn tree_draw(view: &TreeView, cache: &mut HyphaCache, gm: &mut Gm, ctx: &TreeDrawCtx) {
    let st = &ctx.sim.state;
    let loc = ctx.loc;
    let (gw, gh) = (view.gui_w, view.gui_h);
    let c = view.canvas();
    let zoom = view.zoom;
    let pulse = ((ctx.time_ms / 340.0).sin() + 1.0) * 0.5;
    gm.halign = FA_LEFT;
    gm.valign = FA_TOP;
    gm.set_colour(0.0);
    gm.set_alpha(0.86);
    gm.rectangle(0.0, 0.0, gw, gh, false);
    gm.set_colour(COLOR_PANEL);
    gm.set_alpha(0.6);
    gm.rectangle(c.x1, c.y1, c.x2, c.y2, false);
    gm.set_alpha(1.0);
    // links
    for d in layout() {
        if d.parent.is_empty() {
            continue;
        }
        let Some(p) = upg_def(d.parent) else { continue };
        let met = upg_requirement_met(st, d.id);
        let grow = upg_grow(ctx.sim, d.id);
        let a = view.screen(p);
        let bb = view.screen(d);
        let via = upg_tree_via(d);
        let vs = via.map(|(x, y)| view.to_screen(x, y));
        let galpha = if met { 0.5 } else { 0.22 };
        tree_draw_guide(
            gm,
            a.0,
            a.1,
            bb.0,
            bb.1,
            (2.4 * zoom).max(1.0),
            COLOR_DIM,
            galpha * (1.0 - grow),
            vs,
        );
        if grow > 0.0 {
            let wa = view.world(p);
            let wb = view.world(d);
            let h = cache.get(d.id, wa.0, wa.1, wb.0, wb.1, via).clone();
            tree_hypha_draw(
                gm,
                &h,
                (c.x1 + c.x2) * 0.5 - view.cx * zoom,
                (c.y1 + c.y2) * 0.5 - view.cy * zoom,
                zoom,
                COLOR_INK,
                grow,
            );
        }
    }
    gm.halign = FA_CENTER;
    // trunk headings: `global.upgrade_trunks` is always empty in upgrades_init, so the original
    // loop draws nothing
    for d in layout() {
        let p = view.screen(d);
        let r = view.radius(d) * zoom;
        let state = upg_state(st, d.id);
        let hover = view.hover_id == d.id;
        let lvl = upg_level(st, d.id) as f64;
        let maxl = upg_max_level(st, d.id);
        let (ring, tcol) = match state {
            UpgState::Root | UpgState::Owned => (COLOR_INK, COLOR_INK),
            UpgState::Ready => (COLOR_GOOD, COLOR_INK),
            UpgState::Maxed => (COLOR_GOLD, COLOR_GOLD),
            UpgState::Short => (COLOR_DIM, COLOR_DIM),
            _ => (COLOR_DEEP, COLOR_DIM),
        };
        if state == UpgState::Ready {
            tree_draw_glow(gm, p.0, p.1, r, COLOR_GOOD, 0.35 + 0.45 * pulse);
        }
        if hover {
            tree_draw_glow(gm, p.0, p.1, r, COLOR_INK, 0.5);
        }
        gm.set_colour(COLOR_PANEL);
        gm.set_alpha(0.95);
        gm.circle(p.0, p.1, r, false);
        gm.set_alpha(1.0);
        if lvl > 0.0 && maxl > 0.0 {
            gm.set_colour(if state == UpgState::Maxed {
                COLOR_GOLD
            } else {
                COLOR_GOOD
            });
            gm.set_alpha(0.22);
            gm.circle(p.0, p.1, r * 0.92 * (lvl / maxl).sqrt(), false);
            gm.set_alpha(1.0);
        }
        let demo = state == UpgState::Demo;
        let owned = upg_is_owned(st, d.id);
        if !demo {
            let isc = if matches!(d.kind, UpgKind::Unlock | UpgKind::Root) {
                vis("tree_shroom_icon_fit")
            } else {
                1.3
            };
            let ialpha = if owned || d.kind == UpgKind::Root {
                1.0
            } else {
                0.4
            };
            tree_draw_icon(gm, d.icon, p.0, p.1, r * isc, ialpha, true, true);
        } else {
            tree_draw_padlock(gm, p.0, p.1, r * 0.95, tcol, COLOR_PANEL);
        }
        if d.id == "unlock_absorption" && !demo {
            tree_draw_leaf_wrap(
                gm,
                p.0,
                p.1,
                r,
                COLOR_INK,
                if owned { 0.95 } else { 0.4 },
                zoom,
            );
        }
        if d.icon.is_none() && d.kind != UpgKind::Ghost && !demo {
            gm.set_colour(ring);
            gm.set_alpha(if owned { 0.9 } else { 0.45 });
            gm.circle(p.0, p.1, (r * 0.16).max(2.0), false);
            gm.set_alpha(1.0);
        }
        gm.set_colour(ring);
        gm.circle(p.0, p.1, r, true);
        gm.circle(p.0, p.1, r - 1.0, true);
        if hover {
            gm.circle(p.0, p.1, r + 4.0 * zoom, true);
        }
        if d.kind == UpgKind::Level && !demo {
            let lcol = if state == UpgState::Maxed {
                COLOR_GOLD
            } else {
                COLOR_GOOD
            };
            tree_draw_level_ring(
                gm,
                p.0,
                p.1,
                r + 8.0 * zoom,
                lvl,
                maxl,
                lcol,
                COLOR_DEEP,
                4.0 * zoom,
            );
        }
        if matches!(state, UpgState::Ready | UpgState::Short | UpgState::Owned) {
            let cg = upg_children(d.id)
                .iter()
                .map(|k| upg_grow(ctx.sim, k.id))
                .fold(0.0, f64::max);
            let cost = gml_string(upg_cost(st, d.id));
            let csc = view.text_scale(vis("tree_cost_scale"), vis("tree_cost_scale_min"));
            let (cw, ch) = (gm.string_width(&cost) * csc, gm.string_height(&cost) * csc);
            view.label_back(gm, p.0 + r * 0.85, p.1 - r * 1.35, cw, ch, zoom, cg);
            gm.set_colour(if state == UpgState::Ready {
                COLOR_GOOD
            } else {
                COLOR_BAD
            });
            gm.text_transformed(p.0 + r * 0.85, p.1 - r * 1.35, &cost, csc, csc);
        }
        let nm = match d.kind {
            UpgKind::Root => String::new(),
            UpgKind::Ghost => loc("upg_symbiosis"),
            _ => loc(&format!("upg_{}", d.id)),
        };
        let mut nsc = view.text_scale(vis("tree_name_scale"), vis("tree_name_scale_min"));
        let nw = vis("tree_name_width") * nsc / vis("tree_name_scale");
        if !nm.is_empty() {
            let maxh = gm.string_height("M") * 3.0 * nsc;
            let minsc = nsc * 0.45;
            let mut k = 0;
            while k < 12 && nsc > minsc {
                if gm.string_width_ext(&nm, -1.0, nw / nsc) * nsc <= nw + 1.0
                    && gm.string_height_ext(&nm, -1.0, nw / nsc) * nsc <= maxh + 1.0
                {
                    break;
                }
                nsc *= 0.92;
                k += 1;
            }
            let (lw, lh) = (
                gm.string_width_ext(&nm, -1.0, nw / nsc) * nsc,
                gm.string_height_ext(&nm, -1.0, nw / nsc) * nsc,
            );
            view.label_back(
                gm,
                p.0,
                p.1 + r + 10.0 * zoom,
                lw,
                lh,
                zoom,
                upg_grow(ctx.sim, d.id),
            );
        }
        gm.set_colour(tcol);
        gm.text_ext_transformed(p.0, p.1 + r + 10.0 * zoom, &nm, -1.0, nw / nsc, nsc, nsc);
    }
    gm.halign = FA_LEFT;
    // header
    gm.set_colour(COLOR_PANEL);
    gm.set_alpha(0.96);
    let hh = view.header_h;
    gm.rectangle(0.0, 0.0, gw, hh, false);
    gm.set_alpha(1.0);
    gm.set_colour(COLOR_DEEP);
    gm.line(0.0, hh, gw, hh);
    let lr = view.logo_button_rect();
    let title = loc("button_upgrades");
    let pts = format!("{}  {}", loc("hud_points"), st.state.upgrade_points);
    let tsc = hud_fit(gm, &title, 2.4, (lr.x1 - 80.0) * 0.5, 0.0);
    let pw = (40.0 + gm.string_width(&title) * tsc + 30.0).max(300.0);
    gm.set_colour(COLOR_INK);
    gm.text_transformed(40.0, 30.0, &title, tsc, tsc);
    gm.set_colour(if st.state.upgrade_points > 0 {
        COLOR_GOOD
    } else {
        COLOR_DIM
    });
    let psc = hud_fit(gm, &pts, 1.8, lr.x1 - 20.0 - pw, 0.0);
    gm.text_transformed(pw, 36.0, &pts, psc, psc);
    for (r, key, sc, pad) in [
        (lr, "logo_title", 1.3, 20.0),
        (view.recentre_button_rect(), "tree_recentre", 1.4, 20.0),
        (view.close_button_rect(), "button_close", 1.4, 22.0),
    ] {
        gm.set_colour(COLOR_DIM);
        gm.rectangle(r.x1, r.y1, r.x2, r.y2, true);
        gm.set_colour(COLOR_INK);
        hud_text_box(gm, r, &loc(key), sc, pad);
    }
    // footer
    let fh = view.footer_h;
    let fy = gh - fh;
    gm.set_colour(COLOR_PANEL);
    gm.set_alpha(0.96);
    gm.rectangle(0.0, fy, gw, gh, false);
    gm.set_alpha(1.0);
    gm.set_colour(COLOR_DEEP);
    gm.line(0.0, fy, gw, fy);
    gm.set_colour(COLOR_DIM);
    let hint = loc("tree_hint");
    let hsc = hud_fit(gm, &hint, 1.3, gw - 96.0, fh - 30.0);
    gm.text_transformed(48.0, fy + 22.0, &hint, hsc, hsc);
    if !view.hover_id.is_empty() {
        tree_tooltip(view, gm, ctx);
    }
    gm.set_alpha(1.0);
    gm.set_colour(16777215.0);
}

/// The hover tooltip: name, level, description and the status line — the reason a node cannot
/// be bought (requires / no points / max / demo / coming later) or its price.
fn tree_tooltip(view: &TreeView, gm: &mut Gm, ctx: &TreeDrawCtx) {
    let st = &ctx.sim.state;
    let loc = ctx.loc;
    let (gw, gh) = (view.gui_w, view.gui_h);
    let id = view.hover_id.as_str();
    let Some(d) = upg_def(id) else { return };
    let state = upg_state(st, id);
    let ghost = d.kind == UpgKind::Ghost;
    let nm = if ghost {
        loc("upg_symbiosis")
    } else {
        loc(&format!("upg_{id}"))
    };
    let mut desc = if ghost {
        loc("upg_symbiosis_desc")
    } else {
        loc(&format!("upg_{id}_desc"))
    };
    let mut lvl_text = String::new();
    let mut status_col = COLOR_DIM;
    if d.kind == UpgKind::Root {
        desc.clear();
    }
    if d.kind != UpgKind::Root && !ghost {
        lvl_text = format!(
            "{} {} / {}",
            loc("upgrade_level"),
            upg_level(st, id),
            gml_string(upg_max_level(st, id))
        );
    }
    let cost = gml_string(upg_cost(st, id));
    let status = match state {
        UpgState::Root => String::new(),
        UpgState::Ghost => loc("upgrade_coming_later"),
        UpgState::Locked => {
            let pd = upg_def(d.parent).map(|p| p.id).unwrap_or("");
            format!(
                "{}  {}  {} {}",
                loc("upgrade_requires"),
                loc(&format!("upg_{pd}")),
                loc("upgrade_level"),
                gml_string(d.req)
            )
        }
        UpgState::Maxed => {
            status_col = COLOR_GOLD;
            loc("upgrade_max")
        }
        UpgState::Demo => loc("upgrade_demo_locked"),
        UpgState::Ready => {
            status_col = COLOR_GOOD;
            format!(
                "{} {}    {}",
                loc("upgrade_cost"),
                cost,
                loc("upgrade_grow")
            )
        }
        _ => {
            status_col = COLOR_BAD;
            format!(
                "{} {}    {}",
                loc("upgrade_cost"),
                cost,
                loc("upgrade_no_points")
            )
        }
    };
    let pad = 18.0;
    let ssc = 1.25;
    let tpw: f64 = 460.0;
    let gap = 10.0;
    let maxw = (tpw + 240.0).min(gw - 24.0 - pad * 2.0);
    let lvl_part = if lvl_text.is_empty() {
        0.0
    } else {
        32.0 + gm.string_width(&lvl_text) * ssc
    };
    let tnsc = hud_fit(gm, &nm, 1.7, maxw - lvl_part, 0.0);
    let stsc = hud_fit(gm, &status, ssc, maxw, 0.0);
    let nmw = gm.string_width(&nm) * tnsc;
    let lvw = gm.string_width(&lvl_text) * ssc;
    let descw = if desc.is_empty() {
        0.0
    } else {
        tpw.min(gm.string_width(&desc) * ssc)
    };
    let stw = gm.string_width(&status) * stsc;
    let lvl_w = if lvl_text.is_empty() { 0.0 } else { 32.0 + lvw };
    let boxw = (nmw + lvl_w).max(descw).max(stw);
    let lh = gm.string_height("M") * tnsc;
    let desch = if desc.is_empty() {
        0.0
    } else {
        gm.string_height_ext(&desc, -1.0, tpw / ssc) * ssc
    };
    let sth = if status.is_empty() {
        0.0
    } else {
        gm.string_height("M") * ssc
    };
    let bw = boxw + pad * 2.0;
    let st_part = if sth > 0.0 { gap + sth } else { 0.0 };
    let de_part = if desch > 0.0 { gap + desch } else { 0.0 };
    let bh = pad * 2.0 + lh + de_part + st_part;
    let (mxg, myg) = ctx.mouse;
    let mut bx = mxg + 26.0;
    let mut by = myg + 26.0;
    if bx + bw > gw - 12.0 {
        bx = mxg - 26.0 - bw;
    }
    if by + bh > gh - 12.0 {
        by = myg - 26.0 - bh;
    }
    bx = gml::clamp(bx, 12.0, (gw - 12.0 - bw).max(12.0));
    by = gml::clamp(by, 12.0, (gh - 12.0 - bh).max(12.0));
    gm.set_colour(COLOR_PANEL);
    gm.set_alpha(0.96);
    gm.rectangle(bx, by, bx + bw, by + bh, false);
    gm.set_alpha(1.0);
    gm.set_colour(if state == UpgState::Ready {
        COLOR_GOOD
    } else {
        COLOR_DEEP
    });
    gm.rectangle(bx, by, bx + bw, by + bh, true);
    let mut ty = by + pad;
    gm.set_colour(COLOR_INK);
    gm.text_transformed(bx + pad, ty, &nm, tnsc, tnsc);
    if !lvl_text.is_empty() {
        gm.halign = FA_RIGHT;
        gm.set_colour(COLOR_DIM);
        let y = ty + (lh - gm.string_height("M") * ssc) * 0.5;
        gm.text_transformed(bx + bw - pad, y, &lvl_text, ssc, ssc);
        gm.halign = FA_LEFT;
    }
    ty += lh;
    if desch > 0.0 {
        ty += gap;
        gm.set_colour(COLOR_DIM);
        gm.text_ext_transformed(bx + pad, ty, &desc, -1.0, tpw / ssc, ssc, ssc);
        ty += desch;
    }
    if sth > 0.0 {
        ty += gap;
        gm.set_colour(status_col);
        gm.text_transformed(bx + pad, ty, &status, stsc, stsc);
    }
}

// ---- Bevy wiring -----------------------------------------------------------------------------

/// The tree screen state (`global.menu_open` + obj_hud tree fields + `global.tree_hyphae`).
#[derive(Resource, Default)]
pub struct TreeScreen {
    /// `global.menu_open`
    pub open: bool,
    pub view: TreeView,
    pub cache: HyphaCache,
    /// set by the logo button (`logo_open()`); the logo screen (Task 18) clears it
    pub logo_requested: bool,
    /// wheel notches since the last fixed step (up positive)
    wheel: f64,
}

impl TreeScreen {
    /// The HUD's upgrade button: `global.menu_open = true`.
    pub fn open(&mut self) {
        self.open = true;
    }
    /// `global.menu_open = false`.
    pub fn close(&mut self) {
        self.open = false;
    }
}

pub struct TreePlugin;

impl Plugin for TreePlugin {
    fn build(&self, app: &mut App) {
        let screen = TreeScreen {
            open: std::env::var("MYCO_TREE_OPEN").is_ok_and(|v| v == "1"),
            ..default()
        };
        app.insert_resource(screen)
            .add_systems(PreUpdate, collect_wheel.after(bevy::input::InputSystems))
            .add_systems(
                FixedUpdate,
                // obj_hud Begin Step's tree branch: after the menus and the HUD's first stage,
                // before its in-game stage (Task 17)
                tree_step_system.in_set(crate::ui::HudStepSet::Screens),
            )
            .add_systems(Update, tree_draw_system);
    }
}

/// Accumulated wheel delta → `mouse_wheel_up() - mouse_wheel_down()` (-1, 0 or 1). Not
/// `f64::signum`, which is 1 for 0.
pub fn wheel_notch(w: f64) -> f64 {
    if w > 0.0 {
        1.0
    } else if w < 0.0 {
        -1.0
    } else {
        0.0
    }
}

fn collect_wheel(mut screen: ResMut<TreeScreen>, mut wheel: MessageReader<MouseWheel>) {
    for ev in wheel.read() {
        let notches = match ev.unit {
            MouseScrollUnit::Line => ev.y as f64,
            MouseScrollUnit::Pixel => ev.y as f64 / 40.0,
        };
        screen.wheel += notches;
    }
}

/// Before each simulation step: the tree branch of obj_hud Step_1 on the step's input. A click
/// on a node becomes `Command::BuyUpgrade`, applied in the same step by `hud_begin_step`.
pub fn tree_step_system(
    mut screen: ResMut<TreeScreen>,
    mut pending: ResMut<PendingInput>,
    mut blockers: ResMut<UiBlockers>,
    mut capture: ResMut<UiCapture>,
    cams: Res<Cameras>,
    sim: Res<SimRes>,
    menus: Option<Res<crate::ui::menu::Menus>>,
) {
    let screen = &mut *screen;
    // obj_hud Step_1 runs the pause and logo branches before the tree's and returns (Task 18);
    // the tree stays open under them, and an Escape they used is not the tree's
    let held = sim.0.pause_open || menus.as_ref().is_some_and(|m| m.logo.open);
    if screen.open && held {
        pending.menu_open = true;
        capture.tree = true;
        blockers.tree = true;
        return;
    }
    screen
        .view
        .set_gui_size(cams.layout.view_w, cams.layout.view_h);
    screen.cache.mode = myco_core::balance::game_mode(&sim.0.state);
    let wheel = std::mem::take(&mut screen.wheel);
    // Escape: the single chain in `Menus::step` (Task 18) closes the tree through
    // `UiFlags.menu_open` before this system runs. The original returns before the tree branch
    // while a cut-scene blocks or the tasks popup is up (ui_4.lua:1053-1056): the screen stays
    // open but takes no input.
    if screen.open
        && (myco_core::camera::cine_blocking(&sim.0)
            || myco_core::progress::tasks::tasks_popup_open(&sim.0))
    {
        pending.menu_open = true;
        capture.tree = true;
        blockers.tree = true;
        return;
    }
    if !screen.open {
        screen.view.closed_step();
        pending.menu_open = false;
        blockers.tree = false;
        // `global.ui_capture_mouse` is recomputed every step (ui_4.lua:1051)
        capture.tree = false;
        return;
    }
    let cursor = pending.screen_cursor();
    let (mx, my) = cursor
        .map(|p| cams.screen_to_gui(p))
        .map(|g| (g.x, g.y))
        .unwrap_or((-1.0, -1.0));
    let input = TreeInput {
        mx,
        my,
        down: pending.lmb.down,
        pressed: pending.lmb.pressed,
        released: pending.lmb.released,
        wheel: wheel_notch(wheel),
        recentre_key: pending.keys_pressed.contains(&Key::Char('R')),
    };
    let out = screen.view.step(&input);
    if let Some(id) = out.buy {
        pending.commands.push(Command::BuyUpgrade { id });
    }
    if out.logo {
        screen.logo_requested = true;
    }
    if out.close {
        screen.open = false;
    }
    pending.menu_open = screen.open;
    capture.tree = screen.open;
    blockers.tree = screen.open;
}

/// Every frame: the tree screen into [`GuiDraw`], and built hypha lengths into the simulation.
#[allow(clippy::too_many_arguments)]
fn tree_draw_system(
    mut screen: ResMut<TreeScreen>,
    mut draw: ResMut<GuiDraw>,
    mut sim: ResMut<SimRes>,
    assets: Res<GameAssets>,
    locale: Option<Res<LocaleRes>>,
    cams: Res<Cameras>,
    pending: Res<PendingInput>,
    time: Res<Time<Real>>,
) {
    let screen = &mut *screen;
    if !screen.open {
        draw.lists.insert(TREE_DRAW_ORDER, Vec::new());
        return;
    }
    screen
        .view
        .set_gui_size(cams.layout.view_w, cams.layout.view_h);
    if !screen.view.fitted {
        screen.view.fit();
    }
    let mouse = pending
        .screen_cursor()
        .map(|p| cams.screen_to_gui(p))
        .map(|g| (g.x, g.y))
        .unwrap_or((-1.0, -1.0));
    let loc = |k: &str| -> String {
        match &locale {
            Some(l) => l.locale.loc(k).to_string(),
            None => k.to_string(),
        }
    };
    let mut gm = Gm::new(&assets);
    if let Some(l) = &locale {
        gm.font = l.font_small.clone();
    }
    let ctx = TreeDrawCtx {
        sim: &sim.0,
        loc: &loc,
        time_ms: (time.elapsed_secs_f64() * 1000.0).floor(),
        mouse,
    };
    tree_draw(&screen.view, &mut screen.cache, &mut gm, &ctx);
    draw.lists.insert(TREE_DRAW_ORDER, gm.items);
    // ruling R16: tree_hypha_length for upgrades_grow_step
    let lens: Vec<(String, f64)> = screen
        .cache
        .lengths()
        .map(|(id, l)| (id.to_string(), l))
        .collect();
    for (id, l) in lens {
        if sim.0.progress.tree_hypha_len.get(&id) != Some(&l) {
            sim.0.progress.tree_hypha_len.insert(id, l);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wheel_means_no_zoom() {
        assert_eq!(wheel_notch(0.0), 0.0);
        assert_eq!(wheel_notch(2.5), 1.0);
        assert_eq!(wheel_notch(-0.1), -1.0);
    }
}
