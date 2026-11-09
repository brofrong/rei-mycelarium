//! The language UI fonts (`locale_font_load` → `locale_font_fit`, `locale_font_ink_h`): the
//! language's TTF sized so its line height and the ink height of "HEXN" match the bitmap
//! reference (`Font1` for `ui_font()`, font `-1` = `fnt_gm_20` for `ui_font_small()`).
//!
//! Sizes are in the painter's convention ([`TtfFont::size`] is the `ab_glyph` pixel scale);
//! `font_add(file, size)` converts LÖVE's em size to it, so the measured heights follow LÖVE.

use super::gm_draw::{TtfFont, UiFont};
use crate::assets::GameAssets;
use ab_glyph::{Font as _, FontArc, ScaleFont as _};
use bevy::prelude::*;
use myco_core::locale::Locale;
use std::collections::HashMap;

/// `ui_font()` and `ui_font_small()` for the current language.
#[derive(Resource, Clone)]
pub struct UiFonts {
    pub font: UiFont,
    pub small: UiFont,
}

impl UiFonts {
    /// Bitmap fonts only (English, or before the language font is fitted).
    pub fn bitmap() -> UiFonts {
        UiFonts {
            font: UiFont::Gm("Font1".into()),
            small: UiFont::Gm("fnt_gm_20".into()),
        }
    }

    // port: locale_font_load
    /// `locale_font_load(lang)`: `Font1` / font `-1`, or the language's TTF fitted to each.
    /// Call again after a language change (the settings, Task 18).
    pub fn load(assets: &GameAssets, loc: &Locale) -> UiFonts {
        let mut out = UiFonts::bitmap();
        let id = loc.font_for();
        let Some(file) = id.file() else {
            return out;
        };
        let Ok(bytes) = std::fs::read(assets.root.join(file)) else {
            return out;
        };
        let pages = FontPages::load(assets);
        let handle = assets.font(id);
        if let Some(f) = locale_font_fit(assets, &pages, &handle, &bytes, Some("Font1")) {
            out.font = UiFont::Ttf(f);
        }
        if let Some(f) = locale_font_fit(assets, &pages, &handle, &bytes, None) {
            out.small = UiFont::Ttf(f);
        }
        out
    }
}

/// `font_add(file, size)`: LÖVE's `newFont(file, size)` (size = px per em).
pub fn font_add(handle: &Handle<Font>, bytes: &[u8], size: f64) -> Option<TtfFont> {
    let font = FontArc::try_from_vec(bytes.to_vec()).ok()?;
    let em = font.units_per_em().unwrap_or(1000.0) as f64;
    let h_units = (font.ascent_unscaled() - font.descent_unscaled()) as f64;
    Some(TtfFont {
        handle: handle.clone(),
        size: (size * h_units / em) as f32,
        font,
    })
}

/// Decoded pages of the bitmap fonts (alpha for `buffer_get_surface`).
pub struct FontPages(HashMap<usize, (u32, u32, Vec<u8>)>);

impl FontPages {
    pub fn load(assets: &GameAssets) -> FontPages {
        use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
        let mut out = HashMap::new();
        for f in &assets.gm_fonts {
            if out.contains_key(&f.page_index) {
                continue;
            }
            let path = assets.root.join(format!("pages/page{}.png", f.page_index));
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(img) = Image::from_buffer(
                &bytes,
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::Default,
                bevy::asset::RenderAssetUsages::MAIN_WORLD,
            ) else {
                continue;
            };
            let Some(img) =
                img.convert(bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb)
            else {
                continue;
            };
            let (w, h) = (img.width(), img.height());
            if let Some(d) = img.data {
                out.insert(f.page_index, (w, h, d));
            }
        }
        FontPages(out)
    }

    fn alpha(&self, page: usize, x: u32, y: u32) -> u8 {
        match self.0.get(&page) {
            Some((w, h, d)) if x < *w && y < *h => d[((y * w + x) * 4 + 3) as usize],
            _ => 0,
        }
    }
}

