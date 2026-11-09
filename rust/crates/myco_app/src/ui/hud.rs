//! obj_hud (Create_0, Step_1 = Begin Step, Draw_75 = Draw GUI) and its `hud_*` methods.
//!
//! [`HudState`] is obj_hud's instance state plus the HUD globals the original keeps in
//! `global` (`menu_open`, `logo_open`, `debug_open`, `hud_hidden`, `units_imperial`,
//! `title_alpha`, `ui_capture_mouse`, `right_click_used`). The geometry methods are pure
//! functions of it; the Begin Step runs in `FixedUpdate` before each simulation step and turns
//! clicks into `Command`s; Draw GUI paints into the GUI painter ([`Gm`]).
//!
//! Branches of Step_1/Draw_75 for the modal screens are split out to the tasks that own those
//! screens: pause, title, slots, credits and logo (Task 18) and the upgrade tree (Task 19) run
//! their systems in [`HudStepSet::Screens`](super::HudStepSet) and
//! [`HudDrawSet::Title`](super::HudDrawSet) / `Modal`, using the rects and [`TreeView`] here.
//! The in-game branch skips while any of them is open, as the original returns early.

pub use super::fonts::UiFonts;
use super::gm_draw::{Gm, GuiDraw};
use super::markers::{self, AnimalMarkers};
use super::menu::{Menus, UiFlags};
use super::placement::{self, RmbAction};
use super::tasks as ui_tasks;
use super::tree::{tree_draw_glow, tree_draw_icon, TreeScreen};
use super::{draw_order, min_text_scale, LocaleRes, MIN_TEXT_PX};
use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::sim_bridge::{PendingInput, SimEventMsg, SimRes, UiBlockers, UiCapture};
use bevy::prelude::*;
use myco_core::balance::{bal, vis};
use myco_core::camera::cine_blocking;
use myco_core::events::SimEvent;
use myco_core::gml;
use myco_core::locale::Locale;
use myco_core::mushrooms::{self, MushroomKind};
use myco_core::mycelium::soil::mycelium_cm;
use myco_core::progress::tasks;
use myco_core::sim::{Command, Key, Sim};
use myco_core::state::GameMode;

/// obj_hud colours (BGR).
pub const COLOR_INK: f64 = 14873582.0;
pub const COLOR_DIM: f64 = 8689804.0;
pub const COLOR_DEEP: f64 = 4609098.0;
pub const COLOR_GOOD: f64 = 9098922.0;
pub const COLOR_BAD: f64 = 6846166.0;
pub const COLOR_GOLD: f64 = 7914718.0;
pub const COLOR_PANEL: f64 = 1185298.0;

/// A rectangle (`x1, y1, x2, y2` structs of the original).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct R {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl R {
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> R {
        R { x1, y1, x2, y2 }
    }
    pub fn contains(&self, x: f64, y: f64) -> bool {
        gml::point_in_rectangle(x, y, self.x1, self.y1, self.x2, self.y2)
    }
}

/// A sprite placement (`___struct___433..439`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
}

/// `hud_ui` (`___struct___432`): the top bar.
#[derive(Clone, Debug, PartialEq)]
pub struct HudUi {
    pub wind_tab: Place,
    pub upgrade_tab: Place,
    pub bar: Place,
    pub upgrade_icon: Place,
    pub food_icon: Place,
    pub water_icon: Place,
    pub upgrade_button: Place,
    pub icon_gap: f64,
    pub length_x: f64,
    pub text_y: f64,
    pub side_scale: f64,
    pub length_scale: f64,
    pub unit_scale: f64,
    pub unit_gap: f64,
    pub points_x: f64,
    pub points_y: f64,
    pub points_scale: f64,
    pub wind_x: f64,
    pub wind_y: f64,
    pub wind_scale: f64,
    pub press_scale: f64,
    pub press_frames: f64,
}

impl HudUi {
    /// `___struct___432(a0, a1, a2)` with Create_0's arguments (-32, -32, -192).
    fn new(a0: f64, a1: f64, a2: f64) -> HudUi {
        let p = |x, y, scale| Place { x, y, scale };
        HudUi {
            wind_tab: p(378.7, a0, 1.2),
            upgrade_tab: p(1363.2, a1, 1.2),
            bar: p(a2, 0.0, 1.2),
            upgrade_icon: p(1408.0, 10.8, 0.4),
            food_icon: p(672.0, 10.8, 0.4),
            water_icon: p(1206.0, 10.8, 0.4),
            upgrade_button: p(1744.0, 61.6, 1.0),
            icon_gap: 6.0,
            length_x: 960.0,
            text_y: 24.0,
            side_scale: 0.5,
            length_scale: 0.8,
            unit_scale: 0.42,
            unit_gap: 8.0,
            points_x: 1442.0,
            points_y: 24.0,
            points_scale: 0.6,
            wind_x: 464.5,
            wind_y: 24.0,
            wind_scale: 0.45,
            press_scale: 0.88,
            press_frames: 10.0,
        }
    }
}

/// `hud_build` (`___struct___442`): the mushroom build button and bar.
#[derive(Clone, Debug, PartialEq)]
pub struct HudBuild {
    pub circle_x: f64,
    pub circle_y: f64,
    pub bar_x: f64,
    pub bar_y: f64,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
    pub bar_right: f64,
    pub slot_x0: f64,
    pub slot_dx: f64,
    pub slot_y: f64,
    pub slot_tail: f64,
    pub price_gap: f64,
    pub icon: f64,
    pub price_scale: f64,
    pub price_line: f64,
    pub bite_size: f64,
    pub slide_frames: f64,
}

impl HudBuild {
    fn new(price_gap: f64) -> HudBuild {
        HudBuild {
            circle_x: 64.0,
            circle_y: 864.0,
            bar_x: 32.0,
            bar_y: 864.0,
            cx: 126.0,
            cy: 925.0,
            r: 60.0,
            bar_right: 1269.0,
            slot_x0: 240.0,
            slot_dx: 125.0,
            slot_y: 925.0,
            slot_tail: 154.0,
            price_gap,
            icon: 74.0,
            price_scale: 0.42,
            price_line: 26.0,
            bite_size: 22.0,
            slide_frames: 24.0,
        }
    }
}

/// `hud_camnav` (`___struct___443`): camera buttons.
#[derive(Clone, Debug, PartialEq)]
pub struct HudCamnav {
    pub left_x: f64,
    pub origin_x: f64,
    pub right_x: f64,
    pub y: f64,
    pub origin_r: f64,
    pub arrow_pad: f64,
    pub hover_scale: f64,
}

/// `myc_place_check` result as the HUD draws it (`hud_place_at`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaceAt {
    pub ok: bool,
    pub x: f64,
    pub y: f64,
    pub face: Option<f64>,
}

