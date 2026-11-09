//! Underground world (scr_world*): tiles, soil, lakes, water, food; caves/relics/perks/bones
//! (Task 6) and stones/maze/obstacles (Task 7) plug into the hooks in `caves`, `stones` and
//! `obstacles`.
//!
//! `WorldRt` is the live `global.world`: its parameters (`params`, a `WorldSave` whose `chunks`
//! map stays empty) and the generated chunks keyed by chunk index. Chunks are generated lazily
//! like the original's `world_get_chunk` / `world_chunk_reach`, so the tile queries take
//! `&mut self`. On load `on_load` moves `sim.state.world` in here; `before_save` writes it
//! back, so the saved world is byte-for-byte what LÖVE saves. Every tile, lake or food change
//! marks its chunk in `dirty_chunks` (ruling R9a) for the renderer to rebake and clear.

pub mod art;
pub mod caves;
pub mod food;
pub mod gen;
pub mod hash;
pub mod lakes;
pub mod maze;
pub mod obstacles;
pub mod soil;
pub mod stones;
pub mod survival;
pub mod water;

pub use food::collect_food_at;
pub use gen::world_init;
pub use water::{
    connect_lake_at, connect_water_at, water_counter_step, water_shown, water_source_amount,
    water_source_take, water_spend,
};

use crate::balance;
use crate::sim::Sim;
use crate::state::{GameMode, GameState, WorldChunkSave, WorldSave};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Asset index of the turf sprite `trintrava1` in the LÖVE build (`world.turf` in saves;
/// load_game replaces whatever a save holds with it).
pub const TURF_ASSET_INDEX: i64 = 124;

/// Tile codes as `world_tile_at` returns them (scr_world).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tile {
    /// -1: above the surface (air)
    Air,
    /// 0: soft soil (also an emptied water cell)
    Soil,
    /// 1: hard soil (`world_dense_row` and below)
    HardSoil,
    /// 2: stone / rock / cave wall / beyond the world
    Stone,
    /// 3: water (water cell or ocean)
    Water,
}

impl Tile {
    pub fn code(self) -> i64 {
        match self {
            Tile::Air => -1,
            Tile::Soil => 0,
            Tile::HardSoil => 1,
            Tile::Stone => 2,
            Tile::Water => 3,
        }
    }

    pub fn from_code(code: i64) -> Option<Tile> {
        Some(match code {
            -1 => Tile::Air,
            0 => Tile::Soil,
            1 => Tile::HardSoil,
            2 => Tile::Stone,
            3 => Tile::Water,
            _ => return None,
        })
    }
}

/// Soil tier as `world_soil_tier(row)` returns it (0 plain, 1 clay/dense, 2 deep).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Soil {
    Plain,
    Clay,
    Deep,
}

impl Soil {
    pub fn code(self) -> i64 {
        match self {
            Soil::Plain => 0,
            Soil::Clay => 1,
            Soil::Deep => 2,
        }
    }

    pub fn from_code(code: i64) -> Option<Soil> {
        Some(match code {
            0 => Soil::Plain,
            1 => Soil::Clay,
            2 => Soil::Deep,
            _ => return None,
        })
    }
}

/// World queries the logic of Tasks 8–10 uses. They take `&mut self` because a query may
/// generate the chunk (or the rows) it looks at, exactly when the original does.
pub trait WorldQuery {
    fn tile_at(&mut self, x: f64, y: f64) -> Tile;
    /// a stone/rock/cave wall tile (`world_tile_at == 2`)
    fn is_solid(&mut self, x: f64, y: f64) -> bool;
    fn water_shown_at(&mut self, x: f64, y: f64) -> bool;
    fn soil_at(&self, x: f64, y: f64) -> Soil;
    fn surface_y(&self) -> f64;
}

/// `world_locate(x, y)`: chunk, row and index of the tile under a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Locate {
    pub cx: i64,
    pub row: i64,
    pub idx: i64,
}

