//! Length and soil helpers of the mycelium: cm conversion and text, the length counter, the
//! camera box growing with the colony, soil penetration.

use crate::balance::vis;
use crate::gml;
use crate::progress::upgrades::upg_bonus;
use crate::sim::Sim;

// port: pixels_to_cm
pub fn pixels_to_cm(sim: &Sim, px: f64) -> f64 {
    px / sim.bal("pixels_per_cm").max(0.01)
}

// port: length_text
/// `length_text(cm)`: "<cm> cm" or, with imperial units, "<in> in". `loc` translates the unit
/// keys (`hud_cm`, `hud_in`); the app passes its locale.
pub fn length_text(units_imperial: bool, cm: f64, loc: &dyn Fn(&str) -> String) -> String {
    if units_imperial {
        return format!("{} {}", gml_string(gml::round(cm / 2.54)), loc("hud_in"));
    }
    format!("{} {}", gml_string(gml::round(cm)), loc("hud_cm"))
}

/// `string(x)` of a whole number (GML prints integers without decimals).
pub(crate) fn gml_string(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        // as the LÖVE port's gm.string: two decimals
        format!("{x:.2}")
    }
}

// port: mycelium_cm
/// The mycelium length in cm (the length counter, tasks, goals, the save meta).
pub fn mycelium_cm(sim: &Sim) -> f64 {
    pixels_to_cm(sim, sim.state.state.length_px)
}

// port: mycelium_add_length
pub fn mycelium_add_length(sim: &mut Sim, px: f64) {
    sim.state.state.length_px += px.max(0.0);
}

// port: reach_expand
/// Grow the camera box (`cam_box_*`) to keep (x, y) reachable, with a margin, inside the world.
pub fn reach_expand(sim: &mut Sim, x: f64, y: f64) {
    let m = vis("camera_reach_margin_px");
    let x1 = x - m;
    let x2 = x + m;
    let y1 = (y - m).max(0.0);
    let y2 = sim.world.params.bottom_y.min(y + m);
    let b = &mut sim.state.state.cam_box;
    b[0] = b[0].min(x1);
    b[2] = b[2].max(x2);
    b[1] = b[1].min(y1);
    b[3] = b[3].max(y2);
}

// port: myc_soil_penetration
/// Growth speed bonus through clay and deep soil (`world_travel_cost`'s bonus).
pub fn soil_penetration(sim: &Sim) -> f64 {
    sim.bal("base_hardsoil_penetration") + upg_bonus(&sim.state, "soil_penetration")
}
