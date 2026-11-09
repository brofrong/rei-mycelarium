//! Task 13: underground world drawing.

use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::render::batch::{QuadBatch, QuadBlend};
use myco_app::render::leaf_art::PagePixels;
use myco_app::render::world::{
    caves_mask_init, chunk_tiles, lakes_init, merge_colour, plan_chunk_meshes, stone_tiles_init,
    stones_init, world_draw, world_draw_turf, DrawCtx, LakeFlares,
};
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets")
}

/// The real extracted assets with placeholder handles; `None` (test skipped) without build/assets.
fn assets() -> Option<GameAssets> {
    let root = assets_dir();
    if !root.join("sprites.json").exists() {
        eprintln!("skipped: no build/assets");
        return None;
    }
    Some(
        GameAssets::load(
            &root,
            |_| Handle::default(),
            |_| Handle::default(),
            |_| Handle::default(),
        )
        .unwrap(),
    )
}

fn sim() -> Sim {
    Sim::new_game(GameMode::Exploration, "", Some(31607))
}

#[test]
fn chunk_mesh_vertex_count_matches_tiles() {
    let Some(a) = assets() else { return };
    let mut sim = sim();
    let mut total = 0;
    // generation leaves every tile 0 (only the disabled tile-stone painters write 2), so put
    // hard soil, stone and water cells into chunk 0 by hand
    sim.world.chunk_reach(0, 40);
    let mut ch0 = sim.world.chunk(0).unwrap().clone();
    let cols = sim.world.params.cols;
    for (i, tile) in [1, 2, 3, 0, 1, 3, 3, 2]
        .into_iter()
        .cycle()
        .take(97)
        .enumerate()
    {
        let i = i as i64 * 3;
        sim.world.chunk_set(&mut ch0, i % cols, i / cols, tile, 0.0);
    }
    sim.world.chunks.insert(0, ch0);
    for cx in -2..=2 {
        sim.world.chunk_reach(cx, 40);
        let ch = sim.world.chunk(cx).unwrap();
        let tiles = ch
            .tiles
            .iter()
            .take((ch.rows * sim.world.params.cols) as usize)
            .filter(|t| matches!(t, 1..=3))
            .count();
        let mut b = QuadBatch::new();
        chunk_tiles(&mut b, &a, &sim.world, ch);
        assert_eq!(
            b.quad_count(),
            tiles,
            "chunk {cx}: one quad per non-empty tile"
        );
        let verts: usize = b.runs.iter().map(|r| r.verts.len()).sum();
        assert_eq!(verts, 6 * tiles, "chunk {cx}: six vertices per quad");
        total += tiles;
    }
    assert!(total > 50, "the hand-made tiles are counted ({total})");
}

#[test]
fn dirty_chunks_are_cleared_and_rebuilt() {
    let mut sim = sim();
    let mut built = HashMap::new();
    for cx in -1..=1 {
        sim.world.get_chunk(cx);
    }
    let (rebuild, dropped) = plan_chunk_meshes(&mut sim.world, &mut built, &[-1, 0, 1]);
    assert_eq!(rebuild, vec![-1, 0, 1]);
    assert!(dropped.is_empty());
    assert!(
        sim.world.dirty_chunks.is_empty(),
        "R9a: the renderer clears dirty_chunks"
    );
    // nothing changed: nothing to rebuild
    let (rebuild, _) = plan_chunk_meshes(&mut sim.world, &mut built, &[-1, 0, 1]);
    assert!(rebuild.is_empty());
    // a deeper layer marks chunk 0 dirty: only it is rebuilt
    sim.world
        .chunk_reach(0, sim.world.chunk(0).unwrap().rows + 5);
    assert!(sim.world.dirty_chunks.contains(&0));
    let (rebuild, _) = plan_chunk_meshes(&mut sim.world, &mut built, &[-1, 0, 1]);
    assert_eq!(rebuild, vec![0]);
    // scrolling right: chunk -1 is dropped, chunk 2 built
    sim.world.get_chunk(2);
    let (rebuild, dropped) = plan_chunk_meshes(&mut sim.world, &mut built, &[0, 1, 2]);
    assert_eq!((rebuild, dropped), (vec![2], vec![-1]));
}

