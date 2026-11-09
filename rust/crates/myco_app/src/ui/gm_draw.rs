//! A GameMaker-style immediate painter for the GUI layer (`draw_set_colour`, `draw_line_width`,
//! `draw_circle`, `draw_rectangle`, `draw_sprite_part_ext`, `draw_text_transformed`, ...), in GUI
//! coordinates (y down, view-sized, as `display_set_gui_size(view_w, view_h)`).
//!
//! UI code fills a [`Gm`] each frame and stores its items in [`GuiDraw`] under an order key;
//! [`GuiDrawPlugin`] turns them into pooled Bevy entities on the GUI layer, painter's order mapped
//! to increasing z inside [`Z_GUI_BASE`, `Z_GUI_TOP`). Coloured geometry is batched into one
//! [`GmBlend`] mesh per run (blend mode `Alpha`), sprite parts and bitmap-font glyphs become
//! `Sprite`s (written with `set_if_neq`, see Task 16 concern 1), TTF text becomes `Text2d`.
//!
//! Text follows `game/gm/draw.lua`: font -1 is `fnt_gm_20`, lines wrap at spaces
//! (`text_layout.wrap`), halign/valign offset each line, glyph quads sit at `x + offset` on the
//! line top. A TTF font (`locale_font_fit`) is measured with `ab_glyph` and drawn with `Text2d`.

use crate::assets::{GameAssets, GmFont};
use crate::camera::GUI_LAYER;
use crate::render::blend::{colour_mesh, empty_colour_mesh, GmBlend, GmBlendMode, Vert};
use crate::render::sprite::{gm_colour_f, part_sprite};
use ab_glyph::{Font as _, FontArc, ScaleFont as _};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::text::{TextBounds, TextLayout};
use std::collections::BTreeMap;

/// GUI z band used by [`GuiDraw`] (T17's HUD may use z below it).
pub const Z_GUI_BASE: f32 = 100.0;
pub const Z_GUI_TOP: f32 = 990.0;

/// GameMaker alignment: 0 left/top, 1 centre/middle, 2 right/bottom.
pub const FA_LEFT: u8 = 0;
pub const FA_CENTER: u8 = 1;
pub const FA_RIGHT: u8 = 2;
pub const FA_TOP: u8 = 0;
pub const FA_MIDDLE: u8 = 1;

/// A TTF UI font as `font_add(file, size)` gives it: the Bevy handle plus `ab_glyph` metrics.
#[derive(Clone)]
pub struct TtfFont {
    pub handle: Handle<Font>,
    pub font: FontArc,
    pub size: f32,
}

impl TtfFont {
    fn scaled(&self) -> ab_glyph::PxScaleFont<&FontArc> {
        self.font.as_scaled(ab_glyph::PxScale::from(self.size))
    }

    /// `font:getWidth(s)`: advances plus kerning.
    pub fn width(&self, s: &str) -> f64 {
        let f = self.scaled();
        let mut w = 0.0f32;
        let mut prev = None;
        for ch in s.chars() {
            let id = f.glyph_id(ch);
            if let Some(p) = prev {
                w += f.kern(p, id);
            }
            w += f.h_advance(id);
            prev = Some(id);
        }
        w as f64
    }

    /// `font:getHeight()`.
    pub fn height(&self) -> f64 {
        self.scaled().height().round() as f64
    }
}

/// The font `draw_set_font` selected.
#[derive(Clone)]
pub enum UiFont {
    Gm(String),
    Ttf(TtfFont),
}

/// One drawn thing, in painter's order.
#[derive(Clone, Debug)]
pub enum DrawItem {
    /// a triangle list in GUI coordinates
    Tris(Vec<Vert>),
    /// a textured axis-aligned quad: `rect` on `image`, top-left at (x, y), size (w, h)
    Quad {
        image: Handle<Image>,
        rect: Rect,
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        col: f64,
        alpha: f64,
    },
    /// one line of TTF text, top-left at (x, y), scaled by `scale`
    Text {
        text: String,
        font: Handle<Font>,
        size: f32,
        x: f64,
        y: f64,
        scale: f64,
        col: f64,
        alpha: f64,
    },
}

