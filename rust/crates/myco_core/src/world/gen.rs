//! World creation and chunk generation (scr_world): `world_init`, the start-area guarantees,
//! layers, `world_generate_chunk` / `world_chunk_reach` / `world_paint_layer*`, ocean
//! clearing. Everything derives from `world_hash01` with the world seed; the only RNG draw is
//! the seed itself.

use super::lakes::{self, lake_layout};
use super::{caves, obstacles, stones, WorldRt};
use crate::balance::{vis, wld};
use crate::gml;
use crate::sim::Sim;
use crate::state::{GameMode, WorldChunkSave, WorldSave};

/// `world_start_ring()`: the band around the start where the first lake and food must lie.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StartRing {
    pub x: f64,
    pub y: f64,
    pub near: f64,
    pub far: f64,
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

// port: world_layer_key
/// Hash key of a chunk layer.
pub fn layer_key(cx: i64, layer: i64) -> f64 {
    if layer == 0 {
        cx as f64
    } else {
        (cx + layer * 0x250000) as f64
    }
}

// port: world_rect_distance
/// Distance from a point to a rectangle (0 inside).
pub fn rect_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = (x1 - px).max(0.0).max(px - x2);
    let dy = (y1 - py).max(0.0).max(py - y2);
    (dx * dx + dy * dy).sqrt()
}

// port: world_init
/// `world_init(start_x)`: a new world with a random seed (`sim.world_seed`, the MYCO_SEED
/// analogue, fixes it after the draw), its start guarantees and start-screen food quota.
pub fn world_init(sim: &mut Sim, start_x: f64) {
    let mut seed = sim.rng.irandom(99999.0);
    if let Some(s) = sim.world_seed {
        seed = s;
    }
    sim.world = WorldRt::init(start_x, seed, sim.state.state.mode);
}

impl WorldRt {
    /// `world_init` after the seed draw: `global.world` with no chunks, then
    /// `world_start_guarantee` and `world_start_food_quota` (`stones_moving` is Task 7's).
    pub fn init(start_x: f64, seed: i64, mode: GameMode) -> WorldRt {
        let tile = wld("reference_tile_size");
        let surface_h = wld("surface_height_px");
        let rows = (wld("underground_screen_depth") * vis("screen_height_px") / tile).ceil();
        let cols = wld("chunk_cols");
        let hard_rows = 4f64
            .max(rows - wld("bottom_water_layer_tiles") - (vis("screen_height_px") / tile).ceil());
        let params = WorldSave {
            bottom_y: surface_h + 1_000_000.0 * tile,
            chunk_index: Vec::new(),
            chunk_w: (cols * tile) as i64,
            chunks: Default::default(),
            cols: cols as i64,
            hard_row: hard_rows as i64,
            layer_rows: hard_rows as i64,
            obstacles: None,
            rows: 1_000_000,
            seed,
            start_x,
            surface_y: surface_h,
            tile: tile as i64,
            turf: super::TURF_ASSET_INDEX,
            water_row: 1_000_000,
            extra: Default::default(),
        };
        let mut w = WorldRt {
            params,
            mode,
            ..Default::default()
        };
        w.start_guarantee();
        w.start_food_quota();
        w
    }

    // port: world_start_ring
    pub fn start_ring(&self) -> StartRing {
        let range = self.bal("base_shot_range");
        let p = &self.params;
        let pad = 56.0;
        StartRing {
            x: p.start_x,
            y: p.surface_y,
            near: range * wld("start_resource_min_shots"),
            far: range * wld("start_resource_max_shots"),
            x1: p.start_x - vis("screen_width_px") * 0.5 + pad,
            x2: p.start_x + vis("screen_width_px") * 0.5 - pad,
            y1: p.surface_y + p.tile as f64,
            y2: vis("screen_height_px") - pad,
        }
    }

    // port: world_start_angle
    fn start_angle(&self, x: f64, y: f64) -> f64 {
        let spread = 50.0;
        if self.hash01(x, 1.0, y) < 0.5 {
            352.0 - self.hash01(x, 0.0, y) * spread
        } else {
            188.0 + self.hash01(x, 0.0, y) * spread
        }
    }

