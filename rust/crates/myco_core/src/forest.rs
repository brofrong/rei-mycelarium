//! Forest state (obj_forest_controller; rulings R5/R10): surface chunks with trees, small
//! trees, bushes, plants, decor trees and stumps; their generation (on `sim.rng`, so not
//! seed-reproducible, like the original); host queries for mushrooms; the camera step; the
//! backdrop leaves; the foreground-leaf frame queries. Drawing is Task 16's.
//!
//! Trees and stumps are mushroom hosts. Each has a runtime `uid`; mushrooms (Task 9) refer to
//! hosts by `HostId`. A save names hosts by the save ids `save_game` hands out (see
//! [`save_ids`]); after a load [`ForestRt::loaded_ids`] maps those ids to hosts.

use crate::balance::{vis, wld};
use crate::camera;
use crate::gml;
use crate::rng::Rng;
use crate::sim::{Key, Sim, StepInput};
use crate::state::{
    ForestChunkSave, GameState, PartSave, RootsRect, SmallTreeSave, StumpRoots, StumpSave, TreeSave,
};
use crate::weather;
use crate::world::art::{art, LeafFgFrame};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

pub const TREE_SPRITES: [&str; 3] = ["tree01", "tree02", "tree03"];
pub const SMALL_TREE_SPRITES: [&str; 3] = ["treesmall01", "treesmall02", "treesmall03"];

/// A cut-out of a surface sprite sheet with its pick weight (bushes, plants, decor trees).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub sprite: &'static str,
    pub left: f64,
    pub top: f64,
    pub w: f64,
    pub h: f64,
    pub weight: f64,
}

const fn part(sprite: &'static str, left: f64, top: f64, w: f64, h: f64, weight: f64) -> Part {
    Part {
        sprite,
        left,
        top,
        w,
        h,
        weight,
    }
}

/// `bush_parts` (sheet `bush_tree`).
pub fn bush_parts() -> &'static [Part] {
    const P: [Part; 7] = [
        part("bush_tree", 6.0, 39.0, 187.0, 35.0, 3.0),
        part("bush_tree", 208.0, 48.0, 220.0, 26.0, 3.0),
        part("bush_tree", 441.0, 25.0, 347.0, 49.0, 2.0),
        part("bush_tree", 0.0, 99.0, 713.0, 49.0, 1.0),
        part("bush_tree", 16.0, 155.0, 123.0, 67.0, 3.0),
        part("bush_tree", 166.0, 148.0, 132.0, 74.0, 3.0),
        part("bush_tree", 343.0, 160.0, 92.0, 62.0, 3.0),
    ];
    &P
}

/// `plant_parts` (sheet `grass`).
pub fn plant_parts() -> &'static [Part] {
    const P: [Part; 16] = [
        part("grass", 121.0, 6.0, 11.0, 28.0, 3.0),
        part("grass", 121.0, 43.0, 11.0, 28.0, 3.0),
        part("grass", 121.0, 80.0, 11.0, 28.0, 3.0),
        part("grass", 142.0, 2.0, 15.0, 33.0, 3.0),
        part("grass", 142.0, 39.0, 15.0, 33.0, 3.0),
        part("grass", 142.0, 76.0, 15.0, 33.0, 3.0),
        part("grass", 172.0, 4.0, 9.0, 31.0, 3.0),
        part("grass", 172.0, 41.0, 9.0, 31.0, 3.0),
        part("grass", 172.0, 78.0, 9.0, 31.0, 3.0),
        part("grass", 190.0, 22.0, 15.0, 13.0, 3.0),
        part("grass", 190.0, 59.0, 15.0, 13.0, 3.0),
        part("grass", 190.0, 96.0, 15.0, 13.0, 3.0),
        part("grass", 263.0, 56.0, 21.0, 53.0, 1.0),
        part("grass", 335.0, 0.0, 27.0, 109.0, 1.0),
        part("grass", 407.0, 56.0, 21.0, 53.0, 1.0),
        part("grass", 479.0, 0.0, 27.0, 109.0, 1.0),
    ];
    &P
}

/// `decor_tree_parts` (sheets `trees`, `Trees02`).
pub fn decor_tree_parts() -> &'static [Part] {
    const P: [Part; 10] = [
        part("trees", 273.0, 122.0, 136.0, 245.0, 2.0),
        part("trees", 458.0, 148.0, 113.0, 219.0, 2.0),
        part("trees", 632.0, 215.0, 51.0, 152.0, 2.0),
        part("trees", 715.0, 215.0, 51.0, 152.0, 2.0),
        part("Trees02", 73.0, 7.0, 203.0, 485.0, 1.0),
        part("Trees02", 332.0, 142.0, 181.0, 350.0, 1.0),
        part("Trees02", 524.0, 246.0, 136.0, 246.0, 2.0),
        part("Trees02", 688.0, 270.0, 113.0, 222.0, 2.0),
        part("Trees02", 819.0, 295.0, 136.0, 197.0, 2.0),
        part("Trees02", 962.0, 284.0, 113.0, 208.0, 2.0),
    ];
    &P
}

/// One root cut-out of the `roots` sheet: [left, top, w, h, ax].
type RootPart = [f64; 5];
const ROOTS_WIDE: [RootPart; 2] = [
    [16.0, 7.0, 348.0, 212.0, 183.0],
    [44.0, 256.0, 276.0, 239.0, 152.0],
];
const ROOTS_MEDIUM: [RootPart; 3] = [
    [448.0, 8.0, 211.0, 189.0, 110.0],
    [448.0, 247.0, 202.0, 213.0, 112.0],
    [452.0, 501.0, 195.0, 239.0, 112.0],
];
const ROOTS_SMALL: [RootPart; 2] = [
    [778.0, 13.0, 106.0, 112.0, 53.0],
    [771.0, 191.0, 113.0, 118.0, 57.0],
];

