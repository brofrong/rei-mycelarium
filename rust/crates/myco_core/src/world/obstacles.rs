//! Surface obstacles (scr_stones, Task 7): bands along the surface on both sides of the start,
//! every `maze_every`-th one a maze (see `maze`), the others a stone wall in one chunk. The
//! bands are generated lazily outward from `start_x` and saved in `world.obstacles` (built the
//! first time it is needed, so a save without the key starts afresh).

use super::stones::{self, CellSet};
use super::WorldRt;
use crate::balance::{vis, wld};
use crate::gml;
use crate::state::{ObstacleSave, ObstacleSideSave, ObstaclesSave, WorldChunkSave};

// port: world_obstacles_reach
/// Extend the obstacle bands on side `dir` (> 0 right, else left) until their cursor passes
/// `target`; the side's whole list.
pub fn world_obstacles_reach(w: &mut WorldRt, dir: f64, target: f64) -> &[ObstacleSave] {
    let start_x = w.params.start_x;
    let cw = w.params.chunk_w as f64;
    let o = w.params.obstacles.get_or_insert_with(|| {
        let start_min = wld("surface_wall_min_start_px");
        ObstaclesSave {
            right: ObstacleSideSave {
                cursor: start_x + start_min,
                ..Default::default()
            },
            left: ObstacleSideSave {
                cursor: start_x - start_min,
                ..Default::default()
            },
            ..Default::default()
        }
    });
    let seed = w.params.seed;
    let s = if dir > 0.0 { &mut o.right } else { &mut o.left };
    if wld("surface_obstacles_enabled") <= 0.0 {
        return &s.list;
    }
    let hash = |a: f64, b: f64, c: f64| super::hash::world_hash01(seed, a, b, c);
    loop {
        let go = if dir > 0.0 {
            s.cursor <= target
        } else {
            s.cursor >= target
        };
        if !go {
            break;
        }
        let idx = s.count * 2 + if dir > 0.0 { 0 } else { 1 };
        let idxf = idx as f64;
        let gap = gml::lerp(
            wld("surface_obstacle_gap_min_px"),
            wld("surface_obstacle_gap_max_px"),
            hash(idxf, 0.0, 210.0),
        );
        let every = 1f64.max(wld("maze_every"));
        let new_cursor = if gml::gmod(s.count as f64, every) == every - 1.0 {
            let wsc = gml::lerp(
                wld("maze_min_width_screens"),
                wld("maze_max_width_screens"),
                hash(idxf, 1.0, 211.0),
            );
            let wcells = 12f64.max(gml::round(wsc * vis("screen_width_px") / 48.0));
            let c = gml::round((s.cursor + dir * gap) / 48.0);
            let c1 = if dir > 0.0 { c } else { c - wcells };
            s.list.push(ObstacleSave {
                kind: "maze".into(),
                key: idx,
                c1: Some(c1),
                c2: Some(c1 + wcells - 1.0),
                x1: c1 * 48.0,
                x2: (c1 + wcells) * 48.0,
                data: Some(None),
                ..Default::default()
            });
            if dir > 0.0 {
                (c1 + wcells) * 48.0
            } else {
                c1 * 48.0
            }
        } else {
            let cx = ((s.cursor + dir * gap) / cw).floor();
            s.list.push(ObstacleSave {
                kind: "wall".into(),
                key: idx,
                cx: Some(cx as i64),
                x1: cx * cw,
                x2: (cx + 1.0) * cw,
                ..Default::default()
            });
            if dir > 0.0 {
                (cx + 1.0) * cw
            } else {
                cx * cw
            }
        };
        s.cursor = new_cursor;
        s.count += 1;
    }
    &s.list
}

// port: world_obstacles_at_chunk
/// The obstacle bands overlapping the chunk (reaching that far on its side of the start).
/// The copies leave out the cached maze data: `world_paint_maze_part` looks it up by key.
pub fn world_obstacles_at_chunk(w: &mut WorldRt, ch: &WorldChunkSave) -> Vec<ObstacleSave> {
    let cw = w.params.chunk_w as f64;
    let x1 = ch.cx as f64 * cw;
    let x2 = x1 + cw;
    let dir = if x1 + cw * 0.5 < w.params.start_x {
        -1.0
    } else {
        1.0
    };
    world_obstacles_reach(w, dir, if dir > 0.0 { x2 } else { x1 })
        .iter()
        .filter(|o| o.x2 > x1 && o.x1 < x2)
        .map(|o| ObstacleSave {
            data: o.data.as_ref().map(|_| None),
            ..o.clone_without_data()
        })
        .collect()
}

