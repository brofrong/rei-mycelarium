//! Task 15: mushrooms and wildlife on screen (obj_life Draw_0 / `life_draw`), the stinky and
//! repeller-cloud art, worms, leaf tendrils and the pixel inits whose data logic embeds.

use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::camera::Cameras;
use myco_app::render::batch::{GmQuad, QuadBlend, WhiteTexture};
use myco_app::render::leaf_art::PagePixels;
use myco_app::render::life::{
    hedgehog_back_init, leaf_art_read, leaf_tendril_cells_draw, life_draw, life_draw_system,
    squirrel_frames_build, squirrel_trunk_edges, worms_draw, LifeCtx, LifeEntity, LifeKey,
    LifePools,
};
use myco_app::render::mushrooms::{
    mushroom_draw, repeller_cloud_surface_size, repeller_soft_sprite, stinky_glow_pixels,
};
use myco_app::sim_bridge::SimRes;
use myco_core::life::{art::art, Hedgehog, TrailDot, Worm};
use myco_core::mycelium::{Cell, Pt};
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::{GameMode, MushroomSave};
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

fn game() -> Sim {
    let mut sim = Sim::new_game(GameMode::Exploration, "life", Some(31607));
    sim.intro_active = false;
    sim.life.animals.clear();
    sim
}

fn view() -> ViewRect {
    Cameras::default().render_view()
}

fn life_app(sim: Sim, assets: GameAssets) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Mesh>()
        .init_asset::<GmQuad>()
        .insert_resource(WhiteTexture(Handle::default()))
        .insert_resource(SimRes(sim))
        .insert_resource(assets)
        .insert_resource(Cameras::default())
        .init_resource::<LifePools>()
        .add_systems(Update, life_draw_system);
    app
}

fn entities_of(app: &mut App, key: LifeKey) -> usize {
    let world = app.world_mut();
    let mut q = world.query::<&LifeEntity>();
    q.iter(world).filter(|e| e.0 == key).count()
}

#[test]
fn animal_entities_follow_sim_ids() {
    let Some(assets) = assets() else { return };
    let v = view();
    let mut app = life_app(game(), assets);
    app.update();
    assert_eq!(entities_of(&mut app, LifeKey::Hedgehog(5)), 0);

    // a hedgehog with id 5 appears in view: one entity
    app.world_mut()
        .resource_mut::<SimRes>()
        .0
        .life
        .animals
        .push(Hedgehog {
            id: 5,
            x: v.x + v.w * 0.5,
            y: v.y + v.h * 0.5,
            ..Default::default()
        });
    app.update();
    app.update();
    assert_eq!(entities_of(&mut app, LifeKey::Hedgehog(5)), 1);

    // it leaves the sim: the entity goes
    app.world_mut()
        .resource_mut::<SimRes>()
        .0
        .life
        .animals
        .clear();
    app.update();
    assert_eq!(entities_of(&mut app, LifeKey::Hedgehog(5)), 0);
    let world = app.world_mut();
    let mut q = world.query::<&LifeEntity>();
    assert!(
        q.iter(world).all(|e| !matches!(e.0, LifeKey::Hedgehog(_))),
        "no hedgehog entity left"
    );
}

