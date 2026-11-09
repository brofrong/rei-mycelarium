//! World rendering. Task 16: the surface forest and the weather; Task 14: the mycelium; `layers` fixes the draw order
//! all render tasks share, `sprite` the GameMaker-style sprite helpers.
pub mod batch;
pub mod blend;
pub mod forest;
pub mod layers;
pub mod leaf_art;
pub mod life;
pub mod mushrooms;
pub mod mycelium;
pub mod mycelium_glow;
pub mod post;
pub mod sprite;
pub mod weather;
pub mod world;

use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::sim_bridge::SimRes;
use bevy::prelude::*;

/// After every fixed step: the world/forest chunk generation the original's draw does for the
/// view ([`world::world_draw_generate`], [`forest::forest_draw_generate`]: forest chunks draw
/// from the game RNG). GameMaker (and LÖVE) draw once after each step; doing this here, for the
/// render view, keeps the game RNG and the saved chunks changing only inside steps, in LÖVE's
/// order, whatever the frame rate. The frame's draws then find their chunks ready.
///
/// Also the lake sparkles' state ([`world::world_lakes_step`]: their respawns advance the saved
/// `flare_seq`); they start over after a save (`save_game` sets `lk.flares = undefined`) and in a
/// replaced game.
pub fn draw_generate_system(
    mut sim: ResMut<SimRes>,
    cams: Res<Cameras>,
    assets: Option<Res<GameAssets>>,
    flares: Option<ResMut<world::LakeFlares>>,
    mut events: MessageReader<crate::sim_bridge::SimEventMsg>,
) {
    let saved = events
        .read()
        .any(|e| matches!(e.0, myco_core::events::SimEvent::Saved));
    let Some(assets) = assets else { return };
    let rv = cams.render_view();
    let sim = &mut sim.0;
    world::world_draw_generate(sim, rv.x, rv.y, rv.w, rv.h);
    if let Some(mut flares) = flares {
        if saved || flares.session != sim.session {
            flares.map.clear();
            flares.session = sim.session;
        }
        let now = sim.now_ms();
        world::world_lakes_step(&mut sim.world, &mut flares, now, rv.x, rv.y, rv.w, rv.h);
    }
    forest::forest_draw_generate(sim, &assets, &rv);
}

/// `global.art_preloaded`: the art that reads sprite pixels is built.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct ArtPreloaded(pub bool);

/// The startup systems that build the pixel-reading art `art_preload_step` builds.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtPreloadSet;

// port: art_preload_step
/// `art_preload_step` (obj_game Draw_77): once, build the art inits that read sprite pixels
/// (stones_init, stone_tiles_init, caves_mask_init, lakes_init → `world::build_world_art`;
/// hedgehog_back_init, squirrel_art_init → `life::build_life_art`; leaf_art_init,
/// leaf_fg_art_init → `forest::build_leaf_art`; stinky_art_init, repeller_cloud_art_init →
/// `mushrooms::build_fx_art`). The original waits for a focused window with a surface (GPU
/// surfaces need one); Bevy builds images without a window, so they are built at startup, in
/// [`ArtPreloadSet`], and this marks them done.
pub fn art_preload_step(mut done: ResMut<ArtPreloaded>) {
    if !done.0 {
        done.0 = true;
    }
}

/// Registers [`art_preload_step`] after every art build in [`ArtPreloadSet`].
pub struct ArtPreloadPlugin;

impl Plugin for ArtPreloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArtPreloaded>()
            .add_systems(Startup, art_preload_step.after(ArtPreloadSet));
    }
}
