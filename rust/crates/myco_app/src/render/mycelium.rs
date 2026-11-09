//! Mycelium rendering (Task 14): obj_mycelium_controller Draw_0 and the world part of Draw_64.
//!
//! **Baked hyphae.** A finished growth is queued by the logic in `MyceliumRt::dirty_growths`
//! (`myc_unbaked`, ruling R9a). [`tiles_step`] (run after every fixed step, where the original
//! calls `myc_tiles_step` at the end of Step_0) pops and bakes them: [`growth_bake`] turns each
//! strand's rasterized cells into merged span quads ([`write_branch_spans`]), files the growth
//! under its 1024 px tile (`myc_tile_add`, kept in core) and shows it as its own mesh until
//! [`tile_merge`] appends it to the tile mesh 30 frames later; a tile mesh over 256 KiB is
//! closed and a new one started. The CPU side lives in [`MycBake`] and reports mesh changes as
//! [`MeshOp`]s, which [`apply_mesh_ops`] turns into `Mesh2d` entities (one per pending growth,
//! tile mesh, closed tile mesh and live strand chunk).
//!
//! **Live strands.** A growing strand shows `ceil(p · n)` of its cells: full 96-cell chunks as
//! cached meshes ([`cells_buffer`]), the rest rebuilt every frame ([`live_strands`]).
//!
//! **World space.** The original draws the hyphae, spores and knots in Draw_64 on the GUI
//! surface through a world matrix (`-floor(camera)`), i.e. 1:1 world pixels over the whole world.
//! Here they are world-space meshes above every world layer (`layers::Z_MYC_*`) so the zoom
//! (Task 21) applies; at zoom 1 the picture is the same. The hyphae entities are also on the
//! glow buffer's layer (`mycelium_glow::GLOW_SEED_LAYER`).
//!
//! The aim preview and the in-world buttons (also Draw_64) are GUI-space: `ui::myc_prompts`.

use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::render::blend::{colour_mesh, empty_colour_mesh, GmBlend, GmBlendMode, Vert};
use crate::render::layers::{z_at, Z_MYCELIUM, Z_MYC_KNOTS, Z_MYC_STRANDS};
use crate::render::mycelium_glow::GLOW_SEED_LAYER;
use crate::render::sprite::{gm_colour_f, SpriteCmd};
use crate::sim_bridge::{SimRes, SimSet};
use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use myco_core::balance::vis;
use myco_core::events::{AudioCall, SimEvent};
use myco_core::mycelium::{self as myc, shot, Cell, Knot, MyceliumRt};
use myco_core::sim::Sim;
use std::collections::HashMap;

/// `myc_core_color`: the baked hyphae.
pub const MYC_CORE_COLOR: f64 = 14873582.0;
/// Draw_64's `col` (0xe2f3ee): live strands, spores, knots.
pub const MYC_COL: f64 = 0xe2f3ee as f64;
/// Draw_64's `col_dark` (0x324036): the knot's centre.
pub const MYC_COL_DARK: f64 = 0x324036 as f64;
/// `global.myc_vformat`: 2D float position + colour = 12 bytes per vertex.
pub const VERTEX_BYTES: usize = 12;
/// A tile vertex buffer is closed once appending would pass this size (`myc_tile_merge`).
pub const TILE_BUFFER_MAX: usize = 262144;
/// Cells per cached live-strand chunk (`myc_cells_buffer` calls in `myc_draw_strands`).
pub const CHUNK_CELLS: usize = 96;
/// Frames a tile's pending growths wait before they are merged (`myc_tiles_step`).
pub const MERGE_DELAY: f64 = 30.0;

/// A vertex in world pixels (y down).
pub type V2 = [f32; 2];

/// A growth's `vbuf` state: -1 unbaked, -2 baked to nothing, a buffer (pending in its tile),
/// -3 merged into its tile's mesh.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Vbuf {
    #[default]
    Unbaked,
    Empty,
    Pending,
    Merged,
}

/// Bake state of one growth: `vbuf` and `raw` (the vertex bytes kept for the merge).
#[derive(Clone, Debug, Default)]
pub struct GrowthBuf {
    pub vbuf: Vbuf,
    pub raw: Option<Vec<V2>>,
}

