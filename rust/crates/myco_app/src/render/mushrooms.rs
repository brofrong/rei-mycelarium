//! Task 15: mushrooms on screen — `mushroom_draw` (sheet cell, grow-in, face, rotation), the
//! stinky stench (`stinky_art_init`, `stinky_draw`) and the repeller clouds (`repeller_*`).
//!
//! The stench is drawn on the CPU like the original's `stinky_draw`: a soft glow sprite per
//! particle (additive) and then the pixel itself. The GPU column shader `sh_stinky` ships with
//! the game but no script uses it (no `shader_set(sh_stinky)` in the decompiled scripts), so it
//! is not part of the look and is not ported.
//!
//! Repeller clouds go through two offscreen surfaces like the original: the puffs and specks at
//! `1/repeller_cloud_pixel` resolution into a body surface (blended as premultiplied colour), the
//! flash into an additive surface cleared to opaque black; both are then drawn back
//! `repeller_cloud_pixel` times larger with the filter off, which gives the clouds their chunky
//! pixels. The composite is part of `life_draw`'s painter order ([`clouds_composite`]); the
//! offscreen passes run on cameras of their own ([`CloudCamera`]).

use crate::assets::GameAssets;
use crate::camera::{Cameras, COMPOSITING};
use crate::render::batch::{
    BatchPlugin, BatchPool, BatchTarget, GmQuad, QuadBatch, QuadBlend, WhiteTexture, C_WHITE,
};
use crate::sim_bridge::SimRes;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use myco_core::balance::vis;
use myco_core::gml;
use myco_core::mushrooms::repeller::RepelCloud;
use myco_core::mushrooms::{stinky, types};
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::MushroomSave;

/// Render layers of the cloud body and flash surfaces (private to their cameras).
pub const CLOUD_LAYERS: [usize; 2] = [30, 31];
/// Camera order of the body surface pass; the flash pass follows.
const CLOUD_ORDER: isize = -70;

// port: mushroom_draw
/// `mushroom_draw(m, x, y, col, alpha, ang, scale)`: the mushroom's variant cell of its current
/// sheet with the sprite origin at (x, y), grown in, mirrored by `face`, rotated by `ang`
/// degrees about the origin.
#[allow(clippy::too_many_arguments)]
pub fn mushroom_draw(
    b: &mut QuadBatch,
    a: &GameAssets,
    sim: &Sim,
    m: &MushroomSave,
    x: f64,
    y: f64,
    col: f64,
    alpha: f64,
    ang: f64,
    scale: f64,
) {
    let spr = types::mushroom_sprite_now(m);
    let Some(def) = a.try_sprite(spr) else {
        return;
    };
    let cw = types::mushroom_cell_width(spr);
    let v = types::mushroom_variant(m);
    let left = types::mushroom_cell_left(spr, v);
    let s = types::shroom_grow_scale(sim, m) * scale;
    // m[$ "face"] ?? 1
    let face = m.extra.get("face").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let xs = s * face;
    let w = cw.min(def.size.x as f64 - left);
    let h = def.size.y as f64;
    let ox = -(def.origin.x as f64) * xs;
    let oy = -(def.origin.y as f64) * s;
    if ang == 0.0 {
        b.sprite_part_ext(
            a,
            spr,
            0,
            [left, 0.0, w, h],
            x + ox,
            y + oy,
            xs,
            s,
            col,
            alpha,
        );
        return;
    }
    let c = gml::dcos(ang);
    let sn = gml::dsin(ang);
    b.sprite_general(
        a,
        spr,
        0,
        [left, 0.0, w, h],
        x + ox * c + oy * sn,
        y - ox * sn + oy * c,
        xs,
        s,
        ang,
        col,
        alpha,
    );
}

