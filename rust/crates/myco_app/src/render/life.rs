//! Task 15: wildlife on screen — obj_life Draw_0 (`life_draw`): leaves, mushrooms and their
//! flies, the stench, hedgehogs with what they carry, boars and flung mushrooms, the repeller
//! clouds, squirrels with stolen mushrooms, crows, water drops; plus `worms_draw` (the forest's
//! Draw_0, under the trees), `leaf_tendrils_draw` (end of the mycelium's `myc_draw_strands`) and
//! the pixel inits (`hedgehog_back_init`, `squirrel_trunk_edges`, `leaf_art_read`, ...) whose
//! data logic embeds (ruling R7; tests check both agree).
//!
//! Persistent world objects are entities keyed by their stable sim id ([`LifeKey`]): `life_draw`
//! returns its painter's order as segments, one per mushroom / animal plus shared segments for
//! the leaves, stench, flung mushrooms, clouds and drops; [`life_draw_system`] keeps one
//! [`LifeEntity`] (with its pooled meshes) per segment key, spawned when the object shows up
//! and despawned when it goes (or leaves the view, as the original culls it).
//!
//! Views: the original culls against the camera view; here the zoomed render view (identical at
//! zoom 1, and nothing visible is culled when zoomed out).

use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::render::batch::{
    BatchPlugin, BatchPool, BatchTarget, GmQuad, QuadBatch, QuadBlend, WhiteTexture, C_WHITE,
};
use crate::render::blend::{GmBlend, GmBlendMode};
use crate::render::layers::{LAYER_SPAN, Z_LIFE, Z_MYC_STRANDS, Z_STEP, Z_TURF, Z_WORMS};
use crate::render::leaf_art::{leaf_shape_images, strip_scan, LeafFgArt, LeafFgSprite, PagePixels};
use crate::render::mushrooms::{clouds_composite, mushroom_draw, stinky_draw, CloudFrame, FxArt};
use crate::render::mycelium::{fill_strand_mesh, strand_layers, V2};
use crate::sim_bridge::SimRes;
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use myco_core::balance::vis;
use myco_core::gml;
use myco_core::life::art::{LeafArt, LeafArtFrame, TrunkEdges};
use myco_core::life::{
    boar_frame_w, crow, hedgehog, squirrel, Cell, Hedgehog, Leaf, LifeRt, Squirrel,
};
use myco_core::mushrooms::{mushroom_index_of, types};
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::MushroomSave;
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------------------------
// pixel inits (R7: the data logic reads is embedded in myco_core; these recompute it)

fn alpha_at(px: &PagePixels, def: &crate::assets::SpriteDef, x: i64, y: i64) -> u8 {
    px.sprite_alpha(def, 0, x, y)
}

// port: hedgehog_back_init
/// `hedgehog_back_init`: per frame of the 8-frame sheet, the column of the lowest smoothed top
/// edge near the first frame's best column, as `[x + 0.5, smoothed top]` (frame px, unscaled).
pub fn hedgehog_back_init(px: &PagePixels, assets: &GameAssets) -> Vec<[f64; 2]> {
    let def = assets.sprite("hedgehog");
    let (w, h) = (def.size.x as i64, def.size.y as i64);
    let fw = w / 8;
    let top: Vec<f64> = (0..w)
        .map(|xx| {
            (0..h)
                .find(|&yy| alpha_at(px, def, xx, yy) >= 128)
                .map_or(h as f64, |yy| yy as f64)
        })
        .collect();
    let smooth = vis("hedgehog_back_smooth_cols") as i64;
    let sm: Vec<f64> = (0..w)
        .map(|xx| {
            let f0 = xx / fw * fw;
            let (mut sum, mut n) = (0.0, 0.0);
            for k in -smooth..=smooth {
                let c = xx + k;
                if c >= f0 && c < f0 + fw {
                    sum += top[c as usize];
                    n += 1.0;
                }
            }
            sum / f64::max(1.0, n)
        })
        .collect();
    let mut best = 0usize;
    for xx in 1..fw as usize {
        if sm[xx] < sm[best] {
            best = xx;
        }
    }
    let follow = vis("hedgehog_back_follow_cols") as i64;
    let fw = fw as usize;
    (0..8usize)
        .map(|f| {
            let mut bx = best.min(fw - 1);
            let lo = (best as i64 - follow).max(0) as usize;
            let hi = (best + follow as usize).min(fw - 1);
            for c in lo..=hi {
                if sm[f * fw + c] < sm[f * fw + bx] {
                    bx = c;
                }
            }
            [bx as f64 + 0.5, sm[f * fw + bx]]
        })
        .collect()
}

