//! Task 14: mycelium rendering — baked hyphae meshes (myc_write_branch_spans, myc_growth_bake,
//! myc_tile_merge, myc_tiles_step, myc_tiles_clear, myc_cells_buffer, myc_growth_free_chunks),
//! live strands, the glow layout and the in-world button rects.

use myco_app::render::mycelium::{
    cells_buffer, growth_bake, live_strands, span_quad, tiles_clear, tiles_step,
    write_branch_spans, MeshKey, MeshOp, MycBake, Vbuf, V2, CHUNK_CELLS, TILE_BUFFER_MAX,
    VERTEX_BYTES,
};
use myco_app::render::mycelium_glow::glow_layout;
use myco_app::ui::myc_prompts::button_rect;
use myco_core::mycelium::shot::{build_branch, rasterize_branch};
use myco_core::mycelium::{Growth, MyceliumRt, Pt};
use myco_core::sim::ViewRect;

fn dist_to_segment(p: (f64, f64), a: Pt, b: Pt) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let l2 = dx * dx + dy * dy;
    let t = if l2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.x) * dx + (p.1 - a.y) * dy) / l2).clamp(0.0, 1.0)
    };
    let (qx, qy) = (a.x + t * dx, a.y + t * dy);
    ((p.0 - qx).powi(2) + (p.1 - qy).powi(2)).sqrt()
}

fn dist_to_polyline(p: (f64, f64), pts: &[Pt]) -> f64 {
    pts.windows(2)
        .map(|w| dist_to_segment(p, w[0], w[1]))
        .fold(f64::INFINITY, f64::min)
}

/// Quads (6 vertices each) → (x1, y1, x2, y2) rectangles.
fn quads(v: &[V2]) -> Vec<(f64, f64, f64, f64)> {
    v.chunks(6)
        .map(|q| {
            let xs = q.iter().map(|p| p[0] as f64);
            let ys = q.iter().map(|p| p[1] as f64);
            (
                xs.clone().fold(f64::INFINITY, f64::min),
                ys.clone().fold(f64::INFINITY, f64::min),
                xs.fold(f64::NEG_INFINITY, f64::max),
                ys.fold(f64::NEG_INFINITY, f64::max),
            )
        })
        .collect()
}

#[test]
fn ribbon_mesh_follows_points() {
    // a 3-point strand, 9 px wide at its root, tapering to 1 px at the tip
    let pts = [
        Pt { x: 100.0, y: 200.0 },
        Pt { x: 160.0, y: 215.0 },
        Pt { x: 230.0, y: 205.0 },
    ];
    let width = 9.0;
    let cells = rasterize_branch(&pts, width);
    let mut v = Vec::new();
    // a wide strand: spans run per row (horiz = false), as myc_growth_bake passes `tall`
    assert!(write_branch_spans(&mut v, &cells, false));
    assert!(!v.is_empty() && v.len() % 6 == 0);
    // cells are floored pixel positions: a cell square is centred on its pixel's centre, so the
    // ribbon's axis is the strand shifted by half a pixel
    let axis: Vec<Pt> = pts
        .iter()
        .map(|p| Pt {
            x: p.x + 0.5,
            y: p.y + 0.5,
        })
        .collect();
    let pts = axis;
    // the ribbon is the union of square cells of side `w` (tapering from `width`) stepped along
    // the axis: every vertex lies within width/2 of the axis, up to a square's half diagonal at
    // its corners (plus the pixel snapping of the cells)
    for p in &v {
        let d = dist_to_polyline((p[0] as f64, p[1] as f64), &pts);
        assert!(
            d <= width / 2.0 * std::f64::consts::SQRT_2 + 0.5,
            "vertex {p:?} is {d} px off the axis"
        );
    }
    // across the strand (vertically, it runs left-right) the ribbon is `width` wide at the root
    // and tapers: each row span's vertical position sits within the local half width
    let rows_at = |x: f64| -> (f64, f64) {
        let qs = quads(&v);
        let ys: Vec<f64> = qs
            .iter()
            .filter(|q| q.0 <= x && q.2 > x)
            .flat_map(|q| [q.1, q.3])
            .collect();
        (
            ys.iter().copied().fold(f64::INFINITY, f64::min),
            ys.iter().copied().fold(f64::NEG_INFINITY, f64::max),
        )
    };
    let (top, bottom) = rows_at(101.0);
    assert_eq!(bottom - top, width, "root cross-section {top}..{bottom}");
    let (top, bottom) = rows_at(228.0);
    assert!(bottom - top <= 3.0, "tip cross-section {top}..{bottom}");
    // near the root the ribbon is the full width: its edges sit at width/2 from the axis
    let root: Vec<_> = quads(&v)
        .into_iter()
        .filter(|q| q.0 < 110.0 && q.2 > 100.0)
        .collect();
    let edge = root
        .iter()
        .flat_map(|q| [(q.0, q.1), (q.2, q.3), (q.0, q.3), (q.2, q.1)])
        .filter(|c| c.0 >= 100.0 && c.0 <= 110.0)
        .map(|c| dist_to_polyline(c, &pts))
        .fold(0.0, f64::max);
    assert!(
        (edge - width / 2.0).abs() <= 1.5,
        "root half-width {edge}, want {}",
        width / 2.0
    );
    // and the ribbon covers the axis all the way: every axis pixel is inside some span
    let qs = quads(&v);
    for w in pts.windows(2) {
        for i in 0..=20 {
            let t = i as f64 / 20.0;
            let (x, y) = (w[0].x + (w[1].x - w[0].x) * t, w[0].y + (w[1].y - w[0].y) * t);
            let (px, py) = (x.floor() + 0.5, y.floor() + 0.5);
            let hit = qs
                .iter()
                .any(|q| px >= q.0 - 1.0 && px <= q.2 + 1.0 && py >= q.1 - 1.0 && py <= q.3 + 1.0);
            assert!(hit, "axis point ({x}, {y}) not covered");
        }
    }
}

