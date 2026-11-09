//! Rain drawing (scr_weather): streaks, gloom, splashes. The rain state is myco_core's
//! (`sim.weather`, Task 12); the functions here build vertex lists from it and
//! [`weather_draw`] puts them in four fixed meshes:
//! - band 0 behind the world (`weather_draw_back`, from forest Draw_0) at `Z_WEATHER_BACK`;
//! - the gloom tint over everything, then bands 1-2 and the splashes (`weather_draw_front`, from
//!   forest Draw_73 after the turf) at `Z_WEATHER_GLOOM..Z_WEATHER_SPLASHES`.

use crate::camera::Cameras;
use crate::render::blend::{colour_mesh, empty_colour_mesh, BlendPlugin, GmBlend, GmBlendMode};
use crate::render::layers;
use crate::sim_bridge::SimRes;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use myco_core::balance::{vis, wld};
use myco_core::gml;
use myco_core::weather::WeatherRt;

pub use crate::render::blend::Vert;

fn v(x: f64, y: f64, col: f64, alpha: f64) -> Vert {
    Vert { x, y, col, alpha }
}

// port: rain_streak
/// `rain_streak(x, y, dx, dy, width, col, alpha)`: a quad from the head (x, y), `width` wide and
/// opaque, to the tail (x - dx, y - dy), 0.3 as wide and transparent (two triangles).
#[allow(clippy::too_many_arguments)]
pub fn rain_streak(
    out: &mut Vec<Vert>,
    x: f64,
    y: f64,
    dx: f64,
    dy: f64,
    width: f64,
    col: f64,
    alpha: f64,
) {
    let len = gml::point_distance(0.0, 0.0, dx, dy);
    if len <= 0.0 {
        return;
    }
    let hw = width * 0.5;
    let nx = -dy / len * hw;
    let ny = dx / len * hw;
    let (tx, ty) = (x - dx, y - dy);
    let (hx1, hy1, hx2, hy2) = (x + nx, y + ny, x - nx, y - ny);
    let (tx1, ty1) = (tx + nx * 0.3, ty + ny * 0.3);
    let (tx2, ty2) = (tx - nx * 0.3, ty - ny * 0.3);
    out.extend([
        v(hx1, hy1, col, alpha),
        v(hx2, hy2, col, alpha),
        v(tx1, ty1, col, 0.0),
        v(tx1, ty1, col, 0.0),
        v(hx2, hy2, col, alpha),
        v(tx2, ty2, col, 0.0),
    ]);
}

// port: rain_bands_draw
/// `rain_bands_draw(w, band_min, band_max, vx, vy, vw, vh)`: streaks of the active drops of the
/// given bands inside the view (additive).
pub fn rain_bands_draw(
    w: &WeatherRt,
    band_min: f64,
    band_max: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) -> Vec<Vert> {
    let mut out = Vec::new();
    let n = gml::round(w.drops.len() as f64 * w.intensity);
    if n <= 0.0 {
        return out;
    }
    let col = w.colour as f64;
    let inten = w.intensity;
    let (left, right) = (vx - 48.0, vx + vw + 48.0);
    let top = vy - 64.0;
    let bottom = (vy + vh).min(wld("surface_height_px")) + 8.0;
    for d in w.drops.iter().take(n as usize) {
        let band = d.band as f64;
        if band < band_min || band > band_max {
            continue;
        }
        if d.x < left || d.x > right || d.y < top || d.y > bottom {
            continue;
        }
        let Some(b) = w.bands.get(d.band) else {
            continue;
        };
        let dx = w.wind * b.drift;
        let dy = b.speed * d.speed_scale;
        let dl = gml::point_distance(0.0, 0.0, dx, dy).max(1.0);
        let len = b.len * d.len_scale;
        rain_streak(
            &mut out,
            d.x,
            d.y,
            dx / dl * len,
            dy / dl * len,
            b.width,
            col,
            b.alpha * d.alpha_scale * inten,
        );
    }
    out
}