/// GPU side of a `MycTile` (`___struct___513`'s `raw`, `size`, `vbuf`, `closed`).
#[derive(Clone, Debug, Default)]
pub struct TileBuf {
    /// vertices of the open tile mesh
    pub raw: Vec<V2>,
    /// `size`: bytes in the open tile mesh
    pub size: usize,
    /// the open tile mesh exists
    pub vbuf: bool,
    /// number of closed (full) tile meshes
    pub closed: usize,
}

/// Which mesh entity a [`MeshOp`] addresses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MeshKey {
    /// a baked growth waiting in its tile (`e.vbuf`)
    Growth(usize),
    /// the open mesh of tile `ti` (`t.vbuf`)
    Tile(usize),
    /// closed mesh `k` of tile `ti` (`t.closed[k]`)
    Closed(usize, usize),
    /// live-strand chunk `ci` of branch `bi` of growth `gi` (`br.chunks[ci]`)
    Chunk(usize, usize, usize),
}

/// A change to the hyphae meshes.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshOp {
    /// create or replace the mesh (triangle list, core colour)
    Set(MeshKey, Vec<V2>),
    Remove(MeshKey),
    /// drop every baked mesh (growths, tiles; not live chunks): `myc_tiles_clear`
    Clear,
}

/// CPU side of the baked hyphae (ruling R9a: the GPU half of `myc_tiles` / `myc_unbaked`).
#[derive(Resource, Default, Debug)]
pub struct MycBake {
    /// per growth index
    pub growths: Vec<GrowthBuf>,
    /// per `MyceliumRt::tile_list` index
    pub tiles: Vec<TileBuf>,
    /// built live chunks per (growth, branch)
    pub chunks: HashMap<(usize, usize), usize>,
    /// mesh changes not yet applied to entities
    pub ops: Vec<MeshOp>,
    /// growths the logic had at the last bake step (they only grow within one game)
    pub seen_growths: usize,
    /// ids of the growths whose hyphae sound may still play (for CleanUp on a replaced Sim)
    pub sounding: Vec<u64>,
}

impl MycBake {
    fn growth(&mut self, gi: usize) -> &mut GrowthBuf {
        if self.growths.len() <= gi {
            self.growths.resize_with(gi + 1, GrowthBuf::default);
        }
        &mut self.growths[gi]
    }

    fn tile(&mut self, ti: usize) -> &mut TileBuf {
        if self.tiles.len() <= ti {
            self.tiles.resize_with(ti + 1, TileBuf::default);
        }
        &mut self.tiles[ti]
    }

    /// The `vbuf` state of growth `gi`.
    pub fn vbuf(&self, gi: usize) -> Vbuf {
        self.growths.get(gi).map_or(Vbuf::Unbaked, |g| g.vbuf)
    }
}

/// GML integer division of a whole number by 2 (`gm.div(w, 2)`).
fn half(w: f64) -> f64 {
    (w / 2.0).trunc()
}

// port: anon__77430__gml_Object_obj_mycelium_controller_Create_0
/// `myc_span_quad(vb, x1, y1, x2, y2)`: one rectangle as two triangles.
pub fn span_quad(vb: &mut Vec<V2>, x1: f64, y1: f64, x2: f64, y2: f64) {
    let (x1, y1, x2, y2) = (x1 as f32, y1 as f32, x2 as f32, y2 as f32);
    vb.extend_from_slice(&[[x1, y1], [x2, y1], [x1, y2], [x2, y1], [x2, y2], [x1, y2]]);
}

