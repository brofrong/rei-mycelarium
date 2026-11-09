//! The main menu and the menu half of obj_hud: the state obj_hud keeps for the title, slot,
//! naming, credits, settings/pause and logo screens, its Begin Step (`gml_Object_obj_hud_Step_1`,
//! the menu branches) and its Draw GUI (`gml_Object_obj_hud_Draw_75`, the menu branches).
//!
//! The rest of obj_hud (HUD bar, build menu, tasks, tooltips) is Task 17's; the upgrade tree is
//! Task 19's. They share [`UiFlags`] with this module.
//!
//! The menus run in the fixed step before the simulation step (obj_hud's Begin Step runs before
//! obj_game's Step) and change the [`Sim`] directly as the original changes its globals:
//! `title_open`, `pause_open`, a new game bound to a slot, a loaded game. Effects outside the
//! simulation (window, language, audio, restart, quit) come back as [`MenuEffect`]s.

use super::draw::{DrawCmd, FontSel, TextMetrics};
use crate::logo::LogoState;
use bevy::prelude::Resource;
use myco_core::balance::wld;
use myco_core::locale::Locale;
use myco_core::save::{self, SlotStatus};
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use std::path::{Path, PathBuf};

/// The version shown under the title menu and the settings.
pub const VERSION: &str = "v1.02";

// obj_hud Create_0 colours (GameMaker BGR)
pub const COLOR_INK: f64 = 14873582.0;
pub const COLOR_DIM: f64 = 8689804.0;
pub const COLOR_DEEP: f64 = 4609098.0;
pub const COLOR_GOOD: f64 = 9098922.0;
pub const COLOR_BAD: f64 = 6846166.0;
pub const COLOR_GOLD: f64 = 7914718.0;
pub const COLOR_PANEL: f64 = 1185298.0;

/// `hud_title_items`
pub const TITLE_ITEMS: [&str; 6] = [
    "exploration",
    "survival",
    "load_game",
    "settings",
    "credits",
    "exit",
];
/// `hud_pause_items_game`
pub const PAUSE_ITEMS_GAME: [&str; 7] = [
    "resume",
    "display_mode",
    "window_size",
    "units",
    "language",
    "main_menu",
    "quit",
];
/// `hud_pause_items_title`
pub const PAUSE_ITEMS_TITLE: [&str; 6] = [
    "resume",
    "display_mode",
    "window_size",
    "units",
    "language",
    "quit",
];

/// A GUI rectangle (`x1 <= x2`, `y1 <= y2`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct R {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl R {
    /// `point_in_rectangle` (edges inclusive).
    pub fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x1 && x <= self.x2 && y >= self.y1 && y <= self.y2
    }
    pub fn grow(&self, d: f64) -> R {
        R {
            x1: self.x1 - d,
            y1: self.y1 - d,
            x2: self.x2 + d,
            y2: self.y2 + d,
        }
    }
    pub fn cx(&self) -> f64 {
        (self.x1 + self.x2) * 0.5
    }
    pub fn cy(&self) -> f64 {
        (self.y1 + self.y2) * 0.5
    }
}

/// HUD/menu flags shared with the HUD (T17) and the tree (T19). `menu_open`, `debug_open` and
/// `hud_hidden` are `global.*` flags the simulation reads through `StepInput` (ruling R15).
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct UiFlags {
    /// `global.menu_open`: the upgrade tree is open (T19 opens it from the HUD button)
    pub menu_open: bool,
    /// `global.debug_open` (its F1 toggle is compiled out in the original)
    pub debug_open: bool,
    /// `global.hud_hidden`
    pub hud_hidden: bool,
    /// `global.units_imperial` (survives a restart to the main menu, not saved)
    pub units_imperial: bool,
    /// the HUD's build menu or placement mode is open (T17 sets it each frame)
    pub build_open: bool,
    /// Escape asked the HUD to run `hud_build_close` (T17 clears it)
    pub build_close_requested: bool,
}

/// `hud_slot_mode`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotMode {
    None,
    New,
    Load,
}