/// Runtime world (`global.world` and the runtime water-counter globals).
#[derive(Clone, Debug, Default)]
pub struct WorldRt {
    /// `global.world` without its chunks (`params.chunks` is always empty)
    pub params: WorldSave,
    /// generated chunks by chunk index (`world.chunks["c<cx>"]`)
    pub chunks: HashMap<i64, WorldChunkSave>,
    /// game mode the world generates for (survival thins resources, has a rock bottom)
    pub mode: GameMode,
    /// `global.water_dip` / `water_dip_hold`: the shown-water dip with infinite water
    pub water_dip: f64,
    pub water_dip_hold: f64,
    /// chunks whose tiles, lakes or foods changed since the renderer last looked (R9a)
    pub dirty_chunks: BTreeSet<i64>,
    /// `global.stones_moving`: (chunk, stone uid) of the rolling stones, oldest first
    pub stones_moving: Vec<(i64, u64)>,
    /// last runtime stone id handed out
    pub next_stone_uid: u64,
    /// `world_rock_clusters()`'s cache (with each cluster's built data)
    pub rock_clusters: Option<Vec<maze::RockCluster>>,
}

impl WorldRt {
    /// The live world from a save's `world` (load_game: `global.world = data.world`, turf
    /// replaced).
    pub fn from_save(mut w: WorldSave, mode: GameMode) -> WorldRt {
        let saved = std::mem::take(&mut w.chunks);
        w.turf = TURF_ASSET_INDEX;
        let mut chunks = HashMap::new();
        for (key, ch) in saved {
            let cx = key
                .strip_prefix('c')
                .and_then(|s| s.parse().ok())
                .unwrap_or(ch.cx);
            chunks.insert(cx, ch);
        }
        // load_game: stones lie still (`stones_moving` is empty)
        for ch in chunks.values_mut() {
            for s in &mut ch.stones {
                s.moving = false;
                s.vx = 0.0;
                s.vy = 0.0;
                s.spin = 0.0;
            }
        }
        let dirty_chunks = chunks.keys().copied().collect();
        let mut rt = WorldRt {
            params: w,
            chunks,
            mode,
            dirty_chunks,
            ..Default::default()
        };
        rt.assign_stone_uids();
        rt
    }

    /// `global.world` as save_game writes it.
    pub fn to_save(&self) -> WorldSave {
        let mut w = self.params.clone();
        w.chunks = self
            .chunks
            .iter()
            .map(|(cx, ch)| (chunk_key(*cx), ch.clone()))
            .collect::<BTreeMap<_, _>>();
        w
    }

    /// `world_hash01(a, b, c)` with this world's seed.
    pub fn hash01(&self, a: f64, b: f64, c: f64) -> f64 {
        hash::world_hash01(self.params.seed, a, b, c)
    }

    /// `bal(key)` under the world's mode.
    pub(crate) fn bal(&self, key: &str) -> f64 {
        balance::bal(self.mode, key)
    }

    /// The chunk if it was generated.
    pub fn chunk(&self, cx: i64) -> Option<&WorldChunkSave> {
        self.chunks.get(&cx)
    }

    pub fn chunk_mut(&mut self, cx: i64) -> Option<&mut WorldChunkSave> {
        self.chunks.get_mut(&cx)
    }

    // port: world_get_chunk
    /// The chunk, generated (and appended to `chunk_index`) on first use.
    pub fn get_chunk(&mut self, cx: i64) -> &mut WorldChunkSave {
        if !self.chunks.contains_key(&cx) {
            let ch = self.generate_chunk(cx);
            self.chunks.insert(cx, ch);
            self.params.chunk_index.push(cx);
            self.dirty_chunks.insert(cx);
        }
        self.chunks.get_mut(&cx).unwrap()
    }

    /// `world_chunk_reach(world_get_chunk(cx), row)`: make sure the chunk has rows through `row`.
    pub fn chunk_reach(&mut self, cx: i64, row: i64) {
        let rows = self.get_chunk(cx).rows;
        if row < rows || row >= self.params.rows {
            return;
        }
        let mut ch = self.chunks.remove(&cx).unwrap();
        self.reach(&mut ch, row);
        self.chunks.insert(cx, ch);
        self.dirty_chunks.insert(cx);
    }

    // port: world_chunk_set
    pub fn chunk_set(&self, chunk: &mut WorldChunkSave, tx: i64, ty: i64, tile: i64, amount: f64) {
        let cols = self.params.cols;
        let lx = tx - chunk.cx * cols;
        if lx < 0 || lx >= cols || ty < 0 || ty >= chunk.rows {
            return;
        }
        let i = (ty * cols + lx) as usize;
        chunk.tiles[i] = tile;
        chunk.amount[i] = amount;
    }

