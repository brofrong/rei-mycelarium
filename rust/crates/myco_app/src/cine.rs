//! Drawing of the intro (the hedgehog bringing the origin mushroom) and of the migration
//! cut-scene. Their logic is myco_core's (`camera::intro_*`, `cine`); this module only draws it.
//!
//! - `intro_draw` is the first thing obj_life's world Draw does while `intro_active`: two world
//!   sprites (the hedgehog, the mushroom on its back or falling) at the start of the life layer.
//! - `cine_draw` is part of obj_hud Draw_75 (GUI), between the hover tooltip and the tasks popup;
//!   [`crate::ui::hud::draw_tooltip`] calls it there.

use crate::assets::GameAssets;
use crate::render::layers::{z_at, Z_LIFE};
use crate::render::sprite::SpriteCmd;
use crate::sim_bridge::SimRes;
use crate::ui::gm_draw::Gm;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use myco_core::balance::vis;
use myco_core::cine::CinePhase;
use myco_core::gml;
use myco_core::sim::Sim;

const C_WHITE: f64 = 16777215.0;
const C_BLACK: f64 = 0.0;

// port: intro_draw
/// The intro's world sprites: the walking hedgehog (`hedgehog_draw` facing right) and the origin
/// mushroom, on its back until dropped, then falling at the start point. Empty after the intro.
pub fn intro_draw_list(sim: &Sim, assets: &GameAssets) -> Vec<SpriteCmd> {
    if !sim.intro_active {
        return Vec::new();
    }
    let it = &sim.life.intro;
    let mut out = Vec::new();
    // hedgehog_draw(x, y, dir = 1, frame): the 8-frame strip, mirrored (it faces left in the art)
    if let Some(def) = assets.try_sprite("hedgehog") {
        let s = vis("hedgehog_scale");
        let fw = def.size.x as f64 / 8.0;
        let fh = def.size.y as f64;
        let sx = -1.0;
        let dx = it.x.floor() - sx * fw * s * 0.5;
        out.push(
            SpriteCmd::part(
                "hedgehog",
                it.frame * fw,
                0.0,
                fw,
                fh,
                dx,
                it.y.floor() - fh * s,
                sx * s,
                s,
            )
            .at_z(z_at(Z_LIFE, 0)),
        );
    }
    let (sx, sy) = if it.dropped {
        (sim.state.state.start_x, it.shroom_y)
    } else {
        (it.x, it.y + myco_core::life::hedgehog::hedgehog_back_y())
    };
    out.push(
        SpriteCmd::ext("originshroom", 0, sx.floor(), sy.floor(), 1.0, 1.0, 0.0)
            .at_z(z_at(Z_LIFE, 1)),
    );
    out
}

/// One of the intro's two sprites.
#[derive(Component)]
struct IntroSprite(usize);

/// Update: keep two entities in sync with [`intro_draw_list`] (hidden after the intro).
fn intro_draw(
    mut commands: Commands,
    sim: Res<SimRes>,
    assets: Option<Res<GameAssets>>,
    mut q: Query<(
        &IntroSprite,
        &mut Sprite,
        &mut Anchor,
        &mut Transform,
        &mut Visibility,
    )>,
) {
    let Some(assets) = assets else { return };
    let list = intro_draw_list(&sim.0, &assets);
    let mut have = [false; 2];
    for (k, mut sprite, mut anchor, mut tf, mut vis) in &mut q {
        have[k.0] = true;
        match list.get(k.0).and_then(|c| c.to_bevy(&assets)) {
            Some((s, a, t)) => {
                *sprite = s;
                anchor.set_if_neq(a);
                tf.set_if_neq(t);
                vis.set_if_neq(Visibility::Inherited);
            }
            None => {
                vis.set_if_neq(Visibility::Hidden);
            }
        }
    }
    for (k, cmd) in list.iter().enumerate() {
        if !have[k] {
            if let Some((s, a, t)) = cmd.to_bevy(&assets) {
                commands.spawn((s, a, t, IntroSprite(k)));
            }
        }
    }
}

// port: cine_ball
/// The planet on the cut-scene screen: centred, 45% down, radius `migration_cine_ball_share` of
/// the GUI height. Returns `(x, y, r)`.
pub fn cine_ball(gw: f64, gh: f64) -> (f64, f64, f64) {
    (gw * 0.5, gh * 0.45, gh * vis("migration_cine_ball_share"))
}

/// `draw_sprite_ext(repel_flash_sprite, …)`: the soft disc `repeller_soft_sprite(256, 2.2)`
/// (alpha `(1 − d)^hardness`, white) of radius `r`, drawn as rings of interpolated alpha.
fn soft_disc(g: &mut Gm, x: f64, y: f64, r: f64, hardness: f64, col: f64, alpha: f64) {
    const RINGS: usize = 24;
    const SEGS: usize = 64;
    let a_at = |d: f64| (1.0 - d).max(0.0).powf(hardness) * alpha;
    for i in 0..RINGS {
        let (d0, d1) = (i as f64 / RINGS as f64, (i + 1) as f64 / RINGS as f64);
        let (a0, a1) = (a_at(d0), a_at(d1));
        for j in 0..SEGS {
            let t0 = j as f64 / SEGS as f64 * std::f64::consts::TAU;
            let t1 = (j + 1) as f64 / SEGS as f64 * std::f64::consts::TAU;
            let p = |d: f64, t: f64| (x + t.cos() * d * r, y + t.sin() * d * r);
            let (p00, p01, p10, p11) = (p(d0, t0), p(d0, t1), p(d1, t0), p(d1, t1));
            g.tri([
                (p00.0, p00.1, col, a0),
                (p10.0, p10.1, col, a1),
                (p11.0, p11.1, col, a1),
            ]);
            g.tri([
                (p00.0, p00.1, col, a0),
                (p11.0, p11.1, col, a1),
                (p01.0, p01.1, col, a0),
            ]);
        }
    }
}