/// Effects of a menu step outside the simulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuEffect {
    /// `game_restart()` after "main menu" (the game was saved first)
    Restart,
    /// `game_end()`: quit; obj_game Other_3 saves on the way out
    Quit,
    /// `display_set_fullscreen(on)`
    SetFullscreen(bool),
    /// `display_cycle_size(dir)`
    CycleWindowSize(i32),
    /// `locale_cycle(dir)` (persisted to settings.ini by the app)
    CycleLanguage(i32),
    /// `audio_set_volume(kind, v)`; 0 = sfx, 1 = music (`hud_volume_channels`)
    SetVolume(usize, f64),
    /// `audio_save()`
    AudioSave,
}

/// Input of one menu step, GUI coordinates (view-sized, R17).
#[derive(Clone, Debug, Default)]
pub struct MenuInput {
    pub mx: f64,
    pub my: f64,
    pub click: bool,
    pub rclick: bool,
    pub lmb_down: bool,
    pub escape: bool,
    pub enter: bool,
    /// Backspace presses since the last step
    pub backspace: u32,
    /// typed text since the last step
    pub text: String,
    /// `display_get_gui_width/height()`
    pub gw: f64,
    pub gh: f64,
}

/// What the menus read from outside the simulation.
pub struct MenuEnv<'a> {
    pub locale: &'a Locale,
    /// current `audio_get_volume("sfx")`, `("music")`
    pub volumes: [f64; 2],
    /// `global.display_fullscreen`
    pub fullscreen: bool,
    /// `display_window_size()`
    pub window_size: (u32, u32),
    pub flags: &'a mut UiFlags,
}

/// Localised "damaged save" mark for a slot that cannot be read (no such key in the original's
/// table: corrupt saves are new in the port, Review Focus 3).
pub fn slot_corrupt_text(lang: &str) -> &'static str {
    match lang {
        "de" => "Beschädigt",
        "fr" => "Endommagé",
        "es" => "Dañada",
        "it" => "Danneggiato",
        "pt-BR" => "Danificado",
        "ru" => "Повреждён",
        "uk" => "Пошкоджено",
        "zh-Hant" => "已損壞",
        "zh-Hans" => "已损坏",
        "ko" => "손상됨",
        "ja" => "破損",
        _ => "Damaged",
    }
}

/// `length_text(cm)`.
pub fn length_text(locale: &Locale, imperial: bool, cm: f64) -> String {
    use myco_core::gml::round;
    if imperial {
        format!("{} {}", round(cm / 2.54) as i64, locale.loc("hud_in"))
    } else {
        format!("{} {}", round(cm) as i64, locale.loc("hud_cm"))
    }
}

/// The menus' (Task 18) share of `ui_blocking()` (`hud::ui_blocking` is the whole port): the
/// in-game menu, debug overlay, pause, logo screen, intro or title covers the game. The rest of
/// `ui_blocking()` (a tasks popup, a blocking cut-scene) is `hud::hud_blocks`, and the tree
/// screen writes its own flag; together they make `UiBlockers::any()` (no focus pause).
pub fn menus_block(sim: &Sim, flags: &UiFlags, logo_open: bool) -> bool {
    flags.menu_open
        || flags.debug_open
        || sim.pause_open
        || logo_open
        || sim.intro_active
        || sim.title_open
}

/// obj_hud's menu state.
#[derive(Resource, Clone, Debug)]
pub struct Menus {
    pub save_dir: Option<PathBuf>,
    /// `global.title_alpha`: menus fade out over half a second after the title closes
    pub title_alpha: f64,
    pub title_hover: i32,
    pub new_mode: GameMode,
    pub credits_open: bool,
    pub credits_back_hover: bool,
    pub slot_mode: SlotMode,
    pub slot_hover: i32,
    /// slot being named, -1 when not naming
    pub slot_naming: i32,
    /// `hud_slot_metas`: what `list_slots` found when the slot screen opened
    pub slot_metas: Vec<SlotStatus>,
    /// `hud_saves_exist`: enables "Load Game"
    pub saves_exist: bool,
    pub pause_hover: i32,
    pub volume_drag: i32,
    /// GameMaker `keyboard_string`
    pub keyboard_string: String,
    /// `global.logo_*`
    pub logo: LogoState,
    /// this step's Escape closed or toggled a menu other than the tree (the tree ignores it)
    pub escape_used: bool,
}

