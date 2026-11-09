//! Slot selection for a new game or a load (`hud_slot_*`). Slots come from `save::list_slots`:
//! an empty slot shows "Empty"; a slot whose save or meta cannot be read shows a "damaged" mark,
//! cannot be loaded (hover and clicks are ignored in load mode) and can be overwritten by a new
//! game. A load that still fails marks the slot damaged and keeps the screen open.

use super::draw::{hud_fit, DrawCmd, FontSel, TextMetrics};
use super::menu::*;
use myco_core::balance::wld;
use myco_core::save::{self, SlotStatus};
use myco_core::sim::Sim;

/// Number of save slots.
pub const SLOTS: usize = 3;

// port: anon__13992__gml_Object_obj_hud_Create_0
/// `hud_slot_button_rect(i)`: three 760×76 slots and "Back" (i = 3) under the surface line.
pub fn hud_slot_button_rect(gw: f64, cam_y: f64, i: usize) -> R {
    let top = wld("surface_height_px") - cam_y + 60.0;
    let (w, h) = (760.0, 76.0);
    let x = (gw - w) * 0.5;
    let y = top + i as f64 * (h + 12.0);
    R {
        x1: x,
        y1: y,
        x2: x + w,
        y2: y + h,
    }
}

impl Menus {
    // port: anon__13634__gml_Object_obj_hud_Create_0
    /// `hud_slot_open(mode)`: read the three slots.
    pub fn hud_slot_open(&mut self, mode: SlotMode) {
        self.slot_mode = mode;
        self.slot_naming = -1;
        self.slot_metas = match self.save_dir() {
            Some(dir) => save::list_slots(dir, SLOTS),
            None => vec![SlotStatus::Empty; SLOTS],
        };
    }

    // port: anon__13863__gml_Object_obj_hud_Create_0
    pub fn hud_slot_close(&mut self) {
        self.slot_mode = SlotMode::None;
        self.slot_naming = -1;
    }

    fn slot_status(&self, i: usize) -> &SlotStatus {
        self.slot_metas.get(i).unwrap_or(&SlotStatus::Empty)
    }

    /// A slot that can be loaded (has a readable meta and save).
    pub fn slot_loadable(&self, i: usize) -> bool {
        matches!(self.slot_status(i), SlotStatus::Ok(_))
    }

    // port: anon__14519__gml_Object_obj_hud_Create_0
    /// `hud_slot_label(i)`: "Slot N  -  name  -  length", "Empty", or the damaged mark.
    pub fn hud_slot_label(&self, i: usize, env: &MenuEnv) -> String {
        let loc = env.locale;
        let head = format!("{} {}  -  ", loc.loc("slot_label"), i + 1);
        match self.slot_status(i) {
            SlotStatus::Empty => head + loc.loc("slot_empty"),
            SlotStatus::Corrupt(_) => head + slot_corrupt_text(loc.language().code()),
            SlotStatus::Ok(m) => format!(
                "{head}{}  -  {}",
                m.name,
                length_text(loc, env.flags.units_imperial, m.length_cm)
            ),
        }
    }

    // port: anon__14951__gml_Object_obj_hud_Create_0
    /// `hud_slot_mode_label(i)`: the save's mode under its label.
    pub fn hud_slot_mode_label(&self, i: usize, env: &MenuEnv) -> String {
        match self.slot_status(i) {
            SlotStatus::Ok(m) => env.locale.loc(&format!("mode_{}", mode_str(m.mode))).to_string(),
            _ => String::new(),
        }
    }

    pub(super) fn slot_step(&mut self, sim: &mut Sim, inp: &MenuInput, env: &MenuEnv) {
        self.slot_hover = -1;
        if self.slot_naming >= 0 {
            self.name_step(sim, inp, env);
            return;
        }
        let cam_y = sim.camera.view_y;
        let Some(i) =
            (0..=SLOTS).find(|&i| hud_slot_button_rect(inp.gw, cam_y, i).contains(inp.mx, inp.my))
        else {
            return;
        };
        if i < SLOTS && self.slot_mode == SlotMode::Load && !self.slot_loadable(i) {
            return;
        }
        self.slot_hover = i as i32;
        if !inp.click {
            return;
        }
        if i == SLOTS {
            self.hud_slot_close();
        } else if self.slot_mode == SlotMode::New {
            self.slot_naming = i as i32;
            self.keyboard_string.clear();
        } else if let Some(dir) = self.save_dir.clone() {
            // load_game(i): continue the save, or keep the screen open if it fails
            match Sim::load_game(&dir, i) {
                Ok(loaded) => {
                    *sim = loaded;
                    self.hud_slot_close();
                }
                Err(e) => self.slot_metas[i] = SlotStatus::Corrupt(format!("{e:?}")),
            }
        }
    }