#[test]
fn span_quad_is_two_triangles() {
    let mut v = Vec::new();
    span_quad(&mut v, 1.0, 2.0, 5.0, 3.0);
    assert_eq!(
        v,
        vec![
            [1.0, 2.0],
            [5.0, 2.0],
            [1.0, 3.0],
            [5.0, 2.0],
            [5.0, 3.0],
            [1.0, 3.0]
        ]
    );
}

#[test]
fn overlapping_cells_merge_into_one_span_per_line() {
    // three 3-px cells in a row along x: per column (horiz = true) one span each
    let cells = [
        myco_core::mycelium::Cell { x: 10.0, y: 10.0, w: 3.0 },
        myco_core::mycelium::Cell { x: 11.0, y: 10.0, w: 3.0 },
        myco_core::mycelium::Cell { x: 12.0, y: 10.0, w: 3.0 },
    ];
    let mut v = Vec::new();
    assert!(write_branch_spans(&mut v, &cells, true));
    let qs = quads(&v);
    // columns 9..14 (lo = 10 - 1 = 9, hi = 12 - 1 + 3 = 14): 5 columns, rows 9..12
    assert_eq!(qs.len(), 5);
    for (k, q) in qs.iter().enumerate() {
        assert_eq!(*q, (9.0 + k as f64, 9.0, 10.0 + k as f64, 12.0));
    }
    // no cells: nothing written
    let mut e = Vec::new();
    assert!(!write_branch_spans(&mut e, &[], true));
    assert!(e.is_empty());
}

#[test]
fn cells_buffer_is_one_square_per_cell() {
    let cells = [
        myco_core::mycelium::Cell { x: 10.0, y: 20.0, w: 4.0 },
        myco_core::mycelium::Cell { x: 30.0, y: 40.0, w: 1.0 },
    ];
    assert!(cells_buffer(&cells, 1, 1).is_none());
    let v = cells_buffer(&cells, 0, 2).unwrap();
    assert_eq!(quads(&v), vec![(8.0, 18.0, 12.0, 22.0), (30.0, 40.0, 31.0, 41.0)]);
}

fn growth(pts: &[Pt], width: f64, raster: bool) -> Growth {
    let b = build_branch(pts, 0.0, width, raster);
    Growth {
        id: 1,
        age: 0.0,
        duration: 60.0,
        finish_age: 62.0,
        bx1: b.bx1,
        by1: b.by1,
        bx2: b.bx2,
        by2: b.by2,
        branches: vec![b],
        scanned_length: 0.0,
        finished: false,
        budding: false,
        no_length: false,
        sound: None,
        instant: false,
        bake_d: 0.0,
        grid_stamp: -1.0,
        extra: Default::default(),
    }
}

fn line(x0: f64, y0: f64, len: f64) -> Vec<Pt> {
    vec![
        Pt { x: x0, y: y0 },
        Pt { x: x0 + len * 0.5, y: y0 + 3.0 },
        Pt { x: x0 + len, y: y0 },
    ]
}