fn roots_rect(r: &RootPart) -> RootsRect {
    RootsRect {
        left: r[0],
        top: r[1],
        w: r[2],
        h: r[3],
        ax: r[4],
        extra: Default::default(),
    }
}

/// A place on a stump where a mushroom can sit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StumpSlot {
    pub x: f64,
    pub y: f64,
    pub face: f64,
}

const fn slot(x: f64, y: f64, face: f64) -> StumpSlot {
    StumpSlot { x, y, face }
}

/// One stump of the `stump` sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StumpPart {
    pub left: f64,
    pub w: f64,
    /// x of the roots anchor; `None` for the log, which has no roots
    pub root_ax: Option<f64>,
    /// what consuming it is called (`stump`, `fallen`, `log`)
    pub consume: &'static str,
    pub slots: &'static [StumpSlot],
}

/// `stump_parts`.
pub fn stump_parts() -> &'static [StumpPart] {
    const S0: [StumpSlot; 1] = [slot(111.0, 122.0, 1.0)];
    const S1: [StumpSlot; 1] = [slot(109.0, 122.0, 1.0)];
    const S2: [StumpSlot; 3] = [
        slot(70.0, 140.0, -1.0),
        slot(160.0, 112.0, 1.0),
        slot(323.0, 135.0, 1.0),
    ];
    const S3: [StumpSlot; 4] = [
        slot(160.0, 121.0, -1.0),
        slot(230.0, 124.0, 1.0),
        slot(380.0, 130.0, -1.0),
        slot(425.0, 132.0, 1.0),
    ];
    const P: [StumpPart; 4] = [
        StumpPart {
            left: 4.0,
            w: 150.0,
            root_ax: Some(75.0),
            consume: "stump",
            slots: &S0,
        },
        StumpPart {
            left: 199.0,
            w: 149.0,
            root_ax: Some(75.0),
            consume: "stump",
            slots: &S1,
        },
        StumpPart {
            left: 387.0,
            w: 361.0,
            root_ax: Some(290.0),
            consume: "fallen",
            slots: &S2,
        },
        StumpPart {
            left: 786.0,
            w: 474.0,
            root_ax: None,
            consume: "log",
            slots: &S3,
        },
    ];
    &P
}

/// A big tree (mushroom host).
#[derive(Clone, Debug, PartialEq)]
pub struct TreeRt {
    pub uid: u64,
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<RootsRect>,
    /// consumption progress (saved; undefined = 0 for a fresh tree)
    pub consume: f64,
    /// fully consumed: no longer a host (`consumed`, set by Task 9)
    pub consumed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SmallTreeRt {
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<RootsRect>,
}

/// A bush, plant or decor tree: index into its part list.
#[derive(Clone, Debug, PartialEq)]
pub struct PartRt {
    pub part: i64,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
}

/// A stump (mushroom host).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StumpRt {
    pub uid: u64,
    /// index into [`stump_parts`]
    pub part: usize,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub roots: Option<StumpRoots>,
    pub consume: f64,
    pub consumed: bool,
}

/// One surface chunk (`surface_chunks["s<cx>"]`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ForestChunk {
    pub trees: Vec<TreeRt>,
    pub small_trees: Vec<SmallTreeRt>,
    pub bushes: Vec<PartRt>,
    pub plants: Vec<PartRt>,
    pub decor_trees: Vec<PartRt>,
    pub stumps: Vec<StumpRt>,
}

/// A mushroom host: a tree or a stump, by uid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HostId {
    Tree(u64),
    Stump(u64),
}

/// One leaf drifting in the backdrop (`___struct___107`).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BackdropLeaf {
    pub lx: f64,
    pub y: f64,
    pub fall: f64,
    pub drift: f64,
    pub phase: f64,
    pub angle: f64,
    pub hold_angle: f64,
    pub face: f64,
    pub change_from: f64,
    pub change_frame: f64,
    pub change_timer: f64,
    pub scale: f64,
    pub alpha: f64,
}

/// Runtime forest (`obj_forest_controller` variables; the camera itself is `sim.camera`).
#[derive(Clone, Debug, Default)]
pub struct ForestRt {
    pub surface_height: f64,
    pub chunk_width: f64,
    pub camera_speed: f64,
    pub camera_edge_margin: f64,
    /// `global.start_x` when the forest was made
    pub start_x: f64,
    pub chunks: HashMap<i64, ForestChunk>,
    /// the stump near the start (also inside its chunk once that chunk exists)
    pub start_stump: StumpRt,
    /// `start_stumps` is `[start_stump]` (false after loading a save without one)
    pub has_start_stump: bool,
    /// save id -> host, from the last load (Task 9 resolves `MushroomSave::host` with it)
    pub loaded_ids: HashMap<i64, HostId>,
    /// `global.backdrop_leaves` (two bands, far and near), made on the first step
    pub backdrop_leaves: Option<[Vec<BackdropLeaf>; 2]>,
    next_uid: u64,
    /// x of hosts taken out by `remove_host` (they keep their position, as structs do in GML)
    removed_x: HashMap<u64, f64>,
}

fn originshroom_w() -> f64 {
    art().sprites.originshroom[0]
}

// port: anon__10580__gml_Object_obj_forest_controller_Create_0
/// `tree_foot(sprite)`: half-width of the ground a tree occupies.
pub fn tree_foot(sprite: &str) -> f64 {
    match sprite {
        "tree01" | "tree02" | "tree03" => 80.0,
        "treesmall03" => 40.0,
        _ => 20.0,
    }
}

// port: anon__7254__gml_Object_obj_forest_controller_Create_0
/// `tree_roots_pick(sprite)`: a random root cut-out for the tree size (none for small trees 1-2).
pub fn tree_roots_pick(rng: &mut Rng, sprite: &str) -> Option<RootsRect> {
    let list: &[RootPart] = match sprite {
        "tree01" | "tree02" => &ROOTS_WIDE,
        "tree03" => &ROOTS_MEDIUM,
        "treesmall03" => &ROOTS_SMALL,
        _ => return None,
    };
    Some(roots_rect(
        &list[rng.irandom((list.len() - 1) as f64) as usize],
    ))
}

