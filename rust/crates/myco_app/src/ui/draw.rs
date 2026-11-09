//! The menus' draw calls. Menu code builds a list of GameMaker draw calls ([`DrawCmd`]:
//! `draw_rectangle`, `draw_text_transformed`, `draw_primitive`) in GUI coordinates (view-sized,
//! y down, ruling R17), which keeps it testable without Bevy; [`paint`] replays the list on the
//! shared GUI painter ([`super::gm_draw::Gm`], Task 19).
//!
//! Fonts as the original picks them: `ui_font()` is `Font1` (the GameMaker bitmap font) for
//! `Font1` languages, else the language TTF fitted to `Font1` (`locale_font_fit(file, Font1)`);
//! `ui_font_small()` is [`super::LocaleRes::font_small`].

use super::gm_draw::{Gm, UiFont};
use crate::assets::GameAssets;
use crate::render::blend::Vert;
use myco_core::locale::Locale;

/// `ui_font()` or `ui_font_small()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontSel {
    Main,
    Small,
}

/// One GameMaker draw call in GUI coordinates. Colours are GameMaker BGR integers.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCmd {
    /// `draw_rectangle(x1, y1, x2, y2, outline)`
    Rect {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        col: f64,
        alpha: f64,
        outline: bool,
    },
    /// `draw_text_transformed(x, y, s, sc, sc, 0)` with the given alignment (0 left/top,
    /// 1 centre/middle, 2 right/bottom)
    Text {
        x: f64,
        y: f64,
        s: String,
        sc: f64,
        col: f64,
        alpha: f64,
        font: FontSel,
        halign: u8,
        valign: u8,
    },
    /// a triangle list (`draw_primitive_begin(pr_trianglelist)`, or strips converted)
    Tris(Vec<Vert>),
}

/// String metrics of the two UI fonts (`string_width` / `string_height`).
pub trait TextMetrics {
    fn width(&self, font: FontSel, s: &str) -> f64;
    fn height(&self, font: FontSel, s: &str) -> f64;
}

// port: anon__1399__gml_Object_obj_hud_Create_0
/// `hud_fit(s, sc, maxw, maxh)` on the string's measured size (`string_width/height(s)` in the
/// current font): the scale `sc`, reduced so the text fits `maxw` (and `maxh` when positive).
/// The menus and the tree screen both measure and call this.
pub fn hud_fit_size(w: f64, h: f64, sc: f64, maxw: f64, maxh: f64) -> f64 {
    let mut k = sc;
    if w * k > maxw {
        k = maxw.max(0.05) / w.max(1.0);
    }
    if maxh > 0.0 && h * k > maxh {
        k = maxh.max(0.05) / h.max(1.0);
    }
    k
}

/// [`hud_fit_size`] with the menus' metrics.
pub fn hud_fit(m: &dyn TextMetrics, font: FontSel, s: &str, sc: f64, maxw: f64, maxh: f64) -> f64 {
    hud_fit_size(m.width(font, s), m.height(font, s), sc, maxw, maxh)
}

// port: anon__1832__gml_Object_obj_hud_Create_0
/// `hud_text_box(r, s, sc, pad)`: where and how big a label goes in a button rectangle; the text
/// is drawn left-aligned and vertically centred at the returned `(x, y)` with scale `k`. Takes the
/// label's measured size and the rectangle `(x1, y1, x2, y2)`.
pub fn hud_text_box_place(
    w: f64,
    h: f64,
    r: (f64, f64, f64, f64),
    sc: f64,
    pad: f64,
) -> (f64, f64, f64) {
    let (x1, y1, x2, y2) = r;
    let k = hud_fit_size(w, h, sc, x2 - x1 - pad * 2.0, y2 - y1 - 6.0);
    (x1 + pad, (y1 + y2) * 0.5, k)
}

/// Triangle strips as a triangle list of one colour.
pub fn strips_to_tris(strips: &[Vec<[f64; 2]>], col: f64, alpha: f64) -> Vec<Vert> {
    let mut out = Vec::new();
    for s in strips {
        for i in 2..s.len() {
            for p in [s[i - 2], s[i - 1], s[i]] {
                out.push(Vert {
                    x: p[0],
                    y: p[1],
                    col,
                    alpha,
                });
            }
        }
    }
    out
}

/// Fixed metrics (`w` per character, `h` per line) for tests.
pub struct FixedMetrics {
    pub w: f64,
    pub h: f64,
}

impl TextMetrics for FixedMetrics {
    fn width(&self, _: FontSel, s: &str) -> f64 {
        s.split('\n')
            .map(|l| l.chars().count() as f64 * self.w)
            .fold(0.0, f64::max)
    }
    fn height(&self, _: FontSel, s: &str) -> f64 {
        s.split('\n').count() as f64 * self.h
    }
}

/// `ui_font()` and `ui_font_small()` for the current language.
#[derive(bevy::prelude::Resource, Clone)]
pub struct MenuFonts {
    pub main: UiFont,
    pub small: UiFont,
}

impl MenuFonts {
    /// `ui_font()`: `Font1`, or the language TTF fitted like `locale_font_fit(file, Font1)`.
    pub fn new(assets: &GameAssets, locale: &Locale, small: UiFont) -> MenuFonts {
        // locale_font_fit(file, Font1) (Task 17's full port in `ui::fonts`)
        let main = super::fonts::UiFonts::load(assets, locale).font;
        MenuFonts { main, small }
    }

    pub fn get(&self, f: FontSel) -> &UiFont {
        match f {
            FontSel::Main => &self.main,
            FontSel::Small => &self.small,
        }
    }
}

/// [`TextMetrics`] of [`MenuFonts`] as the painter measures them.
pub struct GmMetrics<'a> {
    pub assets: &'a GameAssets,
    pub fonts: &'a MenuFonts,
}

impl GmMetrics<'_> {
    fn gm(&self, f: FontSel) -> Gm<'_> {
        let mut gm = Gm::new(self.assets);
        gm.font = self.fonts.get(f).clone();
        gm
    }
}

impl TextMetrics for GmMetrics<'_> {
    fn width(&self, font: FontSel, s: &str) -> f64 {
        self.gm(font).string_width(s)
    }
    fn height(&self, font: FontSel, s: &str) -> f64 {
        self.gm(font).string_height(s)
    }
}

/// Replay `cmds` on the painter.
pub fn paint(cmds: &[DrawCmd], gm: &mut Gm, fonts: &MenuFonts) {
    for c in cmds {
        match c {
            DrawCmd::Rect {
                x1,
                y1,
                x2,
                y2,
                col,
                alpha,
                outline,
            } => {
                gm.set_colour(*col);
                gm.set_alpha(*alpha);
                gm.rectangle(*x1, *y1, *x2, *y2, *outline);
            }
            DrawCmd::Text {
                x,
                y,
                s,
                sc,
                col,
                alpha,
                font,
                halign,
                valign,
            } => {
                gm.font = fonts.get(*font).clone();
                gm.set_colour(*col);
                gm.set_alpha(*alpha);
                gm.halign = *halign;
                gm.valign = *valign;
                gm.text_transformed(*x, *y, s, *sc, *sc);
            }
            DrawCmd::Tris(v) => {
                for t in v.as_chunks::<3>().0 {
                    gm.tri([
                        (t[0].x, t[0].y, t[0].col, t[0].alpha),
                        (t[1].x, t[1].y, t[1].col, t[1].alpha),
                        (t[2].x, t[2].y, t[2].col, t[2].alpha),
                    ]);
                }
            }
        }
    }
    gm.set_alpha(1.0);
    gm.halign = 0;
    gm.valign = 0;
}
