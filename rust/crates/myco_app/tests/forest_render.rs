//! Task 16: surface forest rendering (obj_forest_controller Draw_0 / Draw_73, backdrop leaves,
//! foreground-leaf art).

use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::camera::Cameras;
use myco_app::display::SafeArea;
use myco_app::render::forest::{
    forest_draw, forest_draw_list, ForestDraw, ForestEntities, ForestItem, ForestKind,
};
use myco_app::render::layers;
use myco_app::render::leaf_art::{leaf_fg_art_init, PagePixels};
use myco_app::sim_bridge::SimRes;
use myco_core::sim::{Sim, ViewRect};
use myco_core::state::GameState;
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

fn fixture_state(name: &str) -> GameState {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../myco_core/tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(text.trim_end_matches('\0')).unwrap()
}

fn chunk_x(key: &str) -> i64 {
    key.trim_start_matches('s').parse().unwrap()
}

/// What a saved forest item must look like on screen: kind, sprite, x, y, flip, part.
type Want = (ForestKind, String, f64, f64, f64, Option<i64>);

fn wanted_items(state: &GameState) -> Vec<(i64, Want)> {
    let mut out = Vec::new();
    for (key, ch) in &state.forest.chunks {
        let cx = chunk_x(key);
        for t in &ch.trees {
            out.push((
                cx,
                (ForestKind::Tree, t.sprite.clone(), t.x, t.y, t.flip, None),
            ));
        }
        for t in &ch.small_trees {
            out.push((
                cx,
                (
                    ForestKind::SmallTree,
                    t.sprite.clone(),
                    t.x,
                    t.y,
                    t.flip,
                    None,
                ),
            ));
        }
        for p in &ch.bushes {
            out.push((
                cx,
                (
                    ForestKind::Bush,
                    "bush_tree".into(),
                    p.x,
                    p.y,
                    p.flip,
                    Some(p.part),
                ),
            ));
        }
        for p in &ch.plants {
            out.push((
                cx,
                (
                    ForestKind::Plant,
                    "grass".into(),
                    p.x,
                    p.y,
                    p.flip,
                    Some(p.part),
                ),
            ));
        }
        for p in &ch.decor_trees {
            let sprite = myco_core::forest::decor_tree_parts()[p.part as usize].sprite;
            out.push((
                cx,
                (
                    ForestKind::DecorTree,
                    sprite.into(),
                    p.x,
                    p.y,
                    p.flip,
                    Some(p.part),
                ),
            ));
        }
        for s in &ch.stumps {
            out.push((
                cx,
                (
                    ForestKind::Stump,
                    "stump".into(),
                    s.x,
                    s.y,
                    s.flip,
                    Some(s.part),
                ),
            ));
        }
    }
    out
}

/// Cameras whose render view shows `[l, r]` horizontally around the surface.
fn cameras_showing(l: f64, r: f64) -> Cameras {
    let mut cams = Cameras::new(
        1920.0,
        1080.0,
        1.0,
        SafeArea {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        },
    );
    cams.zoom = cams.game_view.w / (r - l);
    cams.set_game_view_center((l + r) * 0.5, 300.0);
    cams
}

#[test]
fn saved_forest_chunk_restored_exactly() {
    let state = fixture_state("exploration.json");
    let want = wanted_items(&state);
    assert!(want.len() >= 6, "fixture has forest items");
    let sim = Sim::from_save(state.clone()).unwrap();

    // core state: every saved chunk is loaded with the same items in the same order
    for (key, ch) in &state.forest.chunks {
        let rt = &sim.forest.chunks[&chunk_x(key)];
        let trees: Vec<_> = rt
            .trees
            .iter()
            .map(|t| (t.sprite.clone(), t.x, t.y, t.flip))
            .collect();
        let saved: Vec<_> = ch
            .trees
            .iter()
            .map(|t| (t.sprite.clone(), t.x, t.y, t.flip))
            .collect();
        assert_eq!(trees, saved, "{key} trees");
        let st: Vec<_> = rt
            .small_trees
            .iter()
            .map(|t| (t.sprite.clone(), t.x, t.y, t.flip))
            .collect();
        let ss: Vec<_> = ch
            .small_trees
            .iter()
            .map(|t| (t.sprite.clone(), t.x, t.y, t.flip))
            .collect();
        assert_eq!(st, ss, "{key} small trees");
        for (name, a, b) in [
            ("bushes", &rt.bushes, &ch.bushes),
            ("plants", &rt.plants, &ch.plants),
            ("decor", &rt.decor_trees, &ch.decor_trees),
        ] {
            let a: Vec<_> = a.iter().map(|p| (p.part, p.x, p.y, p.flip)).collect();
            let b: Vec<_> = b.iter().map(|p| (p.part, p.x, p.y, p.flip)).collect();
            assert_eq!(a, b, "{key} {name}");
        }
        let a: Vec<_> = rt
            .stumps
            .iter()
            .map(|s| (s.part as i64, s.x, s.y, s.flip))
            .collect();
        let b: Vec<_> = ch
            .stumps
            .iter()
            .map(|s| (s.part, s.x, s.y, s.flip))
            .collect();
        assert_eq!(a, b, "{key} stumps");
    }

    let Some(assets) = assets() else { return };
    // a render view wide enough to show every saved chunk
    let cw = sim.forest.chunk_width;
    let keys: Vec<i64> = state.forest.chunks.keys().map(|k| chunk_x(k)).collect();
    let (lo, hi) = (*keys.iter().min().unwrap(), *keys.iter().max().unwrap());
    let cams = cameras_showing(lo as f64 * cw, (hi + 1) as f64 * cw - 1.0);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(SimRes(sim))
        .insert_resource(assets)
        .insert_resource(cams)
        .init_resource::<ForestEntities>()
        .add_systems(Update, forest_draw);
    app.update();
    app.update(); // second pass updates in place: still one entity per item

    let world = app.world_mut();
    let mut q = world.query::<&ForestItem>();
    let items: Vec<ForestItem> = q.iter(world).cloned().collect();
    for (cx, (kind, sprite, x, y, flip, part)) in &want {
        let n = items
            .iter()
            .filter(|i| {
                i.kind == *kind
                    && i.sprite == *sprite
                    && i.x == *x
                    && i.y == *y
                    && i.flip == *flip
                    && i.part == *part
            })
            .count();
        assert_eq!(
            n, 1,
            "chunk {cx}: {kind:?} {sprite} x={x} part={part:?} drawn once"
        );
    }
    let ents = world.resource::<ForestEntities>().len();
    assert_eq!(ents, items.len(), "entity map matches spawned items");
}

