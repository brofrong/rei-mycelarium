//! Surface obstacles (walls and mazes, Task 7). The obstacle bands are saved in
//! `world.obstacles` (kept as loaded by `WorldRt`); Task 5's chunk generation asks which bands
//! touch a chunk and paints them. Until Task 7 ports these entry points they are neutral (no
//! obstacles), as in the Task 5 golden references (rust/golden/scripts/world_lib.lua).

use super::WorldRt;
use crate::state::{ObstacleSave, WorldChunkSave};

/// `world_obstacles_at_chunk(chunk)` (Task 7): the obstacle bands overlapping the chunk.
pub fn world_obstacles_at_chunk(_w: &mut WorldRt, _chunk: &WorldChunkSave) -> Vec<ObstacleSave> {
    Vec::new()
}

/// `world_paint_maze_part(chunk, obstacle)` (Task 7).
pub fn world_paint_maze_part(_w: &mut WorldRt, _chunk: &mut WorldChunkSave, _obs: &ObstacleSave) {}

/// `world_paint_surface_wall(chunk)` (Task 7).
pub fn world_paint_surface_wall(_w: &mut WorldRt, _chunk: &mut WorldChunkSave) {}
