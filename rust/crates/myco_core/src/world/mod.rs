//! Underground world (scr_world*): tiles, soil, water, food, caves, stones. Task 5 implements
//! `WorldRt` and [`WorldQuery`]; Tasks 6–7 add caves/relics/perks/bones and stones/maze.
//! Tasks 8–10 are written against the trait.

use crate::sim::Sim;
use crate::state::{GameState, WaterSave};

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

/// World queries the logic of Tasks 8–10 uses; `WorldRt` implements it in Task 5.
pub trait WorldQuery {
    fn tile_at(&self, x: f64, y: f64) -> Tile;
    fn is_solid(&self, x: f64, y: f64) -> bool;
    fn water_shown_at(&self, x: f64, y: f64) -> bool;
    fn soil_at(&self, x: f64, y: f64) -> Soil;
    fn surface_y(&self) -> f64;
}

/// Runtime world (`global.world` and its caches). Task 5 fills it.
#[derive(Clone, Debug, Default)]
pub struct WorldRt {}

/// `water_source_amount(src)` (Task 5): water left in a connected lake or water cell.
/// Until then: 0.
pub fn water_source_amount(_sim: &Sim, _src: &WaterSave) -> f64 {
    0.0
}

/// `game_mode_start`'s survival pass (Task 5): `world_survival_thin(chunk)` for every chunk
/// of an existing world.
pub fn survival_thin_all(_sim: &mut Sim) {}

/// `load_game`'s world part (Task 5): turf sprite, `stones_moving`, runtime caches from
/// `sim.state.world` / `sim.state.water`.
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s world part (Task 5): write runtime world into `out.world` / `out.water`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
