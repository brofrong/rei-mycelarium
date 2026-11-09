//! obj_game orchestration hooks. Task 10 ports `game_init` and `game_step` into these bodies
//! (ruling R8); until then they do nothing. `Sim` calls them from `new_game` (obj_game
//! Create_0) and from `step` (obj_game Step_0).

use crate::sim::{Sim, StepInput};

/// `game_init` (Task 10): balance/visual/worldgen init, upgrades/tutorial/tasks init, the
/// starting globals, `world_init` (Task 5, honours `sim.world_seed`), wind, `weather_init`,
/// camera box.
pub fn init(sim: &mut Sim) {
    // world_init(global.start_x) (Task 5); Task 10 ports the rest of game_init around it in
    // the original order (start_x = 0 and the resource globals before it, wind after)
    let start_x = sim.state.state.start_x;
    crate::world::world_init(sim, start_x);
}

/// `game_step` (Task 10): the per-frame game logic run from obj_game's Step event.
pub fn step(_sim: &mut Sim, _input: &StepInput) {}
