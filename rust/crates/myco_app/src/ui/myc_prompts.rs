//! The GUI part of obj_mycelium_controller Draw_64 (Task 14): the aim preview while dragging a
//! shot and the in-world buttons (release spores, regenerate a repeller, the build menu and its
//! hint), plus the hit test that turns a click on a button into a `Command` (ruling R9b: the
//! rects measure text, so they are the app's).
//!
//! GUI coordinates are view-sized (`display_set_gui_size(view_w, view_h)`), so the original's
//! `(wx - camera_x) · sx` is `Cameras::world_to_gui`, which also follows the zoom; sizes stay in
//! GUI units (`ui` = 1).
//!
//! The buttons do not capture the mouse (`UiCapture`): the original's in-world prompts never
//! set `ui_capture_mouse`, and the logic drops button commands while the UI captures.

use crate::assets::GameAssets;
use crate::autopilot::run_script;
use crate::camera::Cameras;
use crate::sim_bridge::{PendingInput, SimRes, SimSet};
use crate::ui::gm_draw::{Gm, GuiDraw, FA_CENTER, FA_LEFT, FA_MIDDLE, FA_TOP};
use crate::ui::LocaleRes;
use bevy::math::DVec2;
use bevy::prelude::*;
use myco_core::gml;
use myco_core::mycelium::place::{build_option_label, can_build_option};
use myco_core::mycelium::soil::{gml_string, length_text, pixels_to_cm};
use myco_core::mycelium::BuildOption;
use myco_core::sim::{Command, Sim};

/// `GuiDraw` key of the mycelium's GUI drawing: Draw_64 of the controller comes before every
/// HUD layer (obj_hud Draw_75; Task 17's keys start at `draw_order::WORLD` = 100).
pub const MYC_DRAW_ORDER: i32 = 90;

/// A button (`___struct___466`): GUI rectangle and label.
#[derive(Clone, Debug, PartialEq)]
pub struct BtnRect {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub label: String,
}

impl BtnRect {
    /// `point_in_rectangle(gx, gy, x1, y1, x2, y2)` (edges included).
    pub fn contains(&self, gx: f64, gy: f64) -> bool {
        gx >= self.x1 && gx <= self.x2 && gy >= self.y1 && gy <= self.y2
    }
}

/// The geometry of `myc_button_rect`: a button at least 110 px wide (label + 18) and 28 high,
/// centred on GUI point (gx, gy), `offy` above it, kept 8 px inside the GUI.
pub fn button_rect(gx: f64, gy: f64, label_w: f64, offy: f64, gw: f64, gh: f64) -> BtnRect {
    let w = (label_w + 18.0).max(110.0);
    let h = 28.0;
    let x = gml::clamp(gx - w * 0.5, 8.0, gw - w - 8.0);
    let y = gml::clamp(gy + offy, 8.0, gh - h - 8.0);
    BtnRect {
        x1: x,
        y1: y,
        x2: x + w,
        y2: y + h,
        label: String::new(),
    }
}

/// What the rects need: the painter (font metrics), the cameras and the locale.
pub struct PromptCtx<'a, 'g> {
    pub gm: &'a Gm<'g>,
    pub cams: &'a Cameras,
    pub loc: &'a dyn Fn(&str) -> String,
}

impl PromptCtx<'_, '_> {
    fn gui_size(&self) -> (f64, f64) {
        (
            self.cams.layout.view_w.max(1.0),
            self.cams.layout.view_h.max(1.0),
        )
    }

    fn gui(&self, wx: f64, wy: f64) -> (f64, f64) {
        let g = self.cams.world_to_gui(DVec2::new(wx, wy));
        (g.x, g.y)
    }
}

// port: anon__6019__gml_Object_obj_mycelium_controller_Create_0
/// `myc_button_rect(wx, wy, label, offy)`: the button for world point (wx, wy).
pub fn myc_button_rect(ctx: &PromptCtx, wx: f64, wy: f64, label: String, offy: f64) -> BtnRect {
    let (gw, gh) = ctx.gui_size();
    let (gx, gy) = ctx.gui(wx, wy);
    let mut r = button_rect(gx, gy, ctx.gm.string_width(&label), offy, gw, gh);
    r.label = label;
    r
}

// port: anon__7134__gml_Object_obj_mycelium_controller_Create_0
/// `myc_spore_button_rect`: "release spores (cost food)" above the clicked spore mushroom.
pub fn spore_button_rect(sim: &Sim, ctx: &PromptCtx) -> BtnRect {
    let loc = ctx.loc;
    let label = format!(
        "{} ({} {})",
        loc("button_release_spores"),
        gml_string(sim.bal("spore_release_cost")),
        loc("hud_food")
    );
    let m = &sim.mycelium;
    myc_button_rect(ctx, m.spore_prompt_x, m.spore_prompt_y, label, -72.0)
}