/// A font as `locale_font_ink_h` draws it.
pub enum InkFont<'a> {
    /// a bitmap font by name
    Gm(&'a str),
    Ttf(&'a TtfFont),
}

fn gm_line_height(assets: &GameAssets, name: &str) -> f64 {
    assets.gm_font(name).map_or(0.0, |f| {
        if f.line_height > 0.0 {
            f.line_height
        } else {
            f.glyphs.values().map(|g| g.h as f64).fold(0.0, f64::max)
        }
    })
}

fn gm_width(assets: &GameAssets, name: &str, s: &str) -> f64 {
    assets.gm_font(name).map_or(0.0, |f| {
        s.chars()
            .filter_map(|c| f.glyphs.get(&(c as u32)))
            .map(|g| g.shift as f64)
            .sum()
    })
}

// port: locale_font_ink_h
/// `locale_font_ink_h(fnt, str)`: draw `str` in white at (8, 8) on a transparent surface of
/// `clamp(ceil(w) + 16, 16, 1024)` × `clamp(ceil(h) * 2 + 16, 16, 1024)` and count the rows from
/// the first to the last that hold a pixel with alpha > 110.
pub fn locale_font_ink_h(assets: &GameAssets, pages: &FontPages, f: InkFont, s: &str) -> f64 {
    let (sw, sh) = match &f {
        InkFont::Gm(n) => (gm_width(assets, n, s), gm_line_height(assets, n)),
        InkFont::Ttf(t) => (t.width(s), t.height()),
    };
    let w = (sw.ceil() + 16.0).clamp(16.0, 1024.0) as usize;
    let h = (sh.ceil() * 2.0 + 16.0).clamp(16.0, 1024.0) as usize;
    let mut surf = vec![0u8; w * h];
    let mut plot = |x: i64, y: i64, a: u8| {
        if x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h {
            let i = y as usize * w + x as usize;
            surf[i] = surf[i].max(a);
        }
    };
    match f {
        InkFont::Gm(name) => {
            let Some(font) = assets.gm_font(name) else {
                return 0.0;
            };
            let mut x = 8.0f32;
            for c in s.chars() {
                let Some(g) = font.glyphs.get(&(c as u32)) else {
                    continue;
                };
                let (fx, fy) = (font.frame.min.x + g.x, font.frame.min.y + g.y);
                for yy in 0..g.h as u32 {
                    for xx in 0..g.w as u32 {
                        let a = pages.alpha(font.page_index, fx as u32 + xx, fy as u32 + yy);
                        plot((x + g.offset) as i64 + xx as i64, 8 + yy as i64, a);
                    }
                }
                x += g.shift;
            }
        }
        InkFont::Ttf(t) => {
            let scale = ab_glyph::PxScale::from(t.size);
            let sf = t.font.as_scaled(scale);
            let mut x = 8.0f32;
            for c in s.chars() {
                let id = sf.glyph_id(c);
                let g = id.with_scale_and_position(scale, ab_glyph::point(x, 8.0 + sf.ascent()));
                if let Some(o) = t.font.outline_glyph(g) {
                    let b = o.px_bounds();
                    o.draw(|px, py, cov| {
                        let a = (cov.clamp(0.0, 1.0) * 255.0).round() as u8;
                        plot(b.min.x as i64 + px as i64, b.min.y as i64 + py as i64, a);
                    });
                }
                x += sf.h_advance(id);
            }
        }
    }
    let rows: Vec<usize> = (0..h)
        .filter(|&y| surf[y * w..(y + 1) * w].iter().any(|&a| a as f64 > 110.0))
        .collect();
    match (rows.first(), rows.last()) {
        (Some(t), Some(b)) => (b - t + 1) as f64,
        _ => 0.0,
    }
}

// port: locale_font_fit
/// `locale_font_fit(file, ref_font)`: add the TTF at the reference font's size (12 for font
/// `-1`), resize once when its "Mg" line height is more than 5 % off, then up to twice until
/// the "HEXN" ink height is within `max(1, 3 %)` of the reference's.
pub fn locale_font_fit(
    assets: &GameAssets,
    pages: &FontPages,
    handle: &Handle<Font>,
    bytes: &[u8],
    ref_font: Option<&str>,
) -> Option<TtfFont> {
    let probe = "HEXN";
    let ref_name = ref_font.unwrap_or("fnt_gm_20");
    let mut size = match ref_font {
        None => 12.0,
        Some(n) => assets.gm_font(n).map_or(12.0, |f| f.size),
    };
    let mut f = font_add(handle, bytes, size)?;
    let ref_h = gm_line_height(assets, ref_name);
    let new_h = f.height();
    if new_h > 0.0 && (new_h - ref_h).abs() > ref_h * 0.05 {
        size = myco_core::gml::round(size * ref_h / new_h).max(4.0);
        f = font_add(handle, bytes, size)?;
    }
    let ref_ink = locale_font_ink_h(assets, pages, InkFont::Gm(ref_name), probe);
    for _ in 0..2 {
        if ref_ink <= 0.0 {
            break;
        }
        let ink = locale_font_ink_h(assets, pages, InkFont::Ttf(&f), probe);
        if ink <= 0.0 || (ink - ref_ink).abs() <= (ref_ink * 0.03).max(1.0) {
            break;
        }
        size = myco_core::gml::round(size * ref_ink / ink).max(4.0);
        f = font_add(handle, bytes, size)?;
    }
    Some(f)
}
