//! Task 15: obj_game's surface events (Draw_74 GUI begin, Draw_75 GUI end, Draw_77 post-draw)
//! and the BLACK METAL post effect (`post_layer_draw`, `bw_shader_set`, `sh_blackmetal`).
//!
//! In the original the world is drawn into `application_surface` and the GUI into
//! `gui_surface`; Draw_77 draws the application surface to the screen and Draw_75 the GUI
//! surface over it (after the mycelium glow composite), each through `post_layer_draw`. With
//! `global.bw_mode` off that is a 1:1 copy, so the Rust cameras draw straight to the window.
//! With it on, each layer is tone-mapped to grey by `sh_blackmetal` and gets a two-level glow:
//! the bright pass downscaled into `bw_glow_near` (1/`glow_downscale`), that again into
//! `bw_glow_far` (1/`glow_wide`), both added back stretched.
//!
//! Rust: while BLACK METAL is on, the world and GUI cameras render into one offscreen frame
//! ([`PostRedirect`], applied by `camera::apply_cameras`) and three post cameras run
//! `post_layer_draw` on it: the near and far glow passes and the final tone-mapped picture with
//! the glows on the window. Differences from the two-surface original: the GUI is tone-mapped
//! together with the world (identical for opaque GUI pixels), and the mycelium glow (composited
//! between the two surfaces, so not tone-mapped there) is inside the frame and turns grey too.
//!
//! `global.bw_mode` is toggled by the pause menu's "black_metal" item, which the shipped pause
//! menus do not list (`hud_pause_items_game/title`), so the effect is off in normal play; the dev
//! hook `MYCO_BW=1` turns it on (comparisons).

use crate::camera::{Cameras, PostRedirect, COMPOSITING};
use crate::render::batch::{
    run_mesh, BatchPlugin, BatchPool, BatchTarget, GmQuad, QuadBatch, QuadBlend, WhiteTexture,
    C_WHITE,
};
use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::RenderTarget;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    TextureDimension, TextureFormat, TextureUsages,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};
use myco_core::balance::vis;
use myco_core::gml;

/// `global.bw_mode` (BLACK METAL).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BwMode(pub bool);

/// Uniforms of one `sh_blackmetal` draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BwParams {
    pub gain: f64,
    pub pivot: f64,
    pub contrast: f64,
    pub gamma: f64,
    pub threshold: f64,
    pub bright: f64,
}

// port: bw_shader_set
/// `bw_shader_set(bright)`: `shader_set(sh_blackmetal)` with the `bw_*` look settings.
pub fn bw_shader_set(bright: f64) -> BwParams {
    BwParams {
        gain: vis("bw_gain"),
        pivot: vis("bw_pivot"),
        contrast: vis("bw_contrast"),
        gamma: vis("bw_gamma"),
        threshold: vis("bw_glow_threshold"),
        bright,
    }
}

/// `sh_blackmetal`'s fragment maths on one (encoded) colour, for tests.
pub fn bw_tone(rgb: [f64; 3], alpha: f64, p: &BwParams) -> [f64; 4] {
    let mut luma = rgb[0] * 0.299 + rgb[1] * 0.587 + rgb[2] * 0.114;
    luma *= p.gain;
    luma = ((luma - p.pivot) * p.contrast + p.pivot).clamp(0.0, 1.0);
    let o = luma.powf(p.gamma);
    let above = (o - p.threshold).max(0.0) / (1.0 - p.threshold).max(0.001);
    let v = gml::lerp(o, above, p.bright);
    [v, v, v, alpha]
}

/// `sh_blackmetal` on textured, vertex-coloured quads (see shaders/blackmetal.wesl).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BwQuad {
    #[texture(0)]
    #[sampler(1)]
    pub image: Handle<Image>,
    /// gain, pivot, contrast, gamma
    #[uniform(2)]
    pub tone: Vec4,
    /// threshold, bright, bilinear filter
    #[uniform(3)]
    pub extra: Vec4,
}

impl BwQuad {
    pub fn new(image: Handle<Image>, p: &BwParams, linear: bool) -> BwQuad {
        BwQuad {
            image,
            tone: Vec4::new(
                p.gain as f32,
                p.pivot as f32,
                p.contrast as f32,
                p.gamma as f32,
            ),
            extra: Vec4::new(
                p.threshold as f32,
                p.bright as f32,
                if linear { 1.0 } else { 0.0 },
                0.0,
            ),
        }
    }
}

const SHADER: &str = "embedded://myco_app/render/shaders/blackmetal.wesl";

impl Material2d for BwQuad {
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
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(frag) = descriptor.fragment.as_mut() {
            for t in frag.targets.iter_mut().flatten() {
                t.blend = Some(QuadBlend::Alpha.state());
            }
        }
        Ok(())
    }
}

