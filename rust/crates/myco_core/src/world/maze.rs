//! Mazes and rock clusters (scr_stones, Task 7). A maze is a surface obstacle band (see
//! `obstacles`); a rock cluster is a round rock body in the clay or deep soil. Both are carved
//! the same way on a grid of 48 px cells: tunnels, chambers and hollows are opened with
//! `maze_rng` (a hash sequence, so the result depends only on the world seed), the rest becomes
//! stone cells, and rooms get foods or a small lake. The built data is cached (in the obstacle,
//! saved; or in the cluster, runtime) and painted into each chunk it overlaps.

use super::stones::{self, CellSet, StoneBlob};
use super::{caves, food, hash, lakes, WorldRt};
use crate::balance::{gml_truthy, vis, wld};
use crate::gml;
use crate::state::{FoodSave, LakeSave, MazeData, MazeFoodSave, WorldChunkSave};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The carving state (`___struct___237__world_maze_build` / `___struct___242__world_cluster_build`):
/// a `w` x `h` grid of cells whose top-left cell is (c1, top); `open` cells are carved out;
/// tunnels bounce inside [xmin, xmax] x [ymin, ymax]; rows above `keep_top` are never opened;
/// clusters also have the `inside` mask of their round body.
#[derive(Clone, Debug, PartialEq)]
pub struct Maze {
    /// world seed (`maze_rng` hashes with it)
    pub seed: i64,
    pub key: f64,
    /// draws so far
    pub n: f64,
    pub c1: f64,
    pub c2: f64,
    pub top: f64,
    pub w: i64,
    pub h: i64,
    pub keep_top: i64,
    pub xmin: f64,
    pub xmax: f64,
    pub ymin: f64,
    pub ymax: f64,
    pub open: Vec<bool>,
    pub inside: Option<Vec<bool>>,
}

impl Maze {
    /// A maze used only as an RNG stream (`{key, n = 0}`).
    pub fn rng_only(seed: i64, key: f64) -> Maze {
        Maze {
            seed,
            key,
            n: 0.0,
            c1: 0.0,
            c2: 0.0,
            top: 0.0,
            w: 0,
            h: 0,
            keep_top: 0,
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
            open: Vec::new(),
            inside: None,
        }
    }
}

/// A chamber or hollow (`___struct___231__maze_room` plus the fields the callers add).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MazeRoom {
    pub cx: f64,
    pub cy: f64,
    pub rx: f64,
    pub ry: f64,
    pub water: bool,
    pub food_min: f64,
    pub food_max: f64,
}

/// A rock cluster (`___struct___241__world_rock_clusters`), `data` cached once built.
#[derive(Clone, Debug, PartialEq)]
pub struct RockCluster {
    /// `"<tier>_<s>"`
    pub id: String,
    /// `"clay"` or `"deep"`
    pub tier: String,
    pub key: f64,
    pub col: f64,
    pub row: f64,
    pub r: f64,
    pub c1: f64,
    pub c2: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub data: Option<MazeData>,
}

impl RockCluster {
    /// The GML struct without `data`, as JSON.
    pub fn to_json(&self) -> Value {
        json!({ "kind": "cluster", "id": self.id, "tier": self.tier, "key": self.key,
                "col": self.col, "row": self.row, "r": self.r, "c1": self.c1, "c2": self.c2,
                "x1": self.x1, "y1": self.y1, "x2": self.x2, "y2": self.y2 })
    }
}

/// What `world_paint_maze_part` paints: a maze obstacle (by key) or a rock cluster (by index in
/// `world_rock_clusters`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MazePart {
    Maze(i64),
    Cluster(usize),
}

// ---------------------------------------------------------------------------------------------
// primitives

// port: maze_rng
/// The next value of the maze's hash sequence: `world_hash01(key, ++n, 220)`.
pub fn maze_rng(m: &mut Maze) -> f64 {
    m.n += 1.0;
    hash::world_hash01(m.seed, m.key, m.n, 220.0)
}

// port: maze_rng_int
/// An integer in [lo, hi] from `maze_rng`.
pub fn maze_rng_int(m: &mut Maze, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo).min((maze_rng(m) * (hi - lo + 1.0)).floor())
}

// port: maze_open_at
/// Carve cell (col, row) if it is inside the grid and below the kept top rows.
pub fn maze_open_at(m: &mut Maze, col: f64, row: f64) {
    let lc = col - m.c1;
    let lr = row - m.top;
    if !(lc >= 0.0 && lc < m.w as f64 && lr >= m.keep_top as f64 && lr < m.h as f64) {
        return;
    }
    m.open[(lc + lr * m.w as f64).floor() as usize] = true;
}

// port: maze_is_open
pub fn maze_is_open(m: &Maze, col: f64, row: f64) -> bool {
    let lc = col - m.c1;
    let lr = row - m.top;
    if !(lc >= 0.0 && lc < m.w as f64 && lr >= 0.0 && lr < m.h as f64) {
        return false;
    }
    m.open[(lc + lr * m.w as f64).floor() as usize]
}

