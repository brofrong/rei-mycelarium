//! Caves (Task 6). Task 5's world generation and tile queries call these entry points; until
//! Task 6 ports them they have neutral bodies (no caves), the same neutral versions the Task 5
//! golden references were captured with (rust/golden/scripts/world_lib.lua).

use super::WorldRt;
use crate::state::WorldChunkSave;

/// `caves_extend(x)` (Task 6): grow the cave network to cover world x.
pub fn caves_extend(_w: &mut WorldRt, _x: f64) {}

/// `world_paint_bones(chunk, layer)` (Task 6).
pub fn world_paint_bones(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _layer: i64) {}

/// `world_clear_caves(chunk)` (Task 6): clear tiles, foods, lakes and stones inside caves.
pub fn world_clear_caves(_w: &mut WorldRt, _chunk: &mut WorldChunkSave) {}

/// `world_cave_hits(x1, y1, x2, y2, pad)` (Task 6): the rectangle touches a cave clear zone.
pub fn world_cave_hits(
    _w: &WorldRt,
    _x1: f64,
    _y1: f64,
    _x2: f64,
    _y2: f64,
    _pad: Option<f64>,
) -> bool {
    false
}

/// `world_cave_solid_any(x, y)` (Task 6): a cave wall is at this point.
pub fn world_cave_solid_any(_w: &WorldRt, _x: f64, _y: f64) -> bool {
    false
}