/// Bevy blends in linear light, GameMaker/LÖVE in sRGB: a translucent black veil (the tree
/// screen's 0.86 backdrop) looks much lighter in linear space. For pure black the sRGB result
/// `dst·(1 − a)` is reproduced exactly by `a' = 1 − (1 − a)^2.2`; other colours keep `a`.
///
/// TODO(R21): only corrects `col == 0`. T13 switches blending to sRGB space (ruling R21); once
/// that lands, remove `gamma_alpha` (it would double-correct the veil).
pub fn gamma_alpha(col: f64, alpha: f64) -> f64 {
    if col == 0.0 {
        1.0 - (1.0 - alpha.clamp(0.0, 1.0)).powf(2.2)
    } else {
        alpha
    }
}

/// Painter state and output for one frame.
pub struct Gm<'a> {
    pub assets: &'a GameAssets,
    pub items: Vec<DrawItem>,
    pub colour: f64,
    pub alpha: f64,
    pub halign: u8,
    pub valign: u8,
    pub font: UiFont,
    /// `matrix_set(matrix_world, matrix_build(x, y, 0, 0, 0, 0, s, s, 1))`
    matrix: Option<(f64, f64, f64)>,
}

impl<'a> Gm<'a> {
    pub fn new(assets: &'a GameAssets) -> Self {
        Gm {
            assets,
            items: Vec::new(),
            colour: 16777215.0,
            alpha: 1.0,
            halign: FA_LEFT,
            valign: FA_TOP,
            font: UiFont::Gm("fnt_gm_20".into()),
            matrix: None,
        }
    }

    pub fn set_colour(&mut self, c: f64) {
        self.colour = c;
    }
    pub fn set_alpha(&mut self, a: f64) {
        self.alpha = a;
    }

    /// World matrix (translate + uniform scale) for following primitives; `None` resets.
    pub fn set_matrix(&mut self, m: Option<(f64, f64, f64)>) {
        self.matrix = m;
    }

    fn tf(&self, x: f64, y: f64) -> (f64, f64) {
        match self.matrix {
            Some((tx, ty, s)) => (tx + x * s, ty + y * s),
            None => (x, y),
        }
    }

    /// Append one triangle (already coloured vertices, local coordinates).
    pub fn tri(&mut self, v: [(f64, f64, f64, f64); 3]) {
        let verts: Vec<Vert> = v
            .iter()
            .map(|&(x, y, col, alpha)| {
                let (x, y) = self.tf(x, y);
                Vert {
                    x,
                    y,
                    col,
                    alpha: gamma_alpha(col, alpha),
                }
            })
            .collect();
        match self.items.last_mut() {
            Some(DrawItem::Tris(t)) => t.extend(verts),
            _ => self.items.push(DrawItem::Tris(verts)),
        }
    }

    fn tri_c(&mut self, a: (f64, f64), b: (f64, f64), c: (f64, f64)) {
        let (col, al) = (self.colour, self.alpha);
        self.tri([
            (a.0, a.1, col, al),
            (b.0, b.1, col, al),
            (c.0, c.1, col, al),
        ]);
    }

    fn quad_c(&mut self, a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) {
        self.tri_c(a, b, c);
        self.tri_c(a, c, d);
    }