// port: maze_disc
/// Carve the cells whose centres lie within `r` of (cx, cy).
pub fn maze_disc(m: &mut Maze, cx: f64, cy: f64, r: f64) {
    let mut row = (cy - r).floor() - 1.0;
    while row <= (cy + r).ceil() + 1.0 {
        let mut col = (cx - r).floor() - 1.0;
        while col <= (cx + r).ceil() + 1.0 {
            let dx = col + 0.5 - cx;
            let dy = row + 0.5 - cy;
            if dx * dx + dy * dy <= r * r {
                maze_open_at(m, col, row);
            }
            col += 1.0;
        }
        row += 1.0;
    }
}

// port: maze_room
/// Carve a wobbly elliptic room, its centre clamped into the maze bounds.
pub fn maze_room(m: &mut Maze, cx: f64, cy: f64, rx: f64, ry: f64) -> MazeRoom {
    let cy = gml::clamp(cy, m.ymin + ry * 0.6, m.ymax + 2.0 - ry);
    let cx = gml::clamp(cx, m.xmin + rx * 0.5, m.xmax - rx * 0.5);
    let p1 = maze_rng(m) * stones::PHASE_RANGE;
    let p2 = maze_rng(m) * stones::PHASE_RANGE;
    let mut row = (cy - ry).floor() - 2.0;
    while row <= (cy + ry).ceil() + 2.0 {
        let mut col = (cx - rx).floor() - 2.0;
        while col <= (cx + rx).ceil() + 2.0 {
            let dx = (col + 0.5 - cx) / rx;
            let dy = (row + 0.5 - cy) / ry;
            let a = dy.atan2(dx);
            let k = 1.0 + (3.0 * a + p1).sin() * 0.18 + (5.0 * a + p2).sin() * 0.1;
            if dx * dx + dy * dy <= k * k {
                maze_open_at(m, col, row);
            }
            col += 1.0;
        }
        row += 1.0;
    }
    MazeRoom {
        cx,
        cy,
        rx,
        ry,
        ..Default::default()
    }
}

// port: maze_tunnel
/// Carve a drifting tunnel of radius between r0 and r1 (with random bulges) for `len` cells,
/// steering toward `target` (stopping within one cell of it) and bouncing off the bounds
/// (horizontally only when not `free`). Returns the visited points.
#[allow(clippy::too_many_arguments)]
pub fn maze_tunnel(
    m: &mut Maze,
    mut x: f64,
    mut y: f64,
    mut dir: f64,
    len: f64,
    r0: f64,
    r1: f64,
    target: Option<(f64, f64)>,
    free: bool,
) -> Vec<(f64, f64)> {
    const PI: f64 = std::f64::consts::PI;
    let ph = maze_rng(m) * stones::PHASE_RANGE;
    let fr = 0.15 + maze_rng(m) * 0.2;
    let mut drift = 0.0;
    let mut bulge = 0.0;
    let mut bulge_amp = 0.0;
    let mut pts = Vec::new();
    let steps = (len / 0.6).floor();
    let bulge_ch = wld("maze_bulge_chance");
    let mut i = 0.0;
    while i < steps {
        let mut rr = gml::lerp(r0, r1, ((i * 0.6 * fr + ph).sin() + 1.0) * 0.5);
        if bulge <= 0.0 && maze_rng(m) < bulge_ch {
            bulge = 6.0 + maze_rng(m) * 6.0;
            bulge_amp = 1.0 + maze_rng(m) * 1.6;
        }
        if bulge > 0.0 {
            bulge -= 0.6;
            rr += bulge_amp * (PI * 1f64.min(bulge / 6.0)).sin();
        }
        maze_disc(m, x, y, rr);
        pts.push((x, y));
        drift = drift * 0.85 + (maze_rng(m) - 0.5) * 14.0;
        dir += drift;
        if let Some((tx, ty)) = target {
            let td = gml::point_direction(x, y, tx, ty);
            dir += gml::angle_difference(td, dir) * 0.12;
            if gml::point_distance(x, y, tx, ty) < 1.0 {
                return pts;
            }
        }
        let mut nx = x + gml::lengthdir_x(0.6, dir);
        let mut ny = y + gml::lengthdir_y(0.6, dir);
        if !free && (nx < m.xmin || nx > m.xmax) {
            dir = 180.0 - dir;
            nx = x;
        }
        if ny < m.ymin || ny > m.ymax {
            dir = -dir;
            ny = y;
        }
        x = nx;
        y = ny;
        i += 1.0;
    }
    pts
}