/// obj_hud state plus the HUD globals.
#[derive(Resource, Clone, Debug)]
pub struct HudState {
    pub ui: HudUi,
    pub build: HudBuild,
    pub camnav: HudCamnav,
    pub layout_ox: f64,
    pub layout_by: f64,
    pub wind_hover: bool,
    pub points_hover: bool,
    pub build_btn_hover: bool,
    pub build_btn_press: f64,
    pub upgrade_btn_hover: bool,
    pub upgrade_btn_press: f64,
    pub cam_hover: i32,
    pub cam_press: [f64; 3],
    pub build_open: bool,
    pub build_slide: f64,
    pub build_hover: i32,
    /// `hud_place_type` (-1 = None)
    pub place_type: Option<MushroomKind>,
    /// `hud_place_at` (the core's placement check, ruling R22)
    pub place_at: Option<PlaceAt>,
    /// `global.ui_capture_mouse`
    pub ui_capture_mouse: bool,
    /// `cave_hover_tip(...)` locale key under the cursor, computed in the Begin Step (it may
    /// build the cave rects, so the read-only Draw GUI uses this)
    pub cave_tip: String,
    /// `global.right_click_used`
    pub right_click_used: bool,
    /// Read-only mirrors of state the menus (Task 18: `UiFlags`, `Menus.logo`) and the tree
    /// (Task 19: `TreeScreen.open`) own, copied at the start of every Begin Step.
    pub menu_open: bool,
    pub debug_open: bool,
    pub logo_open: bool,
    pub hud_hidden: bool,
    pub units_imperial: bool,
    /// `global.animal_marker_hover` (marker source id)
    pub animal_marker_hover: Option<u64>,
    /// the marker `camera_fly` flies to (`global.camera_fly.marker`; its source id)
    pub fly_marker: Option<u64>,
    /// the cursor (GUI) of the last Begin Step
    pub mouse_gui: (f64, f64),
    /// cursor in the world of the last Begin Step (`mouse_x/y`)
    pub mouse_world: (f64, f64),
}

impl Default for HudState {
    fn default() -> Self {
        HudState::new()
    }
}

impl HudState {
    // port: gml_Object_obj_hud_Create_0
    /// obj_hud Create_0, in-game part (the menu fields are Task 18's `Menus`, the tree's
    /// Task 19's `TreeScreen`).
    pub fn new() -> HudState {
        let mut h = HudState {
            ui: HudUi::new(-32.0, -32.0, -192.0),
            build: HudBuild::new(-10.0),
            camnav: HudCamnav {
                left_x: 1571.0,
                origin_x: 1663.0,
                right_x: 1755.0,
                y: 864.0,
                origin_r: 60.0,
                arrow_pad: 8.0,
                hover_scale: 1.08,
            },
            layout_ox: 0.0,
            layout_by: 0.0,
            wind_hover: false,
            points_hover: false,
            build_btn_hover: false,
            build_btn_press: 0.0,
            upgrade_btn_hover: false,
            upgrade_btn_press: 0.0,
            cam_hover: -1,
            cam_press: [0.0; 3],
            build_open: false,
            build_slide: 0.0,
            build_hover: -1,
            place_type: None,
            place_at: None,
            ui_capture_mouse: false,
            cave_tip: String::new(),
            right_click_used: false,
            menu_open: false,
            debug_open: false,
            logo_open: false,
            hud_hidden: false,
            units_imperial: false,
            animal_marker_hover: None,
            fly_marker: None,
            mouse_gui: (-1.0, -1.0),
            mouse_world: (0.0, 0.0),
        };
        // Create_0 calls hud_layout_apply once with ui_ox = ui_by = 0: no change
        h.layout_apply(0.0, 0.0);
        h
    }

    // port: hud_layout_apply
    /// Follow `global.ui_ox` / `ui_by` (1920-wide UI centred, bottom-anchored): shift the top
    /// bar by the x change, the camera buttons by twice the x change and the bottom rows by the
    /// y change.
    pub fn layout_apply(&mut self, ui_ox: f64, ui_by: f64) {
        let dx = ui_ox - self.layout_ox;
        let dy = ui_by - self.layout_by;
        if dx == 0.0 && dy == 0.0 {
            return;
        }
        self.layout_ox = ui_ox;
        self.layout_by = ui_by;
        let u = &mut self.ui;
        for p in [
            &mut u.wind_tab,
            &mut u.upgrade_tab,
            &mut u.bar,
            &mut u.upgrade_icon,
            &mut u.food_icon,
            &mut u.water_icon,
            &mut u.upgrade_button,
        ] {
            p.x += dx;
        }
        u.length_x += dx;
        u.points_x += dx;
        u.wind_x += dx;
        let c = &mut self.camnav;
        c.left_x += dx * 2.0;
        c.origin_x += dx * 2.0;
        c.right_x += dx * 2.0;
        c.y += dy;
        let b = &mut self.build;
        b.circle_y += dy;
        b.bar_y += dy;
        b.cy += dy;
        b.slot_y += dy;
    }

    fn tab_rect(t: &Place, assets: &GameAssets) -> R {
        let s = assets.sprite("UI_PIPKA").size.as_dvec2();
        R::new(t.x, 0.0, t.x + s.x * t.scale, t.y + s.y * t.scale)
    }

    // port: hud_points_tab_rect
    /// The upgrade-points tab under the top bar.
    pub fn points_tab_rect(&self, assets: &GameAssets) -> R {
        Self::tab_rect(&self.ui.upgrade_tab, assets)
    }

    // port: hud_wind_tab_rect
    /// The wind tab under the top bar.
    pub fn wind_tab_rect(&self, assets: &GameAssets) -> R {
        Self::tab_rect(&self.ui.wind_tab, assets)
    }

