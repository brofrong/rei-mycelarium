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

// ---------------------------------------------------------------------------------------------
// Task 8 needs (the mycelium's build menu and placement). Task 9 owns the mushroom functions,
// Task 10 the squirrel ones. The pure ones are ported here faithfully but without `// port:`
// markers (their owners mark them, or move them and keep these names); the ones that need
// Task 9's mushroom runtime or art are HOOKS with neutral bodies, to be replaced by the real
// port: `mushroom_add`, `mushroom_hit_box`, `consumptors_on`, `set_consumptor`, and Task 10's
// `squirrel_edge_x`.

use crate::balance::{bal, experience_price_mult, game_mode, perk_cost_mult};
use crate::forest::{stump_parts, HostId};
use crate::gml;
use crate::progress::upgrades::{upg_bonus, upg_is_owned, upg_level, upg_max_level};
use crate::state::MushroomSave;

/// `mushroom_type_unlock_id(t)`.
pub fn mushroom_type_unlock_id(t: MushroomKind) -> &'static str {
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

/// `mushroom_type_name_key(t)`.
pub fn mushroom_type_name_key(t: MushroomKind) -> String {
    format!("upg_{}", mushroom_type_unlock_id(t))
}

/// `mushroom_type_unlocked(t)`.
pub fn mushroom_type_unlocked(st: &GameState, t: MushroomKind) -> bool {
    upg_is_owned(st, mushroom_type_unlock_id(t))
}

/// `mushroom_type_cost_base(t)`.
pub fn mushroom_type_cost_base(st: &GameState, t: MushroomKind) -> f64 {
    let mode = game_mode(st);
    match t {
        MushroomKind::Experience => {
            gml::round(bal(mode, "experience_mushroom_cost") * experience_price_mult(st))
        }
        MushroomKind::Consumptor => bal(mode, "consumptor_mushroom_cost"),
        MushroomKind::Spore => bal(mode, "spore_mushroom_cost"),
        MushroomKind::Predator => bal(mode, "predator_mushroom_cost"),
        MushroomKind::Repeller => bal(mode, "repeller_mushroom_cost"),
        MushroomKind::Tree => bal(mode, "tree_mushroom_cost"),
        MushroomKind::Digger => bal(mode, "digger_mushroom_cost"),
        MushroomKind::Stinky => bal(mode, "stinky_mushroom_cost"),
    }
}

/// `mushroom_type_water_cost_base(t)`.
pub fn mushroom_type_water_cost_base(st: &GameState, t: MushroomKind) -> f64 {
    let mode = game_mode(st);
    match t {
        MushroomKind::Spore => bal(mode, "spore_mushroom_water_cost"),
        MushroomKind::Predator => bal(mode, "predator_mushroom_water_cost"),
        MushroomKind::Repeller => bal(mode, "repeller_mushroom_water_cost"),
        MushroomKind::Tree => bal(mode, "tree_mushroom_water_cost"),
        MushroomKind::Stinky => bal(mode, "stinky_mushroom_water_cost"),
        MushroomKind::Experience => {
            gml::round(bal(mode, "experience_mushroom_water_cost") * experience_price_mult(st))
        }
        MushroomKind::Consumptor => bal(mode, "consumptor_mushroom_water_cost"),
        MushroomKind::Digger => {
            let k =
                upg_level(st, "digger_length") as f64 / upg_max_level(st, "digger_length").max(1.0);
            gml::round(gml::lerp(
                bal(mode, "digger_water_cost_min"),
                bal(mode, "digger_water_cost_max"),
                k,
            ))
        }
    }
}

/// `mushroom_type_cost(t)`.
pub fn mushroom_type_cost(st: &GameState, t: MushroomKind) -> f64 {
    gml::round(mushroom_type_cost_base(st, t) * perk_cost_mult(st))
}

/// `mushroom_type_water_cost(t)`.
pub fn mushroom_type_water_cost(st: &GameState, t: MushroomKind) -> f64 {
    gml::round(mushroom_type_water_cost_base(st, t) * perk_cost_mult(st))
}

/// `mushroom_type_affordable(t)`.
pub fn mushroom_type_affordable(sim: &Sim, t: MushroomKind) -> bool {
    sim.state.state.food >= mushroom_type_cost(&sim.state, t)
        && sim.water_total() >= mushroom_type_water_cost(&sim.state, t)
}

/// `repeller_regen_price()`.
pub fn repeller_regen_price(st: &GameState) -> f64 {
    let k = (1.0 - upg_bonus(st, "repeller_regen_cost")).max(0.2);
    gml::round(bal(game_mode(st), "repeller_regen_cost") * k).max(1.0)
}

