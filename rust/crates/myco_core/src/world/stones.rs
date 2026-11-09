//! Stones (scr_stones, Task 7): loose stones that hyphae shove around, and stone deposits made
//! of 48 px cells (`chunk.stone_cells` / `chunk.stone_map`) painted from blobs, surface walls,
//! mazes and rock clusters. A deposit cell is solid where its `stone_tile` atlas tile has opaque
//! pixels; those spans, and each loose-stone kind's size, come from the sprites (R7: the render
//! task builds the sprites, the data is embedded in `stone_art.json`, captured from LÖVE).

use super::{caves, gen, lakes, WorldRt};
use crate::balance::wld;
use crate::gml;
use crate::sim::Sim;
use crate::state::{StoneCellSave, StoneSave, WorldChunkSave};
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::OnceLock;

/// Deposit cell size in px (stone cells and `stone_tile` tiles are 48 px; atlas tiles are 64).
pub const CELL: f64 = 48.0;

/// The original's phase range for wobbles: 6.283, not exactly 2π.
#[allow(clippy::approx_constant)]
pub const PHASE_RANGE: f64 = 6.283;

/// A set of stone cells by (col, row): the GML structs keyed by `stone_cell_key`.
pub type CellSet = HashSet<(i64, i64)>;

// ---------------------------------------------------------------------------------------------
// pixel-derived data

/// One loose-stone kind (`global.stone_kinds[k]`, built by `stones_init` from the `stones`
/// sheet): its column in the sheet, centre, bounding radius and collision body radius.
#[derive(Clone, Debug, Deserialize)]
pub struct StoneKind {
    pub left: f64,
    pub w: f64,
    pub ox: f64,
    pub oy: f64,
    pub radius: f64,
    pub body: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StoneArt {
    /// `global.stone_reach`: the largest kind radius
    pub reach: f64,
    pub kinds: Vec<StoneKind>,
    /// `global.stone_tiles`: per atlas tile (row-major 7x7), per pixel row (48), solid spans
    /// `[x0, x1, x0, x1, ...]` inclusive
    pub tiles: Vec<Vec<Vec<f64>>>,
}

/// The embedded stone data (golden `stones_art`; tests/stones.rs checks the copy).
pub fn stone_art() -> &'static StoneArt {
    static ART: OnceLock<StoneArt> = OnceLock::new();
    ART.get_or_init(|| {
        serde_json::from_str(include_str!("stone_art.json")).expect("stone_art.json is valid")
    })
}

// port: stones_ready
/// The loose-stone kinds exist. Always true here: the data is embedded (in LÖVE it appears with
/// the first `world_draw`).
pub fn stones_ready() -> bool {
    true
}

// port: stone_tiles_ready
/// The stone tile spans exist (embedded, so always).
pub fn stone_tiles_ready() -> bool {
    true
}

/// A column run of the `stones` sheet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SheetSlice {
    pub left: f64,
    pub width: f64,
}

// port: stones_slice_sheet
/// Split the `stones` sheet into its seven stones: runs of columns with any non-transparent
/// pixel (`alpha(x, y)`); anything other than seven runs falls back to equal sevenths. Used by
/// the render task's `stones_init`.
pub fn stones_slice_sheet(
    alpha: impl Fn(usize, usize) -> u8,
    w: usize,
    h: usize,
) -> Vec<SheetSlice> {
    let mut slices = Vec::new();
    let mut start: Option<usize> = None;
    for x in 0..w {
        let filled = (0..h).any(|y| alpha(x, y) > 0);
        if filled {
            if start.is_none() {
                start = Some(x);
            }
        } else if let Some(s) = start.take() {
            slices.push(SheetSlice {
                left: s as f64,
                width: (x - s) as f64,
            });
        }
    }
    if let Some(s) = start {
        slices.push(SheetSlice {
            left: s as f64,
            width: (w - s) as f64,
        });
    }
    if slices.len() != 7 {
        let wf = w as f64;
        slices = (0..7)
            .map(|i| {
                let l = (i as f64 * wf / 7.0).floor();
                SheetSlice {
                    left: l,
                    width: ((i + 1) as f64 * wf / 7.0).floor() - l,
                }
            })
            .collect();
    }
    slices
}

/// `global.stone_reach` (the largest stone radius).
pub fn stone_reach(_w: &WorldRt) -> f64 {
    stone_art().reach
}

// ---------------------------------------------------------------------------------------------
// loose stones

impl WorldRt {
    /// Index of the stone with this runtime id in chunk `cx`.
    fn stone_index(&self, cx: i64, uid: u64) -> Option<usize> {
        self.chunks
            .get(&cx)?
            .stones
            .iter()
            .position(|s| s.uid == uid)
    }

    /// Give every stone a runtime id (after loading a save).
    pub(crate) fn assign_stone_uids(&mut self) {
        let mut keys: Vec<i64> = self.chunks.keys().copied().collect();
        keys.sort_by_key(|cx| super::chunk_key(*cx));
        for cx in keys {
            let n = self.chunks[&cx].stones.len();
            for k in 0..n {
                self.next_stone_uid += 1;
                let uid = self.next_stone_uid;
                self.chunks.get_mut(&cx).unwrap().stones[k].uid = uid;
            }
        }
    }
}