// port: squirrel_trunk_edges
/// `squirrel_trunk_edges(spr)`: per sprite row, the opaque run around the origin column (the
/// previous row's edges where none), median-smoothed over `squirrel_edge_smooth_rows`.
pub fn squirrel_trunk_edges(px: &PagePixels, assets: &GameAssets, name: &str) -> TrunkEdges {
    let def = assets.sprite(name);
    let (w, h) = (def.size.x as i64, def.size.y as i64);
    let xo = def.origin.x as i64;
    let solid = |x: i64, y: i64| alpha_at(px, def, x, y) >= 128;
    let (mut left, mut right) = (-34.0, 32.0);
    let mut lefts = Vec::with_capacity(h as usize);
    let mut rights = Vec::with_capacity(h as usize);
    for yy in 0..h {
        let mut seed = -1;
        let mut d = 0;
        while d <= 40 && seed < 0 {
            if xo - d >= 0 && solid(xo - d, yy) {
                seed = xo - d;
            } else if xo + d < w && solid(xo + d, yy) {
                seed = xo + d;
            }
            d += 1;
        }
        if seed >= 0 {
            let (mut l, mut r) = (seed, seed);
            while l > 0 && solid(l - 1, yy) {
                l -= 1;
            }
            while r < w - 1 && solid(r + 1, yy) {
                r += 1;
            }
            left = (l - xo) as f64;
            right = (r + 1 - xo) as f64;
        }
        lefts.push(left);
        rights.push(right);
    }
    let smooth = vis("squirrel_edge_smooth_rows") as i64;
    let median = |v: &[f64], yy: i64| {
        let a = (yy - smooth).max(0);
        let n = (yy + smooth).min(h - 1) - a + 1;
        let mut t = v[a as usize..(a + n) as usize].to_vec();
        t.sort_by(|x, y| x.partial_cmp(y).unwrap());
        t[(n / 2) as usize]
    };
    TrunkEdges {
        left: (0..h).map(|y| median(&lefts, y)).collect(),
        right: (0..h).map(|y| median(&rights, y)).collect(),
        origin_y: def.origin.y as f64,
        h: h as f64,
    }
}

/// `global.squirrel_frames`: the sheet's 9 strips, each a sprite with its origin at the body
/// point (`squirrel_body_x`, `squirrel_body_y`).
#[derive(Clone, Debug, PartialEq)]
pub struct SquirrelFrames {
    /// `[left, top, w, h]` of each strip in the sheet
    pub parts: Vec<[f64; 4]>,
    pub origin: (f64, f64),
}

// port: squirrel_frames_build
/// `squirrel_frames_build`: the strips copied one by one into a 9-frame sprite (a straight copy:
/// `bm_one, bm_inv_src_alpha` onto a cleared surface), so drawing a frame = drawing its strip.
pub fn squirrel_frames_build(assets: &GameAssets) -> SquirrelFrames {
    let (fw, fh) = assets
        .try_sprite("squirrel")
        .map_or((0.0, 0.0), |d| (d.size.x as f64 / 9.0, d.size.y as f64));
    SquirrelFrames {
        parts: (0..9).map(|i| [i as f64 * fw, 0.0, fw, fh]).collect(),
        origin: (vis("squirrel_body_x"), vis("squirrel_body_y")),
    }
}

/// What `squirrel_art_init` builds.
#[derive(Resource, Clone, Debug)]
pub struct SquirrelArt {
    pub trunks: HashMap<String, TrunkEdges>,
    pub frames: SquirrelFrames,
}

// port: squirrel_art_init
/// `squirrel_art_init`: the trunk edges of tree01–03 and the squirrel frames.
pub fn squirrel_art_init(px: &PagePixels, assets: &GameAssets) -> SquirrelArt {
    let trunks = ["tree01", "tree02", "tree03"]
        .iter()
        .map(|t| (t.to_string(), squirrel_trunk_edges(px, assets, t)))
        .collect();
    SquirrelArt {
        trunks,
        frames: squirrel_frames_build(assets),
    }
}

/// One `leaf_art_read` frame: `[x1, y1, x2, y2, mid_x, mid_y, off_x, off_y]` and its white
/// silhouette (`shape`).
pub struct LeafArtRead {
    pub data: LeafArtFrame,
    pub shape: Image,
}

// port: leaf_art_read
/// `leaf_art_read(spr, frames)`: per strip the alpha ≥ 128 bounds and centroid, the offset of
/// the centroid from the first strip's, and the white silhouette on a `(fw + 4)²` canvas.
pub fn leaf_art_read(
    px: &PagePixels,
    assets: &GameAssets,
    spr: &str,
    frames: i64,
) -> Vec<LeafArtRead> {
    let def = assets.sprite(spr);
    let (w, h) = (def.size.x as i64, def.size.y as i64);
    let fw = w / frames;
    let mut shapes = leaf_shape_images(px, def, frames).into_iter();
    let mut out: Vec<LeafArtRead> = (0..frames)
        .map(|i| {
            let [x1, y1, x2, y2, mx, my] = strip_scan(px, def, i, fw, h);
            LeafArtRead {
                data: [x1, y1, x2, y2, mx, my, 0.0, 0.0],
                shape: shapes.next().expect("one shape per strip"),
            }
        })
        .collect();
    if let Some(first) = out.first().map(|f| f.data) {
        for f in out.iter_mut() {
            f.data[6] = f.data[4] - first[4];
            f.data[7] = f.data[5] - first[5];
        }
    }
    out
}

