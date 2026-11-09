//! Caves (scr_world): the five perk caves, their placement on mazes and rock clusters, the
//! clearance they cut into generated chunks, cave walls in tile queries, touching a cave for its
//! perk, the yellow cave that appears late, and the wrap outlines the mycelium can follow.
//!
//! * **Layout.** `world_cave_rects` places purple (on a maze or near walls), red and green (on
//!   clay rock clusters) and blue (`blue_0`, more with `caves_blue_more`), plus yellow once
//!   `global.cave_yellow` is set. Like the original it is cached per start_x/seed/yellow spot
//!   and grows as `caves_extend` runs; `world_big_bones`, `world_clear_zones` and
//!   `world_relics` are cached the same way. The caches are runtime state in [`CaveRt`]
//!   (`WorldRt.caves`), filled lazily in the original's call order.
//! * **Masks.** `caves_mask_init` reads the cave/relic sprites' alpha (a render-task function,
//!   ruling R7); the masks it produces are embedded (`cave_art.json`, golden `caves_art`) and
//!   `cave_outline_build` runs on them here. The original falls back to a rectangle before the
//!   first draw; myco_core always has the masks.
//! * **Rewards.** Touching a cave grants its perk (`state.cave_perks`, `state.cave_taken`) and
//!   leaves the runtime flash/wrap/popup/burst globals in [`CaveRt`]; popups hold the text key
//!   (`perk_<id>`), the app localizes. Caves give perks, not resources, so no
//!   `SimEvent::Resource` is emitted; the touch sound is `SimEvent::Sound("water_touch")`.
//! * **Time.** `current_time` (ms) is the sim clock `frame * 1000 / 60`.

// `!(a >= b)` and friends are kept as the original writes them (same result for NaN).
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use super::bones;
use super::obstacles;
use super::maze;
use super::stones::{self, CellSet};
use super::WorldRt;
use crate::balance::{gml_truthy, vis, wld, wld_arr};
use crate::events::{SimEvent, SoundCue};
use crate::gml;
use crate::sim::Sim;
use crate::state::{
    BoneSave, FoodSave, GameState, LakeSave, StoneCellSave, StoneSave, WorldChunkSave,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------------------------
// definitions

/// The five perk caves, in `cave_defs()` order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CaveKind {
    Blue,
    Purple,
    Red,
    Green,
    Yellow,
}

impl CaveKind {
    pub const ALL: [CaveKind; 5] = [
        CaveKind::Blue,
        CaveKind::Purple,
        CaveKind::Red,
        CaveKind::Green,
        CaveKind::Yellow,
    ];

    /// The cave id (also the perk id and the mask id).
    pub fn id(self) -> &'static str {
        match self {
            CaveKind::Blue => "blue",
            CaveKind::Purple => "purple",
            CaveKind::Red => "red",
            CaveKind::Green => "green",
            CaveKind::Yellow => "yellow",
        }
    }

    pub fn from_id(id: &str) -> Option<CaveKind> {
        CaveKind::ALL.into_iter().find(|k| k.id() == id)
    }
}

/// A cave definition (`___struct___294..298__cave_defs`).
#[derive(Clone, Debug, PartialEq)]
pub struct CaveDef {
    pub kind: CaveKind,
    /// cave sprite (`cave_perk_<id>`)
    pub sprite: &'static str,
    /// HUD icon sprite
    pub hud: &'static str,
    pub left: f64,
    pub w: f64,
    /// where the stems of the wrap outline reach down to (sprite pixels)
    pub feet: &'static [[f64; 2]],
}

// port: cave_defs
/// `cave_defs()`: blue, purple, red, green, yellow.
pub fn cave_defs() -> &'static [CaveDef; 5] {
    static DEFS: [CaveDef; 5] = [
        CaveDef {
            kind: CaveKind::Blue,
            sprite: "cave_perk_blue",
            hud: "Blue",
            left: 0.0,
            w: 436.0,
            feet: &[
                [145.0, 172.0],
                [170.0, 167.0],
                [202.0, 160.0],
                [212.0, 157.0],
                [240.0, 160.0],
            ],
        },
        CaveDef {
            kind: CaveKind::Purple,
            sprite: "cave_perk_purple",
            hud: "Purple",
            left: 0.0,
            w: 508.0,
            feet: &[
                [245.0, 182.0],
                [262.0, 190.0],
                [285.0, 182.0],
                [305.0, 177.0],
                [317.0, 180.0],
            ],
        },
        CaveDef {
            kind: CaveKind::Red,
            sprite: "cave_perk_red",
            hud: "Red",
            left: 0.0,
            w: 436.0,
            feet: &[
                [150.0, 172.0],
                [172.0, 175.0],
                [187.0, 165.0],
                [207.0, 162.0],
                [235.0, 165.0],
            ],
        },
        CaveDef {
            kind: CaveKind::Green,
            sprite: "cave_perk_green",
            hud: "Green",
            left: 0.0,
            w: 436.0,
            feet: &[[142.0, 165.0], [172.0, 160.0], [190.0, 157.0]],
        },
        CaveDef {
            kind: CaveKind::Yellow,
            sprite: "cave_perk_yellow",
            hud: "Yellow",
            left: 0.0,
            w: 508.0,
            feet: &[
                [245.0, 180.0],
                [265.0, 190.0],
                [285.0, 185.0],
                [302.0, 177.0],
                [312.0, 185.0],
            ],
        },
    ];
    &DEFS
}

// port: cave_def
/// `cave_def(id)`: the definition with this id; blue (the first) for an unknown id.
pub fn cave_def(id: &str) -> &'static CaveDef {
    cave_defs()
        .iter()
        .find(|d| d.kind.id() == id)
        .unwrap_or(&cave_defs()[0])
}

fn def_of(kind: CaveKind) -> &'static CaveDef {
    &cave_defs()[CaveKind::ALL.iter().position(|k| *k == kind).unwrap()]
}

// port: cave_range
/// `cave_range(r, t)`: `lerp(r[0], r[1], t)` over a [lo, hi] balance range.
pub fn cave_range(r: &[f64], t: f64) -> f64 {
    gml::lerp(r[0], r[1], t)
}

// ---------------------------------------------------------------------------------------------
// masks (caves_mask_init's data, ruling R7) and outlines

/// A wrap branch of a cave outline (`___struct___328/329__cave_outline_build`).
#[derive(Clone, Debug, PartialEq)]
pub struct CaveBranch {
    /// outline index the branch starts at
    pub at: i64,
    /// 0.5 for a fork (struct 329), 0 for a main branch (struct 328 has no delay)
    pub delay: f64,
    pub points: Vec<[f64; 2]>,
}

/// One entry of `global.cave_masks`: the solid pixels (alpha >= 250) of a cave or relic
/// sprite and the outline, branches and stems `cave_outline_build` derives from them.
#[derive(Clone, Debug)]
pub struct CaveMask {
    pub id: String,
    pub w: usize,
    pub h: usize,
    /// sprite [width, height]
    pub sprite: [f64; 2],
    solid: Vec<bool>,
    pub outline: Vec<[f64; 2]>,
    pub branches: Vec<CaveBranch>,
    pub stems: Vec<Vec<[f64; 2]>>,
}

