//! Underground world drawing (Task 13): `world_draw` (forest Draw_0) and `world_draw_turf`
//! (forest Draw_73), with soil rows and seams, dirt, bones, caves and their wrap outlines, tiles,
//! stone deposits, lakes, loose stones, the ocean / rock bottom, foods and their glow; plus the
//! cave popups and perk icons (HUD Draw_64) and the mixed inits (R7) that read sprite pixels.
//!
//! Every `world_draw*` port records into a [`QuadBatch`] in the original painter's order; the
//! systems turn the batches into pooled meshes ([`BatchPool`]):
//!
//! - `back` (soil, seams, dirt, bones, caves) at `Z_WORLD`;
//! - the tiles (hard soil, stone, water fill) as one mesh per chunk, rebuilt only when the chunk
//!   is in `WorldRt.dirty_chunks` (cleared here, R9a) or comes into view, at `Z_WORLD_TILES`;
//! - `front` (deposits, lakes, stones, ocean, food glow, foods, inner glow) at `Z_WORLD_FRONT`;
//! - turf at `Z_TURF`.
//!
//! The food glow renders offscreen like the original surfaces: foods into the seed image, then
//! the two `sh_glow_blur` passes (three images, three cameras ordered before the world camera, on
//! their own render layers), and the result is drawn additively into `front`.
//!
//! `gm.current_time` is `Sim::now_ms` (frame clock), the same clock cave flash / wrap / popup
//! births use.

#![allow(
    clippy::neg_cmp_op_on_partial_ord,
    reason = "`not (a < b)` mirrors the Lua conditions exactly, NaN included"
)]

use crate::assets::GameAssets;
use crate::camera::{Cameras, COMPOSITING};
use crate::render::batch::{
    BatchPlugin, BatchPool, BatchTarget, GmQuad, QuadBatch, QuadBlend, WhiteTexture, C_WHITE,
};
use crate::render::layers::{Z_STEP, Z_TURF, Z_WORLD};
use crate::render::leaf_art::PagePixels;
use crate::sim_bridge::SimRes;
use crate::ui::gm_draw::{Gm, GuiDraw, UiFont, FA_CENTER, FA_LEFT, FA_TOP};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use myco_core::balance::{vis, wld};
use myco_core::gml;
use myco_core::progress::goals::{goal_fit, TextMeasure};
use myco_core::sim::Sim;
use myco_core::state::{GameState, WorldChunkSave};
use myco_core::world::bones::{bone_kinds, world_big_bones};
use myco_core::world::caves::{cave_def, cave_defs, cave_mask, cave_taken, CaveRt};
use myco_core::world::food::food_sprite;
use myco_core::world::lakes::{
    lake_cut_row, lake_flare_alpha, lake_flare_life_ms, lake_flare_spot, lake_init_flares,
    lakes_ready, LakeFlare,
};
use myco_core::world::perks::perk_has;
use myco_core::world::relics::{relic_taken, world_relics};
use myco_core::world::soil::{dirt_sprite, ocean_row, soil_band, soil_edge_sprite, soil_sprite};
use myco_core::world::stones::{stone_art, stone_pick_tile, stone_role_tiles};
use myco_core::world::{maze, WorldRt};
use std::collections::HashMap;

/// z of the chunk tile meshes (after caves, before deposits, as world_draw's chunk loop).
pub const Z_WORLD_TILES: f32 = Z_WORLD + 100.0;
/// z base of the part of world_draw after the tiles.
pub const Z_WORLD_FRONT: f32 = Z_WORLD + 110.0;
/// Render layers of the food-glow passes (seed, across, down).
pub const GLOW_LAYERS: [usize; 3] = [5, 6, 7];
/// Draw-list keys in [`GuiDraw`]: perk icons with the HUD, popups after the HUD text.
pub const PERK_ICONS_DRAW_ORDER: i32 = crate::ui::draw_order::CAVES;
pub const CAVE_POPUPS_DRAW_ORDER: i32 = 310;

fn div(a: f64, b: f64) -> f64 {
    (a / b).trunc()
}

/// `merge_colour(c1, c2, amount)` (LÖVE: per channel, rounded half up).
pub fn merge_colour(c1: f64, c2: f64, amount: f64) -> f64 {
    let (a, b) = (c1.max(0.0) as u32, c2.max(0.0) as u32);
    let mut out = 0u32;
    for sh in [0, 8, 16] {
        let (x, y) = (((a >> sh) & 255) as f64, ((b >> sh) & 255) as f64);
        let m = (x + (y - x) * amount + 0.5).floor().clamp(0.0, 255.0) as u32;
        out |= m << sh;
    }
    out as f64
}

fn sprite_size(assets: &GameAssets, name: &str) -> (f64, f64) {
    assets
        .try_sprite(name)
        .map(|d| (d.size.x as f64, d.size.y as f64))
        .unwrap_or((0.0, 0.0))
}

// ---------------------------------------------------------------------------------------------
// pixel-derived art (mixed inits, R7)

/// What `stones_init` builds: the stone sheet and its greyed copy as untrimmed images (both
/// `sprite_create_from_surface` sources), and the kinds measured from the pixels.
#[derive(Resource, Clone)]
pub struct StonesArt {
    pub plain: Handle<Image>,
    pub grey: Handle<Image>,
    pub kinds: Vec<[f64; 6]>,
    pub reach: f64,
}

/// Untrimmed RGBA pixels of a sprite frame (what drawing it onto a cleared surface gives).
pub fn sprite_pixels(px: &PagePixels, assets: &GameAssets, name: &str) -> (usize, usize, Vec<u8>) {
    let def = assets.sprite(name);
    let (w, h) = (def.size.x as usize, def.size.y as usize);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let p = px.sprite_rgba(def, 0, x as i64, y as i64);
            out[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&p);
        }
    }
    (w, h, out)
}

// port: stones_init
/// The stone kinds from the `stones` sheet: columns split at empty pixel columns
/// (`stones_slice_sheet`), each with its alpha ≥ 128 bounds, origin, radius and body; and the
/// greyed copy (`stone_grey_amount`) the deeper stones use.
pub fn stones_init(px: &PagePixels, assets: &GameAssets) -> (Vec<[f64; 6]>, f64, Vec<u8>, Vec<u8>) {
    let (sw, sh, buf) = sprite_pixels(px, assets, "stones");
    let alpha = |x: usize, y: usize| buf[(y * sw + x) * 4 + 3];
    let slices = myco_core::world::stones::stones_slice_sheet(alpha, sw, sh);
    let mut kinds = Vec::new();
    let mut reach: f64 = 0.0;
    for s in slices.iter().take(7) {
        let (left, width) = (s.left as usize, s.width as usize);
        let (mut x0, mut y0, mut x1, mut y1): (f64, f64, f64, f64) =
            (width as f64, sh as f64, -1.0, -1.0);
        for yy in 0..sh {
            for xx in 0..width {
                if alpha(left + xx, yy) >= 128 {
                    let (fx, fy) = (xx as f64, yy as f64);
                    x0 = x0.min(fx);
                    y0 = y0.min(fy);
                    x1 = x1.max(fx);
                    y1 = y1.max(fy);
                }
            }
        }
        if x1 < x0 {
            (x0, y0, x1, y1) = (0.0, 0.0, width as f64 - 1.0, sh as f64 - 1.0);
        }
        let ox = gml::round((x0 + x1 + 1.0) * 0.5);
        let oy = gml::round((y0 + y1 + 1.0) * 0.5);
        let ax = (ox - x0).max(x1 + 1.0 - ox);
        let ay = (oy - y0).max(y1 + 1.0 - oy);
        let radius = gml::point_distance(0.0, 0.0, ax, ay);
        reach = reach.max(radius);
        kinds.push([s.left, s.width, ox, oy, radius, (ax + ay) * 0.5]);
    }
    let grey = vis("stone_grey_amount");
    let mut g = buf.clone();
    for p in 0..sw * sh {
        let o = p * 4;
        let (r, gg, b) = (g[o] as f64, g[o + 1] as f64, g[o + 2] as f64);
        let l = 0.3 * r + 0.59 * gg + 0.11 * b;
        g[o] = gml::round(gml::lerp(r, l, grey)).clamp(0.0, 255.0) as u8;
        g[o + 1] = gml::round(gml::lerp(gg, l, grey)).clamp(0.0, 255.0) as u8;
        g[o + 2] = gml::round(gml::lerp(b, l * 1.04, grey)).clamp(0.0, 255.0) as u8;
    }
    (kinds, reach, buf, g)
}

