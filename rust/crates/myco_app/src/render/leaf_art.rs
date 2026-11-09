//! Leaf art read from the sprite pixels.
//!
//! - Foreground leaves (`leaf_fg_art_init` / `leaf_fg_art_read`): each of the 14 strips of
//!   `leaf_fg_anim` becomes a sprite anchored on a reference point (the turn frames on the first
//!   frame's centroid, the back frames on their own), plus its alpha bounds. Logic keeps the
//!   bounds embedded in myco_core (`world::art`, R7); the app recomputes them from the pixels for
//!   the anchors, and a test checks both agree. `leaf_draw` (T15) draws with [`LeafFgSprite`].
//! - Backdrop leaves need the white silhouettes `leaf_art_read` makes (`shape`), tinted with
//!   `backdrop_leaf_colour`. [`LeafShapes`] holds them, built by Task 15's `leaf_art_init`.

use crate::assets::{GameAssets, SpriteDef};
use crate::render::sprite::part_sprite;
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::Anchor;
use std::collections::HashMap;
use std::path::Path;

/// Decoded RGBA8 texture pages (only those some sprites need).
pub struct PagePixels {
    pages: HashMap<usize, (u32, u32, Vec<u8>)>,
}

impl PagePixels {
    /// Decode the pages holding `sprites` from `<root>/pages/page<N>.png`.
    pub fn load(root: &Path, assets: &GameAssets, sprites: &[&str]) -> Result<PagePixels, String> {
        let mut pages = HashMap::new();
        for name in sprites {
            let def = assets
                .try_sprite(name)
                .ok_or_else(|| format!("unknown sprite {name}"))?;
            for fr in &def.frames {
                if pages.contains_key(&fr.page_index) {
                    continue;
                }
                let path = root.join(format!("pages/page{}.png", fr.page_index));
                let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                let img = Image::from_buffer(
                    &bytes,
                    ImageType::Extension("png"),
                    CompressedImageFormats::NONE,
                    true,
                    ImageSampler::Default,
                    RenderAssetUsages::MAIN_WORLD,
                )
                .map_err(|e| format!("{}: {e}", path.display()))?
                .convert(TextureFormat::Rgba8UnormSrgb)
                .ok_or("page is not convertible to RGBA8")?;
                let (w, h) = (img.width(), img.height());
                let data = img.data.ok_or("page has no pixel data")?;
                pages.insert(fr.page_index, (w, h, data));
            }
        }
        Ok(PagePixels { pages })
    }

    /// RGBA of pixel (x, y) of a sprite frame in untrimmed coordinates (transparent outside the
    /// trimmed rectangle).
    pub fn sprite_rgba(&self, def: &SpriteDef, frame: usize, x: i64, y: i64) -> [u8; 4] {
        let Some(fr) = def.frames.get(frame) else {
            return [0; 4];
        };
        let (tx, ty) = (x - fr.trim_offset.x as i64, y - fr.trim_offset.y as i64);
        let size = fr.rect.size();
        if tx < 0 || ty < 0 || tx >= size.x as i64 || ty >= size.y as i64 {
            return [0; 4];
        }
        let Some((w, _h, data)) = self.pages.get(&fr.page_index) else {
            return [0; 4];
        };
        let o =
            (((fr.rect.min.y as i64 + ty) * *w as i64 + fr.rect.min.x as i64 + tx) * 4) as usize;
        [data[o], data[o + 1], data[o + 2], data[o + 3]]
    }

    /// Alpha (0..255) of pixel (x, y) of a sprite frame in untrimmed coordinates; 0 outside the
    /// trimmed rectangle (what drawing the sprite onto a cleared surface leaves).
    pub fn sprite_alpha(&self, def: &SpriteDef, frame: usize, x: i64, y: i64) -> u8 {
        let Some(fr) = def.frames.get(frame) else {
            return 0;
        };
        let (tx, ty) = (x - fr.trim_offset.x as i64, y - fr.trim_offset.y as i64);
        let size = fr.rect.size();
        if tx < 0 || ty < 0 || tx >= size.x as i64 || ty >= size.y as i64 {
            return 0;
        }
        let Some((w, _h, data)) = self.pages.get(&fr.page_index) else {
            return 0;
        };
        let px = fr.rect.min.x as i64 + tx;
        let py = fr.rect.min.y as i64 + ty;
        data[((py * *w as i64 + px) * 4 + 3) as usize]
    }
}

