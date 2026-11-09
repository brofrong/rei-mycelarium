//! The red tip at the cursor when a right-click node is refused (the owner's addition): the
//! reason from `SimEvent::NodeRejected`, translated, for 1.2 s (fading in the last 0.3 s).

use super::gm_draw::{Gm, GuiDraw};
use super::hud::{text_ex, HudState, UiFonts, COLOR_BAD};
use super::{draw_order, LocaleRes, MIN_TEXT_PX};
use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::sim_bridge::{PendingInput, SimEventMsg};
use bevy::prelude::*;
use myco_core::events::{NodeReject, SimEvent};
use myco_core::gml;
use myco_core::locale::Locale;

/// How long a tip stays, seconds.
pub const TIP_LIFE: f64 = 1.2;
/// Fade-out at the end, seconds.
pub const TIP_FADE: f64 = 0.3;

/// The current tip.
#[derive(Resource, Default, Clone, Debug)]
pub struct CursorTip {
    pub reason: Option<NodeReject>,
    pub age: f64,
}

/// The locale key of a refusal.
pub fn tip_key(r: &NodeReject) -> &'static str {
    match r {
        NodeReject::NotEnoughFood { .. } => "tip_node_need_food",
        NodeReject::NotOnMycelium => "tip_node_not_on_mycelium",
        NodeReject::InWater => "hint_no_nodes_in_water",
        NodeReject::TooCloseToNode => "tip_node_too_close",
        NodeReject::Blocked => "tip_node_blocked",
    }
}

/// The translated tip ("NEED 12 FOOD").
pub fn tip_text(loc: &Locale, r: &NodeReject) -> String {
    let s = loc.loc(tip_key(r)).to_string();
    match r {
        NodeReject::NotEnoughFood { need } => s.replace("{n}", &need.to_string()),
        _ => s,
    }
}

impl CursorTip {
    /// The step's last refusal replaces the tip.
    pub fn add_step<'a>(&mut self, events: impl IntoIterator<Item = &'a SimEvent>) {
        for e in events {
            if let SimEvent::NodeRejected { reason, .. } = e {
                self.reason = Some(reason.clone());
                self.age = 0.0;
            }
        }
    }

    pub fn tick(&mut self, dt: f64) {
        if self.reason.is_some() {
            self.age += dt;
            if self.age >= TIP_LIFE {
                self.reason = None;
            }
        }
    }

    pub fn alpha(&self) -> f64 {
        if self.reason.is_none() {
            return 0.0;
        }
        gml::clamp((TIP_LIFE - self.age) / TIP_FADE, 0.0, 1.0)
    }
}

pub fn cursor_tip_collect(mut tip: ResMut<CursorTip>, mut ev: MessageReader<SimEventMsg>) {
    let events: Vec<SimEvent> = ev.read().map(|e| e.0.clone()).collect();
    tip.add_step(events.iter());
}

pub fn cursor_tip_tick(mut tip: ResMut<CursorTip>, time: Res<Time>) {
    tip.tick(time.delta_secs_f64());
}

/// Draw the tip right below the cursor, kept on screen.
pub fn draw_tip(
    g: &mut Gm,
    tip: &CursorTip,
    loc: &Locale,
    uf: &UiFonts,
    mouse: (f64, f64),
    gw: f64,
    gh: f64,
) {
    let Some(r) = &tip.reason else {
        return;
    };
    let a = tip.alpha();
    if a <= 0.0 {
        return;
    }
    let s = tip_text(loc, r);
    g.font = uf.small.clone();
    g.halign = 0;
    g.valign = 0;
    let lh = g.string_height("M").max(1.0);
    let sc = super::min_text_scale(1.0, lh, MIN_TEXT_PX);
    let (tw, th) = (g.string_width(&s) * sc, lh * sc);
    let x = gml::clamp(mouse.0 + 18.0, 8.0, (gw - tw - 8.0).max(8.0));
    let y = gml::clamp(mouse.1 + 22.0, 8.0, (gh - th - 8.0).max(8.0));
    g.set_colour(0.0);
    g.set_alpha(a * 0.6);
    text_ex(g, x + 2.0, y + 2.0, &s, sc, MIN_TEXT_PX);
    g.set_colour(COLOR_BAD);
    g.set_alpha(a);
    text_ex(g, x, y, &s, sc, MIN_TEXT_PX);
    g.set_alpha(1.0);
}

#[allow(clippy::too_many_arguments)]
pub fn cursor_tip_draw(
    mut draw: ResMut<GuiDraw>,
    tip: Res<CursorTip>,
    loc: Res<LocaleRes>,
    uf: Res<UiFonts>,
    assets: Res<GameAssets>,
    cams: Res<Cameras>,
    pending: Res<PendingInput>,
    hud: Res<HudState>,
) {
    let mut g = Gm::new(&assets);
    let mouse = pending
        .screen_cursor()
        .map(|p| cams.screen_to_gui(p).into())
        .unwrap_or(hud.mouse_gui);
    draw_tip(
        &mut g,
        &tip,
        &loc.locale,
        &uf,
        mouse,
        cams.layout.view_w,
        cams.layout.view_h,
    );
    draw.lists.insert(draw_order::CURSOR, g.items);
}
