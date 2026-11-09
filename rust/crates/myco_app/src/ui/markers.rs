//! Off-screen animal markers (`animal_marker_*`, scr_life) and the progress bars shown when
//! hovering a working mushroom, an absorbing leaf or a caught worm (`hover_bar_*`).
//!
//! The animals, leaves and worms come from the logic through hooks Task 10 fills
//! (`life::animal_views`, `life::absorbing_leaves`, `life::caught_worms`) and the mushroom
//! progress / hit boxes through Task 9's (`mushrooms::hover_progress`, `mushrooms::mushroom_hit_box`).
//! The hover bars belong to obj_life's world Draw (after the crows); here they are drawn on the
//! GUI layer at the world position (zoom applied), under the HUD.

use super::gm_draw::Gm;
use super::hud::{triangle, C_WHITE};
use super::hud::{ui_blocking, HudState};
use crate::assets::GameAssets;
use crate::camera::Cameras;
use bevy::math::DVec2;
use bevy::prelude::*;
use myco_core::balance::vis;
use myco_core::gml;
use myco_core::life::{self, AnimalKind, AnimalView};
use myco_core::mushrooms;
use myco_core::sim::Sim;

/// One marker (`___struct___108..112` plus the update's fields).
#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub src: u64,
    pub sprite: &'static str,
    pub frames: f64,
    pub frame: f64,
    pub mirror: f64,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub warn: bool,
    pub alpha: f64,
    pub vx: f64,
    pub vy: f64,
    pub gone: bool,
    pub seen: bool,
    pub follow: f64,
    /// where the marker was drawn (GUI), set by the draw
    pub ui: Option<(f64, f64)>,
}

/// `global.animal_markers`.
#[derive(Resource, Default, Clone, Debug)]
pub struct AnimalMarkers {
    pub list: Vec<Marker>,
}

fn sprite_size(assets: &GameAssets, name: &str) -> (f64, f64) {
    assets
        .try_sprite(name)
        .map_or((0.0, 0.0), |d| (d.size.x as f64, d.size.y as f64))
}

// port: animal_marker_list
/// The live animals as markers: hedgehogs and boars centred on their bodies (mirror against
/// the walking direction), squirrels at their position, crows and pending crows (warnings).
pub fn animal_marker_list(views: &[AnimalView], assets: &GameAssets, now_ms: f64) -> Vec<Marker> {
    let mk =
        |v: &AnimalView, sprite: &'static str, frames: f64, frame, mirror, x, y, w, h, warn| {
            Marker {
                src: v.src,
                sprite,
                frames,
                frame,
                mirror,
                x,
                y,
                w,
                h,
                warn,
                alpha: 0.0,
                vx: 0.0,
                vy: 0.0,
                gone: false,
                seen: false,
                follow: 0.0,
                ui: None,
            }
        };
    let mut out = Vec::new();
    for kind in [
        AnimalKind::Hedgehog,
        AnimalKind::Boar,
        AnimalKind::Squirrel,
        AnimalKind::Crow,
        AnimalKind::CrowPending,
    ] {
        for v in views.iter().filter(|v| v.kind == kind) {
            let dir_mirror = if v.dir >= 0.0 { -1.0 } else { 1.0 };
            out.push(match kind {
                AnimalKind::Hedgehog => {
                    let (sw, sh) = sprite_size(assets, "hedgehog");
                    let sc = vis("hedgehog_scale");
                    let h = sh * sc;
                    mk(
                        v,
                        "hedgehog",
                        8.0,
                        v.frame,
                        dir_mirror,
                        v.x,
                        v.y - h * 0.5,
                        sw / 8.0 * sc,
                        h,
                        false,
                    )
                }
                AnimalKind::Boar => {
                    let (sw, sh) = sprite_size(assets, "boar");
                    let sc = vis("boar_scale");
                    let h = sh * sc;
                    mk(
                        v,
                        "boar",
                        6.0,
                        v.frame,
                        dir_mirror,
                        v.x,
                        v.y - h * 0.5,
                        sw / 6.0 * sc,
                        h,
                        false,
                    )
                }
                AnimalKind::Squirrel => {
                    let (sw, sh) = sprite_size(assets, "squirrel");
                    let sc = vis("squirrel_scale");
                    mk(
                        v,
                        "squirrel",
                        9.0,
                        v.frame,
                        v.mirror,
                        v.x,
                        v.y,
                        sw / 9.0 * sc,
                        sh * sc,
                        false,
                    )
                }
                AnimalKind::Crow | AnimalKind::CrowPending => {
                    let (sw, sh) = sprite_size(assets, "crow");
                    let sc = vis("crow_scale");
                    let h = sh * sc;
                    let pending = kind == AnimalKind::CrowPending;
                    let frame = if pending {
                        gml::gmod(now_ms / 120.0, 9.0)
                    } else {
                        v.frame
                    };
                    mk(
                        v,
                        "crow",
                        9.0,
                        frame,
                        -v.dir,
                        v.x,
                        v.y - h * 0.5,
                        (sw / 9.0).trunc() * sc,
                        h,
                        pending,
                    )
                }
            });
        }
    }
    out
}