    /// `draw_line_width` (LÖVE line: a quad of width `w`, butt ends).
    pub fn line_width(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, w: f64) {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            return;
        }
        let (nx, ny) = (-dy / len * w / 2.0, dx / len * w / 2.0);
        self.quad_c(
            (x1 + nx, y1 + ny),
            (x2 + nx, y2 + ny),
            (x2 - nx, y2 - ny),
            (x1 - nx, y1 - ny),
        );
    }

    /// `draw_line` (1 px).
    pub fn line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        self.line_width(x1, y1, x2, y2, 1.0);
    }

    fn circle_segments(r: f64) -> usize {
        (r.abs().sqrt() * 4.0).clamp(12.0, 96.0) as usize
    }

    /// `draw_circle(x, y, r, outline)`.
    pub fn circle(&mut self, x: f64, y: f64, r: f64, outline: bool) {
        let n = Self::circle_segments(r);
        let pt = |i: usize| {
            let a = i as f64 / n as f64 * std::f64::consts::TAU;
            (x + a.cos() * r, y + a.sin() * r)
        };
        for i in 0..n {
            let (p, q) = (pt(i), pt(i + 1));
            if outline {
                // 1 px ring centred on the radius, like love's line circle
                let (ip, iq) = (
                    (x + (p.0 - x) * (r - 0.5) / r, y + (p.1 - y) * (r - 0.5) / r),
                    (x + (q.0 - x) * (r - 0.5) / r, y + (q.1 - y) * (r - 0.5) / r),
                );
                let (op, oq) = (
                    (x + (p.0 - x) * (r + 0.5) / r, y + (p.1 - y) * (r + 0.5) / r),
                    (x + (q.0 - x) * (r + 0.5) / r, y + (q.1 - y) * (r + 0.5) / r),
                );
                self.quad_c(ip, op, oq, iq);
            } else {
                self.tri_c((x, y), p, q);
            }
        }
    }

    /// `draw_rectangle(x1, y1, x2, y2, outline)` (fill covers the far pixel, as GameMaker).
    pub fn rectangle(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, outline: bool) {
        if outline {
            self.quad_c(
                (x1 - 0.5, y1 - 0.5),
                (x2 + 0.5, y1 - 0.5),
                (x2 + 0.5, y1 + 0.5),
                (x1 - 0.5, y1 + 0.5),
            );
            self.quad_c(
                (x1 - 0.5, y2 - 0.5),
                (x2 + 0.5, y2 - 0.5),
                (x2 + 0.5, y2 + 0.5),
                (x1 - 0.5, y2 + 0.5),
            );
            self.quad_c(
                (x1 - 0.5, y1 + 0.5),
                (x1 + 0.5, y1 + 0.5),
                (x1 + 0.5, y2 - 0.5),
                (x1 - 0.5, y2 - 0.5),
            );
            self.quad_c(
                (x2 - 0.5, y1 + 0.5),
                (x2 + 0.5, y1 + 0.5),
                (x2 + 0.5, y2 - 0.5),
                (x2 - 0.5, y2 - 0.5),
            );
        } else {
            self.quad_c(
                (x1, y1),
                (x2 + 1.0, y1),
                (x2 + 1.0, y2 + 1.0),
                (x1, y2 + 1.0),
            );
        }
    }

    /// `draw_roundrect_ext(x1, y1, x2, y2, rx, ry, false)` (filled; love corner radius rx/2).
    pub fn roundrect_fill(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, rx: f64, ry: f64) {
        let rx = (rx / 2.0).min((x2 - x1) / 2.0).max(0.0);
        let ry = (ry / 2.0).min((y2 - y1) / 2.0).max(0.0);
        let (cx, cy) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
        let corners = [
            (x2 - rx, y1 + ry, -90.0f64),
            (x2 - rx, y2 - ry, 0.0),
            (x1 + rx, y2 - ry, 90.0),
            (x1 + rx, y1 + ry, 180.0),
        ];
        let mut pts = Vec::new();
        for (ox, oy, a0) in corners {
            for k in 0..=6 {
                let a = (a0 + 90.0 * k as f64 / 6.0).to_radians();
                pts.push((ox + a.cos() * rx, oy + a.sin() * ry));
            }
        }
        for i in 0..pts.len() {
            let (p, q) = (pts[i], pts[(i + 1) % pts.len()]);
            self.tri_c((cx, cy), p, q);
        }
    }

    /// `draw_sprite_ext(spr, sub, x, y, xs, ys, 0, col, alpha)` (no rotation).
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_ext(
        &mut self,
        spr: &str,
        sub: usize,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        col: f64,
        alpha: f64,
    ) {
        let Some(def) = self.assets.try_sprite(spr) else {
            return;
        };
        let Some(fr) = def.frames.get(sub % def.frames.len().max(1)) else {
            return;
        };
        let ox = (def.origin.x - fr.trim_offset.x) as f64;
        let oy = (def.origin.y - fr.trim_offset.y) as f64;
        let size = fr.rect.size();
        self.items.push(DrawItem::Quad {
            image: fr.page.clone(),
            rect: fr.rect,
            x: x - ox * xs,
            y: y - oy * ys,
            w: size.x as f64 * xs,
            h: size.y as f64 * ys,
            col,
            alpha,
        });
    }

    /// `draw_sprite_part_ext(spr, sub, left, top, w, h, x, y, xs, ys, col, alpha)`.
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_part_ext(
        &mut self,
        spr: &str,
        sub: usize,
        left: f64,
        top: f64,
        w: f64,
        h: f64,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        col: f64,
        alpha: f64,
    ) {
        let Some(def) = self.assets.try_sprite(spr) else {
            return;
        };
        let Some((s, dx, dy)) = part_sprite(def, sub, left, top, w, h) else {
            return;
        };
        let rect = s.rect.unwrap_or_default();
        self.items.push(DrawItem::Quad {
            image: s.image,
            rect,
            x: x + dx * xs,
            y: y + dy * ys,
            w: rect.width() as f64 * xs,
            h: rect.height() as f64 * ys,
            col,
            alpha,
        });
    }

    // ---- text ---------------------------------------------------------------------------

    fn gm_font(&self) -> Option<&'a GmFont> {
        match &self.font {
            UiFont::Gm(name) => self
                .assets
                .gm_font(name)
                .or_else(|| self.assets.gm_font("fnt_gm_20")),
            UiFont::Ttf(_) => None,
        }
    }

    fn glyph_width(&self, s: &str) -> f64 {
        match &self.font {
            UiFont::Ttf(t) => t.width(s),
            UiFont::Gm(_) => {
                let Some(f) = self.gm_font() else { return 0.0 };
                s.chars()
                    .filter_map(|c| f.glyphs.get(&(c as u32)))
                    .map(|g| g.shift as f64)
                    .sum()
            }
        }
    }

    fn line_height(&self) -> f64 {
        match &self.font {
            UiFont::Ttf(t) => t.height(),
            UiFont::Gm(_) => {
                let Some(f) = self.gm_font() else { return 0.0 };
                if f.line_height > 0.0 {
                    f.line_height
                } else {
                    f.glyphs.values().map(|g| g.h as f64).fold(0.0, f64::max)
                }
            }
        }
    }

    /// `text_layout.wrap`: split on newlines, wrap at spaces to `max_w` (< 0: no wrap).
    pub fn wrap(&self, s: &str, max_w: f64) -> Vec<String> {
        let mut out = Vec::new();
        for para in s.split('\n') {
            if max_w < 0.0 || self.glyph_width(para) <= max_w {
                out.push(para.to_string());
                continue;
            }
            let mut line = String::new();
            for word in para.split_whitespace() {
                let cand = if line.is_empty() {
                    word.to_string()
                } else {
                    format!("{line} {word}")
                };
                if !line.is_empty() && self.glyph_width(&cand) > max_w {
                    out.push(std::mem::replace(&mut line, word.to_string()));
                } else {
                    line = cand;
                }
            }
            out.push(line);
        }
        out
    }

    pub fn string_width(&self, s: &str) -> f64 {
        self.wrap(s, -1.0)
            .iter()
            .map(|l| self.glyph_width(l))
            .fold(0.0, f64::max)
    }
    pub fn string_height(&self, s: &str) -> f64 {
        self.line_height() * self.wrap(s, -1.0).len() as f64
    }
    pub fn string_width_ext(&self, s: &str, _sep: f64, w: f64) -> f64 {
        self.wrap(s, w)
            .iter()
            .map(|l| self.glyph_width(l))
            .fold(0.0, f64::max)
    }
    pub fn string_height_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
        let lh = if sep >= 0.0 { sep } else { self.line_height() };
        lh * self.wrap(s, w).len() as f64
    }

    /// `draw_text_transformed(x, y, s, xs, ys, 0)`.
    pub fn text_transformed(&mut self, x: f64, y: f64, s: &str, xs: f64, ys: f64) {
        self.text_lines(x, y, s, -1.0, -1.0, xs, ys);
    }

    /// `draw_text_ext_transformed(x, y, s, sep, w, xs, ys, 0)`.
    #[allow(clippy::too_many_arguments)]
    pub fn text_ext_transformed(
        &mut self,
        x: f64,
        y: f64,
        s: &str,
        sep: f64,
        w: f64,
        xs: f64,
        ys: f64,
    ) {
        self.text_lines(x, y, s, sep, w, xs, ys);
    }

    #[allow(clippy::too_many_arguments)]
    fn text_lines(&mut self, x: f64, y: f64, s: &str, sep: f64, max_w: f64, xs: f64, ys: f64) {
        let lines = self.wrap(s, max_w);
        let lh = if sep >= 0.0 { sep } else { self.line_height() };
        let total = lh * lines.len() as f64;
        let y0 = match self.valign {
            FA_MIDDLE => -total / 2.0,
            2 => -total,
            _ => 0.0,
        };
        let (col, alpha) = (self.colour, self.alpha);
        for (i, line) in lines.iter().enumerate() {
            let lw = self.glyph_width(line);
            let x0 = match self.halign {
                FA_CENTER => -lw / 2.0,
                FA_RIGHT => -lw,
                _ => 0.0,
            };
            let ly = y0 + i as f64 * lh;
            match &self.font {
                UiFont::Ttf(t) => {
                    if !line.is_empty() {
                        self.items.push(DrawItem::Text {
                            text: line.clone(),
                            font: t.handle.clone(),
                            size: t.size,
                            x: x + x0 * xs,
                            y: y + ly * ys,
                            scale: xs,
                            col,
                            alpha,
                        });
                    }
                }
                UiFont::Gm(_) => {
                    let Some(f) = self.gm_font() else { continue };
                    let mut gx = x0;
                    for ch in line.chars() {
                        let Some(g) = f.glyphs.get(&(ch as u32)) else {
                            continue;
                        };
                        if g.w > 0.0 && g.h > 0.0 {
                            let min = f.frame.min + Vec2::new(g.x, g.y);
                            self.items.push(DrawItem::Quad {
                                image: f.page.clone(),
                                rect: Rect::from_corners(min, min + Vec2::new(g.w, g.h)),
                                x: x + (gx + g.offset as f64) * xs,
                                y: y + ly * ys,
                                w: g.w as f64 * xs,
                                h: g.h as f64 * ys,
                                col,
                                alpha,
                            });
                        }
                        gx += g.shift as f64;
                    }
                }
            }
        }
    }
}

