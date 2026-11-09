//! Soil bands and depth rows (scr_world): plain soil, clay (dense) and deep soil, the ocean row,
//! and the band sprite names the renderer draws. The row helpers depend only on the visual and
//! worldgen tables, like the original's static caches.

use super::WorldRt;
use crate::balance::{vis, wld};
use crate::gml;
use std::sync::OnceLock;

/// `gm.div`: truncating division.
fn div(a: f64, b: f64) -> f64 {
    (a / b).trunc()
}

// port: world_soil_band
pub fn soil_band(row: f64) -> f64 {
    let first = wld("soil_first_rows").max(1.0);
    if row < first {
        return 0.0;
    }
    1.0 + div(row - first, wld("soil_band_rows").max(1.0))
}

// port: world_seam_row_at_screens
/// The first band seam at or below `screens` screen heights of depth.
pub fn seam_row_at_screens(screens: f64) -> f64 {
    let first = wld("soil_first_rows").max(1.0);
    let band = wld("soil_band_rows").max(1.0);
    let row = (screens * vis("screen_height_px") / wld("reference_tile_size")).ceil();
    first + ((row - first) / band).ceil().max(0.0) * band
}

// port: world_dense_row
/// First clay row (cached like the original's static: the tables never change).
pub fn dense_row() -> f64 {
    static ROW: OnceLock<f64> = OnceLock::new();
    *ROW.get_or_init(|| seam_row_at_screens(wld("dense_soil_depth_screens")))
}

// port: world_deep_row
pub fn deep_row() -> f64 {
    static ROW: OnceLock<f64> = OnceLock::new();
    *ROW.get_or_init(|| dense_row().max(seam_row_at_screens(wld("deep_soil_depth_screens"))))
}

// port: world_soil_tier
/// 0 plain, 1 clay (dense), 2 deep.
pub fn soil_tier(row: f64) -> f64 {
    if row >= deep_row() {
        2.0
    } else if row >= dense_row() {
        1.0
    } else {
        0.0
    }
}

// port: world_ocean_row
pub fn ocean_row() -> f64 {
    static ROW: OnceLock<f64> = OnceLock::new();
    *ROW.get_or_init(|| {
        let screens = wld("deep_soil_depth_screens") + wld("ocean_after_deep_screens");
        (deep_row() + 1.0).max(seam_row_at_screens(screens))
    })
}

/// The six-step band cycle shared by the three sprite pickers.
fn band_pick(tier: f64, m: f64, names: [[&'static str; 4]; 3]) -> &'static str {
    let set = if tier == 2.0 {
        &names[2]
    } else if tier == 1.0 {
        &names[1]
    } else {
        &names[0]
    };
    // m is fmod(band, 6) of an integral band: 0..5 (or negative for negative bands)
    if m == 0.0 {
        set[0]
    } else if m == 1.0 {
        set[1]
    } else if m == 2.0 {
        set[2]
    } else if m == 3.0 {
        set[3]
    } else if m == 4.0 {
        set[2]
    } else {
        set[1]
    }
}

// port: world_soil_sprite
pub fn soil_sprite(row: f64) -> &'static str {
    let m = gml::gmod(soil_band(row), 6.0);
    band_pick(
        soil_tier(row),
        m,
        [
            ["soil1", "soil2", "soil3", "soil4"],
            ["densesoil1", "densesoil2", "densesoil3", "densesoil4"],
            ["deepsoil1", "deepsoil2", "deepsoil3", "deepsoil4"],
        ],
    )
}

// port: world_dirt_sprite
pub fn dirt_sprite(row: f64) -> &'static str {
    let m = gml::gmod(soil_band(row), 6.0);
    band_pick(
        soil_tier(row),
        m,
        [
            ["dirt1", "dirt2", "dirt3", "dirt4"],
            ["densedirt1", "densedirt2", "densedirt3", "densedirt4"],
            ["deepdirt1", "deepdirt2", "deepdirt3", "deepdirt4"],
        ],
    )
}

// port: world_soil_edge_sprite
/// Edge strip drawn at the top of `band`; its tier is the one of the row above.
pub fn soil_edge_sprite(band: f64, row: f64) -> &'static str {
    let m = gml::gmod(band, 6.0);
    band_pick(
        soil_tier(row - 1.0),
        m,
        [
            ["soil1_edge", "soil2_edge", "soil3_edge", "soil4_edge"],
            [
                "densesoil1_edge",
                "densesoil2_edge",
                "densesoil3_edge",
                "densesoil4_edge",
            ],
            [
                "deepsoil1_edge",
                "deepsoil2_edge",
                "deepsoil3_edge",
                "deepsoil4_edge",
            ],
        ],
    )
}

impl WorldRt {
    // port: world_deep_y
    pub fn deep_y(&self) -> f64 {
        self.params.surface_y + deep_row() * self.params.tile as f64
    }

    // port: world_ocean_y
    pub fn ocean_y(&self) -> f64 {
        self.params.surface_y + ocean_row() * self.params.tile as f64
    }

    // port: world_is_ocean_at
    pub fn is_ocean_at(&self, y: f64) -> bool {
        y >= self.ocean_y()
    }

    // port: world_dense_y
    pub fn dense_y(&self) -> f64 {
        self.params.surface_y + dense_row() * self.params.tile as f64
    }
}