// port: stone_tiles_init
/// Solid spans (alpha ≥ 128) of each 48×48 cell of the 7×7 `stone_tile` atlas (cells 64 apart).
pub fn stone_tiles_init(px: &PagePixels, assets: &GameAssets) -> Vec<Vec<Vec<f64>>> {
    let (tw, th, buf) = sprite_pixels(px, assets, "stone_tile");
    let mut tiles = Vec::new();
    for ty in 0..7 {
        for tx in 0..7 {
            let (x0, y0) = (tx * 64, ty * 64);
            let mut rows = Vec::new();
            for yy in 0..48 {
                let mut row = Vec::new();
                let mut start: i64 = -1;
                let py = y0 + yy;
                for xx in 0..=48usize {
                    let solid = xx < 48
                        && x0 + xx < tw
                        && py < th
                        && buf[(py * tw + x0 + xx) * 4 + 3] >= 128;
                    if solid && start < 0 {
                        start = xx as i64;
                    } else if !solid && start >= 0 {
                        row.push(start as f64);
                        row.push(xx as f64 - 1.0);
                        start = -1;
                    }
                }
                rows.push(row);
            }
            tiles.push(rows);
        }
    }
    tiles
}

/// A lake kind as `lakes_init` measures it: left, width, height, top, bottom, spans per row.
#[derive(Clone, Debug, PartialEq)]
pub struct LakeKindPx {
    pub left: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub bottom: f64,
    pub spans: Vec<Vec<f64>>,
}

// port: lakes_init
/// The three lake masks from the `water` sheet (alpha ≥ 128 spans per row), falling back to the
/// layout's body rectangle for an empty mask.
pub fn lakes_init(px: &PagePixels, assets: &GameAssets) -> Vec<LakeKindPx> {
    let (w, h, buf) = sprite_pixels(px, assets, "water");
    let layout = myco_core::world::lakes::lake_layout();
    let mut kinds = Vec::new();
    for l in layout.iter() {
        let left = l.left as usize;
        let lw = (l.width as i64).min(w as i64 - left as i64).max(0) as usize;
        let mut rows = vec![Vec::new(); h];
        let (mut top, mut bottom) = (h as f64, -1.0f64);
        for (y, row) in rows.iter_mut().enumerate() {
            let mut start: i64 = -1;
            for x in 0..=lw {
                let solid = x < lw && buf[(y * w + left + x) * 4 + 3] >= 128;
                if solid && start < 0 {
                    start = x as i64;
                } else if !solid && start >= 0 {
                    row.push(start as f64);
                    row.push(x as f64 - 1.0);
                    start = -1;
                }
            }
            if !row.is_empty() {
                top = top.min(y as f64);
                bottom = bottom.max(y as f64);
            }
        }
        if bottom < top {
            top = l.body_y;
            bottom = (h as f64 - 1.0).min(l.body_y + l.body_h - 1.0);
            let mut y = top;
            while y <= bottom {
                rows[y as usize] = vec![l.body_x, l.body_x + l.body_w - 1.0];
                y += 1.0;
            }
        }
        kinds.push(LakeKindPx {
            left: l.left,
            width: lw as f64,
            height: h as f64,
            top,
            bottom,
            spans: rows,
        });
    }
    kinds
}

// port: caves_mask_init
/// The alpha masks of the five cave parts and two relics (`draw_sprite_part` onto a cleared
/// surface with `bm_one, bm_zero`), each as `(id, w, h, alpha)`; myco_core embeds the same
/// masks (R7) and builds the outlines from them (`cave_outline_build`).
pub fn caves_mask_init(
    px: &PagePixels,
    assets: &GameAssets,
) -> Vec<(String, usize, usize, Vec<u8>)> {
    let mut out = Vec::new();
    for c in cave_defs() {
        let def = assets.sprite(c.sprite);
        let h = def.size.y as usize;
        let w = c.w as usize;
        let mut a = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                a[y * w + x] = px.sprite_alpha(def, 0, (c.left as usize + x) as i64, y as i64);
            }
        }
        out.push((c.kind.id().to_string(), w, h, a));
    }
    for (id, spr) in [("t_rex", "spr_skeleton_T_rex"), ("ufo", "spr_UFO")] {
        let def = assets.sprite(spr);
        let (w, h) = (def.size.x as usize, def.size.y as usize);
        let mut a = vec![0u8; w * h];
        for y in 0..h {
            for x in 0..w {
                a[y * w + x] = px.sprite_alpha(def, 0, x as i64, y as i64);
            }
        }
        out.push((id.to_string(), w, h, a));
    }
    out
}

// ---------------------------------------------------------------------------------------------
// world_draw and helpers

/// Lake sparkles (`lk.flares`): runtime only, keyed by chunk, lake index and lake seed. Like
/// the original's `lk.flares = undefined`, they start over after every save and in a replaced
/// game (`session`: the `Sim::session` they belong to).
#[derive(Resource, Default)]
pub struct LakeFlares {
    pub map: HashMap<(i64, usize, i64), Vec<LakeFlare>>,
    pub session: u64,
}

/// The sparkle half of `world_draw_lakes` that changes state (`lake_init_flares` for a lake seen
/// without flares, the respawns that advance the saved `flare_seq`), for the chunks and lakes the
/// view shows. Run once after every fixed step for the render view
/// ([`crate::render::draw_generate_system`]), as the original's draw runs once per step.
#[allow(clippy::too_many_arguments)]
pub fn world_lakes_step(
    w: &mut WorldRt,
    flares: &mut LakeFlares,
    now: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    if !lakes_ready() || vy + vh < w.params.surface_y {
        return;
    }
    let life = lake_flare_life_ms();
    let seed = w.params.seed;
    let cw = w.params.chunk_w as f64;
    for cx in (vx / cw).floor() as i64..=((vx + vw) / cw).floor() as i64 {
        let Some(ch) = w.chunk_mut(cx) else {
            continue;
        };
        for (li, lk) in ch.lakes.iter_mut().enumerate() {
            let kd = myco_core::world::lakes::lake_kind(lk.kind);
            if lk.x + kd.width < vx || lk.x > vx + vw || lk.y + kd.height < vy || lk.y > vy + vh {
                continue;
            }
            if lk.amount <= 0.0 {
                continue;
            }
            let cut = lake_cut_row(lk, kd);
            let fl = flares
                .map
                .entry((cx, li, lk.seed))
                .or_insert_with(|| lake_init_flares(seed, lk, kd, cut, now));
            for j in 0..fl.len() {
                let f = fl[j];
                let respawn = if f.placed {
                    now - f.born >= life
                } else {
                    now - f.born >= 500.0
                };
                if respawn {
                    let spot = lake_flare_spot(seed, lk, fl, kd, j, cut);
                    let f = &mut fl[j];
                    f.born = now;
                    f.placed = spot.is_some();
                    if let Some((x, y)) = spot {
                        f.x = x;
                        f.y = y;
                    }
                }
            }
        }
    }
}

/// What drawing needs besides the simulation.
pub struct DrawCtx<'a> {
    pub assets: &'a GameAssets,
    pub stones: Option<&'a StonesArt>,
    /// the blurred glow image (`global.food_glow_seed` after the passes)
    pub glow_image: Option<&'a Handle<Image>>,
    pub now: f64,
}

/// The food glow request of one frame (`world_draw_food_glow`'s surface work).
#[derive(Clone, Debug)]
pub struct GlowPass {
    pub sw: u32,
    pub sh: u32,
    /// foods drawn into the seed surface (surface coordinates)
    pub seed: QuadBatch,
    /// blur step in texels
    pub rad: f64,
}

/// The inner-glow struct `world_draw_food_glow` returns (`___struct___365`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlowInfo {
    pub x: f64,
    pub y: f64,
    pub step: f64,
    pub light: f64,
    pub sw: f64,
    pub sh: f64,
}

/// One frame of `world_draw`.
#[derive(Clone, Debug, Default)]
pub struct WorldFrame {
    /// soil, seams, dirt, bones, caves
    pub back: QuadBatch,
    /// deposits, lakes, stones, ocean, glow, foods
    pub front: QuadBatch,
    pub glow: Option<GlowPass>,
    /// chunks the view covers (tiles are drawn from their chunk meshes)
    pub chunks: Vec<i64>,
}

/// The chunk work `world_draw` does for a view, in its order: the chunks `world_draw_bones`
/// looks at (600 px margin to the left), then the view's chunks generated, reached down to the
/// view's bottom row and their late clusters painted. [`crate::render::draw_generate_system`]
/// runs it after every fixed step for the render view (GameMaker draws once after each step), so
/// the saved world only changes inside steps; the frame's drawing then finds the chunks ready.
pub fn world_draw_generate(sim: &mut Sim, vx: f64, vy: f64, vw: f64, vh: f64) {
    let (surface_y, t, chunk_w, rows) = {
        let p = &sim.world.params;
        (p.surface_y, p.tile as f64, p.chunk_w as f64, p.rows)
    };
    if vy + vh < surface_y {
        return;
    }
    for c in ((vx - 600.0) / chunk_w).floor() as i64..=((vx + vw) / chunk_w).floor() as i64 {
        sim.world.get_chunk(c);
    }
    let r1 = gml::clamp(((vy + vh - surface_y) / t).floor(), 0.0, (rows - 1) as f64);
    for ci in (vx / chunk_w).floor() as i64..=((vx + vw) / chunk_w).floor() as i64 {
        sim.world.get_chunk(ci);
        sim.world.chunk_reach(ci, r1 as i64);
        maze::world_paint_clusters_late(&mut sim.world, ci);
    }
}