/// `global.leaf_art` with its silhouettes as images.
pub struct LeafArtImages {
    pub art: LeafArt,
    pub faces: [Vec<Image>; 2],
    pub turns: [Vec<Image>; 2],
}

// port: leaf_art_init
/// `leaf_art_init`: the two rest faces (`leaf`, `leaf_2`) and the two 8-frame turn sheets.
pub fn leaf_art_init(px: &PagePixels, assets: &GameAssets) -> LeafArtImages {
    let read = |spr: &str, n: i64| -> (Vec<LeafArtFrame>, Vec<Image>) {
        leaf_art_read(px, assets, spr, n)
            .into_iter()
            .map(|f| (f.data, f.shape))
            .unzip()
    };
    let (f0, i0) = read("leaf", 1);
    let (f1, i1) = read("leaf_2", 1);
    let (t0, j0) = read("leaf_change", 8);
    let (t1, j1) = read("leaf_change_2", 8);
    LeafArtImages {
        art: LeafArt {
            faces: [f0, f1],
            turns: [t0, t1],
        },
        faces: [i0, i1],
        turns: [j0, j1],
    }
}

/// `global.hedgehog_back` (the app's own scan; logic reads the embedded copy).
#[derive(Resource, Clone, Debug)]
pub struct HedgehogBack(pub Vec<[f64; 2]>);

// ---------------------------------------------------------------------------------------------
// drawing

/// A sheet part drawn `draw_sprite_ext`-style with its origin (`ox`, `oy`, part-local) at
/// (x, y), scaled, rotated counter-clockwise by `rot` degrees about the origin.
#[allow(clippy::too_many_arguments)]
fn part_origin_ext(
    b: &mut QuadBatch,
    a: &GameAssets,
    spr: &str,
    part: [f64; 4],
    ox: f64,
    oy: f64,
    x: f64,
    y: f64,
    xs: f64,
    ys: f64,
    rot: f64,
    col: f64,
    alpha: f64,
) {
    let (lx, ly) = (-ox * xs, -oy * ys);
    let (c, s) = (gml::dcos(rot), gml::dsin(rot));
    b.sprite_general(
        a,
        spr,
        0,
        part,
        x + lx * c + ly * s,
        y - lx * s + ly * c,
        xs,
        ys,
        rot,
        col,
        alpha,
    );
}

// port: hedgehog_draw
/// `hedgehog_draw(x, y, dir, frame)`: frame `frame` of the 8-frame sheet, feet at (x, y),
/// facing `dir`.
pub fn hedgehog_draw(b: &mut QuadBatch, a: &GameAssets, x: f64, y: f64, dir: f64, frame: f64) {
    let Some(def) = a.try_sprite("hedgehog") else {
        return;
    };
    let s = vis("hedgehog_scale");
    let fw = def.size.x as f64 / 8.0;
    let fh = def.size.y as f64;
    let sx = if dir >= 0.0 { -1.0 } else { 1.0 };
    let dx = x.floor() - sx * fw * s * 0.5;
    b.sprite_part_ext(
        a,
        "hedgehog",
        0,
        [frame * fw, 0.0, fw, fh],
        dx,
        y.floor() - fh * s,
        sx * s,
        s,
        C_WHITE,
        1.0,
    );
}

/// `intro_draw` (Task 21's row, unmarked here): the intro hedgehog and the origin mushroom on its
/// back, then falling at the start. `life_draw` calls it first while the intro runs.
pub fn intro_draw(b: &mut QuadBatch, a: &GameAssets, sim: &Sim) {
    let it = &sim.life.intro;
    hedgehog_draw(b, a, it.x, it.y, 1.0, it.frame);
    let sy = if !it.dropped {
        it.y + hedgehog::hedgehog_back_y()
    } else {
        it.shroom_y
    };
    let sx = if !it.dropped {
        it.x
    } else {
        sim.state.state.start_x
    };
    b.sprite(a, "originshroom", 0, sx.floor(), sy.floor());
}

/// `leaf_fg_frame(l)` on the app's frame sprites.
fn fg_frame<'a>(fg: &'a LeafFgArt, l: &Leaf) -> &'a LeafFgSprite {
    if l.turn >= 0.0 {
        return fg
            .turn
            .get(gml::clamp(l.turn, 0.0, 13.0).floor() as usize)
            .unwrap_or(&fg.rest);
    }
    if l.turned {
        return &fg.turned;
    }
    &fg.rest
}

// port: leaf_draw
/// `leaf_draw(l, alpha)`: the leaf's foreground frame at its rounded position, mirrored, turned
/// by its angle.
pub fn leaf_draw(b: &mut QuadBatch, a: &GameAssets, fg: &LeafFgArt, l: &Leaf, alpha: f64) {
    let f = fg_frame(fg, l);
    let m = if l.mirror > 0.0 { -1.0 } else { 1.0 };
    part_origin_ext(
        b,
        a,
        &f.sprite,
        f.part,
        f.origin_x,
        f.origin_y,
        gml::round(l.x),
        gml::round(l.y),
        m,
        1.0,
        l.angle,
        C_WHITE,
        alpha,
    );
}

