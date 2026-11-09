//! The black-metal logo generator screen (obj_hud Step_1 / Draw_75, `global.logo_open`): type a
//! name, GROW it, REROLL for another growth, EDIT / CLEAR, CLOSE. The tree screen (T19) opens it
//! through `TreeScreen::logo_requested`, which the menu step turns into
//! [`crate::logo::LogoState::logo_open`].

use super::draw::{hud_fit, strips_to_tris, DrawCmd, FontSel, TextMetrics};
use super::menu::*;
use crate::logo::LogoPhase;
use myco_core::locale::Locale;

// port: anon__17388__gml_Object_obj_hud_Create_0
/// `hud_logo_reroll_rect()`: GROW / REROLL, left of CLOSE.
pub fn hud_logo_reroll_rect(gw: f64) -> R {
    let c = hud_close_button_rect(gw);
    R {
        x1: c.x1 - 208.0,
        y1: c.y1,
        x2: c.x1 - 20.0,
        y2: c.y2,
    }
}

// port: anon__17592__gml_Object_obj_hud_Create_0
/// `hud_logo_clear_rect()`: CLEAR / EDIT, left of GROW.
pub fn hud_logo_clear_rect(gw: f64) -> R {
    let c = hud_logo_reroll_rect(gw);
    R {
        x1: c.x1 - 188.0,
        y1: c.y1,
        x2: c.x1 - 20.0,
        y2: c.y2,
    }
}

impl Menus {
    pub(super) fn logo_screen_step(&mut self, inp: &MenuInput) {
        let kb = &mut self.keyboard_string;
        self.logo.logo_step(kb, inp.enter);
        if inp.click && hud_close_button_rect(inp.gw).contains(inp.mx, inp.my) {
            self.logo.open = false;
            return;
        }
        let typing = self.logo.phase == LogoPhase::Type;
        if inp.click && hud_logo_reroll_rect(inp.gw).contains(inp.mx, inp.my) {
            if typing {
                if self.logo.logo_has_letters() {
                    self.logo.logo_grow_start();
                }
            } else {
                self.logo.logo_reroll();
            }
            return;
        }
        if inp.click && hud_logo_clear_rect(inp.gw).contains(inp.mx, inp.my) {
            if typing {
                self.keyboard_string.clear();
                self.logo.text.clear();
                return;
            }
            self.logo.logo_edit(&mut self.keyboard_string);
        }
    }

    pub(super) fn logo_screen_draw(
        &mut self,
        d: &mut Vec<DrawCmd>,
        m: &dyn TextMetrics,
        loc: &Locale,
        gw: f64,
        gh: f64,
        now_ms: f64,
    ) {
        let f = FontSel::Small;
        let text =
            |d: &mut Vec<DrawCmd>, x: f64, y: f64, s: &str, sc: f64, col: f64, h: u8, v: u8| {
                d.push(DrawCmd::Text {
                    x,
                    y,
                    s: s.into(),
                    sc,
                    col,
                    alpha: 1.0,
                    font: f,
                    halign: h,
                    valign: v,
                })
            };
        // hud_text_box(r, s, sc, pad): left-aligned, vertically centred, fitted into the box
        let text_box = |d: &mut Vec<DrawCmd>, r: R, s: &str, sc: f64, pad: f64| {
            let k = hud_fit(m, f, s, sc, r.x2 - r.x1 - pad * 2.0, r.y2 - r.y1 - 6.0);
            d.push(DrawCmd::Text {
                x: r.x1 + pad,
                y: r.cy(),
                s: s.into(),
                sc: k,
                col: COLOR_INK,
                alpha: 1.0,
                font: f,
                halign: 0,
                valign: 1,
            })
        };
        let outline = |d: &mut Vec<DrawCmd>, r: R, col: f64| {
            d.push(DrawCmd::Rect {
                x1: r.x1,
                y1: r.y1,
                x2: r.x2,
                y2: r.y2,
                col,
                alpha: 1.0,
                outline: true,
            })
        };
        d.push(DrawCmd::Rect {
            x1: 0.0,
            y1: 0.0,
            x2: gw,
            y2: gh,
            col: 0.0,
            alpha: 0.97,
            outline: false,
        });
        let typing = self.logo.phase == LogoPhase::Type;
        let title = loc.loc("logo_title");
        text(
            d,
            gw * 0.5,
            38.0,
            title,
            hud_fit(m, f, title, 1.5, gw - 80.0, 0.0),
            COLOR_DIM,
            1,
            0,
        );
        if typing {
            let caret = if ((now_ms / 500.0).trunc() as i64) % 2 == 0 {
                loc.loc("logo_caret")
            } else {
                " "
            };
            let (shown, col) = if self.logo.text.is_empty() {
                (loc.loc("logo_empty").to_string(), COLOR_DEEP)
            } else {
                (format!("{}{caret}", self.logo.text), COLOR_INK)
            };
            text(d, gw * 0.5, gh * 0.5 - 40.0, &shown, 4.0, col, 1, 0);
            let hint = loc.loc("logo_typing_hint");
            text(
                d,
                gw * 0.5,
                gh - 72.0,
                hint,
                hud_fit(m, f, hint, 1.3, gw - 80.0, 0.0),
                COLOR_DEEP,
                1,
                0,
            );
        } else {
            let strips = self
                .logo
                .logo_draw(gw * 0.5, gh * 0.5 - 10.0, gw - 200.0, gh - 400.0);
            d.push(DrawCmd::Tris(strips_to_tris(&strips, COLOR_INK, 1.0)));
            let t = self.logo.text.clone();
            text(d, gw * 0.5, gh - 112.0, &t, 1.5, COLOR_DEEP, 1, 0);
            let hint = loc.loc("logo_grown_hint");
            text(
                d,
                gw * 0.5,
                gh - 72.0,
                hint,
                hud_fit(m, f, hint, 1.3, gw - 80.0, 0.0),
                COLOR_DEEP,
                1,
                0,
            );
        }
        let cl = hud_logo_clear_rect(gw);
        outline(d, cl, COLOR_DIM);
        text_box(
            d,
            cl,
            loc.loc(if typing {
                "button_clear"
            } else {
                "button_edit"
            }),
            1.4,
            20.0,
        );
        let rr = hud_logo_reroll_rect(gw);
        let rcol = if typing && self.logo.logo_has_letters() {
            COLOR_GOOD
        } else {
            COLOR_DIM
        };
        outline(d, rr, rcol);
        text_box(
            d,
            rr,
            loc.loc(if typing {
                "button_grow"
            } else {
                "button_reroll"
            }),
            1.4,
            20.0,
        );
        let cr = hud_close_button_rect(gw);
        outline(d, cr, COLOR_DIM);
        text_box(d, cr, loc.loc("button_close"), 1.4, 22.0);
    }
}
