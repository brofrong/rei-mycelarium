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

// ---------------------------------------------------------------------------------------------
// Ported early by Task 6 (unmarked: the row is Task 7's). Caves sit on mazes and avoid walls
// (`world_cave_rects`, `caves_extend`), and the Task 6 references are captured with it live
// (ruling R19). Task 7 owns it: on merge keep Task 7's version and adapt caves.rs.

/// `variable_struct_exists(world, "obstacles")`: a world that never asked for obstacles has the
/// default (zero) state; a created one has its cursors at start_x -/+ surface_wall_min_start_px.
fn obstacles_exist(o: &crate::state::ObstaclesSave) -> bool {
    let side =
        |s: &crate::state::ObstacleSideSave| s.count != 0 || s.cursor != 0.0 || !s.list.is_empty();
    side(&o.left) || side(&o.right)
}

/// `world_obstacles_reach(dir, target)`: extend the obstacle bands on one side (`dir` > 0 is
/// right) until their cursor passes `target`; the side's whole list.
pub fn world_obstacles_reach(w: &mut WorldRt, dir: f64, target: f64) -> Vec<ObstacleSave> {
    use crate::balance::{vis, wld};
    use crate::gml;
    if !obstacles_exist(&w.params.obstacles) {
        let start_min = wld("surface_wall_min_start_px");
        let o = &mut w.params.obstacles;
        o.right.cursor = w.params.start_x + start_min;
        o.left.cursor = w.params.start_x - start_min;
    }
    let chunk_w = w.params.chunk_w as f64;
    let seed = w.params.seed;
    let hash = |a: f64, b: f64, c: f64| super::hash::world_hash01(seed, a, b, c);
    let s = if dir > 0.0 {
        &mut w.params.obstacles.right
    } else {
        &mut w.params.obstacles.left
    };
    if wld("surface_obstacles_enabled") <= 0.0 {
        return s.list.clone();
    }
    loop {
        let go = if dir > 0.0 {
            s.cursor <= target
        } else {
            s.cursor >= target
        };
        if !go {
            break;
        }
        let idx = (s.count * 2 + if dir > 0.0 { 0 } else { 1 }) as f64;
        let gap = gml::lerp(
            wld("surface_obstacle_gap_min_px"),
            wld("surface_obstacle_gap_max_px"),
            hash(idx, 0.0, 210.0),
        );
        let every = 1f64.max(wld("maze_every"));
        let new_cursor = if gml::gmod(s.count as f64, every) == every - 1.0 {
            let wsc = gml::lerp(
                wld("maze_min_width_screens"),
                wld("maze_max_width_screens"),
                hash(idx, 1.0, 211.0),
            );
            let wcells = 12f64.max(gml::round(wsc * vis("screen_width_px") / 48.0));
            let c = gml::round((s.cursor + dir * gap) / 48.0);
            let c1 = if dir > 0.0 { c } else { c - wcells };
            s.list.push(ObstacleSave {
                key: idx as i64,
                kind: "maze".into(),
                x1: c1 * 48.0,
                x2: (c1 + wcells) * 48.0,
                cx: None,
                c1: Some(c1),
                c2: Some(c1 + wcells - 1.0),
                extra: Default::default(),
            });
            if dir > 0.0 {
                (c1 + wcells) * 48.0
            } else {
                c1 * 48.0
            }
        } else {
            let cx = ((s.cursor + dir * gap) / chunk_w).floor();
            s.list.push(ObstacleSave {
                key: idx as i64,
                kind: "wall".into(),
                x1: cx * chunk_w,
                x2: (cx + 1.0) * chunk_w,
                cx: Some(cx as i64),
                c1: None,
                c2: None,
                extra: Default::default(),
            });
            if dir > 0.0 {
                (cx + 1.0) * chunk_w
            } else {
                cx * chunk_w
            }
        };
        s.cursor = new_cursor;
        s.count += 1;
    }
    s.list.clone()
}