    // port: hud_cam_sprite
    pub fn cam_sprite(i: usize) -> &'static str {
        ["CAMERA_LEFT", "ORIGIN_SHROOM", "CAMERA_RIGHT"][i]
    }

    // port: hud_cam_x
    pub fn cam_x(&self, i: usize) -> f64 {
        let c = &self.camnav;
        [c.left_x, c.origin_x, c.right_x][i]
    }

    // port: hud_cam_pick
    /// The camera button under the point: the round origin button by radius, the arrows by
    /// their padded bbox; -1 for none.
    pub fn cam_pick(&self, assets: &GameAssets, mx: f64, my: f64) -> i32 {
        let c = &self.camnav;
        for i in 0..3 {
            let spr = assets.sprite(Self::cam_sprite(i));
            let x = self.cam_x(i);
            let (w, h) = (spr.size.x as f64, spr.size.y as f64);
            if i == 1 {
                if gml::point_distance(mx, my, x + w * 0.5, c.y + h * 0.5) <= c.origin_r {
                    return 1;
                }
            } else {
                let p = c.arrow_pad;
                let bb = spr.bbox.map(|v| v as f64);
                if gml::point_in_rectangle(
                    mx,
                    my,
                    x + bb[0] - p,
                    c.y + bb[1] - p,
                    x + bb[2] + p,
                    c.y + bb[3] + p,
                ) {
                    return i as i32;
                }
            }
        }
        -1
    }

    fn press_scale(&self, press: f64) -> f64 {
        let t = 1.0 - press / self.ui.press_frames.max(1.0);
        if press > 0.0 {
            gml::lerp(1.0, self.ui.press_scale, (t * std::f64::consts::PI).sin())
        } else {
            1.0
        }
    }

    // port: hud_cam_draw
    /// The three camera buttons, pressed (squeeze) and hovered (grow).
    pub fn cam_draw(&self, g: &mut Gm, assets: &GameAssets) {
        let c = &self.camnav;
        for i in 0..3 {
            let name = Self::cam_sprite(i);
            let spr = assets.sprite(name);
            let (w, h) = (spr.size.x as f64, spr.size.y as f64);
            let mut s = self.press_scale(self.cam_press[i]);
            if self.cam_hover == i as i32 {
                s *= c.hover_scale;
            }
            let x = self.cam_x(i);
            g.sprite_ext(
                name,
                0,
                x + w * 0.5 * (1.0 - s),
                c.y + h * 0.5 * (1.0 - s),
                s,
                s,
                C_WHITE,
                1.0,
            );
        }
    }

    // port: hud_draw_button
    /// A round button: hover sprite when `cond`, squeezed while pressed.
    #[allow(clippy::too_many_arguments)]
    pub fn draw_button(
        &self,
        g: &mut Gm,
        assets: &GameAssets,
        spr_a: &str,
        spr_b: &str,
        x: f64,
        y: f64,
        cond: bool,
        press: f64,
    ) {
        let spr = if cond { spr_b } else { spr_a };
        let def = assets.sprite(spr);
        let (w, h) = (def.size.x as f64, def.size.y as f64);
        let s = self.press_scale(press);
        g.sprite_ext(
            spr,
            0,
            x + w * 0.5 * (1.0 - s),
            y + h * 0.5 * (1.0 - s),
            s,
            s,
            C_WHITE,
            1.0,
        );
    }

    // port: hud_build_all_types
    pub fn build_all_types() -> [MushroomKind; 8] {
        MushroomKind::ALL
    }

    // port: hud_build_key_of
    /// The number key of a type (1..8), 0 when none.
    pub fn build_key_of(t: MushroomKind) -> usize {
        Self::build_all_types()
            .iter()
            .position(|k| *k == t)
            .map_or(0, |i| i + 1)
    }

    // port: hud_build_types
    /// The unlocked types, in key order.
    pub fn build_types(sim: &Sim) -> Vec<MushroomKind> {
        Self::build_all_types()
            .into_iter()
            .filter(|t| mushrooms::mushroom_type_unlocked(&sim.state, *t))
            .collect()
    }

    // port: hud_build_slot_step
    /// Slot spacing, squeezed so all unlocked types fit the bar.
    pub fn build_slot_step(&self, n: usize) -> f64 {
        let b = &self.build;
        if n <= 1 {
            return b.slot_dx;
        }
        b.slot_dx
            .min((b.bar_right - b.slot_x0 - b.slot_tail) / (n as f64 - 1.0))
    }

    // port: hud_build_slot_rect
    /// Hit rect of slot `i` (icon plus price) for `n` unlocked types.
    pub fn build_slot_rect(&self, i: usize, n: usize) -> R {
        let b = &self.build;
        let step = self.build_slot_step(n);
        let x = b.slot_x0 + i as f64 * step;
        R::new(
            x - b.icon * 0.5,
            b.slot_y - b.icon * 0.5,
            x + step - 24.0,
            b.slot_y + b.icon * 0.5,
        )
    }

    // port: hud_build_mouse_over
    /// The cursor is on the build button or (when mostly slid out) on the bar.
    pub fn build_mouse_over(&self, mx: f64, my: f64) -> bool {
        let b = &self.build;
        if gml::point_distance(mx, my, b.cx, b.cy) <= b.r {
            return true;
        }
        self.build_slide > 0.5
            && gml::point_in_rectangle(mx, my, b.cx, b.bar_y, b.bar_right, b.bar_y + 123.0)
    }

    // port: hud_build_close
    /// Close the bar and leave placement mode.
    pub fn build_close(&mut self) {
        self.build_open = false;
        self.place_type = None;
        self.place_at = None;
    }

    // port: hud_upgrades_button_rect
    pub fn upgrades_button_rect(&self, assets: &GameAssets) -> R {
        let b = &self.ui.upgrade_button;
        let s = assets.sprite("UPGRADE_CALM").size.as_dvec2();
        R::new(b.x, b.y, b.x + s.x, b.y + s.y)
    }

    // port: hud_upgrades_button_over
    /// Round hit test of the upgrades button.
    pub fn upgrades_button_over(&self, assets: &GameAssets, mx: f64, my: f64) -> bool {
        let r = self.upgrades_button_rect(assets);
        gml::point_distance(mx, my, (r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5) <= (r.x2 - r.x1) * 0.5
    }

    /// `ui_blocking()`.
    pub fn ui_blocking(&self, sim: &Sim) -> bool {
        ui_blocking(sim, self)
    }

    /// Any of the screens whose Begin Step branch returns before the in-game HUD.
    pub fn modal_open(&self, sim: &Sim) -> bool {
        tasks::tasks_popup_open(sim)
            || cine_blocking(sim)
            || sim.pause_open
            || self.debug_open
            || sim.title_open
            || self.logo_open
            || self.menu_open
    }
}

// port: ui_blocking
/// A menu, the debug overlay, the pause or logo screen, the intro, the title, a tasks popup or
/// a blocking cut-scene takes the mouse.
pub fn ui_blocking(sim: &Sim, hud: &HudState) -> bool {
    hud.menu_open
        || hud.debug_open
        || sim.pause_open
        || hud.logo_open
        || sim.intro_active
        || sim.title_open
        || tasks::tasks_popup_open(sim)
        || cine_blocking(sim)
}

/// `length_text(cm)` (Task 8's row, used by the slot labels and the points tooltip).
pub fn length_text(cm: f64, imperial: bool, loc: &Locale) -> String {
    if imperial {
        format!("{} {}", fmt_num(gml::round(cm / 2.54)), loc.loc("hud_in"))
    } else {
        format!("{} {}", fmt_num(gml::round(cm)), loc.loc("hud_cm"))
    }
}

/// `string(x)` for a number: integers without decimals, otherwise up to 2 decimals (GML).
pub fn fmt_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn mode_str(m: GameMode) -> &'static str {
    match m {
        GameMode::Exploration => "exploration",
        GameMode::Survival => "survival",
    }
}

// -----------------------------------------------------------------------------------------
// Begin Step

/// The input of one Begin Step as the original reads it.
pub struct StepIn {
    pub mx: f64,
    pub my: f64,
    pub world: Option<(f64, f64)>,
    pub click: bool,
    pub rclick: bool,
    pub lmb_down: bool,
    pub keys: Vec<Key>,
    pub gw: f64,
    pub gh: f64,
}

impl StepIn {
    pub fn from_pending(p: &PendingInput, cams: &Cameras) -> StepIn {
        let screen = p.cursor_override.or(p.cursor);
        let (mx, my) = screen.map_or((-1.0, -1.0), |s| cams.screen_to_gui(s).into());
        StepIn {
            mx,
            my,
            world: screen.map(|s| cams.screen_to_world(s).into()),
            click: p.lmb.pressed,
            rclick: p.rmb.pressed,
            lmb_down: p.lmb.down,
            keys: p.keys_pressed.clone(),
            gw: cams.layout.view_w,
            gh: cams.layout.view_h,
        }
    }