/// The procedural sprites of `stinky_art_init` and `repeller_cloud_art_init`.
#[derive(Resource, Clone, Debug)]
pub struct FxArt {
    /// `global.stinky_glow` (16×16, origin 8, 8)
    pub stinky_glow: Handle<Image>,
    /// `global.repel_flash_sprite` (256×256, origin at the centre)
    pub repel_flash: Handle<Image>,
    /// `global.repel_puff_sprite` (64×64, origin at the centre)
    pub repel_puff: Handle<Image>,
}
// `global.stinky_pixel` (a 1×1 white sprite) is the batch's white texture.

/// RGBA8 pixels of `stinky_glow`: on a cleared 16×16 surface, filled white circles of radius
/// r = 8 down to 1 in half-pixel steps at alpha `0.1·k² + 0.02` (k = 1 − rr/r), blended like the
/// LÖVE port (its 16-segment fan, "alpha" blend: colour and alpha both accumulate
/// `v = v·(1 − a) + a`, quantised to 8 bits per circle).
pub fn stinky_glow_pixels() -> Vec<u8> {
    const SZ: usize = 16;
    let r = SZ as f64 * 0.5;
    let mut v = vec![0u8; SZ * SZ];
    let mut rr = r;
    while rr >= 1.0 {
        let k = 1.0 - rr / r;
        let a = 0.1 * k * k + 0.02;
        let seg = 16.0f64.max(96.0f64.min(rr.floor()));
        let n = seg as usize;
        let pts: Vec<(f64, f64)> = (0..=n)
            .map(|i| {
                let t = i as f64 / seg * std::f64::consts::TAU;
                (r + t.cos() * rr, r + t.sin() * rr)
            })
            .collect();
        for y in 0..SZ {
            for x in 0..SZ {
                let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
                // inside the convex fan: on the inner side of every edge
                let inside = pts.windows(2).all(|e| {
                    let (x1, y1) = e[0];
                    let (x2, y2) = e[1];
                    (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1) >= 0.0
                });
                if inside {
                    let d = v[y * SZ + x] as f64 / 255.0;
                    v[y * SZ + x] = ((d * (1.0 - a) + a) * 255.0).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
        rr -= 0.5;
    }
    v.iter().flat_map(|&c| [c, c, c, c]).collect()
}

fn rgba_image(size: u32, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

// port: stinky_art_init
/// `stinky_art_init`: the stench glow sprite (the pixel is the white texture).
pub fn stinky_art_init(images: &mut Assets<Image>) -> Handle<Image> {
    images.add(rgba_image(16, stinky_glow_pixels()))
}

// port: repeller_soft_sprite
/// `repeller_soft_sprite(size, hardness)`: RGBA8 pixels of a white disc whose alpha falls off as
/// `(1 − d)^hardness` from the centre (d = distance / radius at pixel centres).
pub fn repeller_soft_sprite(size: usize, hardness: f64) -> Vec<u8> {
    let r = size as f64 * 0.5;
    let mut out = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let d = gml::point_distance(x as f64 + 0.5, y as f64 + 0.5, r, r) / r;
            let a = gml::power((1.0 - d).max(0.0), hardness);
            out.extend([255, 255, 255, gml::round(a * 255.0) as u8]);
        }
    }
    out
}

// port: repeller_cloud_art_init
/// `repeller_cloud_art_init`: the flash (256 px, hardness 2.2) and puff (64 px, 1.4) sprites.
pub fn repeller_cloud_art_init(images: &mut Assets<Image>) -> (Handle<Image>, Handle<Image>) {
    (
        images.add(rgba_image(256, repeller_soft_sprite(256, 2.2))),
        images.add(rgba_image(64, repeller_soft_sprite(64, 1.4))),
    )
}

/// A procedural sprite of `size` texels drawn with its centre (the origin) at (x, y).
#[allow(clippy::too_many_arguments)]
fn centred(
    b: &mut QuadBatch,
    img: &Handle<Image>,
    size: f64,
    x: f64,
    y: f64,
    xs: f64,
    ys: f64,
    col: f64,
    alpha: f64,
) {
    let h = size * 0.5;
    b.image_ext(
        img,
        [0.0, 0.0, size, size],
        [-h, -h, h, h],
        x,
        y,
        xs,
        ys,
        0.0,
        col,
        alpha,
    );
}

/// `draw_sprite_ext(global.stinky_pixel, 0, x, y, p, p, 0, col, a)`: a `p × p` square.
fn pixel(b: &mut QuadBatch, x: f64, y: f64, p: f64, col: f64, alpha: f64) {
    let q = [[x, y], [x + p, y], [x + p, y + p], [x, y + p]];
    b.quad(None, q, [[0.5, 0.5]; 4], col, alpha);
}

// port: stinky_draw
/// `stinky_draw(m)`: the live stench particles inside the view (+ the glow's pad): first every
/// glow additively, then every pixel.
pub fn stinky_draw(b: &mut QuadBatch, fx: &FxArt, m: &MushroomSave, view: &ViewRect) {
    let Some(s) = stinky::pool_of(m) else {
        return;
    };
    if s.n == 0 || !m.stench_live.unwrap_or(true) {
        return;
    }
    let n = s.n.min(s.px.len()).min(s.py.len());
    let gr = (1.0 + vis("stinky_glow_radius_px")) / 8.0;
    let ga = vis("stinky_glow_alpha") * 2.0;
    let pad = 8.0 * gr * 1.2 + 4.0;
    let l = view.x - pad;
    let t = view.y - pad;
    let r = l + view.w + pad * 2.0;
    let bt = t + view.h + pad * 2.0;
    let inside = |i: usize| {
        let (x, y) = (s.px[i], s.py[i]);
        !(x < l || x > r || y < t || y > bt)
    };
    b.set_blend(QuadBlend::Add);
    for i in (0..n).filter(|&i| inside(i)) {
        let p = s.pix[i];
        let sc = gr * (0.6 + 0.2 * p);
        centred(
            b,
            &fx.stinky_glow,
            16.0,
            s.px[i] + p * 0.5,
            s.py[i] + p * 0.5,
            sc,
            sc,
            s.colour[i],
            (s.alpha[i] * ga).min(1.0),
        );
    }
    b.set_blend(QuadBlend::Alpha);
    for i in (0..n).filter(|&i| inside(i)) {
        pixel(b, s.px[i], s.py[i], s.pix[i], s.colour[i], s.alpha[i]);
    }
}

// port: repeller_cloud_draw_body
/// `repeller_cloud_draw_body(c, vx, vy, s)`: into the body surface (`s` = 1 / pixel size): a
/// darker shadow pass of every puff, the puffs, then the specks as pixels.
pub fn repeller_cloud_draw_body(
    b: &mut QuadBatch,
    fx: &FxArt,
    c: &RepelCloud,
    vx: f64,
    vy: f64,
    s: f64,
) {
    let half = 32.0;
    let alpha = vis("repeller_cloud_alpha");
    let shade = 11054250.0;
    for p in &c.puffs {
        let a = alpha * (p.life / p.total / 0.5).min(1.0) * 0.45;
        let sc = p.size * 0.62 / half * s;
        centred(
            b,
            &fx.repel_puff,
            64.0,
            (p.x + 2.0 - vx) * s,
            (p.y + p.size * 0.18 - vy) * s,
            sc,
            sc,
            gml::merge_colour(p.colour, shade, 0.35),
            a,
        );
    }
    for p in &c.puffs {
        let a = alpha * (p.life / p.total / 0.5).min(1.0);
        let sc = p.size * 0.5 / half * s;
        centred(
            b,
            &fx.repel_puff,
            64.0,
            (p.x - vx) * s,
            (p.y - p.size * 0.08 - vy) * s,
            sc,
            sc,
            p.colour,
            a,
        );
    }
    let px = (2.0 * s).max(1.0);
    for k in &c.specks {
        let sx = (k.x - vx) * s;
        let sy = (k.y - vy) * s;
        let a = (k.life / k.total * 2.0).min(1.0) * 0.8;
        pixel(b, sx.floor(), sy.floor(), px, C_WHITE, a);
    }
}

// port: repeller_cloud_draw_flash
/// `repeller_cloud_draw_flash(c, vx, vy, s)`: the firing flash (halo, core pushed toward the
/// shot, hot centre) per layer, growing and fading over `repeller_cloud_flash_frames`.
pub fn repeller_cloud_draw_flash(
    b: &mut QuadBatch,
    fx: &FxArt,
    c: &RepelCloud,
    vx: f64,
    vy: f64,
    s: f64,
) {
    if c.flash <= 0.0 {
        return;
    }
    let half = 128.0;
    let t = 1.0 - c.flash / vis("repeller_cloud_flash_frames").max(1.0);
    let rad = gml::lerp(30.0, vis("repeller_cloud_flash_px"), t.sqrt()) * s;
    let a = 1.0 - t * t;
    let sx = (c.x - vx) * s;
    let sy = (c.y - vy) * s;
    let layers = vis("repeller_cloud_flash_layers").max(1.0);
    let mut i = 0.0;
    while i < layers {
        let halo = rad * 1.8 / half;
        centred(
            b,
            &fx.repel_flash,
            256.0,
            sx,
            sy,
            halo,
            halo,
            C_WHITE,
            a * 0.6,
        );
        let core_x = rad / half;
        let core_y = rad * 0.85 / half;
        centred(
            b,
            &fx.repel_flash,
            256.0,
            sx + c.dir * rad * 0.3,
            sy,
            core_x,
            core_y,
            C_WHITE,
            a,
        );
        let hot = rad * 0.45 / half;
        centred(b, &fx.repel_flash, 256.0, sx, sy, hot, hot, C_WHITE, a);
        i += 1.0;
    }
}

/// The body/flash surface placement for a view: the texel grid origin (world px, snapped to
/// the pixel size) and the surface size in texels (`repeller_clouds_draw`).
pub fn repeller_cloud_surface_size(v: &ViewRect) -> (f64, f64, u32, u32) {
    let px = gml::round(vis("repeller_cloud_pixel")).max(1.0);
    let ox = (v.x / px).floor() * px;
    let oy = (v.y / px).floor() * px;
    let sw = (v.w / px).ceil() + 2.0;
    let sh = (v.h / px).ceil() + 2.0;
    (ox, oy, sw as u32, sh as u32)
}

fn surface_image(w: u32, h: u32) -> Image {
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

// port: repeller_cloud_surface
/// `repeller_cloud_surface(name, w, h)`: the named surface, recreated when its size changed.
pub fn repeller_cloud_surface(
    images: &mut Assets<Image>,
    slot: &mut Option<(Handle<Image>, UVec2)>,
    w: u32,
    h: u32,
) -> Handle<Image> {
    let size = UVec2::new(w.max(1), h.max(1));
    if let Some((img, s)) = slot {
        if *s == size {
            return img.clone();
        }
    }
    let img = images.add(surface_image(size.x, size.y));
    *slot = Some((img.clone(), size));
    img
}

/// What `repeller_clouds_draw` puts into its surfaces this frame and where they go back.
#[derive(Clone, Debug)]
pub struct CloudPlan {
    /// texel grid origin, world px
    pub ox: f64,
    pub oy: f64,
    /// world px per texel
    pub px: f64,
    pub sw: u32,
    pub sh: u32,
    /// the body surface's draws (texel coordinates)
    pub body: QuadBatch,
    /// the flash surface's draws, when any shown cloud flashes
    pub flash: Option<QuadBatch>,
}

// port: repeller_clouds_draw
/// `repeller_clouds_draw`: the clouds within 500 px of the view, drawn into the body surface
/// (filter on, premultiplying blend `src_alpha, inv_src_alpha / one, inv_src_alpha`) and, when
/// one flashes, the flash surface (cleared opaque black, additive). The surfaces are drawn back
/// by [`clouds_composite`] at the cloud's place in `life_draw`. `None`: nothing to draw (the
/// original returns before touching the texture filter).
///
/// The view is the zoomed render view (the original's camera view at zoom 1), so zooming out
/// keeps every visible cloud on its surface.
pub fn repeller_clouds_draw(sim: &Sim, fx: &FxArt, v: &ViewRect) -> Option<CloudPlan> {
    let shown: Vec<&RepelCloud> = sim
        .mushrooms
        .repel_clouds
        .iter()
        .filter(|c| c.x >= v.x - 500.0 && c.x <= v.x + v.w + 500.0)
        .collect();
    if shown.is_empty() {
        return None;
    }
    let px = gml::round(vis("repeller_cloud_pixel")).max(1.0);
    let inv = 1.0 / px;
    let (ox, oy, sw, sh) = repeller_cloud_surface_size(v);
    let mut body = QuadBatch::new();
    body.set_filter(true);
    body.set_blend(QuadBlend::Alpha);
    for c in &shown {
        repeller_cloud_draw_body(&mut body, fx, c, ox, oy, inv);
    }
    let flash = shown.iter().any(|c| c.flash > 0.0).then(|| {
        let mut f = QuadBatch::new();
        f.set_filter(true);
        f.set_blend(QuadBlend::Add);
        for c in &shown {
            repeller_cloud_draw_flash(&mut f, fx, c, ox, oy, inv);
        }
        f
    });
    Some(CloudPlan {
        ox,
        oy,
        px,
        sw,
        sh,
        body,
        flash,
    })
}

/// The offscreen surfaces as `life_draw` composites them this frame.
#[derive(Resource, Clone, Debug, Default)]
pub struct CloudFrame {
    /// a cloud is shown (the passes ran this frame)
    pub active: bool,
    pub ox: f64,
    pub oy: f64,
    pub px: f64,
    pub sw: u32,
    pub sh: u32,
    pub any_flash: bool,
    pub body: Handle<Image>,
    pub flash: Handle<Image>,
}

/// The end of `repeller_clouds_draw`: filter off, the body surface premultiplied
/// (`bm_one, bm_inv_src_alpha`), the flash surface additively, `pixel` times larger; then the
/// filter back to `gpu_get_tex_filter()`'s value — always true in the LÖVE port.
pub fn clouds_composite(b: &mut QuadBatch, f: &CloudFrame) {
    if !f.active {
        return;
    }
    let (w, h) = (f.sw as f64, f.sh as f64);
    b.set_filter(false);
    b.set_blend(QuadBlend::Premul);
    b.image_ext(
        &f.body,
        [0.0, 0.0, w, h],
        [0.0, 0.0, w, h],
        f.ox,
        f.oy,
        f.px,
        f.px,
        0.0,
        C_WHITE,
        1.0,
    );
    if f.any_flash {
        b.set_blend(QuadBlend::Add);
        b.image_ext(
            &f.flash,
            [0.0, 0.0, w, h],
            [0.0, 0.0, w, h],
            f.ox,
            f.oy,
            f.px,
            f.px,
            0.0,
            C_WHITE,
            1.0,
        );
    }
    b.set_blend(QuadBlend::Alpha);
    b.set_filter(true);
}

/// Cloud surface pass camera: pass index (0 body, 1 flash) and its current target.
#[derive(Component)]
pub struct CloudCamera(usize, AssetId<Image>);

#[derive(Resource, Default)]
struct CloudTargets {
    body: Option<(Handle<Image>, UVec2)>,
    flash: Option<(Handle<Image>, UVec2)>,
    pools: [BatchPool; 2],
}

fn build_fx_art(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let stinky_glow = stinky_art_init(&mut images);
    let (repel_flash, repel_puff) = repeller_cloud_art_init(&mut images);
    commands.insert_resource(FxArt {
        stinky_glow,
        repel_flash,
        repel_puff,
    });
}

fn spawn_cloud_cameras(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    for (i, &layer) in CLOUD_LAYERS.iter().enumerate() {
        let img = images.add(surface_image(8, 8));
        let clear = if i == 0 { Color::NONE } else { Color::BLACK };
        commands.spawn((
            Camera2d,
            Camera {
                order: CLOUD_ORDER + i as isize,
                clear_color: ClearColorConfig::Custom(clear),
                is_active: false,
                ..default()
            },
            RenderTarget::Image(img.clone().into()),
            RenderLayers::layer(layer),
            COMPOSITING,
            CloudCamera(i, img.id()),
        ));
    }
}

/// The cloud surfaces each frame: plan, fill the passes, aim the cameras, publish
/// [`CloudFrame`] for `life_draw`.
#[allow(clippy::too_many_arguments)]
fn clouds_system(
    mut commands: Commands,
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    fx: Option<Res<FxArt>>,
    mut targets: ResMut<CloudTargets>,
    mut frame: ResMut<CloudFrame>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmQuad>>,
    white: Res<WhiteTexture>,
    mut cam_q: Query<(
        Entity,
        &mut Camera,
        &mut Projection,
        &mut Transform,
        &mut CloudCamera,
    )>,
) {
    let plan = fx
        .as_deref()
        .and_then(|fx| repeller_clouds_draw(&sim.0, fx, &cams.render_view()));
    let targets = &mut *targets;
    let active = plan.is_some();
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        white: &white.0,
    };
    let mut new_frame = CloudFrame::default();
    if let Some(p) = &plan {
        let body = repeller_cloud_surface(&mut images, &mut targets.body, p.sw, p.sh);
        let flash = repeller_cloud_surface(&mut images, &mut targets.flash, p.sw, p.sh);
        let empty = QuadBatch::new();
        targets.pools[0].sync(
            &mut t,
            &p.body,
            0.0,
            crate::render::layers::Z_STEP,
            &RenderLayers::layer(CLOUD_LAYERS[0]),
        );
        targets.pools[1].sync(
            &mut t,
            p.flash.as_ref().unwrap_or(&empty),
            0.0,
            crate::render::layers::Z_STEP,
            &RenderLayers::layer(CLOUD_LAYERS[1]),
        );
        new_frame = CloudFrame {
            active: true,
            ox: p.ox,
            oy: p.oy,
            px: p.px,
            sw: p.sw,
            sh: p.sh,
            any_flash: p.flash.is_some(),
            body,
            flash,
        };
    }
    let imgs = [new_frame.body.clone(), new_frame.flash.clone()];
    let (w, h) = (new_frame.sw as f32, new_frame.sh as f32);
    for (e, mut cam, mut proj, mut tf, mut cc) in &mut cam_q {
        let on = active && (cc.0 == 0 || new_frame.any_flash);
        if cam.is_active != on {
            cam.is_active = on;
        }
        if !on {
            continue;
        }
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
        // the surface's top-left is texel (0, 0) (y down)
        tf.set_if_neq(Transform::from_xyz(w * 0.5, -h * 0.5, 0.0));
        if cc.1 != imgs[cc.0].id() {
            t.commands
                .entity(e)
                .insert(RenderTarget::Image(imgs[cc.0].clone().into()));
            cc.1 = imgs[cc.0].id();
        }
    }
    *frame = new_frame;
}

/// The set of [`clouds_system`] (life drawing composites its surfaces after it).
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CloudsSet;

pub struct MushroomFxPlugin;

impl Plugin for MushroomFxPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BatchPlugin>() {
            app.add_plugins(BatchPlugin);
        }
        app.init_resource::<CloudTargets>()
            .init_resource::<CloudFrame>()
            .add_systems(Startup, (build_fx_art, spawn_cloud_cameras))
            .add_systems(Update, clouds_system.in_set(CloudsSet));
    }
}