#[test]
fn life_draw_follows_obj_life_draw_order() {
    let Some(assets) = assets() else { return };
    let v = view();
    let mut sim = game();
    let (cx, cy) = (v.x + v.w * 0.5, v.y + v.h * 0.5);
    for (kind, dx) in [(1i64, -300.0), (5, 0.0), (0, 300.0)] {
        let i = myco_core::mushrooms::mushroom_add(
            &mut sim,
            myco_core::mushrooms::MushroomKind::from_code(kind).unwrap(),
            cx + dx,
            cy,
        );
        sim.state.mushrooms[i].born = None;
    }
    sim.life.animals.push(Hedgehog {
        id: 77,
        x: cx,
        y: cy,
        ..Default::default()
    });
    sim.life.drops.push(myco_core::life::Drop {
        x: cx,
        y: cy,
        target_y: cy + 100.0,
        speed: 1.0,
        source: 0,
    });
    let ctx = LifeCtx {
        assets: &assets,
        fg: None,
        fx: None,
        clouds: None,
        start_linear: false,
        game_view: None,
    };
    let d = life_draw(&sim, &ctx, &v);
    let keys: Vec<LifeKey> = d.segs.iter().map(|s| s.0).collect();
    let ids: Vec<u64> = sim.state.mushrooms.iter().map(|m| m.live.id).collect();
    assert_eq!(
        keys,
        vec![
            LifeKey::Mushroom(ids[0]),
            LifeKey::Mushroom(ids[1]),
            LifeKey::Mushroom(ids[2]),
            LifeKey::Hedgehog(77),
            LifeKey::Drops,
        ],
        "mushrooms, (stinky: no particles yet), hedgehogs, drops"
    );
    // the predator draws its flies after itself: two runs (sheet, fly)
    assert!(d.segs[0].1.quad_count() > 1, "predator flies");
    // nothing switched the filter on: nearest as world_draw left it
    assert!(d.segs.iter().all(|s| s.1.runs.iter().all(|r| !r.linear)));
}

#[test]
fn mushroom_draw_places_the_variant_cell_at_its_origin() {
    let Some(assets) = assets() else { return };
    let sim = game();
    let m = MushroomSave {
        kind: 0,
        variant: 1,
        born: None,
        ..Default::default()
    };
    let spr = myco_core::mushrooms::types::mushroom_sprite_now(&m);
    let def = assets.sprite(spr);
    let mut b = myco_app::render::batch::QuadBatch::new();
    mushroom_draw(
        &mut b, &assets, &sim, &m, 100.0, 200.0, 16777215.0, 1.0, 0.0, 1.0,
    );
    assert_eq!(b.quad_count(), 1);
    let v = &b.runs[0].verts;
    let (minx, miny) = v.iter().fold((f32::MAX, f32::MAX), |a, p| {
        (a.0.min(p.pos[0]), a.1.min(p.pos[1]))
    });
    let maxy = v.iter().fold(f32::MIN, |a, p| a.max(p.pos[1]));
    // the cell's untrimmed top-left is at (x - xoff, y - yoff); trimming only moves inwards
    assert!(minx >= 100.0 - def.origin.x - 0.01, "{minx}");
    assert!(miny >= 200.0 - def.origin.y - 0.01, "{miny}");
    assert!(maxy <= 200.0 - def.origin.y + def.size.y + 0.01, "{maxy}");

    // rotated: the origin stays put (rotation is about the foot)
    let mut r = myco_app::render::batch::QuadBatch::new();
    mushroom_draw(
        &mut r, &assets, &sim, &m, 100.0, 200.0, 16777215.0, 1.0, 90.0, 1.0,
    );
    let c: Vec<[f32; 2]> = r.runs[0].verts.iter().map(|p| p.pos).collect();
    let cx = c.iter().map(|p| p[0]).sum::<f32>() / c.len() as f32;
    assert!(
        cx < 100.0,
        "turned 90° counter-clockwise: the cap leans left ({cx})"
    );
}

#[test]
fn stinky_glow_and_repeller_sprites_match_the_original_art() {
    // stinky_art_init: 16 px, alpha rings 0.1·k² + 0.02 every half pixel (LÖVE: rgb = alpha)
    let g = stinky_glow_pixels();
    assert_eq!(g.len(), 16 * 16 * 4);
    let at = |x: usize, y: usize| &g[(y * 16 + x) * 4..(y * 16 + x) * 4 + 4];
    let centre = at(8, 8);
    let edge = at(0, 8);
    assert!(centre[3] > 100, "centre {centre:?}");
    assert_eq!(centre[0], centre[3], "white premultiplied by coverage");
    assert!(edge[3] < centre[3]);
    assert_eq!(at(0, 0)[3], 0, "corner outside every circle");
    // repeller_soft_sprite(64, 1.4): white, alpha (1 - d)^hardness
    let p = repeller_soft_sprite(64, 1.4);
    let a = |x: usize, y: usize| p[(y * 64 + x) * 4 + 3];
    assert_eq!(p[0..3], [255, 255, 255]);
    let d = ((31.5f64 - 32.0).powi(2) * 2.0).sqrt() / 32.0;
    assert_eq!(a(31, 31), (((1.0 - d).powf(1.4)) * 255.0).round() as u8);
    assert_eq!(a(0, 0), 0);
    // cloud surface: the view in `repeller_cloud_pixel` texels plus 2
    let (ox, oy, sw, sh) = repeller_cloud_surface_size(&ViewRect {
        x: 100.0,
        y: -50.0,
        w: 1920.0,
        h: 1080.0,
    });
    assert_eq!((ox, oy, sw, sh), (99.0, -51.0, 642, 362));
}