/// Registers [`BwQuad`] (idempotent).
pub struct BwPlugin;

impl Plugin for BwPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<Material2dPlugin<BwQuad>>() {
            return;
        }
        embedded_asset!(app, "shaders/blackmetal.wesl");
        app.add_plugins(Material2dPlugin::<BwQuad>::default());
    }
}

fn target_image(w: u32, h: u32) -> Image {
    let mut img = Image::new_fill(
        Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.texture_descriptor.usage |=
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
    img
}

// port: surface_fit
/// `surface_fit(surf, w, h)`: the surface at `floor(w) × floor(h)` (at least 1×1), recreated
/// when its size differs.
pub fn surface_fit(
    images: &mut Assets<Image>,
    slot: &mut Option<(Handle<Image>, UVec2)>,
    w: f64,
    h: f64,
) -> Handle<Image> {
    let size = UVec2::new(w.floor().max(1.0) as u32, h.floor().max(1.0) as u32);
    if let Some((img, s)) = slot {
        if *s == size {
            return img.clone();
        }
    }
    let img = images.add(target_image(size.x, size.y));
    *slot = Some((img.clone(), size));
    img
}

/// The glow passes of one `post_layer_draw`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowPlan {
    /// near surface size (`w / glow_downscale`), drawn stretched to it
    pub gw: f64,
    pub gh: f64,
    /// far surface size (`gw / glow_wide`)
    pub fw: f64,
    pub fh: f64,
    /// alpha of the near and far glows added over the picture
    pub near_alpha: f64,
    pub far_alpha: f64,
}

/// What `post_layer_draw` does with a `w × h` layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostPlan {
    /// `draw_surface_stretched` as is
    Plain,
    /// tone-mapped by `sh_blackmetal` (bright 0), plus the glow when its strength is > 0
    BlackMetal { glow: Option<GlowPlan> },
}

// port: post_layer_draw
/// `post_layer_draw(surf, x, y, w, h)`: plain without BLACK METAL; with it the bright pass into
/// the near glow surface (filter on), that into the far one, the tone-mapped layer (filter
/// off), then both glows added (filter on) at `bw_glow_strength` and `strength · bw_glow_halo`.
pub fn post_layer_draw(bw: bool, w: f64, h: f64) -> PostPlan {
    if !bw {
        return PostPlan::Plain;
    }
    let glow = vis("bw_glow_strength");
    let halo = vis("bw_glow_halo");
    if glow <= 0.0 {
        return PostPlan::BlackMetal { glow: None };
    }
    let ds = vis("glow_downscale").max(2.0);
    let wide = vis("glow_wide").max(2.0);
    let gw = (w / ds).max(1.0);
    let gh = (h / ds).max(1.0);
    PostPlan::BlackMetal {
        glow: Some(GlowPlan {
            gw,
            gh,
            fw: (gw / wide).max(1.0),
            fh: (gh / wide).max(1.0),
            near_alpha: gml::clamp(glow, 0.0, 1.0),
            far_alpha: gml::clamp(glow * halo, 0.0, 1.0),
        }),
    }
}

// port: gml_Object_obj_game_Draw_74
/// obj_game Draw_74 (Draw GUI Begin): the GUI goes into `gui_surface` (sized to the GUI). Here:
/// the GUI camera's target — the offscreen frame while BLACK METAL is on (shared with the
/// world, see the module docs), else the window (the original's 1:1 copy of the surface).
pub fn gui_target(bw: bool, frame: Option<&Handle<Image>>) -> Option<Handle<Image>> {
    if bw {
        frame.cloned()
    } else {
        None
    }
}

// port: gml_Object_obj_game_Draw_75
// port: gml_Object_obj_game_Draw_77
/// obj_game Draw_77 (post-draw: `post_layer_draw(application_surface)` over a black clear) and
/// Draw_75 (Draw GUI End: the mycelium glow composite — Task 14, on the world camera — then
/// `post_layer_draw(gui_surface)`): one `post_layer_draw` over the frame both cameras rendered.
pub fn frame_post(bw: bool, picture_w: f64, picture_h: f64) -> PostPlan {
    post_layer_draw(bw, picture_w, picture_h)
}

/// Post pass cameras: 0 near glow, 1 far glow, 2 final picture.
#[derive(Component)]
struct PostCamera(usize, Option<AssetId<Image>>);

const POST_LAYERS: [usize; 3] = [40, 41, 42];
const POST_ORDER: isize = 10;

#[derive(Resource, Default)]
struct PostState {
    frame: Option<(Handle<Image>, UVec2)>,
    near: Option<(Handle<Image>, UVec2)>,
    far: Option<(Handle<Image>, UVec2)>,
    near_quad: Option<(Entity, Handle<Mesh>, Handle<BwQuad>)>,
    final_quad: Option<(Entity, Handle<Mesh>, Handle<BwQuad>)>,
    far_pool: BatchPool,
    glow_pool: BatchPool,
}

