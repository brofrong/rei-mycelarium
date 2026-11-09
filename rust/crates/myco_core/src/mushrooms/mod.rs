//! Mushrooms (`global.mushrooms`, scr_mushrooms): growth, production, repeller, stinky,
//! consumptor. Task 9 ports it; the live list is `sim.state.mushrooms`.

use crate::sim::Sim;
use crate::state::GameState;

/// Mushroom types by their GML code (`mushroom_type_unlock_id`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MushroomKind {
    Spore,
    Predator,
    Repeller,
    Tree,
    Digger,
    Stinky,
    Experience,
    Consumptor,
}

impl MushroomKind {
    pub const ALL: [MushroomKind; 8] = [
        MushroomKind::Spore,
        MushroomKind::Predator,
        MushroomKind::Repeller,
        MushroomKind::Tree,
        MushroomKind::Digger,
        MushroomKind::Stinky,
        MushroomKind::Experience,
        MushroomKind::Consumptor,
    ];

    /// The `type` field of a mushroom (0..=7).
    pub fn code(self) -> i64 {
        self as i64
    }

    pub fn from_code(code: i64) -> Option<MushroomKind> {
        usize::try_from(code)
            .ok()
            .and_then(|i| MushroomKind::ALL.get(i).copied())
    }
}

/// Runtime mushroom state outside `GameState` (caches, art). Task 9 fills it.
#[derive(Clone, Debug, Default)]
pub struct MushroomsRt {}

/// Per-frame mushroom update (Task 9); called from `game_step` (Task 10).
pub fn step(_sim: &mut Sim) {}

/// `load_game`'s mushroom part beyond the data fixups `Sim::from_save` does (Task 9).
pub fn on_load(_sim: &mut Sim) {}

/// `save_game`'s mushroom part beyond the transforms `Sim::to_save` does (Task 9).
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