/// Segment keys of `life_draw`'s painter order: a persistent object's stable sim id, or a
/// shared group.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LifeKey {
    Intro,
    Leaves,
    Mushroom(u64),
    Stinky,
    Hedgehog(u64),
    Boar(u64),
    Flung,
    Clouds,
    Squirrel(u64),
    Crow(u64),
    Drops,
}

/// The entity drawing one [`LifeKey`] segment.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LifeEntity(pub LifeKey);

/// `life_draw`'s output: segments in painter's order.
#[derive(Debug, Default)]
pub struct LifeDraw {
    pub segs: Vec<(LifeKey, QuadBatch)>,
    start_linear: bool,
}

impl LifeDraw {
    fn new(start_linear: bool) -> Self {
        LifeDraw {
            segs: Vec::new(),
            start_linear,
        }
    }

    /// The batch of `key`, continuing the last segment or opening a new one that carries the
    /// blend mode and texture filter on (they are global state in the original).
    fn seg(&mut self, key: LifeKey) -> &mut QuadBatch {
        if self.segs.last().map(|s| s.0) != Some(key) {
            let (blend, linear) = self
                .segs
                .last()
                .map_or((QuadBlend::Alpha, self.start_linear), |s| {
                    (s.1.blend, s.1.linear)
                });
            let mut b = QuadBatch::new();
            b.set_blend(blend);
            b.set_filter(linear);
            self.segs.push((key, b));
        }
        &mut self.segs.last_mut().unwrap().1
    }
}

/// What `life_draw` reads besides the sim.
pub struct LifeCtx<'a> {
    pub assets: &'a GameAssets,
    pub fg: Option<&'a LeafFgArt>,
    pub fx: Option<&'a FxArt>,
    pub clouds: Option<&'a CloudFrame>,
    /// the texture filter `world_draw` left on (`WorldDrawState`)
    pub start_linear: bool,
}

/// The body colour of a mushroom (dim when inactive, except repellers; pinkish while flashing).
fn shroom_col(m: &MushroomSave) -> f64 {
    if m.active || m.kind == 2 {
        16777215.0
    } else {
        7766138.0
    }
}

// port: boars_draw
/// `boars_draw(vx, vw)`: the boars in view (6-frame sheet, feet at their position), then the
/// flung mushrooms spinning about their centres, fading out.
pub fn boars_draw(d: &mut LifeDraw, a: &GameAssets, life: &LifeRt, vx: f64, vw: f64) {
    let scale = vis("boar_scale");
    if let Some(def) = a.try_sprite("boar") {
        let fw = def.size.x as f64 / 6.0;
        let fh = def.size.y as f64;
        let margin = boar_frame_w() + 40.0;
        for bo in &life.boars {
            if bo.x < vx - margin || bo.x > vx + vw + margin {
                continue;
            }
            let face = if bo.dir >= 0.0 { -1.0 } else { 1.0 };
            d.seg(LifeKey::Boar(bo.id)).sprite_part_ext(
                a,
                "boar",
                0,
                [bo.frame * fw, 0.0, fw, fh],
                bo.x.floor() - face * fw * scale * 0.5,
                bo.y.floor() - fh * scale,
                face * scale,
                scale,
                C_WHITE,
                1.0,
            );
        }
    }
    for f in &life.boar_flung {
        let Some(def) = a.try_sprite(f.sprite) else {
            continue;
        };
        let left = types::mushroom_cell_left(f.sprite, f.variant);
        let w = types::mushroom_cell_width(f.sprite).min(def.size.x as f64 - left);
        let h = def.size.y as f64;
        let (c, s) = (gml::dcos(f.angle), gml::dsin(f.angle));
        let (hx, hy) = (-w * 0.5, -h * 0.5);
        let alpha = gml::clamp(f.life / (vis("boar_fling_frames") * 0.6), 0.0, 1.0);
        d.seg(LifeKey::Flung).sprite_general(
            a,
            f.sprite,
            0,
            [left, 0.0, w, h],
            f.x + hx * c + hy * s,
            f.y - hx * s + hy * c,
            1.0,
            1.0,
            f.angle,
            C_WHITE,
            alpha,
        );
    }
}

// port: squirrel_draw
/// `squirrel_draw(sq)`: its frame of the squirrel sprite (origin at the body point), mirrored,
/// turned by its angle.
pub fn squirrel_draw(b: &mut QuadBatch, a: &GameAssets, sq: &Squirrel) {
    let frames = squirrel_frames_build(a);
    let s = vis("squirrel_scale");
    let i = gml::gmod(sq.frame.floor(), 9.0).max(0.0) as usize;
    let Some(&part) = frames.parts.get(i) else {
        return;
    };
    part_origin_ext(
        b,
        a,
        "squirrel",
        part,
        frames.origin.0,
        frames.origin.1,
        sq.x,
        sq.y,
        sq.mirror * s,
        s,
        sq.angle,
        C_WHITE,
        1.0,
    );
}

fn mushroom_by_id(sim: &Sim, id: Option<u64>) -> Option<&MushroomSave> {
    let i = mushroom_index_of(sim, id?);
    (i >= 0).then(|| &sim.state.mushrooms[i as usize])
}