/// What post camera `i` clears to: the glow passes start transparent; the final picture starts
/// black, as Draw_77 does `draw_clear(c_black)` before `post_layer_draw(application_surface)`.
pub fn post_camera_clear(i: usize) -> ClearColorConfig {
    if i == 2 {
        ClearColorConfig::Custom(Color::BLACK)
    } else {
        ClearColorConfig::Custom(Color::NONE)
    }
}

fn spawn_post_cameras(mut commands: Commands) {
    for (i, &layer) in POST_LAYERS.iter().enumerate() {
        let clear = post_camera_clear(i);
        commands.spawn((
            Camera2d,
            Camera {
                order: POST_ORDER + i as isize,
                clear_color: clear,
                is_active: false,
                ..default()
            },
            RenderLayers::layer(layer),
            COMPOSITING,
            PostCamera(i, None),
        ));
    }
}

/// A quad over `[0, w] × [0, h]` showing all of `img` (`sw × sh` texels).
fn stretched(img: &Handle<Image>, sw: f64, sh: f64, w: f64, h: f64, alpha: f64) -> QuadBatch {
    let mut b = QuadBatch::new();
    b.image_ext(
        img,
        [0.0, 0.0, sw, sh],
        [0.0, 0.0, sw, sh],
        0.0,
        0.0,
        w / sw,
        h / sh,
        0.0,
        C_WHITE,
        alpha,
    );
    b
}

/// One BwQuad mesh entity, created on first use and rewritten each frame.
#[allow(clippy::too_many_arguments)]
fn bw_quad(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<BwQuad>,
    slot: &mut Option<(Entity, Handle<Mesh>, Handle<BwQuad>)>,
    b: &QuadBatch,
    material: BwQuad,
    layer: usize,
) {
    let verts = b.runs.first().map(|r| r.verts.clone()).unwrap_or_default();
    match slot {
        Some((_, mesh, mat)) => {
            if let Some(mut m) = meshes.get_mut(mesh) {
                *m = run_mesh(&verts);
            }
            if let Some(mut m) = mats.get_mut(mat) {
                *m = material;
            }
        }
        None => {
            let mesh = meshes.add(run_mesh(&verts));
            let mat = mats.add(material);
            let e = commands
                .spawn((
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(mat.clone()),
                    Transform::from_xyz(0.0, 0.0, 1.0),
                    NoFrustumCulling,
                    RenderLayers::layer(layer),
                ))
                .id();
            *slot = Some((e, mesh, mat));
        }
    }
}

fn camera_fit(proj: &mut Projection, tf: &mut Transform, w: f32, h: f32) {
    let current = match &*proj {
        Projection::Orthographic(o) => Some((o.area.width(), o.area.height())),
        _ => None,
    };
    if current != Some((w, h)) {
        use bevy::camera::CameraProjection;
        let mut o = OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: w,
                height: h,
            },
            ..OrthographicProjection::default_2d()
        };
        o.update(w, h);
        *proj = Projection::Orthographic(o);
    }
    let want = Transform::from_xyz(w * 0.5, -h * 0.5, 0.0);
    if *tf != want {
        *tf = want;
    }
}

