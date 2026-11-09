//! A GameMaker-style painter for world drawing: [`QuadBatch`] records `draw_sprite_ext`,
//! `draw_sprite_part_ext`, `draw_rectangle(_colour)`, `draw_line_width_colour` and surface draws
//! in painter's order, in world coordinates (y down), with the GameMaker blend mode and texture
//! filter current at the call.
//!
//! Consecutive draws sharing (texture, blend, filter, shader mode) form a [`Run`]; [`BatchPool`]
//! keeps one pooled `Mesh2d` entity per run with a [`GmQuad`] material, at increasing z from the
//! owner's base. One run = one draw call, and the painter's order is exact.
//!
//! [`GmQuad`] (shaders/gm_quad.wesl) samples with `textureLoad` (nearest or bilinear per run,
//! like `gpu_set_tex_filter`), multiplies by the vertex colour, can run `sh_glow_blur`, and encodes
//! to sRGB under `SRGB_OUTPUT` (blending in sRGB space, `camera::COMPOSITING`). Blend states follow
//! the LÖVE port (`game/gm/draw.lua`): alpha = "alpha"/alphamultiply, add = "add"/alphamultiply,
//! replace = `gpu_set_blendmode_ext(bm_one, bm_zero)`.

use crate::assets::{GameAssets, SpriteDef};
use crate::render::sprite::{gm_colour_f, part_quad};
use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d,
    RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureDimension, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};

/// Blend mode of a run.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum QuadBlend {
    /// `bm_normal`
    #[default]
    Alpha,
    /// `bm_add`
    Add,
    /// `gpu_set_blendmode_ext(bm_one, bm_zero)`
    Replace,
    /// `gpu_set_blendmode_ext(bm_one, bm_inv_src_alpha)` (premultiplied over, every channel)
    Premul,
}

impl QuadBlend {
    pub fn state(self) -> BlendState {
        let c = |s, d| BlendComponent {
            src_factor: s,
            dst_factor: d,
            operation: BlendOperation::Add,
        };
        match self {
            QuadBlend::Alpha => BlendState {
                color: c(BlendFactor::SrcAlpha, BlendFactor::OneMinusSrcAlpha),
                alpha: c(BlendFactor::One, BlendFactor::OneMinusSrcAlpha),
            },
            QuadBlend::Add => BlendState {
                color: c(BlendFactor::SrcAlpha, BlendFactor::One),
                alpha: c(BlendFactor::Zero, BlendFactor::One),
            },
            QuadBlend::Replace => BlendState {
                color: c(BlendFactor::One, BlendFactor::Zero),
                alpha: c(BlendFactor::One, BlendFactor::Zero),
            },
            QuadBlend::Premul => BlendState {
                color: c(BlendFactor::One, BlendFactor::OneMinusSrcAlpha),
                alpha: c(BlendFactor::One, BlendFactor::OneMinusSrcAlpha),
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GmQuadKey {
    pub blend: QuadBlend,
}

impl From<&GmQuad> for GmQuadKey {
    fn from(m: &GmQuad) -> Self {
        GmQuadKey { blend: m.blend }
    }
}

/// Textured, vertex-coloured quads with a GameMaker blend mode (see the module docs).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(GmQuadKey)]
pub struct GmQuad {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    /// x: mode (0 sprite, 1 glow blur), y: bilinear filter, zw: blur step in texels
    #[uniform(2)]
    pub params: Vec4,
    pub blend: QuadBlend,
}

const SHADER: &str = "embedded://myco_app/render/shaders/gm_quad.wesl";

impl Material2d for GmQuad {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Path(SHADER.into())
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(frag) = descriptor.fragment.as_mut() {
            for t in frag.targets.iter_mut().flatten() {
                t.blend = Some(key.bind_group_data.blend.state());
            }
        }
        Ok(())
    }
}

/// The 1×1 white texture untextured draws (rectangles, lines) sample.
#[derive(Resource, Clone)]
pub struct WhiteTexture(pub Handle<Image>);

pub struct BatchPlugin;

impl Plugin for BatchPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<Material2dPlugin<GmQuad>>() {
            return;
        }
        embedded_asset!(app, "shaders/gm_quad.wesl");
        app.add_plugins(Material2dPlugin::<GmQuad>::default());
        let mut images = app.world_mut().resource_mut::<Assets<Image>>();
        let white = images.add(Image::new_fill(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[255, 255, 255, 255],
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        ));
        app.insert_resource(WhiteTexture(white));
    }
}

/// One vertex: world position (y down), texel coordinates, encoded RGBA (see [`lin`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QVert {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub col: [f32; 4],
}