// port: anon__7351__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_button_rect(i, opt)`: build-menu option `i`, stacked 34 px apart above the node.
pub fn build_button_rect(sim: &Sim, ctx: &PromptCtx, i: usize, opt: &BuildOption) -> BtnRect {
    let label = build_option_label(sim, opt).text(ctx.loc);
    let m = &sim.mycelium;
    myc_button_rect(
        ctx,
        m.build_node_x,
        m.build_node_y,
        label,
        -44.0 - i as f64 * 34.0,
    )
}

// port: anon__7521__gml_Object_obj_mycelium_controller_Create_0
/// `myc_regen_button_rect`: "regenerate (price food)" above the clicked repeller.
pub fn regen_button_rect(sim: &Sim, ctx: &PromptCtx) -> BtnRect {
    let loc = ctx.loc;
    let label = format!(
        "{} ({} {})",
        loc("button_regenerate"),
        gml_string(myco_core::mushrooms::repeller_regen_price(&sim.state)),
        loc("hud_food")
    );
    let m = &sim.mycelium;
    myc_button_rect(ctx, m.spore_prompt_x, m.spore_prompt_y, label, -72.0)
}

// port: anon__8636__gml_Object_obj_mycelium_controller_Create_0
/// `myc_build_hint_rect`: the build hint above the menu's options.
pub fn build_hint_rect(sim: &Sim, ctx: &PromptCtx) -> BtnRect {
    let m = &sim.mycelium;
    let label = (ctx.loc)(&m.build_hint);
    let n = m.build_options.len() as f64;
    myc_button_rect(ctx, m.build_node_x, m.build_node_y, label, n * -34.0 - 44.0)
}

