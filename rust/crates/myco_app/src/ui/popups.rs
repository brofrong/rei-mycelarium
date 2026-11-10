//! Floating resource numbers (the owner's addition): `+N` in green / `-N` in red with the
//! resource's icon (`apple_bite`, `water65`, `UPGRADE`) rise 60 px over 1 s and fade out.
//!
//! - One number per source per simulation step: the step's `SimEvent::Resource` events are
//!   summed by (source, kind).
//! - A source outside the drawn camera view makes no number.
//! - At most 64 at once: a new one then adds to the nearest live number of the same resource.
//! - Numbers live in world coordinates and scale with the zoom, but stay at least 12 px tall
//!   on screen.
//! - A number born where another one still young is (planting a mushroom spends food and water
//!   at one spot) takes the next line up, so both are readable.
//! - Food icons are red-burgundy, water icons blue (`hud::icon_tint`).

use super::gm_draw::{Gm, GuiDraw};
use super::hud::{icon_tint, text_ex, HudState, UiFonts, COLOR_BAD, COLOR_GOOD};
use super::{draw_order, min_text_scale};
use crate::assets::GameAssets;
use crate::camera::Cameras;
use crate::sim_bridge::{SimEventMsg, SimRes};
use bevy::math::DVec2;
use bevy::prelude::*;
use myco_core::events::{ResourceKind, SimEvent};
use myco_core::sim::ViewRect;

/// Rise over the life, world px.
pub const RISE: f64 = 60.0;
/// Life, seconds.
pub const LIFE: f64 = 1.0;
/// Live numbers at most.
pub const CAP: usize = 64;
/// Minimum text height on screen, logical px.
pub const MIN_PX: f64 = 12.0;
/// Text line height at zoom 1, world px.
pub const LINE_PX: f64 = 30.0;
/// A new number within this distance (world px) of a live one younger than [`STACK_AGE`]
/// goes on the next free line above it.
pub const STACK_PX: f64 = 24.0;
/// Seconds a number keeps its line taken for numbers born at its spot.
pub const STACK_AGE: f64 = 0.5;

#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub kind: ResourceKind,
    pub value: f64,
    /// world position where it appeared
    pub x: f64,
    pub y: f64,
    /// seconds
    pub age: f64,
    /// line above the spot (0: on it), so numbers born at one spot do not cover each other
    pub row: u32,
}

impl Popup {
    /// Upward offset (world px) at this age.
    pub fn rise(&self) -> f64 {
        RISE * (self.age / LIFE).clamp(0.0, 1.0)
    }

    /// Opacity at this age.
    pub fn alpha(&self) -> f64 {
        (1.0 - self.age / LIFE).clamp(0.0, 1.0)
    }

    /// "+12" / "-3" ("+0.5" for fractions below one).
    pub fn label(&self) -> String {
        let v = self.value;
        let sign = if v < 0.0 { "-" } else { "+" };
        let a = v.abs();
        let r = myco_core::gml::round(a);
        if r == 0.0 && a > 0.0 {
            format!("{sign}{:.1}", a)
        } else {
            format!("{sign}{}", r as i64)
        }
    }
}

/// The live numbers.
#[derive(Resource, Default, Clone, Debug)]
pub struct Popups {
    pub list: Vec<Popup>,
}

fn on_view(v: &ViewRect, x: f64, y: f64) -> bool {
    x >= v.x && x <= v.x + v.w && y >= v.y && y <= v.y + v.h
}