// port: anon__78411__gml_Object_obj_mycelium_controller_Create_0
/// `myc_write_branch_spans(vb, cells, horiz)`: the union of the strand's square cells as
/// 1-px-thick spans, one per column (`horiz`) or row; overlapping cells in a line merge into one
/// span. False when there are no cells.
pub fn write_branch_spans(vb: &mut Vec<V2>, cells: &[Cell], horiz: bool) -> bool {
    if cells.is_empty() {
        return false;
    }
    let along = |c: &Cell| if horiz { c.x } else { c.y };
    let across = |c: &Cell| if horiz { c.y } else { c.x };
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    for c in cells {
        let s = along(c) - half(c.w);
        lo = lo.min(s);
        hi = hi.max(s + c.w);
    }
    let span = (hi - lo).max(0.0) as usize;
    // the original fills amin with a runtime constant; any value works, amax = -inf marks empty
    let mut amin = vec![0.0f64; span];
    let mut amax = vec![f64::NEG_INFINITY; span];
    let emit = |vb: &mut Vec<V2>, k: usize, a0: f64, a1: f64| {
        let p = lo + k as f64;
        if horiz {
            span_quad(vb, p, a0, p + 1.0, a1);
        } else {
            span_quad(vb, a0, p, a1, p + 1.0);
        }
    };
    for c in cells {
        let w = c.w;
        let hw = half(w);
        let a0 = across(c) - hw;
        let a1 = a0 + w;
        let k0 = (along(c) - hw - lo) as i64;
        for k in k0..k0 + w as i64 {
            let k = k as usize;
            if a0 <= amax[k] && a1 >= amin[k] {
                amin[k] = amin[k].min(a0);
                amax[k] = amax[k].max(a1);
            } else {
                if amax[k] > f64::NEG_INFINITY {
                    emit(vb, k, amin[k], amax[k]);
                }
                amin[k] = a0;
                amax[k] = a1;
            }
        }
    }
    for k in 0..span {
        if amax[k] > f64::NEG_INFINITY {
            emit(vb, k, amin[k], amax[k]);
        }
    }
    true
}

// port: anon__55774__gml_Object_obj_mycelium_controller_Create_0
/// `myc_cells_buffer(cells, from, to)`: cells `from..to` as squares; `None` (-2) when empty.
pub fn cells_buffer(cells: &[Cell], from: usize, to: usize) -> Option<Vec<V2>> {
    if to <= from {
        return None;
    }
    let mut vb = Vec::with_capacity((to - from) * 6);
    for c in &cells[from..to.min(cells.len())] {
        let h = half(c.w);
        let (x1, y1) = (c.x - h, c.y - h);
        span_quad(&mut vb, x1, y1, x1 + c.w, y1 + c.w);
    }
    Some(vb)
}

// port: anon__57044__gml_Object_obj_mycelium_controller_Create_0
/// `myc_growth_free_chunks(g)`: drop a growth's cached live-strand chunks.
pub fn growth_free_chunks(m: &MyceliumRt, b: &mut MycBake, gi: usize) {
    let Some(g) = m.growths.get(gi) else { return };
    for bi in 0..g.branches.len() {
        if let Some(n) = b.chunks.remove(&(gi, bi)) {
            for ci in 0..n {
                b.ops.push(MeshOp::Remove(MeshKey::Chunk(gi, bi, ci)));
            }
        }
    }
}

// port: anon__80501__gml_Object_obj_mycelium_controller_Create_0
/// `myc_growth_bake(e)`: an unbaked growth's strands → one span mesh (rasterizing strands that
/// were built without cells), filed under its tile; the cells are dropped afterwards.
pub fn growth_bake(m: &mut MyceliumRt, b: &mut MycBake, gi: usize) {
    if gi >= m.growths.len() || b.vbuf(gi) != Vbuf::Unbaked {
        return;
    }
    let mut vb = Vec::new();
    let mut any = false;
    for br in &mut m.growths[gi].branches {
        let cells = br
            .cells
            .take()
            .unwrap_or_else(|| shot::rasterize_branch(&br.points, br.width));
        let tall = (br.by2 - br.by1) > (br.bx2 - br.bx1);
        if write_branch_spans(&mut vb, &cells, tall) {
            any = true;
        }
    }
    if !any {
        b.growth(gi).vbuf = Vbuf::Empty;
    } else {
        b.ops.push(MeshOp::Set(MeshKey::Growth(gi), vb.clone()));
        let g = b.growth(gi);
        g.raw = Some(vb);
        g.vbuf = Vbuf::Pending;
        myc::tile_add(m, gi);
    }
}