// port: world_draw
/// `world_draw(vx, vy, vw, vh)`. The chunk generation and reach it does in the original is
/// [`world_draw_generate`], run after each fixed step; chunks missing here (a view that changed
/// since) are drawn from the next frame.
pub fn world_draw(
    sim: &mut Sim,
    ctx: &DrawCtx,
    flares: &mut LakeFlares,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) -> WorldFrame {
    let mut out = WorldFrame::default();
    let a = ctx.assets;
    // stones_init / lakes_init / stone_tiles_init: done at startup (`build_world_art`)
    let (surface_y, t, chunk_w, rows) = {
        let p = &sim.world.params;
        (p.surface_y, p.tile as f64, p.chunk_w as f64, p.rows)
    };
    if vy + vh < surface_y {
        return out;
    }
    let cx0 = (vx / chunk_w).floor() as i64;
    let cx1 = ((vx + vw) / chunk_w).floor() as i64;
    let r0 = gml::clamp(((vy - surface_y) / t).floor(), 0.0, (rows - 1) as f64);
    let r1 = gml::clamp(((vy + vh - surface_y) / t).floor(), 0.0, (rows - 1) as f64);
    let shade = merge_colour(0.0, C_WHITE, vis("food_shade"));
    // Texture filter state: the LÖVE port's `gpu_get_tex_filter()` always returns true, so the
    // HUD's `cave_perk_icons_draw` leaves `gpu_set_tex_filter(true)` behind every frame; world
    // drawing therefore starts linear-filtered until the food glow switches it off (as below).
    out.back.set_filter(true);
    out.front.set_filter(true);
    let b = &mut out.back;
    let mut r = r0;
    while r <= r1 {
        let yy = surface_y + r * t;
        let spr = soil_sprite(r);
        let mut xx = (vx / t).floor() * t;
        while xx < vx + vw {
            b.sprite(a, spr, 0, xx, yy);
            xx += t;
        }
        r += 1.0;
    }
    world_draw_soil_edges(b, a, &sim.world, vx, vy, vw, vh);
    world_draw_dirt(b, a, &sim.world, vx, vy, vw, vh);
    world_draw_bones(b, a, sim, ctx.now, vx, vy, vw, vh);
    world_draw_cave(&mut out.back, a, sim, ctx.now, vx, vy, vw, vh);
    out.chunks.extend(cx0..=cx1);
    let f = &mut out.front;
    for ci in cx0..=cx1 {
        let w = &sim.world;
        if let Some(ch) = w.chunk(ci) {
            world_draw_stone_deposits(f, a, w, ch, vx, vy, vw, vh);
        }
    }
    for ci in cx0..=cx1 {
        world_draw_lakes(f, a, &mut sim.world, flares, ci, ctx.now, vx, vy, vw, vh);
    }
    for ci in cx0..=cx1 {
        let w = &sim.world;
        if let Some(ch) = w.chunk(ci) {
            world_draw_stones(f, a, ctx.stones, w, ch, vx, vy, vw, vh);
        }
    }
    world_draw_ocean(f, a, &sim.world, ctx.now, vx, vy, vw, vh);
    let mut vis_foods = Vec::new();
    for ci in cx0..=cx1 {
        if let Some(ch) = sim.world.chunk(ci) {
            for fd in &ch.foods {
                if fd.collected
                    || fd.x < vx - 48.0
                    || fd.x > vx + vw + 48.0
                    || fd.y < vy - 48.0
                    || fd.y > vy + vh + 48.0
                {
                    continue;
                }
                vis_foods.push((fd.kind, fd.cell, fd.x, fd.y));
            }
        }
    }
    let glow = world_draw_food_glow(&mut out, a, ctx, &vis_foods, vx, vy, vw, vh);
    for &(kind, cell, x, y) in &vis_foods {
        let spr = food_sprite(kind as f64);
        let (sw, _) = sprite_size(a, spr);
        let half = sw * 0.5;
        out.front.sprite_part_ext(
            a,
            spr,
            0,
            [0.0, cell as f64 * sw, sw, sw],
            (x - half).floor(),
            (y - half).floor(),
            1.0,
            1.0,
            shade,
            1.0,
        );
    }
    world_draw_food_glow_inner(&mut out.front, ctx, glow);
    out
}

// port: world_draw_soil_edges
/// The band seams: every `soil_band_rows` rows after the first rows, tiled across the view.
pub fn world_draw_soil_edges(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    x0: f64,
    y0: f64,
    vw: f64,
    vh: f64,
) {
    let p = &w.params;
    let (sy, tile) = (p.surface_y, p.tile as f64);
    let first = wld("soil_first_rows").max(1.0);
    let band_rows = wld("soil_band_rows").max(1.0);
    let mut band = (div(((y0 - sy) / tile - first).floor(), band_rows) - 4.0).max(0.0);
    let row_end = (p.rows as f64).min(((y0 + vh - sy) / tile).ceil() + 1.0);
    let mut row = first + band * band_rows;
    while row < row_end {
        let spr = soil_edge_sprite(band, row);
        let (sw, sh) = sprite_size(a, spr);
        band += 1.0;
        if sw > 0.0 {
            let yy = sy + row * tile;
            if !(yy > y0 + vh) && !(yy + sh < y0) {
                let mut xx = (x0 / sw).floor() * sw;
                while xx < x0 + vw {
                    b.sprite(a, spr, 0, xx, yy);
                    xx += sw;
                }
            }
        }
        row += band_rows;
    }
}

// port: world_draw_dirt
/// Scattered dirt specks on a hashed grid, sheet by soil band.
pub fn world_draw_dirt(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let p = &w.params;
    let (_, dh) = sprite_size(a, "dirt1");
    let sp = dh.max(wld("dirt_spacing_px"));
    let chance = wld("dirt_chance");
    let y0 = vy.max(p.surface_y);
    let y1 = (vy + vh).min(p.bottom_y);
    if y1 <= y0 {
        return;
    }
    let cx0 = (vx / sp).floor() - 1.0;
    let cx1 = ((vx + vw) / sp).floor();
    let cy0 = ((y0 - p.surface_y) / sp).floor() - 1.0;
    let cy1 = ((y1 - p.surface_y) / sp).floor();
    let jit = sp - dh;
    let mut cy = cy0;
    while cy <= cy1 {
        let mut cx = cx0;
        while cx <= cx1 {
            if w.hash01(cx, cy, 71.0) < chance {
                let py = p.surface_y + cy * sp + w.hash01(cx, cy, 73.0) * jit;
                if !(py < p.surface_y) && py + dh <= p.bottom_y {
                    let px = cx * sp + w.hash01(cx, cy, 72.0) * jit;
                    let spr = dirt_sprite(((py + dh * 0.5 - p.surface_y) / p.tile as f64).floor());
                    let (sw, sh) = sprite_size(a, spr);
                    let nf = div(sw, sh).max(1.0);
                    let f = (nf - 1.0).min((w.hash01(cx, cy, 74.0) * nf).floor());
                    b.sprite_part(a, spr, 0, [f * sh, 0.0, sh, sh], px.floor(), py.floor());
                }
            }
            cx += 1.0;
        }
        cy += 1.0;
    }
}

// port: world_draw_bones
/// Small bones of the chunks around the view (tinted by depth), the big bones, then the wraps of
/// taken relics.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_bones(
    b: &mut QuadBatch,
    a: &GameAssets,
    sim: &mut Sim,
    now: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let kinds = bone_kinds();
    let w = &mut sim.world;
    let (dense, deep) = (w.dense_y(), w.deep_y());
    let tints = [
        vis("bones_tint_plain"),
        vis("bones_tint_clay"),
        vis("bones_tint_deep"),
    ];
    let cw = w.params.chunk_w as f64;
    let c0 = ((vx - 600.0) / cw).floor() as i64;
    let c1 = ((vx + vw) / cw).floor() as i64;
    for c in c0..=c1 {
        let Some(ch) = w.chunk(c) else { continue };
        let list = ch.bones.clone();
        for bn in &list {
            if bn.x + bn.w < vx || bn.x > vx + vw || bn.y + bn.h < vy || bn.y > vy + vh {
                continue;
            }
            let cy = bn.y + bn.h * 0.5;
            let tint = tints[if cy >= deep {
                2
            } else if cy >= dense {
                1
            } else {
                0
            }];
            let dx = if bn.flip >= 0.0 { bn.x } else { bn.x + bn.w };
            let Some(k) = kinds.get(bn.kind.max(0) as usize) else {
                continue;
            };
            b.sprite_ext(a, k.sprite, 0, dx, bn.y, bn.flip, 1.0, 0.0, tint, 1.0);
        }
    }
    let big: Vec<_> = world_big_bones(w).to_vec();
    for bb in &big {
        if bb.x2 < vx || bb.x1 > vx + vw || bb.y2 < vy || bb.y1 > vy + vh {
            continue;
        }
        b.sprite_ext(
            a,
            bb.kind.sprite(),
            0,
            bb.x1,
            bb.y1,
            1.0,
            1.0,
            0.0,
            vis("bones_tint_deep"),
            1.0,
        );
    }
    relics_wrap_draw_visible(b, sim, now, vx, vy, vx + vw, vy + vh);
}