// port: animal_markers_update
/// Match the live animals to the markers by source: new ones start transparent, live ones
/// follow (not while the UI blocks); vanished ones keep drifting for
/// `animal_marker_follow_seconds` (warnings stop at once); alpha eases to 1 while the animal is
/// off the view's edge... and to 0 when on screen or gone.
#[allow(clippy::too_many_arguments)]
pub fn animal_markers_update(
    m: &mut AnimalMarkers,
    cur: Vec<Marker>,
    blocked: bool,
    dt: f64,
    vx: f64,
    vy: f64,
    vw: f64,
    vh: f64,
) {
    let fade = vis("animal_marker_fade_seconds").max(0.01);
    for o in &mut m.list {
        o.seen = false;
    }
    for c in cur {
        if let Some(o) = m.list.iter_mut().find(|o| o.src == c.src) {
            // a warned crow that arrived keeps the warning's id (crow_due rebinds the marker
            // to the crow and clears `warn`)
            if o.warn && !c.warn {
                o.warn = false;
            }
            if !blocked {
                o.vx = c.x - o.x;
                o.vy = c.y - o.y;
                o.x = c.x;
                o.y = c.y;
                o.frame = c.frame;
                o.mirror = c.mirror;
            }
            o.seen = true;
        } else {
            let mut n = c;
            n.alpha = 0.0;
            n.vx = 0.0;
            n.vy = 0.0;
            n.gone = false;
            n.seen = true;
            m.list.push(n);
        }
    }
    let mut keep = Vec::new();
    for mut o in std::mem::take(&mut m.list) {
        if !o.seen && !o.gone {
            o.gone = true;
            o.follow = if o.warn {
                0.0
            } else {
                vis("animal_marker_follow_seconds")
            };
            if o.warn {
                o.vx = 0.0;
                o.vy = 0.0;
            }
        }
        if o.gone && !blocked {
            o.x += o.vx;
            o.y += o.vy;
            o.frame = gml::gmod(o.frame + 0.25, o.frames);
            o.follow -= dt;
        }
        let (sx, sy) = (o.x - vx, o.y - vy);
        let on = sx + o.w * 0.5 > 0.0
            && sx - o.w * 0.5 < vw
            && sy + o.h * 0.5 > 0.0
            && sy - o.h * 0.5 < vh;
        let target = if !on || (o.gone && o.follow <= 0.0) {
            0.0
        } else {
            1.0
        };
        o.alpha = if target > o.alpha {
            target.min(o.alpha + dt / fade)
        } else {
            target.max(o.alpha - dt / fade)
        };
        if !o.gone || o.alpha > 0.0 || o.follow > 0.0 {
            keep.push(o);
        }
    }
    m.list = keep;
}