// port: anon__5146__gml_Object_obj_forest_controller_Create_0
/// `part_pick(list)`: weighted random index.
pub fn part_pick(rng: &mut Rng, list: &[Part]) -> usize {
    let total: f64 = list.iter().map(|p| p.weight).sum();
    let mut r = rng.random(total);
    for (i, p) in list.iter().enumerate() {
        r -= p.weight;
        if r < 0.0 {
            return i;
        }
    }
    0
}

// port: anon__9461__gml_Object_obj_forest_controller_Create_0
/// `stump_slot_at(stump, i)`: world position and facing of a stump's mushroom slot.
pub fn stump_slot_at(d: &StumpRt, i: usize) -> StumpSlot {
    let part = &stump_parts()[d.part];
    let s = part.slots[i];
    StumpSlot {
        x: d.x + (s.x - part.w * 0.5) * d.flip,
        y: d.y - art().sprites.stump[1] + s.y + vis("stump_offset_y"),
        face: s.face * d.flip,
    }
}

// port: anon__10906__gml_Object_obj_forest_controller_Create_0
/// `surface_near_stump(list, x, hw)`: something `hw` wide at x would crowd one of the stumps.
pub fn surface_near_stump(list: &[StumpRt], x: f64, hw: f64) -> bool {
    let gap = wld("stump_tree_gap_px");
    list.iter()
        .any(|t| (t.x - x).abs() < stump_parts()[t.part].w * 0.5 + hw + gap)
}

impl ForestRt {
    fn uid(&mut self) -> u64 {
        self.next_uid += 1;
        self.next_uid
    }

    /// `start_stumps`: the start stump, unless a loaded save had none.
    pub fn start_stumps(&self) -> Vec<StumpRt> {
        if self.has_start_stump {
            vec![self.start_stump.clone()]
        } else {
            Vec::new()
        }
    }

    pub fn tree(&self, uid: u64) -> Option<&TreeRt> {
        self.chunks
            .values()
            .flat_map(|c| &c.trees)
            .find(|t| t.uid == uid)
    }

    pub fn tree_mut(&mut self, uid: u64) -> Option<&mut TreeRt> {
        self.chunks
            .values_mut()
            .flat_map(|c| &mut c.trees)
            .find(|t| t.uid == uid)
    }

    pub fn stump(&self, uid: u64) -> Option<&StumpRt> {
        self.chunks
            .values()
            .flat_map(|c| &c.stumps)
            .find(|t| t.uid == uid)
    }

    pub fn stump_mut(&mut self, uid: u64) -> Option<&mut StumpRt> {
        self.chunks
            .values_mut()
            .flat_map(|c| &mut c.stumps)
            .find(|t| t.uid == uid)
    }

    /// x of a host still standing in some chunk.
    pub fn host_x(&self, id: HostId) -> Option<f64> {
        match id {
            HostId::Tree(u) => self.tree(u).map(|t| t.x),
            HostId::Stump(u) => self.stump(u).map(|s| s.x),
        }
    }

    // port: anon__1944__gml_Object_obj_forest_controller_Create_0
    /// `bush_pick()`: weighted random bush index.
    pub fn bush_pick(&self, rng: &mut Rng) -> usize {
        part_pick(rng, bush_parts())
    }

    // port: anon__9873__gml_Object_obj_forest_controller_Create_0
    /// `stump_make(part, x)`: a stump on the surface, with random roots unless it is the log.
    pub fn stump_make(&mut self, rng: &mut Rng, part: usize, x: f64) -> StumpRt {
        let p = &stump_parts()[part];
        let flip = 1.0;
        let roots = p.root_ax.map(|root_ax| {
            let pool: &[RootPart] = if rng.choose(&[0, 1]) == 0 {
                &ROOTS_WIDE
            } else {
                &ROOTS_MEDIUM
            };
            let rp = &pool[rng.irandom((pool.len() - 1) as f64) as usize];
            StumpRoots {
                x: x + (root_ax - p.w * 0.5) * flip,
                y: self.surface_height,
                flip,
                roots: Some(roots_rect(rp)),
                extra: Default::default(),
            }
        });
        StumpRt {
            uid: self.uid(),
            part,
            x,
            y: self.surface_height,
            flip,
            roots,
            consume: 0.0,
            consumed: false,
        }
    }

    // port: anon__12392__gml_Object_obj_forest_controller_Create_0
    /// `surface_in_start_clearing(x)`: no trees or plants right around the origin mushroom.
    pub fn surface_in_start_clearing(&self, x: f64) -> bool {
        (x - self.start_x).abs() <= originshroom_w() * 0.5 + wld("start_tree_clear_px")
    }

    fn crowded(&self, stumps: &[StumpRt], x: f64, hw: f64) -> bool {
        self.surface_in_start_clearing(x)
            || surface_near_stump(stumps, x, hw)
            || surface_near_stump(&self.start_stumps(), x, hw)
    }

    // port: anon__12537__gml_Object_obj_forest_controller_Create_0
    /// `surface_get_chunk(cx)`: the surface chunk, generated on first use with `rng`.
    pub fn surface_get_chunk(&mut self, rng: &mut Rng, cx: i64) -> &ForestChunk {
        if !self.chunks.contains_key(&cx) {
            let ch = self.generate_chunk(rng, cx);
            self.chunks.insert(cx, ch);
        }
        &self.chunks[&cx]
    }

