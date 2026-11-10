//! The mushroom type catalogue (scr_life, scr_game): unlocks, names, costs, the sprite sheets
//! the logic measures (cells, cell widths, hit boxes), variants, the grow-in scale, repeller
//! and experience periods and the hover-bar progress values.

use super::MushroomKind;
use crate::balance::{bal, experience_price_mult, game_mode, perk_cost_mult, vis};
use crate::gml;
use crate::progress::upgrades::{upg_bonus, upg_is_owned, upg_level, upg_max_level};
use crate::sim::Sim;
use crate::state::{GameState, MushroomSave};
use crate::world::perks::perk_time_mult;

/// Size, origin and bounding box of a mushroom sprite sheet (sprites.json; the golden checks
/// them against `sprite_get_*` in the LÖVE port).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShroomSprite {
    pub name: &'static str,
    pub w: f64,
    pub h: f64,
    pub xoff: f64,
    pub yoff: f64,
    pub bbox_l: f64,
    pub bbox_t: f64,
    pub bbox_r: f64,
    pub bbox_b: f64,
}

const fn spr(name: &'static str, w: f64, h: f64, origin: [f64; 2], bbox: [f64; 4]) -> ShroomSprite {
    ShroomSprite {
        name,
        w,
        h,
        xoff: origin[0],
        yoff: origin[1],
        bbox_l: bbox[0],
        bbox_t: bbox[1],
        bbox_r: bbox[2],
        bbox_b: bbox[3],
    }
}

const SPRITES: [ShroomSprite; 9] = [
    spr(
        "sporeshroom",
        96.0,
        64.0,
        [26.0, 64.0],
        [4.0, 15.0, 84.0, 63.0],
    ),
    spr(
        "predatorshroom",
        128.0,
        60.0,
        [31.0, 60.0],
        [19.0, 8.0, 109.0, 59.0],
    ),
    spr(
        "explosiveshroom",
        96.0,
        48.0,
        [24.0, 48.0],
        [10.0, 10.0, 90.0, 47.0],
    ),
    spr(
        "explosiveshroom_exploded",
        96.0,
        48.0,
        [24.0, 48.0],
        [10.0, 24.0, 87.0, 47.0],
    ),
    spr(
        "treeshroom",
        70.0,
        48.0,
        [40.0, 27.0],
        [6.0, 12.0, 58.0, 40.0],
    ),
    spr(
        "diggershroom",
        128.0,
        64.0,
        [30.0, 62.0],
        [2.0, 9.0, 125.0, 63.0],
    ),
    spr(
        "stinkyshroom",
        160.0,
        57.0,
        [24.0, 57.0],
        [10.0, 6.0, 150.0, 56.0],
    ),
    spr(
        "experienceshroom",
        96.0,
        64.0,
        [27.0, 64.0],
        [4.0, 8.0, 91.0, 63.0],
    ),
    spr(
        "consumptorshroom",
        96.0,
        48.0,
        [24.0, 43.0],
        [1.0, 3.0, 89.0, 43.0],
    ),
];

/// The metadata of a mushroom sprite sheet by name.
pub fn shroom_sprite(name: &str) -> Option<&'static ShroomSprite> {
    SPRITES.iter().find(|s| s.name == name)
}

fn sprite(name: &str) -> &'static ShroomSprite {
    shroom_sprite(name).unwrap_or_else(|| panic!("not a mushroom sprite: {name}"))
}

// port: mushroom_type_unlock_id
/// The upgrade that unlocks the type.
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

// port: mushroom_type_name_key
/// Locale key of the type's name.
pub fn mushroom_type_name_key(t: MushroomKind) -> String {
    format!("upg_{}", mushroom_type_unlock_id(t))
}

// port: mushroom_type_unlocked
pub fn mushroom_type_unlocked(st: &GameState, t: MushroomKind) -> bool {
    upg_is_owned(st, mushroom_type_unlock_id(t))
}

// port: mushroom_type_cost_base
/// Food price before the green perk; the experience mushroom gets dearer with each one built.
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

// port: mushroom_type_water_cost_base
/// Water price before the green perk; the digger's grows with its strand length upgrade.
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

// port: mushroom_type_cost
pub fn mushroom_type_cost(st: &GameState, t: MushroomKind) -> f64 {
    gml::round(mushroom_type_cost_base(st, t) * perk_cost_mult(st))
}

// port: mushroom_type_water_cost
pub fn mushroom_type_water_cost(st: &GameState, t: MushroomKind) -> f64 {
    gml::round(mushroom_type_water_cost_base(st, t) * perk_cost_mult(st))
}

// port: mushroom_type_affordable
/// Enough food and water (all sources) for the type.
pub fn mushroom_type_affordable(sim: &Sim, t: MushroomKind) -> bool {
    sim.state.state.food >= mushroom_type_cost(&sim.state, t)
        && sim.water_total() >= mushroom_type_water_cost(&sim.state, t)
}

/// Rust only (the owner's request, shown in the build bar): the colony's mushrooms of the type
/// on the map (not the ones stolen by an animal).
pub fn mushroom_type_count(state: &GameState, t: MushroomKind) -> usize {
    state
        .mushrooms
        .iter()
        .filter(|m| m.kind == t.code() && !m.stolen)
        .count()
}

// port: mushroom_type_sprite
/// The sprite sheet of the type (by name).
pub fn mushroom_type_sprite(t: MushroomKind) -> &'static str {
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

/// `mushroom_type_sprite(m.type)` for a saved type code (unknown codes: the spore sheet).
pub fn sprite_of_code(t: i64) -> &'static str {
    MushroomKind::from_code(t).map_or("sporeshroom", mushroom_type_sprite)
}