// port: crows_draw
/// `crows_draw(vx, vy, vw)`: per crow in view the mushroom it stole, then the crow with its
/// claws (frame 0's) at its position.
pub fn crows_draw(d: &mut LifeDraw, a: &GameAssets, sim: &Sim, vx: f64, vw: f64) {
    let Some(def) = a.try_sprite("crow") else {
        return;
    };
    let fw = crow::crow_frame_w();
    let fh = def.size.y as f64;
    let sc = vis("crow_scale");
    let claw = crow::crow_claw(0.0);
    let margin = fw * sc + 200.0;
    for c in &sim.life.crows {
        if c.x < vx - margin || c.x > vx + vw + margin {
            continue;
        }
        let b = d.seg(LifeKey::Crow(c.id));
        if let Some(m) = mushroom_by_id(sim, c.mushroom).filter(|m| m.stolen) {
            mushroom_draw(b, a, sim, m, m.x.floor(), m.y.floor(), C_WHITE, 1.0, 0.0, 1.0);
        }
        let xs = -c.dir * sc;
        b.sprite_part_ext(
            a,
            "crow",
            0,
            [c.frame * fw, 0.0, fw, fh],
            (c.x - claw.0 * xs).floor(),
            (c.y - claw.1 * sc).floor(),
            xs,
            sc,
            C_WHITE,
            1.0,
        );
    }
}

/// A stolen mushroom in a squirrel's mouth (`life_draw`): its type's sheet cell with the
/// squirrel's hold scale and angle about its origin.
fn held_mushroom(b: &mut QuadBatch, a: &GameAssets, m: &MushroomSave) {
    let spr = types::sprite_of_code(m.kind);
    let Some(def) = a.try_sprite(spr) else {
        return;
    };
    let left = types::mushroom_cell_left(spr, types::mushroom_variant(m));
    let w = types::mushroom_cell_width(spr).min(def.size.x as f64 - left);
    let ox = -(def.origin.x as f64) * m.hold_xscale;
    let oy = -(def.origin.y as f64) * m.hold_yscale;
    let (c, sn) = (gml::dcos(m.hold_angle), gml::dsin(m.hold_angle));
    b.sprite_general(
        a,
        spr,
        0,
        [left, 0.0, w, def.size.y as f64],
        m.x.floor() + ox * c + oy * sn,
        m.y.floor() - ox * sn + oy * c,
        m.hold_xscale,
        m.hold_yscale,
        m.hold_angle,
        C_WHITE,
        1.0,
    );
}

// port: life_draw
/// `life_draw`: the intro, leaves, mushrooms (+ predator flies), the stench, hedgehogs (+ what
/// they carry), boars, the clouds, squirrels (+ stolen mushrooms), crows, drops — in that order,
/// culled to the view. (`squirrel_art_init` / `hedgehog_back_init` run at startup;
/// `hover_bar_step_draw` is the HUD's, Task 17.)
pub fn life_draw(sim: &Sim, ctx: &LifeCtx, v: &ViewRect) -> LifeDraw {
    let a = ctx.assets;
    let mut d = LifeDraw::new(ctx.start_linear);
    if sim.intro_active {
        intro_draw(d.seg(LifeKey::Intro), a, sim);
    }
    let (cx, cy, cw, ch) = (v.x, v.y, v.w, v.h);
    if let Some(fg) = ctx.fg {
        for l in &sim.life.leaves {
            if l.x < cx - 64.0 || l.x > cx + cw + 64.0 {
                continue;
            }
            let mut al = if !l.landed || l.absorbing {
                1.0
            } else {
                gml::clamp(l.life / 60.0, 0.15, 1.0)
            };
            if l.absorbing {
                let total = l.total.unwrap_or(0.0).max(1.0);
                al *= gml::clamp(l.timer / total / 0.3, 0.0, 1.0);
            }
            leaf_draw(d.seg(LifeKey::Leaves), a, fg, l, al);
        }
    }
    for m in &sim.state.mushrooms {
        if m.x < cx - 160.0 || m.x > cx + cw + 160.0 {
            continue;
        }
        if m.y < cy - 200.0 || m.y > cy + ch + 200.0 || m.stolen || m.carried_by != -1 {
            continue;
        }
        let mut col = shroom_col(m);
        if m.flash > 0.0 {
            col = 11067135.0;
        }
        let b = d.seg(LifeKey::Mushroom(m.live.id));
        mushroom_draw(b, a, sim, m, m.x.floor(), m.y.floor(), col, 1.0, 0.0, 1.0);
        if m.kind == 1 && m.active {
            for f in &m.flies {
                let fx = m.x + gml::lengthdir_x(f.radius, f.angle);
                let fy = m.y - 62.0
                    + gml::lengthdir_y(f.radius * 0.45, f.angle)
                    + gml::lengthdir_y(3.0, f.bob);
                b.sprite(a, "fly", 0, fx.floor(), fy.floor());
            }
        }
    }
    if let Some(fx) = ctx.fx {
        for m in sim.state.mushrooms.iter().filter(|m| m.kind == 5) {
            stinky_draw(d.seg(LifeKey::Stinky), fx, m, v);
        }
    }
    let margin = hedgehog::hedgehog_border_margin();
    for an in &sim.life.animals {
        if an.x < cx - margin || an.x > cx + cw + margin {
            continue;
        }
        hedgehog_with_load(d.seg(LifeKey::Hedgehog(an.id)), a, sim, an);
    }
    boars_draw(&mut d, a, &sim.life, cx, cw);
    if let Some(cf) = ctx.clouds {
        clouds_composite(d.seg(LifeKey::Clouds), cf);
    }
    let sq_margin = squirrel::squirrel_frame_w() + 80.0;
    for s in &sim.life.squirrels {
        if s.x < cx - sq_margin || s.x > cx + cw + sq_margin {
            continue;
        }
        let b = d.seg(LifeKey::Squirrel(s.id));
        if let Some(m) = mushroom_by_id(sim, s.mushroom).filter(|m| m.stolen) {
            held_mushroom(b, a, m);
        }
        squirrel_draw(b, a, s);
    }
    crows_draw(&mut d, a, sim, cx, cw);
    for dr in &sim.life.drops {
        if dr.x < cx - 64.0 || dr.x > cx + cw + 64.0 {
            continue;
        }
        d.seg(LifeKey::Drops)
            .sprite(a, "waterdrop", 0, dr.x.floor(), dr.y.floor());
    }
    // a key that drew nothing (sprite missing, nothing in view) gets no entity; keep the state
    // the dropped segment carried in the next one
    let mut kept: Vec<(LifeKey, QuadBatch)> = Vec::with_capacity(d.segs.len());
    for (k, b) in d.segs.drain(..) {
        if b.is_empty() {
            if let Some(last) = kept.last_mut() {
                last.1.set_blend(b.blend);
                last.1.set_filter(b.linear);
            }
            continue;
        }
        kept.push((k, b));
    }
    d.segs = kept;
    d
}

