//! Surface forest (obj_forest_controller Draw_0): parallax backdrop, backdrop leaves, roots,
//! stumps, decor trees, bushes, plants, small trees and trees.
//!
//! The forest state is myco_core's (`sim.forest`, Task 5). Each frame [`forest_draw_list`] turns
//! the visible part of it into [`SpriteCmd`]s keyed by a stable [`ForestKey`] (host uid, or chunk
//! and index for the rest), and [`forest_draw`] keeps one Bevy entity per key (spawn, update,
//! despawn). Like the original Draw_0, the draw pass generates missing chunks around the view
//! with `sim.rng` (`surface_get_chunk`), between fixed steps as LÖVE does between updates.
//!
//! World coordinates throughout; the visible range is the render view (`Cameras::render_view`),
//! so zoom (T21) just shows more. Parallax phases use the game view's x, so zooming does not
//! move the backdrop.

use crate::assets::GameAssets;
use crate::camera::{world_to_bevy, Cameras};
use crate::render::layers::{self, z_at};
use crate::render::leaf_art::{build_leaf_art, LeafShapes};
use crate::render::sprite::{gm_colour_f, SpriteCmd};
use crate::sim_bridge::SimRes;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use myco_core::balance::{vis, wld};
use myco_core::forest::{
    backdrop_leaf_parallax, bush_parts, decor_tree_parts, plant_parts, stump_parts, ForestChunk,
};
use myco_core::gml;
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::RootsRect;
use std::collections::{HashMap, HashSet};

/// `parallax_layer2..4` (forest Create_0).
pub const PARALLAX_LAYERS: [f64; 3] = [0.12, 0.24, 0.42];

/// What a forest entity shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForestKind {
    Backdrop,
    BackdropLeaf,
    TreeRoots,
    Stump,
    DecorTree,
    Bush,
    Plant,
    SmallTree,
    Tree,
}

/// Stable identity of a forest entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ForestKey {
    /// backdrop layer (1..4) and tile number (`x = cam_x * (1 - parallax) + tile * width`)
    Backdrop(u8, i64),
    BackdropLeaf(u8, usize),
    TreeRoots(u64),
    SmallTreeRoots(i64, usize),
    StumpRoots(u64),
    Stump(u64),
    DecorTree(i64, usize),
    Bush(i64, usize),
    Plant(i64, usize),
    SmallTree(i64, usize),
    Tree(u64),
}

/// The forest item an entity draws (sprite name, position, flip, part index).
#[derive(Clone, Debug, PartialEq, Component)]
pub struct ForestItem {
    pub kind: ForestKind,
    pub sprite: String,
    pub x: f64,
    pub y: f64,
    pub flip: f64,
    pub part: Option<i64>,
}

/// One draw of the forest pass.
#[derive(Clone, Debug, PartialEq)]
pub struct ForestDraw {
    pub key: ForestKey,
    pub item: ForestItem,
    pub cmd: SpriteCmd,
}

/// Forest entities by key.
#[derive(Default, Resource, Deref, DerefMut)]
pub struct ForestEntities(pub HashMap<ForestKey, Entity>);

fn item(
    kind: ForestKind,
    sprite: &str,
    x: f64,
    y: f64,
    flip: f64,
    part: Option<i64>,
) -> ForestItem {
    ForestItem {
        kind,
        sprite: sprite.to_owned(),
        x,
        y,
        flip,
        part,
    }
}

/// Painter's order inside one layer.
struct Layer {
    base: f32,
    n: usize,
}

impl Layer {
    fn new(base: f32) -> Self {
        Layer { base, n: 0 }
    }
    fn z(&mut self) -> f32 {
        self.n += 1;
        z_at(self.base, self.n - 1)
    }
}

fn sprite_w(assets: &GameAssets, name: &str) -> f64 {
    assets.try_sprite(name).map_or(0.0, |d| d.size.x as f64)
}

fn sprite_h(assets: &GameAssets, name: &str) -> f64 {
    assets.try_sprite(name).map_or(0.0, |d| d.size.y as f64)
}

// port: anon__7714__gml_Object_obj_forest_controller_Create_0
/// `tree_roots_draw(d, scale, l, r)`: the root cut-out under a tree or stump.
#[allow(clippy::too_many_arguments)]
pub fn tree_roots_draw(
    rp: Option<&RootsRect>,
    x: f64,
    y: f64,
    flip: f64,
    scale: f64,
    l: f64,
    r: f64,
) -> Option<SpriteCmd> {
    let rp = rp?;
    let sx = flip * scale;
    let x0 = x - rp.ax * sx;
    let x1 = x0 + rp.w * sx;
    if x0.max(x1) >= l && x0.min(x1) <= r {
        return Some(SpriteCmd::part(
            "roots",
            rp.left,
            rp.top,
            rp.w,
            rp.h,
            x0,
            y + vis("roots_offset_y"),
            sx,
            scale,
        ));
    }
    None
}