// port: relics_wrap_draw_visible
/// Wrap outlines of the taken relics overlapping (x1, y1)-(x2, y2). Also used by the mycelium
/// glow seed pass (T14).
pub fn relics_wrap_draw_visible(
    b: &mut QuadBatch,
    sim: &mut Sim,
    now: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) {
    let rs: Vec<_> = world_relics(&mut sim.world).to_vec();
    for r in &rs {
        if !relic_taken(&sim.state, r) || r.x2 < x1 || r.x1 > x2 || r.y2 < y1 || r.y1 > y2 {
            continue;
        }
        cave_wrap_draw(
            b,
            &sim.world.caves,
            r.relic.id(),
            &r.id,
            r.x1,
            r.y1,
            r.k,
            now,
        );
    }
}

// port: world_draw_cave
/// The caves in view; a taken cave glows (additive copy pulsing, flashing after the take) and
/// shows its wrap outline.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_cave(
    b: &mut QuadBatch,
    a: &GameAssets,
    sim: &mut Sim,
    now: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    // caves_mask_init: the masks are embedded in myco_core (R7)
    let rects: Vec<_> = sim.world.cave_rects().to_vec();
    for (i, r) in rects.iter().enumerate() {
        if r.x2 < vx || r.x1 > vx + vw || r.y2 < vy || r.y1 > vy + vh {
            continue;
        }
        let d = cave_def(r.kind.id());
        let (_, h) = sprite_size(a, d.sprite);
        let k = r.k;
        let part = [d.left, 0.0, d.w, h];
        b.sprite_part_ext(a, d.sprite, 0, part, r.x1, r.y1, k, k, C_WHITE, 1.0);
        if !cave_taken(&sim.state, &r.id) {
            continue;
        }
        let fl = sim
            .world
            .caves
            .flash
            .get(&r.id)
            .copied()
            .unwrap_or(-100000.0);
        let age = now - fl;
        let f = (1.0 - age / vis("cave_flash_ms").max(1.0)).max(0.0);
        let glow = vis("cave_glow_alpha") * ((now / 700.0 + i as f64).sin() * 0.25 + 0.75);
        let al = glow.max(f * vis("cave_flash_alpha"));
        let old = b.blend;
        b.set_blend(QuadBlend::Add);
        b.sprite_part_ext(a, d.sprite, 0, part, r.x1, r.y1, k, k, C_WHITE, al);
        b.set_blend(old);
        cave_wrap_draw(b, &sim.world.caves, r.kind.id(), &r.id, r.x1, r.y1, k, now);
    }
}

// port: caves_wrap_draw_visible
/// Wrap outlines of the taken caves overlapping (x1, y1)-(x2, y2) (the mycelium glow seed, T14).
pub fn caves_wrap_draw_visible(
    b: &mut QuadBatch,
    sim: &mut Sim,
    now: f64,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
) {
    let rects: Vec<_> = sim.world.cave_rects().to_vec();
    for r in &rects {
        if cave_taken(&sim.state, &r.id)
            && !(r.x2 < x1)
            && !(r.x1 > x2)
            && !(r.y2 < y1)
            && !(r.y1 > y2)
        {
            cave_wrap_draw(
                b,
                &sim.world.caves,
                r.kind.id(),
                &r.id,
                r.x1,
                r.y1,
                r.k,
                now,
            );
        }
    }
}

// port: cave_wrap_draw
/// The hypha outline wrapping a taken cave or relic: grows both ways from the touch point over
/// `cave_wrap_ms`, branches sprout as it passes, stems grow after `cave_stem_start`. Two passes:
/// a 0.45-alpha halo, then the core.
#[allow(clippy::too_many_arguments)]
pub fn cave_wrap_draw(
    b: &mut QuadBatch,
    caves: &CaveRt,
    mask_id: &str,
    wrap_id: &str,
    x1: f64,
    y1: f64,
    k: f64,
    now: f64,
) {
    let Some(m) = cave_mask(mask_id) else {
        return;
    };
    let pts = &m.outline;
    let n = pts.len();
    if n == 0 {
        return;
    }
    let (mut t, mut ts, mut start) = (1.0, 1.0, 0usize);
    if let Some(wr) = caves.wrap.get(wrap_id) {
        let age = now - wr.born;
        let dur = vis("cave_wrap_ms").max(1.0);
        t = gml::clamp(age / dur, 0.0, 1.0);
        ts = gml::clamp(
            (age - dur * vis("cave_stem_start")) / vis("cave_stem_ms").max(1.0),
            0.0,
            1.0,
        );
        let mut best = f64::INFINITY;
        for (i, p) in pts.iter().enumerate() {
            let d = gml::point_distance(p[0], p[1], wr.from_x, wr.from_y);
            if d < best {
                best = d;
                start = i;
            }
        }
    }
    let e = 1.0 - gml::power(1.0 - t, 2.0);
    let len = (n as f64 * 0.5 * e).ceil();
    let halo = vis("cave_wrap_halo_colour");
    let core = vis("cave_wrap_core_colour");
    let halo_px = vis("cave_wrap_halo_px");
    let core_px = vis("cave_wrap_core_px");
    let ni = n as i64;
    let at = |i: i64| pts[i.rem_euclid(ni) as usize];
    for pass in 0..2 {
        let (col, lw, alpha) = if pass == 0 {
            (halo, halo_px, 0.45)
        } else {
            (core, core_px, 1.0)
        };
        for dir in [-1i64, 1] {
            let mut s = 0i64;
            while (s as f64) < len {
                let p = at(start as i64 + dir * s + ni * 2);
                let q = at(start as i64 + dir * (s + 1) + ni * 2);
                b.line_width_colour(
                    x1 + p[0] * k,
                    y1 + p[1] * k,
                    x1 + q[0] * k,
                    y1 + q[1] * k,
                    lw,
                    col,
                    col,
                    alpha,
                );
                s += 1;
            }
        }
        for br in &m.branches {
            let mut dd = (br.at - start as i64).abs() as f64;
            dd = dd.min(n as f64 - dd);
            let bt = gml::clamp(
                (len - dd) / vis("cave_branch_grow").max(1.0) - br.delay,
                0.0,
                1.0,
            );
            if bt > 0.0 {
                let bp = &br.points;
                let nn = bp.len() as f64 - 1.0;
                let lim = nn * bt;
                let mut s = 0usize;
                while (s as f64) < nn && (s as f64) < lim {
                    let (p, c) = (bp[s], bp[s + 1]);
                    let f = (lim - s as f64).min(1.0);
                    let wv = gml::lerp(0.9, 0.45, s as f64 / nn.max(1.0));
                    b.line_width_colour(
                        x1 + p[0] * k,
                        y1 + p[1] * k,
                        x1 + gml::lerp(p[0], c[0], f) * k,
                        y1 + gml::lerp(p[1], c[1], f) * k,
                        lw * wv,
                        col,
                        col,
                        alpha,
                    );
                    s += 1;
                }
            }
        }
        for sp in &m.stems {
            let nn = sp.len() as f64 - 1.0;
            let lim = nn * ts;
            let mut s = 0usize;
            while (s as f64) < nn && (s as f64) < lim {
                let (p, c) = (sp[s], sp[s + 1]);
                let f = (lim - s as f64).min(1.0);
                b.line_width_colour(
                    x1 + p[0] * k,
                    y1 + p[1] * k,
                    x1 + gml::lerp(p[0], c[0], f) * k,
                    y1 + gml::lerp(p[1], c[1], f) * k,
                    lw * 0.8,
                    col,
                    col,
                    alpha,
                );
                s += 1;
            }
        }
    }
}

/// The tiles of one chunk (world_draw's tile loop over every generated row): hard soil, stone
/// and water cells. Drawn as the chunk mesh.
pub fn chunk_tiles(b: &mut QuadBatch, a: &GameAssets, w: &WorldRt, ch: &WorldChunkSave) {
    let p = &w.params;
    let t = p.tile as f64;
    let bx = (ch.cx * p.chunk_w) as f64;
    for r in 0..ch.rows {
        let yy = p.surface_y + r as f64 * t;
        for c in 0..p.cols {
            let xx = bx + c as f64 * t;
            match ch
                .tiles
                .get((r * p.cols + c) as usize)
                .copied()
                .unwrap_or(0)
            {
                1 => b.sprite(a, "hardsoil_tile", 0, xx, yy),
                2 => b.sprite(a, "stone_tile", 0, xx, yy),
                3 => world_draw_water_fill(b, xx, yy, t),
                _ => {}
            }
        }
    }
}

// port: world_draw_water_fill
pub fn world_draw_water_fill(b: &mut QuadBatch, x: f64, y: f64, s: f64) {
    b.rect(
        x,
        y,
        x + s - 1.0,
        y + s - 1.0,
        vis("water_reservoir_colour"),
        1.0,
    );
}