impl ObstacleSave {
    fn clone_without_data(&self) -> ObstacleSave {
        ObstacleSave {
            key: self.key,
            kind: self.kind.clone(),
            x1: self.x1,
            x2: self.x2,
            cx: self.cx,
            c1: self.c1,
            c2: self.c2,
            data: None,
            extra: self.extra.clone(),
        }
    }
}

/// `world_paint_maze_part(chunk, obstacle)` for a maze obstacle (see `maze::paint_part`).
pub fn world_paint_maze_part(w: &mut WorldRt, ch: &mut WorldChunkSave, obs: &ObstacleSave) {
    super::maze::paint_part(w, ch, super::maze::MazePart::Maze(obs.key));
}

// port: world_paint_surface_wall
/// A stone wall rising from the surface in this chunk: a band of 3+ cells whose sides wander
/// with random slopes down to the wall depth, rounded at the bottom, its edges bitten, with a
/// round hole and an outlet tunnel around each lake it meets. Up to four placements are tried;
/// none may come near the start.
pub fn world_paint_surface_wall(w: &mut WorldRt, ch: &mut WorldChunkSave) {
    const SLOPES: [f64; 8] = [0.0, 0.0, -1.0, 1.0, -0.5, 0.5, -0.5, 0.5];
    let cx = ch.cx as f64;
    let cw = w.params.chunk_w as f64;
    let start_x = w.params.start_x;
    let min_w = 3f64.max(wld("surface_wall_min_width_cells"));
    let max_w = min_w.max(wld("surface_wall_max_width_cells"));
    let width = min_w + (w.hash01(cx, 1.0, 192.0) * (max_w - min_w + 1.0)).floor();
    let depth = 3f64.max(gml::round(
        wld("surface_wall_depth_screens") * vis("screen_height_px") / 48.0,
    ));
    let x0 = cx * cw;
    let c0 = (x0 / 48.0).ceil() + 1.0;
    let c1 = ((x0 + cw) / 48.0).floor() - 2.0;
    let top = (w.params.surface_y / 48.0).floor();
    let min_start = wld("surface_wall_min_start_px");
    let clear = wld("stone_water_clearance_px");
    let span = c1 - c0 - width + 2.0;
    if span <= 0.0 {
        return;
    }
    for attempt in 0..4 {
        let attempt = attempt as f64;
        let x0c = c0 + (w.hash01(cx, 3.0 + attempt, 194.0) * span).floor();
        let rows = (depth + 2.0) as usize;
        let mut lft = vec![x0c; rows];
        let mut rgt = vec![x0c + width - 1.0; rows];
        let min_deep = 3f64.max(wld("surface_wall_min_deep_width_cells"));
        let mut lx = x0c;
        let mut rx = x0c + width - 1.0;
        let (mut sl, mut sr, mut runl, mut runr) = (0.0, 0.0, 0.0, 0.0);
        let mut minc = x0c;
        let mut maxc = x0c + width - 1.0;
        let salt_b = cx + attempt * 7919.0;
        for r in 3..rows {
            let rf = r as f64;
            if runl <= 0.0 {
                runl = 3.0 + (w.hash01(rf, salt_b, 199.0) * 8.0).floor();
                sl = SLOPES[(w.hash01(rf, salt_b, 200.0) * 8.0).floor() as usize];
            }
            if runr <= 0.0 {
                runr = 3.0 + (w.hash01(rf, salt_b, 201.0) * 8.0).floor();
                sr = SLOPES[(w.hash01(rf, salt_b, 202.0) * 8.0).floor() as usize];
            }
            runl -= 1.0;
            runr -= 1.0;
            let nl = gml::clamp(lx + sl, c0, c1);
            let nr = gml::clamp(rx + sr, c0, c1);
            if (nr + 0.5).floor() - (nl + 0.5).floor() + 1.0 >= min_deep {
                lx = nl;
                rx = nr;
            } else {
                sl = 0.0;
                sr = 0.0;
            }
            lft[r] = (lx + 0.5).floor();
            rgt[r] = (rx + 0.5).floor();
            minc = minc.min(lft[r]);
            maxc = maxc.max(rgt[r]);
        }
        let x0px = minc * 48.0;
        let x1px = (maxc + 1.0) * 48.0;
        if x1px > start_x - min_start && x0px < start_x + min_start {
            continue;
        }
        let mut cut = CellSet::new();
        let bl = lft[depth as usize - 1];
        let br = rgt[depth as usize - 1];
        let mid = (bl + br) * 0.5;
        let half = (br - bl + 1.0) * 0.5 + 0.5;
        let bite = wld("surface_wall_side_bite_chance");
        let mut c = minc;
        while c <= maxc {
            let t = gml::clamp((c - mid) / half, -1.0, 1.0);
            let dd = gml::round((1.0 - 0f64.max(1.0 - t * t).sqrt()) * half);
            let inner =
                3f64.max(depth - dd + gml::round((w.hash01(c, cx, 195.0) * 2.0 - 1.0) * 1.2));
            let mut r = 0usize;
            while (r as f64) < inner.min(rows as f64) {
                if !(c < lft[r] || c > rgt[r]) {
                    let rf = r as f64;
                    // bitten edges (if / elseif in the original; the hashes are pure)
                    let skip = r > 2
                        && rgt[r] - lft[r] + 1.0 - 2.0 >= min_deep
                        && ((c == lft[r] && w.hash01(rf, cx, 197.0) < bite)
                            || (c == rgt[r] && w.hash01(rf, cx, 198.0) < bite));
                    if !skip {
                        cut.insert((c as i64, (top + rf) as i64));
                    }
                }
                r += 1;
            }
            c += 1.0;
        }
        let tunnel_rows = 0f64.max(wld("surface_wall_lake_tunnel_rows") - 1.0) * 0.5;
        for (i, lk) in ch.lakes.iter().enumerate() {
            let lx0 = lk.body_x - clear;
            let lx1 = lk.body_x + lk.body_w + clear;
            let ly0 = lk.body_y - clear;
            let ly1 = lk.body_y + lk.body_h + clear;
            if lx1 <= x0px || lx0 >= x1px || ly1 <= top * 48.0 || ly0 >= (top + depth + 2.0) * 48.0
            {
                continue;
            }
            let seed = cx * 13.0 + i as f64;
            let lcx = (lk.body_x + lk.body_w * 0.5) / 48.0;
            let lcy = (lk.body_y + lk.body_h * 0.5) / 48.0;
            let lrx = (lx1 - lx0) * 0.5 / 48.0 + 0.4;
            let lry = (ly1 - ly0) * 0.5 / 48.0 + 0.4;
            stones::stone_cut_round(w, &mut cut, lcx, lcy, lrx, lry, seed);
            let mx = lk.body_x + lk.body_w * 0.5;
            let lrow = lcy.floor();
            let rr = gml::clamp(lrow - top, 0.0, rows as f64 - 1.0) as usize;
            let (lr_, rr_) = (lft[rr], rgt[rr]);
            let go_left = mx - lr_ * 48.0 <= (rr_ + 1.0) * 48.0 - mx;
            let ex = if go_left { lr_ - 1.5 } else { rr_ + 2.5 };
            let ey = lcy + (w.hash01(seed, 1.0, 231.0) - 0.5) * 3.0;
            stones::stone_cut_tunnel(
                w,
                &mut cut,
                lcx,
                lcy,
                ex,
                ey,
                1f64.max(tunnel_rows + 0.6),
                seed,
            );
        }
        let mut blob = stones::stone_blob_finish(
            &mut cut,
            minc as i64 - 1,
            maxc as i64 + 1,
            top as i64 - 1,
            (top + rows as f64 + 1.0) as i64,
            6,
        );
        if (blob.list.len() as f64) < width * 3.0 {
            continue;
        }
        let t = top as i64;
        let tops: Vec<i64> = blob.list.iter().filter(|c| c.1 == t).map(|c| c.0).collect();
        for col in tops {
            blob.keys.insert((col, t - 1));
        }
        let band = (w.hash01(cx, 9.0, 196.0) * 2.0).floor() as i64;
        stones::stone_dress_blob(w, &mut ch.stone_cells, &mut ch.stone_map, &blob, band);
        return;
    }
}
