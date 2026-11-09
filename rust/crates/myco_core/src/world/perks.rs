//! Cave perks (scr_world): which perks the colony has and the multipliers they give. Perks
//! live in `global.cave_perks` (`state.cave_perks`, saved), keyed by cave id ("blue",
//! "purple", "red", "green", "yellow"); `caves::cave_take` grants them.

use crate::balance::{bal, game_mode, gml_truthy};
use crate::state::GameState;

// port: perk_has
/// `perk_has(id)`: the perk was granted (GML truthiness: `true`, or a number above 0.5).
pub fn perk_has(state: &GameState, id: &str) -> bool {
    gml_truthy(state.state.cave_perks.get(id))
}

// port: perk_time_mult
/// `perk_time_mult()`: the yellow perk shortens mushroom and animal periods.
pub fn perk_time_mult(state: &GameState) -> f64 {
    if perk_has(state, "yellow") {
        return 1.0 - bal(game_mode(state), "perk_yellow_time_cut");
    }
    1.0
}

/// `perk_cost_mult()`: the green perk discounts mushroom costs (ported in `balance`, Task 3).
pub fn perk_cost_mult(state: &GameState) -> f64 {
    crate::balance::perk_cost_mult(state)
}