// port: anon__7725__gml_Object_obj_mycelium_controller_Create_0
/// `myc_draw_prompt(r, hover, pressed)`: dark plate, light outline (brighter on hover, full when
/// hovered and `pressed` — the callers pass "affordable"), centred label.
pub fn draw_prompt(gm: &mut Gm, r: &BtnRect, hover: bool, pressed: bool) {
    let mut a = if hover { 0.78 } else { 0.44 };
    if pressed && hover {
        a = 1.0;
    }
    gm.set_colour(0.0);
    gm.set_alpha(0.78);
    gm.rectangle(r.x1, r.y1, r.x2, r.y2, false);
    gm.set_colour(0xeaf8f5 as f64);
    gm.set_alpha(a);
    gm.rectangle(r.x1, r.y1, r.x2, r.y2, true);
    gm.halign = FA_CENTER;
    gm.valign = FA_MIDDLE;
    gm.text_transformed((r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5, &r.label, 1.0, 1.0);
    gm.halign = FA_LEFT;
    gm.valign = FA_TOP;
}

/// Draw_64's aim preview: origin rings, the shaft and head of the arrow to the aimed length (red
/// when the shot can't be paid or is too short), a faint line to the reach, and the label
/// "length  -water  -food".
pub fn draw_aim(gm: &mut Gm, sim: &Sim, ctx: &PromptCtx, units_imperial: bool) {
    let m = &sim.mycelium;
    if !(m.dragging && m.has_aim) {
        return;
    }
    let col = super::super::render::mycelium::MYC_COL;
    let ui = 1.0;
    let dir = m.aim_dir;
    let (ox, oy) = (m.aim_origin_x, m.aim_origin_y);
    let (ax, ay) = ctx.gui(ox, oy);
    let (tx, ty) = ctx.gui(
        ox + gml::lengthdir_x(m.aim_len, dir),
        oy + gml::lengthdir_y(m.aim_len, dir),
    );
    let (bx, by) = ctx.gui(
        ox + gml::lengthdir_x(m.tile * 0.45, dir),
        oy + gml::lengthdir_y(m.tile * 0.45, dir),
    );
    let ah = 17.0 * ui;
    let aw = 10.0 * ui;
    let hx = tx + gml::lengthdir_x(ah, dir + 180.0);
    let hy = ty + gml::lengthdir_y(ah, dir + 180.0);
    let p1 = (
        hx + gml::lengthdir_x(aw, dir + 90.0),
        hy + gml::lengthdir_y(aw, dir + 90.0),
    );
    let p2 = (
        hx + gml::lengthdir_x(aw, dir - 90.0),
        hy + gml::lengthdir_y(aw, dir - 90.0),
    );
    let ok = sim.water_total() >= m.aim_cost
        && sim.state.state.food >= m.aim_food
        && m.aim_len >= m.min_growth_len;
    let acol = if ok { col } else { 0x6876d6 as f64 };
    let line = |gm: &mut Gm, x1, y1, x2, y2, w, c| {
        gm.set_colour(c);
        gm.line_width(x1, y1, x2, y2, w);
    };
    let tri = |gm: &mut Gm, c: f64| {
        let a = gm.alpha;
        gm.tri([(tx, ty, c, a), (p1.0, p1.1, c, a), (p2.0, p2.1, c, a)]);
    };
    if m.aim_reach > m.aim_len + 1.0 {
        let (rx, ry) = ctx.gui(
            ox + gml::lengthdir_x(m.aim_reach, dir),
            oy + gml::lengthdir_y(m.aim_reach, dir),
        );
        let tkx = gml::lengthdir_x(6.0 * ui, dir + 90.0);
        let tky = gml::lengthdir_y(6.0 * ui, dir + 90.0);
        gm.set_alpha(0.2);
        line(gm, tx, ty, rx, ry, 2.0 * ui, col);
        line(gm, rx - tkx, ry - tky, rx + tkx, ry + tky, 3.0 * ui, col);
    }
    gm.set_colour(acol);
    gm.set_alpha(0.18);
    gm.circle(ax, ay, 13.0 * ui, false);
    line(gm, bx, by, hx, hy, 13.0 * ui, acol);
    tri(gm, acol);
    gm.set_alpha(0.62);
    line(gm, bx, by, hx, hy, 6.0 * ui, acol);
    gm.circle(ax, ay, 5.0 * ui, false);
    gm.set_alpha(1.0);
    line(gm, bx, by, hx, hy, 3.0 * ui, acol);
    tri(gm, acol);
    gm.circle(ax, ay, 2.5 * ui, false);
    gm.halign = FA_CENTER;
    gm.valign = FA_MIDDLE;
    let loc = ctx.loc;
    let mut label = format!(
        "{}  -{} {}",
        length_text(units_imperial, pixels_to_cm(sim, m.aim_len), loc),
        gml_string(gml::round(m.aim_cost)),
        loc("hud_water")
    );
    if m.aim_food > 0.0 {
        label.push_str(&format!(
            "  -{} {}",
            gml_string(m.aim_food),
            loc("hud_food")
        ));
    }
    gm.text_transformed(tx, ty - 26.0 * ui, &label, 1.0, 1.0);
    gm.halign = FA_LEFT;
    gm.valign = FA_TOP;
}

/// Draw_64's buttons: spore / regen prompt and the build menu with its hint. `mouse` is the GUI
/// cursor for the hover look.
///
/// `build_ok[i]` is `myc_can_build_option(option i)`, evaluated after the last fixed step
/// ([`BuildAffordable`]): the check may generate surface chunks, so it stays out of the draw.
pub fn draw_buttons(
    gm: &mut Gm,
    sim: &Sim,
    build_ok: &[bool],
    cams: &Cameras,
    loc: &dyn Fn(&str) -> String,
    mouse: (f64, f64),
) {
    let m = &sim.mycelium;
    let (spore, regen, menu) = (m.spore_prompt, m.regen_prompt, m.mushroom_prompt);
    let mut rects: Vec<(BtnRect, bool, bool)> = Vec::new();
    {
        let ctx = PromptCtx {
            gm: &*gm,
            cams,
            loc,
        };
        if spore {
            let r = spore_button_rect(sim, &ctx);
            let hover = r.contains(mouse.0, mouse.1);
            let ok = sim.state.state.food >= sim.bal("spore_release_cost");
            rects.push((r, hover, ok));
        }
        if regen {
            let r = regen_button_rect(sim, &ctx);
            let hover = r.contains(mouse.0, mouse.1);
            let ok = sim.state.state.food >= myco_core::mushrooms::repeller_regen_price(&sim.state);
            rects.push((r, hover, ok));
        }
    }
    if menu {
        let opts = sim.mycelium.build_options.clone();
        let ctx = PromptCtx {
            gm: &*gm,
            cams,
            loc,
        };
        let opt_rects: Vec<BtnRect> = opts
            .iter()
            .enumerate()
            .map(|(i, o)| build_button_rect(sim, &ctx, i, o))
            .collect();
        let hint = (!sim.mycelium.build_hint.is_empty()).then(|| build_hint_rect(sim, &ctx));
        // myc_build_hover: the first option under the cursor
        let hover_i = opt_rects.iter().position(|r| r.contains(mouse.0, mouse.1));
        for (i, r) in opt_rects.into_iter().enumerate() {
            let ok = build_ok.get(i).copied().unwrap_or(false);
            rects.push((r, hover_i == Some(i), ok));
        }
        if let Some(h) = hint {
            rects.push((h, false, false));
        }
    }
    for (r, hover, ok) in &rects {
        draw_prompt(gm, r, *hover, *ok);
    }
    gm.set_alpha(1.0);
    gm.set_colour(0xffffff as f64);
}

/// The command a left click at GUI point (gx, gy) gives, if it lands on a button (the original's
/// hover tests in Step_0: build options first, then the spore and regen prompts).
pub fn button_click(sim: &Sim, ctx: &PromptCtx, gx: f64, gy: f64) -> Option<Command> {
    let m = &sim.mycelium;
    if m.mushroom_prompt {
        for (i, o) in m.build_options.iter().enumerate() {
            if build_button_rect(sim, ctx, i, o).contains(gx, gy) {
                return Some(Command::MycBuildOption { index: i });
            }
        }
    }
    if m.spore_prompt && spore_button_rect(sim, ctx).contains(gx, gy) {
        return Some(Command::MycReleaseSpores);
    }
    if m.regen_prompt && regen_button_rect(sim, ctx).contains(gx, gy) {
        return Some(Command::MycRegenRepeller);
    }
    None
}

fn gui_mouse(pending: &PendingInput, cams: &Cameras) -> Option<(f64, f64)> {
    pending
        .cursor_override
        .or(pending.cursor)
        .map(|p| cams.screen_to_gui(p))
        .map(|g| (g.x, g.y))
}

fn loc_fn<'a>(locale: &'a Option<Res<'_, LocaleRes>>) -> impl Fn(&str) -> String + 'a {
    move |k: &str| match locale {
        Some(l) => l.locale.loc(k).to_string(),
        None => k.to_string(),
    }
}

fn painter<'a>(assets: &'a GameAssets, locale: &Option<Res<LocaleRes>>) -> Gm<'a> {
    let mut gm = Gm::new(assets);
    if let Some(l) = locale {
        gm.font = l.font_small.clone();
    }
    gm
}