    fn pressed(&self, k: Key) -> bool {
        self.keys.contains(&k)
    }
}

/// Step_1 up to the screen branches: layout, the build-close request of the Escape chain, hovers, tasks headers, animal markers,
/// mouse capture. Returns the commands to send.
#[allow(clippy::too_many_arguments)]
pub fn step_pre(
    hud: &mut HudState,
    sim: &Sim,
    assets: &GameAssets,
    ui_fonts: &UiFonts,
    loc: &Locale,
    markers: &AnimalMarkers,
    ui_ox: f64,
    ui_by: f64,
    build_close_requested: bool,
    inp: &StepIn,
) -> Vec<Command> {
    let mut cmds = Vec::new();
    hud.layout_apply(ui_ox, ui_by);
    // water_counter_step: the logic's hud_begin_step. The Escape chain is Task 18's
    // (`Menus::step`, run just before): its build-menu step arrives here as a request.
    if build_close_requested {
        hud.build_close();
    }
    let (mx, my) = (inp.mx, inp.my);
    hud.mouse_gui = (mx, my);
    if let Some(w) = inp.world {
        hud.mouse_world = w;
    }
    hud.right_click_used = false;
    let hb = &hud.build;
    hud.build_btn_hover = gml::point_distance(mx, my, hb.cx, hb.cy) <= hb.r;
    hud.upgrade_btn_hover = hud.upgrades_button_over(assets, mx, my);
    hud.build_btn_press = (hud.build_btn_press - 1.0).max(0.0);
    for p in &mut hud.cam_press {
        *p = (*p - 1.0).max(0.0);
    }
    let blocking = ui_blocking(sim, hud);
    hud.cam_hover = if hud.hud_hidden || sim.title_open || sim.intro_active || blocking {
        -1
    } else {
        hud.cam_pick(assets, mx, my)
    };
    hud.wind_hover = !blocking && hud.wind_tab_rect(assets).contains(mx, my);
    hud.points_hover = !blocking && hud.points_tab_rect(assets).contains(mx, my);
    hud.upgrade_btn_press = (hud.upgrade_btn_press - 1.0).max(0.0);
    let tasks_hit = ui_tasks::tasks_mouse_step(
        sim, hud, assets, ui_fonts, loc, ui_ox, mx, my, inp.click, &mut cmds,
    );
    hud.animal_marker_hover = if !sim.title_open && !sim.intro_active && !blocking {
        markers::animal_marker_at(markers, mx, my)
    } else {
        None
    };
    // a running fly reads its marker live (`fly.marker.x/y` in the forest step): send where the
    // marker is now (the markers move with their animals, and drift on once the animal is gone)
    if sim.camera.fly.is_none() {
        hud.fly_marker = None;
    } else if let Some(m) = hud
        .fly_marker
        .and_then(|src| markers.list.iter().find(|m| m.src == src))
    {
        cmds.push(Command::CameraFlyTrack { x: m.x, y: m.y });
    }
    if let Some(src) = hud.animal_marker_hover {
        if inp.click {
            if let Some(m) = markers.list.iter().find(|m| m.src == src) {
                // camera_fly_to_marker
                cmds.push(Command::CameraFlyTo { x: m.x, y: m.y });
                hud.fly_marker = Some(src);
            }
        }
    }
    // the original's if/elseif chain (all `true` branches merged)
    let capture = blocking
        || tasks_hit
        || (!hud.hud_hidden && hud.upgrade_btn_hover)
        || hud.cam_hover >= 0
        || hud.animal_marker_hover.is_some()
        || (!hud.hud_hidden && (hud.place_type.is_some() || hud.build_mouse_over(mx, my)));
    hud.ui_capture_mouse = capture;
    // cine_step and its gates: the logic's (`progress::hud_begin_step`)
    cmds
}

/// What the in-game branch did besides commands (for tests).
#[derive(Debug, Default, PartialEq)]
pub struct GameStepOut {
    pub commands: Vec<Command>,
    pub opened_tree: bool,
}

// port: gml_Object_obj_hud_Step_1
/// obj_hud Begin Step, in-game branch (after [`step_pre`]; skipped while a popup, cut-scene,
/// pause, title, logo, debug or tree screen is up): upgrade button → tree, build bar slide,
/// right click (cancel placement, else the Rust port's quick node), camera buttons, number
/// keys, slot hover, placement mode and the build button.
pub fn step_game(hud: &mut HudState, sim: &Sim, inp: &StepIn) -> GameStepOut {
    let mut out = GameStepOut::default();
    if hud.modal_open(sim) {
        return out;
    }
    let (mx, my) = (inp.mx, inp.my);
    let click = inp.click;
    // hud_tree_fitted / hud_tree_drag reset: Task 19's TreeView::closed_step
    if !hud.hud_hidden && click && hud.upgrade_btn_hover {
        hud.upgrade_btn_press = hud.ui.press_frames;
        hud.build_close();
        hud.menu_open = true;
        out.opened_tree = true;
        return out;
    }
    let dir = if hud.build_open { 1.0 } else { -1.0 };
    hud.build_slide = gml::clamp(
        hud.build_slide + dir / hud.build.slide_frames.max(1.0),
        0.0,
        1.0,
    );
    hud.build_hover = -1;
    if hud.hud_hidden || sim.intro_active {
        hud.build_close();
        return out;
    }
    // right click: in placement mode it cancels (original); otherwise the Rust port places a
    // node where the cursor is, unless the cursor is over the HUD or the left button is held
    match placement::rmb_route(
        inp.rclick,
        inp.lmb_down,
        hud.place_type.is_some() || hud.build_open,
        hud.ui_capture_mouse,
        inp.world,
    ) {
        RmbAction::CancelPlacement => {
            hud.right_click_used = true;
            hud.build_close();
            return out;
        }
        RmbAction::QuickNode { x, y } => {
            hud.right_click_used = true;
            out.commands.push(Command::QuickNode { x, y });
        }
        RmbAction::None => {}
    }
    if click && hud.cam_hover >= 0 {
        let ch = hud.cam_hover as usize;
        hud.cam_press[ch] = hud.ui.press_frames;
        out.commands.push(match ch {
            0 => Command::CameraJumpEdge { dir: -1.0 },
            1 => Command::CameraJumpOrigin,
            _ => Command::CameraJumpEdge { dir: 1.0 },
        });
        return out;
    }
    let b = hud.build.clone();
    let on_circle = gml::point_distance(mx, my, b.cx, b.cy) <= b.r;
    let types = HudState::build_types(sim);
    let all = HudState::build_all_types();
    let key = (0..all.len().min(8)).find(|&k| {
        let digit = char::from(b'1' + k as u8);
        inp.pressed(Key::Char(digit)) || inp.pressed(Key::Numpad(k as u8 + 1))
    });
    if let Some(k) = key {
        let t = all[k];
        if mushrooms::mushroom_type_unlocked(&sim.state, t) {
            hud.build_open = true;
            if mushrooms::mushroom_type_affordable(sim, t) {
                hud.place_type = Some(t);
                hud.place_at = None;
            }
            return out;
        }
    }
    if hud.build_open && hud.place_type.is_none() && hud.build_slide > 0.9 {
        for i in 0..types.len() {
            if hud.build_slot_rect(i, types.len()).contains(mx, my) {
                hud.build_hover = i as i32;
            }
        }
    }
    if let Some(t) = hud.place_type {
        let (wx, wy) = hud.mouse_world;
        // hud_place_at = myc_place_check(...): ruling R22, the core runs it from
        // `StepInput.placement` (= `place_type`, set by `hud_step_game`) and stores
        // `sim.mushrooms.place_preview`, which the Draw GUI reads
        hud.place_at = None;
        if click && !on_circle && !hud.build_mouse_over(mx, my) {
            // myc_place_mushroom: Task 9 answers with SimEvent::MushroomPlaced, which closes
            // the bar (`hud_events`)
            out.commands.push(Command::PlaceMushroom {
                kind: t,
                x: wx,
                y: wy,
            });
        } else if click && on_circle {
            hud.build_btn_press = hud.ui.press_frames;
            hud.build_close();
        }
        return out;
    }
    if click && on_circle {
        hud.build_btn_press = hud.ui.press_frames;
        hud.build_open = !hud.build_open;
    } else if click && hud.build_hover >= 0 {
        let t = types[hud.build_hover as usize];
        if mushrooms::mushroom_type_affordable(sim, t) {
            hud.place_type = Some(t);
        }
    }
    out
}