// port: anon__11222__gml_Object_obj_forest_controller_Create_0
/// `stump_draw(d, l, r)`.
pub fn stump_draw(
    assets: &GameAssets,
    part: usize,
    x: f64,
    y: f64,
    flip: f64,
    l: f64,
    r: f64,
) -> Option<SpriteCmd> {
    let p = stump_parts().get(part)?;
    let hw = p.w * 0.5;
    if x + hw >= l && x - hw <= r {
        let h = sprite_h(assets, "stump");
        return Some(SpriteCmd::part(
            "stump",
            p.left,
            0.0,
            p.w,
            h,
            x - hw * flip,
            y - h + vis("stump_offset_y"),
            flip,
            1.0,
        ));
    }
    None
}

// port: anon__4485__gml_Object_obj_forest_controller_Create_0
/// `decor_tree_draw(d, l, r)` (parts past the list are skipped, as the original does).
pub fn decor_tree_draw(part: i64, x: f64, y: f64, flip: f64, l: f64, r: f64) -> Option<SpriteCmd> {
    let p = decor_tree_parts().get(usize::try_from(part).ok()?)?;
    let hw = p.w * 0.5;
    if x + hw >= l && x - hw <= r {
        return Some(SpriteCmd::part(
            p.sprite,
            p.left,
            p.top,
            p.w,
            p.h,
            x - hw * flip,
            y - p.h + vis("decor_tree_offset_y"),
            flip,
            1.0,
        ));
    }
    None
}

// port: anon__5590__gml_Object_obj_forest_controller_Create_0
/// `plant_draw(d, l, r)`: a cut-out of `grass`.
pub fn plant_draw(part: i64, x: f64, y: f64, flip: f64, l: f64, r: f64) -> Option<SpriteCmd> {
    let p = plant_parts().get(usize::try_from(part).ok()?)?;
    let hw = p.w * 0.5;
    if x + hw >= l && x - hw <= r {
        return Some(SpriteCmd::part(
            "grass",
            p.left,
            p.top,
            p.w,
            p.h,
            x - hw * flip,
            y - p.h + vis("plant_offset_y"),
            flip,
            1.0,
        ));
    }
    None
}

// port: anon__6104__gml_Object_obj_forest_controller_Create_0
/// `bush_draw(d, l, r)`: a cut-out of `bush_tree`.
pub fn bush_draw(part: i64, x: f64, y: f64, flip: f64, l: f64, r: f64) -> Option<SpriteCmd> {
    let p = bush_parts().get(usize::try_from(part).ok()?)?;
    let hw = p.w * 0.5;
    if x + hw >= l && x - hw <= r {
        return Some(SpriteCmd::part(
            "bush_tree",
            p.left,
            p.top,
            p.w,
            p.h,
            x - hw * flip,
            y - p.h + vis("bush_offset_y"),
            flip,
            1.0,
        ));
    }
    None
}

/// The tiles of one parallax layer covering `[l, r)`: `(tile number, floor(x))`.
fn parallax_tiles(cam_x: f64, par: f64, w: f64, l: f64, r: f64) -> Vec<(i64, f64)> {
    let mut out = Vec::new();
    if w <= 0.0 {
        return out;
    }
    let mut off = gml::gmod(cam_x * par, w);
    if off < 0.0 {
        off += w;
    }
    let first = cam_x - off; // the original starts here (l = cam_x at zoom 1)
    let base = cam_x * (1.0 - par);
    let mut xx = first + ((l - first) / w).floor() * w;
    while xx < r {
        out.push((((xx - base) / w).round() as i64, xx.floor()));
        xx += w;
    }
    out
}

/// Bottom of what forest Draw_0 draws (roots and backdrop layers) plus 256 px: below it the
/// draw returns at once.
fn forest_scene_h(sim: &Sim, assets: &GameAssets) -> f64 {
    let surface = sim.forest.surface_height;
    let scale_big = vis("roots_scale_big");
    let scale_small = vis("roots_scale_small");
    // TODO(port): DAT_141379b70 (bss constant) assumed to be 0, as in the LÖVE port
    let roots_bottom = surface
        + vis("roots_offset_y").max(0.0)
        + sprite_h(assets, "roots") * scale_big.max(scale_small);
    let layers_h = (1..=4)
        .map(|i| sprite_h(assets, &format!("background_B_layer{i}")))
        .fold(roots_bottom, f64::max);
    layers_h + 256.0
}

