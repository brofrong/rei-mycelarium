//! The pause menu (Escape in game, or focus loss) and the title's Settings screen: a dark veil,
//! the volume sliders, the settings buttons and, in game, "Main menu" (saves, then restarts to
//! the title) and "Quit" (obj_game Other_3 saves on the way out).

use super::draw::{DrawCmd, FontSel, TextMetrics};
use super::menu::*;
use super::settings::*;
use myco_core::sim::Sim;

impl Menus {
    pub(super) fn pause_step(
        &mut self,
        inp: &MenuInput,
        env: &mut MenuEnv,
        sim: &mut Sim,
        fx: &mut Vec<MenuEffect>,
    ) {
        let (gw, gh) = (inp.gw, inp.gh);
        if inp.lmb_down {
            for i in 0..VOLUME_LABELS.len() {
                let r = hud_volume_slider_rect(gw, gh, i);
                if inp.click && r.grow(12.0).contains(inp.mx, inp.my) {
                    self.volume_drag = i as i32;
                }
                if self.volume_drag == i as i32 {
                    let v = (inp.mx - r.x1) / (r.x2 - r.x1).max(1.0);
                    fx.push(MenuEffect::SetVolume(i, v));
                    env.volumes[i] = v.clamp(0.0, 1.0);
                }
            }
        } else {
            if self.volume_drag != -1 {
                fx.push(MenuEffect::AudioSave);
            }
            self.volume_drag = -1;
        }
        if self.volume_drag != -1 {
            return;
        }
        let items = Menus::pause_items(sim.title_open);
        let Some(i) =
            (0..items.len()).find(|&i| hud_pause_button_rect(gw, gh, i).contains(inp.mx, inp.my))
        else {
            return;
        };
        let item = items[i];
        if hud_pause_item_disabled(item, env.fullscreen) {
            return;
        }
        self.pause_hover = i as i32;
        if inp.rclick && item == "window_size" {
            fx.push(MenuEffect::CycleWindowSize(-1));
        }
        if inp.rclick && item == "language" {
            fx.push(MenuEffect::CycleLanguage(-1));
        }
        if !inp.click {
            return;
        }
        match item {
            "resume" => sim.pause_open = false,
            "display_mode" => fx.push(MenuEffect::SetFullscreen(!env.fullscreen)),
            "window_size" => fx.push(MenuEffect::CycleWindowSize(1)),
            "units" => env.flags.units_imperial = !env.flags.units_imperial,
            "language" => fx.push(MenuEffect::CycleLanguage(1)),
            "main_menu" => {
                fx.push(MenuEffect::AudioSave);
                sim.save_game();
                fx.push(MenuEffect::Restart);
            }
            "quit" => fx.push(MenuEffect::Quit),
            _ => {}
        }
    }

    pub(super) fn pause_draw(
        &self,
        d: &mut Vec<DrawCmd>,
        m: &dyn TextMetrics,
        env: &MenuEnv,
        title_open: bool,
        gw: f64,
        gh: f64,
    ) {
        let loc = env.locale;
        let text = |d: &mut Vec<DrawCmd>, x: f64, y: f64, s: String, sc: f64, col: f64, alpha: f64| {
            d.push(DrawCmd::Text {
                x,
                y,
                s,
                sc,
                col,
                alpha,
                font: FontSel::Small,
                halign: 0,
                valign: 0,
            })
        };
        let rect = |d: &mut Vec<DrawCmd>, r: R, col: f64, alpha: f64, outline: bool| {
            d.push(DrawCmd::Rect {
                x1: r.x1,
                y1: r.y1,
                x2: r.x2,
                y2: r.y2,
                col,
                alpha,
                outline,
            })
        };
        let full = R {
            x1: 0.0,
            y1: 0.0,
            x2: gw,
            y2: gh,
        };
        rect(d, full, 0.0, 0.68, false);
        for (i, label) in VOLUME_LABELS.iter().enumerate() {
            let r = hud_volume_slider_rect(gw, gh, i);
            let v = env.volumes[i];
            let kx = r.x1 + (r.x2 - r.x1) * v;
            text(d, r.x1, r.y1 - 30.0, loc.loc(label).to_string(), 1.2, COLOR_DIM, 1.0);
            let pct = format!("{}%", myco_core::gml::round(v * 100.0) as i64);
            text(d, r.x2 - 60.0, r.y1 - 30.0, pct, 1.2, COLOR_DIM, 1.0);
            rect(d, r, COLOR_PANEL, 1.0, false);
            rect(d, R { x2: kx, ..r }, COLOR_GOOD, 1.0, false);
            let edge = if self.volume_drag == i as i32 {
                COLOR_INK
            } else {
                COLOR_DIM
            };
            rect(d, r, edge, 1.0, true);
            let knob = R {
                x1: kx - 4.0,
                y1: r.y1 - 6.0,
                x2: kx + 4.0,
                y2: r.y2 + 6.0,
            };
            rect(d, knob, edge, 1.0, false);
        }
        let items = Menus::pause_items(title_open);
        for (i, item) in items.iter().enumerate() {
            let r = hud_pause_button_rect(gw, gh, i);
            let label = pause_item_label(item, loc, env);
            let hover = self.pause_hover == i as i32;
            let disabled = hud_pause_item_disabled(item, env.fullscreen);
            let col = if hover { COLOR_INK } else { COLOR_DIM };
            rect(d, r, COLOR_PANEL, 0.95, false);
            rect(d, r, col, 1.0, true);
            let w = m.width(FontSel::Small, &label);
            let sc = 1.8f64.min((r.x2 - r.x1 - 32.0) / w.max(1.0));
            let ty = r.cy() - m.height(FontSel::Small, &label) * sc * 0.5;
            let alpha = if disabled { 0.35 } else { 1.0 };
            text(d, r.cx() - w * sc * 0.5, ty, label, sc, col, alpha);
        }
        let lr = hud_pause_button_rect(gw, gh, items.len() - 1);
        let vw = m.width(FontSel::Small, VERSION);
        text(d, lr.cx() - vw * 0.5, lr.y2 + 12.0, VERSION.into(), 1.0, COLOR_DIM, 0.7);
    }
}