// port: rain_gloom_draw
/// `rain_gloom_draw(w, ...)`: the colour (BGR) the view is multiplied by, `None` when there is
/// no gloom. The caller fills the view rectangle with it (multiply blend).
pub fn rain_gloom_draw(w: &WeatherRt) -> Option<f64> {
    let strength = vis("rain_gloom_strength");
    let s = w.gloom_shade() * strength;
    if s <= 0.0 {
        return None;
    }
    let gc = w.gloom_colour as f64;
    let r = gml::gmod(gc.floor(), 256.0);
    let g = gml::gmod((gc / 256.0).floor(), 256.0);
    let b = gml::gmod((gc / 65536.0).floor(), 256.0);
    let (r, g, b) = (
        gml::lerp(255.0, r, s).floor(),
        gml::lerp(255.0, g, s).floor(),
        gml::lerp(255.0, b, s).floor(),
    );
    Some(b * 65536.0 + g * 256.0 + r) // make_colour_rgb
}

/// Segments of an ellipse outline (GameMaker's default circle precision).
const ELLIPSE_SEGMENTS: usize = 24;

/// `draw_ellipse(x1, y1, x2, y2, true)`: a 1-px outline as a ring of quads.
fn ellipse_outline(out: &mut Vec<Vert>, cx: f64, cy: f64, rx: f64, ry: f64, col: f64, a: f64) {
    let (ix, iy) = ((rx - 0.5).max(0.0), (ry - 0.5).max(0.0));
    let (ox, oy) = (rx + 0.5, ry + 0.5);
    let pt = |k: usize, rx: f64, ry: f64| {
        let t = k as f64 / ELLIPSE_SEGMENTS as f64 * std::f64::consts::TAU;
        v(cx + rx * t.cos(), cy + ry * t.sin(), col, a)
    };
    for k in 0..ELLIPSE_SEGMENTS {
        let (i0, i1) = (pt(k, ix, iy), pt(k + 1, ix, iy));
        let (o0, o1) = (pt(k, ox, oy), pt(k + 1, ox, oy));
        out.extend([i0, o0, o1, i0, o1, i1]);
    }
}

// port: rain_splashes_draw
/// `rain_splashes_draw(w, vx, vw)`: expanding rings, then spray streaks (all additive).
pub fn rain_splashes_draw(w: &WeatherRt, vx: f64, vw: f64) -> Vec<Vert> {
    let mut out = Vec::new();
    let col = w.colour as f64;
    let (left, right) = (vx - 32.0, vx + vw + 32.0);
    for s in &w.splashes {
        if s.x < left || s.x > right {
            continue;
        }
        let k = s.t / s.life;
        let r = s.size * (0.25 + k * 1.6);
        ellipse_outline(&mut out, s.x, s.y, r, r * 0.3, col, (1.0 - k) * 0.45);
    }
    for p in &w.spray {
        if p.x < left || p.x > right {
            continue;
        }
        let f = 1.0 - p.t / p.life;
        rain_streak(
            &mut out,
            p.x,
            p.y,
            p.vx * 1.6,
            p.vy * 1.6,
            1.6,
            col,
            0.5 * f,
        );
    }
    out
}

// port: weather_draw_back
/// `weather_draw_back(vx, vy, vw, vh)`: band 0, behind the world.
pub fn weather_draw_back(w: &WeatherRt, vx: f64, vy: f64, vw: f64, vh: f64) -> Vec<Vert> {
    if vy >= wld("surface_height_px") || w.intensity <= 0.0 {
        return Vec::new();
    }
    rain_bands_draw(w, 0.0, 0.0, vx, vy, vw, vh)
}

/// What `weather_draw_front` draws, in order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WeatherFront {
    /// gloom tint (BGR) over the whole view
    pub gloom: Option<f64>,
    pub bands: Vec<Vert>,
    pub splashes: Vec<Vert>,
}