// port: animal_markers_draw
/// Update, then draw each marker on the ray from the view centre to the animal, clamped to the
/// edges: a pointer triangle, a dark disc with the animal's current frame, rings (doubled when
/// hovered). `rv` is the drawn world view; the GUI is `vw`×`vh` (zoom = vw / rv.w; 1 in the
/// original).
#[allow(clippy::too_many_arguments)]
pub fn animal_markers_draw(
    g: &mut Gm,
    m: &mut AnimalMarkers,
    sim: &Sim,
    assets: &GameAssets,
    hud: &HudState,
    blocked: bool,
    rv: myco_core::sim::ViewRect,
    vw: f64,
    vh: f64,
    dt: f64,
) {
    let now_ms = sim.frame as f64 * 1000.0 / 60.0;
    let cur = animal_marker_list(&life::animal_views(sim), assets, now_ms);
    animal_markers_update(m, cur, blocked, dt, rv.x, rv.y, rv.w, rv.h);
    let (vx, vy) = (rv.x, rv.y);
    let z = if rv.w > 0.0 { vw / rv.w } else { 1.0 };
    let r = vis("animal_marker_radius");
    let edge = r + vis("animal_marker_edge_px");
    let (hw, hh) = (vw * 0.5, vh * 0.5);
    let col = 14873582.0;
    let bg = 1185298.0;
    for o in &mut m.list {
        let a = o.alpha;
        if a <= 0.0 {
            continue;
        }
        let (sx, sy) = ((o.x - vx) * z, (o.y - vy) * z);
        let (dx, dy) = (sx - hw, sy - hh);
        let tx = if dx > 0.0 {
            (vw - edge - hw) / dx
        } else if dx < 0.0 {
            (edge - hw) / dx
        } else {
            f64::INFINITY
        };
        let ty = if dy > 0.0 {
            (vh - vis("animal_marker_bottom_px") - hh) / dy
        } else if dy < 0.0 {
            (vis("animal_marker_top_px") - hh) / dy
        } else {
            f64::INFINITY
        };
        let t = 1.0f64.min(tx).min(ty);
        let (ux, uy) = (hw + dx * t, hh + dy * t);
        let dir = gml::point_direction(hw, hh, sx, sy);
        let hover = hud.animal_marker_hover == Some(o.src);
        o.ui = Some((ux, uy));
        g.set_alpha(a);
        g.set_colour(col);
        triangle(
            g,
            ux + gml::lengthdir_x(r + 18.0, dir),
            uy + gml::lengthdir_y(r + 18.0, dir),
            ux + gml::lengthdir_x(r + 4.0, dir + 16.0),
            uy + gml::lengthdir_y(r + 4.0, dir + 16.0),
            ux + gml::lengthdir_x(r + 4.0, dir - 16.0),
            uy + gml::lengthdir_y(r + 4.0, dir - 16.0),
        );
        g.set_colour(bg);
        g.set_alpha(0.92 * a);
        g.circle(ux, uy, r, false);
        let (sw, sh) = sprite_size(assets, o.sprite);
        let fw = sw / o.frames;
        let sc = r * 2.0 * vis("animal_marker_fill") / fw.max(sh).max(1e-9);
        let fr = gml::clamp(o.frame.floor(), 0.0, o.frames - 1.0);
        g.sprite_part_ext(
            o.sprite,
            0,
            fr * fw,
            0.0,
            fw,
            sh,
            ux - o.mirror * fw * sc * 0.5,
            uy - sh * sc * 0.5,
            o.mirror * sc,
            sc,
            C_WHITE,
            a,
        );
        g.set_alpha(a);
        g.set_colour(col);
        g.circle(ux, uy, r, true);
        g.circle(ux, uy, r - 1.0, true);
        if hover {
            g.circle(ux, uy, r + 3.0, true);
            g.circle(ux, uy, r + 4.0, true);
        }
    }
    g.set_colour(C_WHITE);
    g.set_alpha(1.0);
}

// port: animal_marker_at
/// The marker under a GUI point (topmost first), when at least half visible.
pub fn animal_marker_at(m: &AnimalMarkers, px: f64, py: f64) -> Option<u64> {
    let hit = vis("animal_marker_radius") + 6.0;
    m.list
        .iter()
        .rev()
        .find(|o| {
            o.alpha >= 0.5
                && o.ui
                    .is_some_and(|(x, y)| gml::point_distance(px, py, x, y) <= hit)
        })
        .map(|o| o.src)
}

