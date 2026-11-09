//! The credits screen (`hud_credits_*`): two columns of groups under the surface line, a Back
//! button; any click goes back.

use super::draw::{hud_fit, DrawCmd, FontSel, TextMetrics};
use super::menu::*;
use myco_core::balance::wld;
use myco_core::locale::Locale;

/// `hud_credits_columns`: per column, (heading key, names).
pub const CREDITS: [&[(&str, &[&str])]; 2] = [
    &[
        (
            "credits_plan_kapkan",
            &["Andrey Tyulenev", "Philipp Zybkovets"],
        ),
        ("credits_smm_qa", &["Daria Konakhova"]),
    ],
    &[
        (
            "credits_art_ui",
            &[
                "LaNa_FeeD",
                "Wothy (@wothella)",
                "Olga Zaporozhskaia",
                "Oniks",
            ],
        ),
        ("credits_forest_assets", &["Eder Muniz"]),
    ],
];

// port: anon__12865__gml_Object_obj_hud_Create_0
/// `hud_credits_top()`: 40 px below the surface line on screen.
pub fn hud_credits_top(cam_y: f64) -> f64 {
    wld("surface_height_px") - cam_y + 40.0
}

// port: anon__13059__gml_Object_obj_hud_Create_0
/// `hud_credits_back_rect()`
pub fn hud_credits_back_rect(gw: f64, cam_y: f64) -> R {
    let (w, h) = (420.0, 64.0);
    let y = hud_credits_top(cam_y) + 380.0;
    R {
        x1: (gw - w) * 0.5,
        y1: y,
        x2: (gw + w) * 0.5,
        y2: y + h,
    }
}

impl Menus {
    pub(super) fn credits_step(&mut self, inp: &MenuInput, cam_y: f64) {
        self.credits_back_hover = hud_credits_back_rect(inp.gw, cam_y).contains(inp.mx, inp.my);
        // any click closes the credits
        if inp.click {
            self.credits_open = false;
        }
    }

    pub(super) fn credits_draw(
        &self,
        d: &mut Vec<DrawCmd>,
        m: &dyn TextMetrics,
        loc: &Locale,
        gw: f64,
        cam_y: f64,
    ) {
        let ta = self.title_alpha;
        let top = hud_credits_top(cam_y);
        let ccx = [gw * 0.5 - 300.0, gw * 0.5 + 300.0];
        for (c, col) in CREDITS.iter().enumerate() {
            let mut y = top;
            for (head, names) in col.iter() {
                let mut lines = vec![(loc.loc(head).to_string(), 0.8, COLOR_GOLD)];
                lines.extend(names.iter().map(|n| (n.to_string(), 0.6, COLOR_INK)));
                for (k, (text, sc0, colour)) in lines.iter().enumerate() {
                    let sc = hud_fit(m, FontSel::Main, text, *sc0, 560.0, 0.0);
                    shadow_text(d, ccx[c], y, text, sc, *colour, ta, FontSel::Main, 1, 1);
                    y += if k == 0 { 52.0 } else { 40.0 };
                }
                y += 28.0;
            }
        }
        let br = hud_credits_back_rect(gw, cam_y);
        let col = if self.credits_back_hover {
            COLOR_INK
        } else {
            COLOR_DIM
        };
        shadow_text(
            d,
            br.cx(),
            br.cy(),
            loc.loc("slot_back"),
            0.7,
            col,
            ta,
            FontSel::Main,
            1,
            1,
        );
    }
}