/// The chunk generation forest Draw_0 does for a view (`surface_get_chunk` one chunk past each
/// side, drawing from `sim.rng`). [`crate::render::draw_generate_system`] runs it after every
/// fixed step for the render view, so the game RNG is drawn inside steps, in the same order as
/// LÖVE (draw after each step); the frame's drawing then finds the chunks ready.
pub fn forest_draw_generate(sim: &mut Sim, assets: &GameAssets, rv: &ViewRect) {
    if rv.y >= forest_scene_h(sim, assets) {
        return;
    }
    let cw = sim.forest.chunk_width;
    let c0 = (rv.x / cw).floor() as i64;
    let c1 = ((rv.x + rv.w) / cw).floor() as i64;
    for c in c0 - 1..=c1 + 1 {
        sim.forest.surface_get_chunk(&mut sim.rng, c);
    }
}

// port: gml_Object_obj_forest_controller_Draw_0
/// Forest Draw_0 without the parts other tasks draw in between: the backdrop layers, then (after
/// backdrop leaves [`backdrop_leaves_draw`], rain band 0, world_draw and worms at their own
/// layers) roots, stumps, decor trees / bushes / plants, small trees and trees. `cam_x` is the
/// game view's left edge (parallax phase); `rv` the visible world rectangle. The chunk
/// generation the original's draw does here is [`forest_draw_generate`], run after each step.
pub fn forest_draw_list(
    sim: &mut Sim,
    assets: &GameAssets,
    cam_x: f64,
    rv: &ViewRect,
) -> Vec<ForestDraw> {
    let mut out = Vec::new();
    let (l, r) = (rv.x, rv.x + rv.w);
    let top = 0.0;
    if rv.y >= forest_scene_h(sim, assets) {
        return out;
    }
    let scale_big = vis("roots_scale_big");
    let scale_small = vis("roots_scale_small");

    let mut bg = Layer::new(layers::Z_BACKDROP);
    let w1 = sprite_w(assets, "background_B_layer1");
    if w1 > 0.0 {
        out.push(ForestDraw {
            key: ForestKey::Backdrop(1, 0),
            item: item(
                ForestKind::Backdrop,
                "background_B_layer1",
                l,
                top,
                1.0,
                None,
            ),
            cmd: SpriteCmd::ext("background_B_layer1", 0, l, top, rv.w / w1, 1.0, 0.0).at_z(bg.z()),
        });
    }
    for (i, par) in PARALLAX_LAYERS.iter().enumerate() {
        let name = format!("background_B_layer{}", i + 2);
        let w = sprite_w(assets, &name);
        for (tile, x) in parallax_tiles(cam_x, *par, w, l, r) {
            out.push(ForestDraw {
                key: ForestKey::Backdrop(i as u8 + 2, tile),
                item: item(ForestKind::Backdrop, &name, x, top, 1.0, None),
                cmd: SpriteCmd::ext(&name, 0, x, top, 1.0, 1.0, 0.0).at_z(bg.z()),
            });
        }
    }

    let cw = sim.forest.chunk_width;
    let c0 = (l / cw).floor() as i64;
    let c1 = (r / cw).floor() as i64;
    // the chunks were generated after the last step (`forest_draw_generate`); one missing here
    // (a view that changed since, e.g. a zoom or a freshly loaded game) is drawn next frame
    let empty = ForestChunk::default();
    let chunks = &sim.forest.chunks;
    let chunk = |c: i64| -> &ForestChunk { chunks.get(&c).unwrap_or(&empty) };

    let mut roots = Layer::new(layers::Z_FOREST_ROOTS);
    for c in c0 - 1..=c1 + 1 {
        let ch = chunk(c);
        for (i, t) in ch.small_trees.iter().enumerate() {
            if let Some(cmd) =
                tree_roots_draw(t.roots.as_ref(), t.x, t.y, t.flip, scale_small, l, r)
            {
                out.push(ForestDraw {
                    key: ForestKey::SmallTreeRoots(c, i),
                    item: item(ForestKind::TreeRoots, "roots", t.x, t.y, t.flip, None),
                    cmd: cmd.at_z(roots.z()),
                });
            }
        }
        for t in &ch.trees {
            if let Some(cmd) = tree_roots_draw(t.roots.as_ref(), t.x, t.y, t.flip, scale_big, l, r)
            {
                out.push(ForestDraw {
                    key: ForestKey::TreeRoots(t.uid),
                    item: item(ForestKind::TreeRoots, "roots", t.x, t.y, t.flip, None),
                    cmd: cmd.at_z(roots.z()),
                });
            }
        }
        for s in &ch.stumps {
            let Some(rt) = &s.roots else { continue };
            if let Some(cmd) =
                tree_roots_draw(rt.roots.as_ref(), rt.x, rt.y, rt.flip, scale_big, l, r)
            {
                out.push(ForestDraw {
                    key: ForestKey::StumpRoots(s.uid),
                    item: item(ForestKind::TreeRoots, "roots", rt.x, rt.y, rt.flip, None),
                    cmd: cmd.at_z(roots.z()),
                });
            }
        }
    }

    let mut stumps = Layer::new(layers::Z_FOREST_STUMPS);
    for c in c0 - 1..=c1 + 1 {
        for s in &chunk(c).stumps {
            if let Some(cmd) = stump_draw(assets, s.part, s.x, s.y, s.flip, l, r) {
                out.push(ForestDraw {
                    key: ForestKey::Stump(s.uid),
                    item: item(
                        ForestKind::Stump,
                        "stump",
                        s.x,
                        s.y,
                        s.flip,
                        Some(s.part as i64),
                    ),
                    cmd: cmd.at_z(stumps.z()),
                });
            }
        }
    }

    let mut under = Layer::new(layers::Z_FOREST_UNDERGROWTH);
    for c in c0 - 1..=c1 + 1 {
        let ch = chunk(c);
        for (i, d) in ch.decor_trees.iter().enumerate() {
            if let Some(cmd) = decor_tree_draw(d.part, d.x, d.y, d.flip, l, r) {
                let sprite = cmd.sprite_name().to_owned();
                out.push(ForestDraw {
                    key: ForestKey::DecorTree(c, i),
                    item: item(
                        ForestKind::DecorTree,
                        &sprite,
                        d.x,
                        d.y,
                        d.flip,
                        Some(d.part),
                    ),
                    cmd: cmd.at_z(under.z()),
                });
            }
        }
        for (i, d) in ch.bushes.iter().enumerate() {
            if let Some(cmd) = bush_draw(d.part, d.x, d.y, d.flip, l, r) {
                out.push(ForestDraw {
                    key: ForestKey::Bush(c, i),
                    item: item(
                        ForestKind::Bush,
                        "bush_tree",
                        d.x,
                        d.y,
                        d.flip,
                        Some(d.part),
                    ),
                    cmd: cmd.at_z(under.z()),
                });
            }
        }
        for (i, d) in ch.plants.iter().enumerate() {
            if let Some(cmd) = plant_draw(d.part, d.x, d.y, d.flip, l, r) {
                out.push(ForestDraw {
                    key: ForestKey::Plant(c, i),
                    item: item(ForestKind::Plant, "grass", d.x, d.y, d.flip, Some(d.part)),
                    cmd: cmd.at_z(under.z()),
                });
            }
        }
    }

    let mut small = Layer::new(layers::Z_FOREST_SMALL_TREES);
    for c in c0..=c1 {
        for (i, t) in chunk(c).small_trees.iter().enumerate() {
            let w = sprite_w(assets, &t.sprite);
            if t.x >= l - w && t.x <= r + w {
                out.push(ForestDraw {
                    key: ForestKey::SmallTree(c, i),
                    item: item(ForestKind::SmallTree, &t.sprite, t.x, t.y, t.flip, None),
                    cmd: SpriteCmd::ext(&t.sprite, 0, t.x, t.y, t.flip, 1.0, 0.0).at_z(small.z()),
                });
            }
        }
    }
    let mut trees = Layer::new(layers::Z_FOREST_TREES);
    for c in c0..=c1 {
        for t in &chunk(c).trees {
            let w = sprite_w(assets, &t.sprite);
            if t.x >= l - w && t.x <= r + w {
                out.push(ForestDraw {
                    key: ForestKey::Tree(t.uid),
                    item: item(ForestKind::Tree, &t.sprite, t.x, t.y, t.flip, None),
                    cmd: SpriteCmd::ext(&t.sprite, 0, t.x, t.y, t.flip, 1.0, 0.0).at_z(trees.z()),
                });
            }
        }
    }
    out
}