/// Alpha scan of one `fw × h` strip (`leaf_fg_art_read` / `leaf_art_read` loops): bounds of the
/// pixels with alpha >= 128 (whole strip when none) and their centroid.
pub(crate) fn strip_scan(px: &PagePixels, def: &SpriteDef, i: i64, fw: i64, h: i64) -> [f64; 6] {
    let (mut x1, mut y1, mut x2, mut y2) = (fw as f64, h as f64, -1.0, -1.0);
    let (mut sx, mut sy, mut n) = (0.0, 0.0, 0.0);
    for yy in 0..h {
        for xx in 0..fw {
            if px.sprite_alpha(def, 0, i * fw + xx, yy) >= 128 {
                let (fx, fy) = (xx as f64, yy as f64);
                x1 = f64::min(x1, fx);
                y1 = f64::min(y1, fy);
                x2 = f64::max(x2, fx);
                y2 = f64::max(y2, fy);
                sx += fx;
                sy += fy;
                n += 1.0;
            }
        }
    }
    if x2 < x1 {
        (x1, y1, x2, y2) = (0.0, 0.0, (fw - 1) as f64, (h - 1) as f64);
    }
    let mx = if n > 0.0 { sx / n } else { fw as f64 * 0.5 };
    let my = if n > 0.0 { sy / n } else { h as f64 * 0.5 };
    [x1, y1, x2, y2, mx, my]
}

/// One foreground-leaf frame (`___struct___58`): the strip `part` = `[left, top, w, h]` of the
/// sheet drawn with its origin at (`origin_x`, `origin_y`) inside the strip, and the bounds
/// relative to that origin.
#[derive(Clone, Debug, PartialEq)]
pub struct LeafFgSprite {
    pub sprite: String,
    pub part: [f64; 4],
    pub origin_x: f64,
    pub origin_y: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub mid_x: f64,
    pub mid_y: f64,
}

impl LeafFgSprite {
    /// Bevy sprite with the origin at the entity (`draw_sprite_ext(f.sprite, 0, x, y, m, 1, ang)`:
    /// the caller applies mirror, rotation and position through the `Transform`).
    pub fn bevy_sprite(&self, assets: &GameAssets) -> Option<(Sprite, Anchor)> {
        let def = assets.try_sprite(&self.sprite)?;
        let [left, top, w, h] = self.part;
        let (sprite, dx, dy) = part_sprite(def, 0, left, top, w, h)?;
        let size = sprite.rect?.size();
        // origin inside the clipped rect, y down
        let (ox, oy) = ((self.origin_x - dx) as f32, (self.origin_y - dy) as f32);
        Some((
            sprite,
            Anchor(Vec2::new(ox / size.x - 0.5, 0.5 - oy / size.y)),
        ))
    }
}

/// `global.leaf_fg_art` (`___struct___57`).
#[derive(Clone, Debug, Resource)]
pub struct LeafFgArt {
    pub rest: LeafFgSprite,
    pub turn: Vec<LeafFgSprite>,
    pub turned: LeafFgSprite,
    pub carry_x: f64,
    pub carry_y: f64,
}

// port: leaf_fg_art_read
/// `leaf_fg_art_read(spr, frames, anchor_first)`.
pub fn leaf_fg_art_read(
    assets: &GameAssets,
    px: &PagePixels,
    spr: &str,
    frames: i64,
    anchor_first: bool,
) -> Vec<LeafFgSprite> {
    let def = assets.sprite(spr);
    let (w, h) = (def.size.x as i64, def.size.y as i64);
    let fw = w / frames; // gm.div truncates
    let (mut ref_x, mut ref_y) = (-1.0, -1.0);
    let mut out = Vec::new();
    for i in 0..frames {
        let [x1, y1, x2, y2, mx, my] = strip_scan(px, def, i, fw, h);
        if ref_x < 0.0 || !anchor_first {
            ref_x = mx;
            ref_y = my;
        }
        // sprite_create_from_surface(shp, 0, 0, fw + 4, h + 4, .., ref_x + 2, ref_y + 2) with the
        // strip drawn at (2, 2): the strip itself, origin at the reference point
        out.push(LeafFgSprite {
            sprite: spr.to_owned(),
            part: [(i * fw) as f64, 0.0, fw as f64, h as f64],
            origin_x: ref_x,
            origin_y: ref_y,
            x1: x1 - ref_x,
            y1: y1 - ref_y,
            x2: x2 + 1.0 - ref_x,
            y2: y2 + 1.0 - ref_y,
            mid_x: mx - ref_x,
            mid_y: my - ref_y,
        });
    }
    out
}

