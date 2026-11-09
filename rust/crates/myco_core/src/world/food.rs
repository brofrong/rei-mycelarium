//! Food in the soil (scr_world): kinds and values, placement per layer, the start-screen
//! quotas, and collection by the growing mycelium.

use super::art::art;
use super::gen::StartRing;
use super::{caves, lakes, stones, WorldRt};
use crate::balance::{vis, wld};
use crate::gml;
use crate::progress::tutorial;
use crate::sim::Sim;
use crate::state::{FoodSave, WorldChunkSave};

// port: world_food_sheets
pub fn food_sheets() -> [&'static str; 3] {
    ["food_small", "food_medium", "food_big"]
}

// port: world_food_kinds
pub fn food_kinds() -> i64 {
    food_sheets().len() as i64
}

fn sheet_index(kind: f64) -> usize {
    gml::clamp(kind, 0.0, (food_kinds() - 1) as f64).trunc() as usize
}

// port: world_food_sprite
pub fn food_sprite(kind: f64) -> &'static str {
    food_sheets()[sheet_index(kind)]
}

// port: world_food_cells
/// Frames in a food sheet (frames are stacked vertically, square).
pub fn food_cells(kind: f64) -> f64 {
    let s = &art().sprites;
    let [w, h] = [s.food_small, s.food_medium, s.food_big][sheet_index(kind)];
    (h / w).trunc().max(1.0)
}

// port: world_food_value
pub fn food_value(kind: f64) -> f64 {
    const VALUES: [f64; 3] = [1.0, 3.0, 5.0];
    VALUES[gml::clamp(kind, 0.0, 2.0).trunc() as usize]
}

/// A food struct (`___struct___342/344/352`).
pub(crate) fn new_food(x: f64, y: f64, kind: i64, cell: f64) -> FoodSave {
    FoodSave {
        cell: cell as i64,
        collected: false,
        kind,
        x,
        y,
        maze: None,
        sv: None,
        extra: Default::default(),
    }
}

// port: world_food_columns
/// The chunk's foods bucketed by tile column (with one spill column on each side): indices
/// into `chunk.foods`. The original caches this per chunk while the list is unchanged; the
/// result is the same, so it is rebuilt here.
pub fn food_columns(w: &WorldRt, chunk: &WorldChunkSave) -> Vec<Vec<usize>> {
    let p = &w.params;
    let x0 = (chunk.cx * p.chunk_w) as f64;
    let n = (p.cols + 2) as usize;
    let mut cols = vec![Vec::new(); n];
    for (i, f) in chunk.foods.iter().enumerate() {
        let c = gml::clamp(
            ((f.x - x0) / p.tile as f64).floor() + 1.0,
            0.0,
            (n - 1) as f64,
        );
        cols[c.trunc() as usize].push(i);
    }
    cols
}

// port: world_collect_food_at
/// Collect every uncollected food within `r` of (x, y); returns the food value gained.
pub fn collect_food_at(sim: &mut Sim, x: f64, y: f64, r: f64) -> f64 {
    let w = &mut sim.world;
    let cw = w.params.chunk_w as f64;
    let tile = w.params.tile as f64;
    let mut total = 0.0;
    let c0 = ((x - r) / cw).floor() as i64;
    let c1 = ((x + r) / cw).floor() as i64;
    for cx in c0..=c1 {
        w.get_chunk(cx);
        let cols = food_columns(w, &w.chunks[&cx]);
        let x0 = cx as f64 * cw;
        let last = (cols.len() - 1) as f64;
        let mut a = gml::clamp(((x - r - x0) / tile).floor() + 1.0, 0.0, last);
        let mut b = gml::clamp(((x + r - x0) / tile).floor() + 1.0, 0.0, last);
        if a == 1.0 {
            a = 0.0;
        }
        if b == last - 1.0 {
            b = last;
        }
        let chunk = w.chunks.get_mut(&cx).unwrap();
        let mut c = a;
        while c <= b {
            for &j in &cols[c.trunc() as usize] {
                let f = &mut chunk.foods[j];
                if !f.collected && gml::point_distance(x, y, f.x, f.y) <= r {
                    f.collected = true;
                    total += food_value(f.kind as f64);
                    w.dirty_chunks.insert(cx);
                }
            }
            c += 1.0;
        }
    }
    if total > 0.0 {
        tutorial::event(sim, "food", None, None);
    }
    total
}