// port: anon__82902__gml_Object_obj_mycelium_controller_Create_0
/// `myc_tile_merge(t)`: append the tile's pending growths to its mesh (closing the mesh first
/// when it would pass [`TILE_BUFFER_MAX`]) and drop their own meshes.
pub fn tile_merge(m: &mut MyceliumRt, b: &mut MycBake, ti: usize) {
    let pend = std::mem::take(&mut m.tile_list[ti].pending);
    let total: usize = pend
        .iter()
        .map(|&gi| {
            b.growths
                .get(gi)
                .and_then(|g| g.raw.as_ref())
                .map_or(0, |r| r.len() * VERTEX_BYTES)
        })
        .sum();
    if total > 0 {
        let t = b.tile(ti);
        let mut close = None;
        if t.size > 0 && t.size + total > TILE_BUFFER_MAX {
            close = Some((t.closed, std::mem::take(&mut t.raw)));
            t.closed += 1;
            t.size = 0;
            t.vbuf = false;
        }
        if let Some((k, v)) = close {
            b.ops.push(MeshOp::Set(MeshKey::Closed(ti, k), v));
        }
        for &gi in &pend {
            if let Some(r) = b.growths.get_mut(gi).and_then(|g| g.raw.take()) {
                b.tile(ti).raw.extend(r);
            }
        }
        let t = b.tile(ti);
        t.size = t.raw.len() * VERTEX_BYTES;
        t.vbuf = true;
        let v = t.raw.clone();
        b.ops.push(MeshOp::Set(MeshKey::Tile(ti), v));
    }
    for &gi in &pend {
        if b.vbuf(gi) == Vbuf::Pending {
            b.ops.push(MeshOp::Remove(MeshKey::Growth(gi)));
        }
        b.growth(gi).vbuf = Vbuf::Merged;
    }
}

// port: anon__85453__gml_Object_obj_mycelium_controller_Create_0
/// `myc_tiles_clear`: drop every tile mesh and the pending growths' meshes (those growths go back
/// to unbaked).
pub fn tiles_clear(m: &mut MyceliumRt, b: &mut MycBake) {
    for t in &m.tile_list {
        for &gi in &t.pending {
            let g = b.growth(gi);
            g.raw = None;
            g.vbuf = Vbuf::Unbaked;
        }
    }
    m.tiles.clear();
    m.tile_list.clear();
    b.tiles.clear();
    b.ops.push(MeshOp::Clear);
}

/// The logic's mycelium is not the one baked so far: a new `Sim` (fewer growths or tiles than
/// already seen; both only grow within one game).
pub fn replaced(m: &MyceliumRt, b: &MycBake) -> bool {
    m.growths.len() < b.seen_growths
        || m.tile_list.len() < b.tiles.len()
        || m.growths.len() < b.growths.len()
}

/// Drop every baked mesh and live chunk when the logic's mycelium was reset under us: a load
/// (`tiles_reset`, set by `on_load`) or a whole new `Sim` (restart to the title, a loaded slot),
/// which shows as fewer growths or tiles than the bake state knows (both only grow in a game).
pub fn sync_reset(m: &mut MyceliumRt, b: &mut MycBake) {
    if !(m.tiles_reset || replaced(m, b)) {
        return;
    }
    m.tiles_reset = false;
    b.seen_growths = m.growths.len();
    tiles_clear(m, b);
    b.growths.clear();
    for ((gi, bi), n) in std::mem::take(&mut b.chunks) {
        for ci in 0..n {
            b.ops.push(MeshOp::Remove(MeshKey::Chunk(gi, bi, ci)));
        }
    }
}

// port: anon__84786__gml_Object_obj_mycelium_controller_Create_0
/// `myc_tiles_step` (end of Step_0): advance `myc_frame`, bake every queued growth (the time
/// budget never runs out here, like an infinitely fast machine) and merge the first tile whose
/// pending growths waited [`MERGE_DELAY`] frames.
///
/// A logic reset (`tiles_reset`, set by load) clears everything first, as `load_game`'s
/// `myc_tiles_clear` does; the old growths' meshes and live chunks go with it.
pub fn tiles_step(m: &mut MyceliumRt, b: &mut MycBake) {
    sync_reset(m, b);
    m.frame += 1.0;
    while let Some(gi) = m.dirty_growths.pop() {
        // myc_finish_growth frees the live chunks before queueing the growth
        growth_free_chunks(m, b, gi);
        growth_bake(m, b, gi);
    }
    if let Some(ti) = m
        .tile_list
        .iter()
        .position(|t| !t.pending.is_empty() && m.frame - t.since >= MERGE_DELAY)
    {
        tile_merge(m, b, ti);
    }
    b.seen_growths = m.growths.len();
}

