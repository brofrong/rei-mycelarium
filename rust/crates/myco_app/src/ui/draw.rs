//! Immediate-mode GUI drawing for the menus: the menu code builds a list of GameMaker-style draw
//! calls ([`DrawCmd`]: `draw_rectangle`, `draw_text_transformed`, `draw_primitive`) in GUI
//! coordinates (view-sized, y down, ruling R17); [`render_draw_list`] turns the list into Bevy
//! entities on the GUI layer, respawning them only when the list changes.
//!
//! Text uses the language's fonts as the original does: `ui_font()` is `Font1` (the GameMaker
//! bitmap font) for English, else the language's TTF fitted to `Font1`'s size
//! (`locale_font_fit`); `ui_font_small()` is `fnt_gm_20` (font `-1` in the LÖVE port) or the TTF
//! fitted to it. [`UiFonts`] measures strings the way `string_width`/`string_height` do.

use crate::assets::GameAssets;
use crate::camera::GUI_LAYER;
use crate::render::blend::{colour_mesh, Vert};
use crate::render::sprite::gm_colour;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::{LineHeight, TextLayout};
use myco_core::locale::{FontId, Locale};
use std::sync::Arc;

/// `ui_font()` or `ui_font_small()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontSel {
    Main,
    Small,
}

/// One GameMaker draw call in GUI coordinates. Colours are GameMaker BGR integers.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    /// `draw_rectangle(x1, y1, x2, y2, outline)`
    Rect {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        col: f64,
        alpha: f64,
        outline: bool,
    },
    /// `draw_text_transformed(x, y, s, sc, sc, 0)` with the given alignment (0 left/top,
    /// 1 centre/middle, 2 right/bottom)
    Text {
        x: f64,
        y: f64,
        s: String,
        sc: f64,
        col: f64,
        alpha: f64,
        font: FontSel,
        halign: u8,
        valign: u8,
    },
    /// a triangle list (`draw_primitive_begin(pr_trianglelist)` / strips converted)
    Tris(Vec<Vert>),
}

/// The menus' draw list for this frame (written in `Update`, rendered in `PostUpdate`).
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct UiDrawList(pub Vec<DrawCmd>);

/// String metrics of the two UI fonts.
pub trait TextMetrics {
    fn width(&self, font: FontSel, s: &str) -> f64;
    fn height(&self, font: FontSel, s: &str) -> f64;
}

/// `hud_fit(s, sc, maxw, maxh)`: the scale `sc`, reduced so `s` fits `maxw` (and `maxh` when
/// positive).
pub fn hud_fit(m: &dyn TextMetrics, font: FontSel, s: &str, sc: f64, maxw: f64, maxh: f64) -> f64 {
    let mut k = sc;
    let w = m.width(font, s);
    let h = m.height(font, s);
    if w * k > maxw {
        k = maxw.max(0.05) / w.max(1.0);
    }
    if maxh > 0.0 && h * k > maxh {
        k = maxh.max(0.05) / h.max(1.0);
    }
    k
}

/// Convert triangle strips to a triangle list of one colour.
pub fn strips_to_tris(strips: &[Vec<[f64; 2]>], col: f64, alpha: f64) -> Vec<Vert> {
    let mut out = Vec::new();
    for s in strips {
        for i in 2..s.len() {
            let (a, b, c) = (s[i - 2], s[i - 1], s[i]);
            for p in [a, b, c] {
                out.push(Vert {
                    x: p[0],
                    y: p[1],
                    col,
                    alpha,
                });
            }
        }
    }
    out
}

/// Glyph advance and line metrics of one font face.
#[derive(Clone, Debug)]
pub enum Face {
    /// a GameMaker bitmap font (index into `GameAssets::gm_fonts`)
    Gm(usize),
    /// a TTF/OTF at a pixel size
    Ttf {
        id: FontId,
        size: f64,
        data: Arc<Vec<u8>>,
    },
}