/// A hedgehog and the mushroom on its back (carry angle and `hedgehog_carry_scale`).
fn hedgehog_with_load(b: &mut QuadBatch, a: &GameAssets, sim: &Sim, an: &Hedgehog) {
    hedgehog_draw(b, a, an.x, an.y, an.dir, an.frame);
    if an.carrying == -1 || an.carrying as usize >= sim.state.mushrooms.len() || an.carrying < 0 {
        return;
    }
    let c = &sim.state.mushrooms[an.carrying as usize];
    // c[$ "carry_angle"] ?? 0
    let ca = c
        .extra
        .get("carry_angle")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    mushroom_draw(
        b,
        a,
        sim,
        c,
        c.x.floor(),
        c.y.floor(),
        shroom_col(c),
        1.0,
        ca,
        vis("hedgehog_carry_scale"),
    );
}

// port: worms_draw
/// `worms_draw(vx, vy, vw, vh)`: the fading trail dots, then each worm as one square per body
/// pixel, thinner at the ends, with its band, whitening and wiggling while caught.
pub fn worms_draw(b: &mut QuadBatch, sim: &Sim, vx: f64, vy: f64, vw: f64, vh: f64) {
    let col = gml::make_colour_rgb(
        vis("worm_colour_r"),
        vis("worm_colour_g"),
        vis("worm_colour_b"),
    );
    let band = gml::merge_colour(col, 8812740.0, 0.55);
    let hi = 14873582.0;
    let th = vis("worm_thickness_px");
    let struggle_frames = vis("worm_struggle_frames").max(1.0);
    let ts = (th - 1.0).max(1.0);
    let half = (ts / 2.0).trunc();
    let trail_frames = vis("worm_trail_frames").max(1.0);
    let trail_alpha = vis("worm_trail_alpha");
    let trail_col = gml::make_colour_rgb(
        vis("worm_trail_colour_r"),
        vis("worm_trail_colour_g"),
        vis("worm_trail_colour_b"),
    );
    let life = &sim.life;
    for t in &life.worm_trail {
        if t.x < vx - 8.0 || t.x > vx + vw + 8.0 || t.y < vy - 8.0 || t.y > vy + vh + 8.0 {
            continue;
        }
        let (x1, y1) = (t.x - half, t.y - half);
        b.rect(
            x1,
            y1,
            x1 + ts - 1.0,
            y1 + ts - 1.0,
            trail_col,
            trail_alpha * t.life / trail_frames,
        );
    }
    let now = sim.now_ms();
    for w in &life.worms {
        let body = &w.body;
        let Some(h) = body.first() else { continue };
        if h.x < vx - 64.0 || h.x > vx + vw + 64.0 || h.y < vy - 64.0 || h.y > vy + vh + 64.0 {
            continue;
        }
        let n = body.len() - 1;
        let (mut a, mut amp, mut wig) = (1.0, 0.0, 0.0);
        if w.caught {
            let p = 1.0 - w.timer / w.total.max(1.0);
            amp = p * 0.6;
            a = gml::clamp(w.timer / w.total.max(1.0) / 0.3, 0.0, 1.0);
            wig = if w.total - w.timer < w.touch_frames.unwrap_or(0.0) {
                1.5 * w.struggle / struggle_frames
            } else {
                0.0
            };
        }
        let body_col = gml::merge_colour(col, hi, amp);
        let band_col = gml::merge_colour(band, hi, amp);
        let total = (0..n)
            .map(|s| gml::point_distance(body[s].x, body[s].y, body[s + 1].x, body[s + 1].y))
            .sum::<f64>()
            .max(1.0);
        let mut acc = 0.0;
        for s in 0..n {
            let (p0, p1) = (body[s], body[s + 1]);
            let d = gml::point_distance(p0.x, p0.y, p1.x, p1.y);
            let dir = gml::point_direction(p0.x, p0.y, p1.x, p1.y);
            let mut k = 0.0;
            while k < d {
                let u = (acc + k) / total;
                let prof = gml::power((u * 2.0 - 1.0).abs(), 3.0);
                let sz = gml::round(gml::lerp(th, th - 1.5, prof)).max(2.0);
                let off = wig * (now * 0.025 + u * 9.0).sin();
                let px = (gml::lerp(p0.x, p1.x, k / d) + gml::lengthdir_x(off, dir + 90.0)).floor();
                let py = (gml::lerp(p0.y, p1.y, k / d) + gml::lengthdir_y(off, dir + 90.0)).floor();
                let hs = (sz / 2.0).trunc();
                let c = if u > 0.22 && u < 0.34 {
                    band_col
                } else {
                    body_col
                };
                b.rect(px - hs, py - hs, px - hs + sz - 1.0, py - hs + sz - 1.0, c, a);
                k += 1.0;
            }
            acc += d;
        }
    }
}

