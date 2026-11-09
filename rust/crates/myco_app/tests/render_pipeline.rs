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