#[test]
fn baked_growth_is_filed_under_its_tile_and_merged_after_30_frames() {
    let mut m = MyceliumRt::default();
    let mut g = growth(&line(100.0, 300.0, 200.0), 6.0, true);
    g.finished = true;
    m.growths.push(g);
    m.dirty_growths.push(0);
    let mut b = MycBake::default();
    tiles_step(&mut m, &mut b);
    assert_eq!(m.frame, 1.0);
    assert!(m.dirty_growths.is_empty());
    assert_eq!(b.growths[0].vbuf, Vbuf::Pending);
    // the strand's cells are dropped once baked (b.cells = undefined)
    assert!(m.growths[0].branches[0].cells.is_none());
    assert_eq!(m.tile_list.len(), 1);
    assert_eq!(m.tile_list[0].pending, vec![0]);
    assert_eq!(m.tile_list[0].key, (0, 0));
    let ops = std::mem::take(&mut b.ops);
    assert!(matches!(&ops[..], [MeshOp::Set(MeshKey::Growth(0), v)] if !v.is_empty()));
    // 29 more frames: still pending
    for _ in 0..29 {
        tiles_step(&mut m, &mut b);
    }
    assert_eq!(b.growths[0].vbuf, Vbuf::Pending);
    assert!(b.ops.is_empty());
    // frame 31: frame - since (1) >= 30 → merged into the tile mesh
    tiles_step(&mut m, &mut b);
    assert_eq!(b.growths[0].vbuf, Vbuf::Merged);
    assert!(m.tile_list[0].pending.is_empty());
    let ops = std::mem::take(&mut b.ops);
    assert!(ops.contains(&MeshOp::Remove(MeshKey::Growth(0))));
    assert!(ops
        .iter()
        .any(|o| matches!(o, MeshOp::Set(MeshKey::Tile(0), v) if !v.is_empty())));
    // baking again is a no-op (vbuf != -1)
    growth_bake(&mut m, &mut b, 0);
    assert!(b.ops.is_empty());
}

#[test]
fn unrasterized_growth_is_rasterized_when_baked_and_empty_one_is_not_filed() {
    let mut m = MyceliumRt::default();
    // loaded growths come without cells
    m.growths.push(growth(&line(5000.0, 3000.0, 80.0), 4.0, false));
    // a single-point strand rasterizes to nothing but its tip
    let mut empty = growth(&line(0.0, 0.0, 10.0), 4.0, false);
    empty.branches[0].points.truncate(1);
    m.growths.push(empty);
    m.dirty_growths = vec![1, 0];
    let mut b = MycBake::default();
    tiles_step(&mut m, &mut b);
    assert_eq!(b.growths[0].vbuf, Vbuf::Pending);
    assert_eq!(m.tile_list[0].key, (4, 2));
    assert_eq!(b.growths[1].vbuf, Vbuf::Empty);
    assert_eq!(m.tile_list.len(), 1);
}

#[test]
fn tile_buffer_is_closed_when_it_would_pass_256k() {
    let mut m = MyceliumRt::default();
    let mut b = MycBake::default();
    let mut total = 0usize;
    let mut i = 0;
    // growths in one tile, each merged on its own, until the tile closes a buffer
    while b.tiles.first().is_none_or(|t| t.closed == 0) {
        let mut g = growth(&line(100.0, 100.0 + (i % 400) as f64, 300.0), 9.0, true);
        g.finished = true;
        m.growths.push(g);
        m.dirty_growths.push(i);
        for _ in 0..31 {
            tiles_step(&mut m, &mut b);
        }
        let sz = b.ops.iter().find_map(|o| match o {
            MeshOp::Set(MeshKey::Growth(_), v) => Some(v.len() * VERTEX_BYTES),
            _ => None,
        });
        if b.tiles[0].closed == 0 {
            total += sz.unwrap();
            assert_eq!(b.tiles[0].size, total);
            assert!(total <= TILE_BUFFER_MAX);
        } else {
            assert!(total + sz.unwrap() > TILE_BUFFER_MAX);
            assert!(b
                .ops
                .iter()
                .any(|o| matches!(o, MeshOp::Set(MeshKey::Closed(0, 0), v) if v.len() * VERTEX_BYTES == total)));
            assert_eq!(b.tiles[0].size, sz.unwrap());
        }
        b.ops.clear();
        i += 1;
        assert!(i < 1000);
    }
}