/// The fonts of the current language and how to measure them.
#[derive(Resource, Clone, Debug)]
pub struct UiFonts {
    pub main: Face,
    pub small: Face,
    /// GameMaker font metrics, copied out of `GameAssets` (index → (advances, line height))
    gm: Vec<GmMetrics>,
}

#[derive(Clone, Debug, Default)]
struct GmMetrics {
    shifts: std::collections::HashMap<u32, f64>,
    line_height: f64,
}

fn split_lines(s: &str) -> impl Iterator<Item = &str> {
    s.split('\n')
}

impl UiFonts {
    /// Fonts for the locale's language. TTF sizes follow `locale_font_fit`: start at the
    /// reference font's size (12 for the small font), match line height within 5 %, then match
    /// the ink height of "HEXN" within 3 % (up to two refinements).
    pub fn build(assets: &GameAssets, locale: &Locale, font1_ink: Option<f64>) -> UiFonts {
        let gm: Vec<GmMetrics> = assets
            .gm_fonts
            .iter()
            .map(|f| {
                let shifts = f
                    .glyphs
                    .iter()
                    .map(|(k, g)| (*k, g.shift as f64))
                    .collect();
                let line_height = if f.line_height > 0.0 {
                    f.line_height
                } else {
                    f.glyphs.values().map(|g| g.h as f64).fold(0.0, f64::max)
                };
                GmMetrics {
                    shifts,
                    line_height,
                }
            })
            .collect();
        let font1 = assets.gm_font("Font1").map(|f| f.index).unwrap_or(0);
        let gm20 = assets.gm_font("fnt_gm_20").map(|f| f.index).unwrap_or(0);
        let mut fonts = UiFonts {
            main: Face::Gm(font1),
            small: Face::Gm(gm20),
            gm,
        };
        let id = locale.font_for();
        let Some(file) = id.file() else {
            return fonts;
        };
        let Ok(bytes) = std::fs::read(assets.root.join(file)) else {
            return fonts;
        };
        let data = Arc::new(bytes);
        if ttf_parser::Face::parse(&data, 0).is_err() {
            return fonts;
        }
        let fit = |fonts: &UiFonts, reference: &Face, start: f64, ref_ink: Option<f64>| -> f64 {
            let ref_h = fonts.face_height(reference);
            let mut size = start;
            let new_h = ttf_height(&data, size);
            if new_h > 0.0 && (new_h - ref_h).abs() > ref_h * 0.05 {
                size = (size * ref_h / new_h).round().max(4.0);
            }
            if let Some(ref_ink) = ref_ink.filter(|r| *r > 0.0) {
                for _ in 0..2 {
                    let ink = ttf_ink(&data, size, "HEXN");
                    if ink <= 0.0 || (ink - ref_ink).abs() <= (ref_ink * 0.03).max(1.0) {
                        break;
                    }
                    size = (size * ref_ink / ink).round().max(4.0);
                }
            }
            size
        };
        let f1_size = assets.gm_font("Font1").map(|f| f.size).unwrap_or(45.0);
        let main_size = fit(&fonts, &fonts.main.clone(), f1_size, font1_ink);
        // the small reference (fnt_gm_20) is not measured for ink here: its height match is kept
        let small_size = fit(&fonts, &fonts.small.clone(), 12.0, None);
        fonts.main = Face::Ttf {
            id,
            size: main_size,
            data: data.clone(),
        };
        fonts.small = Face::Ttf {
            id,
            size: small_size,
            data,
        };
        fonts
    }

    /// Plain monospace-ish metrics for tests (`w` per char, `h` per line), no assets.
    pub fn fixed(w: f64, h: f64) -> UiFonts {
        let shifts = (32u32..0x2FFFF).map(|c| (c, w)).collect();
        UiFonts {
            main: Face::Gm(0),
            small: Face::Gm(0),
            gm: vec![GmMetrics {
                shifts,
                line_height: h,
            }],
        }
    }

