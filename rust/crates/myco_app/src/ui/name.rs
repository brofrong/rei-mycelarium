//! Naming the mycelium of a new game (obj_hud Step_1 / Draw_75, `hud_slot_naming >= 0`): up to
//! 16 typed characters; Enter starts the game in the chosen slot (an empty name becomes
//! "Mycelium N"), Escape goes back to the slots. Starting hands over to the game: the title
//! closes and the intro (Task 21) or the game takes over.

use super::menu::*;
use myco_core::save::SlotStatus;
use myco_core::sim::Sim;

/// Longest mycelium name.
pub const NAME_MAX: usize = 16;

impl Menus {
    pub(super) fn name_step(&mut self, sim: &mut Sim, inp: &MenuInput, env: &MenuEnv) {
        if self.keyboard_string.chars().count() > NAME_MAX {
            self.keyboard_string = self.keyboard_string.chars().take(NAME_MAX).collect();
        }
        if !inp.enter {
            return;
        }
        let slot = self.slot_naming;
        let mut name = self.keyboard_string.trim().to_string();
        if name.is_empty() {
            name = format!("{} {}", env.locale.loc("slot_default_name"), slot + 1);
        }
        // save_start_new(slot, name); game_mode_start(hud_new_mode); title_open = false
        sim.start_from_title(slot as i64, &name, self.new_mode);
        self.hud_slot_close();
    }

    /// The naming screen's lines: typed name with a blinking cursor, what it overwrites, the hint.
    pub(super) fn name_lines(&self, env: &MenuEnv, now_ms: f64) -> Vec<(String, f64, f64)> {
        let loc = env.locale;
        let cursor = if ((now_ms / 500.0).trunc() as i64) % 2 == 0 {
            "_"
        } else {
            " "
        };
        let mut lines = vec![(
            format!("{}{cursor}", self.keyboard_string),
            1.0,
            COLOR_INK,
        )];
        if let Some(SlotStatus::Ok(m)) = self.slot_metas.get(self.slot_naming as usize) {
            lines.push((
                format!("{} {}", loc.loc("slot_overwrites"), m.name),
                0.6,
                COLOR_BAD,
            ));
        }
        lines.push((loc.loc("slot_name_hint").to_string(), 0.6, COLOR_DIM));
        lines
    }
}
