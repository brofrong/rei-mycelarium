//! Stones and rock clusters (Task 7). Task 5's world generation and tile queries call these
//! entry points; until Task 7 ports them they have neutral bodies (no stones), the same
//! neutral versions the Task 5 golden references were captured with
//! (rust/golden/scripts/world_lib.lua).

use super::WorldRt;
use crate::sim::Sim;
use crate::state::WorldChunkSave;

/// `world_paint_stone_clusters(chunk, cx)` (Task 7).
pub fn world_paint_stone_clusters(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _cx: i64) {}

/// `world_paint_stone_pillar(chunk, cx)` (Task 7).
pub fn world_paint_stone_pillar(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _cx: i64) {}

/// `world_paint_clusters(chunk, layer)` (Task 7): rock/water clusters of deeper layers.
pub fn world_paint_clusters(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _layer: i64) {}

/// `world_paint_stone_deposits(chunk, layer)` (Task 7).
pub fn world_paint_stone_deposits(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _layer: i64) {}

/// `world_paint_stones(chunk, layer)` (Task 7): loose stones.
pub fn world_paint_stones(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _layer: i64) {}

/// `stone_solid_at(chunk, x, y)` (Task 7): a stone deposit cell is solid here.
pub fn stone_solid_at(_w: &WorldRt, _chunk: &WorldChunkSave, _x: f64, _y: f64) -> bool {
    false
}

/// `world_stone_covers(chunk, x, y)` (Task 7): a loose stone covers this point.
pub fn world_stone_covers(_w: &WorldRt, _chunk: &WorldChunkSave, _x: f64, _y: f64) -> bool {
    false
}

/// `global.stone_reach` (set by Task 7's stone init; 48 while it does not exist).
pub fn stone_reach(_w: &WorldRt) -> f64 {
    48.0
}

/// `stones_step` (Task 7), called from obj_forest_controller Step_0.
pub fn stones_step(_sim: &mut Sim) {}

// ---------------------------------------------------------------------------------------------
// Ported early by Task 6 (unmarked: the rows are Task 7's). Caves place themselves on rock
// clusters (`world_cave_rects`, `cave_yellow_pick`), and the Task 6 references are captured
// with these live (ruling R19). Task 7 owns them: on merge keep Task 7's version and adapt the
// call sites in caves.rs.

/// Rock-cluster tier (`tier.id`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClusterTier {
    /// between `world_dense_y` and `world_deep_y` ("clay", salt 501)
    Clay,
    /// between `world_deep_y` and the ocean margin ("deep", salt 502)
    Deep,
}

impl ClusterTier {
    pub fn id(self) -> &'static str {
        match self {
            ClusterTier::Clay => "clay",
            ClusterTier::Deep => "deep",
        }
    }
}

/// One rock cluster (`___struct___241`, without its lazily built maze `data`).
#[derive(Clone, Debug, PartialEq)]
pub struct RockCluster {
    /// "<tier>_<s>"
    pub id: String,
    pub tier: ClusterTier,
    pub key: i64,
    pub col: i64,
    pub row: i64,
    pub r: i64,
    pub c1: i64,
    pub c2: i64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

/// `world_rock_clusters()`: the rock clusters of the clay and deep tiers, a pure function of
/// the world seed and start (the original caches it per start_x:seed).
pub fn world_rock_clusters(w: &WorldRt) -> Vec<RockCluster> {
    use crate::balance::{vis, wld};
    let mut out = Vec::new();
    if wld("rock_clusters_enabled") <= 0.0 {
        return out;
    }
    let sw = vis("screen_width_px");
    let tiers = [
        (
            ClusterTier::Clay,
            w.dense_y(),
            w.deep_y(),
            wld("rock_cluster_every_x"),
            501.0,
        ),
        (
            ClusterTier::Deep,
            w.deep_y(),
            w.ocean_y() - wld("ocean_clear_margin_px"),
            wld("rock_cluster_deep_every_x"),
            502.0,
        ),
    ];
    let minc = wld("rock_cluster_min_cells");
    let maxc = wld("rock_cluster_max_cells");
    let per_side = wld("rock_cluster_per_side");
    for (t, &(tier, top, bottom, every, salt)) in tiers.iter().enumerate() {
        let mut s = -per_side;
        while s <= per_side {
            let r = crate::gml::lerp(minc, maxc + 1.0, w.hash01(s, 1.0, salt)).floor();
            let jitter =
                wld("rock_cluster_jitter_x") * every / 0.1f64.max(wld("rock_cluster_every_x"));
            let cx =
                w.params.start_x + sw * (s * every + (w.hash01(s, 2.0, salt) - 0.5) * 2.0 * jitter);
            let pad = (r + 2.0) * 48.0;
            let ymin = top + pad + wld("caves_margin_px");
            let ymax = bottom - pad - wld("caves_margin_px");
            if ymax >= ymin {
                let cy = crate::gml::lerp(ymin, ymax, w.hash01(s, 3.0, salt));
                let col = (cx / 48.0).floor();
                let row = (cy / 48.0).floor();
                out.push(RockCluster {
                    id: format!("{}_{}", tier.id(), s as i64),
                    tier,
                    key: 90000 + t as i64 * 1000 + s as i64,
                    col: col as i64,
                    row: row as i64,
                    r: r as i64,
                    c1: (col - r - 3.0) as i64,
                    c2: (col + r + 3.0) as i64,
                    x1: (col - r - 3.0) * 48.0,
                    y1: (row - r - 3.0) * 48.0,
                    x2: (col + r + 4.0) * 48.0,
                    y2: (row + r + 4.0) * 48.0,
                });
            }
            s += 1.0;
        }
    }
    out
}

/// `stone_redress_after_cut(chunk, cut)` (Task 7): re-dress the stone cells around cells that
/// `world_clear_caves` cut out (keys "col:row"). Neutral until Task 7 ports it.
pub fn stone_redress_after_cut(
    _w: &mut WorldRt,
    _chunk: &mut WorldChunkSave,
    _cut: &std::collections::BTreeSet<String>,
) {
}