    pub(super) fn slot_draw(
        &self,
        d: &mut Vec<DrawCmd>,
        m: &dyn TextMetrics,
        env: &MenuEnv,
        gw: f64,
        cam_y: f64,
        now_ms: f64,
    ) {
        let loc = env.locale;
        let ta = self.title_alpha;
        let r0 = hud_slot_button_rect(gw, cam_y, 0);
        let scx = r0.cx();
        let mut title = loc
            .loc(if self.slot_mode == SlotMode::New {
                "slot_title_new"
            } else {
                "slot_title_load"
            })
            .to_string();
        if self.slot_mode == SlotMode::New {
            title += &format!(
                "  -  {}",
                loc.loc(&format!("mode_{}", mode_str(self.new_mode))).to_uppercase()
            );
        }
        let mut lines: Vec<(String, f64, f64)> = Vec::new();
        if self.slot_naming >= 0 {
            title = loc.loc("slot_name_prompt").to_string();
            lines = self.name_lines(env, now_ms);
        }
        let tsc = hud_fit(m, FontSel::Main, &title, 0.8, gw - 80.0, 0.0);
        d.push(DrawCmd::Text {
            x: scx + 3.0,
            y: r0.y1 - 40.0 + 3.0,
            s: title.clone(),
            sc: tsc,
            col: 0.0,
            alpha: ta * 0.6,
            font: FontSel::Main,
            halign: 1,
            valign: 1,
        });
        d.push(DrawCmd::Text {
            x: scx,
            y: r0.y1 - 40.0,
            s: title,
            sc: tsc,
            col: COLOR_GOLD,
            alpha: ta,
            font: FontSel::Main,
            halign: 1,
            valign: 1,
        });
        if self.slot_naming >= 0 {
            for (i, (text, sc0, col)) in lines.iter().enumerate() {
                let r = hud_slot_button_rect(gw, cam_y, i);
                let sc = hud_fit(m, FontSel::Main, text, *sc0, r.x2 - r.x1 - 24.0, 0.0);
                shadow_text(d, scx, r.cy(), text, sc, *col, ta, FontSel::Main, 1, 1);
            }
            return;
        }
        for i in 0..=SLOTS {
            let r = hud_slot_button_rect(gw, cam_y, i);
            let label = if i == SLOTS {
                loc.loc("slot_back").to_string()
            } else {
                self.hud_slot_label(i, env)
            };
            let unusable = i < SLOTS && !self.slot_loadable(i);
            let mut col = if self.slot_hover == i as i32 {
                COLOR_INK
            } else {
                COLOR_DIM
            };
            if unusable && self.slot_mode == SlotMode::Load {
                col = COLOR_DEEP;
            }
            if i < SLOTS && matches!(self.slot_status(i), SlotStatus::Corrupt(_)) {
                // the damaged mark reads as a warning, not as a choice
                col = if self.slot_mode == SlotMode::Load {
                    COLOR_BAD
                } else {
                    col
                };
            }
            let sc = hud_fit(m, FontSel::Main, &label, 0.7, r.x2 - r.x1 - 24.0, r.y2 - r.y1);
            let sub = if i < SLOTS {
                self.hud_slot_mode_label(i, env)
            } else {
                String::new()
            };
            let ly = if sub.is_empty() { r.cy() } else { r.cy() - 9.0 };
            shadow_text(d, r.cx(), ly, &label, sc, col, ta, FontSel::Main, 1, 1);
            if !sub.is_empty() {
                let ssc = hud_fit(m, FontSel::Main, &sub, 0.36, r.x2 - r.x1 - 24.0, 0.0);
                d.push(DrawCmd::Text {
                    x: r.cx(),
                    y: r.y2 - 13.0,
                    s: sub,
                    sc: ssc,
                    col: COLOR_DEEP,
                    alpha: ta,
                    font: FontSel::Main,
                    halign: 1,
                    valign: 1,
                });
            }
        }
    }
}