/// Per-frame GUI draw lists, painted in key order (lower first). UI tasks replace their own key
/// every frame (an empty list hides it).
#[derive(Resource, Default)]
pub struct GuiDraw {
    pub lists: BTreeMap<i32, Vec<DrawItem>>,
}

#[derive(Component)]
struct GuiMeshSlot;
#[derive(Component)]
struct GuiQuadSlot;
#[derive(Component)]
struct GuiTextSlot;

#[derive(Resource, Default)]
struct GuiPools {
    meshes: Vec<(Entity, Handle<Mesh>)>,
    quads: Vec<Entity>,
    texts: Vec<Entity>,
    material: Option<Handle<GmBlend>>,
}

pub struct GuiDrawPlugin;

impl Plugin for GuiDrawPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::render::blend::BlendPlugin>() {
            app.add_plugins(crate::render::blend::BlendPlugin);
        }
        app.init_resource::<GuiDraw>()
            .init_resource::<GuiPools>()
            .add_systems(PostUpdate, paint_gui);
    }
}

fn z_of(i: usize, n: usize) -> f32 {
    let step = ((Z_GUI_TOP - Z_GUI_BASE) / n.max(1) as f32).min(0.01);
    Z_GUI_BASE + i as f32 * step
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn paint_gui(
    mut commands: Commands,
    draw: Res<GuiDraw>,
    mut pools: ResMut<GuiPools>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmBlend>>,
    mut q_mesh: Query<
        (&mut Transform, &mut Visibility),
        (
            With<GuiMeshSlot>,
            Without<GuiQuadSlot>,
            Without<GuiTextSlot>,
        ),
    >,
    mut q_quad: Query<
        (&mut Sprite, &mut Transform, &mut Visibility),
        (
            With<GuiQuadSlot>,
            Without<GuiMeshSlot>,
            Without<GuiTextSlot>,
        ),
    >,
    mut q_text: Query<
        (
            &mut Text2d,
            &mut TextFont,
            &mut TextColor,
            &mut Transform,
            &mut Visibility,
        ),
        (
            With<GuiTextSlot>,
            Without<GuiMeshSlot>,
            Without<GuiQuadSlot>,
        ),
    >,
) {
    let material = pools
        .material
        .get_or_insert_with(|| materials.add(GmBlend::new(GmBlendMode::Alpha)))
        .clone();
    let n: usize = draw.lists.values().map(Vec::len).sum();
    let (mut mi, mut qi, mut ti) = (0usize, 0usize, 0usize);
    let layer = RenderLayers::layer(GUI_LAYER);
    for (i, item) in draw.lists.values().flatten().enumerate() {
        let z = z_of(i, n);
        match item {
            DrawItem::Tris(verts) => {
                if mi == pools.meshes.len() {
                    let h = meshes.add(empty_colour_mesh());
                    let e = commands
                        .spawn((
                            Mesh2d(h.clone()),
                            MeshMaterial2d(material.clone()),
                            Transform::from_xyz(0.0, 0.0, z),
                            Visibility::Visible,
                            layer.clone(),
                            GuiMeshSlot,
                        ))
                        .id();
                    pools.meshes.push((e, h));
                }
                let (e, h) = pools.meshes[mi].clone();
                if let Some(m) = meshes.get_mut(&h) {
                    colour_mesh(m.into_inner(), verts);
                }
                if let Ok((mut tf, mut vis)) = q_mesh.get_mut(e) {
                    tf.translation.z = z;
                    vis.set_if_neq(Visibility::Visible);
                }
                mi += 1;
            }
            DrawItem::Quad {
                image,
                rect,
                x,
                y,
                w,
                h,
                col,
                alpha,
            } => {
                let sprite = Sprite {
                    image: image.clone(),
                    rect: Some(*rect),
                    color: gm_colour_f(*col, *alpha),
                    custom_size: Some(Vec2::new(w.abs() as f32, h.abs() as f32)),
                    flip_x: *w < 0.0,
                    flip_y: *h < 0.0,
                    ..default()
                };
                let pos = Vec3::new(x.min(x + w) as f32, -(y.min(y + h) as f32), z);
                if qi == pools.quads.len() {
                    let e = commands
                        .spawn((
                            sprite.clone(),
                            Anchor::TOP_LEFT,
                            Transform::from_translation(pos),
                            Visibility::Visible,
                            layer.clone(),
                            GuiQuadSlot,
                        ))
                        .id();
                    pools.quads.push(e);
                } else if let Ok((mut s, mut tf, mut vis)) = q_quad.get_mut(pools.quads[qi]) {
                    s.set_if_neq(sprite);
                    let t = Transform::from_translation(pos);
                    tf.set_if_neq(t);
                    vis.set_if_neq(Visibility::Visible);
                }
                qi += 1;
            }
            DrawItem::Text {
                text,
                font,
                size,
                x,
                y,
                scale,
                col,
                alpha,
            } => {
                let pos = Transform::from_xyz(*x as f32, -(*y as f32), z).with_scale(Vec3::new(
                    *scale as f32,
                    *scale as f32,
                    1.0,
                ));
                let tfont = TextFont::from_font_size(*size).with_font(font.clone());
                let tcol = TextColor(gm_colour_f(*col, *alpha));
                if ti == pools.texts.len() {
                    let e = commands
                        .spawn((
                            Text2d::new(text.clone()),
                            tfont,
                            tcol,
                            TextLayout::default(),
                            TextBounds::UNBOUNDED,
                            Anchor::TOP_LEFT,
                            pos,
                            Visibility::Visible,
                            layer.clone(),
                            GuiTextSlot,
                        ))
                        .id();
                    pools.texts.push(e);
                } else if let Ok((mut t, mut f, mut c, mut tf, mut vis)) =
                    q_text.get_mut(pools.texts[ti])
                {
                    if t.0 != *text {
                        t.0 = text.clone();
                    }
                    if f.font_size != tfont.font_size || f.font != tfont.font {
                        *f = tfont;
                    }
                    c.set_if_neq(tcol);
                    tf.set_if_neq(pos);
                    vis.set_if_neq(Visibility::Visible);
                }
                ti += 1;
            }
        }
    }
    for (e, _) in &pools.meshes[mi..] {
        if let Ok((_, mut vis)) = q_mesh.get_mut(*e) {
            vis.set_if_neq(Visibility::Hidden);
        }
    }
    for e in &pools.quads[qi..] {
        if let Ok((_, _, mut vis)) = q_quad.get_mut(*e) {
            vis.set_if_neq(Visibility::Hidden);
        }
    }
    for e in &pools.texts[ti..] {
        if let Ok((_, _, _, _, mut vis)) = q_text.get_mut(*e) {
            vis.set_if_neq(Visibility::Hidden);
        }
    }
}