    fn generate_chunk(&mut self, rng: &mut Rng, cx: i64) -> ForestChunk {
        let cw = self.chunk_width;
        let x0 = cx as f64 * cw;
        let sy = self.surface_height;
        let mut ch = ForestChunk::default();
        let n = TREE_SPRITES.len() as i64;
        let start_part_w = stump_parts()[self.start_stump.part].w;
        if self.start_stump.x >= x0 && self.start_stump.x < x0 + cw {
            ch.stumps.push(self.start_stump.clone());
        } else if rng.random(1.0) < wld("stump_chunk_chance") {
            let part = rng.irandom((stump_parts().len() - 1) as f64) as usize;
            let hw = stump_parts()[part].w * 0.5;
            let margin = tree_foot("tree01") + wld("stump_tree_gap_px") + hw;
            let sx = rng.random_range(x0 + margin, x0 + cw - margin).floor();
            let clear = originshroom_w() * 0.5 + wld("start_tree_clear_px") + hw;
            let gap =
                start_part_w * 0.5 + hw + (tree_foot("tree01") + wld("stump_tree_gap_px")) * 2.0;
            if margin * 2.0 < cw
                && (sx - self.start_x).abs() > clear
                && (sx - self.start_stump.x).abs() > gap
            {
                let s = self.stump_make(rng, part, sx);
                ch.stumps.push(s);
            }
        }
        let mut last = -1;
        let mut x = x0 + rng.irandom_range(0.0, 320.0) as f64;
        while x < x0 + cw {
            let mut idx = rng.irandom((n - 1) as f64);
            if idx == last && n > 1 {
                idx = (idx + 1 + rng.irandom((n - 2) as f64)) % n;
            }
            let spr = TREE_SPRITES[idx as usize];
            if !self.crowded(&ch.stumps, x, tree_foot(spr)) {
                let roots = tree_roots_pick(rng, spr);
                let uid = self.uid();
                ch.trees.push(TreeRt {
                    uid,
                    sprite: spr.into(),
                    x,
                    y: sy,
                    flip: 1.0,
                    roots,
                    consume: 0.0,
                    consumed: false,
                });
                last = idx;
            }
            x += rng.irandom_range(420.0, 900.0) as f64;
        }
        x = x0 + rng.irandom_range(0.0, 160.0) as f64;
        while x < x0 + cw {
            let spr =
                SMALL_TREE_SPRITES[rng.irandom((SMALL_TREE_SPRITES.len() - 1) as f64) as usize];
            if rng.random(1.0) < 0.68 && !self.crowded(&ch.stumps, x, tree_foot(spr)) {
                let roots = tree_roots_pick(rng, spr);
                ch.small_trees.push(SmallTreeRt {
                    sprite: spr.into(),
                    x,
                    y: sy,
                    flip: 1.0,
                    roots,
                });
            }
            x += rng.irandom_range(160.0, 520.0) as f64;
        }
        x = x0 + rng.irandom_range(0.0, wld("bush_spacing_max_px")) as f64;
        while x < x0 + cw {
            let b = self.bush_pick(rng);
            let hw = bush_parts()[b].w * 0.5;
            if rng.random(1.0) < wld("bush_chance") && !self.crowded(&ch.stumps, x, hw) {
                ch.bushes.push(PartRt {
                    part: b as i64,
                    x,
                    y: sy,
                    flip: 1.0,
                });
            }
            x += rng.irandom_range(wld("bush_spacing_min_px"), wld("bush_spacing_max_px")) as f64;
        }
        x = x0 + rng.irandom_range(0.0, wld("plant_spacing_max_px")) as f64;
        while x < x0 + cw {
            let p = part_pick(rng, plant_parts());
            let hw = plant_parts()[p].w * 0.5;
            if rng.random(1.0) < wld("plant_chance") && !self.crowded(&ch.stumps, x, hw) {
                ch.plants.push(PartRt {
                    part: p as i64,
                    x,
                    y: sy,
                    flip: 1.0,
                });
            }
            x += rng.irandom_range(wld("plant_spacing_min_px"), wld("plant_spacing_max_px")) as f64;
        }
        x = x0 + rng.irandom_range(0.0, wld("decor_tree_spacing_max_px")) as f64;
        while x < x0 + cw {
            let p = part_pick(rng, decor_tree_parts());
            let hw = decor_tree_parts()[p].w * 0.5;
            if rng.random(1.0) < wld("decor_tree_chance") && !self.crowded(&ch.stumps, x, hw) {
                ch.decor_trees.push(PartRt {
                    part: p as i64,
                    x,
                    y: sy,
                    flip: 1.0,
                });
            }
            x += rng.irandom_range(
                wld("decor_tree_spacing_min_px"),
                wld("decor_tree_spacing_max_px"),
            ) as f64;
        }
        ch
    }

    // port: anon__18383__gml_Object_obj_forest_controller_Create_0
    /// `forest_find_host_near(x, r)`: the nearest unconsumed tree or stump within `r` (stumps
    /// measured from their edge), trees first on ties.
    pub fn find_host_near(&mut self, rng: &mut Rng, x: f64, r: f64) -> Option<HostId> {
        let cw = self.chunk_width;
        let c0 = ((x - r) / cw).floor() as i64 - 1;
        let c1 = ((x + r) / cw).floor() as i64 + 1;
        let mut best = None;
        let mut bestd = r;
        for c in c0..=c1 {
            let ch = self.surface_get_chunk(rng, c);
            for t in &ch.trees {
                let dist = ((t.x - x).abs()).max(0.0);
                if dist <= bestd && !t.consumed {
                    bestd = dist;
                    best = Some(HostId::Tree(t.uid));
                }
            }
            for s in &ch.stumps {
                let hw = stump_parts()[s.part].w * 0.5;
                let dist = ((s.x - x).abs() - hw).max(0.0);
                if dist <= bestd && !s.consumed {
                    bestd = dist;
                    best = Some(HostId::Stump(s.uid));
                }
            }
        }
        best
    }

