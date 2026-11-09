//! Logic inputs derived from art (ruling R7: the render task owns `hedgehog_back_init`,
//! `squirrel_trunk_edges`, `leaf_art_init` and the sprites; logic gets their data). `art.json`
//! holds what those compute from the pixels plus the sprite metadata the animals measure,
//! captured from the LÖVE port (golden `life_assets`; tests/life.rs checks the copy).

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// `[width, height, xoffset, yoffset]` of a sprite.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(from = "[f64; 4]")]
pub struct SpriteMeta {
    pub w: f64,
    pub h: f64,
    pub xoff: f64,
    pub yoff: f64,
}

impl From<[f64; 4]> for SpriteMeta {
    fn from(a: [f64; 4]) -> Self {
        SpriteMeta {
            w: a[0],
            h: a[1],
            xoff: a[2],
            yoff: a[3],
        }
    }
}

/// `squirrel_trunk_edges(spr)`: smoothed trunk edge columns per sprite row, relative to the
/// sprite origin (`___struct___76`).
#[derive(Clone, Debug, Deserialize)]
pub struct TrunkEdges {
    pub left: Vec<f64>,
    pub right: Vec<f64>,
    pub origin_y: f64,
    pub h: f64,
}

/// One `leaf_art_read` frame: `[x1, y1, x2, y2, mid_x, mid_y, off_x, off_y]` (the sprites
/// themselves are the renderer's).
pub type LeafArtFrame = [f64; 8];

/// `global.leaf_art`: the two rest faces (1 frame each) and the two turn sheets (8 frames).
#[derive(Clone, Debug, Deserialize)]
pub struct LeafArt {
    pub faces: [Vec<LeafArtFrame>; 2],
    pub turns: [Vec<LeafArtFrame>; 2],
}

#[derive(Clone, Debug, Deserialize)]
pub struct LifeArt {
    pub sprites: HashMap<String, SpriteMeta>,
    /// `global.hedgehog_back[frame]` = `[x, y]` (frame-local px, unscaled)
    pub hedgehog_back: Vec<[f64; 2]>,
    pub trunks: HashMap<String, TrunkEdges>,
    pub leaf_art: LeafArt,
}

impl LifeArt {
    /// Sprite metadata by name (panics for a sprite not captured: a programming error).
    pub fn sprite(&self, name: &str) -> SpriteMeta {
        *self
            .sprites
            .get(name)
            .unwrap_or_else(|| panic!("life art: no sprite {name}"))
    }

    /// Trunk edges of a tree sprite (`tree01` / `tree02` / `tree03`).
    pub fn trunk(&self, name: &str) -> &TrunkEdges {
        self.trunks
            .get(name)
            .unwrap_or_else(|| panic!("life art: no trunk {name}"))
    }
}

pub fn art() -> &'static LifeArt {
    static ART: OnceLock<LifeArt> = OnceLock::new();
    ART.get_or_init(|| serde_json::from_str(include_str!("art.json")).expect("life/art.json"))
}
