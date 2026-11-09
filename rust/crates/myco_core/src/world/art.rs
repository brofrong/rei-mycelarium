//! Logic inputs derived from art (ruling R7: the render task owns the functions that read the
//! pixels; logic gets their data). `art.json` holds what `lakes_init` (lake masks) and
//! `leaf_fg_art_init` (frame bounds) compute from the sprites, and the sprite sizes the logic
//! reads, captured from the LÖVE port (golden `world_assets`; tests/world.rs checks the copy).

use serde::Deserialize;
use std::sync::OnceLock;

/// One lake shape of the `water` sprite (`global.lake_kinds[k]`).
#[derive(Clone, Debug, Deserialize)]
pub struct LakeKind {
    pub left: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub bottom: f64,
    /// per row: solid spans as [x0, x1, x0, x1, ...] (inclusive)
    pub spans: Vec<Vec<f64>>,
}

/// Bounds of one foreground-leaf frame relative to its anchor (`leaf_fg_art_read`).
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "[f64; 6]")]
pub struct LeafFgFrame {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub mid_x: f64,
    pub mid_y: f64,
}

impl From<[f64; 6]> for LeafFgFrame {
    fn from(a: [f64; 6]) -> Self {
        LeafFgFrame {
            x1: a[0],
            y1: a[1],
            x2: a[2],
            y2: a[3],
            mid_x: a[4],
            mid_y: a[5],
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct LeafFgArt {
    pub rest: LeafFgFrame,
    pub turn: Vec<LeafFgFrame>,
    pub turned: LeafFgFrame,
    pub carry_x: f64,
    pub carry_y: f64,
}

/// `[width, height]` of the sprites logic measures.
#[derive(Clone, Debug, Deserialize)]
pub struct SpriteSizes {
    pub food_small: [f64; 2],
    pub food_medium: [f64; 2],
    pub food_big: [f64; 2],
    pub water_flare: [f64; 2],
    pub originshroom: [f64; 2],
    pub stump: [f64; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct Art {
    pub lake_kinds: Vec<LakeKind>,
    pub leaf_fg: LeafFgArt,
    pub sprites: SpriteSizes,
    pub turf: i64,
}

pub fn art() -> &'static Art {
    static ART: OnceLock<Art> = OnceLock::new();
    ART.get_or_init(|| serde_json::from_str(include_str!("art.json")).expect("world/art.json"))
}