/// One backdrop leaf to draw: a tinted silhouette (`leaf_art_frame(l).shape`).
#[derive(Clone, Debug, PartialEq)]
pub struct BackdropLeafDraw {
    pub key: ForestKey,
    pub change_from: f64,
    pub change_frame: f64,
    pub face: f64,
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub angle: f64,
    /// BGR
    pub col: f64,
    pub alpha: f64,
    pub z: f32,
}

// port: backdrop_leaves_draw
/// `backdrop_leaves_draw(cam_x, view_y, view_w, view_h)`; culling against the render view.
pub fn backdrop_leaves_draw(sim: &Sim, cam_x: f64, rv: &ViewRect) -> Vec<BackdropLeafDraw> {
    let mut out = Vec::new();
    let Some(bands) = &sim.forest.backdrop_leaves else {
        return out;
    };
    let floor_y = wld("surface_height_px");
    if rv.y > floor_y || rv.y + rv.h < 0.0 {
        return out;
    }
    let col = vis("backdrop_leaf_colour");
    let mut layer = Layer::new(layers::Z_BACKDROP_LEAVES);
    for (band, arr) in bands.iter().enumerate() {
        let par = backdrop_leaf_parallax(band);
        for (i, lf) in arr.iter().enumerate() {
            let wx = cam_x + (lf.lx - cam_x * par);
            if wx < rv.x - 80.0 || wx > rv.x + rv.w + 80.0 {
                continue;
            }
            out.push(BackdropLeafDraw {
                key: ForestKey::BackdropLeaf(band as u8, i),
                change_from: lf.change_from,
                change_frame: lf.change_frame,
                face: lf.face,
                x: gml::round(wx),
                y: gml::round(lf.y),
                scale: lf.scale,
                angle: lf.angle,
                col,
                alpha: lf.alpha.clamp(0.0, 1.0),
                z: layer.z(),
            });
        }
    }
    out
}

type ForestQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Sprite,
        &'static mut Anchor,
        &'static mut Transform,
        &'static mut ForestItem,
    ),
>;

fn upsert(
    commands: &mut Commands,
    ents: &mut ForestEntities,
    q: &mut ForestQuery,
    key: ForestKey,
    bundle: (Sprite, Anchor, Transform),
    it: ForestItem,
) {
    if let Some(&e) = ents.get(&key) {
        if let Ok((mut s, mut a, mut t, mut i)) = q.get_mut(e) {
            // only touch what changed (change detection drives re-extraction)
            s.set_if_neq(bundle.0);
            a.set_if_neq(bundle.1);
            t.set_if_neq(bundle.2);
            i.set_if_neq(it);
            return;
        }
    }
    let e = commands.spawn((bundle.0, bundle.1, bundle.2, it)).id();
    ents.insert(key, e);
}

/// Update: sync the forest entities with `sim.forest` for the current view.
pub fn forest_draw(
    mut commands: Commands,
    mut sim: ResMut<SimRes>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    shapes: Option<Res<LeafShapes>>,
    mut ents: ResMut<ForestEntities>,
    mut q: ForestQuery,
) {
    let rv = cams.render_view();
    let cam_x = cams.game_view.x;
    let mut seen = HashSet::new();
    for d in forest_draw_list(&mut sim.0, &assets, cam_x, &rv) {
        if let Some(b) = d.cmd.to_bevy(&assets) {
            seen.insert(d.key);
            upsert(&mut commands, &mut ents, &mut q, d.key, b, d.item);
        }
    }
    if let Some(shapes) = shapes {
        for d in backdrop_leaves_draw(&sim.0, cam_x, &rv) {
            let Some(image) = shapes.frame(d.change_from, d.change_frame, d.face) else {
                continue;
            };
            let sprite = Sprite {
                image: image.clone(),
                color: gm_colour_f(d.col, d.alpha),
                ..default()
            };
            let tf = Transform {
                translation: world_to_bevy(d.x, d.y).extend(d.z),
                rotation: Quat::from_rotation_z((d.angle as f32).to_radians()),
                scale: Vec3::new(d.scale as f32, d.scale as f32, 1.0),
            };
            seen.insert(d.key);
            let it = item(ForestKind::BackdropLeaf, "leaf", d.x, d.y, 1.0, None);
            upsert(
                &mut commands,
                &mut ents,
                &mut q,
                d.key,
                (sprite, Anchor::CENTER, tf),
                it,
            );
        }
    }
    ents.retain(|k, e| {
        let keep = seen.contains(k);
        if !keep {
            commands.entity(*e).despawn();
        }
        keep
    });
}

/// Forest rendering: leaf art at startup, the forest sync every frame.
pub struct ForestRenderPlugin;

impl Plugin for ForestRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ForestEntities>()
            .add_systems(Startup, build_leaf_art)
            .add_systems(Update, forest_draw);
    }
}