/// The live (unfinished) strands of `myc_draw_strands` inside `view` = (x1, y1, x2, y2): the
/// shown cells past the full chunks as squares; full 96-cell chunks are built once as
/// [`MeshKey::Chunk`] meshes.
pub fn live_strands(m: &MyceliumRt, b: &mut MycBake, view: (f64, f64, f64, f64)) -> Vec<V2> {
    let (vx1, vy1, vx2, vy2) = view;
    let mut out = Vec::new();
    for &gi in &m.active {
        let Some(e) = m.growths.get(gi) else { continue };
        if e.finished {
            continue;
        }
        for (bi, br) in e.branches.iter().enumerate() {
            if br.bx2 < vx1 || br.bx1 > vx2 || br.by2 < vy1 || br.by1 > vy2 {
                continue;
            }
            let Some(cells) = &br.cells else { continue };
            let n = cells.len();
            if n == 0 {
                continue;
            }
            let p = ((e.age - br.start) / e.duration).clamp(0.0, 1.0);
            if p <= 0.0 {
                continue;
            }
            let shown = ((p * n as f64).ceil() as usize).min(n);
            let nfull = shown / CHUNK_CELLS;
            if nfull > 0 {
                let have = b.chunks.entry((gi, bi)).or_insert(0);
                while *have < nfull {
                    let ci = *have;
                    if let Some(v) = cells_buffer(cells, ci * CHUNK_CELLS, (ci + 1) * CHUNK_CELLS) {
                        b.ops.push(MeshOp::Set(MeshKey::Chunk(gi, bi, ci), v));
                    }
                    *have += 1;
                }
            }
            for c in &cells[nfull * CHUNK_CELLS..shown] {
                let hw = half(c.w);
                let (x0, y0) = (c.x - hw, c.y - hw);
                span_quad(&mut out, x0, y0, x0 + c.w, y0 + c.w);
            }
        }
    }
    out
}

// port: anon__62655__gml_Object_obj_mycelium_controller_Create_0
/// `myc_draw_strands(col, view)`: bake queued growths that are in view now (the rest wait for
/// `myc_tiles_step`), then the live strands. Tile and pending meshes are entities already; Bevy
/// culls them by their bounds as the original culls by tile box.
///
/// `leaf_tendrils_draw(col)`, called at the end of the original, is Task 15's.
pub fn draw_strands(m: &mut MyceliumRt, b: &mut MycBake, view: (f64, f64, f64, f64)) -> Vec<V2> {
    let (vx1, vy1, vx2, vy2) = view;
    sync_reset(m, b);
    let queued: Vec<usize> = m.dirty_growths.clone();
    for gi in queued {
        if b.vbuf(gi) != Vbuf::Unbaked {
            continue;
        }
        let Some(e) = m.growths.get(gi) else { continue };
        if !(e.bx2 < vx1 || e.bx1 > vx2 || e.by2 < vy1 || e.by1 > vy2) {
            growth_bake(m, b, gi);
        }
    }
    live_strands(m, b, view)
}

/// `shroom_grow_scale(m)` (Task 9's row, scr_life_b_1): the back-out pop of a new smallshroom.
/// TODO(T9): call the canonical port once it lands.
fn shroom_grow_scale(sim: &Sim, k: &Knot) -> f64 {
    let Some(born) = k.born else { return 1.0 };
    let t = ((sim.now_ms() - born) / vis("shroom_grow_ms").max(1.0)).clamp(0.0, 1.0);
    let c1 = 1.70158;
    let u = t - 1.0;
    let c3 = c1 + 1.0;
    (1.0 + c3 * u * u * u + c1 * u * u).max(0.0)
}

// port: gml_Object_obj_mycelium_controller_Draw_0
/// obj_mycelium_controller Draw_0: the origin mushroom and the spore knots' smallshrooms (culled
/// to the view ± 64 px). Nothing while the intro runs.
pub fn draw_0(sim: &Sim, view_x: f64, view_w: f64) -> Vec<SpriteCmd> {
    let mut out = Vec::new();
    if sim.intro_active {
        return out;
    }
    let m = &sim.mycelium;
    out.push(SpriteCmd::ext(
        "originshroom",
        0,
        m.x.floor(),
        m.y.floor(),
        1.0,
        1.0,
        0.0,
    ));
    for k in &m.knots {
        let Some(sh) = &k.shroom else { continue };
        if k.x >= view_x - 64.0 && k.x <= view_x + view_w + 64.0 {
            let r = myc::knots::smallshroom_rect(sh);
            let s = vis("spore_shroom_scale") * shroom_grow_scale(sim, k);
            out.push(SpriteCmd::part(
                r.sprite,
                r.left,
                r.top,
                r.width,
                r.height,
                k.x.floor() - r.width * s * 0.5,
                k.y.floor() - r.height * s,
                s,
                s,
            ));
        }
    }
    for (i, c) in out.iter_mut().enumerate() {
        c.z = z_at(Z_MYCELIUM, i);
    }
    out
}