impl CaveMask {
    /// `buffer_peek(alpha) >= 250` at a pixel (false outside).
    pub fn solid(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && (x as usize) < self.w
            && (y as usize) < self.h
            && self.solid[y as usize * self.w + x as usize]
    }
}

#[derive(Deserialize)]
struct MaskJson {
    id: String,
    w: usize,
    h: usize,
    sprite: [f64; 2],
    spans: Vec<Vec<usize>>,
}

#[derive(Deserialize)]
struct CaveArtJson {
    masks: Vec<MaskJson>,
    bones: HashMap<String, [f64; 2]>,
}

struct CaveArt {
    masks: Vec<CaveMask>,
    bones: HashMap<String, [f64; 2]>,
}

fn cave_art() -> &'static CaveArt {
    static ART: OnceLock<CaveArt> = OnceLock::new();
    ART.get_or_init(|| {
        let j: CaveArtJson =
            serde_json::from_str(include_str!("cave_art.json")).expect("world/cave_art.json");
        let masks = j
            .masks
            .into_iter()
            .map(|m| {
                let mut solid = vec![false; m.w * m.h];
                for (y, row) in m.spans.iter().enumerate() {
                    for s in row.chunks(2) {
                        for x in s[0]..=s[1] {
                            solid[y * m.w + x] = true;
                        }
                    }
                }
                let mut mask = CaveMask {
                    id: m.id,
                    w: m.w,
                    h: m.h,
                    sprite: m.sprite,
                    solid,
                    outline: Vec::new(),
                    branches: Vec::new(),
                    stems: Vec::new(),
                };
                let feet: &[[f64; 2]] = CaveKind::from_id(&mask.id).map_or(&[], |k| def_of(k).feet);
                let id = mask.id.clone();
                cave_outline_build(&mut mask, &id, feet);
                mask
            })
            .collect();
        CaveArt {
            masks,
            bones: j.bones,
        }
    })
}

/// `global.cave_masks[$ id]` for a cave ("blue".."yellow") or relic ("t_rex", "ufo").
pub fn cave_mask(id: &str) -> Option<&'static CaveMask> {
    cave_art().masks.iter().find(|m| m.id == id)
}

/// `[sprite_get_width, sprite_get_height]` of a bone skeleton sprite (bone kind id).
pub(crate) fn bone_sprite_size(id: &str) -> [f64; 2] {
    cave_art().bones[id]
}

// port: cave_branch_points
/// `cave_branch_points(start, dir, len, seed)`: a wobbly polyline of `max(2, ceil(len / 3))`
/// steps from `start`.
pub fn cave_branch_points(start: [f64; 2], dir: f64, len: f64, seed: f64) -> Vec<[f64; 2]> {
    let mut pts = vec![start];
    let n = 2f64.max((len / 3.0).ceil());
    let (mut px, mut py) = (start[0], start[1]);
    let mut d = dir;
    let mut i = 1.0;
    while i <= n {
        d += (super::hash::cave_hash(seed, i, 9.0) - 0.5) * 30.0;
        px += gml::lengthdir_x(len / n, d);
        py += gml::lengthdir_y(len / n, d);
        pts.push([px, py]);
        i += 1.0;
    }
    pts
}

// port: cave_outline_build
/// `cave_outline_build(m, c)`: from the mask's solid pixels, the 360-point wrap outline around
/// their centre, the branches off it (seeded by the id) and the stems down to `feet`.
pub fn cave_outline_build(m: &mut CaveMask, id: &str, feet: &[[f64; 2]]) {
    let (w, h) = (m.w as f64, m.h as f64);
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    let mut y = 0;
    while y < m.h {
        let mut x = 0;
        while x < m.w {
            if m.solid(x as i64, y as i64) {
                sx += x as f64;
                sy += y as f64;
                n += 1.0;
            }
            x += 3;
        }
        y += 3;
    }
    let cx = sx / 1f64.max(n);
    let cy = sy / 1f64.max(n);
    let maxr = gml::point_distance(0.0, 0.0, w, h);
    let inside = |px: f64, py: f64| px >= 0.0 && py >= 0.0 && px < w && py < h;
    let mut radii = Vec::with_capacity(360);
    for a in 0..360 {
        let a = a as f64;
        let dx = gml::dcos(a);
        let dy = -gml::dsin(a);
        let mut r = 0.0;
        let mut rr = maxr;
        while rr > 0.0 {
            let px = gml::round(cx + dx * rr);
            let py = gml::round(cy + dy * rr);
            if inside(px, py) && m.solid(px as i64, py as i64) {
                r = rr;
                while r + 1.0 < maxr {
                    let qx = gml::round(cx + dx * (r + 1.0));
                    let qy = gml::round(cy + dy * (r + 1.0));
                    if !inside(qx, qy) || !m.solid(qx as i64, qy as i64) {
                        break;
                    }
                    r += 1.0;
                }
                break;
            }
            rr -= 3.0;
        }
        radii.push(r);
    }
    let out = vis("cave_wrap_out_px");
    let wob = vis("cave_wrap_wobble_px");
    m.outline.clear();
    for a in 0..360usize {
        let mut rs = (radii[(a + 358) % 360]
            + radii[(a + 359) % 360]
            + radii[a] * 2.0
            + radii[(a + 1) % 360]
            + radii[(a + 2) % 360])
            / 6.0;
        let af = a as f64;
        rs += out + wob * ((af * 0.13 + 1.7).sin() * 0.4 + (af * 0.37).sin() * 0.6);
        m.outline
            .push([cx + gml::dcos(af) * rs, cy - gml::dsin(af) * rs]);
    }
    m.branches.clear();
    let first = id.chars().next().map_or(0, |c| c as u32) as f64;
    let seed = id.chars().count() as f64 * 131.0 + first;
    let every = 3f64.max(vis("cave_branch_every"));
    let hash = |b: f64, c: f64| super::hash::cave_hash(seed, b, c);
    let mut b = 0.0;
    while b < 360.0 {
        let at = gml::gmod(b + (hash(b, 1.0) * every).floor(), 360.0);
        let len = gml::lerp(
            vis("cave_branch_min_px"),
            vis("cave_branch_max_px"),
            hash(b, 2.0),
        );
        let dir = at + (hash(b, 3.0) - 0.5) * 2.0 * vis("cave_branch_spread_deg");
        let pts = cave_branch_points(m.outline[at as usize], dir, len, seed + b);
        let fork = hash(b, 4.0) < vis("cave_branch_fork_chance");
        let fp = pts[(pts.len() as f64 * 0.5).floor() as usize];
        m.branches.push(CaveBranch {
            at: at as i64,
            delay: 0.0,
            points: pts,
        });
        if fork {
            let side = if hash(b, 5.0) < 0.5 { -1.0 } else { 1.0 };
            m.branches.push(CaveBranch {
                at: at as i64,
                delay: 0.5,
                points: cave_branch_points(
                    fp,
                    dir + side * gml::lerp(25.0, 50.0, hash(b, 6.0)),
                    len * 0.55,
                    seed + b + 7.0,
                ),
            });
        }
        b += every;
    }
    m.stems.clear();
    for (f, ft) in feet.iter().enumerate() {
        let mut best: Option<[f64; 2]> = None;
        let mut bd = f64::INFINITY;
        for p in &m.outline {
            if p[1] <= ft[1] {
                continue;
            }
            let d = (p[0] - ft[0]).abs() * 3.0 + p[1];
            if d < bd {
                bd = d;
                best = Some(*p);
            }
        }
        let Some(best) = best else { continue };
        let steps = 4f64.max((gml::point_distance(best[0], best[1], ft[0], ft[1]) / 4.0).ceil());
        let ph = f as f64 * 1.9;
        let mut stem = Vec::new();
        let mut j = 0.0;
        while j <= steps {
            let t = j / steps;
            let sway = vis("cave_stem_sway_px")
                * (t * std::f64::consts::PI * 2.5 + ph).sin()
                * (t * std::f64::consts::PI).sin();
            stem.push([
                gml::lerp(best[0], ft[0], t) + sway,
                gml::lerp(best[1], ft[1], t),
            ]);
            j += 1.0;
        }
        m.stems.push(stem);
    }
}