/// Consecutive draws with one texture, blend, filter and shader mode.
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    /// `None`: the white texture
    pub tex: Option<Handle<Image>>,
    pub blend: QuadBlend,
    pub linear: bool,
    /// `Some(step)`: the `sh_glow_blur` pass with this texel step
    pub blur: Option<[f32; 2]>,
    /// six vertices per quad
    pub verts: Vec<QVert>,
}

impl Run {
    pub fn quads(&self) -> usize {
        self.verts.len() / 6
    }
}

/// GameMaker colour (BGR) + alpha as the gamma-encoded RGBA vertex colour [`GmQuad`] expects
/// (interpolated across a quad in encoded space, like GameMaker; the shader decodes it when the
/// camera composites in linear).
pub fn lin(col: f64, alpha: f64) -> [f32; 4] {
    gm_colour_f(col, alpha).to_srgba().to_f32_array()
}

/// The painter. Positions are world coordinates (y down).
#[derive(Clone, Debug, Default)]
pub struct QuadBatch {
    pub runs: Vec<Run>,
    pub blend: QuadBlend,
    pub linear: bool,
    pub blur: Option<[f32; 2]>,
}

impl QuadBatch {
    pub fn new() -> Self {
        Self::default()
    }

    /// `gpu_set_blendmode` / `_ext` (the modes the world uses).
    pub fn set_blend(&mut self, b: QuadBlend) {
        self.blend = b;
    }

    /// `gpu_set_tex_filter`
    pub fn set_filter(&mut self, linear: bool) {
        self.linear = linear;
    }

    /// `shader_set(sh_glow_blur)` + `u_step` in texels (`None` = `shader_reset`)
    pub fn set_blur(&mut self, step: Option<[f32; 2]>) {
        self.blur = step;
    }