// port: stones_wake
/// Start moving stone `k` of chunk `cx` (append it to `global.stones_moving`).
pub fn stones_wake(w: &mut WorldRt, cx: i64, k: usize) {
    let s = &mut w.chunks.get_mut(&cx).unwrap().stones[k];
    if s.moving {
        return;
    }
    s.moving = true;
    let uid = s.uid;
    w.stones_moving.push((cx, uid));
}

// port: stones_shove
/// A hypha passing (x, y) in direction `dir` pushes nearby stones aside, with spin and a
/// random wobble (one `random_range` per stone hit).
pub fn stones_shove(sim: &mut Sim, x: f64, y: f64, dir: f64, force: f64) {
    if !stones_ready() {
        return;
    }
    let art = stone_art();
    let reach = wld("stone_shove_reach");
    let r = art.reach + reach;
    let dx = gml::lengthdir_x(1.0, dir);
    let dy = gml::lengthdir_y(1.0, dir);
    let px = -dy;
    let py = dx;
    let cw = sim.world.params.chunk_w as f64;
    let c0 = ((x - r) / cw).floor() as i64;
    let c1 = ((x + r) / cw).floor() as i64;
    let lead = wld("stone_shove_lead");
    let spin = wld("stone_shove_spin");
    let wobble = wld("stone_shove_wobble");
    for c in c0..=c1 {
        sim.world.get_chunk(c);
        let n = sim.world.chunks[&c].stones.len();
        for k in 0..n {
            let s = &mut sim.world.chunks.get_mut(&c).unwrap().stones[k];
            let ox = s.x - x;
            let oy = s.y - y;
            let d = gml::point_distance(0.0, 0.0, ox, oy);
            let rr = art.kinds[s.kind as usize].radius + reach;
            if d > rr {
                continue;
            }
            let lat = ox * px + oy * py;
            let side = if lat.abs() <= 1.0 {
                s.side
            } else {
                gml::sign(lat)
            };
            let f = force * (1.0 - d / rr);
            let mut nx = px * side + dx * lead;
            let mut ny = py * side + dy * lead;
            let l = gml::point_distance(0.0, 0.0, nx, ny);
            if l > 0.001 {
                nx /= l;
                ny /= l;
            }
            s.vx += nx * f;
            s.vy += ny * f;
            s.spin += (ox * ny - oy * nx) * f * spin;
            s.spin += sim.rng.random_range(-1.0, 1.0) * f * wobble;
            stones_wake(&mut sim.world, c, k);
        }
    }
}

// port: stones_separate
/// Stone `si` of chunk `cx` pushes the stones it overlaps (waking them) and is pushed back.
pub fn stones_separate(w: &mut WorldRt, cx: i64, si: usize) {
    let art = stone_art();
    let pass = wld("stone_pass_on");
    let spin = wld("stone_shove_spin");
    let n = w.chunks[&cx].stones.len();
    let (sx, sy, sbody) = {
        let s = &w.chunks[&cx].stones[si];
        (s.x, s.y, art.kinds[s.kind as usize].body)
    };
    for k in 0..n {
        if k == si {
            continue;
        }
        let list = &mut w.chunks.get_mut(&cx).unwrap().stones;
        let (svx, svy) = (list[si].vx, list[si].vy);
        let o = &mut list[k];
        let dx = o.x - sx;
        let dy = o.y - sy;
        let min_d = art.kinds[o.kind as usize].body + sbody;
        let d = gml::point_distance(0.0, 0.0, dx, dy);
        if d >= min_d || d <= 0.0 {
            continue;
        }
        let push = (min_d - d) / d;
        o.vx += dx * push * pass;
        o.vy += dy * push * pass;
        o.spin += push * (dx * svy - dy * svx) * spin;
        let s = &mut list[si];
        s.vx -= dx * push * pass;
        s.vy -= dy * push * pass;
        stones_wake(w, cx, k);
    }
}

// port: stones_step
/// Move the stones in `global.stones_moving` one frame (newest first): drag, stop when slow,
/// stay inside their chunk and above the water row, push neighbours.
pub fn stones_step(sim: &mut Sim) {
    if !stones_ready() {
        return;
    }
    let w = &mut sim.world;
    let drag = wld("stone_drag");
    let tile = w.params.tile as f64;
    let ylo = w.params.surface_y + tile * 0.5;
    let yhi = w.params.surface_y + w.params.water_row as f64 * tile;
    let cw = w.params.chunk_w as f64;
    let mut i = w.stones_moving.len() as i64 - 1;
    while i >= 0 {
        let (cx, uid) = w.stones_moving[i as usize];
        // a stone removed from its chunk (Lua keeps moving the orphaned struct, unseen)
        let Some(k) = w.stone_index(cx, uid) else {
            w.stones_moving.remove(i as usize);
            i -= 1;
            continue;
        };
        let s = &mut w.chunks.get_mut(&cx).unwrap().stones[k];
        s.vx *= drag;
        s.vy *= drag;
        s.spin *= drag;
        if gml::point_distance(0.0, 0.0, s.vx, s.vy) < 0.04 && s.spin.abs() < 0.12 {
            s.vx = 0.0;
            s.vy = 0.0;
            s.spin = 0.0;
            s.moving = false;
            w.stones_moving.remove(i as usize);
        } else {
            s.x += s.vx;
            s.y += s.vy;
            s.angle += s.spin;
            let lo = s.cx as f64 * cw + 4.0;
            let hi = lo + cw - 8.0;
            if s.x < lo || s.x > hi {
                s.x = gml::clamp(s.x, lo, hi);
                s.vx = 0.0;
            }
            if s.y < ylo || s.y > yhi {
                s.y = gml::clamp(s.y, ylo, yhi);
                s.vy = 0.0;
            }
            let scx = s.cx;
            w.get_chunk(scx);
            stones_separate(w, scx, k);
        }
        i -= 1;
    }
}

