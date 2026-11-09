//! The purple perk's glow and its pulse rings (Task 14): `myc_draw_glow`, `myc_pulse_ring`,
//! `myc_glow_composite` and `sh_glow_blur` (shaders/glow_blur.wesl).
//!
//! The original draws the hyphae into a buffer at 1/`myc_glow_downscale` resolution (the seed),
//! thickens it (eight offset copies plus the centre, added), blurs it `myc_glow_passes` times
//! across and down with `sh_glow_blur` (alpha only; the colour becomes the draw colour, white),
//! then adds it over the world (obj_game Draw_75, below the GUI surface with the hyphae) tinted
//! `myc_glow_colour`, `myc_glow_strength` times. Pulse rings (`myc_pulse_ring`) are drawn into
//! another buffer, multiplied by the blurred seed and added tinted `myc_pulse_colour`.
//!
//! Here each buffer pass is an offscreen camera rendering one full-buffer quad with
//! [`GlowMaterial`] into an image: the seed camera renders the hyphae entities (which carry
//! [`GLOW_SEED_LAYER`]; Task 13's cave and relic wraps, `caves_wrap_draw_visible` and
//! `relics_wrap_draw_visible`, join the seed by adding that layer to their entities), then the
//! thicken pass, then the blur passes ping-ponging between two images. One world-space quad
//! composites the result; its shader adds the glow and computes the rings analytically (the
//! original's 96-segment strips interpolate the same radial ramps).
//!
//! The pulse buffer follows the LÖVE port's blend mapping: `bm_dest_alpha, bm_zero` is
//! "multiply" there (`game/gm/draw.lua`), so a pulse texel is the blurred seed times the ring
//! (alpha = seed · ring, colour white where the ring strip covers).

use crate::camera::Cameras;
use crate::render::layers::Z_MYC_GLOW;
use crate::render::sprite::gm_colour_f;
use crate::sim_bridge::SimRes;
use bevy::asset::embedded_asset;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::camera::{CameraProjection, RenderTarget, ScalingMode};
use bevy::image::ImageSampler;
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState, RenderPipelineDescriptor,
    ShaderType, SpecializedMeshPipelineError, TextureFormat,
};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{
    AlphaMode2d, Material2d, Material2dKey, Material2dPipeline, Material2dPlugin,
};
use myco_core::balance::vis;
use myco_core::mycelium as myc;
use myco_core::sim::ViewRect;

/// Render layer of everything drawn into the glow seed (the hyphae; T13's cave / relic wraps).
pub const GLOW_SEED_LAYER: usize = 20;
/// First render layer of the buffer passes (one per pass).
const PASS_LAYER_BASE: usize = 21;
/// Camera order of the seed pass; the buffer passes follow.
const SEED_ORDER: isize = -40;
/// Pulse rings alive at once that the composite shader takes (one every 20 s, each lives about
/// `(far_max + 280) / 900` s).
pub const MAX_PULSES: usize = 8;

const SHADER: &str = "embedded://myco_app/render/shaders/glow_blur.wesl";

/// Where the glow buffers lie (`myc_glow_at`): `w × h` texels of `step` world px from (ox, oy).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowLayout {
    pub w: u32,
    pub h: u32,
    pub ox: f64,
    pub oy: f64,
    pub step: f64,
}

/// `myc_draw_glow`'s buffer placement for the visible world rectangle `v`: the view at
/// 1/`ds` resolution plus two texels of margin, snapped to the texel grid.
pub fn glow_layout(v: &ViewRect, ds: f64) -> GlowLayout {
    let ds = ds.max(1.0);
    let w = (v.w / ds).ceil().max(1.0) + 4.0;
    let h = (v.h / ds).ceil().max(1.0) + 4.0;
    GlowLayout {
        w: w as u32,
        h: h as u32,
        ox: (v.x / ds).floor() * ds - ds * 2.0,
        oy: (v.y / ds).floor() * ds - ds * 2.0,
        step: ds,
    }
}

/// [`GlowMaterial`]'s uniform. Lengths are in buffer texels.
#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct GlowParams {
    /// x: mode (0 copy, 1 thicken, 2 blur, 3 composite); y: thicken offset; zw: blur step
    pub mode: Vec4,
    /// rgb: `myc_glow_colour` (sRGB-encoded); a: total glow strength (0 when the glow is off)
    pub glow: Vec4,
    /// rgb: `myc_pulse_colour` (sRGB-encoded); a: total pulse strength
    pub pulse: Vec4,
    /// xy: pulse centre (the origin mushroom); z: ring band half-width; w: ring count
    pub origin: Vec4,
    /// per ring: x radius, y fade (the ring's alpha)
    pub rings: [Vec4; MAX_PULSES],
}