    pub fn face(&self, f: FontSel) -> &Face {
        match f {
            FontSel::Main => &self.main,
            FontSel::Small => &self.small,
        }
    }

    fn face_height(&self, face: &Face) -> f64 {
        match face {
            Face::Gm(i) => self.gm.get(*i).map_or(0.0, |m| m.line_height),
            Face::Ttf { size, data, .. } => ttf_height(data, *size),
        }
    }

    fn line_width(&self, face: &Face, line: &str) -> f64 {
        match face {
            Face::Gm(i) => {
                let Some(m) = self.gm.get(*i) else { return 0.0 };
                line.chars()
                    .map(|c| m.shifts.get(&(c as u32)).copied().unwrap_or(0.0))
                    .sum()
            }
            Face::Ttf { size, data, .. } => ttf_width(data, *size, line),
        }
    }
}

impl TextMetrics for UiFonts {
    fn width(&self, font: FontSel, s: &str) -> f64 {
        let face = self.face(font);
        split_lines(s)
            .map(|l| self.line_width(face, l))
            .fold(0.0, f64::max)
    }

    fn height(&self, font: FontSel, s: &str) -> f64 {
        self.face_height(self.face(font)) * split_lines(s).count() as f64
    }
}

fn ttf_height(data: &[u8], size: f64) -> f64 {
    let Ok(f) = ttf_parser::Face::parse(data, 0) else {
        return 0.0;
    };
    let k = size / f.units_per_em() as f64;
    ((f.ascender() as f64 - f.descender() as f64 + f.line_gap() as f64) * k).round()
}

fn ttf_width(data: &[u8], size: f64, s: &str) -> f64 {
    let Ok(f) = ttf_parser::Face::parse(data, 0) else {
        return 0.0;
    };
    let k = size / f.units_per_em() as f64;
    s.chars()
        .filter_map(|c| f.glyph_index(c))
        .map(|g| (f.glyph_hor_advance(g).unwrap_or(0) as f64 * k).round())
        .sum()
}

fn ttf_ink(data: &[u8], size: f64, s: &str) -> f64 {
    let Ok(f) = ttf_parser::Face::parse(data, 0) else {
        return 0.0;
    };
    let k = size / f.units_per_em() as f64;
    let (mut top, mut bottom) = (f64::MIN, f64::MAX);
    for c in s.chars() {
        if let Some(bb) = f.glyph_index(c).and_then(|g| f.glyph_bounding_box(g)) {
            top = top.max(bb.y_max as f64);
            bottom = bottom.min(bb.y_min as f64);
        }
    }
    if top < bottom {
        return 0.0;
    }
    ((top - bottom) * k).round()
}

/// `locale_font_ink_h(Font1, "HEXN")`: rows of the bitmap glyphs with alpha > 110.
pub fn gm_font_ink(assets: &GameAssets, font: &str, s: &str) -> Option<f64> {
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
    use bevy::render::render_resource::TextureFormat;
    let f = assets.gm_font(font)?;
    let bytes = std::fs::read(assets.root.join(format!("pages/page{}.png", f.page_index))).ok()?;
    let img = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::MAIN_WORLD,
    )
    .ok()?
    .convert(TextureFormat::Rgba8UnormSrgb)?;
    let w = img.width() as usize;
    let data = img.data?;
    let (mut top, mut bottom) = (i64::MAX, i64::MIN);
    for c in s.chars() {
        let g = f.glyphs.get(&(c as u32))?;
        for yy in 0..g.h as usize {
            for xx in 0..g.w as usize {
                let px = f.frame.min.x as usize + g.x as usize + xx;
                let py = f.frame.min.y as usize + g.y as usize + yy;
                if data.get((py * w + px) * 4 + 3).copied().unwrap_or(0) > 110 {
                    top = top.min(yy as i64);
                    bottom = bottom.max(yy as i64);
                    break;
                }
            }
        }
    }
    (top <= bottom).then(|| (bottom - top + 1) as f64)
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// Entities drawn from [`UiDrawList`].
#[derive(Component, Clone, Copy)]
pub struct UiDrawn;