// port: world_paint_stones
/// Scatter loose stones on a jittered grid through this layer of the chunk, away from the
/// start column, water cells, caves, lakes and deposits.
pub fn world_paint_stones(w: &mut WorldRt, chunk: &mut WorldChunkSave, layer: i64) {
    let p = w.params.clone();
    let tile = p.tile as f64;
    let cw = p.chunk_w as f64;
    let key = gen::layer_key(chunk.cx, layer);
    let x0 = chunk.cx as f64 * cw;
    let spacing = 48f64.max(wld("stone_spacing_px"));
    let band = (p.layer_rows * p.tile) as f64;
    let y0 = p.surface_y + layer as f64 * band;
    let nx = 1f64.max(gml::round(cw / spacing));
    let ny = 1f64.max(gml::round(band / spacing));
    let cell_w = cw / nx;
    let cell_h = band / ny;
    let chance = wld("stone_chance");
    let tx0 = (p.start_x / tile).floor();
    let mut j = 0.0;
    while j < ny {
        let mut i = 0.0;
        while i < nx {
            let idx = j * nx + i;
            if w.hash01(key, idx, 61.0) < chance {
                let px = x0 + (i + 0.15 + w.hash01(key, idx, 62.0) * 0.7) * cell_w;
                let py = y0 + (j + 0.15 + w.hash01(key, idx, 63.0) * 0.7) * cell_h;
                let tx = (px / tile).floor();
                let ty = ((py - p.surface_y) / tile).floor();
                let skip = ty < 1.0
                    || (ty <= 5.0 && (tx - tx0).abs() <= 3.0)
                    || w.chunk_get(chunk, tx as i64, ty as i64) == 3
                    || caves::world_cave_hits(w, px, py, px, py, Some(24.0))
                    || lakes::world_lake_covers(chunk, px, py)
                    || world_stone_covers(w, chunk, px, py);
                if !skip {
                    let kind = 6f64.min((w.hash01(key, idx, 64.0) * 7.0).floor());
                    let side = if w.hash01(key, idx, 65.0) >= 0.5 {
                        1.0
                    } else {
                        -1.0
                    };
                    w.next_stone_uid += 1;
                    chunk.stones.push(StoneSave {
                        cx: chunk.cx,
                        kind: kind as i64,
                        x: px,
                        y: py,
                        side,
                        uid: w.next_stone_uid,
                        ..Default::default()
                    });
                }
            }
            i += 1.0;
        }
        j += 1.0;
    }
}

// ---------------------------------------------------------------------------------------------
// stone tiles

type Tiles = &'static [(i64, i64)];

// port: stone_role_tiles
/// Atlas tiles (col, row) that fit a deposit cell with these exposed edges; `depth` 0/1
/// picks the outer or inner row of top/bottom edges.
pub fn stone_role_tiles(up: bool, down: bool, left: bool, right: bool, depth: f64) -> Tiles {
    const FILL: Tiles = &[(3, 3)];
    const UP: [Tiles; 2] = [&[(2, 0), (3, 0), (4, 0)], &[(2, 1), (3, 1), (4, 1)]];
    const DOWN: [Tiles; 2] = [&[(2, 6), (3, 6), (4, 6)], &[(2, 5), (3, 5), (4, 5)]];
    const LEFT: Tiles = &[(1, 2), (1, 3), (1, 4)];
    const RIGHT: Tiles = &[(5, 2), (5, 3), (5, 4)];
    const UP_LEFT: [Tiles; 2] = [&[(0, 0), (1, 0)], &[(0, 1), (1, 1)]];
    const UP_RIGHT: [Tiles; 2] = [&[(5, 0), (6, 0)], &[(5, 1), (6, 1)]];
    const DOWN_LEFT: [Tiles; 2] = [&[(0, 6), (1, 6)], &[(0, 5), (1, 5)]];
    const DOWN_RIGHT: [Tiles; 2] = [&[(5, 6), (6, 6)], &[(5, 5), (6, 5)]];
    let di = gml::clamp(depth, 0.0, 1.0).floor() as usize;
    if up && left {
        return UP_LEFT[di];
    }
    if up && right {
        return UP_RIGHT[di];
    }
    if down && left {
        return DOWN_LEFT[di];
    }
    if down && right {
        return DOWN_RIGHT[di];
    }
    if up {
        return UP[di];
    }
    if down {
        return DOWN[di];
    }
    if left {
        return LEFT;
    }
    if right {
        return RIGHT;
    }
    FILL
}

// port: stone_inner_corner_tile
/// The inner-corner atlas tile for a missing diagonal neighbour.
pub fn stone_inner_corner_tile(up: bool, left: bool) -> (i64, i64) {
    match (up, left) {
        (true, true) => (2, 2),
        (true, false) => (4, 2),
        (false, true) => (2, 4),
        (false, false) => (4, 4),
    }
}