// port: leaf_tendril_cells_draw
/// `leaf_tendril_cells_draw(cells, count, col)`: the first `count` cells as squares (two
/// triangles each, the original's vertex order) appended to `out`. The colour is the strand
/// mesh's (`col` is always the strand colour 0xe2f3ee, or white into the glow seed, which only
/// reads alpha).
pub fn leaf_tendril_cells_draw(out: &mut Vec<V2>, cells: &[Cell], count: usize) {
    for c in cells.iter().take(count) {
        let h = (c.w / 2.0).trunc();
        let (x0, y0) = (c.x - h, c.y - h);
        let (x1, y1) = (x0 + c.w, y0 + c.w);
        let v = |x: f64, y: f64| [x as f32, y as f32];
        out.extend([
            v(x0, y0),
            v(x1, y0),
            v(x0, y1),
            v(x1, y0),
            v(x1, y1),
            v(x0, y1),
        ]);
    }
}

fn shown(n: usize, p: f64) -> usize {
    n.min((gml::clamp(p, 0.0, 1.0) * n as f64).ceil() as usize)
}

// port: leaf_tendrils_draw
/// `leaf_tendrils_draw(col)`: the growing tendrils of absorbing leaves and caught worms, then the
/// retracting ones.
pub fn leaf_tendrils_draw(out: &mut Vec<V2>, life: &LifeRt) {
    for l in life.leaves.iter().filter(|l| l.absorbing) {
        let Some(t) = &l.tendril else { continue };
        let gf = l
            .grow_frames
            .unwrap_or_else(|| vis("leaf_tendril_grow_frames").max(1.0));
        let p = (l.total.unwrap_or(0.0) - l.timer) / gf;
        leaf_tendril_cells_draw(out, t, shown(t.len(), p));
    }
    for w in life.worms.iter().filter(|w| w.caught) {
        let Some(t) = &w.tendril else { continue };
        let p = (w.total - w.timer) / w.grow_frames.unwrap_or(0.0).max(1.0);
        leaf_tendril_cells_draw(out, t, shown(t.len(), p));
    }
    for t in &life.leaf_tendrils_out {
        let n = t.cells.len();
        let k = n.min((t.timer / t.total.max(1.0) * n as f64).ceil().max(0.0) as usize);
        leaf_tendril_cells_draw(out, &t.cells, k);
    }
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// One entity (and its pooled meshes) per `life_draw` segment key; a key drawn twice in a frame
/// gets a second slot.
#[derive(Resource, Default)]
pub struct LifePools {
    map: HashMap<(LifeKey, u8), (Entity, BatchPool)>,
}

impl LifePools {
    /// Draw `segs` from z = `base` (each segment right above the previous one's runs, up to
    /// `max`); despawn the keys that are gone.
    fn sync(
        &mut self,
        t: &mut BatchTarget,
        segs: &[(LifeKey, QuadBatch)],
        base: f32,
        max: f32,
        layers: &RenderLayers,
    ) {
        let mut seen: HashSet<(LifeKey, u8)> = HashSet::new();
        let mut z = base;
        for (key, batch) in segs {
            let mut slot = (*key, 0u8);
            while seen.contains(&slot) {
                slot.1 += 1;
            }
            seen.insert(slot);
            let entry = self.map.entry(slot).or_insert_with(|| {
                let e = t.commands.spawn((LifeEntity(*key), Name::new("life"))).id();
                (e, BatchPool::default())
            });
            entry.1.sync(t, batch, z.min(max), Z_STEP, layers);
            z += batch.runs.len().max(1) as f32 * Z_STEP;
        }
        self.map.retain(|k, (e, pool)| {
            if seen.contains(k) {
                return true;
            }
            pool.despawn(t.commands);
            t.commands.entity(*e).despawn();
            false
        });
    }
}

// port: gml_Object_obj_life_Draw_0
/// obj_life Draw_0: `life_draw` every frame, into the life layer's keyed entities.
#[allow(clippy::too_many_arguments)]
pub fn life_draw_system(
    mut commands: Commands,
    sim: Res<SimRes>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    fg: Option<Res<LeafFgArt>>,
    fx: Option<Res<FxArt>>,
    clouds: Option<Res<CloudFrame>>,
    world: Option<Res<crate::render::world::WorldDrawState>>,
    mut pools: ResMut<LifePools>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmQuad>>,
    white: Res<WhiteTexture>,
) {
    let ctx = LifeCtx {
        assets: &assets,
        fg: fg.as_deref(),
        fx: fx.as_deref(),
        clouds: clouds.as_deref(),
        start_linear: world.map_or(true, |w| w.end_linear),
    };
    let d = life_draw(&sim.0, &ctx, &cams.render_view());
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        white: &white.0,
    };
    pools.sync(
        &mut t,
        &d.segs,
        Z_LIFE,
        Z_TURF - LAYER_SPAN,
        &RenderLayers::layer(0),
    );
}