// port: maze_distance
/// Breadth-first distance (in cells) of every cell from the nearest seed: open cells (except
/// those in `seeds`), the grid border, and cells outside the `inside` mask. -1 = unreached.
pub fn maze_distance(m: &Maze, seeds: Option<&[bool]>) -> Vec<i64> {
    let w = m.w as usize;
    let h = m.h as usize;
    let n = w * h;
    let mut dist = vec![-1i64; n];
    let mut q = vec![0usize; n + 4];
    let mut head = 0;
    let mut tail = 0;
    for i in 0..n {
        let cx = i % w;
        let cy = i / w;
        let is_seed = (m.open[i] && seeds.is_none_or(|s| !s[i]))
            || cx == 0
            || cx == w - 1
            || cy == 0
            || cy == h - 1
            || m.inside.as_ref().is_some_and(|inside| !inside[i]);
        if is_seed {
            dist[i] = 0;
            q[tail] = i;
            tail += 1;
        }
    }
    while head < tail {
        let i = q[head];
        let cx = i % w;
        let cy = i / w;
        head += 1;
        for (ok, j) in [
            (cx > 0, i.wrapping_sub(1)),
            (cx + 1 < w, i + 1),
            (cy > 0, i.wrapping_sub(w)),
            (cy + 1 < h, i + w),
        ] {
            if ok && dist[j] < 0 {
                dist[j] = dist[i] + 1;
                q[tail] = j;
                tail += 1;
            }
        }
    }
    dist
}

// port: maze_inside
/// The cell is inside the cluster body (always, for a maze).
pub fn maze_inside(m: &Maze, lc: i64, lr: i64) -> bool {
    match &m.inside {
        None => true,
        Some(inside) => inside[(lc + lr * m.w) as usize],
    }
}

// port: maze_dig_out
/// Walk downhill on `dist` from local cell (lc, lr) to the nearest open cell, carving a
/// narrow wavy passage.
pub fn maze_dig_out(m: &mut Maze, dist: &[i64], mut lc: i64, mut lr: i64) {
    let tmin = wld("maze_tunnel_min_cells");
    let ph = maze_rng(m) * 6.0;
    let idx = |m: &Maze, c: i64, r: i64| (c + r * m.w) as usize;
    let mut k = 0.0;
    while k < 120.0 {
        if dist[idx(m, lc, lr)] <= 0 {
            return;
        }
        let (mut bc, mut br) = (lc, lr);
        let mut bd = dist[idx(m, lc, lr)];
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nc = lc + dx;
                let nr = lr + dy;
                if nc >= 0 && nr >= 0 && nc < m.w && nr < m.h {
                    let d = dist[idx(m, nc, nr)];
                    if d >= 0 && d < bd {
                        bd = d;
                        bc = nc;
                        br = nr;
                    }
                }
            }
        }
        if bc == lc && br == lr {
            return;
        }
        lc = bc;
        lr = br;
        let x = m.c1 + lc as f64 + 0.5 + (k * 0.5 + ph).sin() * 0.6;
        let y = m.top + lr as f64 + 0.5;
        let r = gml::lerp(tmin * 0.9, tmin * 1.5, ((k * 0.3 + ph).sin() + 1.0) * 0.5);
        maze_disc(m, x, y, r);
        k += 1.0;
    }
}

// port: maze_cave_hollows
/// Open a rounded hollow around every cave rectangle overlapping (x1, y1)-(x2, y2) and dig it
/// out to the rest of the maze.
pub fn maze_cave_hollows(w: &mut WorldRt, m: &mut Maze, x1: f64, y1: f64, x2: f64, y2: f64) {
    caves::caves_extend(w, x1);
    caves::caves_extend(w, x2);
    let rects: Vec<(f64, f64, f64, f64)> = w
        .cave_rects()
        .iter()
        .map(|r| (r.x1, r.y1, r.x2, r.y2))
        .collect();
    let pad = wld("caves_clearance_px") + 24.0;
    let (mw, mh) = (m.w, m.h);
    for (rx1, ry1, rx2, ry2) in rects {
        if rx2 <= x1 || rx1 >= x2 || ry2 <= y1 || ry1 >= y2 {
            continue;
        }
        let cx = (rx1 + rx2) * 0.5 / 48.0;
        let cy = (ry1 + ry2) * 0.5 / 48.0;
        let rx = ((rx2 - rx1) * 0.5 + pad) / 48.0;
        let ry = ((ry2 - ry1) * 0.5 + pad) / 48.0;
        let phase = maze_rng(m) * stones::PHASE_RANGE;
        let mut mask = vec![false; (mw * mh) as usize];
        let mut yy = (cy - ry * 1.2).floor() - 1.0;
        while yy <= (cy + ry * 1.2).ceil() + 1.0 {
            let mut xx = (cx - rx * 1.2).floor() - 1.0;
            while xx <= (cx + rx * 1.2).ceil() + 1.0 {
                let dx = (xx + 0.5 - cx) / rx;
                let dy = (yy + 0.5 - cy) / ry;
                let a = dy.atan2(dx);
                let k = 1.05 + 0.08 * (4.0 * a + phase).sin();
                if dx.abs().powf(4.0) + dy.abs().powf(4.0) <= k.powf(4.0) {
                    let lx = (xx - m.c1) as i64;
                    let ly = (yy - m.top) as i64;
                    if lx >= 0 && ly >= 0 && lx < mw && ly < mh {
                        m.open[(ly * mw + lx) as usize] = true;
                        mask[(ly * mw + lx) as usize] = true;
                    }
                }
                xx += 1.0;
            }
            yy += 1.0;
        }
        let sx = gml::clamp(cx.floor() - m.c1, 0.0, (mw - 1) as f64) as i64;
        let sy = gml::clamp(cy.floor() - m.top, 0.0, (mh - 1) as f64) as i64;
        let dist = maze_distance(m, Some(&mask));
        maze_dig_out(m, &dist, sx, sy);
    }
}