// port: world_draw_stone_deposits
/// Deposit cells of a chunk, clipped to the chunk's columns, deep ones tinted.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_stone_deposits(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    ch: &WorldChunkSave,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let cw = w.params.chunk_w as f64;
    let x0 = ch.cx as f64 * cw;
    let x1 = x0 + cw;
    for c in &ch.stone_cells {
        let (px, py) = (c.col as f64 * 48.0, c.row as f64 * 48.0);
        if px + 48.0 < vx || px > vx + vw || py + 48.0 < vy || py > vy + vh {
            continue;
        }
        let l = px.max(x0);
        let r = (px + 48.0).min(x1);
        if r <= l {
            continue;
        }
        let deep = c.deep.unwrap_or(py + 48.0 > w.deep_y());
        let col = if deep {
            vis("deep_stone_colour")
        } else {
            C_WHITE
        };
        b.sprite_part_ext(
            a,
            "stone_tile",
            0,
            [
                c.tc as f64 * 64.0 + (l - px),
                c.tr as f64 * 64.0,
                r - l,
                48.0,
            ],
            l,
            py,
            1.0,
            1.0,
            col,
            1.0,
        );
    }
}

// port: world_draw_lakes
/// A chunk's lakes: the dark basin, the water below the cut row (sinking as the lake empties),
/// sparkles respawning in free spots, and the surface edge strip while not full.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_lakes(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &mut WorldRt,
    flares: &mut LakeFlares,
    cx: i64,
    now: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    if !lakes_ready() {
        return;
    }
    let (fsw, fh) = sprite_size(a, "water_flare");
    let fw = div(fsw, 3.0);
    let (ew, eh) = sprite_size(a, "water_edge");
    let ffps = vis("water_flare_fps").max(1.0);
    let Some(ch) = w.chunk_mut(cx) else {
        return;
    };
    for (li, lk) in ch.lakes.iter_mut().enumerate() {
        let kd = myco_core::world::lakes::lake_kind(lk.kind);
        if lk.x + kd.width < vx || lk.x > vx + vw || lk.y + kd.height < vy || lk.y > vy + vh {
            continue;
        }
        b.sprite_part(
            a,
            "water_shade",
            0,
            [kd.left, 0.0, kd.width, kd.height],
            lk.x,
            lk.y,
        );
        if lk.amount <= 0.0 {
            continue;
        }
        let cut = lake_cut_row(lk, kd);
        if cut < kd.height {
            b.sprite_part(
                a,
                "water",
                0,
                [kd.left, cut, kd.width, kd.height - cut],
                lk.x,
                lk.y + cut,
            );
        }
        // init and respawns: `world_lakes_step` after each step; a lake without flares yet
        // (it came into view since) sparkles from the next step
        let empty = Vec::new();
        let fl = flares.map.get(&(cx, li, lk.seed)).unwrap_or(&empty);
        for &f in fl {
            if f.placed {
                let age = now - f.born;
                let al = lake_flare_alpha(age);
                if al > 0.0 {
                    let top = (cut - f.y).max(0.0);
                    if top < fh {
                        let frame = (age * ffps / 1000.0).floor() % 3.0;
                        b.sprite_part_ext(
                            a,
                            "water_flare",
                            0,
                            [frame * fw, top, fw, fh - top],
                            lk.x + f.x,
                            lk.y + f.y + top,
                            1.0,
                            1.0,
                            C_WHITE,
                            al,
                        );
                    }
                }
            }
        }
        if lk.amount < lk.full {
            let row = vis("water_edge_surface_row");
            let mut r = 0.0;
            while r < eh {
                let yy = cut - row + r;
                if !(yy < 0.0 || yy >= kd.height) {
                    let si = yy.max(cut.min(kd.height - 1.0)) as usize;
                    if let Some(spans) = kd.spans.get(si) {
                        let mut s = 0;
                        while s + 1 < spans.len() {
                            let x1 = spans[s + 1];
                            let mut xx = spans[s];
                            while xx <= x1 {
                                let ox = gml::gmod(xx, ew);
                                let cw = (ew - ox).min(x1 + 1.0 - xx);
                                b.sprite_part(
                                    a,
                                    "water_edge",
                                    0,
                                    [ox, r, cw, 1.0],
                                    lk.x + xx,
                                    lk.y + yy,
                                );
                                xx += cw;
                            }
                            s += 2;
                        }
                    }
                }
                r += 1.0;
            }
        }
    }
}

// port: world_draw_stones
/// Loose stones of a chunk: grey sheet below the dense line, tinted below the deep line.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_stones(
    b: &mut QuadBatch,
    a: &GameAssets,
    art: Option<&StonesArt>,
    w: &WorldRt,
    ch: &WorldChunkSave,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let pad = stone_art().reach + 2.0;
    let kinds = &stone_art().kinds;
    let (_, sh) = sprite_size(a, "stones");
    for s in &ch.stones {
        if s.x < vx - pad || s.x > vx + vw + pad || s.y < vy - pad || s.y > vy + vh + pad {
            continue;
        }
        let Some(k) = kinds.get(s.kind.max(0) as usize) else {
            continue;
        };
        let bottom = s.y + k.radius;
        let col = if bottom <= w.deep_y() {
            C_WHITE
        } else {
            vis("deep_stone_colour")
        };
        let grey = bottom > w.dense_y();
        match art {
            Some(art) => {
                let img = if grey { &art.grey } else { &art.plain };
                b.image_ext(
                    img,
                    [k.left, 0.0, k.w, sh],
                    [-k.ox, -k.oy, k.w - k.ox, sh - k.oy],
                    s.x.floor(),
                    s.y.floor(),
                    1.0,
                    1.0,
                    s.angle,
                    col,
                    1.0,
                );
            }
            None => b.sprite_part_ext(
                a,
                "stones",
                0,
                [k.left, 0.0, k.w, sh],
                s.x.floor() - k.ox,
                s.y.floor() - k.oy,
                1.0,
                1.0,
                col,
                1.0,
            ),
        }
    }
}

// port: world_draw_bottom_rock
/// Survival's rock bottom: stone tiles from the ocean line down (top row edge tiles).
#[allow(clippy::too_many_arguments)]
pub fn world_draw_bottom_rock(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    x0: f64,
    y0: f64,
    vw: f64,
    vh: f64,
    oy: f64,
) {
    let edge = stone_role_tiles(true, false, false, false, 0.0);
    let fill = stone_role_tiles(false, false, false, false, 0.0);
    let ts = 48.0;
    let col = vis("deep_stone_colour");
    let c0 = (x0 / ts).floor() as i64;
    let c1 = ((x0 + vw) / ts).floor() as i64;
    let r0 = ((y0 - oy) / ts).floor().max(0.0) as i64;
    let r1 = ((y0 + vh - oy) / ts).floor() as i64;
    for r in r0..=r1 {
        let set = if r == 0 { edge } else { fill };
        let yy = oy + r as f64 * ts;
        for c in c0..=c1 {
            let (tc, tr) = stone_pick_tile(w, c as f64, r as f64, set, 41.0);
            b.sprite_part_ext(
                a,
                "stone_tile",
                0,
                [tc as f64 * 64.0, tr as f64 * 64.0, ts, ts],
                c as f64 * ts,
                yy,
                1.0,
                1.0,
                col,
                1.0,
            );
        }
    }
}

// port: world_draw_ocean
/// Below the ocean line: rock bottom (survival) or the water fading to deep, its sparkles, and
/// the seam sprite along the line.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_ocean(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    now: f64,
    x0: f64,
    y0: f64,
    vw: f64,
    vh: f64,
) {
    let oy = w.ocean_y();
    let bottom = y0 + vh;
    if bottom <= oy {
        return;
    }
    let rock = w.bottom_is_rock();
    if rock {
        world_draw_bottom_rock(b, a, w, x0, y0, vw, vh, oy);
    }
    let x1 = x0 - 1.0;
    let x2 = x0 + vw + 1.0;
    let col_water = vis("water_reservoir_colour");
    let col_deep = vis("ocean_deep_colour");
    let fade = vis("ocean_fade_px");
    let top = oy.max(y0);
    if !rock && top < oy + fade {
        let fy = bottom.min(oy + fade);
        let ca = merge_colour(col_water, col_deep, (top - oy) / fade);
        let cb = merge_colour(col_water, col_deep, (fy - oy) / fade);
        b.rect_colour(x1, top, x2, fy, [ca, ca, cb, cb], 1.0);
    }
    if !rock && bottom > oy + fade {
        b.rect(x1, top.max(oy + fade), x2, bottom, col_deep, 1.0);
    }
    if !rock {
        world_draw_ocean_flares(b, a, w, now, x0, y0, vw, vh, oy);
    }
    let orow = ocean_row();
    let spr = soil_edge_sprite(soil_band(orow - 1.0), orow);
    let (sw, sh) = sprite_size(a, spr);
    if sw > 0.0 && oy + sh >= y0 && oy <= bottom {
        let mut xx = (x0 / sw).floor() * sw;
        while xx < x0 + vw {
            b.sprite(a, spr, 0, xx, oy);
            xx += sw;
        }
    }
}

