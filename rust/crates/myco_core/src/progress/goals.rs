//! Goals after the tasks (scr_game): migration (spore mushrooms, food, water) and magic (all
//! cave perks); number formatting and text fitting for the goal panels and popups.

use crate::balance::gml_truthy;
use crate::gml;
use crate::sim::Sim;

/// A goal of the second stage (`goal_met(id)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Goal {
    Migration,
    Magic,
}

impl Goal {
    pub const ALL: [Goal; 2] = [Goal::Migration, Goal::Magic];

    pub fn as_str(self) -> &'static str {
        match self {
            Goal::Migration => "migration",
            Goal::Magic => "magic",
        }
    }

    pub fn from_key(s: &str) -> Option<Goal> {
        Goal::ALL.into_iter().find(|g| g.as_str() == s)
    }
}

/// Font metrics for the text fitting below (`string_width` & co. of the current font). Logic
/// never measures text (ruling R9): the app passes its measurer in.
pub trait TextMeasure {
    fn string_width(&self, s: &str) -> f64;
    fn string_height(&self, s: &str) -> f64;
    /// `string_width_ext(s, sep, w)`: wrapped at `w`, line separation `sep` (-1: font height)
    fn string_width_ext(&self, s: &str, sep: f64, w: f64) -> f64;
    fn string_height_ext(&self, s: &str, sep: f64, w: f64) -> f64;
}

/// Cave perk ids (`cave_defs()[i].id`, scr_world; Task 6 ports the cave definitions).
const CAVE_IDS: [&str; 5] = ["blue", "purple", "red", "green", "yellow"];

/// `perk_has(id)` (Task 6 ports it; it reads `global.cave_perks` the same way).
fn perk_has(sim: &Sim, id: &str) -> bool {
    gml_truthy(sim.state.state.cave_perks.get(id))
}

// port: goal_spore_count
/// Spore mushrooms in the colony (not stolen, not carried); the debug flag
/// `tasks.debug_spores` counts the goal as reached.
pub fn goal_spore_count(sim: &Sim) -> f64 {
    let mut n = sim
        .state
        .mushrooms
        .iter()
        .filter(|m| m.kind == 0 && !m.stolen && m.carried_by == -1)
        .count() as f64;
    if gml_truthy(sim.state.state.tasks.extra.get("debug_spores")) {
        n = n.max(sim.bal("migration_spore_mushrooms"));
    }
    n
}

// port: goal_met
/// Migration: enough spore mushrooms, food and water. Magic: every cave perk.
pub fn goal_met(sim: &Sim, id: Goal) -> bool {
    match id {
        Goal::Migration => {
            let need = sim.bal("migration_spore_mushrooms");
            if goal_spore_count(sim) < need {
                return false;
            }
            if sim.state.state.food < sim.bal("migration_food") {
                return false;
            }
            sim.water_total() >= sim.bal("migration_water")
        }
        Goal::Magic => CAVE_IDS.iter().all(|id| perk_has(sim, id)),
    }
}

/// `string(x)` for numbers (`gm.string`): integers without decimals, others with two.
fn gm_string(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v:.2}")
    }
}

// port: goal_number_text
/// `round(v)` with dots between thousands: 1234567 → "1.234.567".
pub fn goal_number_text(v: f64) -> String {
    let mut s: Vec<char> = gm_string(gml::round(v)).chars().collect();
    let mut out = String::new();
    while s.len() > 3 {
        let tail: String = s[s.len() - 3..].iter().collect();
        out = format!(".{tail}{out}");
        s.truncate(s.len() - 3);
    }
    s.into_iter().collect::<String>() + &out
}

// port: goal_text_scale
/// Scale that makes the font's line `h` px tall.
pub fn goal_text_scale(m: &dyn TextMeasure, h: f64) -> f64 {
    h / m.string_height("A").max(1.0)
}

// port: goal_fit
/// Shrink `scale` by 8% steps (at most 16, not below 0.05) until `text` fits `maxw` (and
/// `maxh` when positive); `sep >= 0` measures it wrapped at `maxw`.
pub fn goal_fit(
    m: &dyn TextMeasure,
    text: &str,
    mut scale: f64,
    maxw: f64,
    maxh: f64,
    sep: f64,
) -> f64 {
    let mut i = 0;
    while i < 16 && scale > 0.05 {
        let (w, h) = if sep >= 0.0 {
            (
                m.string_width_ext(text, sep, maxw / scale) * scale,
                m.string_height_ext(text, sep, maxw / scale) * scale,
            )
        } else {
            (m.string_width(text) * scale, m.string_height(text) * scale)
        };
        if w <= maxw + 1.0 {
            if maxh <= 0.0 {
                break;
            }
            if h <= maxh + 1.0 {
                break;
            }
        }
        scale *= 0.92;
        i += 1;
    }
    scale
}