// port: maze_hollows
/// Up to three passes: put a hollow (a room, sometimes with water) into the solid areas
/// farthest from any opening, until none is thicker than `maze_solid_max_cells`.
pub fn maze_hollows(m: &mut Maze, out: &mut Vec<MazeRoom>) {
    let max_cells = wld("maze_solid_max_cells");
    let (w, h) = (m.w, m.h);
    for _pass in 0..3 {
        let mut dist = maze_distance(m, None);
        let mut made = 0;
        for _n in 0..16 {
            let mut best: i64 = -1;
            let mut best_d = max_cells;
            for c in 0..w * h {
                let cy = m.top + (c / w) as f64;
                let d = dist[c as usize] as f64;
                if d > best_d && cy >= m.ymin && cy <= m.ymax + 2.0 {
                    best_d = d;
                    best = c;
                }
            }
            if best < 0 {
                break;
            }
            let hx = best % w;
            let hy = best / w;
            let water = maze_rng(m) < wld("maze_hollow_water_share");
            let (x, y) = (m.c1 + hx as f64 + 0.5, m.top + hy as f64 + 0.5);
            let mut room = if water {
                maze_room(m, x, y, 3.0, 2.2)
            } else {
                let rx = 2.8 + maze_rng(m) * 2.4;
                let ry = 2.0 + maze_rng(m) * 1.6;
                maze_room(m, x, y, rx, ry)
            };
            room.water = water;
            room.food_min = wld("maze_hollow_food_min");
            room.food_max = wld("maze_hollow_food_max");
            out.push(room);
            made += 1;
            maze_dig_out(m, &dist, hx, hy);
            let rr = max_cells as i64 + 4;
            for dy in -rr..=rr {
                for dx in -rr..=rr {
                    let xx = hx + dx;
                    let yy = hy + dy;
                    if xx >= 0 && yy >= 0 && xx < w && yy < h {
                        let k = (yy * w + xx) as usize;
                        dist[k] = dist[k].min(0.max(dy.abs() + dx.abs() - 3));
                    }
                }
            }
        }
        if made == 0 {
            break;
        }
    }
}

// port: maze_branches
/// Side tunnels from random points of the main path, some ending in a chamber (water or food).
pub fn maze_branches(m: &mut Maze, cells: &[(f64, f64)], out: &mut Vec<MazeRoom>) {
    let min_cells = wld("maze_tunnel_min_cells");
    let max_cells = wld("maze_tunnel_max_cells");
    let n = maze_rng_int(m, wld("maze_branches_min"), wld("maze_branches_max"));
    let mut i = 0.0;
    while i < n {
        if cells.is_empty() {
            break;
        }
        let hub = cells[(maze_rng(m) * cells.len() as f64).floor() as usize];
        let dir = maze_rng(m) * 360.0;
        let len = 8.0 + maze_rng(m) * 22.0;
        let path = maze_tunnel(
            m,
            hub.0,
            hub.1,
            dir,
            len,
            min_cells * 0.8,
            max_cells * 0.8,
            None,
            false,
        );
        i += 1.0;
        if path.is_empty() || maze_rng(m) >= wld("maze_chamber_chance") {
            continue;
        }
        let e = path[path.len() - 1];
        let ex = gml::clamp(e.0.floor() - m.c1, 0.0, (m.w - 1) as f64) as i64;
        let ey = gml::clamp(e.1.floor() - m.top, 0.0, (m.h - 1) as f64) as i64;
        if !maze_inside(m, ex, ey) {
            continue;
        }
        let water = maze_rng(m) < wld("maze_water_share");
        let mut room = if water {
            maze_room(m, e.0, e.1, 3.0, 2.2)
        } else {
            let rx = 2.4 + maze_rng(m) * 2.0;
            let ry = 1.8 + maze_rng(m) * 1.3;
            maze_room(m, e.0, e.1, rx, ry)
        };
        room.water = water;
        room.food_min = wld("maze_food_min");
        room.food_max = wld("maze_food_max");
        out.push(room);
    }
}