fn rect_fill(v: &mut Vec<Vert>, x1: f64, y1: f64, x2: f64, y2: f64, col: f64, alpha: f64) {
    // draw_rectangle(.., false) covers the far pixel
    let (x2, y2) = (x2 + 1.0, y2 + 1.0);
    let p = |x, y| Vert { x, y, col, alpha };
    v.extend_from_slice(&[
        p(x1, y1),
        p(x2, y1),
        p(x1, y2),
        p(x2, y1),
        p(x2, y2),
        p(x1, y2),
    ]);
}

/// Draw_64's spores and knots, in world pixels (the original's GUI pixels minus the camera
/// offset). `view` = (x1, y1, x2, y2) is the visible world rectangle.
pub fn spores_and_knots(m: &MyceliumRt, view: (f64, f64, f64, f64)) -> Vec<Vert> {
    let (vx1, vy1, vx2, vy2) = view;
    let mut v = Vec::new();
    for (i, sp) in m.spores.iter().enumerate() {
        let (px, py) = (sp.x.floor(), sp.y.floor());
        if px < vx1 - 8.0 || px > vx2 + 8.0 || py < vy1 - 8.0 || py > vy2 + 8.0 {
            continue;
        }
        let r = (i % 2) as f64 + 1.0;
        let a = (sp.life / 80.0).clamp(0.35, 1.0);
        rect_fill(&mut v, px, py, px + r, py + r, MYC_COL, a);
    }
    let ks = vis("knot_scale");
    let k1 = (3.5 * ks - 0.5).floor();
    let k2 = (2.5 * ks - 0.5).floor();
    let k3 = (1.5 * ks - 0.5).floor();
    for kn in &m.knots {
        let (kx, ky) = (kn.x.floor(), kn.y.floor());
        if kx < vx1 - 32.0 || kx > vx2 + 32.0 || ky < vy1 - 32.0 || ky > vy2 + 32.0 {
            continue;
        }
        rect_fill(&mut v, kx - k1, ky - k2, kx + k1, ky + k2, MYC_COL, 1.0);
        rect_fill(&mut v, kx - k2, ky - k1, kx + k2, ky + k1, MYC_COL, 1.0);
        rect_fill(
            &mut v,
            kx - k3,
            ky - k3,
            kx + k3,
            ky + k3,
            MYC_COL_DARK,
            1.0,
        );
    }
    v
}