#[test]
fn forest_draw_order_follows_draw_0() {
    let Some(assets) = assets() else { return };
    let mut sim = Sim::from_save(fixture_state("exploration.json")).unwrap();
    let rv = ViewRect {
        x: -1200.0,
        y: 0.0,
        w: 2400.0,
        h: 1350.0,
    };
    let list: Vec<ForestDraw> = forest_draw_list(&mut sim, &assets, rv.x, &rv);
    // Draw_0 order: backdrop layers, roots, stumps, decor/bushes/plants, small trees, trees
    let rank = |k: ForestKind| match k {
        ForestKind::Backdrop => 0,
        ForestKind::TreeRoots => 1,
        ForestKind::Stump => 2,
        ForestKind::DecorTree | ForestKind::Bush | ForestKind::Plant => 3,
        ForestKind::SmallTree => 4,
        ForestKind::Tree => 5,
        ForestKind::BackdropLeaf => unreachable!("backdrop leaves are drawn separately"),
    };
    let mut last = (-1, f32::MIN);
    for d in &list {
        let r = rank(d.item.kind);
        assert!(r >= last.0, "{:?} after rank {}", d.item.kind, last.0);
        assert!(d.cmd.z > last.1, "z strictly increases in draw order");
        last = (r, d.cmd.z);
    }
    for k in [
        ForestKind::Backdrop,
        ForestKind::TreeRoots,
        ForestKind::Stump,
        ForestKind::Plant,
        ForestKind::SmallTree,
        ForestKind::Tree,
    ] {
        assert!(list.iter().any(|d| d.item.kind == k), "{k:?} drawn");
    }
    // the whole forest sits between the world (T13) and the mycelium (T14)
    assert!(list.iter().all(|d| d.cmd.z < layers::Z_MYCELIUM));
    let first_item = list
        .iter()
        .find(|d| d.item.kind != ForestKind::Backdrop)
        .unwrap();
    assert!(first_item.cmd.z > layers::Z_WORMS);
    assert!(list[0].cmd.z < layers::Z_BACKDROP_LEAVES);
}

#[test]
fn layer_order_matches_original_draw_events() {
    use layers::*;
    // forest Draw_0, then mycelium Draw_0, life Draw_0, then forest Draw_73 (turf, weather front)
    let order = [
        Z_BACKDROP,
        Z_BACKDROP_LEAVES,
        Z_WEATHER_BACK,
        Z_WORLD,
        Z_WORMS,
        Z_FOREST_ROOTS,
        Z_FOREST_STUMPS,
        Z_FOREST_UNDERGROWTH,
        Z_FOREST_SMALL_TREES,
        Z_FOREST_TREES,
        Z_MYCELIUM,
        Z_LIFE,
        Z_TURF,
        Z_WEATHER_GLOOM,
        Z_WEATHER_FRONT,
        Z_WEATHER_SPLASHES,
        Z_POST,
    ];
    for w in order.windows(2) {
        assert!(w[0] + LAYER_SPAN <= w[1], "{} then {}", w[0], w[1]);
    }
    assert!(order[0] >= 0.0 && *order.last().unwrap() + LAYER_SPAN < 1000.0);
    assert_eq!(z_at(Z_FOREST_TREES, 0), Z_FOREST_TREES);
    assert!(z_at(Z_FOREST_TREES, 4999) < Z_FOREST_TREES + LAYER_SPAN);
}

#[test]
fn leaf_fg_art_matches_core_bounds() {
    let Some(assets) = assets() else { return };
    let pages = PagePixels::load(&assets_dir(), &assets, &["leaf_fg_anim"]).unwrap();
    let art = leaf_fg_art_init(&assets, &pages);
    let core = &myco_core::world::art::art().leaf_fg;
    let close = |a: f64, b: f64, what: &str| assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
    let cmp = |got: &myco_app::render::leaf_art::LeafFgSprite,
               want: &myco_core::world::art::LeafFgFrame,
               what: &str| {
        close(got.x1, want.x1, what);
        close(got.y1, want.y1, what);
        close(got.x2, want.x2, what);
        close(got.y2, want.y2, what);
        close(got.mid_x, want.mid_x, what);
        close(got.mid_y, want.mid_y, what);
    };
    assert_eq!(art.turn.len(), 14);
    cmp(&art.rest, &core.rest, "rest");
    cmp(&art.turned, &core.turned, "turned");
    for (i, (g, w)) in art.turn.iter().zip(&core.turn).enumerate() {
        cmp(g, w, &format!("turn {i}"));
    }
    close(art.carry_x, core.carry_x, "carry_x");
    close(art.carry_y, core.carry_y, "carry_y");
    // every frame is a 96-px-wide strip of the sheet, anchored on the reference point
    assert_eq!(art.turn[3].part, [3.0 * 96.0, 0.0, 96.0, 75.0]);
}