#[test]
fn tiles_reset_and_clear_drop_every_mesh() {
    let mut m = MyceliumRt::default();
    let mut g = growth(&line(100.0, 300.0, 200.0), 6.0, true);
    g.finished = true;
    m.growths.push(g);
    m.dirty_growths.push(0);
    let mut b = MycBake::default();
    tiles_step(&mut m, &mut b);
    b.ops.clear();
    tiles_clear(&mut m, &mut b);
    assert!(m.tile_list.is_empty() && m.tiles.is_empty());
    assert!(b.tiles.is_empty());
    // a pending growth goes back to unbaked (p.vbuf = -1), as the original
    assert_eq!(b.growths[0].vbuf, Vbuf::Unbaked);
    assert_eq!(b.ops, vec![MeshOp::Clear]);
    // tiles_reset (set by the logic on load) clears and is consumed
    m.tiles_reset = true;
    b.ops.clear();
    tiles_step(&mut m, &mut b);
    assert!(!m.tiles_reset);
    assert_eq!(b.ops.first(), Some(&MeshOp::Clear));
}

#[test]
fn live_strand_shows_its_grown_part_in_96_cell_chunks() {
    let mut m = MyceliumRt::default();
    let pts = vec![Pt { x: 0.0, y: 100.0 }, Pt { x: 800.0, y: 100.0 }];
    let mut g = growth(&pts, 3.0, true);
    let n = g.branches[0].cells.as_ref().unwrap().len();
    assert!(n > 2 * CHUNK_CELLS);
    g.age = 45.0; // p = 0.75
    m.growths.push(g);
    m.active.push(0);
    let mut b = MycBake::default();
    let view = (-64.0, -64.0, 2000.0, 2000.0);
    let v = live_strands(&m, &mut b, view);
    let shown = ((0.75 * n as f64).ceil() as usize).min(n);
    let nfull = shown / CHUNK_CELLS;
    assert_eq!(v.len(), (shown - nfull * CHUNK_CELLS) * 6);
    let chunks: Vec<_> = b
        .ops
        .iter()
        .filter_map(|o| match o {
            MeshOp::Set(MeshKey::Chunk(0, 0, c), v) => Some((*c, v.len())),
            _ => None,
        })
        .collect();
    assert_eq!(chunks.len(), nfull);
    assert!(chunks.iter().all(|&(_, l)| l == CHUNK_CELLS * 6));
    // chunks are built once
    b.ops.clear();
    live_strands(&m, &mut b, view);
    assert!(b.ops.is_empty());
    // a strand out of view draws nothing
    assert!(live_strands(&m, &mut b, (5000.0, 5000.0, 6000.0, 6000.0)).is_empty());
    // finishing frees the chunks (myc_growth_free_chunks)
    m.growths[0].finished = true;
    m.dirty_growths.push(0);
    tiles_step(&mut m, &mut b);
    for c in 0..nfull {
        assert!(b.ops.contains(&MeshOp::Remove(MeshKey::Chunk(0, 0, c))));
    }
}

#[test]
fn glow_buffer_covers_the_view_with_a_two_texel_margin() {
    let v = ViewRect {
        x: 1001.0,
        y: -37.0,
        w: 1920.0,
        h: 1080.0,
    };
    let g = glow_layout(&v, 2.0);
    assert_eq!((g.w, g.h), (964, 544));
    assert_eq!((g.ox, g.oy), (996.0, -42.0));
    assert_eq!(g.step, 2.0);
    // the buffer covers the view
    assert!(g.ox <= v.x && g.ox + g.w as f64 * g.step >= v.x + v.w);
    assert!(g.oy <= v.y && g.oy + g.h as f64 * g.step >= v.y + v.h);
}

#[test]
fn button_rect_is_centred_above_the_point_and_clamped() {
    // label 200 px: w = 218, h = 28, centred on the point, offset up
    let r = button_rect(500.0, 400.0, 200.0, -72.0, 1920.0, 1080.0);
    assert_eq!((r.x1, r.y1, r.x2, r.y2), (391.0, 328.0, 609.0, 356.0));
    // a short label still gets 110 px
    let r = button_rect(500.0, 400.0, 20.0, -44.0, 1920.0, 1080.0);
    assert_eq!(r.x2 - r.x1, 110.0);
    // clamped 8 px inside the GUI
    let r = button_rect(0.0, 0.0, 200.0, -72.0, 1920.0, 1080.0);
    assert_eq!((r.x1, r.y1), (8.0, 8.0));
    let r = button_rect(1920.0, 1080.0, 200.0, 0.0, 1920.0, 1080.0);
    assert_eq!((r.x2, r.y2), (1912.0, 1072.0));
}