    // port: world_chunk_get
    /// Tile code at a tile of this chunk; -1 outside it or below its generated rows.
    pub fn chunk_get(&self, chunk: &WorldChunkSave, tx: i64, ty: i64) -> i64 {
        let cols = self.params.cols;
        let lx = tx - chunk.cx * cols;
        if lx < 0 || lx >= cols || ty < 0 || ty >= chunk.rows {
            return -1;
        }
        chunk.tiles[(ty * cols + lx) as usize]
    }

    // port: world_locate
    pub fn locate(&self, x: f64, y: f64) -> Locate {
        let p = &self.params;
        let row = ((y - p.surface_y) / p.tile as f64).floor() as i64;
        let col = (x / p.tile as f64).floor() as i64;
        let cx = (col as f64 / p.cols as f64).floor() as i64;
        Locate {
            cx,
            row,
            idx: row * p.cols + (col - cx * p.cols),
        }
    }

    // port: world_bottom_is_rock
    /// Survival worlds end in rock instead of the ocean.
    pub fn bottom_is_rock(&self) -> bool {
        balance::wld("bottom_rock") > 0.0 && self.mode == GameMode::Survival
    }

    // port: world_tile_at
    pub fn tile_at(&mut self, x: f64, y: f64) -> Tile {
        let p = &self.params;
        if x.is_nan() || y.is_nan() {
            return Tile::Stone;
        }
        if y < p.surface_y {
            return Tile::Air;
        }
        if y >= p.bottom_y {
            return Tile::Stone;
        }
        let tile = p.tile as f64;
        let row = ((y - p.surface_y) / tile).floor() as i64;
        let col = (x / tile).floor() as i64;
        let cx = (col as f64 / p.cols as f64).floor() as i64;
        let cols = p.cols;
        if row < 0 {
            return Tile::Air;
        }
        if row >= p.rows {
            return Tile::Stone;
        }
        if row as f64 >= soil::ocean_row() {
            return if self.bottom_is_rock() {
                Tile::Stone
            } else {
                Tile::Water
            };
        }
        if caves::world_cave_solid_any(self, x, y) {
            return Tile::Stone;
        }
        self.chunk_reach(cx, row);
        let chunk = &self.chunks[&cx];
        let t = chunk.tiles[(row * cols + (col - cx * cols)) as usize];
        if t == 0 && stones::stone_solid_at(self, chunk, x, y) {
            return Tile::Stone;
        }
        if t == 0 && row as f64 >= soil::dense_row() {
            return Tile::HardSoil;
        }
        Tile::from_code(t).unwrap_or(Tile::Stone)
    }

    // port: world_water_at
    /// A water cell or the body of a lake (any fill level), or the ocean.
    pub fn water_at(&mut self, x: f64, y: f64) -> bool {
        if x.is_nan() || y.is_nan() {
            return false;
        }
        if y < self.params.surface_y || y >= self.params.bottom_y {
            return false;
        }
        if y >= self.ocean_y() {
            return !self.bottom_is_rock();
        }
        let loc = self.locate(x, y);
        if loc.row < 0 || loc.row >= self.params.rows {
            return false;
        }
        self.chunk_reach(loc.cx, loc.row);
        let chunk = &self.chunks[&loc.cx];
        if chunk.tiles[loc.idx as usize] == 3 {
            return true;
        }
        let ready = lakes::lakes_ready();
        chunk.lakes.iter().any(|l| {
            if ready {
                lakes::lake_solid_at(
                    lakes::lake_kind(l.kind),
                    (x - l.x).floor(),
                    (y - l.y).floor(),
                )
            } else {
                x >= l.body_x && x < l.body_x + l.body_w && y >= l.body_y && y < l.body_y + l.body_h
            }
        })
    }

    // port: world_water_shown_at
    /// Visible water: the ocean, or a lake's mask below its current water level.
    pub fn water_shown_at(&mut self, x: f64, y: f64) -> bool {
        if x.is_nan() || y.is_nan() {
            return false;
        }
        if y < self.params.surface_y {
            return false;
        }
        if y >= self.ocean_y() {
            return !self.bottom_is_rock();
        }
        if !lakes::lakes_ready() {
            return false;
        }
        let cx = (x / self.params.chunk_w as f64).floor() as i64;
        self.get_chunk(cx).lakes.iter().any(|l| {
            if l.amount <= 0.0 {
                return false;
            }
            let kd = lakes::lake_kind(l.kind);
            let ly = (y - l.y).floor();
            ly >= lakes::lake_cut_row(l, kd) && lakes::lake_solid_at(kd, (x - l.x).floor(), ly)
        })
    }