#[derive(Resource, Default)]
struct WormPool(BatchPool);

/// `worms_draw` from the forest's Draw_0 (between `world_draw` and the roots: `Z_WORMS`).
fn worms_draw_system(
    mut commands: Commands,
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    mut pool: ResMut<WormPool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmQuad>>,
    white: Res<WhiteTexture>,
) {
    let v = cams.render_view();
    let mut b = QuadBatch::new();
    worms_draw(&mut b, &sim.0, v.x, v.y, v.w, v.h);
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        white: &white.0,
    };
    pool.0
        .sync(&mut t, &b, Z_WORMS, Z_STEP, &RenderLayers::layer(0));
}

/// The leaf / worm tendril mesh (drawn with the hyphae: world camera and glow seed).
#[derive(Resource)]
struct TendrilMesh {
    mesh: Handle<Mesh>,
    verts: Vec<V2>,
}

fn spawn_tendrils(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmBlend>>,
) {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    fill_strand_mesh(&mut m, &[]);
    let mesh = meshes.add(m);
    commands.spawn((
        Name::new("leaf tendrils"),
        Mesh2d(mesh.clone()),
        MeshMaterial2d(materials.add(GmBlend::new(GmBlendMode::Opaque))),
        // after the strands (leaf_tendrils_draw ends myc_draw_strands), below the knots
        Transform::from_xyz(0.0, 0.0, Z_MYC_STRANDS + 0.5),
        strand_layers(),
        NoFrustumCulling,
    ));
    commands.insert_resource(TendrilMesh {
        mesh,
        verts: Vec::new(),
    });
}

fn tendrils_system(
    sim: Res<SimRes>,
    tm: Option<ResMut<TendrilMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(mut tm) = tm else { return };
    let mut v = Vec::new();
    leaf_tendrils_draw(&mut v, &sim.0.life);
    if v != tm.verts {
        if let Some(mut m) = meshes.get_mut(&tm.mesh) {
            fill_strand_mesh(&mut m, &v);
        }
        tm.verts = v;
    }
}

/// Startup pixel inits: `hedgehog_back_init`, `squirrel_art_init` (logic reads the embedded
/// copies; the leaf art is built with the leaf sheets in `leaf_art::build_leaf_art`).
fn build_life_art(mut commands: Commands, assets: Res<GameAssets>) {
    let Ok(px) = PagePixels::load(
        &assets.root,
        &assets,
        &["hedgehog", "tree01", "tree02", "tree03"],
    ) else {
        return;
    };
    commands.insert_resource(HedgehogBack(hedgehog_back_init(&px, &assets)));
    commands.insert_resource(squirrel_art_init(&px, &assets));
}

pub struct LifeRenderPlugin;

impl Plugin for LifeRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BatchPlugin>() {
            app.add_plugins(BatchPlugin);
        }
        if !app.is_plugin_added::<crate::render::mushrooms::MushroomFxPlugin>() {
            app.add_plugins(crate::render::mushrooms::MushroomFxPlugin);
        }
        if !app.is_plugin_added::<crate::render::blend::BlendPlugin>() {
            app.add_plugins(crate::render::blend::BlendPlugin);
        }
        app.init_resource::<LifePools>()
            .init_resource::<WormPool>()
            .add_systems(Startup, (build_life_art, spawn_tendrils))
            .add_systems(
                Update,
                (
                    worms_draw_system,
                    tendrils_system,
                    life_draw_system
                        .after(crate::render::world::WorldDrawSet)
                        .after(crate::render::mushrooms::CloudsSet),
                ),
            );
    }
}