// port: hover_bar_draw
/// A progress bar centred at `cx` with its bottom at `by` (world units, drawn through `to_gui`).
pub fn hover_bar_draw(
    g: &mut Gm,
    to_gui: &dyn Fn(f64, f64) -> (f64, f64),
    zoom: f64,
    cx: f64,
    by: f64,
    p: f64,
) {
    let w = vis("hover_bar_w");
    let h = vis("hover_bar_h");
    let x = (cx - w * 0.5).floor();
    let y = (by - h).floor();
    let fw = ((w - 2.0) * gml::clamp(p, 0.0, 1.0)).floor();
    let rect = |g: &mut Gm, x1: f64, y1: f64, x2: f64, y2: f64| {
        // a filled draw_rectangle covers x1..x2+1 in world units
        let (a, b) = (to_gui(x1, y1), to_gui(x2 + 1.0, y2 + 1.0));
        let (c, al) = (g.colour, g.alpha);
        g.tri([(a.0, a.1, c, al), (b.0, a.1, c, al), (b.0, b.1, c, al)]);
        g.tri([(a.0, a.1, c, al), (b.0, b.1, c, al), (a.0, b.1, c, al)]);
    };
    let _ = zoom;
    g.set_colour(0.0);
    g.set_alpha(vis("hover_bar_back_alpha"));
    rect(g, x, y, x + w - 1.0, y + h - 1.0);
    g.set_alpha(1.0);
    g.set_colour(C_WHITE);
    if fw > 0.0 {
        rect(g, x + 1.0, y + 1.0, x + fw, y + h - 2.0);
    }
}

// port: hover_bar_step_draw
/// The bar of the first hovered thing: a working mushroom (last first), else an absorbing
/// leaf, else a caught worm. Nothing while the UI blocks.
pub fn hover_bar_step_draw(g: &mut Gm, sim: &Sim, hud: &HudState, cams: &Cameras) {
    if ui_blocking(sim, hud) {
        return;
    }
    let to_gui = |x: f64, y: f64| {
        let p = cams.world_to_gui(DVec2::new(x, y));
        (p.x, p.y)
    };
    let (mx, my) = hud.mouse_world;
    let gap = vis("hover_bar_gap");
    for i in (0..sim.state.mushrooms.len()).rev() {
        let Some(p) = mushrooms::hover_progress(sim, i) else {
            continue;
        };
        let b = mushrooms::mushroom_hit_box(sim, i);
        if gml::point_in_rectangle(mx, my, b.x1, b.y1, b.x2, b.y2) {
            hover_bar_draw(g, &to_gui, cams.zoom, (b.x1 + b.x2) * 0.5, b.y1 - gap, p);
            return;
        }
    }
    let lr = vis("hover_leaf_radius_px");
    for l in life::absorbing_leaves(sim) {
        if gml::point_distance(mx, my, l.x, l.y) <= lr {
            hover_bar_draw(
                g,
                &to_gui,
                cams.zoom,
                l.x,
                l.y - lr - gap,
                1.0 - l.timer / l.total.max(1.0),
            );
            return;
        }
    }
    let wr = vis("hover_worm_radius_px");
    for w in life::caught_worms(sim) {
        let top = w.body.iter().map(|b| b.1).fold(f64::INFINITY, f64::min);
        let hit = w
            .body
            .iter()
            .any(|b| gml::point_distance(mx, my, b.0, b.1) <= wr);
        if hit && !w.body.is_empty() {
            let mid = w.body[w.body.len() / 2];
            let th = vis("worm_thickness_px");
            hover_bar_draw(
                g,
                &to_gui,
                cams.zoom,
                mid.0,
                top - th - gap,
                1.0 - w.timer / w.total.max(1.0),
            );
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn marker(src: u64, warn: bool) -> Marker {
        Marker {
            src,
            sprite: "crow",
            frames: 9.0,
            frame: 0.0,
            mirror: 1.0,
            x: 0.0,
            y: 0.0,
            w: 10.0,
            h: 10.0,
            warn,
            alpha: 0.0,
            vx: 0.0,
            vy: 0.0,
            gone: false,
            seen: false,
            follow: 0.0,
            ui: None,
        }
    }

    /// crow_due: the pending crow's marker becomes the arriving crow's (same source id) and
    /// stops being a warning.
    #[test]
    fn arriving_crow_takes_over_its_warning_marker() {
        let mut m = AnimalMarkers::default();
        animal_markers_update(
            &mut m,
            vec![marker(7, true)],
            false,
            0.016,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        assert!(m.list[0].warn);
        animal_markers_update(
            &mut m,
            vec![marker(7, false)],
            false,
            0.016,
            0.0,
            0.0,
            100.0,
            100.0,
        );
        assert_eq!(m.list.len(), 1);
        assert!(!m.list[0].warn);
    }
}