// port: world_draw_ocean_flares
/// Sparkles on the open ocean: one hashed slot per cell, cycling with the flare life.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_ocean_flares(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    now: f64,
    x0: f64,
    y0: f64,
    vw: f64,
    vh: f64,
    oy: f64,
) {
    let cell = vis("ocean_flare_cell_px").max(32.0);
    let chance = vis("ocean_flare_chance");
    let (fsw, fh) = sprite_size(a, "water_flare");
    let fw = div(fsw, 3.0);
    let gap = vis("ocean_flare_top_gap_px");
    let life = lake_flare_life_ms();
    let fps = vis("water_flare_fps").max(1.0);
    let ytop = (y0 - fh).max(oy + gap);
    if ytop >= y0 + vh {
        return;
    }
    let c0 = ((x0 - fw) / cell).floor();
    let c1 = ((x0 + vw) / cell).floor();
    let r0 = ((ytop - oy) / cell).floor();
    let r1 = ((y0 + vh - oy) / cell).floor();
    let mut r = r0;
    while r <= r1 {
        let mut c = c0;
        while c <= c1 {
            let t = now + w.hash01(c, r, 97.0) * life;
            let cyc = (t / life).floor();
            let age = t - cyc * life;
            if w.hash01(c * 7.0 + cyc, r, 98.0) < chance {
                let al = lake_flare_alpha(age);
                if al > 0.0 {
                    let fx = c * cell + w.hash01(c + cyc * 131.0, r, 99.0) * (cell - fw);
                    let fy = oy + r * cell + w.hash01(c, r + cyc * 131.0, 100.0) * (cell - fh);
                    if !(fy < oy + gap) {
                        let frame = gml::gmod((age * fps / 1000.0).floor(), 3.0);
                        b.sprite_part_ext(
                            a,
                            "water_flare",
                            0,
                            [frame * fw, 0.0, fw, fh],
                            fx.floor(),
                            fy.floor(),
                            1.0,
                            1.0,
                            C_WHITE,
                            al,
                        );
                    }
                }
            }
            c += 1.0;
        }
        r += 1.0;
    }
}

// port: food_glow_pulse
pub fn food_glow_pulse(now: f64) -> f64 {
    let lo = vis("food_glow_pulse_low").max(0.0);
    let hi = vis("food_glow_pulse_high").max(lo);
    let period = vis("food_glow_pulse_seconds").max(0.1) * 2000.0;
    gml::lerp(
        lo,
        hi,
        ((std::f64::consts::TAU * now / period).sin() + 1.0) * 0.5,
    )
}

// port: world_draw_food_glow
/// The glow behind the visible foods: the foods into a downscaled seed surface, blurred across
/// and down (offscreen passes, see [`GlowPass`]), drawn additively in `food_glow_colour` at
/// `strength·(1 − inner)`. Returns what the inner pass needs.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_food_glow(
    out: &mut WorldFrame,
    a: &GameAssets,
    ctx: &DrawCtx,
    foods: &[(i64, i64, f64, f64)],
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) -> Option<GlowInfo> {
    let strength = vis("food_glow_strength") * food_glow_pulse(ctx.now);
    if strength <= 0.0 || foods.is_empty() {
        return None;
    }
    let ds = vis("food_glow_downscale").max(2.0);
    let sw = div(vw, ds).max(1.0) + 2.0;
    let sh = div(vh, ds).max(1.0) + 2.0;
    let ox = (vx / ds).floor() * ds;
    let oy = (vy / ds).floor() * ds;
    let swell = vis("food_glow_swell").max(0.1) / ds;
    let inner = gml::clamp(vis("food_glow_inner"), 0.0, 1.0);
    let mut seed = QuadBatch::new();
    seed.set_filter(true);
    for &(kind, cell, x, y) in foods {
        let spr = food_sprite(kind as f64);
        let (fw, _) = sprite_size(a, spr);
        let half = fw * swell * 0.5;
        seed.sprite_part_ext(
            a,
            spr,
            0,
            [0.0, cell as f64 * fw, fw, fw],
            (x - ox) / ds - half,
            (y - oy) / ds - half,
            swell,
            swell,
            C_WHITE,
            1.0,
        );
    }
    let rad = vis("food_glow_radius").max(0.5) * 0.25;
    out.glow = Some(GlowPass {
        sw: sw as u32,
        sh: sh as u32,
        seed,
        rad,
    });
    let info = GlowInfo {
        x: ox,
        y: oy,
        step: ds,
        light: strength,
        sw,
        sh,
    };
    draw_glow_surface(
        &mut out.front,
        ctx,
        &info,
        gml::clamp(strength * (1.0 - inner), 0.0, 1.0),
    );
    Some(info)
}

/// `gpu_set_tex_filter(true)`, `bm_add`, `draw_surface_ext(food_glow_seed, x, y, step, step,
/// 0, food_glow_colour, a)`, `bm_normal`, `gpu_set_tex_filter(false)` (the filter stays off).
fn draw_glow_surface(b: &mut QuadBatch, ctx: &DrawCtx, g: &GlowInfo, alpha: f64) {
    b.set_filter(true);
    let Some(img) = ctx.glow_image else {
        b.set_filter(false);
        return;
    };
    let old_b = b.blend;
    b.set_blend(QuadBlend::Add);
    b.image_ext(
        img,
        [0.0, 0.0, g.sw, g.sh],
        [0.0, 0.0, g.sw, g.sh],
        g.x,
        g.y,
        g.step,
        g.step,
        0.0,
        vis("food_glow_colour"),
        alpha,
    );
    b.set_blend(old_b);
    b.set_filter(false);
}

// port: world_draw_food_glow_inner
/// The glow again over the foods, at `light·food_glow_inner`.
pub fn world_draw_food_glow_inner(b: &mut QuadBatch, ctx: &DrawCtx, g: Option<GlowInfo>) {
    let Some(g) = g else {
        return;
    };
    let al = gml::clamp(g.light * vis("food_glow_inner"), 0.0, 1.0);
    if al <= 0.0 {
        return;
    }
    draw_glow_surface(b, ctx, &g, al);
}

// port: world_draw_turf
/// The turf strip along the surface (forest Draw_73), tiled across the view.
#[allow(clippy::too_many_arguments)]
pub fn world_draw_turf(
    b: &mut QuadBatch,
    a: &GameAssets,
    w: &WorldRt,
    x0: f64,
    y0: f64,
    vw: f64,
    vh: f64,
) {
    if w.params.turf < 0 {
        return;
    }
    let Some(def) = a.sprites.get(w.params.turf as usize) else {
        return;
    };
    let sw = def.size.x as f64;
    if sw <= 0.0 {
        return;
    }
    let ty = w.params.surface_y + wld("turf_offset_y");
    let top = ty - def.origin.y as f64;
    if top > y0 + vh || top + (def.size.y as f64) < y0 {
        return;
    }
    let mut xx = (x0 / sw).floor() * sw;
    while xx < x0 + vw {
        b.def_ext(def, 0, xx, ty, 1.0, 1.0, 0.0, C_WHITE, 1.0);
        xx += sw;
    }
}

// ---------------------------------------------------------------------------------------------
// GUI: popups, perk icons, debug overlay

struct Measure<'a, 'b>(&'a Gm<'b>);

impl TextMeasure for Measure<'_, '_> {
    fn string_width(&self, s: &str) -> f64 {
        self.0.string_width(s)
    }
    fn string_height(&self, s: &str) -> f64 {
        self.0.string_height(s)
    }
    fn string_width_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
        self.0.string_width_ext(s, sep, w)
    }
    fn string_height_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
        self.0.string_height_ext(s, sep, w)
    }
}

// port: cave_popups_draw
/// Perk names rising over taken caves for `cave_popup_ms` (GUI, black shadow then white);
/// expired popups are dropped. `vx, vy`: the view's top-left; `gui_w` the GUI width.
#[allow(clippy::too_many_arguments)]
pub fn cave_popups_draw(
    g: &mut Gm,
    caves: &mut CaveRt,
    loc: &dyn Fn(&str) -> String,
    now: f64,
    vx: f64,
    vy: f64,
    gui_scale: f64,
    gui_w: f64,
) {
    let ms = vis("cave_popup_ms");
    g.halign = FA_CENTER;
    g.valign = 2;
    let mut keep = Vec::new();
    for p in caves.popups.drain(..) {
        let k = (now - p.born) / ms.max(1.0);
        if k >= 1.0 {
            continue;
        }
        let a = (k * 6.0).min(1.0) * ((1.0 - k) * 3.0).min(1.0);
        let px = (p.x - vx) * gui_scale;
        let py = (p.y - vy - vis("cave_popup_rise_px") * k) * gui_scale;
        let text = loc(&p.text);
        let sc = goal_fit(
            &Measure(g),
            &text,
            vis("cave_popup_scale"),
            gui_w - 48.0,
            0.0,
            -1.0,
        );
        g.set_alpha(a * 0.85);
        g.set_colour(0.0);
        g.text_transformed(px + 2.0, py + 2.0, &text, sc, sc);
        g.set_alpha(a);
        g.set_colour(C_WHITE);
        g.text_transformed(px, py, &text, sc, sc);
        keep.push(p);
    }
    caves.popups = keep;
    g.set_alpha(1.0);
    g.halign = FA_LEFT;
    g.valign = FA_TOP;
}