// port: cave_solid_at
/// `cave_solid_at(r, x, y)`: the cave/relic mask `mask_id`, drawn at (x1, y1) with scale `k`,
/// is solid at world (x, y).
pub fn cave_solid_at(mask_id: &str, k: f64, x1: f64, y1: f64, x: f64, y: f64) -> bool {
    let lx = ((x - x1) / k).floor();
    let ly = ((y - y1) / k).floor();
    let Some(mk) = cave_mask(mask_id) else {
        return false;
    };
    if lx < 0.0 || ly < 0.0 || lx >= mk.w as f64 || ly >= mk.sprite[1] {
        return false;
    }
    mk.solid(lx as i64, ly as i64)
}

// ---------------------------------------------------------------------------------------------
// layout

/// A placed cave (`___struct___299__cave_place`): `id` is "purple", "red", "green", "yellow" or
/// "blue_<n>" (the key of `cave_taken`), `kind` its definition.
#[derive(Clone, Debug, PartialEq)]
pub struct Cave {
    pub id: String,
    pub kind: CaveKind,
    /// draw scale (`vis("cave_scale")`)
    pub k: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Cave {
    /// `cave_solid_at(self, x, y)`.
    pub fn solid_at(&self, x: f64, y: f64) -> bool {
        cave_solid_at(self.kind.id(), self.k, self.x1, self.y1, x, y)
    }

    pub fn zone(&self) -> Zone {
        Zone {
            id: self.id.clone(),
            x1: self.x1,
            y1: self.y1,
            x2: self.x2,
            y2: self.y2,
        }
    }
}

/// A box caves keep clear of resources (`world_clear_zones` entries: caves and big bones).
#[derive(Clone, Debug, PartialEq)]
pub struct Zone {
    pub id: String,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// What a cave host was made from.
#[derive(Clone, Debug, PartialEq)]
pub enum HostKey {
    /// a maze or wall band (`obstacle.key`)
    Obstacle(i64),
    /// a rock cluster (`cluster.id`)
    Cluster(String),
}

/// A place a cave can sit in (`___struct___300/301/306`): a maze/wall band down to the wall
/// depth, or a rock cluster (with its centre and radius in cells).
#[derive(Clone, Debug, PartialEq)]
pub struct CaveHost {
    pub key: HostKey,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub used: bool,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}

/// One side of the blue-cave chain (`___struct___303/304`).
#[derive(Clone, Debug, PartialEq)]
pub struct BlueLane {
    pub dir: f64,
    pub n: f64,
    pub gap: f64,
    pub cursor: f64,
}

/// `global.cave_blue_state` (`___struct___302`): the hosts blue caves avoid or claim and the
/// two lanes `caves_extend` advances.
#[derive(Clone, Debug, PartialEq)]
pub struct BlueState {
    pub mazes: Vec<CaveHost>,
    pub walls: Vec<CaveHost>,
    pub left: BlueLane,
    pub right: BlueLane,
}

/// `global.cave_yellow` (`___struct___305__cave_yellow_pick`), saved as `state.cave_yellow`.
#[derive(Clone, Debug, PartialEq, serde::Serialize, Deserialize)]
pub struct YellowSpot {
    pub x: f64,
    pub y: f64,
    pub top: f64,
    pub bottom: f64,
    /// id of the deep rock cluster it sits in
    pub cluster: String,
}

/// `global.cave_wrap[$ id]` (`___struct___324/334`): when a cave/relic was taken and where
/// (sprite pixels) the wrap animation starts.
#[derive(Clone, Debug, PartialEq)]
pub struct CaveWrap {
    pub born: f64,
    pub from_x: f64,
    pub from_y: f64,
}

/// `global.cave_popups` entry (`___struct___335`): `text` is the locale key (`perk_<id>`).
#[derive(Clone, Debug, PartialEq)]
pub struct CavePopup {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub born: f64,
}

/// Cache key of `world_cave_rects`: start_x, seed and the yellow spot.
#[derive(Clone, Copy, Debug, PartialEq)]
struct RectsKey {
    start_x: f64,
    seed: i64,
    yx: Option<f64>,
    yy: Option<f64>,
}

/// Runtime cave state: the cave globals that are not saved (reset by game_init and load_game,
/// as `WorldRt` is rebuilt then) and the statics of the cached layout functions.
#[derive(Clone, Debug, Default)]
pub struct CaveRt {
    /// `global.cave_yellow` (live copy of `state.cave_yellow`; [`set_cave_yellow`] sets both)
    pub yellow: Option<YellowSpot>,
    /// `global.cave_blue_state`
    pub blue_state: Option<BlueState>,
    /// `global.cave_flash[$ id]`: current_time of the take
    pub flash: BTreeMap<String, f64>,
    /// `global.cave_wrap`
    pub wrap: BTreeMap<String, CaveWrap>,
    /// `global.cave_popups` (the drawing code drops them after `cave_popup_ms`)
    pub popups: Vec<CavePopup>,
    /// `global.cave_burst_pending`: a blue take's node burst, consumed by the mycelium step
    pub burst_pending: Option<(f64, f64)>,
    rects: Vec<Cave>,
    rects_key: Option<RectsKey>,
    pub(crate) big_bones: Vec<bones::BigBone>,
    pub(crate) big_bones_key: Option<(f64, i64)>,
    pub(crate) bones_gen: u64,
    pub(crate) relics: Vec<super::relics::Relic>,
    pub(crate) relics_gen: Option<u64>,
    zones: Vec<Zone>,
    zones_key: Option<(f64, i64, usize)>,
}

impl WorldRt {
    /// `world_cave_rects()`: the placed caves (computed and cached on first use).
    pub fn cave_rects(&mut self) -> &[Cave] {
        world_cave_rects(self);
        &self.caves.rects
    }

