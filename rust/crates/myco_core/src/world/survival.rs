//! Survival layout (scr_world): outside the full zone around the start, only a hashed share of
//! the food survives and lakes hold a share of their water.

use super::WorldRt;
use crate::balance::{vis, wld};
use crate::gml;
use crate::state::{GameMode, WorldChunkSave};

impl WorldRt {
    // port: world_survival_full_at
    /// Inside the zone around the start that keeps all its resources in survival.
    pub fn survival_full_at(&self, x: f64, y: f64) -> bool {
        let half = vis("screen_width_px") * wld("survival_full_zone_x_screens");
        if (x - self.params.start_x).abs() > half {
            return false;
        }
        let depth = self.params.surface_y
            + vis("screen_height_px") * wld("survival_full_zone_depth_screens");
        y <= depth
    }

    // port: world_survival_thin
    /// Thin a chunk once for survival (`sv` marks what was already decided): drop foods by
    /// hash, cut generated lakes' water (start lakes, seed >= 700, keep theirs).
    pub fn survival_thin(&self, chunk: &mut WorldChunkSave) {
        if self.mode != GameMode::Survival {
            return;
        }
        let share = gml::clamp(wld("survival_resource_share"), 0.0, 1.0);
        let foods = std::mem::take(&mut chunk.foods);
        for mut f in foods {
            let mut skip = false;
            if !f.sv.unwrap_or(false) {
                f.sv = Some(true);
                if !f.maze.unwrap_or(false)
                    && !self.survival_full_at(f.x, f.y)
                    && self.hash01(f.x.floor(), f.y.floor(), 611.0) >= share
                {
                    skip = true;
                }
            }
            if !skip {
                chunk.foods.push(f);
            }
        }
        for l in &mut chunk.lakes {
            if !l.sv.unwrap_or(false) {
                l.sv = Some(true);
                if (l.seed as f64) < 700.0
                    && !self.survival_full_at(l.body_x + l.body_w * 0.5, l.body_y + l.body_h * 0.5)
                {
                    l.full = gml::round(l.full * share).max(1.0);
                    l.amount = l.amount.min(l.full);
                }
            }
        }
    }
}