// port: cave_perk_icons_draw
/// The HUD row of held perks' icons at (x, y), each fitted into `cave_icon_px`.
pub fn cave_perk_icons_draw(g: &mut Gm, state: &GameState, mut x: f64, y: f64) {
    let px = vis("cave_icon_px");
    let gap = vis("cave_icon_gap_px");
    for d in cave_defs() {
        if !perk_has(state, d.kind.id()) {
            continue;
        }
        let Some(def) = g.assets.try_sprite(d.hud) else {
            continue;
        };
        let (w, h) = (def.size.x as f64, def.size.y as f64);
        let (oxs, oys) = (def.origin.x as f64, def.origin.y as f64);
        let s = px / w.max(h);
        g.sprite_ext(
            d.hud,
            0,
            x + (px - w * s) * 0.5 + oxs * s,
            y + (px - h * s) * 0.5 + oys * s,
            s,
            s,
            C_WHITE,
            1.0,
        );
        x += px + gap;
    }
}

/// `anon__68900__caves_debug_draw`: nearest bones first.
// port: anon__68900__caves_debug_draw__scr_world
pub fn caves_debug_cmp(a: &DebugMark, b: &DebugMark) -> std::cmp::Ordering {
    a.d.total_cmp(&b.d)
}

/// One labelled point of the cave debug overlay.
#[derive(Clone, Debug, PartialEq)]
pub struct DebugMark {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub colour: f64,
    pub dim: bool,
    pub r: f64,
    pub d: f64,
}

// port: caves_debug_draw
/// Debug overlay of caves, big bones and the nearest small bones (circles on screen, arrows at
/// the edge for off-screen ones). The original only calls it under `if false and ...`, so the
/// app never draws it; `label` formats the distance (`length_text(pixels_to_cm(d))`).
#[allow(clippy::too_many_arguments)]
pub fn caves_debug_draw(
    g: &mut Gm,
    sim: &mut Sim,
    label: &dyn Fn(f64) -> String,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let colour = |id: &str| match id {
        "blue" => 16765550.0,
        "purple" => 16742590.0,
        "red" => 7237375.0,
        "green" => 7270550.0,
        "yellow" => 5956095.0,
        _ => C_WHITE,
    };
    let (hw, hh) = (vw * 0.5, vh * 0.5);
    let (cx, cy) = (vx + hw, vy + hh);
    let margin = 48.0;
    let bone_col = 13492715.0;
    let mut items = Vec::new();
    let rects: Vec<_> = sim.world.cave_rects().to_vec();
    for r in &rects {
        items.push(DebugMark {
            x: (r.x1 + r.x2) * 0.5,
            y: (r.y1 + r.y2) * 0.5,
            name: r.id.clone(),
            colour: colour(r.kind.id()),
            dim: cave_taken(&sim.state, &r.id),
            r: 40.0,
            d: 0.0,
        });
    }
    let big: Vec<_> = world_big_bones(&mut sim.world).to_vec();
    for b in &big {
        items.push(DebugMark {
            x: (b.x1 + b.x2) * 0.5,
            y: (b.y1 + b.y2) * 0.5,
            name: b.kind.id().to_string(),
            colour: C_WHITE,
            dim: false,
            r: 60.0,
            d: 0.0,
        });
    }
    let kinds = bone_kinds();
    let mut bones = Vec::new();
    let c0 = (cx / sim.world.params.chunk_w as f64).floor() as i64;
    for c in c0 - 24..=c0 + 24 {
        let Some(ch) = sim.world.chunk(c) else {
            continue;
        };
        for b in &ch.bones {
            let (bx, by) = (b.x + b.w * 0.5, b.y + b.h * 0.5);
            bones.push(DebugMark {
                x: bx,
                y: by,
                name: kinds
                    .get(b.kind.max(0) as usize)
                    .map(|k| k.id)
                    .unwrap_or("")
                    .to_string(),
                colour: bone_col,
                dim: false,
                r: 24.0,
                d: gml::point_distance(cx, cy, bx, by),
            });
        }
    }
    bones.sort_by(caves_debug_cmp);
    let n = (bones.len() as f64)
        .min(vis("debug_bones_nearest"))
        .max(0.0) as usize;
    items.extend(bones.into_iter().take(n));
    g.halign = FA_CENTER;
    g.valign = 1;
    for it in &items {
        let (sx, sy) = (it.x - vx, it.y - vy);
        let text = format!(
            "{}  {}",
            it.name,
            label(gml::point_distance(cx, cy, it.x, it.y))
        );
        g.set_alpha(if it.dim { 0.45 } else { 1.0 });
        g.set_colour(it.colour);
        if sx >= 0.0 && sx <= vw && sy >= 0.0 && sy <= vh {
            g.circle(sx, sy, it.r, true);
            g.circle(sx, sy, it.r + 1.0, true);
            g.text_transformed(sx, sy - it.r - 20.0, &text, 0.5, 0.5);
        } else {
            let (dx, dy) = (sx - hw, sy - hh);
            let k = ((hw - margin) / dx.abs().max(0.001)).min((hh - margin) / dy.abs().max(0.001));
            let (ax, ay) = (hw + dx * k, hh + dy * k);
            let dir = gml::point_direction(hw, hh, sx, sy);
            let p = |l: f64, d: f64| (ax + gml::lengthdir_x(l, d), ay + gml::lengthdir_y(l, d));
            let (p1, p2, p3) = (p(22.0, dir), p(14.0, dir + 140.0), p(14.0, dir - 140.0));
            let (c, al) = (g.colour, g.alpha);
            g.tri([
                (p1.0, p1.1, c, al),
                (p2.0, p2.1, c, al),
                (p3.0, p3.1, c, al),
            ]);
            g.text_transformed(
                ax - gml::lengthdir_x(34.0, dir),
                ay - gml::lengthdir_y(34.0, dir),
                &text,
                0.45,
                0.45,
            );
        }
    }
    g.set_alpha(1.0);
    g.set_colour(C_WHITE);
    g.halign = FA_LEFT;
    g.valign = FA_TOP;
}

/// Chunk-mesh bookkeeping of one frame. `built[cx]` says whether chunk `cx`'s mesh is current.
/// The world's `dirty_chunks` are taken (cleared, R9a) and their meshes marked stale; chunks out
/// of `visible` are forgotten (returned second, to despawn); visible chunks without a current
/// mesh are returned first, to rebuild (and are then marked current).
pub fn plan_chunk_meshes(
    w: &mut WorldRt,
    built: &mut HashMap<i64, bool>,
    visible: &[i64],
) -> (Vec<i64>, Vec<i64>) {
    for cx in std::mem::take(&mut w.dirty_chunks) {
        if let Some(b) = built.get_mut(&cx) {
            *b = false;
        }
    }
    let mut dropped: Vec<i64> = built
        .keys()
        .filter(|cx| !visible.contains(cx))
        .copied()
        .collect();
    dropped.sort_unstable();
    for cx in &dropped {
        built.remove(cx);
    }
    let mut rebuild = Vec::new();
    for &cx in visible {
        let b = built.entry(cx).or_insert(false);
        if !*b && w.chunk(cx).is_some() {
            *b = true;
            rebuild.push(cx);
        }
    }
    (rebuild, dropped)
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// The images of the food-glow passes: seed, across-blurred, final (down-blurred).
#[derive(Resource, Clone)]
pub struct GlowImages {
    pub seed: Handle<Image>,
    pub blur: Handle<Image>,
    pub out: Handle<Image>,
    pub size: UVec2,
}

/// Glow pass camera: pass index and the image it currently targets.
#[derive(Component)]
struct GlowCamera(usize, AssetId<Image>);

/// The texture filter `world_draw` leaves behind (`gpu_set_tex_filter` is global state in the
/// original): linear when it drew nothing (view above the surface) or no food glow ran, nearest
/// after the food glow. Task 15's life drawing (obj_life Draw_0, after the forest's Draw_0)
/// starts from it.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct WorldDrawState {
    pub end_linear: bool,
}

impl Default for WorldDrawState {
    fn default() -> Self {
        WorldDrawState { end_linear: true }
    }
}

#[derive(Resource, Default)]
struct WorldPools {
    back: BatchPool,
    front: BatchPool,
    turf: BatchPool,
    glow: [BatchPool; 3],
    chunks: HashMap<i64, BatchPool>,
    built: HashMap<i64, bool>,
}

pub struct WorldRenderPlugin;

/// The set of the world-drawing system (Task 15's life drawing runs after it).
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WorldDrawSet;

impl Plugin for WorldRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<BatchPlugin>() {
            app.add_plugins(BatchPlugin);
        }
        app.init_resource::<LakeFlares>()
            .init_resource::<WorldPools>()
            .init_resource::<WorldDrawState>()
            .add_systems(Startup, (build_world_art, spawn_glow))
            .add_systems(Update, (draw_world.in_set(WorldDrawSet), draw_world_gui))
            .add_systems(
                FixedUpdate,
                crate::render::draw_generate_system
                    .in_set(crate::sim_bridge::SimSet::After)
                    .before(crate::render::mycelium::tiles_step_system)
                    .before(crate::autopilot::dump_state),
            );
    }
}