/// z of the menu layer on the GUI camera (above the HUD, which stays below 500).
pub const Z_MENU: f32 = 500.0;

/// What is on screen: per draw command, its entities; plus replaced entities still showing.
#[derive(Resource, Default)]
pub(crate) struct DrawnList {
    slots: Vec<(DrawCmd, Vec<Entity>)>,
    /// replaced entities and the frames they have been kept
    retiring: Vec<(Entity, u32)>,
    material: Option<Handle<ColorMaterial>>,
}

fn align(a: u8, size: f64) -> f64 {
    match a {
        1 => -size / 2.0,
        2 => -size,
        _ => 0.0,
    }
}

/// GameMaker blends in sRGB space, Bevy in linear space, so a black veil or shadow of alpha `a`
/// darkens less here. `1 - (1 - a)^1.8` makes it darken like the original over the dark and
/// mid-tone backgrounds the menus sit on (exact matching depends on the background: the sRGB
/// curve is not a pure power).
pub fn blend_alpha(col: f64, a: f64) -> f64 {
    if col == 0.0 {
        1.0 - (1.0 - a.clamp(0.0, 1.0)).powf(1.8)
    } else {
        a
    }
}

/// Update the GUI entities to the draw list: commands that did not change keep their entities;
/// a changed command gets new ones. Replaced entities stay three more frames, because freshly
/// spawned sprites and meshes are not drawn in their first frame(s) and removing the old ones at
/// once made the menu blink.
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_draw_list(
    mut commands: Commands,
    list: Res<UiDrawList>,
    mut drawn: ResMut<DrawnList>,
    assets: Res<GameAssets>,
    fonts: Option<Res<UiFonts>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    drawn.retiring.retain_mut(|(e, age)| {
        *age += 1;
        if *age >= 3 {
            commands.entity(*e).despawn();
            false
        } else {
            true
        }
    });
    let Some(fonts) = fonts else { return };
    let unchanged = drawn.slots.len() == list.0.len()
        && drawn.slots.iter().zip(&list.0).all(|((a, _), b)| a == b);
    if unchanged {
        return;
    }
    let white = drawn
        .material
        .get_or_insert_with(|| {
            materials.add(ColorMaterial {
                color: Color::WHITE,
                alpha_mode: bevy::sprite_render::AlphaMode2d::Blend,
                ..default()
            })
        })
        .clone();
    let old = std::mem::take(&mut drawn.slots);
    let mut retire = Vec::new();
    let mut slots = Vec::with_capacity(list.0.len());
    let mut old_iter = old.into_iter();
    for (i, cmd) in list.0.iter().enumerate() {
        if let Some((prev, ents)) = old_iter.next() {
            if &prev == cmd {
                slots.push((prev, ents));
                continue;
            }
            retire.extend(ents);
        }
        let z = Z_MENU + i as f32 * 0.01;
        let ents = spawn_cmd(&mut commands, cmd, z, &assets, &fonts, &mut meshes, &white);
        slots.push((cmd.clone(), ents));
    }
    for (_, ents) in old_iter {
        retire.extend(ents);
    }
    drawn.slots = slots;
    drawn.retiring.extend(retire.into_iter().map(|e| (e, 0)));
}