/// Before each step (after the autopilot): a fresh LMB press on a button becomes its command.
fn click_system(
    sim: Res<SimRes>,
    cams: Res<Cameras>,
    assets: Res<GameAssets>,
    locale: Option<Res<LocaleRes>>,
    mut pending: ResMut<PendingInput>,
) {
    if !pending.lmb.pressed {
        return;
    }
    let Some((gx, gy)) = gui_mouse(&pending, &cams) else {
        return;
    };
    let loc = loc_fn(&locale);
    let gm = painter(&assets, &locale);
    let ctx = PromptCtx {
        gm: &gm,
        cams: &cams,
        loc: &loc,
    };
    if let Some(c) = button_click(&sim.0, &ctx, gx, gy) {
        pending.commands.push(c);
    }
}

/// Every frame: the aim preview and the buttons into [`GuiDraw`].
/// `myc_can_build_option` for each open build-menu option, refreshed after every fixed step.
#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub struct BuildAffordable(pub Vec<bool>);

fn build_affordable_system(mut sim: ResMut<SimRes>, mut ok: ResMut<BuildAffordable>) {
    let sim = &mut sim.0;
    let v: Vec<bool> = if sim.mycelium.mushroom_prompt {
        let opts = sim.mycelium.build_options.clone();
        opts.iter().map(|o| can_build_option(sim, o)).collect()
    } else {
        Vec::new()
    };
    if ok.0 != v {
        ok.0 = v;
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_system(
    sim: Res<SimRes>,
    build_ok: Res<BuildAffordable>,
    cams: Res<Cameras>,
    assets: Res<GameAssets>,
    locale: Option<Res<LocaleRes>>,
    pending: Res<PendingInput>,
    flags: Option<Res<crate::ui::menu::UiFlags>>,
    mut draw: ResMut<GuiDraw>,
) {
    let loc = loc_fn(&locale);
    let mut gm = painter(&assets, &locale);
    let mouse = gui_mouse(&pending, &cams).unwrap_or((-1.0, -1.0));
    {
        let measure = painter(&assets, &locale);
        let ctx = PromptCtx {
            gm: &measure,
            cams: &cams,
            loc: &loc,
        };
        let imperial = flags.is_some_and(|f| f.units_imperial);
        draw_aim(&mut gm, &sim.0, &ctx, imperial);
    }
    draw_buttons(&mut gm, &sim.0, &build_ok.0, &cams, &loc, mouse);
    draw.lists.insert(MYC_DRAW_ORDER, gm.items);
}

pub struct MycPromptsPlugin;

impl Plugin for MycPromptsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuiDraw>()
            .init_resource::<BuildAffordable>()
            .add_systems(FixedUpdate, build_affordable_system.in_set(SimSet::After))
            .add_systems(
                FixedUpdate,
                click_system.in_set(SimSet::Prepare).after(run_script),
            )
            .add_systems(Update, draw_system);
    }
}
