//! Headless GPU checks of the render pipeline (R21): a camera draws into an image target that is
//! read back, so the tests see real pixels.
//!
//! - R21a: custom `Material2d` types do not stop sprites from drawing (the Task 16 symptom was the
//!   world camera's stale projection area, fixed and unit-tested in `camera.rs`).
//! - R21b: blending happens in gamma-encoded sRGB space, like GameMaker / LÖVE.

use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};
use bevy::window::ExitCondition;
use bevy::winit::WinitPlugin;
use myco_app::camera::COMPOSITING;
use myco_app::render::blend::{colour_mesh, BlendPlugin, GmBlend, GmBlendMode, Vert};
use std::sync::{Arc, Mutex};

const W: u32 = 64;
const H: u32 = 64;

#[derive(Resource, Clone, Default)]
struct Pixels(Arc<Mutex<Vec<u8>>>);

fn headless_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .disable::<WinitPlugin>()
            .disable::<bevy::audio::AudioPlugin>()
            .disable::<bevy::log::LogPlugin>()
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                close_when_requested: false,
                ..default()
            }),
    );
    app.insert_resource(Pixels::default());
    app.add_systems(Startup, setup_target);
    app
}

/// A 64×64 sRGB image target, a camera with the app's compositing space, and a readback.
fn setup_target(mut commands: Commands, mut images: ResMut<Assets<Image>>, px: Res<Pixels>) {
    let mut img = Image::new_fill(
        Extent3d {
            width: W,
            height: H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.texture_descriptor.usage |=
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING;
    let h = images.add(img);
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        COMPOSITING,
        RenderTarget::Image(h.clone().into()),
    ));
    let px = px.0.clone();
    commands
        .spawn(Readback::texture(h))
        .observe(move |ev: On<ReadbackComplete>| {
            *px.lock().unwrap() = ev.data.clone();
        });
}

fn run(app: &mut App, frames: usize) -> Vec<u8> {
    app.finish();
    app.cleanup();
    for _ in 0..frames {
        app.update();
    }
    let data = app.world().resource::<Pixels>().0.lock().unwrap().clone();
    assert!(!data.is_empty(), "no readback");
    data
}

/// RGBA of pixel (x, y) (y down) of the read-back target.
fn pixel(data: &[u8], x: u32, y: u32) -> [u8; 4] {
    let o = ((y * W + x) * 4) as usize;
    [data[o], data[o + 1], data[o + 2], data[o + 3]]
}

fn square(col: Color, z: f32) -> (Sprite, Transform) {
    (
        Sprite {
            color: col,
            custom_size: Some(Vec2::new(W as f32, H as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, z),
    )
}

// --- R21a -----------------------------------------------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug, Default)]
struct MatA {
    #[uniform(0)]
    tint: LinearRgba,
}
impl Material2d for MatA {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Asset, TypePath, AsBindGroup, Clone, Debug, Default)]
struct MatB {
    #[uniform(0)]
    tint: LinearRgba,
}
impl Material2d for MatB {
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(Component)]
struct Rewritten;

#[derive(Component)]
struct Remeshed;

/// a small green triangle in the top-left corner (vertex colours, default mesh2d shaders)
fn corner_triangle() -> Mesh {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let p = [
        [-30.0f32, 20.0, 0.0],
        [-20.0, 20.0, 0.0],
        [-20.0, 30.0, 0.0],
    ];
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, p.to_vec());
    m.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0f32, 1.0, 0.0, 1.0]; 3]);
    m
}

fn spawn_materials_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut ma: ResMut<Assets<MatA>>,
    mut mb: ResMut<Assets<MatB>>,
) {
    let mut red = square(Color::srgb(1.0, 0.0, 0.0), 1.0);
    red.0.custom_size = Some(Vec2::new(16.0, 16.0));
    commands.spawn((red, Rewritten));
    commands.spawn((
        Mesh2d(meshes.add(corner_triangle())),
        MeshMaterial2d(ma.add(MatA::default())),
        Transform::from_xyz(0.0, 0.0, 2.0),
        Remeshed,
    ));
    commands.spawn((
        Mesh2d(meshes.add(corner_triangle())),
        MeshMaterial2d(mb.add(MatB::default())),
        Transform::from_xyz(4.0, 0.0, 3.0),
        Remeshed,
    ));
}

/// What the forest once did: write the Sprite component every frame, even when unchanged.
fn rewrite_sprite(mut q: Query<&mut Sprite, With<Rewritten>>) {
    for mut s in &mut q {
        s.color = Color::srgb(1.0, 0.0, 0.0);
    }
}

/// What the weather does: rebuild its meshes every frame.
fn remesh(q: Query<&Mesh2d, With<Remeshed>>, mut meshes: ResMut<Assets<Mesh>>) {
    for m in &q {
        if let Some(mut mesh) = meshes.get_mut(&m.0) {
            *mesh = corner_triangle();
        }
    }
}