// port: shroom_sheet_cells
/// Variants side by side in a sheet (1 for any other sprite).
pub fn shroom_sheet_cells(spr: &str) -> f64 {
    match spr {
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

// port: mushroom_type_variants
pub fn mushroom_type_variants(t: MushroomKind) -> f64 {
    shroom_sheet_cells(mushroom_type_sprite(t))
}

// port: mushroom_cell_width
/// Width of one variant cell of a mushroom sheet.
pub fn mushroom_cell_width(spr: &str) -> f64 {
    sheet_cell_width(spr, sprite(spr).w)
}

/// `mushroom_cell_width` for any sprite of width `sprite_width` (`max(1, w div cells)`): the
/// app passes the asset width for sheets outside the mushroom table (upgrade-tree icons).
pub fn sheet_cell_width(spr: &str, sprite_width: f64) -> f64 {
    (sprite_width / shroom_sheet_cells(spr)).trunc().max(1.0)
}

// port: mushroom_variant
/// The mushroom's variant clamped to its sheet.
pub fn mushroom_variant(m: &MushroomSave) -> f64 {
    let n = MushroomKind::from_code(m.kind).map_or(2.0, mushroom_type_variants);
    gml::clamp(m.variant as f64, 0.0, n - 1.0)
}

// port: shroom_grow_scale
/// The grow-in scale (back-ease over `shroom_grow_ms` from `born`; 1 without `born`).
pub fn shroom_grow_scale(sim: &Sim, m: &MushroomSave) -> f64 {
    let Some(born) = m.born else {
        return 1.0;
    };
    let t = gml::clamp(
        (sim.now_ms() - born) / vis("shroom_grow_ms").max(1.0),
        0.0,
        1.0,
    );
    let c1 = 1.70158;
    let u = t - 1.0;
    let c3 = c1 + 1.0;
    (1.0 + c3 * u * u * u + c1 * u * u).max(0.0)
}

// port: mushroom_cell_left
/// Left edge of variant `v` in a sheet (two sheets have hand-tuned offsets).
pub fn mushroom_cell_left(spr: &str, v: f64) -> f64 {
    let mut l = v * mushroom_cell_width(spr);
    if spr == "stinkyshroom" && v == 2.0 {
        l += 7.0;
    }
    if spr == "experienceshroom" && v == 1.0 {
        l -= 4.0;
    }
    l
}

// port: mushroom_sprite_now
/// The sheet to draw now (a spent repeller shows its exploded sheet).
pub fn mushroom_sprite_now(m: &MushroomSave) -> &'static str {
    if m.kind == MushroomKind::Repeller.code() && !m.active {
        return "explosiveshroom_exploded";
    }
    sprite_of_code(m.kind)
}

// port: repeller_max_charges
pub fn repeller_max_charges(st: &GameState) -> f64 {
    bal(game_mode(st), "repeller_base_charges").max(1.0)
}

// port: repeller_regen_price
/// Food to recharge a spent repeller by hand.
pub fn repeller_regen_price(st: &GameState) -> f64 {
    let k = (1.0 - upg_bonus(st, "repeller_regen_cost")).max(0.2);
    gml::round(bal(game_mode(st), "repeller_regen_cost") * k).max(1.0)
}

// port: repeller_autoregen_frames
/// Frames until a spent repeller recharges itself; -1 without the autoregen upgrade.
pub fn repeller_autoregen_frames(sim: &Sim) -> f64 {
    let lvl = upg_level(&sim.state, "repeller_autoregen") as f64;
    if lvl <= 0.0 {
        return -1.0;
    }
    let mmin = sim.bal("repeller_autoregen_minutes_min");
    let cut = (lvl - 1.0) * sim.bal("repeller_autoregen_minutes_step");
    let mins = mmin.max(sim.bal("repeller_autoregen_minutes_first") - cut);
    mins * 60.0 * sim.fps * perk_time_mult(&sim.state)
}

// port: experience_period
/// Frames between two upgrade points of an experience mushroom.
pub fn experience_period(sim: &Sim) -> f64 {
    (sim.bal("experience_point_interval") * sim.fps * perk_time_mult(&sim.state)).max(60.0)
}

// port: experience_progress
/// Hover-bar progress of an experience mushroom (`None` when stolen).
pub fn experience_progress(sim: &Sim, i: usize) -> Option<f64> {
    let m = &sim.state.mushrooms[i];
    if m.stolen {
        return None;
    }
    Some(gml::clamp(1.0 - m.timer / experience_period(sim), 0.0, 1.0))
}

// port: tree_shroom_progress
/// Hover-bar progress of a tree mushroom toward its next drop (`None` when stolen).
pub fn tree_shroom_progress(sim: &Sim, i: usize) -> Option<f64> {
    let m = &sim.state.mushrooms[i];
    if m.stolen {
        return None;
    }
    let period = crate::life::tree_drop_period(sim).max(1.0);
    Some(gml::clamp(1.0 - m.timer / period, 0.0, 1.0))
}

// port: repeller_regen_progress
/// Hover-bar progress of a spent repeller's autoregen (`None` unless it is recharging).
pub fn repeller_regen_progress(sim: &Sim, i: usize) -> Option<f64> {
    let rf = repeller_autoregen_frames(sim);
    let m = &sim.state.mushrooms[i];
    if !m.active && rf >= 0.0 && !m.stolen && m.carried_by == -1 {
        let rt = m.regen_timer.unwrap_or(0.0);
        return Some(gml::clamp(rt / rf, 0.0, 1.0));
    }
    None
}