// port: stone_tile_solid_at
/// Atlas tile (tx, ty) has an opaque pixel at local (x, y).
pub fn stone_tile_solid_at(tx: i64, ty: i64, x: f64, y: f64) -> bool {
    if !(0.0..48.0).contains(&y) {
        return false;
    }
    let row = &stone_art().tiles[(ty * 7 + tx) as usize][y.floor() as usize];
    row.chunks(2).any(|p| x >= p[0] && x <= p[1])
}

// port: stone_cell_key
/// `"col:row"`, the key of `chunk.stone_map`.
pub fn stone_cell_key(cx: i64, cy: i64) -> String {
    format!("{cx}:{cy}")
}

// port: stone_cell_at
/// The deposit cell covering world (x, y) in this chunk.
pub fn stone_cell_at(chunk: &WorldChunkSave, x: f64, y: f64) -> Option<&StoneCellSave> {
    if chunk.stone_cells.is_empty() {
        return None;
    }
    chunk.stone_map.get(&stone_cell_key(
        (x / CELL).floor() as i64,
        (y / CELL).floor() as i64,
    ))
}

// port: stone_solid_at
/// A deposit cell's tile is opaque at world (x, y).
pub fn stone_solid_at(_w: &WorldRt, chunk: &WorldChunkSave, x: f64, y: f64) -> bool {
    let Some(c) = stone_cell_at(chunk, x, y) else {
        return false;
    };
    if !stone_tiles_ready() {
        return true;
    }
    let lx = x.floor() - (c.col * 48) as f64;
    let ly = y.floor() - (c.row * 48) as f64;
    stone_tile_solid_at(c.tc, c.tr, lx, ly)
}

// port: world_stone_covers
/// A deposit cell covers world (x, y) in this chunk.
pub fn world_stone_covers(_w: &WorldRt, chunk: &WorldChunkSave, x: f64, y: f64) -> bool {
    stone_cell_at(chunk, x, y).is_some()
}

// port: stone_pick_tile
/// One of `list`, by hash.
pub fn stone_pick_tile(w: &WorldRt, a: f64, b: f64, list: Tiles, salt: f64) -> (i64, i64) {
    let n = list.len() as f64;
    let i = (n - 1.0).min((w.hash01(a, b, salt) * n).floor());
    list[i as usize]
}

// port: stone_tile_for
/// The atlas tile of cell (col, row) given which neighbours are stone (`has`).
pub fn stone_tile_for(
    w: &WorldRt,
    has: impl Fn(i64, i64) -> bool,
    col: i64,
    row: i64,
    depth: f64,
) -> (i64, i64) {
    let up = !has(col, row - 1);
    let down = !has(col, row + 1);
    let left = !has(col - 1, row);
    let right = !has(col + 1, row);
    let tiles = stone_role_tiles(up, down, left, right, depth);
    let mut tile = stone_pick_tile(w, col as f64, row as f64, tiles, 41.0);
    if !up && !down && !left && !right {
        let mut corners = Vec::new();
        if !has(col - 1, row - 1) {
            corners.push((true, true));
        }
        if !has(col + 1, row - 1) {
            corners.push((true, false));
        }
        if !has(col - 1, row + 1) {
            corners.push((false, true));
        }
        if !has(col + 1, row + 1) {
            corners.push((false, false));
        }
        let n = corners.len() as f64;
        if n > 0.0 {
            let idx = (n - 1.0).min((w.hash01(col as f64, row as f64, 42.0) * n).floor());
            let c = corners[idx as usize];
            tile = stone_inner_corner_tile(c.0, c.1);
        }
    }
    tile
}

// ---------------------------------------------------------------------------------------------
// blobs

/// A finished blob (`___struct___219__stone_blob_finish`): its cells in order and the key set
/// used to pick tiles (may hold extra sealing keys above the top row).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StoneBlob {
    pub keys: CellSet,
    pub list: Vec<(i64, i64)>,
}

// port: stone_blob_cells
/// An irregular blob of `count` overlapping ellipses around (cx, cy), nibbled and grown at the
/// edges, then smoothed and reduced to its largest piece.
pub fn stone_blob_cells(w: &WorldRt, cx: i64, cy: i64, rx: i64, ry: i64, count: i64) -> StoneBlob {
    let mut grid = CellSet::new();
    let (x0, x1, y0, y1) = (cx - rx - 1, cx + rx + 1, cy - ry - 1, cy + ry + 1);
    let (cxf, rxf, ryf) = (cx as f64, rx as f64, ry as f64);
    for b in 0..count {
        let seed = (cy * 8 + b) as f64;
        let brx = 1f64.max(gml::round(rxf * (0.3 + 0.55 * w.hash01(cxf, seed, 56.0))));
        let bry = 1f64.max(gml::round(ryf * (0.3 + 0.55 * w.hash01(cxf, seed, 57.0))));
        let ox = gml::round((w.hash01(cxf, seed, 58.0) * 2.0 - 1.0) * (rxf - brx)) as i64;
        let oy = gml::round((w.hash01(cxf, seed, 59.0) * 2.0 - 1.0) * (ryf - bry)) as i64;
        let (bx, by) = (brx as i64, bry as i64);
        for dy in -by..=by {
            for dx in -bx..=bx {
                let nx = dx as f64 / (brx + 0.5);
                let ny = dy as f64 / (bry + 0.5);
                if nx * nx + ny * ny > 1.0 {
                    continue;
                }
                grid.insert((cx + ox + dx, cy + oy + dy));
            }
        }
    }
    let nibble = wld("stone_deposit_nibble_chance");
    let grow = wld("stone_deposit_grow_chance");
    let mut drop = Vec::new();
    let mut add = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let here = grid.contains(&(x, y));
            let up = grid.contains(&(x, y - 1));
            let down = grid.contains(&(x, y + 1));
            let left = grid.contains(&(x - 1, y));
            let right = grid.contains(&(x + 1, y));
            if here {
                if !(up && down && left && right) && w.hash01(x as f64, y as f64, 62.0) < nibble {
                    drop.push((x, y));
                }
            } else if (x - cx).abs() <= rx
                && (y - cy).abs() <= ry
                && (up || down || left || right)
                && w.hash01(x as f64, y as f64, 63.0) < grow
            {
                add.push((x, y));
            }
        }
    }
    for k in drop {
        grid.remove(&k);
    }
    for k in add {
        grid.insert(k);
    }
    stone_blob_finish(&mut grid, x0, x1, y0, y1, 0)
}