    // port: anon__19468__gml_Object_obj_forest_controller_Create_0
    /// `forest_remove_host(t)`: take a host out of its chunk (searching the chunks around it).
    /// A host that was already removed still has its x (the original keeps the struct), so
    /// the chunks around it are still visited (and generated, which may draw from `rng`).
    pub fn remove_host(&mut self, rng: &mut Rng, id: HostId) {
        let uid = match id {
            HostId::Tree(u) | HostId::Stump(u) => u,
        };
        let Some(x) = self
            .host_x(id)
            .or_else(|| self.removed_x.get(&uid).copied())
        else {
            return;
        };
        let cw = self.chunk_width;
        let c0 = (x / cw).floor() as i64 - 1;
        for c in c0..=c0 + 2 {
            self.surface_get_chunk(rng, c);
            let ch = self.chunks.get_mut(&c).unwrap();
            match id {
                HostId::Tree(u) => {
                    if let Some(i) = ch.trees.iter().rposition(|t| t.uid == u) {
                        ch.trees.remove(i);
                        self.removed_x.insert(u, x);
                        return;
                    }
                }
                HostId::Stump(u) => {
                    if let Some(i) = ch.stumps.iter().rposition(|t| t.uid == u) {
                        ch.stumps.remove(i);
                        self.removed_x.insert(u, x);
                        return;
                    }
                }
            }
        }
    }

    // port: anon__20014__gml_Object_obj_forest_controller_Create_0
    /// `forest_find_tree_near(x, r)`: the nearest big tree within `r` (consumed or not).
    pub fn find_tree_near(&mut self, rng: &mut Rng, x: f64, r: f64) -> Option<u64> {
        let cw = self.chunk_width;
        let c0 = ((x - r) / cw).floor() as i64;
        let c1 = ((x + r) / cw).floor() as i64;
        let mut best = None;
        let mut bestd = r;
        for c in c0..=c1 {
            for t in &self.surface_get_chunk(rng, c).trees {
                let d = (t.x - x).abs();
                if d <= bestd {
                    bestd = d;
                    best = Some(t.uid);
                }
            }
        }
        best
    }
}

// port: gml_Object_obj_forest_controller_Create_0
/// obj_forest_controller Create_0: camera at the start, the start stump next to the origin.
pub fn create(sim: &mut Sim) {
    let start_x = sim.state.state.start_x;
    let f = &mut sim.forest;
    f.surface_height = wld("surface_height_px");
    f.chunk_width = sim.world.params.chunk_w as f64;
    f.camera_speed = vis("camera_speed");
    f.camera_edge_margin = vis("camera_edge_margin_px");
    f.start_x = start_x;
    let cam = &mut sim.camera;
    cam.x = start_x - cam.width * 0.5;
    cam.y = 0.0;
    cam.vx = 0.0;
    cam.vy = 0.0;
    cam.view_x = cam.x.floor();
    cam.view_y = cam.y.floor();
    let rng = &mut sim.rng;
    let sp = rng.irandom((stump_parts().len() - 1) as f64) as usize;
    let hw = stump_parts()[sp].w * 0.5;
    let lo = originshroom_w() * 0.5 + wld("start_tree_clear_px") + hw + 40.0;
    let hi = lo.max(vis("screen_width_px") * 0.5 - hw - 40.0);
    let side = rng.choose(&[-1.0, 1.0]);
    let off = rng.random_range(lo, hi);
    let start = f.stump_make(rng, sp, (start_x + side * off).floor());
    f.start_stump = start;
    f.has_start_stump = true;
    f.chunks.clear();
}

/// `ui_blocking()`: menu, debug and logo state come from the app through `StepInput`.
pub(crate) fn ui_blocking(sim: &Sim, input: &StepInput) -> bool {
    input.menu_open
        || input.debug_open
        || sim.pause_open
        || input.logo_open
        || sim.intro_active
        || sim.title_open
        || input.tasks_popup_open
        || camera::cine_blocking(sim)
}