/// The post pipeline each frame (see the module docs).
#[allow(clippy::too_many_arguments)]
fn post_system(
    mut commands: Commands,
    bw: Res<BwMode>,
    cams: Res<Cameras>,
    mut redirect: ResMut<PostRedirect>,
    mut st: ResMut<PostState>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut bw_mats: ResMut<Assets<BwQuad>>,
    mut quad_mats: ResMut<Assets<GmQuad>>,
    white: Res<WhiteTexture>,
    mut cam_q: Query<(
        Entity,
        &mut Camera,
        &mut Projection,
        &mut Transform,
        &mut PostCamera,
    )>,
) {
    let (w, h) = (cams.picture.w.max(1.0), cams.picture.h.max(1.0));
    let plan = frame_post(bw.0, w, h);
    let PostPlan::BlackMetal { glow } = plan else {
        if redirect.0.is_some() {
            redirect.0 = None;
        }
        for (_, mut cam, ..) in &mut cam_q {
            if cam.is_active {
                cam.is_active = false;
            }
        }
        return;
    };
    let st = &mut *st;
    let frame = surface_fit(&mut images, &mut st.frame, w, h);
    let gui = gui_target(bw.0, Some(&frame));
    if redirect.0.as_ref().map(|i| i.id()) != gui.as_ref().map(|i| i.id()) {
        redirect.0 = gui;
    }
    let fsize = st.frame.as_ref().map(|f| f.1).unwrap_or(UVec2::ONE);
    let (fw_px, fh_px) = (fsize.x as f64, fsize.y as f64);
    let mut targets: [Option<(Handle<Image>, UVec2)>; 3] = [None, None, None];
    let mut glow_batch = QuadBatch::new();
    if let Some(g) = glow {
        let near = surface_fit(&mut images, &mut st.near, g.gw, g.gh);
        let far = surface_fit(&mut images, &mut st.far, g.fw, g.fh);
        let ns = st.near.as_ref().unwrap().1;
        let fs = st.far.as_ref().unwrap().1;
        targets[0] = Some((near.clone(), ns));
        targets[1] = Some((far.clone(), fs));
        // bright pass: the frame stretched to gw × gh with the filter on
        let nb = stretched(&frame, fw_px, fh_px, g.gw, g.gh, 1.0);
        bw_quad(
            &mut commands,
            &mut meshes,
            &mut bw_mats,
            &mut st.near_quad,
            &nb,
            BwQuad::new(frame.clone(), &bw_shader_set(1.0), true),
            POST_LAYERS[0],
        );
        // far: the near surface stretched to fw × fh (filter on, normal blend)
        let mut fb = stretched(&near, ns.x as f64, ns.y as f64, g.fw, g.fh, 1.0);
        fb.runs.iter_mut().for_each(|r| r.linear = true);
        // the glows added over the picture (filter on)
        glow_batch.set_filter(true);
        glow_batch.set_blend(QuadBlend::Add);
        glow_batch.image_ext(
            &near,
            [0.0, 0.0, ns.x as f64, ns.y as f64],
            [0.0, 0.0, ns.x as f64, ns.y as f64],
            0.0,
            0.0,
            fw_px / g.gw,
            fh_px / g.gh,
            0.0,
            C_WHITE,
            g.near_alpha,
        );
        glow_batch.image_ext(
            &far,
            [0.0, 0.0, fs.x as f64, fs.y as f64],
            [0.0, 0.0, fs.x as f64, fs.y as f64],
            0.0,
            0.0,
            fw_px / g.fw,
            fh_px / g.fh,
            0.0,
            C_WHITE,
            g.far_alpha,
        );
        let mut t = BatchTarget {
            commands: &mut commands,
            meshes: &mut meshes,
            materials: &mut quad_mats,
            white: &white.0,
        };
        st.far_pool
            .sync(&mut t, &fb, 1.0, 0.01, &RenderLayers::layer(POST_LAYERS[1]));
    }
    // the tone-mapped picture (filter off), then the glows
    let pb = stretched(&frame, fw_px, fh_px, fw_px, fh_px, 1.0);
    bw_quad(
        &mut commands,
        &mut meshes,
        &mut bw_mats,
        &mut st.final_quad,
        &pb,
        BwQuad::new(frame.clone(), &bw_shader_set(0.0), false),
        POST_LAYERS[2],
    );
    {
        let mut t = BatchTarget {
            commands: &mut commands,
            meshes: &mut meshes,
            materials: &mut quad_mats,
            white: &white.0,
        };
        st.glow_pool.sync(
            &mut t,
            &glow_batch,
            2.0,
            0.01,
            &RenderLayers::layer(POST_LAYERS[2]),
        );
    }
    for (e, mut cam, mut proj, mut tf, mut pc) in &mut cam_q {
        let on = pc.0 == 2 || targets[pc.0].is_some();
        if cam.is_active != on {
            cam.is_active = on;
        }
        if !on {
            continue;
        }
        if pc.0 == 2 {
            let vp = cams.viewport();
            let key = |v: &Option<bevy::camera::Viewport>| {
                v.as_ref().map(|v| (v.physical_position, v.physical_size))
            };
            if key(&cam.viewport) != key(&vp) {
                cam.viewport = vp;
            }
            camera_fit(&mut proj, &mut tf, fw_px as f32, fh_px as f32);
            continue;
        }
        let (img, size) = targets[pc.0].clone().unwrap();
        camera_fit(&mut proj, &mut tf, size.x as f32, size.y as f32);
        if pc.1 != Some(img.id()) {
            commands
                .entity(e)
                .insert(RenderTarget::Image(img.clone().into()));
            pc.1 = Some(img.id());
        }
    }
}

pub struct PostPlugin;

impl Plugin for PostPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BatchPlugin>() {
            app.add_plugins(BatchPlugin);
        }
        app.add_plugins(BwPlugin);
        // off in the shipped game; `MYCO_BW=1` (autopilot::DevHooks) turns it on
        app.insert_resource(BwMode(false))
            .init_resource::<PostRedirect>()
            .init_resource::<PostState>()
            .add_systems(Startup, spawn_post_cameras)
            .add_systems(Update, post_system);
    }
}