// port: stone_blob_finish
/// Fill one-cell holes and drop one-cell-thin cells until stable (in place), then keep the
/// largest piece (`protect` 0) or every piece of at least `protect` cells.
pub fn stone_blob_finish(
    grid: &mut CellSet,
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
    protect: i64,
) -> StoneBlob {
    let mut changed = true;
    while changed {
        changed = false;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let here = grid.contains(&(x, y));
                let up = grid.contains(&(x, y - 1));
                let down = grid.contains(&(x, y + 1));
                let left = grid.contains(&(x - 1, y));
                let right = grid.contains(&(x + 1, y));
                if !here {
                    if up && down && left && right {
                        grid.insert((x, y));
                        changed = true;
                    }
                } else if (!left && !right) || (!up && !down) {
                    grid.remove(&(x, y));
                    changed = true;
                }
            }
        }
    }
    let list = stone_blob_largest_piece(grid, x0, x1, y0, y1, protect);
    let keys = list.iter().copied().collect();
    StoneBlob { keys, list }
}

// port: stone_blob_largest_piece
/// Connected pieces (4-neighbour flood fill, scan order); the largest one, or with `protect`
/// > 0 all pieces of at least that many cells, concatenated.
pub fn stone_blob_largest_piece(
    grid: &CellSet,
    x0: i64,
    x1: i64,
    y0: i64,
    y1: i64,
    protect: i64,
) -> Vec<(i64, i64)> {
    const DIRS: [(i64, i64); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut seen = CellSet::new();
    let mut best: Vec<(i64, i64)> = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            if !grid.contains(&(x, y)) || seen.contains(&(x, y)) {
                continue;
            }
            let mut piece = Vec::new();
            let mut stack = vec![(x, y)];
            seen.insert((x, y));
            while let Some(c) = stack.pop() {
                piece.push(c);
                for d in DIRS {
                    let n = (c.0 + d.0, c.1 + d.1);
                    if !grid.contains(&n) || seen.contains(&n) {
                        continue;
                    }
                    seen.insert(n);
                    stack.push(n);
                }
            }
            if protect <= 0 {
                if piece.len() > best.len() {
                    best = piece;
                }
            } else if piece.len() as i64 >= protect {
                best.extend(piece);
            }
        }
    }
    best
}

// port: stone_blob_fits
/// The blob keeps clear of the start tree, caves, the chunk's lakes and its other deposits.
pub fn stone_blob_fits(w: &mut WorldRt, chunk: &WorldChunkSave, blob: &StoneBlob) -> bool {
    let p = &w.params;
    let clear = wld("stone_water_clearance_px");
    let gap = wld("stone_deposit_gap_cells") * 48.0;
    let tx0 = p.start_x - wld("start_tree_clear_px");
    let tx1 = p.start_x + wld("start_tree_clear_px");
    let ty1 = p.surface_y + 6.0 * p.tile as f64;
    let g = (gap / 48.0).floor() as i64;
    for &(col, row) in &blob.list {
        let x0 = (col * 48) as f64;
        let y0 = (row * 48) as f64;
        let x1 = x0 + 48.0;
        let y1 = y0 + 48.0;
        if x1 > tx0 && x0 < tx1 && y0 < ty1 {
            return false;
        }
        if caves::world_cave_hits(w, x0, y0, x1, y1, None) {
            return false;
        }
        for l in &chunk.lakes {
            if x1 + clear > l.body_x
                && x0 - clear < l.body_x + l.body_w
                && y1 + clear > l.body_y
                && y0 - clear < l.body_y + l.body_h
            {
                return false;
            }
        }
        for dy in -g..=g {
            for dx in -g..=g {
                if chunk
                    .stone_map
                    .contains_key(&stone_cell_key(col + dx, row + dy))
                {
                    return false;
                }
            }
        }
    }
    true
}

// port: anon__32538__stone_dress_blob__scr_stones
/// `stone_dress_blob`'s `is_stone`: the cell is in the blob's key set.
fn blob_has(keys: &CellSet, x: i64, y: i64) -> bool {
    keys.contains(&(x, y))
}

