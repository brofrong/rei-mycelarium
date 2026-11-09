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