fn glow_image(images: &mut Assets<Image>, size: UVec2) -> Handle<Image> {
    let mut img = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.texture_descriptor.usage |=
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC;
    images.add(img)
}

fn spawn_glow(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let size = UVec2::new(8, 8);
    let gi = GlowImages {
        seed: glow_image(&mut images, size),
        blur: glow_image(&mut images, size),
        out: glow_image(&mut images, size),
        size,
    };
    for (i, target) in [&gi.seed, &gi.blur, &gi.out].into_iter().enumerate() {
        commands.spawn((
            Camera2d,
            Camera {
                order: -10 + i as isize,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                is_active: false,
                ..default()
            },
            RenderTarget::Image(target.clone().into()),
            RenderLayers::layer(GLOW_LAYERS[i]),
            COMPOSITING,
            GlowCamera(i, target.id()),
        ));
    }
    commands.insert_resource(gi);
}

/// Pixel scans at startup (`stones_init`; the others' data is embedded in myco_core and
/// checked against these scans by tests).
fn build_world_art(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok(px) = PagePixels::load(&assets.root, &assets, &["stones"]) else {
        return;
    };
    let (kinds, reach, plain, grey) = stones_init(&px, &assets);
    let def = assets.sprite("stones");
    let size = Extent3d {
        width: def.size.x as u32,
        height: def.size.y as u32,
        depth_or_array_layers: 1,
    };
    let mk = |data: Vec<u8>| {
        Image::new(
            size,
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD,
        )
    };
    commands.insert_resource(StonesArt {
        plain: images.add(mk(plain)),
        grey: images.add(mk(grey)),
        kinds,
        reach,
    });
}

#[allow(clippy::too_many_arguments)]
fn draw_world(
    mut commands: Commands,
    mut sim: ResMut<SimRes>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    stones: Option<Res<StonesArt>>,
    mut glow: ResMut<GlowImages>,
    mut flares: ResMut<LakeFlares>,
    mut draw_state: ResMut<WorldDrawState>,
    mut pools: ResMut<WorldPools>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmQuad>>,
    mut images: ResMut<Assets<Image>>,
    white: Res<WhiteTexture>,
    mut glow_cams: Query<(
        Entity,
        &mut Camera,
        &mut Projection,
        &mut Transform,
        &mut GlowCamera,
    )>,
) {
    let rv = cams.render_view();
    let sim = &mut sim.0;
    let now = sim.now_ms();
    let glow_out = glow.out.clone();
    let ctx = DrawCtx {
        assets: &assets,
        stones: stones.as_deref(),
        glow_image: Some(&glow_out),
        now,
    };
    let frame = world_draw(sim, &ctx, &mut flares, rv.x, rv.y, rv.w, rv.h);
    let end_linear = if rv.y + rv.h < sim.world.params.surface_y {
        true
    } else {
        frame.front.linear
    };
    if draw_state.end_linear != end_linear {
        draw_state.end_linear = end_linear;
    }
    let mut turf = QuadBatch::new();
    world_draw_turf(&mut turf, &assets, &sim.world, rv.x, rv.y, rv.w, rv.h);

    let pools = &mut *pools;
    let mut t = BatchTarget {
        commands: &mut commands,
        meshes: &mut meshes,
        materials: &mut materials,
        white: &white.0,
    };
    let world_layer = RenderLayers::layer(0);
    pools
        .back
        .sync(&mut t, &frame.back, Z_WORLD, Z_STEP, &world_layer);
    pools
        .front
        .sync(&mut t, &frame.front, Z_WORLD_FRONT, Z_STEP, &world_layer);
    pools.turf.sync(&mut t, &turf, Z_TURF, Z_STEP, &world_layer);

    // chunk tile meshes: rebuilt when dirty (R9a) or when they come into view
    let (rebuild, dropped) = plan_chunk_meshes(&mut sim.world, &mut pools.built, &frame.chunks);
    for cx in dropped {
        if let Some(mut pool) = pools.chunks.remove(&cx) {
            pool.despawn(t.commands);
        }
    }
    for cx in rebuild {
        if let Some(ch) = sim.world.chunk(cx) {
            let mut b = QuadBatch::new();
            chunk_tiles(&mut b, &assets, &sim.world, ch);
            pools.chunks.entry(cx).or_default().sync(
                &mut t,
                &b,
                Z_WORLD_TILES,
                Z_STEP,
                &world_layer,
            );
        }
    }

    // food glow passes
    let active = frame.glow.is_some();
    if let Some(gp) = &frame.glow {
        let size = UVec2::new(gp.sw.max(1), gp.sh.max(1));
        if size != glow.size {
            glow.seed = glow_image(&mut images, size);
            glow.blur = glow_image(&mut images, size);
            glow.out = glow_image(&mut images, size);
            glow.size = size;
        }
        let (w, h) = (size.x as f64, size.y as f64);
        let full = |b: &mut QuadBatch, img: &Handle<Image>| {
            b.image_ext(
                img,
                [0.0, 0.0, w, h],
                [0.0, 0.0, w, h],
                0.0,
                0.0,
                1.0,
                1.0,
                0.0,
                C_WHITE,
                1.0,
            );
        };
        let mut across = QuadBatch::new();
        across.set_filter(true);
        across.set_blend(QuadBlend::Replace);
        across.set_blur(Some([gp.rad as f32, 0.0]));
        full(&mut across, &glow.seed);
        let mut down = QuadBatch::new();
        down.set_filter(true);
        down.set_blend(QuadBlend::Replace);
        down.set_blur(Some([0.0, gp.rad as f32]));
        full(&mut down, &glow.blur);
        for (i, b) in [&gp.seed, &across, &down].into_iter().enumerate() {
            pools.glow[i].sync(&mut t, b, 0.0, Z_STEP, &RenderLayers::layer(GLOW_LAYERS[i]));
        }
    }
    let targets = [glow.seed.clone(), glow.blur.clone(), glow.out.clone()];
    for (e, mut cam, mut proj, mut tf, mut gc) in &mut glow_cams {
        if cam.is_active != active {
            cam.is_active = active;
        }
        if !active {
            continue;
        }
        let (w, h) = (glow.size.x as f32, glow.size.y as f32);
        // written only when the surface size changes, with `area` already computed (as
        // camera::apply_cameras does; R21a)
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
        // the surface's top-left is world (0, 0) (y down)
        tf.set_if_neq(Transform::from_xyz(w * 0.5, -h * 0.5, 0.0));
        if gc.1 != targets[gc.0].id() {
            t.commands
                .entity(e)
                .insert(RenderTarget::Image(targets[gc.0].clone().into()));
            gc.1 = targets[gc.0].id();
        }
    }
}

/// HUD Draw_75's two Task 13 calls, inside its `!hud_hidden` part: `cave_perk_icons_draw` under
/// [`draw_order::CAVES`](crate::ui::draw_order::CAVES), and `cave_popups_draw` (after the HUD's
/// `draw_set_font(ui_font())`) under [`CAVE_POPUPS_DRAW_ORDER`] (between the tasks panel and the
/// markers, as Task 17 reserved).
#[allow(clippy::too_many_arguments)]
fn draw_world_gui(
    mut sim: ResMut<SimRes>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    locale: Option<Res<crate::ui::LocaleRes>>,
    fonts: Option<Res<crate::ui::fonts::UiFonts>>,
    hud: Option<Res<crate::ui::hud::HudState>>,
    mut draw: ResMut<GuiDraw>,
) {
    if hud.is_some_and(|h| h.hud_hidden) {
        draw.lists.insert(PERK_ICONS_DRAW_ORDER, Vec::new());
        draw.lists.insert(CAVE_POPUPS_DRAW_ORDER, Vec::new());
        return;
    }
    let sim = &mut sim.0;
    let now = sim.now_ms();
    let mut g = Gm::new(&assets);
    cave_perk_icons_draw(
        &mut g,
        &sim.state,
        vis("cave_icons_x") + cams.layout.ui_ox,
        vis("cave_icons_y"),
    );
    draw.lists
        .insert(PERK_ICONS_DRAW_ORDER, std::mem::take(&mut g.items));
    g.font = fonts
        .map(|f| f.font.clone())
        .unwrap_or_else(|| UiFont::Gm("Font1".into()));
    let (vx, vy) = crate::ui::hud::view_xy(&cams);
    let loc = |k: &str| -> String {
        locale
            .as_ref()
            .map(|l| l.locale.loc(k).to_string())
            .unwrap_or_else(|| k.to_string())
    };
    cave_popups_draw(
        &mut g,
        &mut sim.world.caves,
        &loc,
        now,
        vx,
        vy,
        1.0,
        cams.layout.view_w,
    );
    draw.lists
        .insert(CAVE_POPUPS_DRAW_ORDER, std::mem::take(&mut g.items));
}