#[test]
fn rewritten_sprite_draws_with_two_custom_materials() {
    let mut app = headless_app();
    app.add_plugins((
        Material2dPlugin::<MatA>::default(),
        Material2dPlugin::<MatB>::default(),
    ))
    .add_systems(Startup, spawn_materials_scene)
    .add_systems(Update, (rewrite_sprite, remesh));
    let data = run(&mut app, 30);
    let c = pixel(&data, W / 2, H / 2);
    assert!(c[0] > 250 && c[1] < 5, "centre {c:?}: sprite not drawn");
    let t = pixel(&data, 9, 9);
    assert!(
        t[1] > 250 && t[0] < 5,
        "corner {t:?}: material meshes not drawn"
    );
}

// --- R21b -----------------------------------------------------------------------------------

/// `draw_set_alpha(0.5)` of 50 % grey over white: GameMaker gives 0.5·0.5 + 1·0.5 = 0.75 encoded
/// (191); blending in linear light would give ≈ 204.
#[test]
fn alpha_blending_happens_in_srgb_space() {
    let mut app = headless_app();
    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn(square(Color::WHITE, 1.0));
        commands.spawn(square(Color::srgba(0.5, 0.5, 0.5, 0.5), 2.0));
    });
    let c = pixel(&run(&mut app, 20), W / 2, H / 2);
    for ch in &c[..3] {
        assert!(
            (*ch as i32 - 191).abs() <= 2,
            "pixel {c:?}, want ≈191 (sRGB-space blend)"
        );
    }
}

/// `bm_add` of 50 % grey over 25 % grey (the additive rain): GameMaker gives 0.75 encoded (191);
/// adding in linear light would give ≈ 140.
#[test]
fn additive_blending_happens_in_srgb_space() {
    let mut app = headless_app();
    app.add_plugins(BlendPlugin).add_systems(
        Startup,
        |mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut mats: ResMut<Assets<GmBlend>>| {
            let g = 64.0 / 255.0;
            commands.spawn(square(Color::srgb(g, g, g), 1.0));
            let mut m = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            );
            let grey = 128.0 + 128.0 * 256.0 + 128.0 * 65536.0;
            let v = |x: f64, y: f64| Vert {
                x,
                y,
                col: grey,
                alpha: 1.0,
            };
            let (a, b) = (-32.0, 32.0);
            colour_mesh(
                &mut m,
                &[v(a, a), v(b, a), v(b, b), v(a, a), v(b, b), v(a, b)],
            );
            commands.spawn((
                Mesh2d(meshes.add(m)),
                MeshMaterial2d(mats.add(GmBlend::new(GmBlendMode::Add))),
                Transform::from_xyz(0.0, 0.0, 2.0),
            ));
        },
    );
    let c = pixel(&run(&mut app, 20), W / 2, H / 2);
    // 128 + 64 = 192 in 8-bit encoded space
    for ch in &c[..3] {
        assert!(
            (*ch as i32 - 192).abs() <= 2,
            "pixel {c:?}, want ≈192 (sRGB-space add)"
        );
    }
}

// --- GmQuad (render::batch) -----------------------------------------------------------------

#[derive(Resource, Default)]
struct Pool(myco_app::render::batch::BatchPool);