// port: stone_dress_blob
/// Turn the blob into deposit cells with their tiles, appended to `cells` and `map` (a chunk's
/// `stone_cells` / `stone_map`, or a maze's entry list).
pub fn stone_dress_blob(
    w: &WorldRt,
    cells: &mut Vec<StoneCellSave>,
    map: &mut BTreeMap<String, StoneCellSave>,
    blob: &StoneBlob,
    band: i64,
) {
    let dy = w.deep_y();
    let deep = blob
        .list
        .iter()
        .any(|&(_, row)| ((row + 1) * 48) as f64 > dy);
    for &(col, row) in &blob.list {
        let t = stone_tile_for(w, |x, y| blob_has(&blob.keys, x, y), col, row, band as f64);
        let cell = StoneCellSave {
            col,
            row,
            tc: t.0,
            tr: t.1,
            deep,
            band: Some(band),
            ..Default::default()
        };
        cells.push(cell.clone());
        map.insert(stone_cell_key(col, row), cell);
    }
}

// port: anon__29665__stone_redress_after_cut__scr_stones
/// `stone_redress_after_cut`'s `is_stone`: not cut, and a deposit cell of this chunk or of the
/// generated chunk it lies in.
fn redress_has(w: &WorldRt, b: &WorldChunkSave, cut: &CellSet, col: i64, row: i64) -> bool {
    if cut.contains(&(col, row)) {
        return false;
    }
    let key = stone_cell_key(col, row);
    if b.stone_map.contains_key(&key) {
        return true;
    }
    let cx = ((col * 48) as f64 / w.params.chunk_w as f64).floor() as i64;
    match w.chunks.get(&cx) {
        Some(c) if cx != b.cx => c.stone_map.contains_key(&key),
        _ => false,
    }
}

// port: anon__31443__stone_redress_after_cut__scr_stones
/// `stone_redress_after_cut`'s filter: the cell was not cut.
fn redress_keep(cut: &CellSet, c: &StoneCellSave) -> bool {
    !cut.contains(&(c.col, c.row))
}

// port: stone_redress_after_cut
/// After cells were cut out of chunk `b` (caves), drop the cells left one cell thin (adding
/// them to `cut`) and re-pick the tiles of every cell around the cut.
///
/// Maze and cluster cells are one shared struct in LÖVE (in the chunk, in the maze's cached
/// data, and for a cell crossing the chunk border in the neighbouring chunk); a re-picked tile
/// is copied to those copies here.
pub fn stone_redress_after_cut(w: &mut WorldRt, b: &mut WorldChunkSave, cut: &mut CellSet) {
    if cut.is_empty() {
        return;
    }
    let mut x0 = i64::MAX;
    let mut x1 = i64::MIN;
    let mut y0 = i64::MAX;
    let mut y1 = i64::MIN;
    for &(cx, cy) in cut.iter() {
        x0 = x0.min(cx);
        x1 = x1.max(cx);
        y0 = y0.min(cy);
        y1 = y1.max(cy);
    }
    let (x0, x1, y0, y1) = (x0 - 3, x1 + 3, y0 - 3, y1 + 3);
    let mut changed = true;
    let mut pass = 0;
    while pass < 12 && changed {
        changed = false;
        for y in y0..=y1 {
            for x in x0..=x1 {
                let key = stone_cell_key(x, y);
                if !b.stone_map.contains_key(&key) || cut.contains(&(x, y)) {
                    continue;
                }
                let has = |c, r| redress_has(w, b, cut, c, r);
                let remove =
                    (!has(x - 1, y) && !has(x + 1, y)) || (!has(x, y - 1) && !has(x, y + 1));
                if remove {
                    cut.insert((x, y));
                    b.stone_map.remove(&key);
                    changed = true;
                }
            }
        }
        pass += 1;
    }
    b.stone_cells.retain(|c| redress_keep(cut, c));
    let mut changed = Vec::new();
    for y in y0..=y1 {
        for x in x0..=x1 {
            let key = stone_cell_key(x, y);
            let Some(cell) = b.stone_map.get(&key) else {
                continue;
            };
            // cell[$ "band"] ?? floor(world_hash01(x, y, 43) * 2)
            let band = cell
                .band
                .unwrap_or_else(|| (w.hash01(x as f64, y as f64, 43.0) * 2.0).floor() as i64);
            let t = stone_tile_for(w, |c, r| redress_has(w, b, cut, c, r), x, y, band as f64);
            let old = cell.clone();
            let cell = b.stone_map.get_mut(&key).unwrap();
            cell.tc = t.0;
            cell.tr = t.1;
            let new = cell.clone();
            if new == old {
                continue;
            }
            // the list holds the same struct as the map (the last one pushed with this key)
            if let Some(c) = b
                .stone_cells
                .iter_mut()
                .rev()
                .find(|c| c.col == x && c.row == y)
            {
                if *c == old {
                    *c = new.clone();
                }
            }
            changed.push((old, new));
        }
    }
    share_redressed_cells(w, b.cx, &changed);
}