#[test]
fn pixel_inits_match_core_embedded_art() {
    let Some(assets) = assets() else { return };
    let px = PagePixels::load(
        &assets.root,
        &assets,
        &[
            "hedgehog",
            "tree01",
            "tree02",
            "tree03",
            "leaf",
            "leaf_2",
            "leaf_change",
            "leaf_change_2",
        ],
    )
    .unwrap();
    let core = art();
    let back = hedgehog_back_init(&px, &assets);
    assert_eq!(back.len(), 8);
    for (a, b) in back.iter().zip(&core.hedgehog_back) {
        assert!(
            (a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9,
            "{a:?} {b:?}"
        );
    }
    for t in ["tree01", "tree02", "tree03"] {
        let e = squirrel_trunk_edges(&px, &assets, t);
        let c = core.trunk(t);
        assert_eq!(e.left, c.left, "{t} left");
        assert_eq!(e.right, c.right, "{t} right");
        assert_eq!((e.origin_y, e.h), (c.origin_y, c.h));
    }
    for (spr, frames, want) in [
        ("leaf", 1, &core.leaf_art.faces[0]),
        ("leaf_2", 1, &core.leaf_art.faces[1]),
        ("leaf_change", 8, &core.leaf_art.turns[0]),
        ("leaf_change_2", 8, &core.leaf_art.turns[1]),
    ] {
        let got = leaf_art_read(&px, &assets, spr, frames);
        assert_eq!(got.len(), want.len());
        for (g, w) in got.iter().zip(want.iter()) {
            for k in 0..8 {
                assert!(
                    (g.data[k] - w[k]).abs() < 1e-9,
                    "{spr} {k}: {:?} {w:?}",
                    g.data
                );
            }
        }
    }
    let f = squirrel_frames_build(&assets);
    assert_eq!(f.parts.len(), 9);
    assert_eq!(f.origin, (51.0, 95.0));
}

#[test]
fn worms_draw_one_rect_per_body_pixel_and_trail_dot() {
    let mut b = myco_app::render::batch::QuadBatch::new();
    let mut sim = game();
    sim.life.worms.push(Worm {
        id: 1,
        body: vec![Pt { x: 100.0, y: 100.0 }, Pt { x: 110.0, y: 100.0 }],
        ..Default::default()
    });
    sim.life.worm_trail.push(TrailDot {
        x: 50.0,
        y: 50.0,
        life: 450.0,
    });
    sim.life.worm_trail.push(TrailDot {
        x: -5000.0,
        y: 50.0,
        life: 450.0,
    });
    worms_draw(&mut b, &sim, 0.0, 0.0, 1920.0, 1080.0);
    // one visible trail dot + 10 body steps (k = 0..9)
    assert_eq!(b.quad_count(), 11);
    assert_eq!(b.runs[0].blend, QuadBlend::Alpha);
    // trail: 3 px square (thickness 4 - 1) at half alpha·life/frames
    let t = &b.runs[0].verts[0..6];
    let w = t.iter().map(|v| v.pos[0]).fold(f32::MIN, f32::max)
        - t.iter().map(|v| v.pos[0]).fold(f32::MAX, f32::min);
    assert_eq!(w, 3.0);
    assert!((t[0].col[3] - 0.25).abs() < 1e-6);
}

#[test]
fn tendril_cells_are_squares_in_the_original_vertex_order() {
    let cells = [
        Cell {
            x: 10.0,
            y: 20.0,
            w: 3.0,
        },
        Cell {
            x: 11.0,
            y: 21.0,
            w: 4.0,
        },
    ];
    let mut out = Vec::new();
    leaf_tendril_cells_draw(&mut out, &cells, 1);
    assert_eq!(out.len(), 6, "only `count` cells");
    // div(3, 2) = 1: x0 = 9, x1 = 12
    assert_eq!(out[0], [9.0, 19.0]);
    assert_eq!(out[1], [12.0, 19.0]);
    assert_eq!(out[4], [12.0, 22.0]);
}

#[test]
fn post_layer_draw_plans_like_the_original() {
    use myco_app::render::post::{post_layer_draw, PostPlan};
    assert_eq!(post_layer_draw(false, 1920.0, 1080.0), PostPlan::Plain);
    let PostPlan::BlackMetal { glow: Some(g) } = post_layer_draw(true, 1920.0, 1080.0) else {
        panic!("glow strength 0.45 > 0: glow passes");
    };
    // glow_downscale 6, glow_wide 3; strength 0.45, halo 0.5
    assert_eq!((g.gw, g.gh), (320.0, 180.0));
    assert_eq!((g.fw, g.fh), (320.0 / 3.0, 60.0));
    assert!((g.near_alpha - 0.45).abs() < 1e-12 && (g.far_alpha - 0.225).abs() < 1e-12);
}

/// Zoomed out, animals spawn and vanish at the 1× game view's edge (logic's view): they fade out
/// over their spawn margin beyond it instead of popping. Inside the 1× view nothing changes.
#[test]
fn animals_fade_beyond_the_1x_view_when_zoomed_out() {
    let Some(assets) = assets() else { return };
    let cams = Cameras {
        zoom: 0.5,
        ..Default::default()
    };
    let gv = cams.game_view;
    let rv = cams.render_view();
    let margin = myco_core::life::hedgehog::hedgehog_border_margin();
    let mut sim = game();
    for (id, x) in [
        (1, gv.x + gv.w * 0.5),
        (2, gv.x - margin * 0.5),
        (3, gv.x - margin - 10.0),
    ] {
        sim.life.animals.push(Hedgehog {
            id,
            x,
            y: gv.y + gv.h * 0.5,
            ..Default::default()
        });
    }
    let ctx = LifeCtx {
        assets: &assets,
        fg: None,
        fx: None,
        clouds: None,
        start_linear: false,
        game_view: Some(gv),
    };
    let d = life_draw(&sim, &ctx, &rv);
    let alpha = |id: u64| {
        d.segs
            .iter()
            .find(|s| s.0 == LifeKey::Hedgehog(id))
            .map(|s| s.1.runs[0].verts[0].col[3])
    };
    assert_eq!(alpha(1), Some(1.0), "inside the 1x view: unchanged");
    let a2 = alpha(2).expect("half way out: drawn");
    assert!((a2 - 0.5).abs() < 0.02, "{a2}");
    assert_eq!(alpha(3), None, "past the margin: fully faded, no entity");
}

/// repeller_clouds_draw (scr_life_a_2.lua:744) ends with `gpu_set_tex_filter(filt)`: the filter
/// saved on entry comes back, not always-linear (the LÖVE stub's gpu_get_tex_filter is true).
#[test]
fn clouds_composite_restores_the_saved_filter() {
    use myco_app::render::batch::QuadBatch;
    use myco_app::render::mushrooms::{clouds_composite, CloudFrame};
    let f = CloudFrame {
        active: true,
        sw: 8,
        sh: 8,
        px: 4.0,
        ..Default::default()
    };
    for start in [false, true] {
        let mut b = QuadBatch::new();
        b.set_filter(start);
        clouds_composite(&mut b, &f);
        assert_eq!(b.filter(), start);
    }
}