#[test]
fn pixel_inits_match_core_embedded_art() {
    let Some(a) = assets() else { return };
    let names = [
        "stones",
        "stone_tile",
        "water",
        "spr_skeleton_T_rex",
        "spr_UFO",
    ];
    let mut all: Vec<&str> = names.to_vec();
    for c in myco_core::world::caves::cave_defs() {
        all.push(c.sprite);
    }
    let px = PagePixels::load(&assets_dir(), &a, &all).unwrap();

    // stones_init
    let (kinds, reach, _, grey) = stones_init(&px, &a);
    let core = myco_core::world::stones::stone_art();
    assert_eq!(kinds.len(), core.kinds.len());
    for (k, c) in kinds.iter().zip(&core.kinds) {
        assert_eq!(k[..4], [c.left, c.w, c.ox, c.oy], "stone kind origin");
        assert!((k[4] - c.radius).abs() < 1e-9 && (k[5] - c.body).abs() < 1e-9);
    }
    assert!((reach - core.reach).abs() < 1e-9);
    assert!(!grey.is_empty());

    // stone_tiles_init
    assert_eq!(stone_tiles_init(&px, &a), core.tiles, "stone tile spans");

    // lakes_init
    let lakes = lakes_init(&px, &a);
    let core_lakes = myco_core::world::lakes::lake_kinds();
    assert_eq!(lakes.len(), core_lakes.len());
    for (l, c) in lakes.iter().zip(core_lakes) {
        assert_eq!(
            (l.left, l.width, l.height, l.top, l.bottom),
            (c.left, c.width, c.height, c.top, c.bottom)
        );
        assert_eq!(l.spans, c.spans, "lake mask spans");
    }

    // caves_mask_init: alpha >= 250 is the core mask
    for (id, w, h, alpha) in caves_mask_init(&px, &a) {
        let m = myco_core::world::caves::cave_mask(&id).unwrap();
        assert_eq!((m.w, m.h), (w, h), "{id} size");
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    alpha[y * w + x] >= 250,
                    m.solid(x as i64, y as i64),
                    "{id} mask at {x},{y}"
                );
            }
        }
    }
}

#[test]
fn world_draw_follows_the_original_passes() {
    let Some(a) = assets() else { return };
    let mut sim = sim();
    let mut flares = LakeFlares::default();
    let img = Handle::<Image>::default();
    let ctx = DrawCtx {
        assets: &a,
        stones: None,
        glow_image: Some(&img),
        now: 1000.0,
        start_linear: false,
    };
    // above the surface: nothing
    let sy = sim.world.params.surface_y;
    let f = world_draw(
        &mut sim,
        &ctx,
        &mut flares,
        0.0,
        sy - 2000.0,
        1920.0,
        1080.0,
    );
    assert!(f.back.is_empty() && f.front.is_empty() && f.chunks.is_empty());

    // the start view, half underground
    let (vx, vy, vw, vh) = (-960.0, sy - 540.0, 1920.0, 1080.0);
    let f = world_draw(&mut sim, &ctx, &mut flares, vx, vy, vw, vh);
    assert_eq!(f.chunks, {
        let cw = sim.world.params.chunk_w as f64;
        ((vx / cw).floor() as i64..=((vx + vw) / cw).floor() as i64).collect::<Vec<_>>()
    });
    // soil rows: whole tiles across the view for every visible row
    let t = sim.world.params.tile as f64;
    let rows = ((vy + vh - sy) / t).floor() + 1.0;
    let cols = ((vx + vw) / t).ceil() - (vx / t).floor();
    assert!(f.back.quad_count() as f64 >= rows * cols);
    // the start area has foods: glow pass, additive glow before and after the foods
    let foods = sim
        .world
        .chunks
        .values()
        .flat_map(|c| &c.foods)
        .filter(|fd| !fd.collected && fd.x >= vx - 48.0 && fd.x <= vx + vw + 48.0)
        .filter(|fd| fd.y >= vy - 48.0 && fd.y <= vy + vh + 48.0)
        .count();
    assert!(foods > 0);
    let g = f.glow.as_ref().expect("food glow pass");
    assert_eq!(g.seed.quad_count(), foods, "one seed quad per visible food");
    let blends: Vec<QuadBlend> = f.front.runs.iter().map(|r| r.blend).collect();
    assert_eq!(
        blends.last(),
        Some(&QuadBlend::Add),
        "inner glow drawn last"
    );
    let first_add = blends.iter().position(|b| *b == QuadBlend::Add).unwrap();
    assert!(
        blends[first_add + 1..].contains(&QuadBlend::Alpha),
        "foods between the outer and inner glow: {blends:?}"
    );

    // turf: whole sprite widths across the view
    let mut turf = QuadBatch::new();
    world_draw_turf(&mut turf, &a, &sim.world, vx, vy, vw, vh);
    let tw = a.sprites[sim.world.params.turf as usize].size.x as f64;
    assert_eq!(
        turf.quad_count() as f64,
        ((vx + vw) / tw).ceil() - (vx / tw).floor()
    );
}