// port: leaf_fg_art_init
/// `leaf_fg_art_init`: the turn frames anchored on the first, the turned pose = last back frame.
pub fn leaf_fg_art_init(assets: &GameAssets, px: &PagePixels) -> LeafFgArt {
    let turn = leaf_fg_art_read(assets, px, "leaf_fg_anim", 14, true);
    let back = leaf_fg_art_read(assets, px, "leaf_fg_anim", 14, false);
    let last = turn.last().cloned().expect("leaf_fg_anim has frames");
    LeafFgArt {
        rest: turn[0].clone(),
        turned: back.last().cloned().expect("leaf_fg_anim has frames"),
        carry_x: last.mid_x,
        carry_y: last.mid_y,
        turn,
    }
}

/// White silhouettes of the leaf frames (`leaf_art_read`'s `shape`: the strip drawn at (2, 2) on
/// a `(fw + 4)²` canvas, colour set to white, alpha kept; origin at the canvas centre).
#[derive(Clone, Debug, Default, Resource)]
pub struct LeafShapes {
    /// `global.leaf_art.faces[face][0]` (leaf, leaf_2)
    pub faces: [Vec<Handle<Image>>; 2],
    /// `global.leaf_art.turns[from][frame]` (leaf_change, leaf_change_2; 8 frames each)
    pub turns: [Vec<Handle<Image>>; 2],
}

impl LeafShapes {
    /// `leaf_art_frame(l).shape` for a backdrop leaf.
    pub fn frame(&self, change_from: f64, change_frame: f64, face: f64) -> Option<&Handle<Image>> {
        if change_from >= 0.0 {
            let set = self.turns.get(change_from.floor() as usize)?;
            return set.get(change_frame.max(0.0).floor() as usize);
        }
        self.faces.get(face.max(0.0).floor() as usize)?.first()
    }
}

/// Silhouette images of `spr` split into `frames` strips.
pub fn leaf_shape_images(px: &PagePixels, def: &SpriteDef, frames: i64) -> Vec<Image> {
    let (w, h) = (def.size.x as i64, def.size.y as i64);
    let fw = w / frames;
    let sz = (fw + 4) as u32;
    (0..frames)
        .map(|i| {
            let mut data = vec![0u8; (sz * sz * 4) as usize];
            for yy in 0..h.min(sz as i64 - 2) {
                for xx in 0..fw {
                    let a = px.sprite_alpha(def, 0, i * fw + xx, yy);
                    let o = (((yy + 2) * sz as i64 + xx + 2) * 4) as usize;
                    data[o..o + 4].copy_from_slice(&[255, 255, 255, a]);
                }
            }
            Image::new(
                Extent3d {
                    width: sz,
                    height: sz,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                data,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            )
        })
        .collect()
}

/// Startup: decode the leaf sheets and build [`LeafFgArt`] and [`LeafShapes`]. Without the
/// pages (no assets) nothing is inserted and backdrop leaves are not drawn.
pub fn build_leaf_art(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    const SHEETS: [&str; 5] = [
        "leaf",
        "leaf_2",
        "leaf_change",
        "leaf_change_2",
        "leaf_fg_anim",
    ];
    let px = match PagePixels::load(&assets.root, &assets, &SHEETS) {
        Ok(p) => p,
        Err(e) => {
            warn!("leaf art: {e}");
            return;
        }
    };
    commands.insert_resource(leaf_fg_art_init(&assets, &px));
    // leaf_art_init (Task 15) measures the frames and makes the silhouettes
    let art = crate::render::life::leaf_art_init(&px, &assets);
    let mut add = |v: Vec<Image>| -> Vec<Handle<Image>> { v.into_iter().map(|i| images.add(i)).collect() };
    let [f0, f1] = art.faces;
    let [t0, t1] = art.turns;
    let shapes = LeafShapes {
        faces: [add(f0), add(f1)],
        turns: [add(t0), add(t1)],
    };
    commands.insert_resource(shapes);
}