// port: cine_draw
/// The cut-scene over the GUI: during the tour the white flash of the origin burst; then the
/// planet screen (black, twinkling stars, the glowing Earth, the mycelium spreading over it
/// segment by segment, the white-out and the first half second's white fade), all faded by the
/// cut-scene's alpha.
///
/// The glow is the original's additive `repel_flash_sprite`; here it blends normally over the
/// black backdrop (the same on black; slightly dimmer where it covers a star).
pub fn cine_draw(g: &mut Gm, sim: &Sim, gw: f64, gh: f64) {
    let Some(c) = sim.progress.tasks.cine.as_ref() else {
        return;
    };
    if c.phase == CinePhase::Show {
        let w = c.white.unwrap_or(0.0);
        if w > 0.0 {
            g.set_alpha(w);
            g.set_colour(C_WHITE);
            g.rectangle(0.0, 0.0, gw, gh, false);
            g.set_alpha(1.0);
        }
        return;
    }
    let tt = if c.phase == CinePhase::Play {
        c.t
    } else {
        vis("migration_cine_seconds") + c.t
    };
    let al = c.alpha;
    let (bx, by, br) = cine_ball(gw, gh);
    g.set_alpha(al);
    g.set_colour(C_BLACK);
    g.rectangle(0.0, 0.0, gw, gh, false);
    for st in &c.stars {
        let tw = 0.85 + 0.15 * gml::dsin(st.ph + tt * 60.0 * st.sp);
        let sx = st.x * gw;
        let sy = st.y * gh;
        g.set_alpha(al * st.a * tw);
        g.set_colour(C_WHITE);
        g.rectangle(sx, sy, sx + st.s - 1.0, sy + st.s - 1.0, false);
        if st.s >= 3.0 {
            let h = (st.s - 1.0) * 0.5;
            g.set_alpha(al * st.a * tw * 0.45);
            g.rectangle(sx - st.s, sy + h - 0.5, sx + st.s * 2.0 - 1.0, sy + h + 0.5, false);
            g.rectangle(sx + h - 0.5, sy - st.s, sx + h + 0.5, sy + st.s * 2.0 - 1.0, false);
        }
    }
    let k = br / 540.0;
    let wk = cine_span(
        tt,
        vis("migration_cine_white_from"),
        vis("migration_cine_white_to"),
    );
    let col = gml::merge_colour(16755310.0, C_WHITE, wk);
    let pulse = 1.0 + 0.04 * (tt * 3.0).sin();
    soft_disc(g, bx, by, br * 2.4 * pulse, 2.2, col, al * gml::lerp(0.35, 0.6, wk));
    soft_disc(g, bx, by, br * 1.5, 2.2, col, al * gml::lerp(0.45, 0.9, wk));
    g.sprite_ext("Earth", 0, bx - 668.0 * k, by - 618.0 * k, k, k, C_WHITE, al);
    let mk = cine_span(
        tt,
        vis("migration_cine_myc_from"),
        vis("migration_cine_myc_to"),
    );
    if mk > 0.0 {
        let lw = 1.0 + 3.0 * wk + 1.5 * mk;
        g.set_colour(15529206.0);
        g.set_alpha(al);
        for sg in &c.segs {
            if sg.time > mk {
                continue;
            }
            g.line_width(
                bx + sg.x1 * br,
                by + sg.y1 * br,
                bx + sg.x2 * br,
                by + sg.y2 * br,
                (lw * (0.4 + 0.6 * sg.w)).max(1.0),
            );
        }
    }
    if wk > 0.0 {
        g.set_alpha(al * wk * wk);
        g.set_colour(C_WHITE);
        g.circle(bx, by, br, false);
    }
    let fade = 1.0 - cine_span(tt, 0.0, vis("migration_white_clear_seconds"));
    if c.phase == CinePhase::Play && fade > 0.0 {
        g.set_alpha(al * fade);
        g.set_colour(C_WHITE);
        g.rectangle(0.0, 0.0, gw, gh, false);
    }
    g.set_alpha(1.0);
    g.set_colour(C_WHITE);
}

fn cine_span(t: f64, a: f64, b: f64) -> f64 {
    myco_core::cine::cine_span(t, a, b)
}

/// The intro's world sprites (the cut-scene is drawn by the HUD's Draw GUI).
pub struct CinePlugin;

impl Plugin for CinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, intro_draw);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cine_ball_is_centred_45_percent_down() {
        let (x, y, r) = cine_ball(1920.0, 1080.0);
        assert_eq!((x, y), (960.0, 486.0));
        assert!((r - 1080.0 * vis("migration_cine_ball_share")).abs() < 1e-9);
    }
}