    pub fn quad_count(&self) -> usize {
        self.runs.iter().map(Run::quads).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    fn run_for(&mut self, tex: Option<&Handle<Image>>) -> &mut Run {
        let id = tex.map(|t| t.id());
        let same = self.runs.last().is_some_and(|r| {
            r.tex.as_ref().map(|t| t.id()) == id
                && r.blend == self.blend
                && r.linear == self.linear
                && r.blur == self.blur
        });
        if !same {
            self.runs.push(Run {
                tex: tex.cloned(),
                blend: self.blend,
                linear: self.linear,
                blur: self.blur,
                verts: Vec::new(),
            });
        }
        self.runs.last_mut().unwrap()
    }

    /// A quad: corners and texel coordinates in order top-left, top-right, bottom-right,
    /// bottom-left, one colour per corner.
    pub fn quad_cols(
        &mut self,
        tex: Option<&Handle<Image>>,
        p: [[f64; 2]; 4],
        uv: [[f64; 2]; 4],
        col: [[f32; 4]; 4],
    ) {
        let v = |i: usize| QVert {
            pos: [p[i][0] as f32, p[i][1] as f32],
            uv: [uv[i][0] as f32, uv[i][1] as f32],
            col: col[i],
        };
        let r = self.run_for(tex);
        r.verts.extend([v(0), v(1), v(2), v(0), v(2), v(3)]);
    }

    pub fn quad(
        &mut self,
        tex: Option<&Handle<Image>>,
        p: [[f64; 2]; 4],
        uv: [[f64; 2]; 4],
        col: f64,
        alpha: f64,
    ) {
        let c = lin(col, alpha);
        self.quad_cols(tex, p, uv, [c; 4]);
    }

    /// Texels `[sx, sy, sw, sh]` of `img` drawn with its local point (`ox`, `oy`) at (x, y),
    /// scaled, then rotated counter-clockwise by `rot` degrees (`draw_sprite_ext` geometry).
    #[allow(clippy::too_many_arguments)]
    pub fn image_ext(
        &mut self,
        img: &Handle<Image>,
        src: [f64; 4],
        local: [f64; 4],
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        rot: f64,
        col: f64,
        alpha: f64,
    ) {
        let [sx, sy, sw, sh] = src;
        let [lx1, ly1, lx2, ly2] = local;
        let (s, c) = rot.to_radians().sin_cos();
        let tf = |lx: f64, ly: f64| {
            let (ax, ay) = (lx * xs, ly * ys);
            [x + ax * c + ay * s, y - ax * s + ay * c]
        };
        let p = [tf(lx1, ly1), tf(lx2, ly1), tf(lx2, ly2), tf(lx1, ly2)];
        let uv = [[sx, sy], [sx + sw, sy], [sx + sw, sy + sh], [sx, sy + sh]];
        self.quad(Some(img), p, uv, col, alpha);
    }

    /// `draw_sprite_ext(sprite, frame, x, y, xs, ys, rot, col, alpha)`; unknown sprites draw
    /// nothing.
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_ext(
        &mut self,
        assets: &GameAssets,
        name: &str,
        frame: usize,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        rot: f64,
        col: f64,
        alpha: f64,
    ) {
        let Some(def) = assets.try_sprite(name) else {
            return;
        };
        self.def_ext(def, frame, x, y, xs, ys, rot, col, alpha);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn def_ext(
        &mut self,
        def: &SpriteDef,
        frame: usize,
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        rot: f64,
        col: f64,
        alpha: f64,
    ) {
        let Some(fr) = def.frames.get(frame % def.frames.len().max(1)) else {
            return;
        };
        let size = fr.rect.size();
        let (tx, ty) = (
            fr.trim_offset.x as f64 - def.origin.x as f64,
            fr.trim_offset.y as f64 - def.origin.y as f64,
        );
        let (w, h) = (size.x as f64, size.y as f64);
        self.image_ext(
            &fr.page,
            [fr.rect.min.x as f64, fr.rect.min.y as f64, w, h],
            [tx, ty, tx + w, ty + h],
            x,
            y,
            xs,
            ys,
            rot,
            col,
            alpha,
        );
    }

    /// `draw_sprite(sprite, frame, x, y)`
    pub fn sprite(&mut self, assets: &GameAssets, name: &str, frame: usize, x: f64, y: f64) {
        self.sprite_ext(assets, name, frame, x, y, 1.0, 1.0, 0.0, C_WHITE, 1.0);
    }

    /// `draw_sprite_part_ext`: the part `[left, top, w, h]` of the untrimmed frame with its
    /// top-left corner at (x, y), scaled (clipped to the trimmed frame like the LÖVE port).
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_part_ext(
        &mut self,
        assets: &GameAssets,
        name: &str,
        frame: usize,
        part: [f64; 4],
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        col: f64,
        alpha: f64,
    ) {
        self.sprite_general(assets, name, frame, part, x, y, xs, ys, 0.0, col, alpha);
    }

    /// `draw_sprite_general` with one colour: the part `[left, top, w, h]` with its top-left
    /// corner at (x, y), scaled, then rotated counter-clockwise by `rot` degrees about (x, y)
    /// (clipped to the trimmed frame like the LÖVE port).
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_general(
        &mut self,
        assets: &GameAssets,
        name: &str,
        frame: usize,
        part: [f64; 4],
        x: f64,
        y: f64,
        xs: f64,
        ys: f64,
        rot: f64,
        col: f64,
        alpha: f64,
    ) {
        let Some(def) = assets.try_sprite(name) else {
            return;
        };
        let Some(fr) = def.frames.get(frame % def.frames.len().max(1)) else {
            return;
        };
        let size = fr.rect.size();
        let trim = [
            fr.trim_offset.x as f64,
            fr.trim_offset.y as f64,
            size.x as f64,
            size.y as f64,
        ];
        let [left, top, w, h] = part;
        let Some([x1, y1, x2, y2]) = part_quad(trim, left, top, w, h) else {
            return;
        };
        let sx = fr.rect.min.x as f64 + x1 - trim[0];
        let sy = fr.rect.min.y as f64 + y1 - trim[1];
        self.image_ext(
            &fr.page,
            [sx, sy, x2 - x1, y2 - y1],
            [x1 - left, y1 - top, x2 - left, y2 - top],
            x,
            y,
            xs,
            ys,
            rot,
            col,
            alpha,
        );
    }

    /// `draw_sprite_part(sprite, frame, left, top, w, h, x, y)`
    #[allow(clippy::too_many_arguments)]
    pub fn sprite_part(
        &mut self,
        assets: &GameAssets,
        name: &str,
        frame: usize,
        part: [f64; 4],
        x: f64,
        y: f64,
    ) {
        self.sprite_part_ext(assets, name, frame, part, x, y, 1.0, 1.0, C_WHITE, 1.0);
    }

    /// `draw_rectangle_colour(x1, y1, x2, y2, c1..c4, false)` (LÖVE: corners to x2 + 1, y2 + 1)
    #[allow(clippy::too_many_arguments)]
    pub fn rect_colour(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, c: [f64; 4], alpha: f64) {
        let p = [
            [x1, y1],
            [x2 + 1.0, y1],
            [x2 + 1.0, y2 + 1.0],
            [x1, y2 + 1.0],
        ];
        let uv = [[0.5, 0.5]; 4];
        let cols = [
            lin(c[0], alpha),
            lin(c[1], alpha),
            lin(c[2], alpha),
            lin(c[3], alpha),
        ];
        self.quad_cols(None, p, uv, cols);
    }

    /// `draw_rectangle(x1, y1, x2, y2, false)` in `col`
    pub fn rect(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, col: f64, alpha: f64) {
        self.rect_colour(x1, y1, x2, y2, [col; 4], alpha);
    }

    /// `draw_line_width_colour` (LÖVE's gradient quad, `w` wide)
    #[allow(clippy::too_many_arguments)]
    pub fn line_width_colour(
        &mut self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        w: f64,
        c1: f64,
        c2: f64,
        alpha: f64,
    ) {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        if len == 0.0 {
            return;
        }
        let (nx, ny) = (-dy / len * w / 2.0, dx / len * w / 2.0);
        let p = [
            [x1 + nx, y1 + ny],
            [x2 + nx, y2 + ny],
            [x2 - nx, y2 - ny],
            [x1 - nx, y1 - ny],
        ];
        let (a, b) = (lin(c1, alpha), lin(c2, alpha));
        self.quad_cols(None, p, [[0.5, 0.5]; 4], [a, b, b, a]);
    }

    /// Append another batch's runs (keeps its blend/filter per run).
    pub fn extend(&mut self, other: QuadBatch) {
        for r in other.runs {
            let mergeable = self.runs.last().is_some_and(|l| {
                l.tex.as_ref().map(|t| t.id()) == r.tex.as_ref().map(|t| t.id())
                    && l.blend == r.blend
                    && l.linear == r.linear
                    && l.blur == r.blur
            });
            if mergeable {
                self.runs.last_mut().unwrap().verts.extend(r.verts);
            } else {
                self.runs.push(r);
            }
        }
    }
}

pub const C_WHITE: f64 = 16777215.0;

/// Mesh of a run's vertices (Bevy space: y up).
pub fn run_mesh(verts: &[QVert]) -> Mesh {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    fill_mesh(&mut m, verts);
    m
}

fn fill_mesh(m: &mut Mesh, verts: &[QVert]) {
    // the mesh allocator warns about empty meshes: keep one degenerate triangle
    let empty = [QVert {
        pos: [0.0, 0.0],
        uv: [0.0, 0.0],
        col: [0.0; 4],
    }; 3];
    let verts = if verts.is_empty() { &empty[..] } else { verts };
    m.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        verts
            .iter()
            .map(|v| [v.pos[0], -v.pos[1], 0.0])
            .collect::<Vec<_>>(),
    );
    m.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        verts.iter().map(|v| v.uv).collect::<Vec<_>>(),
    );
    m.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        verts.iter().map(|v| v.col).collect::<Vec<_>>(),
    );
}

