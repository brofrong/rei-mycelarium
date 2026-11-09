//! Task 21: drawing the intro and the migration cut-scene.
use bevy::prelude::*;
use myco_app::assets::GameAssets;
use myco_app::cine::{cine_draw, intro_draw_list};
use myco_app::render::sprite::SpriteSrc;
use myco_app::ui::gm_draw::{DrawItem, Gm};
use myco_core::cine::{self, CinePhase};
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use std::path::Path;

fn assets() -> Option<GameAssets> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build/assets");
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

#[test]
fn intro_draws_the_hedgehog_and_the_falling_mushroom() {
    let Some(assets) = assets() else {
        return;
    };
    let mut sim = Sim::new_game(GameMode::Exploration, "intro", Some(31607));
    assert!(sim.intro_active, "new games start in the intro");
    let list = intro_draw_list(&sim, &assets);
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].sprite_name(), "hedgehog");
    assert!(matches!(list[0].src, SpriteSrc::Part { .. }));
    assert!(list[0].xs < 0.0, "mirrored to face right");
    assert_eq!(list[1].sprite_name(), "originshroom");
    // carried on its back
    assert_eq!(list[1].x, sim.life.intro.x.floor());
    // dropped: falls at the start point
    let start_x = sim.state.state.start_x;
    let mut guard = 0;
    while !sim.life.intro.dropped && guard < 10_000 {
        myco_core::camera::intro_step(&mut sim);
        guard += 1;
    }
    let list = intro_draw_list(&sim, &assets);
    assert_eq!(list[1].x, start_x.floor());
    assert_eq!(list[1].y, sim.life.intro.shroom_y.floor());
    sim.intro_active = false;
    assert!(intro_draw_list(&sim, &assets).is_empty());
}

#[test]
fn cine_draw_paints_the_flash_then_the_planet() {
    let Some(assets) = assets() else {
        return;
    };
    let mut sim = Sim::new_game(GameMode::Exploration, "cine", Some(31607));
    sim.intro_active = false;
    let mut g = Gm::new(&assets);
    cine_draw(&mut g, &sim, 1920.0, 1080.0);
    assert!(g.items.is_empty(), "nothing without a cut-scene");
    cine::cine_show_start(&mut sim);
    let mut g = Gm::new(&assets);
    cine_draw(&mut g, &sim, 1920.0, 1080.0);
    assert!(
        g.items.is_empty(),
        "the tour before the burst draws nothing"
    );
    sim.progress.tasks.cine.as_mut().unwrap().white = Some(0.5);
    let mut g = Gm::new(&assets);
    cine_draw(&mut g, &sim, 1920.0, 1080.0);
    assert!(!g.items.is_empty(), "the white flash");
    let mut guard = 0;
    while sim.progress.tasks.cine.as_ref().unwrap().phase == CinePhase::Show && guard < 5000 {
        cine::cine_step(&mut sim);
        guard += 1;
    }
    // into the planet: past the mycelium start so the segments draw too
    for _ in 0..120 {
        cine::cine_step(&mut sim);
    }
    let mut g = Gm::new(&assets);
    cine_draw(&mut g, &sim, 1920.0, 1080.0);
    let quads = g
        .items
        .iter()
        .filter(|i| matches!(i, DrawItem::Quad { .. }))
        .count();
    assert_eq!(quads, 1, "the Earth");
    let tris: usize = g
        .items
        .iter()
        .map(|i| match i {
            DrawItem::Tris(t) => t.len() / 3,
            _ => 0,
        })
        .sum();
    assert!(tris > 1000, "stars, glow and mycelium: {tris}");
}