impl WorldRt {
    // port: world_paint_food
    /// Scatter food (with clumps) through this layer of the chunk; rarer far from the start,
    /// denser in clay and deep soil.
    pub(crate) fn paint_food(&mut self, chunk: &mut WorldChunkSave, layer: i64) {
        let p = self.params.clone();
        let key = super::gen::layer_key(chunk.cx, layer);
        let cw = p.chunk_w as f64;
        let tile = p.tile as f64;
        let x0 = chunk.cx as f64 * cw;
        let dist = (x0 + cw * 0.5 - p.start_x).abs();
        let rarity = 1.0 + dist / wld("food_distance_rarity_px").max(1.0);
        let spacing = gml::lerp(
            wld("food_spawn_near_min_px"),
            wld("food_spawn_near_max_px"),
            self.hash01(key, 0.0, 31.0),
        ) * rarity;
        let mut count = gml::round(cw / spacing.max(1.0));
        let top = self.layer_top(layer);
        let bottom = self.layer_bottom(layer);
        let base = count;
        let dense_y = p.surface_y + super::soil::dense_row() * tile;
        let deep_y = self.deep_y();
        let mut dense_count = count;
        if bottom > dense_y {
            dense_count = gml::round(base * wld("dense_food_multiplier").max(1.0));
            count = dense_count;
        }
        if bottom > deep_y {
            count = count.max(gml::round(base * wld("deep_food_multiplier").max(1.0)));
        }
        let kinds = food_kinds() as f64;
        let mut i = 0.0;
        while i < count {
            let fx = x0 + self.hash01(key, i, 32.0) * cw;
            let fy = gml::lerp(top, bottom, self.hash01(key, i, 33.0));
            if !(i >= base && fy < dense_y) && !(i >= dense_count && fy < deep_y) {
                let deep = fy >= deep_y;
                let chance = if !deep {
                    wld("food_clump_chance")
                } else {
                    wld("deep_food_clump_chance")
                };
                let max_extra = if !deep {
                    wld("food_clump_max_extra")
                } else {
                    wld("deep_food_clump_max_extra")
                };
                let spread = tile
                    * if !deep {
                        3.0
                    } else {
                        wld("deep_food_clump_spread_tiles")
                    };
                let mut extra = 0.0;
                if self.hash01(key, i, 34.0) < chance {
                    extra = 15f64.min(1.0 + (self.hash01(key, i, 35.0) * max_extra).floor());
                }
                let mut j = 0.0;
                while j <= extra {
                    let mut px = fx;
                    let mut py = fy;
                    let q = i * 16.0 + j;
                    if j > 0.0 {
                        px += (self.hash01(key, q, 36.0) - 0.5) * spread;
                        py += (self.hash01(key, q, 37.0) - 0.5) * spread;
                    }
                    px = gml::clamp(px, x0 + 8.0, x0 + cw - 8.0);
                    py = gml::clamp(py, top, bottom);
                    let tx = (px / tile).floor() as i64;
                    let ty = ((py - p.surface_y) / tile).floor() as i64;
                    let free = self.chunk_get(chunk, tx, ty) == 0
                        && !caves::world_cave_hits(self, px, py, px, py, None)
                        && !lakes::world_lake_covers(chunk, px, py)
                        && !stones::world_stone_covers(self, chunk, px, py);
                    if free {
                        let tier = (kinds - 1.0).min((self.hash01(key, q, 38.0) * kinds).floor());
                        let cells = food_cells(tier);
                        let cell = (cells - 1.0).min((self.hash01(key, q, 39.0) * cells).floor());
                        chunk.foods.push(new_food(px, py, tier as i64, cell));
                    }
                    j += 1.0;
                }
            }
            i += 1.0;
        }
    }

    // port: world_food_value_in
    /// Value of the uncollected food inside a rectangle (generating what it covers).
    pub fn food_value_in(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let cw = self.params.chunk_w as f64;
        let reach = ((y2 - self.params.surface_y) / self.params.tile as f64).floor() as i64;
        let mut total = 0.0;
        let mut c = (x1 / cw).floor() as i64;
        while c <= (x2 / cw).floor() as i64 {
            self.chunk_reach(c, reach);
            for f in &self.chunks[&c].foods {
                if !f.collected && f.x >= x1 && f.x < x2 && f.y >= y1 && f.y < y2 {
                    total += food_value(f.kind as f64);
                }
            }
            c += 1;
        }
        total
    }

    // port: world_food_spot_ok
    fn food_spot_ok(&mut self, x: f64, y: f64, rect: [f64; 4], ctx: &StartRing) -> bool {
        let [x1, y1, x2, y2] = rect;
        let p = &self.params;
        if x < x1 || x >= x2 || y < y1 || y >= y2 || y < p.surface_y + p.tile as f64 {
            return false;
        }
        if gml::point_distance(ctx.x, ctx.y, x, y) < ctx.near {
            return false;
        }
        if caves::world_cave_hits(self, x, y, x, y, None) {
            return false;
        }
        if self.tile_at(x, y) != super::Tile::Soil {
            return false;
        }
        let cx = (x / self.params.chunk_w as f64).floor() as i64;
        self.get_chunk(cx);
        let chunk = &self.chunks[&cx];
        !lakes::world_lake_covers(chunk, x, y) && !stones::world_stone_covers(self, chunk, x, y)
    }