impl Menus {
    /// obj_hud Create_0 (menu part) and `logo_init` (game_init).
    pub fn new(save_dir: Option<PathBuf>) -> Menus {
        let saves_exist = save_dir.as_deref().is_some_and(save::any_save_exists);
        Menus {
            save_dir,
            title_alpha: 1.0,
            title_hover: -1,
            new_mode: GameMode::Exploration,
            credits_open: false,
            credits_back_hover: false,
            slot_mode: SlotMode::None,
            slot_hover: -1,
            slot_naming: -1,
            slot_metas: Vec::new(),
            saves_exist,
            pause_hover: -1,
            volume_drag: -1,
            keyboard_string: String::new(),
            logo: LogoState::logo_init(),
            escape_used: false,
        }
    }

    pub fn save_dir(&self) -> Option<&Path> {
        self.save_dir.as_deref()
    }

    /// `hud_pause_items`: the title's settings have no "main menu".
    pub fn pause_items(title_open: bool) -> &'static [&'static str] {
        if title_open {
            &PAUSE_ITEMS_TITLE
        } else {
            &PAUSE_ITEMS_GAME
        }
    }

    /// The menu branches of obj_hud Step_1.
    pub fn step(&mut self, sim: &mut Sim, inp: &MenuInput, env: &mut MenuEnv) -> Vec<MenuEffect> {
        let mut fx = Vec::new();
        // GameMaker keeps keyboard_string up to date from typed text and Backspace
        self.keyboard_string.push_str(&inp.text);
        for _ in 0..inp.backspace {
            self.keyboard_string.pop();
        }
        let title = sim.title_open;
        let popup = myco_core::progress::tasks::tasks_popup_open(sim);
        let cine = myco_core::camera::cine_blocking(sim);
        self.escape_used = false;
        if inp.escape && !popup && !cine {
            self.escape_used = !env.flags.menu_open
                || self.slot_mode != SlotMode::None
                || self.credits_open
                || env.flags.debug_open
                || self.logo.open;
            if self.slot_mode != SlotMode::None {
                if self.slot_naming >= 0 {
                    self.slot_naming = -1;
                } else {
                    self.hud_slot_close();
                }
            } else if self.credits_open {
                self.credits_open = false;
            } else if env.flags.debug_open {
                env.flags.debug_open = false;
            } else if self.logo.open {
                self.logo.open = false;
            } else if env.flags.menu_open {
                env.flags.menu_open = false;
            } else if env.flags.build_open {
                env.flags.build_close_requested = true;
            } else if sim.pause_open || !title {
                sim.pause_open = !sim.pause_open;
            }
        }
        if !title && self.title_alpha > 0.0 {
            self.title_alpha = (self.title_alpha - 0.03333333333333333).max(0.0);
        }
        self.pause_hover = -1;
        self.title_hover = -1;
        if cine && !popup {
            return fx;
        }
        if popup {
            // tasks_popup_step is the HUD's (T17)
            return fx;
        }
        if sim.pause_open {
            self.pause_step(inp, env, sim, &mut fx);
            return fx;
        }
        if env.flags.debug_open {
            return fx;
        }
        if title && self.slot_mode != SlotMode::None {
            self.slot_step(sim, inp, env);
            return fx;
        }
        if title && self.credits_open {
            self.credits_step(inp, sim.camera.view_y);
            return fx;
        }
        if title {
            self.title_step(sim, inp, &mut fx);
            return fx;
        }
        if self.logo.open {
            self.logo_screen_step(inp);
        }
        fx
    }

    // port: anon__15291__gml_Object_obj_hud_Create_0
    /// `hud_title_item_disabled(i)`: "Load Game" without any save.
    pub fn hud_title_item_disabled(&self, i: usize) -> bool {
        TITLE_ITEMS[i] == "load_game" && !self.saves_exist
    }

    fn title_step(&mut self, sim: &mut Sim, inp: &MenuInput, fx: &mut Vec<MenuEffect>) {
        let cam_y = sim.camera.view_y;
        let Some(i) = (0..TITLE_ITEMS.len())
            .find(|&i| hud_title_button_rect(inp.gw, cam_y, i).contains(inp.mx, inp.my))
        else {
            return;
        };
        if self.hud_title_item_disabled(i) {
            return;
        }
        self.title_hover = i as i32;
        if !inp.click {
            return;
        }
        match TITLE_ITEMS[i] {
            "exploration" => {
                self.new_mode = GameMode::Exploration;
                self.hud_slot_open(SlotMode::New);
            }
            "survival" => {
                self.new_mode = GameMode::Survival;
                self.hud_slot_open(SlotMode::New);
            }
            "load_game" => self.hud_slot_open(SlotMode::Load),
            "settings" => sim.pause_open = true,
            "credits" => {
                self.credits_open = true;
                self.credits_back_hover = false;
            }
            "exit" => fx.push(MenuEffect::Quit),
            _ => {}
        }
    }

    /// Title menu items, the version line, and the vignette while the title fades.
    fn title_draw(
        &self,
        d: &mut Vec<DrawCmd>,
        m: &dyn TextMetrics,
        loc: &Locale,
        gw: f64,
        cam_y: f64,
    ) {
        let ta = self.title_alpha;
        for (i, item) in TITLE_ITEMS.iter().enumerate() {
            let r = hud_title_button_rect(gw, cam_y, i);
            let label = loc.loc(&format!("title_{item}")).to_string();
            let sc = super::draw::hud_fit(
                m,
                FontSel::Main,
                &label,
                1.0,
                r.x2 - r.x1 - 24.0,
                r.y2 - r.y1,
            );
            let col = if self.hud_title_item_disabled(i) {
                COLOR_DEEP
            } else if self.title_hover == i as i32 {
                COLOR_INK
            } else {
                COLOR_DIM
            };
            shadow_text(d, r.cx(), r.cy(), &label, sc, col, ta, FontSel::Main, 1, 1);
        }
        let lr = hud_title_button_rect(gw, cam_y, TITLE_ITEMS.len() - 1);
        d.push(DrawCmd::Text {
            x: lr.cx(),
            y: lr.y2 + 12.0,
            s: VERSION.into(),
            sc: 1.1,
            col: COLOR_DIM,
            alpha: ta * 0.7,
            font: FontSel::Small,
            halign: 1,
            valign: 0,
        });
    }

    /// The menu branches of obj_hud Draw_75, in its order: vignette, slot / credits / title
    /// screens, then the settings overlay or the logo generator.
    pub fn draw(
        &mut self,
        sim: &Sim,
        m: &dyn TextMetrics,
        env: &MenuEnv,
        gw: f64,
        gh: f64,
        now_ms: f64,
    ) -> Vec<DrawCmd> {
        let mut d = Vec::new();
        let loc = env.locale;
        let cam_y = sim.camera.view_y;
        if self.title_alpha > 0.0 {
            d.push(DrawCmd::Tris(vignette(gw, gh, self.title_alpha)));
        }
        let ta = self.title_alpha;
        if ta > 0.0 && !sim.pause_open && self.slot_mode != SlotMode::None {
            self.slot_draw(&mut d, m, env, gw, cam_y, now_ms);
        } else if ta > 0.0 && !sim.pause_open && self.credits_open {
            self.credits_draw(&mut d, m, loc, gw, cam_y);
        } else if ta > 0.0 && !sim.pause_open {
            self.title_draw(&mut d, m, loc, gw, cam_y);
        }
        if env.flags.debug_open {
            return d;
        }
        if sim.pause_open {
            self.pause_draw(&mut d, m, env, sim.title_open, gw, gh);
            return d;
        }
        if self.logo.open {
            self.logo_screen_draw(&mut d, m, loc, gw, gh, now_ms);
        }
        d
    }
}