// port: maze_finish
/// Turn the uncarved cells (`cut`) into dressed stone cells (largest pieces of 6+ cells; the
/// top row sealed against the surface when `seal_top`) and fill the rooms: a lake when the
/// room is watery and the lake fits in one chunk, else foods.
#[allow(clippy::too_many_arguments)]
pub fn maze_finish(
    w: &WorldRt,
    m: &mut Maze,
    cut: &mut CellSet,
    rooms: &[MazeRoom],
    c0: i64,
    c1: i64,
    r0: i64,
    r1: i64,
    seal_top: bool,
) -> MazeData {
    let cw = w.params.chunk_w as f64;
    let mut blob: StoneBlob = stones::stone_blob_finish(cut, c0, c1, r0, r1, 6);
    if seal_top {
        let top = m.top as i64;
        let tops: Vec<i64> = blob
            .list
            .iter()
            .filter(|c| c.1 == top)
            .map(|c| c.0)
            .collect();
        for col in tops {
            blob.keys.insert((col, top - 1));
        }
    }
    let mut entries = Vec::new();
    let mut map = BTreeMap::new();
    let band = maze_rng_int(m, 0.0, 1.0) as i64;
    stones::stone_dress_blob(w, &mut entries, &mut map, &blob, band);
    let mut foods = Vec::new();
    let mut lake_list = Vec::new();
    let kinds = food::food_kinds() as f64;
    let lake = &lakes::lake_layout()[2];
    for (i, r) in rooms.iter().enumerate() {
        let x0 = (r.cx - r.rx * 0.7) * 48.0;
        let x1 = (r.cx + r.rx * 0.7) * 48.0;
        let y0 = (r.cy - r.ry * 0.6) * 48.0;
        let y1 = (r.cy + r.ry * 0.6) * 48.0;
        let mut made_lake = false;
        if r.water {
            let lx = (r.cx * 48.0 - lake.body_w * 0.5).floor();
            let ly = (r.cy * 48.0 - lake.body_h * 0.5).floor();
            if (lx / cw).floor() == ((lx + lake.body_w - 1.0) / cw).floor() {
                let amount = wld("maze_water_amount");
                lake_list.push(LakeSave {
                    cx: (lx / cw).floor() as i64,
                    seed: 700 + i as i64,
                    kind: 2,
                    x: lx - lake.body_x,
                    y: ly - lake.body_y,
                    body_x: lx,
                    body_y: ly,
                    body_w: lake.body_w,
                    body_h: lake.body_h,
                    amount,
                    full: amount,
                    connected: false,
                    flares: None,
                    flare_seq: 0,
                    ..Default::default()
                });
                made_lake = true;
            }
        }
        if !made_lake {
            let n = maze_rng_int(m, r.food_min, r.food_max);
            let mut j = 0.0;
            while j < n {
                let kind = maze_rng_int(m, 0.0, kinds - 1.0);
                let tx = maze_rng(m);
                let ty = maze_rng(m);
                let cells = food::food_cells(kind) - 1.0;
                let cell = maze_rng_int(m, 0.0, cells);
                foods.push(MazeFoodSave {
                    x: gml::lerp(x0, x1, tx),
                    y: gml::lerp(y0, y1, ty),
                    kind: kind as i64,
                    cell: cell as i64,
                    ..Default::default()
                });
                j += 1.0;
            }
        }
    }
    MazeData {
        entries,
        foods,
        lakes: lake_list,
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------------------------
// mazes

// port: maze_data
/// The built data of a maze or cluster (built and cached on first use).
pub fn maze_data(w: &mut WorldRt, part: MazePart) -> MazeData {
    match part {
        MazePart::Cluster(i) => world_cluster_build(w, i),
        MazePart::Maze(key) => world_maze_build(w, key),
    }
}

/// The maze obstacle with this key.
fn maze_obstacle(w: &mut WorldRt, key: i64) -> &mut crate::state::ObstacleSave {
    let o = w
        .params
        .obstacles
        .as_mut()
        .expect("maze obstacles exist once reached");
    o.right
        .list
        .iter_mut()
        .chain(o.left.list.iter_mut())
        .find(|p| p.key == key)
        .unwrap_or_else(|| panic!("no obstacle with key {key}"))
}

// port: world_maze_build
/// Carve the maze of obstacle `key` (cached in the obstacle's `data`, which is saved): a main
/// tunnel crossing it left to right through 2-4 waypoints, extra entrances, branches with
/// chambers, cave and solid-area hollows; the rest becomes stone down to the wall depth.
pub fn world_maze_build(w: &mut WorldRt, key: i64) -> MazeData {
    let part = maze_obstacle(w, key);
    if let Some(Some(d)) = &part.data {
        return d.clone();
    }
    let (pc1, pc2) = (part.c1.unwrap_or(0.0), part.c2.unwrap_or(0.0));
    let top = (w.params.surface_y / 48.0).floor();
    let depth = 24f64.max(gml::round(
        wld("surface_wall_depth_screens") * vis("screen_height_px") / 48.0,
    ));
    let bottom = top + depth - 1.0;
    let width = pc2 - pc1 + 1.0;
    let mut m = Maze {
        seed: w.params.seed,
        key: key as f64 * 31.0 + 7.0,
        n: 0.0,
        c1: pc1,
        c2: pc2,
        top,
        w: width as i64,
        h: (depth + 2.0) as i64,
        keep_top: 3,
        xmin: pc1 + 2.0,
        xmax: pc2 - 2.0,
        ymin: top + 4.0,
        ymax: bottom - 6.0,
        open: Vec::new(),
        inside: None,
    };
    m.open = vec![false; (m.w * m.h) as usize];
    let tmin = wld("maze_tunnel_min_cells");
    let tmax = wld("maze_tunnel_max_cells");
    let mut rooms = Vec::new();
    let mut x = m.c1 - 2.0;
    let mut y = gml::lerp(m.ymin, m.ymax, maze_rng(&mut m));
    let mut cells = Vec::new();
    let segs = maze_rng_int(&mut m, 2.0, 4.0);
    let mut s = 0.0;
    while s <= segs {
        let target = if s < segs {
            let t = (s + 1.0) / (segs + 1.0);
            let ry = maze_rng(&mut m);
            (gml::lerp(m.xmin, m.xmax, t), gml::lerp(m.ymin, m.ymax, ry))
        } else {
            let ry = maze_rng(&mut m);
            (m.c2 + 2.0, gml::lerp(m.ymin, m.ymax, ry))
        };
        let dir = gml::point_direction(x, y, target.0, target.1);
        let path = maze_tunnel(
            &mut m,
            x,
            y,
            dir,
            400.0,
            tmin,
            tmax * 1.1,
            Some(target),
            true,
        );
        if let Some(&last) = path.last() {
            x = last.0;
            y = last.1;
            cells.extend(path);
        }
        s += 1.0;
    }
    let extra = maze_rng_int(
        &mut m,
        wld("maze_extra_entrances_min"),
        wld("maze_extra_entrances_max"),
    );
    let mut e = 0.0;
    while e < extra {
        let left = maze_rng(&mut m) < 0.5;
        let ty = maze_rng(&mut m);
        let sx = if left { m.c1 - 2.0 } else { m.c2 + 2.0 };
        let sy = gml::lerp(m.ymin, m.ymax, ty);
        let dir = if left { 0.0 } else { 180.0 };
        let len = 10.0 + maze_rng(&mut m) * 25.0;
        maze_tunnel(
            &mut m,
            sx,
            sy,
            dir,
            len,
            tmin * 0.9,
            tmax * 0.85,
            None,
            true,
        );
        e += 1.0;
    }
    maze_branches(&mut m, &cells, &mut rooms);
    maze_cave_hollows(
        w,
        &mut m,
        pc1 * 48.0,
        top * 48.0,
        (pc2 + 1.0) * 48.0,
        (bottom + 1.0) * 48.0,
    );
    maze_hollows(&mut m, &mut rooms);
    let mut cut = CellSet::new();
    let mid = (m.c1 + m.c2) * 0.5;
    let half = width * 0.5 + 0.5;
    let bite = wld("surface_wall_side_bite_chance");
    let mut c = m.c1;
    while c <= m.c2 {
        let t = gml::clamp((c - mid) / half, -1.0, 1.0);
        let rows = depth - gml::round((1.0 - 0f64.max(1.0 - t * t).sqrt()) * 4.0)
            + maze_rng_int(&mut m, -1.0, 1.0);
        let mut r = 0.0;
        while r < rows {
            let edge = r > 2.0 && (c == m.c1 || c == m.c2);
            if edge && maze_rng(&mut m) < bite {
                // bitten off the side
            } else if !maze_is_open(&m, c, top + r) {
                cut.insert((c as i64, (top + r) as i64));
            }
            r += 1.0;
        }
        c += 1.0;
    }
    let (c0, c1) = (m.c1 as i64 - 1, m.c2 as i64 + 1);
    let d = maze_finish(
        w,
        &mut m,
        &mut cut,
        &rooms,
        c0,
        c1,
        top as i64 - 1,
        bottom as i64 + 3,
        true,
    );
    maze_obstacle(w, key).data = Some(Some(d.clone()));
    d
}

// ---------------------------------------------------------------------------------------------
// rock clusters

// port: world_rock_clusters
/// The rock clusters of this world: per tier (clay between the dense and deep lines, deep
/// between the deep line and the ocean margin) one every `rock_cluster_every_x` screens on each
/// side of the start, jittered. Cached in the world.
pub fn world_rock_clusters(w: &mut WorldRt) -> &[RockCluster] {
    if w.rock_clusters.is_none() {
        let list = rock_clusters_build(w);
        w.rock_clusters = Some(list);
    }
    w.rock_clusters.as_deref().unwrap()
}

fn rock_clusters_build(w: &WorldRt) -> Vec<RockCluster> {
    let mut out = Vec::new();
    if wld("rock_clusters_enabled") <= 0.0 {
        return out;
    }
    let sw = vis("screen_width_px");
    let tiers = [
        (
            "clay",
            w.dense_y(),
            w.deep_y(),
            501.0,
            wld("rock_cluster_every_x"),
        ),
        (
            "deep",
            w.deep_y(),
            w.ocean_y() - wld("ocean_clear_margin_px"),
            502.0,
            wld("rock_cluster_deep_every_x"),
        ),
    ];
    let minc = wld("rock_cluster_min_cells");
    let maxc = wld("rock_cluster_max_cells");
    let per_side = wld("rock_cluster_per_side");
    for (t, &(id, top, bottom, salt, every)) in tiers.iter().enumerate() {
        let mut s = -per_side;
        while s <= per_side {
            let r = gml::lerp(minc, maxc + 1.0, w.hash01(s, 1.0, salt)).floor();
            let jitter =
                wld("rock_cluster_jitter_x") * every / 0.1f64.max(wld("rock_cluster_every_x"));
            let cx =
                w.params.start_x + sw * (s * every + (w.hash01(s, 2.0, salt) - 0.5) * 2.0 * jitter);
            let pad = (r + 2.0) * 48.0;
            let ymin = top + pad + wld("caves_margin_px");
            let ymax = bottom - pad - wld("caves_margin_px");
            if ymax >= ymin {
                let cy = gml::lerp(ymin, ymax, w.hash01(s, 3.0, salt));
                let col = (cx / 48.0).floor();
                let row = (cy / 48.0).floor();
                out.push(RockCluster {
                    id: format!("{id}_{}", s as i64),
                    tier: id.to_string(),
                    key: 90000.0 + t as f64 * 1000.0 + s,
                    col,
                    row,
                    r,
                    c1: col - r - 3.0,
                    c2: col + r + 3.0,
                    x1: (col - r - 3.0) * 48.0,
                    y1: (row - r - 3.0) * 48.0,
                    x2: (col + r + 4.0) * 48.0,
                    y2: (row + r + 4.0) * 48.0,
                    data: None,
                });
            }
            s += 1.0;
        }
    }
    out
}

// port: world_cluster_build
/// Carve rock cluster `idx` of `world_rock_clusters` (cached): a wobbly round body crossed by
/// a tunnel through a middle point, extra entrances, branches and hollows.
pub fn world_cluster_build(w: &mut WorldRt, idx: usize) -> MazeData {
    let cl = world_rock_clusters(w)[idx].clone();
    if let Some(d) = cl.data {
        return d;
    }
    let n = cl.r * 2.0 + 7.0;
    let ni = n as i64;
    let mut m = Maze {
        seed: w.params.seed,
        key: cl.key * 31.0 + 11.0,
        n: 0.0,
        c1: cl.col - cl.r - 3.0,
        c2: cl.col + cl.r + 3.0,
        top: cl.row - cl.r - 3.0,
        w: ni,
        h: ni,
        keep_top: 0,
        xmin: cl.col - cl.r - 1.0,
        xmax: cl.col + cl.r + 1.0,
        ymin: cl.row - cl.r - 1.0,
        ymax: cl.row + cl.r + 1.0,
        open: vec![false; (ni * ni) as usize],
        inside: None,
    };
    let mut inside = vec![false; (ni * ni) as usize];
    let phase1 = maze_rng(&mut m) * stones::PHASE_RANGE;
    let phase2 = maze_rng(&mut m) * stones::PHASE_RANGE;
    for j in 0..ni {
        for i in 0..ni {
            let dx = m.c1 + i as f64 + 0.5 - (cl.col + 0.5);
            let dy = m.top + j as f64 + 0.5 - (cl.row + 0.5);
            let a = dy.atan2(dx);
            let rr =
                cl.r * (0.07 * (5.0 * a + phase2).sin() + 0.12 * (3.0 * a + phase1).sin() + 1.0);
            inside[(j * ni + i) as usize] = dx * dx + dy * dy <= rr * rr;
        }
    }
    m.inside = Some(inside);
    let tmin = wld("maze_tunnel_min_cells");
    let tmax = wld("maze_tunnel_max_cells");
    let mut rooms = Vec::new();
    let cx = cl.col + 0.5;
    let cy = cl.row + 0.5;
    let a_in = maze_rng(&mut m) * 360.0;
    let a_out = a_in + 180.0 + (maze_rng(&mut m) - 0.5) * 80.0;
    let p_in = (
        cx + gml::lengthdir_x(cl.r + 2.0, a_in),
        cy + gml::lengthdir_y(cl.r + 2.0, a_in),
    );
    let mx = cx + (maze_rng(&mut m) - 0.5) * cl.r * 0.6;
    let p_mid = (mx, cy + (maze_rng(&mut m) - 0.5) * cl.r * 0.6);
    let p_out = (
        cx + gml::lengthdir_x(cl.r + 2.0, a_out),
        cy + gml::lengthdir_y(cl.r + 2.0, a_out),
    );
    let dir = gml::point_direction(p_in.0, p_in.1, p_mid.0, p_mid.1);
    let mut path = maze_tunnel(
        &mut m,
        p_in.0,
        p_in.1,
        dir,
        200.0,
        tmin,
        tmax * 1.2,
        Some(p_mid),
        true,
    );
    let last = path.last().copied().unwrap_or(p_mid);
    let dir = gml::point_direction(last.0, last.1, p_out.0, p_out.1);
    let more = maze_tunnel(
        &mut m,
        last.0,
        last.1,
        dir,
        200.0,
        tmin,
        tmax * 1.2,
        Some(p_out),
        true,
    );
    path.extend(more);
    let extra = maze_rng_int(
        &mut m,
        wld("maze_extra_entrances_min"),
        wld("maze_extra_entrances_max"),
    );
    let mut e = 0.0;
    while e < extra {
        let a = maze_rng(&mut m) * 360.0;
        let sx = cx + gml::lengthdir_x(cl.r + 2.0, a);
        let sy = cy + gml::lengthdir_y(cl.r + 2.0, a);
        let dir = a + 180.0 + (maze_rng(&mut m) - 0.5) * 60.0;
        let len = cl.r * (0.5 + maze_rng(&mut m) * 0.6);
        maze_tunnel(
            &mut m,
            sx,
            sy,
            dir,
            len,
            tmin * 0.9,
            tmax * 0.85,
            None,
            true,
        );
        e += 1.0;
    }
    maze_branches(&mut m, &path, &mut rooms);
    maze_cave_hollows(w, &mut m, cl.x1, cl.y1, cl.x2, cl.y2);
    maze_hollows(&mut m, &mut rooms);
    let mut cut = CellSet::new();
    let inside = m.inside.as_ref().unwrap();
    for j in 0..ni {
        for i in 0..ni {
            let k = (j * ni + i) as usize;
            if inside[k] && !m.open[k] {
                cut.insert((m.c1 as i64 + i, m.top as i64 + j));
            }
        }
    }
    let (c0, c1, r0, r1) = (
        m.c1 as i64 - 1,
        m.c2 as i64 + 1,
        m.top as i64 - 1,
        m.top as i64 + ni + 1,
    );
    let d = maze_finish(w, &mut m, &mut cut, &rooms, c0, c1, r0, r1, false);
    w.rock_clusters.as_mut().unwrap()[idx].data = Some(d.clone());
    d
}

/// Rock clusters passing `hit` that the chunk has not painted yet (marking them done).
fn clusters_to_paint(
    w: &mut WorldRt,
    ch: &mut WorldChunkSave,
    hit: impl Fn(&RockCluster) -> bool,
) -> Vec<usize> {
    let todo: Vec<(usize, String)> = world_rock_clusters(w)
        .iter()
        .enumerate()
        .filter(|(_, cl)| hit(cl))
        .map(|(i, cl)| (i, cl.id.clone()))
        .collect();
    let done = ch.clusters_done.get_or_insert_with(BTreeMap::new);
    let mut paint = Vec::new();
    for (i, id) in todo {
        if gml_truthy(done.get(&id)) {
            continue;
        }
        done.insert(id, Value::Bool(true));
        paint.push(i);
    }
    paint
}

// port: world_paint_clusters_late
/// Draw-time pass (called by the renderer for visible chunks): paint the clusters reaching into
/// the chunk's generated rows that its layers have not painted yet.
pub fn world_paint_clusters_late(w: &mut WorldRt, cx: i64) {
    let Some(mut ch) = w.chunks.remove(&cx) else {
        return;
    };
    let cw = w.params.chunk_w as f64;
    let x0 = cx as f64 * cw;
    let x1 = x0 + cw;
    let y1 = w.params.surface_y + (ch.rows * w.params.tile) as f64;
    let paint = clusters_to_paint(w, &mut ch, |cl| {
        !(cl.x2 <= x0 || cl.x1 >= x1 || cl.y1 >= y1)
    });
    // the chunk stays in the world while the clusters are built, as in LÖVE
    w.chunks.insert(cx, ch);
    if paint.is_empty() {
        return;
    }
    let data: Vec<MazeData> = paint
        .iter()
        .map(|&i| maze_data(w, MazePart::Cluster(i)))
        .collect();
    let cw = w.params.chunk_w as f64;
    let ch = w.chunks.get_mut(&cx).unwrap();
    for d in &data {
        paint_data(ch, d, cw);
    }
    w.dirty_chunks.insert(cx);
}

/// `world_paint_clusters(ch, layer)` (see `stones::world_paint_clusters`).
pub(crate) fn paint_clusters_in_layer(w: &mut WorldRt, ch: &mut WorldChunkSave, layer: i64) {
    let cw = w.params.chunk_w as f64;
    let x0 = ch.cx as f64 * cw;
    let x1 = x0 + cw;
    let y0 = w.layer_top(layer);
    let y1 = w.layer_bottom(layer);
    let paint = clusters_to_paint(w, ch, |cl| {
        !(cl.x2 <= x0 || cl.x1 >= x1 || cl.y2 <= y0 || cl.y1 >= y1)
    });
    for i in paint {
        paint_part(w, ch, MazePart::Cluster(i));
    }
}

// port: world_paint_maze_part
/// Paint the part of a maze or cluster inside this chunk: its stone cells (a cell crossing the
/// chunk border goes into both chunks), its foods and its lakes.
pub fn paint_part(w: &mut WorldRt, ch: &mut WorldChunkSave, part: MazePart) {
    let d = maze_data(w, part);
    paint_data(ch, &d, w.params.chunk_w as f64);
}

/// `world_paint_maze_part` after `maze_data`.
fn paint_data(ch: &mut WorldChunkSave, d: &MazeData, cw: f64) {
    let x0 = ch.cx as f64 * cw;
    let x1 = x0 + cw;
    for e in &d.entries {
        let ex = (e.col * 48) as f64;
        if ex < x1 && ex + 48.0 > x0 {
            ch.stone_cells.push(e.clone());
            ch.stone_map
                .insert(stones::stone_cell_key(e.col, e.row), e.clone());
        }
    }
    for f in &d.foods {
        if f.x >= x0 && f.x < x1 {
            ch.foods.push(FoodSave {
                x: f.x,
                y: f.y,
                kind: f.kind,
                cell: f.cell,
                collected: false,
                maze: Some(true),
                ..Default::default()
            });
        }
    }
    for lk in &d.lakes {
        if lk.cx == ch.cx {
            ch.lakes.push(lk.clone());
        }
    }
}