// port: gml_Object_obj_forest_controller_Step_0
/// obj_forest_controller Step_0: camera from keys and screen edges (eased), fly-to-marker,
/// shake; then weather, backdrop leaves and moving stones.
pub fn step(sim: &mut Sim, input: &StepInput) {
    if input.view.w > 0.0
        && input.view.h > 0.0
        && (sim.camera.width != input.view.w || sim.camera.height != input.view.h)
    {
        camera::forest_fit_view(sim, input.view.w, input.view.h);
    }
    let down = |k: Key| input.keys_down.contains(&k) as i64 as f64;
    let mut dx = down(Key::Right) - down(Key::Left);
    dx += down(Key::Char('D')) - down(Key::Char('A'));
    let mut dy = down(Key::Down) - down(Key::Up);
    dy += down(Key::Char('S')) - down(Key::Char('W'));
    let key_move = dx != 0.0 || dy != 0.0;
    let blocking = ui_blocking(sim, input);
    let (cw, ch) = (sim.camera.width, sim.camera.height);
    if !blocking {
        if let Some((mx, my)) = input.cursor_gui {
            // display_get_gui_width/height: the GUI layer is view-sized (ruling R17)
            let (gw, gh) = (input.view.w, input.view.h);
            let margin = sim.forest.camera_edge_margin;
            if mx >= 0.0 && mx < margin {
                dx -= 1.0;
            }
            if mx <= gw - 1.0 && mx > gw - margin {
                dx += 1.0;
            }
            if my >= 0.0 && my < margin {
                dy -= 1.0;
            }
            if my <= gh - 1.0 && my > gh - margin {
                dy += 1.0;
            }
        }
    }
    if sim.title_open || camera::cine_showing(sim) {
        dx = 0.0;
        dy = 0.0;
    }
    dx = gml::clamp(dx, -1.0, 1.0);
    dy = gml::clamp(dy, -1.0, 1.0);
    if key_move && input.keys_down.contains(&Key::Shift) {
        dx *= vis("camera_shift_boost");
        dy *= vis("camera_shift_boost");
    }
    let b = camera::camera_bounds(sim, cw, ch);
    let accel = gml::clamp(vis("camera_accel"), 0.01, 1.0);
    let speed = sim.forest.camera_speed;
    let cam = &mut sim.camera;
    let tvx = dx * speed;
    let tvy = dy * speed;
    cam.vx = if (tvx - cam.vx).abs() >= 0.25 {
        gml::lerp(cam.vx, tvx, accel)
    } else {
        tvx
    };
    cam.vy = if (tvy - cam.vy).abs() >= 0.25 {
        gml::lerp(cam.vy, tvy, accel)
    } else {
        tvy
    };
    let nx = gml::clamp(cam.x + cam.vx, b.x1, b.x2);
    let ny = gml::clamp(cam.y + cam.vy, b.y1, b.y2);
    if nx != cam.x + cam.vx {
        cam.vx = 0.0;
    }
    if ny != cam.y + cam.vy {
        cam.vy = 0.0;
    }
    cam.x = nx;
    cam.y = ny;
    if !blocking {
        if let Some(mut fly) = cam.fly.take() {
            fly.t += 1.0 / sim.fps.max(1.0);
            let tx = gml::clamp(fly.x - cw * 0.5, b.x1, b.x2);
            let ty = gml::clamp(fly.y - ch * 0.5, b.y1, b.y2);
            let ease = gml::clamp(vis("camera_fly_ease"), 0.01, 1.0);
            cam.x = gml::lerp(cam.x, tx, ease);
            cam.y = gml::lerp(cam.y, ty, ease);
            cam.vx = 0.0;
            cam.vy = 0.0;
            let done = key_move
                || fly.t >= vis("camera_fly_seconds")
                || gml::point_distance(cam.x, cam.y, tx, ty) < 2.0;
            if !done {
                cam.fly = Some(fly);
            }
        }
    }
    let cam_x = sim.camera.x;
    let blocking_now = ui_blocking(sim, input);
    let shake = camera::boar_shake(sim, cam_x, cw, blocking_now);
    let cam = &mut sim.camera;
    cam.shake = shake;
    cam.view_x = gml::clamp(cam.x + shake[0], b.x1, b.x2).floor();
    cam.view_y = gml::clamp(cam.y + shake[1], b.y1, b.y2).floor();
    let (x, y, view_x) = (cam.x, cam.y, cam.view_x);
    if !blocking {
        weather::step(sim, x, y, cw, ch);
    }
    if !input.menu_open
        && !input.debug_open
        && !sim.pause_open
        && !input.logo_open
        && !input.tasks_popup_open
        && !camera::cine_blocking(sim)
    {
        backdrop_leaves_step(sim, view_x, cw);
    }
    if !blocking {
        crate::world::stones::stones_step(sim);
    }
}

/// `load_game`'s forest part: surface chunks (hosts get fresh uids, `loaded_ids` maps the save
/// ids), the start stump, the camera.
pub fn on_load(sim: &mut Sim) {
    let saved = std::mem::take(&mut sim.state.forest.chunks);
    let f = &mut sim.forest;
    f.chunks.clear();
    f.loaded_ids.clear();
    let mut start = None;
    for (key, c) in saved {
        let Some(cx) = key.strip_prefix('s').and_then(|s| s.parse::<i64>().ok()) else {
            continue;
        };
        let mut ch = ForestChunk::default();
        for t in c.trees {
            let uid = f.uid();
            f.loaded_ids.insert(t.id, HostId::Tree(uid));
            ch.trees.push(TreeRt {
                uid,
                sprite: t.sprite,
                x: t.x,
                y: t.y,
                flip: 1.0,
                roots: t.roots,
                consume: t.consume,
                consumed: false,
            });
        }
        for t in c.small_trees {
            ch.small_trees.push(SmallTreeRt {
                sprite: t.sprite,
                x: t.x,
                y: t.y,
                flip: 1.0,
                roots: t.roots,
            });
        }
        for s in c.stumps {
            let part = (s.part.max(0) as usize).min(stump_parts().len() - 1);
            let p = &stump_parts()[part];
            let mut roots = s.roots;
            if let (Some(r), Some(root_ax)) = (roots.as_mut(), p.root_ax) {
                r.x = s.x + (root_ax - p.w * 0.5);
                r.flip = 1.0;
            }
            let uid = f.uid();
            f.loaded_ids.insert(s.id, HostId::Stump(uid));
            let sn = StumpRt {
                uid,
                part,
                x: s.x,
                y: s.y,
                flip: 1.0,
                roots,
                consume: s.consume,
                consumed: false,
            };
            if s.start {
                start = Some(sn.clone());
            }
            ch.stumps.push(sn);
        }
        let parts = |v: Vec<PartSave>| -> Vec<PartRt> {
            v.into_iter()
                .map(|p| PartRt {
                    part: p.part,
                    x: p.x,
                    y: p.y,
                    flip: 1.0,
                })
                .collect()
        };
        ch.bushes = parts(c.bushes);
        ch.plants = parts(c.plants);
        ch.decor_trees = parts(c.decor_trees);
        f.chunks.insert(cx, ch);
    }
    match start {
        Some(s) => {
            f.start_stump = s;
            f.has_start_stump = true;
        }
        None => {
            let uid = f.uid();
            f.start_stump = StumpRt {
                uid,
                part: 0,
                x: -1_000_000_000.0,
                y: f.surface_height,
                flip: 1.0,
                ..Default::default()
            };
            f.has_start_stump = false;
        }
    }
    let cam = &mut sim.camera;
    cam.x = sim.state.forest.camera_x;
    cam.y = sim.state.forest.camera_y;
    cam.vx = 0.0;
    cam.vy = 0.0;
}