/// Copy the state the menus (Task 18) and the tree (Task 19) own into the HUD's mirrors.
pub fn sync_mirrors(
    hud: &mut HudState,
    flags: Option<&UiFlags>,
    menus: Option<&Menus>,
    tree: Option<&TreeScreen>,
) {
    hud.menu_open = tree.is_some_and(|t| t.open);
    if let Some(f) = flags {
        hud.debug_open = f.debug_open;
        hud.hud_hidden = f.hud_hidden;
        hud.units_imperial = f.units_imperial;
    }
    hud.logo_open = menus.is_some_and(|m| m.logo.open);
}

/// The HUD's part of `ui_blocking()` that the menus' blocker lacks: a tasks popup or a blocking
/// cut-scene (no pause on focus loss then).
pub fn hud_blocks(sim: &Sim) -> bool {
    tasks::tasks_popup_open(sim) || cine_blocking(sim)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn hud_step_pre(
    mut hud: ResMut<HudState>,
    mut sim: ResMut<SimRes>,
    assets: Res<GameAssets>,
    ui_fonts: Res<UiFonts>,
    loc: Res<LocaleRes>,
    markers: Res<AnimalMarkers>,
    cams: Res<Cameras>,
    ui: (
        Option<Res<TreeScreen>>,
        Option<ResMut<UiFlags>>,
        Option<Res<Menus>>,
    ),
    mut pending: ResMut<PendingInput>,
    mut out: (ResMut<UiBlockers>, ResMut<UiCapture>),
) {
    let (tree, mut flags, menus) = ui;
    sync_mirrors(
        &mut hud,
        flags.as_deref(),
        menus.as_deref(),
        tree.as_deref(),
    );
    // the Escape chain's build step (Task 18's `Menus::step`, run just before)
    let close = flags
        .as_mut()
        .is_some_and(|f| std::mem::take(&mut f.build_close_requested));
    let inp = StepIn::from_pending(&pending, &cams);
    let cmds = step_pre(
        &mut hud,
        &sim.0,
        &assets,
        &ui_fonts,
        &loc.locale,
        &markers,
        cams.layout.ui_ox,
        cams.layout.ui_by,
        close,
        &inp,
    );
    pending.commands.extend(cmds);
    // Draw_75's cave tooltip lookup, done here so drawing stays read-only
    hud.cave_tip = if ui_blocking(&sim.0, &hud) {
        String::new()
    } else {
        let (vx, vy) = view_xy(&cams);
        myco_core::world::caves::cave_hover_tip(
            &mut sim.0,
            inp.mx,
            inp.my,
            vis("cave_icons_x") + cams.layout.ui_ox,
            vis("cave_icons_y"),
            vx,
            vy,
        )
    };
    pending.tasks_popup_open = tasks::tasks_popup_open(&sim.0);
    out.0.hud = hud_blocks(&sim.0);
    out.1.hud = hud.ui_capture_mouse;
    if let Some(f) = flags.as_mut() {
        f.build_open = hud.build_open || hud.place_type.is_some();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn hud_step_game(
    mut hud: ResMut<HudState>,
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    tree: Option<ResMut<TreeScreen>>,
    flags: Option<ResMut<UiFlags>>,
    menus: Option<Res<Menus>>,
    mut pending: ResMut<PendingInput>,
    mut capture: ResMut<UiCapture>,
) {
    let (mut tree, mut flags) = (tree, flags);
    sync_mirrors(
        &mut hud,
        flags.as_deref(),
        menus.as_deref(),
        tree.as_deref(),
    );
    let inp = StepIn::from_pending(&pending, &cams);
    let out = step_game(&mut hud, &sim.0, &inp);
    pending.commands.extend(out.commands);
    // ruling R22: the core runs `myc_place_check` for this type (obj_hud Step_1's placement
    // branch, gated in core by `hud_reaches_placement`) and stores the preview
    pending.placement = hud.place_type;
    if out.opened_tree {
        // the upgrade button: `global.menu_open = true` (the tree screen, Task 19)
        if let Some(t) = tree.as_mut() {
            t.open();
        }
        if let Some(f) = flags.as_mut() {
            f.menu_open = true;
        }
    }
    capture.hud = hud.ui_capture_mouse;
    if let Some(f) = flags.as_mut() {
        f.build_open = hud.build_open || hud.place_type.is_some();
    }
}

/// After each step: the tasks panel's check-mark fill (`tasks_block_draw`'s animation).
pub fn hud_tasks_fill(mut sim: ResMut<SimRes>, hud: Res<HudState>, loc: Res<LocaleRes>) {
    ui_tasks::tasks_fill_step(&mut sim.0, &hud, &loc.locale);
}

/// Events after a step: a placed mushroom closes the bar (`if ok then hud_build_close()`).
pub fn hud_events(mut hud: ResMut<HudState>, mut ev: MessageReader<SimEventMsg>) {
    for e in ev.read() {
        if let SimEvent::MushroomPlaced { .. } = e.0 {
            hud.build_close();
        }
    }
}

// -----------------------------------------------------------------------------------------
// Draw GUI

/// `draw_text_transformed(x, y, s, xs, ys, 0)` with the owner's minimum on-screen size
/// ([`MIN_TEXT_PX`]).
pub fn text(g: &mut Gm, x: f64, y: f64, s: &str, xs: f64, ys: f64) {
    let lh = g.string_height("M");
    let k = min_text_scale(ys, lh, MIN_TEXT_PX) / if ys == 0.0 { 1.0 } else { ys };
    g.text_transformed(x, y, s, xs * k, ys * k);
}

/// The scale text drawn with [`text`] at scale `sc` really gets (the minimum size applied) in
/// the painter's current font: layout measures with it so enlarged text keeps its place.
pub fn eff_scale(g: &Gm, sc: f64) -> f64 {
    min_text_scale(sc, g.string_height("M"), MIN_TEXT_PX)
}

/// Text at scale `sc` but at least `min_px` logical px tall.
pub fn text_ex(g: &mut Gm, x: f64, y: f64, s: &str, sc: f64, min_px: f64) {
    let lh = g.string_height("M");
    let k = min_text_scale(sc, lh, min_px);
    g.text_transformed(x, y, s, k, k);
}

/// GameMaker's `c_white`.
pub const C_WHITE: f64 = 16777215.0;

/// `draw_text_ext_transformed(x, y, s, sep, w, xs, ys, 0)` with the minimum size (the wrap
/// width is in unscaled units, so it shrinks with the extra scale).
#[allow(clippy::too_many_arguments)]
pub fn text_ext(g: &mut Gm, x: f64, y: f64, s: &str, sep: f64, w: f64, xs: f64, ys: f64) {
    let lh = if sep >= 0.0 {
        sep
    } else {
        g.string_height("M")
    };
    let k = min_text_scale(ys, lh, MIN_TEXT_PX) / if ys == 0.0 { 1.0 } else { ys };
    let w = if w >= 0.0 { w / k } else { w };
    g.text_ext_transformed(x, y, s, sep, w, xs * k, ys * k);
}

/// `draw_triangle(.., false)` in the current colour and alpha.
#[allow(clippy::too_many_arguments)]
pub fn triangle(g: &mut Gm, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) {
    let (c, a) = (g.colour, g.alpha);
    g.tri([(x1, y1, c, a), (x2, y2, c, a), (x3, y3, c, a)]);
}

/// `TextMeasure` over the painter's current font (the logic's `goal_fit`).
pub struct GmMeasure<'a, 'b>(pub &'a Gm<'b>);

impl myco_core::progress::goals::TextMeasure for GmMeasure<'_, '_> {
    fn string_width(&self, s: &str) -> f64 {
        self.0.string_width(s)
    }
    fn string_height(&self, s: &str) -> f64 {
        self.0.string_height(s)
    }
    fn string_width_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
        self.0.string_width_ext(s, sep, w)
    }
    fn string_height_ext(&self, s: &str, sep: f64, w: f64) -> f64 {
        self.0.string_height_ext(s, sep, w)
    }
}

// port: hud_text
/// `draw_text_transformed(x, y, s, sc, sc, 0)`.
pub fn hud_text(g: &mut Gm, x: f64, y: f64, s: &str, sc: f64) {
    text(g, x, y, s, sc, sc);
}

/// `hud_fit` on the painter's current font (Task 18's `draw::hud_fit_size` is the marked port).
pub fn hud_fit(g: &Gm, s: &str, sc: f64, maxw: f64, maxh: f64) -> f64 {
    super::draw::hud_fit_size(g.string_width(s), g.string_height(s), sc, maxw, maxh)
}

// port: hud_icon_number
/// An icon and a number centred together at (cx, cy).
#[allow(clippy::too_many_arguments)]
pub fn hud_icon_number(
    g: &mut Gm,
    assets: &GameAssets,
    hud: &HudState,
    spr: &str,
    cx: f64,
    cy: f64,
    iscale: f64,
    text: &str,
    tscale: f64,
) {
    let s = assets.sprite(spr).size.as_dvec2();
    let (w, h) = (s.x * iscale, s.y * iscale);
    let tw = g.string_width(text) * eff_scale(g, tscale);
    let x = cx - (w + hud.ui.icon_gap + tw) * 0.5;
    g.sprite_ext(spr, 0, x, cy - h * 0.5, iscale, iscale, C_WHITE, 1.0);
    g.halign = 0;
    hud_text(g, x + w + hud.ui.icon_gap, cy, text, tscale);
}

/// The world → GUI offset of the drawn camera (`camera_view_x/y` at zoom 1).
pub fn view_xy(cams: &Cameras) -> (f64, f64) {
    let rv = cams.render_view();
    (rv.x, rv.y)
}

/// Draw_75, first part: tabs, bar, upgrade icon and button, length, food, water.
pub fn hud_draw_top(
    mut draw: ResMut<GuiDraw>,
    hud: Res<HudState>,
    sim: Res<SimRes>,
    assets: Res<GameAssets>,
    ui_fonts: Res<UiFonts>,
    loc: Res<LocaleRes>,
) {
    let mut g = Gm::new(&assets);
    draw_top(&mut g, &hud, &sim.0, &assets, &ui_fonts, &loc.locale);
    draw.lists.insert(draw_order::TOP, g.items);
}

pub fn draw_top(
    g: &mut Gm,
    hud: &HudState,
    sim: &Sim,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
) {
    g.halign = 0;
    g.valign = 0;
    g.font = uf.small.clone();
    if hud.hud_hidden {
        return;
    }
    let u = &hud.ui;
    let cm = mycelium_cm(sim);
    for (spr, p) in [
        ("UI_PIPKA", &u.wind_tab),
        ("UI_PIPKA", &u.upgrade_tab),
        ("UPPER_UI", &u.bar),
        ("UPGRADE", &u.upgrade_icon),
    ] {
        g.sprite_ext(spr, 0, p.x, p.y, p.scale, p.scale, C_WHITE, 1.0);
    }
    hud.draw_button(
        g,
        assets,
        "UPGRADE_CALM",
        "UPGRADE_HOVER",
        u.upgrade_button.x,
        u.upgrade_button.y,
        hud.upgrade_btn_hover,
        hud.upgrade_btn_press,
    );
    g.font = uf.font.clone();
    g.valign = 1;
    g.halign = 1;
    g.set_colour(COLOR_INK);
    let len_val = if hud.units_imperial { cm / 2.54 } else { cm };
    let len_str = fmt_num(gml::round(len_val));
    let unit_str = loc
        .loc(if hud.units_imperial {
            "hud_in"
        } else {
            "hud_cm"
        })
        .to_string();
    let len_w = g.string_width(&len_str) * eff_scale(g, u.length_scale);
    let unit_w = g.string_width(&unit_str) * eff_scale(g, u.unit_scale);
    let len_x = u.length_x - (len_w + u.unit_gap + unit_w) * 0.5;
    g.halign = 0;
    hud_text(g, len_x, u.text_y, &len_str, u.length_scale);
    g.set_colour(COLOR_DIM);
    hud_text(
        g,
        len_x + len_w + u.unit_gap,
        u.text_y + 4.0,
        &unit_str,
        u.unit_scale,
    );
    g.set_colour(COLOR_INK);
    let (fi, wi) = (&u.food_icon, &u.water_icon);
    let aw = assets.sprite("apple_bite").size.x as f64;
    let ww = assets.sprite("water65").size.x as f64;
    hud_icon_number(
        g,
        assets,
        hud,
        "apple_bite",
        fi.x + aw * fi.scale * 0.5,
        u.text_y,
        fi.scale,
        &fmt_num(sim.state.state.food),
        u.side_scale,
    );
    hud_icon_number(
        g,
        assets,
        hud,
        "water65",
        wi.x + ww * wi.scale * 0.5,
        u.text_y,
        wi.scale,
        &fmt_num(gml::round(myco_core::world::water_shown(sim))),
        u.side_scale,
    );
}

/// Draw_75, main part after the cave icons (Task 13 draws `cave_perk_icons_draw` in
/// `HudDrawSet::Caves`): tasks panel, cave popups (Task 13), animal markers, points, wind,
/// placement preview, build bar, build and camera buttons, cheat text, tutorial tips.
#[allow(clippy::too_many_arguments)]
pub fn hud_draw_main(
    mut draw: ResMut<GuiDraw>,
    hud: Res<HudState>,
    sim: Res<SimRes>,
    assets: Res<GameAssets>,
    ui_fonts: Res<UiFonts>,
    loc: Res<LocaleRes>,
    cams: Res<Cameras>,
    mut markers: ResMut<AnimalMarkers>,
    time: Res<Time>,
    anchor: Option<Res<crate::autopilot::MyceliumAnchor>>,
) {
    let anchor = anchor.map_or((640.0, 480.0), |a| (a.0.x, a.0.y));
    let mut g = Gm::new(&assets);
    draw_main(
        &mut g,
        &hud,
        &sim.0,
        &assets,
        &ui_fonts,
        &loc.locale,
        &cams,
        &mut markers,
        time.delta_secs_f64(),
        anchor,
    );
    draw.lists.insert(draw_order::HUD, g.items);
}

#[allow(clippy::too_many_arguments)]
pub fn draw_main(
    g: &mut Gm,
    hud: &HudState,
    sim: &Sim,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
    cams: &Cameras,
    markers: &mut AnimalMarkers,
    dt: f64,
    myc: (f64, f64),
) {
    let (gw, gh) = (cams.layout.view_w, cams.layout.view_h);
    let (vx, vy) = view_xy(cams);
    if !hud.hud_hidden {
        let u = &hud.ui;
        ui_tasks::tasks_draw(g, sim, hud, uf, loc, cams.layout.ui_ox, gw, assets);
        g.font = uf.font.clone();
        g.valign = 1;
        g.halign = 1;
        g.set_colour(COLOR_INK);
        // cave_popups_draw(vx, vy): Task 13
        if !sim.intro_active {
            let blocked = ui_blocking(sim, hud);
            markers::animal_markers_draw(
                g,
                markers,
                sim,
                assets,
                hud,
                blocked,
                cams.render_view(),
                gw,
                gh,
                dt,
            );
        }
        g.font = uf.font.clone();
        g.valign = 1;
        g.set_colour(COLOR_INK);
        g.halign = 0;
        g.set_colour(if sim.state.state.upgrade_points > 0 {
            COLOR_GOOD
        } else {
            COLOR_INK
        });
        hud_text(
            g,
            u.points_x,
            u.points_y,
            &sim.state.state.upgrade_points.to_string(),
            u.points_scale,
        );
        let [wind_dir, wind_speed, _] = sim.state.state.wind;
        let mut arrows = String::new();
        let mut i = 0.0;
        while i < wind_speed {
            arrows.push(if wind_dir < 0.0 { '<' } else { '>' });
            i += 1.0;
        }
        g.set_colour(COLOR_INK);
        g.halign = 1;
        hud_text(g, u.wind_x, u.wind_y, &arrows, u.wind_scale);
        g.font = uf.small.clone();
        g.halign = 0;
        g.valign = 0;
    }
    if !hud.hud_hidden && !sim.title_open && !sim.intro_active {
        draw_build(g, hud, sim, assets, uf, loc, cams);
        let b = &hud.build;
        hud.draw_button(
            g,
            assets,
            "SHROOM_CALM",
            "SHROOM_HOVER",
            b.circle_x,
            b.circle_y,
            hud.build_btn_hover,
            hud.build_btn_press,
        );
        hud.cam_draw(g, assets);
    }
    let p = &sim.progress;
    if p.cheat_timer > 0.0 {
        g.set_colour(COLOR_GOOD);
        g.set_alpha(gml::clamp(p.cheat_timer / 45.0, 0.0, 1.0));
        let t = p.cheat_text.clone();
        hud_text(g, 40.0, 204.0, &t, 1.4);
        g.set_alpha(1.0);
    }
    let box_ = hud.upgrades_button_rect(assets);
    ui_tasks::tutorial_draw(
        g, sim, assets, uf, loc, box_, COLOR_INK, gw, gh, vx, vy, myc,
    );
}

/// The placement preview and the build bar (Draw_75's build block).
fn draw_build(
    g: &mut Gm,
    hud: &HudState,
    sim: &Sim,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
    cams: &Cameras,
) {
    let b = &hud.build;
    // hud_place_at: the core's placement check of this step (ruling R22)
    let place_at = sim
        .mushrooms
        .place_preview
        .as_ref()
        .filter(|p| Some(p.kind) == hud.place_type)
        .map(|p| PlaceAt {
            ok: p.plan.ok,
            x: p.plan.x,
            y: p.plan.y,
            face: Some(p.plan.face),
        });
    if let (Some(t), Some(pa)) = (hud.place_type, place_at) {
        let spr = mushrooms::types::mushroom_type_sprite(t);
        if let Some(def) = assets.try_sprite(spr) {
            let sw = def.size.x as f64;
            let cw = mushrooms::types::sheet_cell_width(spr, sw);
            let col = if pa.ok {
                0x78e678 as f64
            } else {
                0x505aeb as f64
            };
            let variant = if t == MushroomKind::Consumptor {
                mushrooms::consumptor_variant(pa.face.unwrap_or(1.0)) as f64
            } else {
                0.0
            };
            let left = mushrooms::types::mushroom_cell_left(spr, variant);
            let p = cams.world_to_gui(bevy::math::DVec2::new(
                pa.x - def.origin.x as f64,
                pa.y - def.origin.y as f64,
            ));
            g.sprite_part_ext(
                spr,
                0,
                left,
                0.0,
                cw,
                def.size.y as f64,
                p.x,
                p.y,
                cams.zoom,
                cams.zoom,
                col,
                0.75,
            );
        }
    }
    if hud.build_slide <= 0.0 {
        return;
    }
    let e = 1.0 - (1.0 - hud.build_slide).powi(3);
    let off = -(b.bar_right - b.cx) * (1.0 - e);
    let mut types = HudState::build_types(sim);
    let bx = b.bar_x + off;
    let cut = (b.cx - bx).max(0.0);
    let bar = assets.sprite("LOWER_UI_BAR").size.as_dvec2();
    if cut < bar.x {
        let a = g.alpha;
        g.sprite_part_ext(
            "LOWER_UI_BAR",
            0,
            cut,
            0.0,
            bar.x - cut,
            bar.y,
            bx + cut,
            b.bar_y,
            1.0,
            1.0,
            C_WHITE,
            a,
        );
    }
    g.font = uf.font.clone();
    g.valign = 1;
    if hud.build_slide < 1.0 {
        types.clear();
    }
    if types.is_empty() && hud.build_slide >= 1.0 {
        g.halign = 0;
        g.set_colour(COLOR_DIM);
        hud_text(
            g,
            b.slot_x0 - 40.0 + off,
            b.slot_y,
            loc.loc("build_none"),
            0.38,
        );
    }
    let step = hud.build_slot_step(types.len());
    for (i, &t) in types.iter().enumerate() {
        let x = b.slot_x0 + i as f64 * step + off;
        let cost = mushrooms::mushroom_type_cost(&sim.state, t);
        let wcost = mushrooms::mushroom_type_water_cost(&sim.state, t);
        let affordable = mushrooms::mushroom_type_affordable(sim, t);
        let can_food = sim.state.state.food >= cost;
        let price_y = if wcost > 0.0 {
            b.slot_y - b.price_line * 0.5
        } else {
            b.slot_y
        };
        if hud.build_hover == i as i32 {
            let gcol = if affordable { COLOR_GOOD } else { COLOR_BAD };
            tree_draw_glow(g, x, b.slot_y, b.icon * 0.5, gcol, 0.35);
        }
        let isc = if t == MushroomKind::Consumptor {
            vis("build_consumptor_icon_scale")
        } else {
            1.0
        };
        let ialpha = if affordable { 1.0 } else { 0.45 };
        tree_draw_icon(
            g,
            Some(mushrooms::types::mushroom_type_sprite(t)),
            x,
            b.slot_y,
            b.icon * isc,
            ialpha,
            true,
            false,
        );
        let key = HudState::build_key_of(t);
        if key > 0 {
            let kr = vis("build_key_badge_radius");
            let ky = b.slot_y - b.icon * 0.5 - kr - vis("build_key_badge_gap");
            g.set_colour(0x121612 as f64);
            g.set_alpha(0.92);
            g.circle(x, ky, kr, false);
            g.set_alpha(1.0);
            g.set_colour(COLOR_INK);
            g.circle(x, ky, kr, true);
            g.font = uf.small.clone();
            g.halign = 1;
            g.valign = 1;
            hud_text(g, x, ky, &key.to_string(), vis("build_key_badge_text"));
            g.font = uf.font.clone();
            g.valign = 1;
        }
        let px = x + b.icon * 0.5 + b.price_gap;
        let apple = assets.sprite("apple_bite").size.as_dvec2();
        let bs = b.bite_size / apple.x.max(1.0);
        let falpha = if can_food { 1.0 } else { 0.5 };
        g.sprite_ext(
            "apple_bite",
            0,
            px,
            price_y - apple.y * bs * 0.5,
            bs,
            bs,
            C_WHITE,
            falpha,
        );
        g.halign = 0;
        g.set_colour(if can_food { COLOR_INK } else { COLOR_BAD });
        hud_text(
            g,
            px + b.bite_size + 4.0,
            price_y,
            &fmt_num(cost),
            b.price_scale,
        );
        if wcost > 0.0 {
            let can_w = sim.water_total() >= wcost;
            let wy = price_y + b.price_line;
            let wd = assets.sprite("water65");
            let (ww, wh) = (wd.size.x as f64, wd.size.y as f64);
            let ws = b.bite_size / ww.max(1.0);
            let walpha = if can_w { 1.0 } else { 0.5 };
            g.sprite_ext(
                "water65",
                0,
                px + b.bite_size * 0.5 + (wd.origin.x as f64 - ww * 0.5) * ws,
                wy + (wd.origin.y as f64 - wh * 0.5) * ws,
                ws,
                ws,
                C_WHITE,
                walpha,
            );
            g.set_colour(if can_w { COLOR_INK } else { COLOR_BAD });
            hud_text(
                g,
                px + b.bite_size + 4.0,
                wy,
                &fmt_num(wcost),
                b.price_scale,
            );
        }
    }
    g.font = uf.small.clone();
    g.halign = 0;
    g.valign = 0;
}

// port: gml_Object_obj_hud_Draw_75
/// Draw_75, tooltip and popup part (after the title screens): the wind / points / cave hover
/// tooltip, `cine_draw` ([`crate::cine::cine_draw`]) and the tasks popup. The debug overlay (`global.debug_open`)
/// cannot open (its F1 toggle is compiled out in the original), pause / logo / tree are drawn
/// by Tasks 18 / 19 in `HudDrawSet::Modal`. Together with [`draw_top`] and [`draw_main`] this
/// is the whole in-game Draw GUI.
#[allow(clippy::too_many_arguments)]
pub fn hud_draw_tooltip(
    mut draw: ResMut<GuiDraw>,
    hud: Res<HudState>,
    sim: Res<SimRes>,
    assets: Res<GameAssets>,
    ui_fonts: Res<UiFonts>,
    loc: Res<LocaleRes>,
    cams: Res<Cameras>,
) {
    let mut g = Gm::new(&assets);
    draw_tooltip(&mut g, &hud, &sim.0, &assets, &ui_fonts, &loc.locale, &cams);
    draw.lists.insert(draw_order::TOOLTIP, g.items);
}

pub fn draw_tooltip(
    g: &mut Gm,
    hud: &HudState,
    sim: &Sim,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
    cams: &Cameras,
) {
    let (gw, gh) = (cams.layout.view_w, cams.layout.view_h);
    let (mx, my) = hud.mouse_gui;
    let cm = mycelium_cm(sim);
    let step_cm = bal(sim.state.state.mode, "xp_length_step_cm");
    let xp_base = cm - sim.state.state.points_awarded as f64 * step_cm;
    let tip = if hud.wind_hover {
        loc.loc("wind_tooltip").to_string()
    } else if hud.points_hover {
        format!(
            "{}\n{} {}",
            loc.loc("points_tooltip"),
            length_text((step_cm - xp_base).ceil().max(0.0), hud.units_imperial, loc),
            loc.loc("points_till_next")
        )
    } else if !ui_blocking(sim, hud) && !hud.cave_tip.is_empty() {
        loc.loc(&hud.cave_tip).to_string()
    } else {
        String::new()
    };
    if !tip.is_empty() && !hud.hud_hidden {
        g.font = uf.small.clone();
        let tsc = hud_fit(g, &tip, 1.25, gw * 0.6, 0.0);
        let pad = 12.0;
        let k = eff_scale(g, tsc);
        let tw = g.string_width(&tip) * k + pad * 2.0;
        let th = g.string_height(&tip) * k + pad * 2.0;
        let tx = gml::clamp(mx + 20.0, 8.0, gw - tw - 8.0);
        let ty = gml::clamp(my + 24.0, 8.0, gh - th - 8.0);
        g.halign = 0;
        g.valign = 0;
        g.set_colour(COLOR_PANEL);
        g.set_alpha(0.95);
        g.rectangle(tx, ty, tx + tw, ty + th, false);
        g.set_alpha(1.0);
        g.set_colour(COLOR_DEEP);
        g.rectangle(tx, ty, tx + tw, ty + th, true);
        g.set_colour(COLOR_INK);
        hud_text(g, tx + pad, ty + pad, &tip, tsc);
    }
    crate::cine::cine_draw(g, sim, gw, gh);
    ui_tasks::tasks_popup_draw(
        g,
        sim,
        uf,
        loc,
        assets,
        COLOR_INK,
        COLOR_DIM,
        COLOR_PANEL,
        gw,
        gh,
        (mx, my),
    );
    g.set_alpha(1.0);
    g.set_colour(C_WHITE);
}
