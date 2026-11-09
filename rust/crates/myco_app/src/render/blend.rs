//! Vertex-coloured 2D mesh materials with GameMaker blend modes, for primitives
//! (`draw_primitive_*`, `draw_rectangle`, `draw_ellipse`) drawn into the world.
//!
//! [`GmBlend`] uses Bevy's default mesh2d shaders (vertex colour out), sorted with sprites by z in
//! the transparent 2D phase; only the blend state differs per [`GmBlendMode`].
//!
//! Vertex colours go in through [`colour_mesh`] (GameMaker BGR + alpha, sRGB).

use crate::render::sprite::gm_colour_f;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    SpecializedMeshPipelineError,
};
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};

fn set_blend(descriptor: &mut RenderPipelineDescriptor, blend: BlendState) {
    if let Some(frag) = descriptor.fragment.as_mut() {
        for t in frag.targets.iter_mut().flatten() {
            t.blend = Some(blend);
        }
    }
}

const KEEP_ALPHA: BlendComponent = BlendComponent {
    src_factor: BlendFactor::Zero,
    dst_factor: BlendFactor::One,
    operation: BlendOperation::Add,
};

/// GameMaker blend modes the world primitives use.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GmBlendMode {
    /// `gpu_set_blendmode(bm_add)` (LÖVE "add", alphamultiply): `dst += src·a`
    #[default]
    Add,
    /// `gpu_set_blendmode_ext(bm_dest_colour, bm_zero)`: `dst *= src`
    Multiply,
}

impl GmBlendMode {
    pub fn state(self) -> BlendState {
        let color = match self {
            GmBlendMode::Add => BlendComponent {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            GmBlendMode::Multiply => BlendComponent {
                src_factor: BlendFactor::Dst,
                dst_factor: BlendFactor::Zero,
                operation: BlendOperation::Add,
            },
        };
        BlendState {
            color,
            alpha: KEEP_ALPHA,
        }
    }
}

/// Pipeline key of [`GmBlend`]: its blend mode.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GmBlendKey {
    pub mode: GmBlendMode,
}

impl From<&GmBlend> for GmBlendKey {
    fn from(m: &GmBlend) -> Self {
        GmBlendKey { mode: m.mode }
    }
}

/// Vertex-coloured mesh material with a GameMaker blend mode.
///
/// ONE material type on purpose: with two custom `Material2dPlugin`s registered, Bevy 0.20 stops
/// drawing sprites whose `Sprite` component is written every frame (reproduced in Task 16).
/// Render tasks needing another blend mode should add a [`GmBlendMode`] variant, not a material.
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug, Default)]
#[bind_group_data(GmBlendKey)]
pub struct GmBlend {
    /// unused by the default mesh2d shaders (a material needs a binding)
    #[uniform(0)]
    pub tint: LinearRgba,
    pub mode: GmBlendMode,
}

impl GmBlend {
    pub fn new(mode: GmBlendMode) -> Self {
        GmBlend {
            tint: LinearRgba::WHITE,
            mode,
        }
    }
}

impl Material2d for GmBlend {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        _pipeline: &Material2dPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        set_blend(descriptor, key.bind_group_data.mode.state());
        Ok(())
    }
}

/// A coloured vertex in world coordinates (y down): GameMaker colour (BGR) and alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vert {
    pub x: f64,
    pub y: f64,
    pub col: f64,
    pub alpha: f64,
}

/// Fill `mesh` with a triangle list of `verts` (Bevy space, linear vertex colours).
pub fn colour_mesh(mesh: &mut Mesh, verts: &[Vert]) {
    let pos: Vec<[f32; 3]> = verts
        .iter()
        .map(|v| [v.x as f32, -v.y as f32, 0.0])
        .collect();
    let col: Vec<[f32; 4]> = verts
        .iter()
        .map(|v| gm_colour_f(v.col, v.alpha).to_linear().to_f32_array())
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
}

/// A triangle-list mesh for [`colour_mesh`], starting as one degenerate (invisible) triangle:
/// the mesh allocator warns about meshes without vertices.
pub fn empty_colour_mesh() -> Mesh {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let p = Vert {
        x: 0.0,
        y: 0.0,
        col: 0.0,
        alpha: 0.0,
    };
    colour_mesh(&mut m, &[p, p, p]);
    m
}

pub struct BlendPlugin;

impl Plugin for BlendPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<Material2dPlugin<GmBlend>>() {
            app.add_plugins(Material2dPlugin::<GmBlend>::default());
        }
    }
}