fn sorted_chunks(f: &ForestRt) -> Vec<(i64, &ForestChunk)> {
    let mut v: Vec<(i64, &ForestChunk)> = f.chunks.iter().map(|(k, c)| (*k, c)).collect();
    // variable_struct_get_names order: chunk keys "s<cx>" sorted as strings
    v.sort_by_key(|(cx, _)| format!("s{cx}"));
    v
}

/// The save ids `save_game` gives hosts: per chunk (sorted keys), trees then stumps.
pub fn save_ids(f: &ForestRt) -> HashMap<u64, i64> {
    let mut ids = HashMap::new();
    let mut sid = 0;
    for (_, ch) in sorted_chunks(f) {
        for t in &ch.trees {
            ids.insert(t.uid, sid);
            sid += 1;
        }
        for s in &ch.stumps {
            ids.insert(s.uid, sid);
            sid += 1;
        }
    }
    ids
}

/// `save_game`'s forest part: surface chunks with host save ids, and the camera.
pub fn before_save(sim: &Sim, out: &mut GameState) {
    let f = &sim.forest;
    let ids = save_ids(f);
    let mut chunks = BTreeMap::new();
    for (cx, ch) in sorted_chunks(f) {
        let parts = |v: &Vec<PartRt>| -> Vec<PartSave> {
            v.iter()
                .map(|p| PartSave {
                    part: p.part,
                    x: p.x,
                    y: p.y,
                    flip: p.flip,
                    extra: Default::default(),
                })
                .collect()
        };
        let save = ForestChunkSave {
            trees: ch
                .trees
                .iter()
                .map(|t| TreeSave {
                    id: ids[&t.uid],
                    sprite: t.sprite.clone(),
                    x: t.x,
                    y: t.y,
                    flip: t.flip,
                    roots: t.roots.clone(),
                    consume: t.consume,
                    extra: Default::default(),
                })
                .collect(),
            small_trees: ch
                .small_trees
                .iter()
                .map(|t| SmallTreeSave {
                    sprite: t.sprite.clone(),
                    x: t.x,
                    y: t.y,
                    flip: t.flip,
                    roots: t.roots.clone(),
                    extra: Default::default(),
                })
                .collect(),
            bushes: parts(&ch.bushes),
            plants: parts(&ch.plants),
            decor_trees: parts(&ch.decor_trees),
            stumps: ch
                .stumps
                .iter()
                .map(|s| StumpSave {
                    id: ids[&s.uid],
                    part: s.part as i64,
                    x: s.x,
                    y: s.y,
                    flip: s.flip,
                    roots: s.roots.clone(),
                    consume: s.consume,
                    start: f.has_start_stump && s.uid == f.start_stump.uid,
                    extra: Default::default(),
                })
                .collect(),
            extra: Default::default(),
        };
        chunks.insert(format!("s{cx}"), save);
    }
    out.forest.chunks = chunks;
    out.forest.camera_x = sim.camera.x;
    out.forest.camera_y = sim.camera.y;
}

// ---------------------------------------------------------------------------------------------
// backdrop leaves (scr_life)

// port: backdrop_leaf_band_setting
/// Band 0 is the far band.
pub fn backdrop_leaf_band_setting(band: usize, near_key: &str, far_key: &str) -> f64 {
    if band == 0 {
        vis(far_key)
    } else {
        vis(near_key)
    }
}

// port: backdrop_leaf_count
pub fn backdrop_leaf_count(sim: &Sim, band: usize) -> f64 {
    let mut n =
        backdrop_leaf_band_setting(band, "backdrop_leaf_near_count", "backdrop_leaf_far_count");
    if sim.state.state.wind[1] >= vis("wind_max_strength") {
        n = gml::round(n * vis("backdrop_leaf_strong_wind_mult"));
    }
    n
}

// port: backdrop_leaf_parallax
pub fn backdrop_leaf_parallax(band: usize) -> f64 {
    backdrop_leaf_band_setting(
        band,
        "backdrop_leaf_near_parallax",
        "backdrop_leaf_far_parallax",
    )
}

// port: backdrop_leaf_make
pub fn backdrop_leaf_make(
    sim: &mut Sim,
    band: usize,
    cam_x: f64,
    w: f64,
    anywhere: bool,
) -> BackdropLeaf {
    let floor_y = wld("surface_height_px");
    let par = backdrop_leaf_parallax(band);
    let [wind_dir, wind_speed, _] = sim.state.state.wind;
    let rng = &mut sim.rng;
    let rs = rng.random_range(0.85, 1.15);
    let sc =
        backdrop_leaf_band_setting(band, "backdrop_leaf_near_scale", "backdrop_leaf_far_scale")
            * rs;
    let lx = cam_x * par + rng.random_range(-80.0, w + 80.0);
    let y = if !anywhere {
        rng.random_range(-90.0, -10.0)
    } else {
        rng.random(floor_y)
    };
    let fall = rng.random_range(vis("backdrop_leaf_fall_min"), vis("backdrop_leaf_fall_max"));
    let drift = wind_dir * wind_speed * vis("backdrop_leaf_wind_push");
    let phase = rng.random(360.0);
    let face = crate::life::leaves::leaf_face_for_wind(wind_dir);
    let ra = rng.random_range(0.85, 1.1);
    let alpha =
        backdrop_leaf_band_setting(band, "backdrop_leaf_near_alpha", "backdrop_leaf_far_alpha")
            * ra;
    BackdropLeaf {
        lx,
        y,
        fall,
        drift,
        phase,
        angle: 0.0,
        hold_angle: 0.0,
        face,
        change_from: -1.0,
        change_frame: 0.0,
        change_timer: 0.0,
        scale: sc,
        alpha,
    }
}