    // port: world_start_clear
    /// No lake or food closer to the start than the ring's inner radius.
    pub(crate) fn start_clear(&self, ch: &mut WorldChunkSave) {
        let ring = self.start_ring();
        ch.lakes.retain(|l| {
            rect_distance(
                ring.x,
                ring.y,
                l.body_x,
                l.body_y,
                l.body_x + l.body_w,
                l.body_y + l.body_h,
            ) >= ring.near
        });
        ch.foods
            .retain(|f| gml::point_distance(ring.x, ring.y, f.x, f.y) >= ring.near);
    }

    // port: world_start_guarantee
    /// A lake and a food inside the start ring, on the start screen, added if generation left
    /// none.
    pub(crate) fn start_guarantee(&mut self) {
        let ring = self.start_ring();
        let cw = self.params.chunk_w as f64;
        let c0 = ((ring.x - ring.far - 400.0) / cw).floor() as i64;
        let c1 = ((ring.x + ring.far + 400.0) / cw).floor() as i64;
        let mut has_lake = false;
        let mut has_food = false;
        for c in c0..=c1 {
            let chunk = self.get_chunk(c);
            for l in &chunk.lakes {
                let d = rect_distance(
                    ring.x,
                    ring.y,
                    l.body_x,
                    l.body_y,
                    l.body_x + l.body_w,
                    l.body_y + l.body_h,
                );
                has_lake = has_lake
                    || (d >= ring.near
                        && d <= ring.far
                        && start_on_screen(
                            &ring,
                            l.body_x,
                            l.body_y,
                            l.body_x + l.body_w,
                            l.body_y + l.body_h,
                        ));
            }
            for f in &chunk.foods {
                let d = gml::point_distance(ring.x, ring.y, f.x, f.y);
                has_food = has_food
                    || (!f.collected
                        && d >= ring.near
                        && d <= ring.far
                        && start_on_screen(&ring, f.x, f.y, f.x, f.y));
            }
        }
        if !has_lake {
            self.start_add_lake(&ring);
        }
        if !has_food {
            self.start_add_food(&ring);
        }
    }

    // port: world_start_add_lake
    fn start_add_lake(&mut self, ring: &StartRing) -> bool {
        let kind = 1;
        let lay = lake_layout()[kind as usize];
        let gap = wld("water_lake_gap");
        let clr = wld("stone_water_clearance_px");
        let p = self.params.clone();
        let top = p.surface_y + p.tile as f64;
        let hard = p.surface_y + (p.hard_row * p.tile) as f64;
        let cw = p.chunk_w as f64;
        for a in 0..80 {
            let af = a as f64;
            let ang = self.start_angle(af, 91.0);
            let dist = gml::lerp(ring.near, ring.far, 0.2 + self.hash01(af, 0.0, 92.0) * 0.6)
                + lay.body_w.max(lay.body_h) * 0.3;
            let x1 = (ring.x + gml::lengthdir_x(dist, ang) - lay.body_w * 0.5).floor();
            let y1 = (ring.y + gml::lengthdir_y(dist, ang) - lay.body_h * 0.5).floor();
            let x2 = x1 + lay.body_w;
            let y2 = y1 + lay.body_h;
            let d = rect_distance(ring.x, ring.y, x1, y1, x2, y2);
            if d < ring.near + 16.0 {
                continue;
            }
            if d > ring.far - 16.0 || y1 < top || y2 > hard {
                continue;
            }
            if !(x1 >= ring.x1 && x2 <= ring.x2 && y1 >= ring.y1 && y2 <= ring.y2) {
                continue;
            }
            let cx = (x1 / cw).floor() as i64;
            if x1 < cx as f64 * cw + gap || x2 > (cx + 1) as f64 * cw - gap {
                continue;
            }
            self.get_chunk(cx);
            let mut ok = !caves::world_cave_hits(self, x1, y1, x2, y2, None);
            let chunk = &self.chunks[&cx];
            let mut i = 0;
            while i < chunk.lakes.len() && ok {
                let l = &chunk.lakes[i];
                ok = !(x1 < l.body_x + l.body_w + gap
                    && x2 + gap > l.body_x
                    && y1 < l.body_y + l.body_h + gap
                    && y2 + gap > l.body_y);
                i += 1;
            }
            let mut yy = y1 - clr;
            while yy <= y2 + clr && ok {
                let mut xx = x1 - clr;
                while xx <= x2 + clr && ok {
                    ok = !stones::world_stone_covers(self, chunk, xx, yy);
                    xx += 16.0;
                }
                yy += 16.0;
            }
            if !ok {
                continue;
            }
            let amount = lakes::lake_amount(kind);
            let mut chunk = self.chunks.remove(&cx).unwrap();
            chunk.lakes.push(lakes::new_lake(
                cx,
                900 + a,
                kind,
                x1 - lay.body_x,
                y1 - lay.body_y,
                [x1, y1, lay.body_w, lay.body_h],
                amount,
            ));
            let foods = std::mem::take(&mut chunk.foods);
            chunk.foods = foods
                .into_iter()
                .filter(|f| !lakes::world_lake_covers(&chunk, f.x, f.y))
                .collect();
            let stones_list = std::mem::take(&mut chunk.stones);
            chunk.stones = stones_list
                .into_iter()
                .filter(|s| !lakes::world_lake_covers(&chunk, s.x, s.y))
                .collect();
            self.chunks.insert(cx, chunk);
            self.dirty_chunks.insert(cx);
            return true;
        }
        false
    }