/// A text with the menus' drop shadow (black, 3 px, 60 % alpha).
#[allow(clippy::too_many_arguments)]
pub fn shadow_text(
    d: &mut Vec<DrawCmd>,
    x: f64,
    y: f64,
    s: &str,
    sc: f64,
    col: f64,
    ta: f64,
    font: FontSel,
    halign: u8,
    valign: u8,
) {
    d.push(DrawCmd::Text {
        x: x + 3.0,
        y: y + 3.0,
        s: s.into(),
        sc,
        col: 0.0,
        alpha: ta * 0.6,
        font,
        halign,
        valign,
    });
    d.push(DrawCmd::Text {
        x,
        y,
        s: s.into(),
        sc,
        col,
        alpha: ta,
        font,
        halign,
        valign,
    });
}

/// The title vignette: a 48×27 grid darkening towards the edges (`menu_vignette_*`).
pub fn vignette(gw: f64, gh: f64, title_alpha: f64) -> Vec<crate::render::blend::Vert> {
    use crate::render::blend::Vert;
    use myco_core::balance::vis;
    let vig = vis("menu_vignette_strength").clamp(0.0, 1.0) * title_alpha;
    let shape = vis("menu_vignette_shape").max(1.0);
    let vstart = vis("menu_vignette_start");
    let vwidth = vis("menu_vignette_width").max(0.01);
    let vpow = vis("menu_vignette_power").max(0.01);
    let (cols, rows) = (48usize, 27usize);
    let mut grid = vec![0.0; (rows + 1) * (cols + 1)];
    for gy in 0..=rows {
        for gx in 0..=cols {
            let nx = (gx as f64 / cols as f64 * 2.0 - 1.0).abs();
            let ny = (gy as f64 / rows as f64 * 2.0 - 1.0).abs();
            let dd = (nx.powf(shape) + ny.powf(shape)).powf(1.0 / shape);
            let mut k = ((dd - vstart) / vwidth).clamp(0.0, 1.0);
            k = k * k * (3.0 - 2.0 * k);
            grid[gy * (cols + 1) + gx] = vig * k.powf(vpow);
        }
    }
    let (cw, ch) = (gw / cols as f64, gh / rows as f64);
    let mut out = Vec::new();
    for gy in 0..rows {
        for gx in 0..cols {
            let a00 = grid[gy * (cols + 1) + gx];
            let a10 = grid[gy * (cols + 1) + gx + 1];
            let a01 = grid[(gy + 1) * (cols + 1) + gx];
            let a11 = grid[(gy + 1) * (cols + 1) + gx + 1];
            if a00 + a10 + a01 + a11 <= 0.0 {
                continue;
            }
            let (x0, y0) = (gx as f64 * cw, gy as f64 * ch);
            let (x1, y1) = (x0 + cw, y0 + ch);
            let v = |x, y, alpha| Vert {
                x,
                y,
                col: 0.0,
                alpha,
            };
            out.extend([
                v(x0, y0, a00),
                v(x1, y0, a10),
                v(x1, y1, a11),
                v(x0, y0, a00),
                v(x1, y1, a11),
                v(x0, y1, a01),
            ]);
        }
    }
    out
}

// port: anon__15409__gml_Object_obj_hud_Create_0
/// `hud_title_button_rect(i)`: 420×60 items from 90 px below the surface line.
pub fn hud_title_button_rect(gw: f64, cam_y: f64, i: usize) -> R {
    let top = wld("surface_height_px") - cam_y + 90.0;
    let (w, h) = (420.0, 60.0);
    let x = (gw - w) * 0.5;
    let y = top + i as f64 * (h + 8.0);
    R {
        x1: x,
        y1: y,
        x2: x + w,
        y2: y + h,
    }
}

// port: anon__16771__gml_Object_obj_hud_Create_0
/// `hud_close_button_rect()`: top-right CLOSE button of the full-screen panels (the tree screen
/// has the same rect as `TreeView::close_button_rect`).
pub fn hud_close_button_rect(gw: f64) -> R {
    R {
        x1: gw - 152.0,
        y1: 30.0,
        x2: gw - 32.0,
        y2: 76.0,
    }
}

/// The save's mode name (`"exploration"` / `"survival"`).
pub fn mode_str(m: GameMode) -> &'static str {
    match m {
        GameMode::Exploration => "exploration",
        GameMode::Survival => "survival",
    }
}