// port: backdrop_leaves_init
pub fn backdrop_leaves_init(sim: &mut Sim, cam_x: f64, w: f64) {
    if sim.forest.backdrop_leaves.is_some() {
        return;
    }
    let mut bands: [Vec<BackdropLeaf>; 2] = Default::default();
    for (band, arr) in bands.iter_mut().enumerate() {
        let n = backdrop_leaf_count(sim, band);
        let mut i = 0.0;
        while i < n {
            arr.push(backdrop_leaf_make(sim, band, cam_x, w, true));
            i += 1.0;
        }
    }
    sim.forest.backdrop_leaves = Some(bands);
}

// port: backdrop_leaves_step
/// Drift the backdrop leaves with the wind; leaves that land or leave the view are replaced
/// (from the windward edge) or dropped when the band has too many.
pub fn backdrop_leaves_step(sim: &mut Sim, cam_x: f64, w: f64) {
    backdrop_leaves_init(sim, cam_x, w);
    let floor_y = wld("surface_height_px");
    let [wind_dir, wind_speed, _] = sim.state.state.wind;
    let force = wind_dir * wind_speed;
    let target_face = crate::life::leaves::leaf_face_for_wind(wind_dir);
    let wind_ease = gml::clamp(vis("backdrop_leaf_wind_ease"), 0.0, 1.0);
    let spin_ease = gml::clamp(vis("leaf_spin_ease"), 0.0, 1.0);
    let push = vis("backdrop_leaf_wind_push");
    let edge = if force >= 0.0 { -100.0 } else { w + 100.0 };
    let mut bands = sim.forest.backdrop_leaves.take().unwrap();
    for (band, arr) in bands.iter_mut().enumerate() {
        let par = backdrop_leaf_parallax(band);
        let count = backdrop_leaf_count(sim, band);
        if (arr.len() as f64) < count {
            let mut nl = backdrop_leaf_make(sim, band, cam_x, w, false);
            nl.lx = cam_x * par + edge;
            nl.y = sim.rng.random(floor_y * 0.7);
            arr.push(nl);
        }
        let mut i = arr.len() as i64 - 1;
        while i >= 0 {
            let l = &mut arr[i as usize];
            if l.change_from < 0.0 && l.face != target_face {
                l.change_from = l.face;
                l.change_frame = 0.0;
                l.change_timer = vis("leaf_change_frame_interval");
                l.hold_angle = l.angle;
            }
            if l.change_from >= 0.0 {
                l.change_timer -= 1.0;
                if l.change_timer <= 0.0 {
                    l.change_timer = vis("leaf_change_frame_interval");
                    l.change_frame += 1.0;
                    if l.change_frame >= 8.0 {
                        l.face = 1.0 - l.change_from;
                        l.change_from = -1.0;
                        l.change_frame = 0.0;
                    }
                }
            }
            l.drift += (force * push - l.drift) * wind_ease;
            l.phase += 2.4;
            let spin = gml::degtorad(l.phase).sin() * vis("leaf_spin_degrees");
            let target = if l.change_from >= 0.0 {
                l.hold_angle
            } else {
                spin
            };
            l.angle += (target - l.angle) * spin_ease;
            l.lx += l.drift + gml::lengthdir_x(vis("backdrop_leaf_sway"), l.phase);
            l.y += l.fall;
            let sx = l.lx - cam_x * par;
            let off = sx < -140.0 || sx > w + 140.0;
            if l.y > floor_y + 40.0 || off {
                if arr.len() as f64 > count {
                    arr.remove(i as usize);
                } else {
                    let mut nl = backdrop_leaf_make(sim, band, cam_x, w, false);
                    if off {
                        nl.lx = cam_x * par + edge;
                        nl.y = sim.rng.random(floor_y * 0.7);
                    }
                    arr[i as usize] = nl;
                }
            }
            i -= 1;
        }
    }
    sim.forest.backdrop_leaves = Some(bands);
}

// ---------------------------------------------------------------------------------------------
// foreground leaves (scr_life): frame queries on the leaf art data

/// The pose fields of a foreground leaf the frame queries read (missing ones are `None`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LeafFgPose {
    pub turn: Option<f64>,
    pub turned: Option<bool>,
    pub angle: Option<f64>,
    pub mirror: Option<f64>,
}

// port: leaf_fg_art_ready
/// The frame data is embedded, so always ready (the original builds it on the first step).
pub fn leaf_fg_art_ready() -> bool {
    true
}

/// `leaf_fg_art.carry_x/carry_y`: offset of a carried leaf.
pub fn leaf_fg_carry() -> (f64, f64) {
    let a = &art().leaf_fg;
    (a.carry_x, a.carry_y)
}

// port: leaf_fg_frame
pub fn leaf_fg_frame(l: &LeafFgPose) -> LeafFgFrame {
    let a = &art().leaf_fg;
    let t = l.turn.unwrap_or(-1.0);
    if t >= 0.0 {
        return a.turn[gml::clamp(t, 0.0, 13.0).floor() as usize];
    }
    if l.turned.unwrap_or(false) {
        return a.turned;
    }
    a.rest
}

// port: leaf_fg_mirror
pub fn leaf_fg_mirror(l: &LeafFgPose) -> f64 {
    if l.mirror.unwrap_or(0.0) > 0.0 {
        -1.0
    } else {
        1.0
    }
}

// port: leaf_fg_drop
/// How far below its anchor the rotated leaf frame reaches (lowest corner).
pub fn leaf_fg_drop(l: &LeafFgPose) -> f64 {
    let f = leaf_fg_frame(l);
    let ang = l.angle.unwrap_or(0.0);
    let m = leaf_fg_mirror(l);
    let pts = [
        [f.x1 * m, f.y1],
        [f.x2 * m, f.y1],
        [f.x1 * m, f.y2],
        [f.x2 * m, f.y2],
    ];
    let mut best = -9999.0f64;
    for p in pts {
        let d = gml::point_distance(0.0, 0.0, p[0], p[1]);
        let a = gml::point_direction(0.0, 0.0, p[0], p[1]) + ang;
        best = best.max(gml::lengthdir_y(d, a));
    }
    best
}