    // port: world_start_add_food
    fn start_add_food(&mut self, ring: &StartRing) -> bool {
        let kinds = super::food::food_kinds();
        let p = self.params.clone();
        let tile = p.tile as f64;
        for a in 0..80 {
            let af = a as f64;
            let ang = self.start_angle(af, 95.0);
            let dist = gml::lerp(ring.near, ring.far, 0.25 + self.hash01(af, 0.0, 96.0) * 0.5);
            let bx = ring.x + gml::lengthdir_x(dist, ang);
            let by = ring.y + gml::lengthdir_y(dist, ang);
            let mut placed = 0;
            for j in 0..4 {
                let jf = j as f64;
                let x = bx
                    + if j == 0 {
                        0.0
                    } else {
                        (self.hash01(af, jf, 97.0) - 0.5) * 80.0
                    };
                let y = by
                    + if j == 0 {
                        0.0
                    } else {
                        (self.hash01(af, jf, 98.0) - 0.5) * 80.0
                    };
                let d = gml::point_distance(ring.x, ring.y, x, y);
                let cx = (x / p.chunk_w as f64).floor() as i64;
                self.get_chunk(cx);
                let row = ((y - p.surface_y) / tile).floor();
                if d < ring.near
                    || d > ring.far
                    || row < 1.0
                    || y >= p.surface_y + (p.hard_row * p.tile) as f64
                    || !start_on_screen(ring, x, y, x, y)
                {
                    continue;
                }
                let chunk = &self.chunks[&cx];
                if self.chunk_get(chunk, (x / tile).floor() as i64, row as i64) != 0 {
                    continue;
                }
                if lakes::world_lake_covers(chunk, x, y)
                    || stones::world_stone_covers(self, chunk, x, y)
                {
                    continue;
                }
                let tier = (kinds - 1).min(1);
                let cells = super::food::food_cells(tier as f64);
                let cell = (cells - 1.0).min((self.hash01(af, jf, 99.0) * cells).floor());
                self.chunks
                    .get_mut(&cx)
                    .unwrap()
                    .foods
                    .push(super::food::new_food(x, y, tier, cell));
                self.dirty_chunks.insert(cx);
                placed += 1;
            }
            if placed > 0 {
                return true;
            }
        }
        false
    }

    // port: world_layer_top
    pub fn layer_top(&self, layer: i64) -> f64 {
        let p = &self.params;
        p.surface_y + (if layer == 0 { 1 } else { layer * p.layer_rows } * p.tile) as f64
    }

    // port: world_layer_bottom
    pub fn layer_bottom(&self, layer: i64) -> f64 {
        let p = &self.params;
        p.surface_y + ((layer + 1) * p.layer_rows * p.tile) as f64
    }

    // port: world_generate_chunk
    /// A new chunk with its first layer painted.
    pub(crate) fn generate_chunk(&mut self, cx: i64) -> WorldChunkSave {
        let mut ch = WorldChunkSave {
            cx,
            rows: 0,
            ..Default::default()
        };
        self.reach(&mut ch, 0);
        ch
    }