/// Pipeline key: additive composite or plain replace (buffer passes).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlowKey {
    pub additive: bool,
}

impl From<&GlowMaterial> for GlowKey {
    fn from(m: &GlowMaterial) -> Self {
        GlowKey {
            additive: m.additive,
        }
    }
}

/// One glow buffer pass or the composite (shaders/glow_blur.wesl).
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
#[bind_group_data(GlowKey)]
pub struct GlowMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub tex: Handle<Image>,
    #[uniform(2)]
    pub params: GlowParams,
    pub additive: bool,
}

impl Material2d for GlowMaterial {
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
        let blend = if key.bind_group_data.additive {
            // the composite: dst += premultiplied glow (the shader folds in alpha and strength)
            BlendState {
                color: BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
                alpha: BlendComponent {
                    src_factor: BlendFactor::Zero,
                    dst_factor: BlendFactor::One,
                    operation: BlendOperation::Add,
                },
            }
        } else {
            // gpu_set_blendmode_ext(bm_one, bm_zero)
            BlendState::REPLACE
        };
        if let Some(frag) = descriptor.fragment.as_mut() {
            for t in frag.targets.iter_mut().flatten() {
                t.blend = Some(blend);
            }
        }
        Ok(())
    }
}

/// One buffer pass: its camera, its quad and the images it reads and writes.
struct Pass {
    camera: Entity,
    quad: Entity,
    material: Handle<GlowMaterial>,
    /// index into [`MycGlow::images`]
    src: usize,
    dst: usize,
}

/// The glow buffers and their passes (`myc_glow_seed`, `myc_glow_blur`, `myc_glow_at`).
#[derive(Resource)]
pub struct MycGlow {
    /// 0: seed, 1 and 2: ping-pong
    images: [Handle<Image>; 3],
    seed_camera: Entity,
    passes: Vec<Pass>,
    composite: Entity,
    composite_material: Handle<GlowMaterial>,
    /// current buffer placement (`myc_glow_at`); `None` until the glow is drawn
    pub at: Option<GlowLayout>,
    final_image: usize,
}

fn target_image(w: u32, h: u32) -> Image {
    let mut img = Image::new_target_texture(w, h, TextureFormat::Rgba8Unorm, None);
    img.sampler = ImageSampler::linear();
    img
}

fn ortho(w: f32, h: f32) -> Projection {
    let mut p = OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: w,
            height: h,
        },
        ..OrthographicProjection::default_2d()
    };
    // computed area now: a fresh projection's unit area culls everything (R21a)
    p.update(w, h);
    Projection::Orthographic(p)
}

/// The vis constants of the glow (read once; they never change in a run).
#[derive(Clone, Copy, Debug)]
struct GlowVis {
    ds: f64,
    thicken: f64,
    radius: f64,
    passes: usize,
}