impl Popups {
    /// Add one simulation step's events (`view`: the drawn world view).
    pub fn add_step<'a>(
        &mut self,
        events: impl IntoIterator<Item = &'a SimEvent>,
        view: &ViewRect,
    ) {
        // (source, kind) → summed delta at the first event's position, in event order
        let mut step: Vec<(u64, ResourceKind, f64, f64, f64)> = Vec::new();
        for e in events {
            if let SimEvent::Resource {
                kind,
                delta,
                x,
                y,
                source,
            } = *e
            {
                match step.iter_mut().find(|s| s.0 == source && s.1 == kind) {
                    Some(s) => s.2 += delta,
                    None => step.push((source, kind, delta, x, y)),
                }
            }
        }
        for (_, kind, delta, x, y) in step {
            if delta == 0.0 || !on_view(view, x, y) {
                continue;
            }
            if self.list.len() < CAP {
                let taken: Vec<u32> = self
                    .list
                    .iter()
                    .filter(|p| {
                        p.age < STACK_AGE && (p.x - x).powi(2) + (p.y - y).powi(2) < STACK_PX.powi(2)
                    })
                    .map(|p| p.row)
                    .collect();
                let row = (0..).find(|r| !taken.contains(r)).unwrap_or(0);
                self.list.push(Popup {
                    kind,
                    value: delta,
                    x,
                    y,
                    age: 0.0,
                    row,
                });
                continue;
            }
            let nearest = self
                .list
                .iter_mut()
                .filter(|p| p.kind == kind)
                .min_by(|a, b| {
                    let da = (a.x - x).powi(2) + (a.y - y).powi(2);
                    let db = (b.x - x).powi(2) + (b.y - y).powi(2);
                    da.total_cmp(&db)
                });
            if let Some(p) = nearest {
                p.value += delta;
            }
        }
    }

    /// Age the numbers; drop the finished ones.
    pub fn tick(&mut self, dt: f64) {
        for p in &mut self.list {
            p.age += dt;
        }
        self.list.retain(|p| p.age < LIFE);
    }
}

/// After each step: this step's resource events.
pub fn popups_collect(
    mut popups: ResMut<Popups>,
    mut ev: MessageReader<SimEventMsg>,
    cams: Res<Cameras>,
) {
    let events: Vec<SimEvent> = ev.read().map(|e| e.0.clone()).collect();
    popups.add_step(events.iter(), &cams.render_view());
}

pub fn popups_tick(mut popups: ResMut<Popups>, time: Res<Time>) {
    popups.tick(time.delta_secs_f64());
}

fn icon_of(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::Food => "apple_bite",
        ResourceKind::Water => "water65",
        ResourceKind::UpgradePoints => "UPGRADE",
    }
}

/// Draw the numbers (world → GUI through the zoomed camera).
pub fn popups_draw(g: &mut Gm, popups: &Popups, assets: &GameAssets, uf: &UiFonts, cams: &Cameras) {
    g.font = uf.font.clone();
    g.halign = 0;
    g.valign = 1;
    let lh = g.string_height("M").max(1.0);
    for p in &popups.list {
        let a = p.alpha();
        if a <= 0.0 {
            continue;
        }
        let mut c = cams.world_to_gui(DVec2::new(p.x, p.y - p.rise()));
        let sc = min_text_scale(LINE_PX / lh * cams.zoom, lh, MIN_PX);
        c.y -= p.row as f64 * lh * sc * 1.05;
        let label = p.label();
        let tw = g.string_width(&label) * sc;
        let ih = lh * sc * 0.8;
        let icon = icon_of(p.kind);
        let (iw, isc) = assets.try_sprite(icon).map_or((0.0, 0.0), |d| {
            let k = ih / (d.size.y as f64).max(1.0);
            (d.size.x as f64 * k, k)
        });
        let gap = lh * sc * 0.12;
        let x0 = c.x - (tw + gap + iw) * 0.5;
        let col = if p.value < 0.0 { COLOR_BAD } else { COLOR_GOOD };
        g.set_colour(0.0);
        g.set_alpha(a * 0.6);
        text_ex(g, x0 + 2.0, c.y + 2.0, &label, sc, MIN_PX);
        g.set_colour(col);
        g.set_alpha(a);
        text_ex(g, x0, c.y, &label, sc, MIN_PX);
        if let Some(d) = assets.try_sprite(icon) {
            let ox = d.origin.x as f64 * isc;
            let oy = d.origin.y as f64 * isc;
            g.sprite_ext(
                icon,
                0,
                x0 + tw + gap + ox,
                c.y - ih * 0.5 + oy,
                isc,
                isc,
                icon_tint(icon),
                a,
            );
        }
    }
    g.set_alpha(1.0);
}

/// Draw_75's world-anchored layer: the floating numbers and the hover bars.
#[allow(clippy::too_many_arguments)]
pub fn world_draw(
    mut draw: ResMut<GuiDraw>,
    popups: Res<Popups>,
    assets: Res<GameAssets>,
    uf: Res<UiFonts>,
    cams: Res<Cameras>,
    sim: Res<SimRes>,
    hud: Res<HudState>,
) {
    let mut g = Gm::new(&assets);
    super::markers::hover_bar_step_draw(&mut g, &sim.0, &hud, &cams);
    popups_draw(&mut g, &popups, &assets, &uf, &cams);
    draw.lists.insert(draw_order::WORLD, g.items);
}