    // port: world_chunk_reach
    /// Paint layers until the chunk has rows through `row` (the chunk is not in `chunks`).
    pub(crate) fn reach(&mut self, chunk: &mut WorldChunkSave, row: i64) {
        let (lr, cols, rows) = (self.params.layer_rows, self.params.cols, self.params.rows);
        if row < chunk.rows || row >= rows {
            return;
        }
        while chunk.rows <= row {
            let layer = chunk.rows / lr;
            let i1 = ((chunk.rows + lr) * cols) as usize;
            chunk.tiles.resize(i1, 0);
            chunk.amount.resize(i1, 0.0);
            chunk.connected.resize(i1, false);
            chunk.rows += lr;
            self.paint_layer(chunk, layer);
        }
    }

    // port: world_paint_layer
    fn paint_layer(&mut self, chunk: &mut WorldChunkSave, layer: i64) {
        caves::caves_extend(self, (chunk.cx as f64 + 0.5) * self.params.chunk_w as f64);
        self.paint_layer_content(chunk, layer);
        super::bones::world_paint_bones(self, chunk, layer);
        caves::world_clear_caves(self, chunk);
        self.survival_thin(chunk);
    }

    // port: world_paint_layer_content
    fn paint_layer_content(&mut self, chunk: &mut WorldChunkSave, layer: i64) {
        let cx = chunk.cx;
        if layer <= 0 {
            for c in cx - 1..=cx + 1 {
                stones::world_paint_stone_clusters(self, chunk, c);
            }
            for c in cx - 1..=cx + 1 {
                stones::world_paint_stone_pillar(self, chunk, c);
            }
            let obs = obstacles::world_obstacles_at_chunk(self, chunk);
            for o in obs.iter().filter(|o| o.kind == "maze") {
                obstacles::world_paint_maze_part(self, chunk, o);
            }
            self.paint_lakes(chunk, 0);
            for o in &obs {
                if o.kind == "wall" && o.cx == Some(cx) {
                    obstacles::world_paint_surface_wall(self, chunk);
                }
            }
            stones::world_paint_stone_deposits(self, chunk, 0);
            let sx = (self.params.start_x / self.params.tile as f64).floor() as i64;
            for row in 0..=4 {
                for dx in -3..=3 {
                    if self.chunk_get(chunk, sx + dx, row) == 2 {
                        self.chunk_set(chunk, sx + dx, row, 0, 0.0);
                    }
                }
            }
            self.paint_food(chunk, 0);
            self.start_clear(chunk);
            stones::world_paint_stones(self, chunk, 0);
            return;
        }
        let oy = self.ocean_y();
        if self.layer_top(layer) >= oy {
            return;
        }
        stones::world_paint_clusters(self, chunk, layer);
        self.paint_lakes(chunk, layer);
        stones::world_paint_stone_deposits(self, chunk, layer);
        self.paint_food(chunk, layer);
        stones::world_paint_stones(self, chunk, layer);
        if self.layer_bottom(layer) > oy {
            self.clear_ocean(chunk, oy);
        }
    }

    // port: world_clear_ocean
    /// Nothing generated reaches into the ocean margin.
    fn clear_ocean(&self, chunk: &mut WorldChunkSave, oy: f64) {
        let margin = wld("ocean_clear_margin_px");
        let reach = stones::stone_reach(self);
        let limit = oy - margin;
        chunk.lakes.retain(|l| lake_above(l, limit));
        chunk.foods.retain(|v| food_above(v, limit));
        let stone_limit = oy - margin - reach;
        chunk.stones.retain(|v| stone_above(v, stone_limit));
        chunk.bones.retain(|v| bone_above(v, limit));
    }
}

// port: anon__109412__world_clear_ocean__scr_world
fn lake_above(l: &crate::state::LakeSave, limit: f64) -> bool {
    l.body_y + l.body_h < limit
}

// port: anon__109575__world_clear_ocean__scr_world
fn food_above(v: &crate::state::FoodSave, limit: f64) -> bool {
    v.y < limit
}

// port: anon__109735__world_clear_ocean__scr_world
fn stone_above(v: &crate::state::StoneSave, limit: f64) -> bool {
    v.y < limit
}

// port: anon__109891__world_clear_ocean__scr_world
fn bone_above(v: &crate::state::BoneSave, limit: f64) -> bool {
    v.y + v.h < limit
}

// port: world_start_on_screen
pub fn start_on_screen(r: &StartRing, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    x1 >= r.x1 && x2 <= r.x2 && y1 >= r.y1 && y2 <= r.y2
}