#[test]
fn merge_colour_rounds_like_love() {
    assert_eq!(
        merge_colour(0.0, 16777215.0, 0.5),
        128.0 * (1.0 + 256.0 + 65536.0)
    );
    assert_eq!(
        merge_colour(0x0000ff as f64, 0x00ff00 as f64, 0.25),
        (191 + (64 << 8)) as f64
    );
}

#[test]
fn deposit_without_deep_falls_back_to_its_row() {
    use myco_app::render::world::world_draw_stone_deposits;
    use myco_core::state::StoneCellSave;
    let Some(a) = assets() else { return };
    let mut sim = sim();
    let cw = sim.world.params.chunk_w as f64;
    let deep_y = sim.world.deep_y();
    let row_above = ((deep_y - 48.0) / 48.0).floor() as i64 - 1;
    let row_below = (deep_y / 48.0).ceil() as i64 + 1;
    let mut ch = sim.world.get_chunk(0).clone();
    let cell = |row: i64| StoneCellSave {
        col: 1,
        row,
        tc: 3,
        tr: 3,
        deep: None,
        ..Default::default()
    };
    ch.stone_cells = vec![cell(row_above), cell(row_below)];
    let mut b = QuadBatch::new();
    let (vy, vh) = (
        row_above as f64 * 48.0 - 10.0,
        (row_below - row_above + 2) as f64 * 48.0,
    );
    world_draw_stone_deposits(&mut b, &a, &sim.world, &ch, 0.0, vy, cw, vh);
    let verts: Vec<_> = b.runs.iter().flat_map(|r| r.verts.iter()).collect();
    assert_eq!(verts.len(), 12, "both cells drawn");
    let white = verts[0].col;
    let tinted = verts[6].col;
    assert_eq!(white, [1.0, 1.0, 1.0, 1.0], "above the deep line: untinted");
    assert_ne!(tinted, white, "below the deep line: deep_stone_colour");
}

/// Ruling R27: the world starts drawing with the filter the frame starts with (off after the
/// first frame), and the perk icons turn it on (`cave_icon_smooth`) and restore it.
#[test]
fn texture_filter_follows_the_original_state() {
    use myco_app::render::world::cave_perk_icons_draw;
    use myco_app::ui::gm_draw::{DrawItem, Gm};
    let Some(mut a) = assets() else { return };
    let mut sim = sim();
    let mut flares = LakeFlares::default();
    let img = Handle::<Image>::default();
    let sy = sim.world.params.surface_y;
    let ctx = DrawCtx {
        assets: &a,
        stones: None,
        glow_image: Some(&img),
        now: 1000.0,
        start_linear: false,
    };
    let f = world_draw(&mut sim, &ctx, &mut flares, -960.0, sy - 540.0, 1920.0, 1080.0);
    assert!(!f.back.filter(), "soil, seams, dirt, bones and caves draw nearest");
    // perk icons: bilinear twin pages while drawn, the painter's filter restored after
    let mut images = Assets::<Image>::default();
    a.pages_linear = (0..a.pages.len()).map(|_| images.add(Image::default())).collect();
    let linear_ids: Vec<_> = a.pages_linear.iter().map(|h| h.id()).collect();
    let mut st = sim.state.clone();
    st.state.cave_perks = serde_json::from_str(r#"{"purple": 1, "blue": 1}"#).unwrap();
    let mut g = Gm::new(&a);
    assert!(!g.tex_filter());
    cave_perk_icons_draw(&mut g, &st, 10.0, 10.0);
    assert!(!g.tex_filter(), "old_filter restored");
    let quads: Vec<_> = g
        .items
        .iter()
        .filter_map(|i| match i {
            DrawItem::Quad { image, .. } => Some(image.id()),
            _ => None,
        })
        .collect();
    assert!(!quads.is_empty());
    assert!(quads.iter().all(|id| linear_ids.contains(id)));
}
