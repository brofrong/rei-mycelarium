//! World draw order as Bevy `z` (larger z on top). The world camera (Task 4: transform z = 0,
//! `OrthographicProjection::default_2d`) drew nothing at z = -900..-360 in the Task 16
//! screenshots while z in 0..1000 works (cause not pinned down), so every layer lives in 0..1000.
//!
//! The original draws Room1's instances, all at depth 100, in creation order: obj_game,
//! obj_forest_controller, obj_mycelium_controller, obj_life, obj_hud; first every Draw_0, then
//! every Draw_73 (Draw End). Only the forest, mycelium and life have world Draw events, so:
//!
//! 1. forest Draw_0: backdrop layers, backdrop leaves, rain band 0 (`weather_draw_back`),
//!    `world_draw` (T13), `worms_draw` (T15), roots, stumps, decor trees / bushes / plants,
//!    small trees, trees;
//! 2. mycelium Draw_0 (T14);
//! 3. life Draw_0 (T15);
//! 4. forest Draw_73: `world_draw_turf` (T13), then `weather_draw_front` (gloom over everything
//!    drawn so far, rain bands 1-2, splashes).
//!
//! Each layer owns `[Z_*, Z_* + LAYER_SPAN)`; items inside a layer keep the original painter's
//! order with [`z_at`].

/// Width of the z range a layer may use.
pub const LAYER_SPAN: f32 = 10.0;
/// z step between consecutive items of one layer (`LAYER_SPAN / Z_STEP` = 5000 items).
pub const Z_STEP: f32 = 0.002;

pub const Z_BACKDROP: f32 = 50.0;
pub const Z_BACKDROP_LEAVES: f32 = 70.0;
pub const Z_WEATHER_BACK: f32 = 90.0;
/// `world_draw` (T13); its sub-layers may use up to `Z_WORMS - LAYER_SPAN` (320 wide).
pub const Z_WORLD: f32 = 110.0;
pub const Z_WORMS: f32 = 440.0;
pub const Z_FOREST_ROOTS: f32 = 460.0;
pub const Z_FOREST_STUMPS: f32 = 480.0;
/// decor trees, bushes and plants (interleaved per chunk, as the original draws them)
pub const Z_FOREST_UNDERGROWTH: f32 = 500.0;
pub const Z_FOREST_SMALL_TREES: f32 = 520.0;
pub const Z_FOREST_TREES: f32 = 540.0;
/// mycelium Draw_0 (T14); may use up to `Z_LIFE - LAYER_SPAN` (130 wide).
pub const Z_MYCELIUM: f32 = 560.0;
/// life Draw_0 (T15); may use up to `Z_TURF - LAYER_SPAN` (130 wide).
pub const Z_LIFE: f32 = 700.0;
/// forest Draw_73: `world_draw_turf` (T13)
pub const Z_TURF: f32 = 840.0;
pub const Z_WEATHER_GLOOM: f32 = 860.0;
pub const Z_WEATHER_FRONT: f32 = 880.0;
pub const Z_WEATHER_SPLASHES: f32 = 900.0;
/// obj_game Draw_77 (post) and later world-space effects
pub const Z_POST: f32 = 920.0;

/// z of the `i`-th item drawn in the layer starting at `base`.
pub fn z_at(base: f32, i: usize) -> f32 {
    base + (i.min(4999) as f32) * Z_STEP
}