    /// The caves as last computed (empty before the first `cave_rects`/generation).
    pub fn caves(&self) -> &[Cave] {
        &self.caves.rects
    }
}

fn boxes<'a>(it: impl IntoIterator<Item = &'a CaveHost>) -> Vec<[f64; 4]> {
    it.into_iter().map(|h| [h.x1, h.y1, h.x2, h.y2]).collect()
}

// port: cave_place
/// `cave_place(list, kind, id, cx, cy, lo, hi, extra)`: the first of 12 spots alternating
/// around `cx` (vertically clamped into [lo, hi] with the margin) that keeps the clearance
/// from every cave in `list` and box in `extra`; pushed onto `list`.
#[allow(clippy::too_many_arguments)]
pub fn cave_place(
    list: &mut Vec<Cave>,
    kind: CaveKind,
    id: &str,
    cx: f64,
    cy: f64,
    lo: f64,
    hi: f64,
    extra: &[[f64; 4]],
) -> Option<Cave> {
    let def = def_of(kind);
    let sc = vis("cave_scale");
    let w = gml::round(def.w * sc);
    let h = gml::round(cave_mask(kind.id()).unwrap().sprite[1] * sc);
    let margin = wld("caves_margin_px");
    let gap = wld("caves_clearance_px") * 2.0;
    let y0 = gml::clamp(
        cy - h * 0.5,
        lo + margin,
        (lo + margin).max(hi - margin - h),
    )
    .floor();
    let clear = |x0: f64, r: [f64; 4]| {
        x0 + w + gap <= r[0] || x0 >= r[2] + gap || y0 + h + gap <= r[1] || y0 >= r[3] + gap
    };
    for i in 0..12 {
        let dir = if i % 2 == 1 { 1.0 } else { -1.0 };
        let off = ((i + 1) / 2) as f64 * dir * (w + gap);
        let x0 = (cx + off - w * 0.5).floor();
        let ok = list.iter().all(|r| clear(x0, [r.x1, r.y1, r.x2, r.y2]))
            && extra.iter().all(|r| clear(x0, *r));
        if ok {
            let p = Cave {
                id: id.to_string(),
                kind,
                k: sc,
                x1: x0,
                y1: y0,
                x2: x0 + w,
                y2: y0 + h,
            };
            list.push(p.clone());
            return Some(p);
        }
    }
    None
}

// port: cave_host_pick
/// `cave_host_pick(hosts, side, dist)`: the unused host on `side` (0: either) whose centre is
/// closest to `dist` from the start.
pub fn cave_host_pick(hosts: &[CaveHost], start_x: f64, side: f64, dist: f64) -> Option<usize> {
    let mut best = None;
    let mut best_d = f64::INFINITY;
    for (i, h) in hosts.iter().enumerate() {
        let d = (h.x1 + h.x2) * 0.5 - start_x;
        if !h.used && (side == 0.0 || gml::sign(d) == side) {
            let e = (d.abs() - dist).abs();
            if e < best_d {
                best_d = e;
                best = Some(i);
            }
        }
    }
    best
}

// port: anon__10783__world_cave_rects__scr_world
/// `place_host`: a cave inside a maze band at (tx, ty) of its usable area; claims the host.
#[allow(clippy::too_many_arguments)]
pub fn place_host(
    list: &mut Vec<Cave>,
    kind: CaveKind,
    id: &str,
    host: &mut CaveHost,
    tx: f64,
    ty: f64,
    lo: f64,
    hi: f64,
) -> Option<Cave> {
    let def = def_of(kind);
    let cx = gml::lerp(
        host.x1 + def.w * 0.5 + 144.0,
        host.x2 - def.w * 0.5 - 144.0,
        tx,
    );
    let cy = gml::lerp(host.y1 + 336.0, host.y2 - 432.0, ty);
    host.used = true;
    cave_place(
        list,
        kind,
        id,
        cx,
        cy,
        lo.max(host.y1),
        hi.min(host.y2),
        &[],
    )
}

// port: anon__11290__world_cave_rects__scr_world
/// `place_cluster`: a cave near a rock cluster's centre; claims the host.
pub fn place_cluster(
    list: &mut Vec<Cave>,
    kind: CaveKind,
    id: &str,
    host: &mut CaveHost,
    tx: f64,
    ty: f64,
) -> Option<Cave> {
    host.used = true;
    cave_place(
        list,
        kind,
        id,
        host.cx + (tx - 0.5) * host.r * 48.0 * 0.4,
        host.cy + (ty - 0.5) * host.r * 48.0 * 0.3,
        host.y1,
        host.y2,
        &[],
    )
}

fn rects_key(w: &WorldRt) -> RectsKey {
    let spot = cave_yellow_spot(w);
    RectsKey {
        start_x: w.params.start_x,
        seed: w.params.seed,
        yx: spot.map(|s| s.x),
        yy: spot.map(|s| s.y),
    }
}

// port: world_cave_rects
/// `world_cave_rects()`: fill `w.caves` with the cave layout unless the cached one is for
/// this start, seed and yellow spot. Read the result with [`WorldRt::cave_rects`].
pub fn world_cave_rects(w: &mut WorldRt) {
    let key = rects_key(w);
    if w.caves.rects_key == Some(key) {
        return;
    }
    w.caves.rects.clear();
    w.caves.rects_key = Some(key);
    if wld("caves_enabled") <= 0.0 {
        return;
    }
    let spot = w.caves.yellow.clone();
    let sw = vis("screen_width_px");
    let sh = vis("screen_height_px");
    let sx = w.params.start_x;
    let sy = w.params.surface_y;
    let top = sy + w.params.tile as f64 * 2.0;
    let dense = w.dense_y();
    let cell = 48.0;
    let reach = sw * wld("caves_host_reach_screens");
    let mut mazes = Vec::new();
    let mut walls = Vec::new();
    let wall_depth = gml::round(wld("surface_wall_depth_screens") * sh / cell) * cell;
    for side in [-1.0, 1.0] {
        for o in obstacles::world_obstacles_reach(w, side, sx + side * reach) {
            let r = CaveHost {
                key: HostKey::Obstacle(o.key),
                x1: o.x1,
                y1: sy,
                x2: o.x2,
                y2: sy + wall_depth,
                used: false,
                cx: 0.0,
                cy: 0.0,
                r: 0.0,
            };
            if o.kind == "maze" {
                mazes.push(r);
            } else {
                walls.push(r);
            }
        }
    }
    let mut clay = Vec::new();
    for c in maze::world_rock_clusters(w) {
        let r = CaveHost {
            key: HostKey::Cluster(c.id.clone()),
            x1: c.x1,
            y1: c.y1,
            x2: c.x2,
            y2: c.y2,
            used: false,
            cx: (c.col + 0.5) * cell,
            cy: (c.row + 0.5) * cell,
            r: c.r,
        };
        if c.tier == "clay" {
            clay.push(r);
        }
    }
    let h = |a: f64, b: f64, c: f64| w.hash01(a, b, c);
    let mut out = Vec::new();
    let purple_x = cave_range(wld_arr("caves_purple_x"), h(2.0, 2.0, 303.0));
    // the side argument is DAT_141379b30 (never written -> 0)
    if let Some(i) = cave_host_pick(&mazes, sx, 0.0, sw * purple_x) {
        let (tx, ty) = (h(2.0, 4.0, 303.0), h(2.0, 3.0, 303.0));
        place_host(
            &mut out,
            CaveKind::Purple,
            "purple",
            &mut mazes[i],
            tx,
            ty,
            top,
            dense,
        );
    } else {
        let cy = sy + sh * cave_range(wld_arr("caves_purple_depth"), h(2.0, 3.0, 303.0));
        cave_place(
            &mut out,
            CaveKind::Purple,
            "purple",
            sx + sw * purple_x,
            cy,
            top,
            dense,
            &boxes(&walls),
        );
    }
    let dir = if h(3.0, 1.0, 304.0) < 0.5 { -1.0 } else { 1.0 };
    for (k, kind) in [CaveKind::Red, CaveKind::Green].into_iter().enumerate() {
        let kf = k as f64;
        let s = if k == 0 { dir } else { -dir };
        let dist = sw * cave_range(wld_arr("caves_clay_x"), h(3.0 + kf, 2.0, 304.0));
        if let Some(i) = cave_host_pick(&clay, sx, s, dist) {
            let (tx, ty) = (h(3.0 + kf, 4.0, 304.0), h(3.0 + kf, 3.0, 304.0));
            place_cluster(&mut out, kind, kind.id(), &mut clay[i], tx, ty);
        }
    }
    let dir = if h(1.0, 1.0, 301.0) < 0.5 { -1.0 } else { 1.0 };
    let first = wld_arr("caves_blue_first_x");
    let every = sw * wld("caves_blue_every_x");
    let lane = |dir: f64| BlueLane {
        dir,
        n: 0.0,
        gap: every,
        cursor: 0.0,
    };
    let mut state = BlueState {
        mazes,
        walls,
        left: lane(-1.0),
        right: lane(1.0),
    };
    let bx0 = sx + dir * sw * cave_range(first, h(1.0, 2.0, 301.0));
    let by0 = sy + sh * cave_range(wld_arr("caves_blue_depth"), h(1.0, 3.0, 301.0));
    let b = cave_blue_place(top, dense, &mut out, &mut state, "blue_0", bx0, by0);
    let bx = match &b {
        None => sx + dir * sw * first[1],
        Some(b) => (b.x1 + b.x2) * 0.5,
    };
    let rbase = if dir > 0.0 { bx } else { sx + sw * first[0] };
    state.right.cursor = rbase + state.right.gap;
    let lbase = if dir < 0.0 { bx } else { sx - sw * first[0] };
    state.left.cursor = lbase - state.left.gap;
    w.caves.rects = out;
    w.caves.blue_state = Some(state);
    caves_extend(w, sx - reach * 0.5);
    caves_extend(w, sx + reach * 0.5);
    if let Some(spot) = spot {
        let extra: Vec<[f64; 4]> = bones::world_big_bones(w)
            .iter()
            .map(|b| [b.x1, b.y1, b.x2, b.y2])
            .collect();
        let mut out = std::mem::take(&mut w.caves.rects);
        cave_place(
            &mut out,
            CaveKind::Yellow,
            "yellow",
            spot.x,
            spot.y,
            spot.top,
            spot.bottom,
            &extra,
        );
        w.caves.rects = out;
    }
}

// port: cave_blue_place
/// `cave_blue_place(list, st, id, x, y)`: a blue cave between the surface (two tiles down) and
/// the dense soil, avoiding walls and claimed mazes; the mazes it lands on become claimed.
pub fn cave_blue_place(
    top: f64,
    dense: f64,
    list: &mut Vec<Cave>,
    st: &mut BlueState,
    id: &str,
    x: f64,
    y: f64,
) -> Option<Cave> {
    let mut avoid = boxes(&st.walls);
    avoid.extend(boxes(st.mazes.iter().filter(|m| m.used)));
    let p = cave_place(list, CaveKind::Blue, id, x, y, top, dense, &avoid);
    if let Some(p) = &p {
        for m in &mut st.mazes {
            if p.x2 > m.x1 && p.x1 < m.x2 && p.y2 > m.y1 && p.y1 < m.y2 {
                m.used = true;
            }
        }
    }
    p
}

// port: caves_extend
/// `caves_extend(x)`: with `caves_blue_more`, add blue caves along the lane toward `x` until it
/// is `caves_extend_ahead_screens` ahead (world generation calls it for every chunk layer).
pub fn caves_extend(w: &mut WorldRt, x: f64) {
    if wld("caves_blue_more") <= 0.0 {
        return;
    }
    world_cave_rects(w);
    let Some(mut st) = w.caves.blue_state.take() else {
        return;
    };
    let sw = vis("screen_width_px");
    let sh = vis("screen_height_px");
    let sy = w.params.surface_y;
    let top = sy + w.params.tile as f64 * 2.0;
    let dense = w.dense_y();
    let dir = if x >= w.params.start_x { 1.0 } else { -1.0 };
    let right = dir > 0.0;
    let target = x + dir * sw * wld("caves_extend_ahead_screens");
    loop {
        let lane = if right { &mut st.right } else { &mut st.left };
        let go = if dir <= 0.0 {
            lane.cursor >= target
        } else {
            lane.cursor <= target
        };
        if !go {
            break;
        }
        lane.n += 1.0;
        let off = dir * lane.n;
        let jit = (w.hash01(off, 4.0, 302.0) - 0.5) * 2.0 * wld("caves_blue_jitter") * lane.gap;
        let cx = lane.cursor + jit;
        for o in obstacles::world_obstacles_reach(w, dir, cx + dir * sw * 4.0) {
            let arr = if o.kind == "maze" {
                &mut st.mazes
            } else {
                &mut st.walls
            };
            if !arr.iter().any(|h| h.key == HostKey::Obstacle(o.key)) {
                arr.push(CaveHost {
                    key: HostKey::Obstacle(o.key),
                    x1: o.x1,
                    y1: sy,
                    x2: o.x2,
                    y2: sy + gml::round(wld("surface_wall_depth_screens") * sh / 48.0) * 48.0,
                    used: false,
                    cx: 0.0,
                    cy: 0.0,
                    r: 0.0,
                });
            }
        }
        let y = sy + sh * cave_range(wld_arr("caves_blue_depth"), w.hash01(off, 5.0, 302.0));
        let id = format!("blue_{}", off as i64);
        cave_blue_place(top, dense, &mut w.caves.rects, &mut st, &id, cx, y);
        let lane = if right { &mut st.right } else { &mut st.left };
        lane.gap *= 1f64.max(wld("caves_blue_gap_growth"));
        lane.cursor += dir * lane.gap;
    }
    w.caves.blue_state = Some(st);
}

// port: world_clear_zones
/// `world_clear_zones()`: the caves and big bones, re-collected when the cave count changes.
pub fn world_clear_zones(w: &mut WorldRt) -> &[Zone] {
    world_cave_rects(w);
    let key = (w.params.start_x, w.params.seed, w.caves.rects.len());
    if w.caves.zones_key != Some(key) {
        w.caves.zones_key = Some(key);
        let mut zones: Vec<Zone> = w.caves.rects.iter().map(Cave::zone).collect();
        zones.extend(bones::world_big_bones(w).iter().map(|b| b.zone()));
        w.caves.zones = zones;
    }
    &w.caves.zones
}

// port: world_cave_hits
/// `world_cave_hits(x1, y1, x2, y2, pad)`: the box comes within the clearance (+ pad) of a
/// cave or big bone.
pub fn world_cave_hits(
    w: &mut WorldRt,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    pad: Option<f64>,
) -> bool {
    let m = wld("caves_clearance_px") + pad.unwrap_or(0.0);
    world_clear_zones(w)
        .iter()
        .any(|z| x2 > z.x1 - m && x1 < z.x2 + m && y2 > z.y1 - m && y1 < z.y2 + m)
}

// port: world_cave_solid_any
/// `world_cave_solid_any(x, y)`: a cave wall is at this point (tile queries then say stone).
pub fn world_cave_solid_any(w: &mut WorldRt, x: f64, y: f64) -> bool {
    world_cave_rects(w);
    w.caves
        .rects
        .iter()
        .any(|r| !(x < r.x1 || x >= r.x2 || y < r.y1 || y >= r.y2) && r.solid_at(x, y))
}

// port: anon__46074__world_clear_caves__scr_world
fn lake_outside(r: &[f64; 4], o: &LakeSave) -> bool {
    o.body_x + o.body_w <= r[0]
        || o.body_x >= r[2]
        || o.body_y + o.body_h <= r[1]
        || o.body_y >= r[3]
}

// port: anon__46304__world_clear_caves__scr_world
fn food_outside(r: &[f64; 4], o: &FoodSave) -> bool {
    o.x < r[0] || o.x > r[2] || o.y < r[1] || o.y > r[3]
}

// port: anon__46678__world_clear_caves__scr_world
fn stone_outside(r: &[f64; 4], o: &StoneSave) -> bool {
    o.x < r[0] || o.x > r[2] || o.y < r[1] || o.y > r[3]
}

// port: anon__46870__world_clear_caves__scr_world
fn bone_outside(r: &[f64; 4], o: &BoneSave) -> bool {
    o.x + o.w <= r[0] || o.x >= r[2] || o.y + o.h <= r[1] || o.y >= r[3]
}

// port: world_clear_caves
/// `world_clear_caves(chunk)`: inside every zone (+ clearance) touching the chunk, drop lakes,
/// foods, stones and bones, turn stone tiles into soil and cut out the stone deposits there
/// (whole deposits up to `caves_clear_deposit_max_cells`, else just the cell).
pub fn world_clear_caves(w: &mut WorldRt, ch: &mut WorldChunkSave) {
    let zones = world_clear_zones(w).to_vec();
    let p = &w.params;
    let (tile, sy, cols) = (p.tile as f64, p.surface_y, p.cols);
    let x0 = (ch.cx * p.chunk_w) as f64;
    let x1 = x0 + p.chunk_w as f64;
    let pad = wld("caves_clearance_px");
    let max_cells = wld("caves_clear_deposit_max_cells");
    for z in &zones {
        let (zx1, zy1, zx2, zy2) = (z.x1 - pad, z.y1 - pad, z.x2 + pad, z.y2 + pad);
        if !(zx2 > x0 && zx1 < x1) {
            continue;
        }
        let r = [zx1, zy1, zx2, zy2];
        ch.lakes.retain(|o| lake_outside(&r, o));
        ch.foods.retain(|o| food_outside(&r, o));
        let sr = stones::stone_reach(w);
        let rs = [zx1 - sr, zy1 - sr, zx2 + sr, zy2 + sr];
        ch.stones.retain(|o| stone_outside(&rs, o));
        ch.bones.retain(|o| bone_outside(&r, o));
        let r0 = 0f64.max(((zy1 - sy) / tile).floor()) as i64;
        let r1 = ((ch.rows - 1) as f64).min(((zy2 - sy) / tile).floor()) as i64;
        let c0 = ((ch.cx * cols) as f64).max((zx1 / tile).floor()) as i64;
        let c1 = (((ch.cx + 1) * cols - 1) as f64).min((zx2 / tile).floor()) as i64;
        for row in r0..=r1 {
            for col in c0..=c1 {
                if w.chunk_get(ch, col, row) == 2 {
                    w.chunk_set(ch, col, row, 0, 0.0);
                }
            }
        }
        let mut cut = CellSet::new();
        let cells = ch.stone_cells.clone();
        for s in &cells {
            let (sx, syy) = (s.col as f64 * 48.0, s.row as f64 * 48.0);
            if sx + 48.0 <= zx1 || sx >= zx2 || syy + 48.0 <= zy1 || syy >= zy2 {
                continue;
            }
            let key = stones::stone_cell_key(s.col, s.row);
            if cut.contains(&(s.col, s.row)) {
                continue;
            }
            let mut dep: Vec<StoneCellSave> = vec![s.clone()];
            let mut seen: BTreeSet<String> = BTreeSet::from([key]);
            let mut q: Vec<StoneCellSave> = vec![s.clone()];
            while !q.is_empty() && dep.len() as f64 <= max_cells {
                let cur = q.pop().unwrap();
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let nk = stones::stone_cell_key(cur.col + dx, cur.row + dy);
                    let Some(nb) = ch.stone_map.get(&nk) else {
                        continue;
                    };
                    if seen.contains(&nk) {
                        continue;
                    }
                    seen.insert(nk);
                    dep.push(nb.clone());
                    q.push(nb.clone());
                }
            }
            if dep.len() as f64 > max_cells {
                dep = vec![s.clone()];
            }
            for d in &dep {
                cut.insert((d.col, d.row));
            }
        }
        if !cut.is_empty() {
            let keep: Vec<StoneCellSave> = cells
                .into_iter()
                .filter(|c| !cut.contains(&(c.col, c.row)))
                .collect();
            for &(c, r) in &cut {
                ch.stone_map.remove(&stones::stone_cell_key(c, r));
            }
            ch.stone_cells = keep;
            stones::stone_redress_after_cut(w, ch, &mut cut);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// the yellow cave

// port: cave_yellow_spot
/// `cave_yellow_spot()`: `global.cave_yellow` once the yellow cave has appeared.
pub fn cave_yellow_spot(w: &WorldRt) -> Option<&YellowSpot> {
    w.caves.yellow.as_ref()
}

/// Set `global.cave_yellow` (live in `WorldRt.caves.yellow`, saved in `state.cave_yellow`).
pub fn set_cave_yellow(sim: &mut Sim, spot: Option<YellowSpot>) {
    sim.state.state.cave_yellow = spot
        .as_ref()
        .map(|s| serde_json::to_value(s).expect("yellow spot"));
    sim.world.caves.yellow = spot;
}

// port: world_area_untouched
/// `world_area_untouched(x1, y1, x2, y2, pad)`: no generated chunk under the box (+ pad)
/// reaches down to it yet.
pub fn world_area_untouched(w: &WorldRt, x1: f64, y1: f64, x2: f64, _y2: f64, pad: f64) -> bool {
    let p = &w.params;
    let cw = p.chunk_w as f64;
    let c1 = ((x1 - pad) / cw).floor() as i64;
    let c2 = ((x2 + pad) / cw).floor() as i64;
    for c in c1..=c2 {
        if let Some(ch) = w.chunk(c) {
            if p.surface_y + (ch.rows * p.tile) as f64 > y1 - pad {
                return false;
            }
        }
    }
    true
}

/// `cave_yellow_pick()` with the mycelium's extreme tips given by `tip(dir)` (what
/// `mycelium_extreme_tip(dir)` returns).
pub fn cave_yellow_pick_with(
    w: &mut WorldRt,
    tip: impl Fn(f64) -> Option<(f64, f64)>,
) -> Option<YellowSpot> {
    let sx = w.params.start_x;
    let sw = vis("screen_width_px");
    let beyond = sw * cave_range(wld_arr("caves_yellow_beyond_x"), w.hash01(5.0, 2.0, 305.0));
    let pad = wld("caves_clearance_px") + wld("caves_margin_px");
    let dir = if w.hash01(5.0, 1.0, 305.0) < 0.5 {
        -1.0
    } else {
        1.0
    };
    maze::world_rock_clusters(w);
    let w: &WorldRt = w;
    let clusters = w.rock_clusters.as_deref().unwrap_or_default();
    let mut best = None;
    for t in 0..2 {
        let s = if t == 0 { dir } else { -dir };
        let tx = tip(s).map_or(sx, |p| p.0);
        let need = 0f64.max((tx - sx) * s) + beyond;
        let mut best_d = f64::INFINITY;
        for c in clusters.iter().filter(|c| c.tier == "deep") {
            let d = ((c.x1 + c.x2) * 0.5 - sx) * s;
            if d >= need
                && !(d - need >= best_d)
                && world_area_untouched(w, c.x1, c.y1, c.x2, c.y2, pad)
            {
                best = Some(c);
                best_d = d - need;
            }
        }
        if best.is_some() {
            break;
        }
    }
    let best = best?;
    Some(YellowSpot {
        x: (best.col + 0.5) * 48.0 + (w.hash01(5.0, 4.0, 305.0) - 0.5) * best.r * 48.0 * 0.4,
        y: (best.row + 0.5) * 48.0 + (w.hash01(5.0, 3.0, 305.0) - 0.5) * best.r * 48.0 * 0.3,
        top: best.y1,
        bottom: best.y2,
        cluster: best.id.clone(),
    })
}

// port: cave_yellow_pick
/// `cave_yellow_pick()`: a spot in the nearest untouched deep rock cluster at least
/// `caves_yellow_beyond_x` screens past the mycelium's furthest tip (preferred side first).
pub fn cave_yellow_pick(sim: &mut Sim) -> Option<YellowSpot> {
    let tips = [
        crate::mycelium::mycelium_extreme_tip(sim, -1.0),
        crate::mycelium::mycelium_extreme_tip(sim, 1.0),
    ];
    cave_yellow_pick_with(&mut sim.world, |dir| if dir < 0.0 { tips[0] } else { tips[1] })
}

// port: cave_yellow_step
/// `cave_yellow_step()` (from game_step): counts play frames; after
/// `caves_yellow_after_minutes` of play, tries every 5 s to make the yellow cave appear.
pub fn cave_yellow_step(sim: &mut Sim) {
    if sim.title_open || sim.intro_active || sim.pause_open {
        return;
    }
    sim.state.state.play_frames += 1;
    if cave_yellow_spot(&sim.world).is_some()
        || super::perks::perk_has(&sim.state, "yellow")
        || wld("caves_enabled") <= 0.0
    {
        return;
    }
    // game_get_speed argument is DAT_141379b30 (never written -> 0 = gamespeed_fps)
    let fps = 1f64.max(60.0);
    let pf = sim.state.state.play_frames as f64;
    if pf < wld("caves_yellow_after_minutes") * 60.0 * fps {
        return;
    }
    if gml::gmod(pf, fps * 5.0) == 0.0 {
        if let Some(spot) = cave_yellow_pick(sim) {
            set_cave_yellow(sim, Some(spot));
            world_cave_rects(&mut sim.world);
        }
    }
}

/// `load_game`'s cave part: `global.cave_yellow` from the save (a struct, else undefined).
/// The other cave globals start empty in the rebuilt `WorldRt`.
pub fn on_load(sim: &mut Sim) {
    sim.world.caves.yellow = sim
        .state
        .state
        .cave_yellow
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());
}

// ---------------------------------------------------------------------------------------------
// rewards

/// `water_touch_sound(x, y)` (Task 20 owns the cooldown): the touch sound at the point.
pub(crate) fn water_touch_sound(sim: &mut Sim, x: f64, y: f64) {
    let mut cue = SoundCue::new("water_touch").at(x, y);
    cue.gain = vis("water_touch_volume");
    sim.emit(SimEvent::Sound(cue));
}

// port: cave_taken
/// `cave_taken(c)`: the cave with this id was taken (`state.cave_taken`).
pub fn cave_taken(state: &GameState, id: &str) -> bool {
    gml_truthy(state.state.cave_taken.get(id))
}

/// Touch test shared by caves and relics: the mask at the point, or at 8 directions at `pad`
/// and `pad / 2` around it.
pub(crate) fn touch_hits(solid: impl Fn(f64, f64) -> bool, x: f64, y: f64, pad: f64) -> bool {
    let mut hit = solid(x, y);
    let mut a = 0.0;
    while a < 360.0 && !hit {
        if solid(x + gml::lengthdir_x(pad, a), y + gml::lengthdir_y(pad, a)) {
            hit = true;
        } else {
            hit = solid(
                x + gml::lengthdir_x(pad * 0.5, a),
                y + gml::lengthdir_y(pad * 0.5, a),
            );
        }
        a += 45.0;
    }
    hit
}

// port: caves_touch_at
/// `caves_touch_at(x, y)` (a growth tip reached x, y): take every untaken cave whose wall is
/// within `cave_touch_px`. Returns the ids of the caves taken.
pub fn caves_touch_at(sim: &mut Sim, x: f64, y: f64) -> Vec<String> {
    let rects = sim.world.cave_rects().to_vec();
    let pad = vis("cave_touch_px");
    let mut taken = Vec::new();
    for r in &rects {
        if cave_taken(&sim.state, &r.id) {
            continue;
        }
        if x < r.x1 - pad || x > r.x2 + pad || y < r.y1 - pad || y > r.y2 + pad {
            continue;
        }
        if touch_hits(|px, py| r.solid_at(px, py), x, y, pad) {
            cave_take(sim, r, Some(x), Some(y));
            taken.push(r.id.clone());
        }
    }
    taken
}

// port: cave_take
/// `cave_take(r, x, y)`: the cave's perk. The first yellow perk cuts the mushroom timers, the
/// first purple turns the mycelium glow on, blue queues a node burst; flash, wrap (from the
/// touch point, else the top middle), popup and the touch sound.
pub fn cave_take(sim: &mut Sim, r: &Cave, x: Option<f64>, y: Option<f64>) {
    if cave_taken(&sim.state, &r.id) {
        return;
    }
    let id = r.kind.id();
    let first = !super::perks::perk_has(&sim.state, id);
    let st = &mut sim.state.state;
    st.cave_taken.insert(r.id.clone(), true.into());
    st.cave_perks.insert(id.to_string(), true.into());
    if r.kind == CaveKind::Yellow && first {
        let k = 1.0 - sim.bal("perk_yellow_time_cut");
        for m in &mut sim.state.mushrooms {
            m.timer *= k;
        }
    }
    let now = sim.now_ms();
    let c = &mut sim.world.caves;
    c.flash.insert(r.id.clone(), now);
    let fx = x.unwrap_or((r.x1 + r.x2) * 0.5);
    let fy = y.unwrap_or(r.y1);
    c.wrap.insert(
        r.id.clone(),
        CaveWrap {
            born: now,
            from_x: (fx - r.x1) / r.k,
            from_y: (fy - r.y1) / r.k,
        },
    );
    c.popups.push(CavePopup {
        text: format!("perk_{id}"),
        x: (r.x1 + r.x2) * 0.5,
        y: r.y1,
        born: now,
    });
    if r.kind == CaveKind::Blue {
        c.burst_pending = Some(((r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5));
    }
    if r.kind == CaveKind::Purple && first {
        crate::mycelium::myc_glow_set(sim, true);
    }
    let sx = x.unwrap_or((r.x1 + r.x2) * 0.5);
    let sy = y.unwrap_or((r.y1 + r.y2) * 0.5);
    water_touch_sound(sim, sx, sy);
}

/// Result of [`caves_wrap_hit`] (`___struct___337__caves_wrap_hit`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WrapHit {
    pub hit: bool,
    pub x: f64,
    pub y: f64,
    pub dist: f64,
}

// port: caves_wrap_hit
/// `caves_wrap_hit(x, y, maxd)`: the nearest point (within `maxd`, outside cave walls) on the
/// outline or branches of a taken cave whose wrap animation has finished.
pub fn caves_wrap_hit(sim: &mut Sim, x: f64, y: f64, maxd: f64) -> WrapHit {
    let mut best = WrapHit {
        hit: false,
        x,
        y,
        dist: maxd,
    };
    let rects = sim.world.cave_rects().to_vec();
    let pad = maxd + vis("cave_branch_max_px") + 8.0;
    let now = sim.now_ms();
    for r in &rects {
        if !(cave_taken(&sim.state, &r.id)
            && !(x < r.x1 - pad)
            && !(x > r.x2 + pad)
            && !(y < r.y1 - pad)
            && !(y > r.y2 + pad))
        {
            continue;
        }
        if let Some(wr) = sim.world.caves.wrap.get(&r.id) {
            if now - wr.born < vis("cave_wrap_ms") {
                continue;
            }
        }
        let m = cave_mask(r.kind.id()).unwrap();
        let lines = std::iter::once(&m.outline).chain(m.branches.iter().map(|b| &b.points));
        for (k, pts) in lines.enumerate() {
            let n = pts.len();
            let segs = if k == 0 { n } else { n.saturating_sub(1) };
            for j in 0..segs {
                let a = pts[j];
                let c = pts[(j + 1) % n];
                let ax = r.x1 + a[0] * r.k;
                let ay = r.y1 + a[1] * r.k;
                let dx = (c[0] - a[0]) * r.k;
                let dy = (c[1] - a[1]) * r.k;
                let l2 = dx * dx + dy * dy;
                let t = if l2 > 0.0 {
                    gml::clamp(((x - ax) * dx + (y - ay) * dy) / l2, 0.0, 1.0)
                } else {
                    0.0
                };
                let px = ax + dx * t;
                let py = ay + dy * t;
                let d = gml::point_distance(x, y, px, py);
                if d <= best.dist && !world_cave_solid_any(&mut sim.world, px, py) {
                    best = WrapHit {
                        hit: true,
                        x: px,
                        y: py,
                        dist: d,
                    };
                }
            }
        }
    }
    best
}

// port: cave_hover_tip
/// `cave_hover_tip(mx, my, ix, iy, vx, vy)`: the description key (`perk_<id>_desc`) of the perk
/// icon (HUD row at ix, iy) or taken cave wall under the GUI point, else "".
pub fn cave_hover_tip(
    sim: &mut Sim,
    mx: f64,
    my: f64,
    ix: f64,
    iy: f64,
    vx: f64,
    vy: f64,
) -> String {
    let px = vis("cave_icon_px");
    let gap = vis("cave_icon_gap_px");
    let mut x = ix;
    for d in cave_defs() {
        if super::perks::perk_has(&sim.state, d.kind.id()) {
            if mx >= x && mx < x + px && my >= iy && my < iy + px {
                return format!("perk_{}_desc", d.kind.id());
            }
            x += px + gap;
        }
    }
    let (wx, wy) = (mx + vx, my + vy);
    let rects = sim.world.cave_rects().to_vec();
    for r in &rects {
        if cave_taken(&sim.state, &r.id)
            && wx >= r.x1
            && wx < r.x2
            && wy >= r.y1
            && wy < r.y2
            && r.solid_at(wx, wy)
        {
            return format!("perk_{}_desc", r.kind.id());
        }
    }
    String::new()
}

// port: perk_debug_next
/// `perk_debug_next()` (debug key): take the first untaken cave of the first missing perk
/// (cave_defs order, then any blue); "cave <id>" or "all caves taken".
pub fn perk_debug_next(sim: &mut Sim) -> String {
    let defs = cave_defs();
    let rects = sim.world.cave_rects().to_vec();
    for i in 0..=defs.len() {
        let id = if i >= defs.len() {
            "blue"
        } else {
            defs[i].kind.id()
        };
        if i < defs.len() && super::perks::perk_has(&sim.state, id) {
            continue;
        }
        if let Some(r) = rects
            .iter()
            .find(|r| r.kind.id() == id && !cave_taken(&sim.state, &r.id))
        {
            cave_take(sim, r, None, None);
            return format!("cave {}", r.id);
        }
    }
    "all caves taken".to_string()
}