/// A 2×1 image (red | green) drawn 32 px wide over the target's left half (nearest texels),
/// tinted 50 % grey; then an additive 25 % grey rectangle over the right half.
fn draw_batch(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<myco_app::render::batch::GmQuad>>,
    mut images: ResMut<Assets<Image>>,
    white: Res<myco_app::render::batch::WhiteTexture>,
    mut pool: ResMut<Pool>,
    mut img: Local<Option<Handle<Image>>>,
) {
    use myco_app::render::batch::{BatchTarget, QuadBatch, QuadBlend};
    let img = img
        .get_or_insert_with(|| {
            images.add(Image::new(
                Extent3d {
                    width: 2,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![255, 0, 0, 255, 0, 255, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            ))
        })
        .clone();
    let grey = 128.0 + 128.0 * 256.0 + 128.0 * 65536.0;
    let mut b = QuadBatch::new();
    // world (y down) covering x -32..0, y -32..32
    b.image_ext(
        &img,
        [0.0, 0.0, 2.0, 1.0],
        [0.0, 0.0, 2.0, 1.0],
        -32.0,
        -32.0,
        16.0,
        64.0,
        0.0,
        grey,
        1.0,
    );
    b.set_blend(QuadBlend::Add);
    let q = 64.0;
    b.rect(0.0, -32.0, 31.0, 31.0, q + q * 256.0 + q * 65536.0, 1.0);
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut mats,
        white: &white.0,
    };
    pool.0.sync(&mut t, &b, 1.0, 0.01, &default());
}

#[test]
fn gm_quad_batch_draws_texels_tint_and_blend() {
    let mut app = headless_app();
    app.add_plugins(myco_app::render::batch::BatchPlugin)
        .init_resource::<Pool>()
        .add_systems(Update, draw_batch);
    let data = run(&mut app, 30);
    let red = pixel(&data, 8, 32);
    let green = pixel(&data, 24, 32);
    let add = pixel(&data, 48, 32);
    assert!(
        (red[0] as i32 - 128).abs() <= 2 && red[1] < 3,
        "left texel {red:?}"
    );
    assert!(
        (green[1] as i32 - 128).abs() <= 2 && green[0] < 3,
        "right texel {green:?}"
    );
    assert!((add[0] as i32 - 64).abs() <= 2, "added rect {add:?}");
}

/// A black | white 2×1 image stretched over the target with the filter on: GameMaker filters
/// encoded values, so the midpoint is ≈ 128 (filtering in linear light would give ≈ 188). A
/// black→white `draw_rectangle_colour` gradient likewise reaches ≈ 128 halfway.
fn draw_filtered(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<myco_app::render::batch::GmQuad>>,
    mut images: ResMut<Assets<Image>>,
    white: Res<myco_app::render::batch::WhiteTexture>,
    mut pool: ResMut<Pool>,
    mut img: Local<Option<Handle<Image>>>,
) {
    use myco_app::render::batch::{BatchTarget, QuadBatch, C_WHITE};
    let img = img
        .get_or_insert_with(|| {
            images.add(Image::new(
                Extent3d {
                    width: 2,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0, 0, 0, 255, 255, 255, 255, 255],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            ))
        })
        .clone();
    let mut b = QuadBatch::new();
    b.set_filter(true);
    // texels 0.5..1.5 (centre to centre) across the top half: x -32..32, y -32..0
    b.image_ext(
        &img,
        [0.5, 0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
        -32.0,
        -32.0,
        64.0,
        32.0,
        0.0,
        C_WHITE,
        1.0,
    );
    b.set_filter(false);
    // bottom half: horizontal gradient black -> white
    b.rect_colour(-32.0, 0.0, 31.0, 31.0, [0.0, C_WHITE, C_WHITE, 0.0], 1.0);
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut mats,
        white: &white.0,
    };
    pool.0.sync(&mut t, &b, 1.0, 0.01, &default());
}

#[test]
fn filtering_and_gradients_interpolate_encoded_values() {
    let mut app = headless_app();
    app.add_plugins(myco_app::render::batch::BatchPlugin)
        .init_resource::<Pool>()
        .add_systems(Update, draw_filtered);
    let data = run(&mut app, 30);
    let filt = pixel(&data, 32, 16);
    let grad = pixel(&data, 32, 48);
    assert!(
        (filt[0] as i32 - 128).abs() <= 6,
        "filtered midpoint {filt:?}, want ≈128"
    );
    assert!(
        (grad[0] as i32 - 128).abs() <= 6,
        "gradient midpoint {grad:?}, want ≈128"
    );
}

// --- sh_blackmetal (render::post, Task 15) ---------------------------------------------------

const BW_RGB: [u8; 3] = [200, 120, 40];

/// A 1×1 image in BW_RGB tone-mapped by `BwQuad`: the left half bright 0 (the picture), the
/// right half bright 1 (the glow's bright pass).
fn draw_bw(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<myco_app::render::post::BwQuad>>,
    mut images: ResMut<Assets<Image>>,
    mut done: Local<bool>,
) {
    use myco_app::render::batch::{run_mesh, QuadBatch, C_WHITE};
    use myco_app::render::post::{bw_shader_set, BwQuad};
    if *done {
        return;
    }
    *done = true;
    let img = images.add(Image::new(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![BW_RGB[0], BW_RGB[1], BW_RGB[2], 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    for (x, bright) in [(-32.0, 0.0), (0.0, 1.0)] {
        let mut b = QuadBatch::new();
        b.image_ext(
            &img,
            [0.0, 0.0, 1.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            x,
            -32.0,
            32.0,
            64.0,
            0.0,
            C_WHITE,
            1.0,
        );
        commands.spawn((
            Mesh2d(meshes.add(run_mesh(&b.runs[0].verts))),
            MeshMaterial2d(mats.add(BwQuad::new(img.clone(), &bw_shader_set(bright), false))),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ));
    }
}

#[test]
fn blackmetal_tone_maps_encoded_values_like_the_glsl() {
    use myco_app::render::post::{bw_shader_set, bw_tone, BwPlugin};
    let mut app = headless_app();
    app.add_plugins(BwPlugin).add_systems(Update, draw_bw);
    let data = run(&mut app, 30);
    let rgb = BW_RGB.map(|c| c as f64 / 255.0);
    for (x, bright) in [(16, 0.0), (48, 1.0)] {
        let want = (bw_tone(rgb, 1.0, &bw_shader_set(bright))[0] * 255.0).round() as i32;
        let got = pixel(&data, x, 32);
        assert!(
            (got[0] as i32 - want).abs() <= 2 && got[0] == got[1] && got[1] == got[2],
            "bright {bright}: {got:?}, want grey {want}"
        );
    }
}