// port: weather_draw_front
/// `weather_draw_front(vx, vy, vw, vh)`: gloom, bands 1-2, splashes.
pub fn weather_draw_front(w: &WeatherRt, vx: f64, vy: f64, vw: f64, vh: f64) -> WeatherFront {
    let mut f = WeatherFront {
        gloom: rain_gloom_draw(w),
        ..Default::default()
    };
    if vy >= wld("surface_height_px") {
        return f;
    }
    if w.intensity > 0.0 {
        f.bands = rain_bands_draw(w, 1.0, 2.0, vx, vy, vw, vh);
    }
    if !w.splashes.is_empty() || !w.spray.is_empty() {
        f.splashes = rain_splashes_draw(w, vx, vw);
    }
    f
}

/// The four weather meshes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub enum WeatherMesh {
    Back,
    Gloom,
    Front,
    Splashes,
}

fn spawn_weather_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmBlend>>,
) {
    let add = materials.add(GmBlend::new(GmBlendMode::Add));
    let mul = materials.add(GmBlend::new(GmBlendMode::Multiply));
    for (kind, z) in [
        (WeatherMesh::Back, layers::Z_WEATHER_BACK),
        (WeatherMesh::Gloom, layers::Z_WEATHER_GLOOM),
        (WeatherMesh::Front, layers::Z_WEATHER_FRONT),
        (WeatherMesh::Splashes, layers::Z_WEATHER_SPLASHES),
    ] {
        let mut e = commands.spawn((
            Mesh2d(meshes.add(empty_colour_mesh())),
            Transform::from_xyz(0.0, 0.0, z),
            Visibility::Hidden,
            NoFrustumCulling,
            kind,
        ));
        if kind == WeatherMesh::Gloom {
            e.insert(MeshMaterial2d(mul.clone()));
        } else {
            e.insert(MeshMaterial2d(add.clone()));
        }
    }
}

// port: gml_Object_obj_forest_controller_Draw_73
/// Update: rebuild the rain meshes from `sim.weather` for the render view. (Draw_73 draws the
/// turf first; that is `world_draw_turf`, Task 13, at `Z_TURF` below these layers.)
pub fn weather_draw(
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q: Query<(&WeatherMesh, &Mesh2d, &mut Visibility)>,
) {
    let w = &sim.0.weather;
    let rv = cams.render_view();
    let back = weather_draw_back(w, rv.x, rv.y, rv.w, rv.h);
    let front = weather_draw_front(w, rv.x, rv.y, rv.w, rv.h);
    let gloom: Vec<Vert> = front.gloom.map_or_else(Vec::new, |c| {
        // draw_rectangle(x, y, x + ww, y + hh, false) fills one pixel past the corner
        let (x1, y1, x2, y2) = (rv.x, rv.y, rv.x + rv.w + 1.0, rv.y + rv.h + 1.0);
        let p = |x, y| v(x, y, c, 1.0);
        vec![
            p(x1, y1),
            p(x2, y1),
            p(x2, y2),
            p(x1, y1),
            p(x2, y2),
            p(x1, y2),
        ]
    });
    for (kind, mesh, mut vis) in &mut q {
        let verts = match kind {
            WeatherMesh::Back => &back,
            WeatherMesh::Gloom => &gloom,
            WeatherMesh::Front => &front.bands,
            WeatherMesh::Splashes => &front.splashes,
        };
        let want = if verts.is_empty() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *vis != want {
            *vis = want;
        }
        if !verts.is_empty() {
            if let Some(mut m) = meshes.get_mut(&mesh.0) {
                colour_mesh(&mut m, verts);
            }
        }
    }
}

/// Weather rendering: blend materials, the four meshes, the per-frame rebuild.
pub struct WeatherRenderPlugin;

impl Plugin for WeatherRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BlendPlugin>() {
            app.add_plugins(BlendPlugin);
        }
        app.add_systems(Startup, spawn_weather_meshes)
            .add_systems(Update, weather_draw);
    }
}