// port: gml_Object_obj_mycelium_controller_CleanUp_0
/// obj_mycelium_controller CleanUp_0 (the controller goes away: game restart / leaving the game):
/// stop the hyphae sounds, drop every tile, chunk and growth mesh. The glow buffers are dropped
/// with their resource (`mycelium_glow`).
pub fn clean_up(sim: &mut Sim, b: &mut MycBake) {
    tiles_clear(&mut sim.mycelium, b);
    let n = sim.mycelium.growths.len();
    for gi in 0..n {
        let g = &sim.mycelium.growths[gi];
        if g.sound.is_some() {
            let id = g.id;
            sim.emit(SimEvent::Audio(AudioCall::HyphaeStop { id }));
        }
        growth_free_chunks(&sim.mycelium, b, gi);
        if b.vbuf(gi) == Vbuf::Pending {
            b.ops.push(MeshOp::Remove(MeshKey::Growth(gi)));
        }
        if gi < b.growths.len() {
            b.growths[gi] = GrowthBuf::default();
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// Entities and materials of the mycelium drawing.
#[derive(Resource)]
pub struct MycRender {
    strand_material: Handle<GmBlend>,
    meshes: HashMap<MeshKey, Entity>,
    live: Handle<Mesh>,
    knots: Handle<Mesh>,
    sprites: Vec<Entity>,
}

/// Layers of every hyphae mesh: the world camera and the glow buffer.
pub fn strand_layers() -> RenderLayers {
    RenderLayers::from_layers(&[0, GLOW_SEED_LAYER])
}

fn strand_colour() -> [f32; 4] {
    gm_colour_f(MYC_CORE_COLOR, 1.0).to_linear().to_f32_array()
}

/// Fill a triangle-list mesh with world-pixel vertices (y down) in the core colour; an empty
/// list becomes one degenerate triangle (the mesh allocator warns about empty meshes).
pub fn fill_strand_mesh(mesh: &mut Mesh, v: &[V2]) {
    let deg = [[0.0f32, 0.0]; 3];
    let v = if v.is_empty() { &deg[..] } else { v };
    let pos: Vec<[f32; 3]> = v.iter().map(|p| [p[0], -p[1], 0.0]).collect();
    let col = vec![strand_colour(); v.len()];
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
}

fn strand_mesh(v: &[V2]) -> Mesh {
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    fill_strand_mesh(&mut m, v);
    m
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GmBlend>>,
) {
    let strand_material = materials.add(GmBlend::new(GmBlendMode::Opaque));
    let knot_material = materials.add(GmBlend::new(GmBlendMode::Alpha));
    let mut live_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    fill_strand_mesh(&mut live_mesh, &[]);
    let live = meshes.add(live_mesh);
    commands.spawn((
        Name::new("myc live strands"),
        Mesh2d(live.clone()),
        MeshMaterial2d(strand_material.clone()),
        Transform::from_xyz(0.0, 0.0, Z_MYC_STRANDS),
        strand_layers(),
        NoFrustumCulling,
    ));
    let knots = meshes.add(empty_colour_mesh());
    commands.spawn((
        Name::new("myc spores and knots"),
        Mesh2d(knots.clone()),
        MeshMaterial2d(knot_material),
        Transform::from_xyz(0.0, 0.0, Z_MYC_KNOTS),
        NoFrustumCulling,
    ));
    commands.insert_resource(MycRender {
        strand_material,
        meshes: HashMap::new(),
        live,
        knots,
        sprites: Vec::new(),
    });
}

/// Keep the autopilot's `wmove` origin on the mycelium controller (`MyceliumAnchor`, Task 4).
fn sync_anchor(sim: Res<SimRes>, mut anchor: ResMut<crate::autopilot::MyceliumAnchor>) {
    let p = bevy::math::DVec2::new(sim.0.mycelium.x, sim.0.mycelium.y);
    if anchor.0 != p {
        anchor.0 = p;
    }
}

/// `myc_tiles_step` after every fixed step (the end of the original's Step_0).
///
/// A replaced `Sim` (restart to the title, a loaded slot: `Sim::load_game` builds a new `Sim`, so
/// its `on_load` never sees the old growths) means the old controller went away: its CleanUp_0
/// stops the hyphae sounds that may still play. A second stop of an id is harmless.
pub fn tiles_step_system(mut sim: ResMut<SimRes>, mut bake: ResMut<MycBake>) {
    if replaced(&sim.0.mycelium, &bake) || sim.0.mycelium.tiles_reset {
        for id in std::mem::take(&mut bake.sounding) {
            sim.0.emit(SimEvent::Audio(AudioCall::HyphaeStop { id }));
        }
    }
    tiles_step(&mut sim.0.mycelium, &mut bake);
    let m = &sim.0.mycelium;
    bake.sounding = m
        .growths
        .iter()
        .filter(|g| g.sound.is_some())
        .map(|g| g.id)
        .collect();
}

/// The visible world rectangle plus Draw_64's 64 px margin.
fn draw_view(cams: &Cameras) -> (f64, f64, f64, f64) {
    let v = cams.render_view();
    (v.x - 64.0, v.y - 64.0, v.x + v.w + 64.0, v.y + v.h + 64.0)
}

// port: gml_Object_obj_mycelium_controller_Draw_64
/// Draw_64's world part every frame: strands (eager bakes + live strands), spores and knots. The
/// glow is `mycelium_glow`, the aim preview and buttons `ui::myc_prompts`. Draw_64's debug
/// overlay is not drawn: `myc_debug_draw` can never turn on (its key check is `if false and …`).
pub fn draw_64_system(
    mut sim: ResMut<SimRes>,
    mut bake: ResMut<MycBake>,
    cams: Res<Cameras>,
    rend: Option<Res<MycRender>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(rend) = rend else { return };
    let view = draw_view(&cams);
    let m = &mut sim.0.mycelium;
    let live = draw_strands(m, &mut bake, view);
    if let Some(mut mesh) = meshes.get_mut(&rend.live) {
        fill_strand_mesh(&mut mesh, &live);
    }
    let v = cams.render_view();
    let sk = spores_and_knots(m, (v.x, v.y, v.x + v.w, v.y + v.h));
    if let Some(mut mesh) = meshes.get_mut(&rend.knots) {
        if sk.is_empty() {
            *mesh = empty_colour_mesh();
        } else {
            colour_mesh(&mut mesh, &sk);
        }
    }
}

/// [`MeshOp`]s → hyphae mesh entities.
pub fn apply_mesh_ops(
    mut commands: Commands,
    mut bake: ResMut<MycBake>,
    rend: Option<ResMut<MycRender>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(mut rend) = rend else { return };
    for op in std::mem::take(&mut bake.ops) {
        match op {
            MeshOp::Set(key, v) => {
                if let Some(e) = rend.meshes.remove(&key) {
                    commands.entity(e).despawn();
                }
                if v.is_empty() {
                    continue;
                }
                let e = commands
                    .spawn((
                        Mesh2d(meshes.add(strand_mesh(&v))),
                        MeshMaterial2d(rend.strand_material.clone()),
                        Transform::from_xyz(0.0, 0.0, Z_MYC_STRANDS),
                        strand_layers(),
                    ))
                    .id();
                rend.meshes.insert(key, e);
            }
            MeshOp::Remove(key) => {
                if let Some(e) = rend.meshes.remove(&key) {
                    commands.entity(e).despawn();
                }
            }
            MeshOp::Clear => {
                let keys: Vec<MeshKey> = rend
                    .meshes
                    .keys()
                    .copied()
                    .filter(|k| !matches!(k, MeshKey::Chunk(..)))
                    .collect();
                for k in keys {
                    if let Some(e) = rend.meshes.remove(&k) {
                        commands.entity(e).despawn();
                    }
                }
            }
        }
    }
}

/// Draw_0 → pooled sprite entities.
pub fn draw_0_system(
    mut commands: Commands,
    sim: Res<SimRes>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    rend: Option<ResMut<MycRender>>,
    mut q: Query<(&mut Sprite, &mut Anchor, &mut Transform, &mut Visibility)>,
) {
    let Some(mut rend) = rend else { return };
    let v = cams.render_view();
    let cmds = draw_0(&sim.0, v.x, v.w);
    let mut used = 0;
    for c in &cmds {
        let Some((sprite, anchor, tf)) = c.to_bevy(&assets) else {
            continue;
        };
        if used < rend.sprites.len() {
            if let Ok((mut s, mut a, mut t, mut vis)) = q.get_mut(rend.sprites[used]) {
                s.set_if_neq(sprite);
                a.set_if_neq(anchor);
                t.set_if_neq(tf);
                vis.set_if_neq(Visibility::Inherited);
            }
        } else {
            let e = commands
                .spawn((
                    Name::new("myc sprite"),
                    sprite,
                    anchor,
                    tf,
                    Visibility::Inherited,
                ))
                .id();
            rend.sprites.push(e);
        }
        used += 1;
    }
    for &e in &rend.sprites[used..] {
        if let Ok((_, _, _, mut vis)) = q.get_mut(e) {
            vis.set_if_neq(Visibility::Hidden);
        }
    }
}

pub struct MyceliumRenderPlugin;

impl Plugin for MyceliumRenderPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::render::blend::BlendPlugin>() {
            app.add_plugins(crate::render::blend::BlendPlugin);
        }
        app.init_resource::<MycBake>()
            .add_systems(Startup, setup)
            .init_resource::<crate::autopilot::MyceliumAnchor>()
            .add_systems(FixedUpdate, tiles_step_system.in_set(SimSet::After))
            .add_systems(
                FixedUpdate,
                sync_anchor
                    .in_set(SimSet::Prepare)
                    .before(crate::autopilot::run_script),
            )
            .add_systems(Update, (draw_64_system, apply_mesh_ops).chain())
            .add_systems(Update, draw_0_system)
            .add_plugins((
                crate::render::mycelium_glow::MyceliumGlowPlugin,
                crate::ui::myc_prompts::MycPromptsPlugin,
            ));
    }
}