/// Copy re-picked tiles to the other copies of maze/cluster cells, which are one shared struct
/// in LÖVE (until a save/load splits them): the cached maze data entry, and for a cell crossing
/// the chunk border the neighbouring chunk's cell. A copy is updated where it still equals the
/// old cell.
fn share_redressed_cells(w: &mut WorldRt, b_cx: i64, changed: &[(StoneCellSave, StoneCellSave)]) {
    if changed.is_empty() {
        return;
    }
    let cw = w.params.chunk_w;
    for (old, new) in changed {
        let key = stone_cell_key(old.col, old.row);
        for cx in [
            (old.col * 48).div_euclid(cw),
            (old.col * 48 + 47).div_euclid(cw),
        ] {
            if cx == b_cx {
                continue;
            }
            if let Some(ch) = w.chunks.get_mut(&cx) {
                if ch.stone_map.get(&key) == Some(old) {
                    ch.stone_map.insert(key.clone(), new.clone());
                    if let Some(c) = ch.stone_cells.iter_mut().rev().find(|c| *c == old) {
                        *c = new.clone();
                    }
                }
            }
        }
    }
    let by_cell: std::collections::HashMap<(i64, i64), &(StoneCellSave, StoneCellSave)> =
        changed.iter().map(|p| ((p.0.col, p.0.row), p)).collect();
    let fix = |d: &mut crate::state::MazeData| {
        for e in d.entries.iter_mut() {
            if let Some((old, new)) = by_cell.get(&(e.col, e.row)) {
                if e == old {
                    *e = new.clone();
                }
            }
        }
    };
    if let Some(o) = w.params.obstacles.as_mut() {
        for side in [&mut o.right, &mut o.left] {
            for ob in side.list.iter_mut() {
                if let Some(Some(d)) = ob.data.as_mut() {
                    fix(d);
                }
            }
        }
    }
    if let Some(list) = w.rock_clusters.as_mut() {
        for cl in list.iter_mut() {
            if let Some(d) = cl.data.as_mut() {
                fix(d);
            }
        }
    }
}

// port: stone_cut_round
/// Remove from `keys` the cells inside a wobbly ellipse (around a lake in a surface wall).
pub fn stone_cut_round(
    w: &WorldRt,
    keys: &mut CellSet,
    cx: f64,
    cy: f64,
    rx: f64,
    ry: f64,
    seed: f64,
) {
    let p1 = w.hash01(seed, 2.0, 232.0) * PHASE_RANGE;
    let p2 = w.hash01(seed, 3.0, 232.0) * PHASE_RANGE;
    let mut r = (cy - ry * 1.3).floor() - 1.0;
    while r <= (cy + ry * 1.3).ceil() + 1.0 {
        let mut c = (cx - rx * 1.3).floor() - 1.0;
        while c <= (cx + rx * 1.3).ceil() + 1.0 {
            let dx = (c + 0.5 - cx) / rx;
            let dy = (r + 0.5 - cy) / ry;
            let a = dy.atan2(dx);
            let k = 1.0 + (3.0 * a + p1).sin() * 0.16 + (5.0 * a + p2).sin() * 0.09;
            if dx * dx + dy * dy <= k * k {
                keys.remove(&(c as i64, r as i64));
            }
            c += 1.0;
        }
        r += 1.0;
    }
}

// port: stone_cut_tunnel
/// Remove from `keys` the cells along a wobbling tunnel from (x0, y0) to (x1, y1).
#[allow(clippy::too_many_arguments)]
pub fn stone_cut_tunnel(
    w: &WorldRt,
    keys: &mut CellSet,
    x0: f64,
    y0: f64,
    x1: f64,
    y1: f64,
    rad: f64,
    seed: f64,
) {
    const PI: f64 = std::f64::consts::PI;
    let len = gml::point_distance(x0, y0, x1, y1);
    let steps = 2f64.max((len / 0.5).ceil());
    let dir = gml::point_direction(x0, y0, x1, y1);
    let ph = w.hash01(seed, 4.0, 232.0) * PHASE_RANGE;
    let mut i = 0.0;
    while i <= steps {
        let t = i / steps;
        let wob = (t * PI).sin() * (t * PI * 2.0 + ph).sin() * 0.9;
        let px = gml::lerp(x0, x1, t) + gml::lengthdir_x(wob, dir + 90.0);
        let py = gml::lerp(y0, y1, t) + gml::lengthdir_y(wob, dir + 90.0);
        let rr = rad * ((t * 9.0 + ph).sin() * 0.5 * 0.35 + 0.9750000000000001);
        let mut r = (py - rr).floor() - 1.0;
        while r <= (py + rr).ceil() + 1.0 {
            let mut c = (px - rr).floor() - 1.0;
            while c <= (px + rr).ceil() + 1.0 {
                let dx = c + 0.5 - px;
                let dy = r + 0.5 - py;
                if dx * dx + dy * dy <= rr * rr {
                    keys.remove(&(c as i64, r as i64));
                }
                c += 1.0;
            }
            r += 1.0;
        }
        i += 1.0;
    }
}