/// `repeller_max_charges()`.
pub fn repeller_max_charges(st: &GameState) -> f64 {
    bal(game_mode(st), "repeller_base_charges").max(1.0)
}

/// `consume_info(host)` (`___struct___86/87`): slots, minutes and food of a host.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConsumeInfo {
    pub slots: f64,
    pub minutes: f64,
    pub food: f64,
}

/// `consume_info(host)`.
pub fn consume_info(sim: &Sim, host: HostId) -> ConsumeInfo {
    if let HostId::Stump(uid) = host {
        if let Some(s) = sim.forest.stump(uid) {
            let part = &stump_parts()[s.part];
            return ConsumeInfo {
                slots: part.slots.len() as f64,
                minutes: sim.bal(&format!("consumptor_{}_minutes", part.consume)),
                food: sim.bal(&format!("consumptor_{}_food", part.consume)),
            };
        }
    }
    ConsumeInfo {
        slots: sim.bal("consumptor_tree_slots"),
        minutes: sim.bal("consumptor_tree_minutes"),
        food: sim.bal("consumptor_tree_food"),
    }
}

/// `consumptor_variant(side)`.
pub fn consumptor_variant(side: f64) -> i64 {
    if side < 0.0 {
        0
    } else {
        1
    }
}

/// `squirrel_tree_has_mushroom(tree)` (Task 10's): a tree mushroom already on the tree.
pub fn squirrel_tree_has_mushroom(sim: &Sim, tree_uid: u64) -> bool {
    let Some(t) = sim.forest.tree(tree_uid) else {
        return false;
    };
    let off = crate::balance::vis("tree_mushroom_offset_x");
    sim.state
        .mushrooms
        .iter()
        .any(|m| m.kind == 3 && !m.stolen && (m.x - off - t.x).abs() < 2.0)
}

/// HOOK (Task 9): `mushroom_add(type, x, y)` → index in `sim.state.mushrooms`. Neutral body:
/// pushes a default mushroom of that type; Task 9 replaces it with the real port (variant and
/// fly rolls, timers, grow sound, task stats).
pub fn mushroom_add(sim: &mut Sim, kind: MushroomKind, x: f64, y: f64) -> usize {
    sim.state.mushrooms.push(MushroomSave {
        kind: kind.code(),
        x,
        y,
        charges: repeller_max_charges(&sim.state),
        ..MushroomSave::default()
    });
    sim.state.mushrooms.len() - 1
}

/// HOOK (Task 9): `mushroom_hit_box(m)` as (x1, y1, x2, y2), needs the mushroom art. Neutral
/// body: no box (mushrooms are not clickable until Task 9).
pub fn mushroom_hit_box(_sim: &Sim, _index: usize) -> Option<(f64, f64, f64, f64)> {
    None
}

/// HOOK (Task 9): `consumptors_on(host)`, indices of the live consumptors on a host. Needs
/// Task 9's runtime host references; neutral body: none.
pub fn consumptors_on(_sim: &Sim, _host: HostId) -> Vec<usize> {
    Vec::new()
}

/// HOOK (Task 9): set a new consumptor's `host`, `side` and `variant` (`myc_place_mushroom`).
/// Neutral body stores side and variant; Task 9 also keeps the host reference.
pub fn set_consumptor(sim: &mut Sim, index: usize, _host: Option<HostId>, side: f64, variant: i64) {
    let m = &mut sim.state.mushrooms[index];
    m.side = side;
    m.variant = variant;
}

/// HOOK (Task 10): `squirrel_edge_x(tree, side, y)`, the trunk edge at a height (needs the
/// trunk pixel edges). Neutral body: the trunk centre.
pub fn squirrel_edge_x(sim: &Sim, tree_uid: u64, _side: f64, _y: f64) -> f64 {
    sim.forest.tree(tree_uid).map_or(0.0, |t| t.x)
}

// ---------------------------------------------------------------------------------------------
// Sprite-sheet helpers the HUD build bar uses (Task 17). Faithful copies of Task 9's
// `mushroom_type_sprite`, `shroom_sheet_cells`, `mushroom_cell_width`, `mushroom_cell_left`
// without `// port:` markers; Task 9 marks them here or replaces them keeping these names.

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

/// The hover-bar progress of mushroom `i` (`predator_progress`, `tree_shroom_progress`,
/// `experience_progress`, `consumptor_progress`, `repeller_regen_progress` by type; Task 9
/// hook). Until Task 9: `None` (no bar).
pub fn hover_progress(_sim: &Sim, _i: usize) -> Option<f64> {
    None
}