/// What a slot's material was built from: texture, blend, filter, blur step bits.
type SlotKey = (AssetId<Image>, QuadBlend, bool, Option<[u32; 2]>);

#[derive(Clone, Debug)]
struct Slot {
    entity: Entity,
    mesh: Handle<Mesh>,
    material: Handle<GmQuad>,
    key: Option<SlotKey>,
    /// what the mesh holds now (empty: the degenerate triangle) and the entity's z
    verts: Vec<QVert>,
    z: f32,
}

/// Pooled entities drawing a [`QuadBatch`]: run `i` at z = `base + i·step`.
#[derive(Debug, Default)]
pub struct BatchPool {
    slots: Vec<Slot>,
}

/// What [`BatchPool::sync`] needs from the world.
pub struct BatchTarget<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<GmQuad>,
    pub white: &'a Handle<Image>,
}

impl BatchPool {
    /// Despawn the pooled entities.
    pub fn despawn(&mut self, commands: &mut Commands) {
        for s in self.slots.drain(..) {
            commands.entity(s.entity).despawn();
        }
    }

    /// Draw `batch` this frame (runs beyond it are hidden).
    pub fn sync(
        &mut self,
        t: &mut BatchTarget,
        batch: &QuadBatch,
        base: f32,
        step: f32,
        layers: &RenderLayers,
    ) {
        for (i, run) in batch.runs.iter().enumerate() {
            let tex = run.tex.clone().unwrap_or_else(|| t.white.clone());
            let key = (
                tex.id(),
                run.blend,
                run.linear,
                run.blur.map(|s| [s[0].to_bits(), s[1].to_bits()]),
            );
            let z = base + i as f32 * step;
            if i == self.slots.len() {
                let mesh = t.meshes.add(run_mesh(&run.verts));
                let material = t.materials.add(quad_material(run, tex.clone()));
                let entity = t
                    .commands
                    .spawn((
                        Mesh2d(mesh.clone()),
                        MeshMaterial2d(material.clone()),
                        Transform::from_xyz(0.0, 0.0, z),
                        NoFrustumCulling,
                        layers.clone(),
                    ))
                    .id();
                self.slots.push(Slot {
                    entity,
                    mesh,
                    material,
                    key: Some(key),
                    verts: run.verts.clone(),
                    z,
                });
                continue;
            }
            let slot = &mut self.slots[i];
            slot.update(t, &run.verts, z);
            if slot.key != Some(key) {
                if let Some(mut m) = t.materials.get_mut(&slot.material) {
                    *m = quad_material(run, tex);
                }
                slot.key = Some(key);
            }
        }
        // spare slots stay visible with empty geometry and nothing is rewritten unless it changed:
        // no per-frame component churn (re-specialisation, re-extraction, re-upload)
        for slot in self.slots.iter_mut().skip(batch.runs.len()) {
            let z = slot.z;
            slot.update(t, &[], z);
        }
    }
}