/// The entities of one draw command.
fn spawn_cmd(
    commands: &mut Commands,
    cmd: &DrawCmd,
    z: f32,
    assets: &GameAssets,
    fonts: &UiFonts,
    meshes: &mut Assets<Mesh>,
    white: &Handle<ColorMaterial>,
) -> Vec<Entity> {
    let layer = RenderLayers::layer(GUI_LAYER);
    let mut out = Vec::new();
    {
        let tag = UiDrawn;
        match cmd {
            DrawCmd::Rect {
                x1,
                y1,
                x2,
                y2,
                col,
                alpha,
                outline,
            } => {
                let c = gm_colour(*col, blend_alpha(*col, *alpha));
                let mut rect = |x: f64, y: f64, w: f64, h: f64| {
                    out.push(commands.spawn((
                        Sprite::from_color(c, Vec2::new(w.max(0.0) as f32, h.max(0.0) as f32)),
                        Anchor::TOP_LEFT,
                        Transform::from_xyz(x as f32, -y as f32, z),
                        layer.clone(),
                        tag,
                    )).id());
                };
                let (l, t) = (x1.min(*x2), y1.min(*y2));
                let (w, h) = ((x2 - x1).abs(), (y2 - y1).abs());
                if *outline {
                    rect(l, t, w + 1.0, 1.0);
                    rect(l, t + h, w + 1.0, 1.0);
                    rect(l, t, 1.0, h + 1.0);
                    rect(l + w, t, 1.0, h + 1.0);
                } else {
                    rect(l, t, w + 1.0, h + 1.0);
                }
            }
            DrawCmd::Text {
                x,
                y,
                s,
                sc,
                col,
                alpha,
                font,
                halign,
                valign,
            } => {
                let colour = gm_colour(*col, blend_alpha(*col, *alpha));
                let face = fonts.face(*font);
                let h_total = fonts.height(*font, s);
                let lh = h_total / s.split('\n').count().max(1) as f64;
                let y0 = align(*valign, h_total);
                for (li, line) in s.split('\n').enumerate() {
                    let lw = fonts.width(*font, line);
                    let lx = x + align(*halign, lw) * sc;
                    let ly = y + (y0 + li as f64 * lh) * sc;
                    match face {
                        Face::Gm(fi) => {
                            let Some(f) = assets.gm_fonts.get(*fi) else { continue };
                            let mut pen = 0.0f64;
                            for ch in line.chars() {
                                let Some(g) = f.glyphs.get(&(ch as u32)) else { continue };
                                if g.w > 0.0 && g.h > 0.0 {
                                    let min = f.frame.min + Vec2::new(g.x, g.y);
                                    out.push(commands.spawn((
                                        Sprite {
                                            image: f.page.clone(),
                                            rect: Some(Rect::from_corners(
                                                min,
                                                min + Vec2::new(g.w, g.h),
                                            )),
                                            custom_size: Some(Vec2::new(
                                                (g.w as f64 * sc) as f32,
                                                (g.h as f64 * sc) as f32,
                                            )),
                                            color: colour,
                                            ..default()
                                        },
                                        Anchor::TOP_LEFT,
                                        Transform::from_xyz(
                                            (lx + (pen + g.offset as f64) * sc) as f32,
                                            -ly as f32,
                                            z,
                                        ),
                                        layer.clone(),
                                        tag,
                                    )).id());
                                }
                                pen += g.shift as f64;
                            }
                        }
                        Face::Ttf { id, size, .. } => {
                            out.push(commands.spawn((
                                Text2d::new(line.to_string()),
                                TextFont::from_font_size((size * sc) as f32)
                                    .with_font(assets.font(*id)),
                                LineHeight::Px((lh * sc) as f32),
                                TextLayout::default(),
                                TextColor(colour),
                                Anchor::TOP_LEFT,
                                Transform::from_xyz(lx as f32, -ly as f32, z),
                                layer.clone(),
                                tag,
                            )).id());
                        }
                    }
                }
            }
            DrawCmd::Tris(verts) => {
                if verts.len() < 3 {
                    return out;
                }
                let verts: Vec<Vert> = verts
                    .iter()
                    .map(|v| Vert {
                        alpha: blend_alpha(v.col, v.alpha),
                        ..*v
                    })
                    .collect();
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                );
                colour_mesh(&mut mesh, &verts);
                out.push(commands.spawn((
                    Mesh2d(meshes.add(mesh)),
                    MeshMaterial2d(white.clone()),
                    Transform::from_xyz(0.0, 0.0, z),
                    layer.clone(),
                    tag,
                )).id());
            }
        }
    }
    out
}
