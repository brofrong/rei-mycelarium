//! Mushrooms (`global.mushrooms`, scr_mushrooms): growth, production, repeller, stinky,
//! consumptor. Task 9 ports it; the live list is `sim.state.mushrooms`.

use crate::balance::{experience_price_mult, perk_cost_mult};
use crate::gml;
use crate::progress::upgrades::{upg_is_owned, upg_level, upg_max_level};
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

// ---------------------------------------------------------------------------------------------
// Type catalogue read by the HUD build bar (Task 17). These are faithful copies of Task 9's
// rows (`mushroom_type_*`, `shroom_sheet_cells`, `mushroom_cell_left`, `consumptor_variant`)
// without `// port:` markers; Task 9 adds the markers here (or replaces them with its own
// functions and keeps these names), like the R16 follow-ups.

/// `mushroom_type_unlock_id(t)`: the upgrade that unlocks the type.
pub fn type_unlock_id(t: MushroomKind) -> &'static str {
    match t {
        MushroomKind::Spore => "unlock_spore",
        MushroomKind::Predator => "unlock_predator",
        MushroomKind::Repeller => "unlock_repeller",
        MushroomKind::Tree => "unlock_tree",
        MushroomKind::Digger => "unlock_digger",
        MushroomKind::Stinky => "unlock_stinky",
        MushroomKind::Experience => "unlock_experience",
        MushroomKind::Consumptor => "unlock_parasitism",
    }
}

/// `mushroom_type_unlocked(t)`.
pub fn type_unlocked(st: &GameState, t: MushroomKind) -> bool {
    upg_is_owned(st, type_unlock_id(t))
}

/// `mushroom_type_cost_base(t)` (food).
pub fn type_cost_base(sim: &Sim, t: MushroomKind) -> f64 {
    match t {
        MushroomKind::Experience => {
            gml::round(sim.bal("experience_mushroom_cost") * experience_price_mult(&sim.state))
        }
        MushroomKind::Consumptor => sim.bal("consumptor_mushroom_cost"),
        MushroomKind::Spore => sim.bal("spore_mushroom_cost"),
        MushroomKind::Predator => sim.bal("predator_mushroom_cost"),
        MushroomKind::Repeller => sim.bal("repeller_mushroom_cost"),
        MushroomKind::Tree => sim.bal("tree_mushroom_cost"),
        MushroomKind::Digger => sim.bal("digger_mushroom_cost"),
        MushroomKind::Stinky => sim.bal("stinky_mushroom_cost"),
    }
}

/// `mushroom_type_water_cost_base(t)`.
pub fn type_water_cost_base(sim: &Sim, t: MushroomKind) -> f64 {
    match t {
        MushroomKind::Spore => sim.bal("spore_mushroom_water_cost"),
        MushroomKind::Predator => sim.bal("predator_mushroom_water_cost"),
        MushroomKind::Repeller => sim.bal("repeller_mushroom_water_cost"),
        MushroomKind::Tree => sim.bal("tree_mushroom_water_cost"),
        MushroomKind::Stinky => sim.bal("stinky_mushroom_water_cost"),
        MushroomKind::Experience => gml::round(
            sim.bal("experience_mushroom_water_cost") * experience_price_mult(&sim.state),
        ),
        MushroomKind::Consumptor => sim.bal("consumptor_mushroom_water_cost"),
        MushroomKind::Digger => {
            let k = upg_level(&sim.state, "digger_length") as f64
                / upg_max_level(&sim.state, "digger_length").max(1.0);
            gml::round(gml::lerp(
                sim.bal("digger_water_cost_min"),
                sim.bal("digger_water_cost_max"),
                k,
            ))
        }
    }
}

/// `mushroom_type_cost(t)`.
pub fn type_cost(sim: &Sim, t: MushroomKind) -> f64 {
    gml::round(type_cost_base(sim, t) * perk_cost_mult(&sim.state))
}

/// `mushroom_type_water_cost(t)`.
pub fn type_water_cost(sim: &Sim, t: MushroomKind) -> f64 {
    gml::round(type_water_cost_base(sim, t) * perk_cost_mult(&sim.state))
}

/// `mushroom_type_affordable(t)`: enough food and water.
pub fn type_affordable(sim: &Sim, t: MushroomKind) -> bool {
    sim.state.state.food >= type_cost(sim, t) && sim.water_total() >= type_water_cost(sim, t)
}

/// `mushroom_type_sprite(t)`: the sprite sheet name.
pub fn type_sprite(t: MushroomKind) -> &'static str {
    match t {
        MushroomKind::Spore => "sporeshroom",
        MushroomKind::Predator => "predatorshroom",
        MushroomKind::Repeller => "explosiveshroom",
        MushroomKind::Tree => "treeshroom",
        MushroomKind::Digger => "diggershroom",
        MushroomKind::Stinky => "stinkyshroom",
        MushroomKind::Experience => "experienceshroom",
        MushroomKind::Consumptor => "consumptorshroom",
    }
}

/// `shroom_sheet_cells(spr)`: variants side by side in the sheet.
pub fn sheet_cells(sprite: &str) -> f64 {
    match sprite {
        "sporeshroom"
        | "explosiveshroom"
        | "explosiveshroom_exploded"
        | "predatorshroom"
        | "diggershroom"
        | "experienceshroom"
        | "consumptorshroom" => 2.0,
        "stinkyshroom" => 3.0,
        _ => 1.0,
    }
}

/// `mushroom_cell_width(spr)` for a sheet `sprite_width` px wide (`max(1, w div cells)`).
pub fn cell_width(sprite: &str, sprite_width: f64) -> f64 {
    (sprite_width / sheet_cells(sprite)).trunc().max(1.0)
}

/// `mushroom_cell_left(spr, v)`.
pub fn cell_left(sprite: &str, sprite_width: f64, v: f64) -> f64 {
    let mut l = v * cell_width(sprite, sprite_width);
    if sprite == "stinkyshroom" && v == 2.0 {
        l += 7.0;
    }
    if sprite == "experienceshroom" && v == 1.0 {
        l -= 4.0;
    }
    l
}

/// `consumptor_variant(side)`.
pub fn consumptor_variant(side: f64) -> f64 {
    if side < 0.0 {
        0.0
    } else {
        1.0
    }
}

/// `myc_place_check(type, x, y)` result (`hud_place_at`): where the mushroom would stand,
/// whether it may, and the consumptor's host side.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaceCheck {
    pub ok: bool,
    pub x: f64,
    pub y: f64,
    pub face: Option<f64>,
}

/// `myc_place_check(type, x, y)` (Task 8/9 hook): the placement preview the HUD draws in
/// placement mode. Until Task 9: `None` (no preview).
pub fn place_check(_sim: &Sim, _t: MushroomKind, _x: f64, _y: f64) -> Option<PlaceCheck> {
    None
}

/// The hover-bar progress of mushroom `i` (`predator_progress`, `tree_shroom_progress`,
/// `experience_progress`, `consumptor_progress`, `repeller_regen_progress` by type; Task 9
/// hook). Until Task 9: `None` (no bar).
pub fn hover_progress(_sim: &Sim, _i: usize) -> Option<f64> {
    None
}

/// `mushroom_hit_box(m)` of mushroom `i` as `[x1, y1, x2, y2]` (Task 9 hook). Until Task 9:
/// `None`.
pub fn hit_box(_sim: &Sim, _i: usize) -> Option<[f64; 4]> {
    None
}