fn glow_vis() -> GlowVis {
    GlowVis {
        ds: vis("myc_glow_downscale").max(1.0),
        thicken: vis("myc_glow_thicken"),
        radius: vis("myc_glow_radius").max(0.5) * 0.25,
        passes: vis("myc_glow_passes").max(1.0) as usize,
    }
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
) {
    let gv = glow_vis();
    let imgs = [
        images.add(target_image(4, 4)),
        images.add(target_image(4, 4)),
        images.add(target_image(4, 4)),
    ];
    let seed_camera = commands
        .spawn((
            Name::new("glow seed camera"),
            Camera2d,
            Camera {
                order: SEED_ORDER,
                is_active: false,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            RenderTarget::Image(imgs[0].clone().into()),
            ortho(4.0, 4.0),
            Msaa::Off,
            RenderLayers::layer(GLOW_SEED_LAYER),
        ))
        .id();
    // seed → 1 (thicken, or a copy without thickening), then 1 → 2 → 1 per blur pass
    let mut chain = vec![(0usize, 1usize, if gv.thicken > 0.0 { 1.0 } else { 0.0 })];
    for _ in 0..gv.passes {
        chain.push((1, 2, 2.0));
        chain.push((2, 1, 2.0));
    }
    let quad = meshes.add(Rectangle::new(1.0, 1.0));
    let mut passes = Vec::new();
    for (i, (src, dst, mode)) in chain.into_iter().enumerate() {
        let layer = RenderLayers::layer(PASS_LAYER_BASE + i);
        let material = materials.add(GlowMaterial {
            tex: imgs[src].clone(),
            params: GlowParams {
                mode: Vec4::new(mode, gv.thicken as f32, 0.0, 0.0),
                ..default()
            },
            additive: false,
        });
        let camera = commands
            .spawn((
                Name::new("glow pass camera"),
                Camera2d,
                Camera {
                    order: SEED_ORDER + 1 + i as isize,
                    is_active: false,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::Image(imgs[dst].clone().into()),
                ortho(4.0, 4.0),
                Msaa::Off,
                layer.clone(),
            ))
            .id();
        let quad = commands
            .spawn((
                Name::new("glow pass quad"),
                Mesh2d(quad.clone()),
                MeshMaterial2d(material.clone()),
                Transform::from_scale(Vec3::new(4.0, 4.0, 1.0)),
                layer,
                NoFrustumCulling,
            ))
            .id();
        passes.push(Pass {
            camera,
            quad,
            material,
            src,
            dst,
        });
    }
    let final_image = passes.last().map_or(0, |p| p.dst);
    let composite_material = materials.add(GlowMaterial {
        tex: imgs[final_image].clone(),
        params: GlowParams {
            mode: Vec4::new(3.0, 0.0, 0.0, 0.0),
            ..default()
        },
        additive: true,
    });
    let composite = commands
        .spawn((
            Name::new("glow composite"),
            Mesh2d(quad),
            MeshMaterial2d(composite_material.clone()),
            Transform::from_xyz(0.0, 0.0, Z_MYC_GLOW),
            Visibility::Hidden,
            NoFrustumCulling,
        ))
        .id();
    commands.insert_resource(MycGlow {
        images: imgs,
        seed_camera,
        passes,
        composite,
        composite_material,
        at: None,
        final_image,
    });
}

// port: anon__67930__gml_Object_obj_mycelium_controller_Create_0
/// `myc_draw_glow(cx, cy, view)`: (re)size the buffers to the view and run the seed, thicken and
/// blur passes this frame. Draw_64 calls it while the glow is on or pulses run; otherwise the
/// passes are off and `myc_glow_at` keeps its last value, as in the original.
#[allow(clippy::too_many_arguments)]
fn draw_glow_system(
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    glow: Option<ResMut<MycGlow>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
    mut cameras: Query<(
        &mut Camera,
        &mut RenderTarget,
        &mut Projection,
        &mut Transform,
    )>,
    mut quads: Query<&mut Transform, Without<Camera>>,
) {
    let Some(mut glow) = glow else { return };
    let m = &sim.0.mycelium;
    let on = m.glow_on || !m.pulses.is_empty();
    let gv = glow_vis();
    let mut ids = vec![glow.seed_camera];
    ids.extend(glow.passes.iter().map(|p| p.camera));
    for &e in &ids {
        if let Ok((mut c, ..)) = cameras.get_mut(e) {
            if c.is_active != on {
                c.is_active = on;
            }
        }
    }
    if !on {
        return;
    }
    let lay = glow_layout(&cams.render_view(), gv.ds);
    let resized = glow.at.is_none_or(|a| (a.w, a.h) != (lay.w, lay.h));
    if resized {
        // surface_fit: new buffers of the new size
        for i in 0..3 {
            glow.images[i] = images.add(target_image(lay.w, lay.h));
        }
    }
    let (w, h) = (lay.w as f32, lay.h as f32);
    // the seed camera looks at the buffer's world rectangle
    if let Ok((_, mut target, mut proj, mut tf)) = cameras.get_mut(glow.seed_camera) {
        if resized {
            *target = RenderTarget::Image(glow.images[0].clone().into());
            *proj = ortho(w * lay.step as f32, h * lay.step as f32);
        }
        let c = Vec3::new(
            (lay.ox + lay.w as f64 * lay.step / 2.0) as f32,
            -(lay.oy + lay.h as f64 * lay.step / 2.0) as f32,
            0.0,
        );
        tf.set_if_neq(Transform::from_translation(c));
    }
    let r = gv.radius as f32;
    for (i, p) in glow.passes.iter().enumerate() {
        if resized {
            if let Ok((_, mut target, mut proj, _)) = cameras.get_mut(p.camera) {
                *target = RenderTarget::Image(glow.images[p.dst].clone().into());
                *proj = ortho(w, h);
            }
            if let Ok(mut tf) = quads.get_mut(p.quad) {
                *tf = Transform::from_scale(Vec3::new(w, h, 1.0));
            }
        }
        if let Some(mut mat) = materials.get_mut(&p.material) {
            mat.tex = glow.images[p.src].clone();
            // blur passes alternate across (odd index) and down
            let step = if i == 0 {
                Vec2::ZERO
            } else if i % 2 == 1 {
                Vec2::new(r, 0.0)
            } else {
                Vec2::new(0.0, r)
            };
            let mode = mat.params.mode.x;
            mat.params.mode = Vec4::new(mode, gv.thicken as f32, step.x, step.y);
        }
    }
    glow.at = Some(lay);
}

// port: anon__72092__gml_Object_obj_mycelium_controller_Create_0
/// `myc_pulse_ring(x, y, r, w, a)`: a ring rising from 0 at `r - w` to `a` at `r` and falling to
/// 0 at `r + w`; returns the ring's alpha at distance `d` from its centre (`None` outside the
/// strips). The composite shader evaluates the same ramps per pixel.
pub fn pulse_ring_alpha(d: f64, r: f64, w: f64, a: f64) -> Option<f64> {
    let rin = (r - w).max(0.0);
    let rout = r + w;
    if d < rin || d > rout {
        return None;
    }
    Some(if d <= r {
        if r > rin {
            a * (d - rin) / (r - rin)
        } else {
            a
        }
    } else {
        a * (rout - d) / (rout - r)
    })
}

// port: anon__73025__gml_Object_obj_mycelium_controller_Create_0
/// `myc_glow_composite` (obj_game Draw_75): add the blurred seed over the world, and the pulse
/// rings masked by it; pulses that ran past their reach are dropped here, as in the original.
fn glow_composite_system(
    mut sim: ResMut<SimRes>,
    glow: Option<Res<MycGlow>>,
    mut materials: ResMut<Assets<GlowMaterial>>,
    mut q: Query<(&mut Transform, &mut Visibility)>,
) {
    let Some(glow) = glow else { return };
    let Some(g) = glow.at else { return };
    let on = sim.0.mycelium.glow_on || !sim.0.mycelium.pulses.is_empty();
    // pulses still running
    let sim_ref = &sim.0;
    let keep: Vec<myc::Pulse> = sim_ref
        .mycelium
        .pulses
        .iter()
        .copied()
        .filter(|p| myc::pulse_radius(sim_ref, p) >= 0.0)
        .collect();
    let rings: Vec<(f64, f64)> = keep
        .iter()
        .map(|p| {
            let rad = myc::pulse_radius(sim_ref, p);
            let band = vis("myc_pulse_band_px");
            let fade = ((p.far - rad) / (band * 2.0).max(1.0)).clamp(0.0, 1.0);
            (rad / g.step, fade)
        })
        .collect();
    let (ox_px, oy_px) = (
        (sim_ref.mycelium.x - g.ox) / g.step,
        (sim_ref.mycelium.y - g.oy) / g.step,
    );
    let glow_on = sim_ref.mycelium.glow_on;
    sim.0.mycelium.pulses = keep;
    let Ok((mut tf, mut visibility)) = q.get_mut(glow.composite) else {
        return;
    };
    let show = on && (glow_on || !rings.is_empty());
    visibility.set_if_neq(if show {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
    if !show {
        return;
    }
    let (ww, wh) = (g.w as f64 * g.step, g.h as f64 * g.step);
    tf.set_if_neq(Transform {
        translation: Vec3::new(
            (g.ox + ww / 2.0) as f32,
            -(g.oy + wh / 2.0) as f32,
            Z_MYC_GLOW,
        ),
        scale: Vec3::new(ww as f32, wh as f32, 1.0),
        ..default()
    });
    let Some(mut mat) = materials.get_mut(&glow.composite_material) else {
        return;
    };
    mat.tex = glow.images[glow.final_image].clone();
    let enc = |col: f64| {
        let c = gm_colour_f(col, 1.0).to_srgba();
        Vec3::new(c.red, c.green, c.blue)
    };
    // `while s > 0: draw with alpha min(1, s); s -= 1` adds up to s in total
    let gs = if glow_on {
        vis("myc_glow_strength").max(0.0)
    } else {
        0.0
    };
    mat.params.glow = enc(vis("myc_glow_colour")).extend(gs as f32);
    mat.params.pulse =
        enc(vis("myc_pulse_colour")).extend(vis("myc_pulse_strength").max(0.0) as f32);
    let n = rings.len().min(MAX_PULSES);
    mat.params.origin = Vec4::new(
        ox_px as f32,
        oy_px as f32,
        (vis("myc_pulse_band_px") / g.step) as f32,
        n as f32,
    );
    for (i, slot) in mat.params.rings.iter_mut().enumerate() {
        *slot = rings.get(i).map_or(Vec4::ZERO, |&(r, a)| {
            Vec4::new(r as f32, a as f32, 0.0, 0.0)
        });
    }
    mat.params.mode = Vec4::new(3.0, g.w as f32, g.h as f32, 0.0);
}

/// `MYCO_PERK=<frame>:<perk>` (development aid for render comparisons, like `MYCO_FORCE_RAIN`):
/// grant a cave perk (e.g. `purple`, which unlocks the glow on F) before that autopilot frame;
/// the LÖVE side does the same with `MYCO_PROBE="<frame>|global.cave_perks.<perk>=1"`.
pub fn parse_perk_hook(s: &str) -> Option<(u64, String)> {
    let (f, id) = s.split_once(':')?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some((f.trim().parse().ok()?, id.to_string()))
}

fn perk_hook(
    mut sim: ResMut<SimRes>,
    autopilot: Option<Res<crate::autopilot::Autopilot>>,
    mut hook: Local<Option<Option<(u64, String)>>>,
) {
    let hook = hook.get_or_insert_with(|| {
        std::env::var("MYCO_PERK")
            .ok()
            .and_then(|s| parse_perk_hook(&s))
    });
    let (Some((frame, id)), Some(ap)) = (hook.as_ref(), autopilot) else {
        return;
    };
    if ap.frame == *frame {
        sim.0
            .state
            .state
            .cave_perks
            .insert(id.clone(), serde_json::Value::from(1));
    }
}

pub struct MyceliumGlowPlugin;

impl Plugin for MyceliumGlowPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "shaders/glow_blur.wesl");
        if !app.is_plugin_added::<Material2dPlugin<GlowMaterial>>() {
            app.add_plugins(Material2dPlugin::<GlowMaterial>::default());
        }
        app.add_systems(
            FixedUpdate,
            perk_hook
                .in_set(crate::sim_bridge::SimSet::Prepare)
                .after(crate::autopilot::run_script),
        );
        app.add_systems(Startup, setup).add_systems(
            PostUpdate,
            (draw_glow_system, glow_composite_system)
                .chain()
                .before(bevy::camera::CameraUpdateSystems),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perk_hook_parses_frame_and_perk() {
        assert_eq!(
            parse_perk_hook("1300:purple"),
            Some((1300, "purple".into()))
        );
        assert_eq!(parse_perk_hook("x:purple"), None);
        assert_eq!(parse_perk_hook("12:"), None);
        assert_eq!(parse_perk_hook("purple"), None);
    }

    #[test]
    fn pulse_ring_ramps_up_to_the_radius_and_down_past_it() {
        assert_eq!(pulse_ring_alpha(10.0, 100.0, 70.0, 0.8), None);
        assert_eq!(pulse_ring_alpha(30.0, 100.0, 70.0, 0.8), Some(0.0));
        assert!((pulse_ring_alpha(65.0, 100.0, 70.0, 0.8).unwrap() - 0.4).abs() < 1e-12);
        assert_eq!(pulse_ring_alpha(100.0, 100.0, 70.0, 0.8), Some(0.8));
        assert!((pulse_ring_alpha(135.0, 100.0, 70.0, 0.8).unwrap() - 0.4).abs() < 1e-12);
        assert_eq!(pulse_ring_alpha(171.0, 100.0, 70.0, 0.8), None);
        // a young ring: the inner strip starts at the centre
        assert!((pulse_ring_alpha(0.0, 20.0, 70.0, 1.0).unwrap()).abs() < 1e-12);
        assert!((pulse_ring_alpha(10.0, 20.0, 70.0, 1.0).unwrap() - 0.5).abs() < 1e-12);
    }
}