// port: world_paint_stone_deposits
/// Deposit blobs in this layer of the chunk: a few tries near the surface, many more (and
/// bigger) below the deep line.
pub fn world_paint_stone_deposits(w: &mut WorldRt, ch: &mut WorldChunkSave, layer: i64) {
    if wld("stone_deposits_enabled") <= 0.0 {
        return;
    }
    let cw = w.params.chunk_w as f64;
    let key = gen::layer_key(ch.cx, layer);
    let mut tries = 0f64.max(wld("stone_deposit_tries"));
    let chance = wld("stone_deposit_chance");
    let min_r = 1f64.max(wld("stone_deposit_min_radius_cells"));
    let max_r = min_r.max(wld("stone_deposit_max_radius_cells"));
    let c0 = (ch.cx as f64 * cw / 48.0).ceil();
    let c1 = ((ch.cx + 1) as f64 * cw / 48.0).floor() - 1.0;
    let r0 = (w.layer_top(layer) / 48.0).ceil();
    let r1 = (w.layer_bottom(layer) / 48.0).floor() - 1.0;
    let base_tries = tries;
    let deep_row = (w.deep_y() / 48.0).floor();
    if r1 >= deep_row {
        tries = gml::round(tries * 1f64.max(wld("stone_deposit_deep_mult")));
    }
    let deep_layer = w.layer_top(layer) >= w.deep_y();
    let mut i = 0.0;
    while i < tries {
        let extra = i >= base_tries || deep_layer;
        let rmax = if extra {
            max_r.max(wld("stone_deposit_deep_max_radius_cells"))
        } else {
            max_r
        };
        let ch_now = if extra {
            wld("stone_deposit_deep_chance")
        } else {
            chance
        };
        if w.hash01(key, i, 51.0) < ch_now {
            let bias = 0.1f64.max(wld("stone_deposit_size_bias"));
            let rx =
                min_r + (gml::power(w.hash01(key, i, 52.0), bias) * (rmax - min_r + 1.0)).floor();
            let ry =
                min_r + (gml::power(w.hash01(key, i, 53.0), bias) * (rmax - min_r + 1.0)).floor();
            let lo_c = c0 + rx;
            let hi_c = c1 - rx;
            let lo_r = r0 + ry;
            let hi_r = r1 - ry;
            if hi_c >= lo_c && hi_r >= lo_r {
                let cc = lo_c + (w.hash01(key, i, 54.0) * (hi_c - lo_c + 1.0)).floor();
                let cr = lo_r + (w.hash01(key, i, 55.0) * (hi_r - lo_r + 1.0)).floor();
                if !(i >= base_tries && cr < deep_row) {
                    let min_l = 1f64.max(wld("stone_deposit_min_lobes"));
                    let max_l = min_l.max(wld("stone_deposit_max_lobes"));
                    let lobes = min_l + (w.hash01(key, i, 60.0) * (max_l - min_l + 1.0)).floor();
                    let blob = stone_blob_cells(
                        w,
                        cc as i64,
                        cr as i64,
                        rx as i64,
                        ry as i64,
                        lobes as i64,
                    );
                    if !blob.list.is_empty() && stone_blob_fits(w, ch, &blob) {
                        let band = (w.hash01(key, i, 61.0) * 2.0).floor() as i64;
                        stone_dress_blob(w, &mut ch.stone_cells, &mut ch.stone_map, &blob, band);
                    }
                }
            }
        }
        i += 1.0;
    }
}

// ---------------------------------------------------------------------------------------------
// tile stones (off by default: `stones_enabled` 0)

// port: world_paint_stone_clusters
/// Elliptic clusters of stone tiles for chunk `cx`, painted into `chunk` where they overlap it.
pub fn world_paint_stone_clusters(w: &mut WorldRt, chunk: &mut WorldChunkSave, cx: i64) {
    if wld("stones_enabled") <= 0.0 {
        return;
    }
    let cols = w.params.cols as f64;
    let hard_row = w.params.hard_row as f64;
    let count = 1f64.max(gml::round(wld("stone_cluster_frequency") * cols));
    let rmin = wld("stone_cluster_min_radius");
    let rmax = wld("stone_cluster_max_radius");
    let cxf = cx as f64;
    let mut k = 0.0;
    while k < count {
        let tx = cxf * cols + (w.hash01(cxf, k, 11.0) * cols).floor();
        let ty0 = 2.0 + (w.hash01(cxf, k, 12.0) * 1f64.max(hard_row - 4.0)).floor();
        let rx = rmin + (w.hash01(cxf, k, 13.0) * (rmax - rmin + 1.0)).floor();
        let ry = 1f64.max(rx - 1.0);
        let mut dy = -ry;
        while dy <= ry {
            let ny = dy / ry;
            let mut dx = -rx;
            while dx <= rx {
                let nx = dx / rx;
                let ty = ty0 + dy;
                if nx * nx + ny * ny <= 1.0 && ty >= 1.0 && ty < hard_row {
                    w.chunk_set(chunk, (tx + dx) as i64, ty as i64, 2, 0.0);
                }
                dx += 1.0;
            }
            dy += 1.0;
        }
        k += 1.0;
    }
}

// port: world_paint_stone_pillar
/// Sometimes a 1-2 tile wide stone pillar down to the hard row in chunk `cx`.
pub fn world_paint_stone_pillar(w: &mut WorldRt, chunk: &mut WorldChunkSave, cx: i64) {
    if wld("stones_enabled") <= 0.0 {
        return;
    }
    let cxf = cx as f64;
    if w.hash01(cxf, 0.0, 21.0) >= wld("stone_pillar_chance") {
        return;
    }
    let cols = w.params.cols as f64;
    let tx = cxf * cols + (w.hash01(cxf, 1.0, 22.0) * cols).floor();
    let width = 1.0 + (w.hash01(cxf, 2.0, 23.0) * 2.0).floor();
    for ty in 1..w.params.hard_row {
        let mut i = 0.0;
        while i < width {
            w.chunk_set(chunk, (tx + i) as i64, ty, 2, 0.0);
            i += 1.0;
        }
    }
}

// port: world_paint_clusters
/// Rock clusters (`maze`) overlapping this layer of the chunk, each painted once per chunk.
pub fn world_paint_clusters(w: &mut WorldRt, chunk: &mut WorldChunkSave, layer: i64) {
    super::maze::paint_clusters_in_layer(w, chunk, layer);
}