impl Slot {
    /// Write the mesh and the Transform only when they change.
    fn update(&mut self, t: &mut BatchTarget, verts: &[QVert], z: f32) {
        if self.verts != verts {
            if let Some(mut m) = t.meshes.get_mut(&self.mesh) {
                fill_mesh(&mut m, verts);
            }
            self.verts.clear();
            self.verts.extend_from_slice(verts);
        }
        if self.z != z {
            t.commands
                .entity(self.entity)
                .insert(Transform::from_xyz(0.0, 0.0, z));
            self.z = z;
        }
    }
}

fn quad_material(run: &Run, image: Handle<Image>) -> GmQuad {
    let (mode, step) = match run.blur {
        Some(s) => (1.0, s),
        None => (0.0, [0.0, 0.0]),
    };
    GmQuad {
        image,
        params: Vec4::new(mode, if run.linear { 1.0 } else { 0.0 }, step[0], step[1]),
        blend: run.blend,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_split_on_texture_blend_and_filter() {
        let mut b = QuadBatch::new();
        let img = Handle::<Image>::default();
        let p = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        b.rect(0.0, 0.0, 1.0, 1.0, C_WHITE, 1.0);
        b.rect(0.0, 0.0, 1.0, 1.0, C_WHITE, 1.0);
        b.quad(Some(&img), p, p, C_WHITE, 1.0);
        b.set_blend(QuadBlend::Add);
        b.quad(Some(&img), p, p, C_WHITE, 1.0);
        b.set_filter(true);
        b.quad(Some(&img), p, p, C_WHITE, 1.0);
        assert_eq!(b.runs.len(), 4);
        assert_eq!(b.runs[0].quads(), 2);
        assert_eq!(b.quad_count(), 5);
    }

    #[test]
    fn ext_rotates_counter_clockwise_on_screen() {
        let mut b = QuadBatch::new();
        let img = Handle::<Image>::default();
        // a 10x0 strip from the origin rotated 90°: its far end points up (y decreases)
        b.image_ext(
            &img,
            [0.0, 0.0, 10.0, 1.0],
            [0.0, 0.0, 10.0, 1.0],
            100.0,
            100.0,
            1.0,
            1.0,
            90.0,
            C_WHITE,
            1.0,
        );
        let tr = b.runs[0].verts[1].pos;
        assert!(
            (tr[0] - 100.0).abs() < 1e-4 && (tr[1] - 90.0).abs() < 1e-4,
            "{tr:?}"
        );
    }
}