    // port: world_is_passable
    pub fn is_passable(&mut self, x: f64, y: f64) -> bool {
        match self.tile_at(x, y) {
            Tile::Air => false,
            Tile::Stone => self.bal("stone_passable") > 0.0,
            _ => true,
        }
    }

    // port: world_is_deep_soil
    pub fn is_deep_soil(&mut self, x: f64, y: f64) -> bool {
        if self.tile_at(x, y) != Tile::HardSoil {
            return false;
        }
        self.locate(x, y).row as f64 >= soil::deep_row()
    }

    // port: world_soil_water_factor
    pub fn soil_water_factor(&mut self, x: f64, y: f64) -> f64 {
        if self.tile_at(x, y) != Tile::HardSoil {
            return 1.0;
        }
        if self.locate(x, y).row as f64 >= soil::deep_row() {
            self.bal("deepsoil_water_multiplier")
        } else {
            self.bal("hardsoil_water_multiplier")
        }
    }

    // port: world_travel_cost
    /// Growth cost multiplier through clay/deep soil (`bonus`: speed upgrades).
    pub fn travel_cost(&mut self, x: f64, y: f64, bonus: f64) -> f64 {
        if self.tile_at(x, y) != Tile::HardSoil {
            return 1.0;
        }
        let row = ((y - self.params.surface_y) / self.params.tile as f64).floor();
        let mult = if row >= soil::deep_row() {
            self.bal("deepsoil_speed_multiplier")
        } else {
            self.bal("hardsoil_speed_multiplier")
        };
        1.0 / crate::gml::clamp(mult + bonus, 0.05, 1.0)
    }

    /// Soil tier under a point (`world_soil_tier` of its row).
    pub fn soil_at(&self, _x: f64, y: f64) -> Soil {
        let row = ((y - self.params.surface_y) / self.params.tile as f64).floor();
        Soil::from_code(soil::soil_tier(row) as i64).unwrap()
    }
}

impl WorldQuery for WorldRt {
    fn tile_at(&mut self, x: f64, y: f64) -> Tile {
        WorldRt::tile_at(self, x, y)
    }
    fn is_solid(&mut self, x: f64, y: f64) -> bool {
        WorldRt::tile_at(self, x, y) == Tile::Stone
    }
    fn water_shown_at(&mut self, x: f64, y: f64) -> bool {
        WorldRt::water_shown_at(self, x, y)
    }
    fn soil_at(&self, x: f64, y: f64) -> Soil {
        WorldRt::soil_at(self, x, y)
    }
    fn surface_y(&self) -> f64 {
        self.params.surface_y
    }
}

// port: world_chunk_key
pub fn chunk_key(cx: i64) -> String {
    format!("c{cx}")
}

/// `game_mode_start`'s survival pass: `world_survival_thin(chunk)` for every chunk of the
/// existing world (sorted chunk keys, like `variable_struct_get_names`).
pub fn survival_thin_all(sim: &mut Sim) {
    let w = &mut sim.world;
    w.mode = sim.state.state.mode;
    let mut keys: Vec<i64> = w.chunks.keys().copied().collect();
    keys.sort_by_key(|cx| chunk_key(*cx));
    for cx in keys {
        let mut ch = w.chunks.remove(&cx).unwrap();
        w.survival_thin(&mut ch);
        w.chunks.insert(cx, ch);
        w.dirty_chunks.insert(cx);
    }
}

/// `load_game`'s world part: the saved world becomes the live one (turf replaced,
/// `stones_moving` and the water dip reset); `sim.state.water` already holds the sources.
pub fn on_load(sim: &mut Sim) {
    let saved = std::mem::take(&mut sim.state.world);
    sim.world = WorldRt::from_save(saved, sim.state.state.mode);
    // load_game: every lake source resolves through world_get_chunk(lake_cx)
    let lake_chunks: Vec<i64> = sim
        .state
        .water
        .iter()
        .filter_map(|w| match w {
            crate::state::WaterSave::Lake { lake_cx, .. } => Some(*lake_cx),
            _ => None,
        })
        .collect();
    for cx in lake_chunks {
        sim.world.get_chunk(cx);
    }
}

/// `save_game`'s world part: the live world into `out.world` (`out.water` is the live
/// `sim.state.water` already).
pub fn before_save(sim: &Sim, out: &mut GameState) {
    out.world = sim.world.to_save();
}