    // port: world_food_fill
    /// Bring the food value inside a rectangle into [fmin, fmax] (fmax < 0: no upper bound):
    /// remove the biggest surplus foods, then add hashed foods sized to the gap.
    pub(crate) fn food_fill(&mut self, rect: [f64; 4], fmin: f64, fmax: f64, salt: f64) {
        let [x1, y1, x2, y2] = rect;
        let cw = self.params.chunk_w as f64;
        let mut v = self.food_value_in(x1, y1, x2, y2);
        if fmax >= 0.0 && v > fmax {
            let mut c = (x1 / cw).floor() as i64;
            while c <= (x2 / cw).floor() as i64 && v > fmax {
                let mut changed = false;
                let foods = &mut self.get_chunk(c).foods;
                let mut k = food_kinds() - 1;
                while k >= 0 && v > fmax {
                    let mut j = foods.len() as i64 - 1;
                    while j >= 0 && v > fmax {
                        let f = &foods[j as usize];
                        let val = food_value(f.kind as f64);
                        if f.kind == k
                            && !(f.collected
                                || f.x < x1
                                || f.x >= x2
                                || f.y < y1
                                || f.y >= y2
                                || v - val < fmin)
                        {
                            foods.remove(j as usize);
                            v -= val;
                            changed = true;
                        }
                        j -= 1;
                    }
                    k -= 1;
                }
                if changed {
                    self.dirty_chunks.insert(c);
                }
                c += 1;
            }
        }
        let target = if fmax >= 0.0 { fmax } else { fmin };
        let ring = self.start_ring();
        let pad = 32.0;
        let (surface_y, tile) = (self.params.surface_y, self.params.tile as f64);
        let mut t = 0.0;
        while t < 400.0 && v < fmin {
            let x = gml::lerp(x1 + pad, x2 - pad, self.hash01(t, 1.0, salt));
            let y = gml::lerp(
                y1.max(surface_y + tile) + pad,
                y2 - pad,
                self.hash01(t, 2.0, salt),
            );
            let mut a = 0.0;
            while a < 4.0 && v < fmin {
                let ox = if a == 0.0 {
                    0.0
                } else {
                    (self.hash01(t, a + 3.0, salt) - 0.5) * 90.0
                };
                let px = x + ox;
                let oy = if a == 0.0 {
                    0.0
                } else {
                    (self.hash01(t, a + 7.0, salt) - 0.5) * 90.0
                };
                let py = y + oy;
                if self.food_spot_ok(px, py, rect, &ring) {
                    let need = target - v;
                    let kind = if need >= 5.0 {
                        2
                    } else if need >= 3.0 {
                        1
                    } else {
                        0
                    };
                    let cx = (px / cw).floor() as i64;
                    let cells = food_cells(kind as f64);
                    let cell = (cells - 1.0).min((self.hash01(t, a + 11.0, salt) * cells).floor());
                    self.get_chunk(cx).foods.push(new_food(px, py, kind, cell));
                    self.dirty_chunks.insert(cx);
                    v += food_value(kind as f64);
                }
                a += 1.0;
            }
            t += 1.0;
        }
    }

    // port: world_start_food_quota
    /// The start screen holds exactly a hashed amount of food in [min, max]; the screens around
    /// it at least `start_side_screen_food_min`.
    pub(crate) fn start_food_quota(&mut self) {
        let sw = vis("screen_width_px");
        let sh = vis("screen_height_px");
        let x0 = self.params.start_x - sw * 0.5;
        let mn = wld("start_screen_food_min");
        let mx = wld("start_screen_food_max");
        let n = mn + (self.hash01(7.0, 7.0, 120.0) * (mx - mn + 1.0)).floor();
        self.food_fill([x0, 0.0, x0 + sw, sh], n, n, 121.0);
        let mut salt = 130.0;
        for row in 0..=1 {
            for col in -1..=1 {
                if row == 0 && col == 0 {
                    continue;
                }
                let cx = x0 + col as f64 * sw;
                let cy = row as f64 * sh;
                self.food_fill(
                    [cx, cy, cx + sw, cy + sh],
                    wld("start_side_screen_food_min"),
                    -1.0,
                    salt,
                );
                salt += 10.0;
            }
        }
    }
}
