//! Drawing and mouse side of the tasks panel, the tasks popup and the tutorial tips
//! (scr_game `tasks_*`, `goal_sprite_*`, `tutorial_draw`). The logic (progress, toggles, popup
//! input) is `myco_core::progress::tasks`; here are the parts that measure text or draw.

use super::gm_draw::Gm;
use super::hud::{text, text_ext, triangle, GmMeasure, C_WHITE};
use super::hud::{HudState, UiFonts, R};
use crate::assets::GameAssets;
use myco_core::balance::{bal, vis};
use myco_core::gml;
use myco_core::locale::Locale;
use myco_core::progress::goals::{goal_fit, goal_number_text, goal_spore_count, goal_text_scale};
use myco_core::progress::tasks::{self, Block};
use myco_core::progress::tutorial::{tutorial_wrap, Tip};
use myco_core::sim::{Command, Sim};
use myco_core::world::perks::perk_has;

/// One line of a block (`___struct___30..32`).
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub key: String,
    pub text: String,
    pub done: bool,
    pub reward: bool,
}

// port: tasks_block_items
/// The lines of a block: the tasks with their done flags; migration needs (spores, water,
/// food) as "have / need"; the five cave perks.
pub fn tasks_block_items(sim: &Sim, loc: &Locale, block: Block) -> Vec<Item> {
    match block {
        Block::Tasks => tasks::tasks_defs()
            .iter()
            .map(|d| Item {
                key: d.as_str().to_string(),
                text: loc.loc(&format!("task_{}", d.as_str())).to_string(),
                done: myco_core::balance::gml_truthy(sim.state.state.tasks.done.get(d.as_str())),
                reward: true,
            })
            .collect(),
        Block::Migration => {
            let have = [
                goal_spore_count(sim),
                sim.water_total(),
                sim.state.state.food,
            ];
            let need = [
                sim.bal("migration_spore_mushrooms"),
                sim.bal("migration_water"),
                sim.bal("migration_food"),
            ];
            ["spores", "water", "food"]
                .iter()
                .enumerate()
                .map(|(i, k)| Item {
                    key: format!("m:{k}"),
                    text: format!(
                        "{}  {} / {}",
                        loc.loc(&format!("goal_item_{k}")),
                        goal_number_text(have[i].min(need[i])),
                        goal_number_text(need[i])
                    ),
                    done: have[i] >= need[i],
                    reward: false,
                })
                .collect()
        }
        Block::Magic => ["green", "yellow", "purple", "blue", "red"]
            .iter()
            .map(|c| Item {
                key: format!("p:{c}"),
                text: loc.loc(&format!("goal_item_{c}")).to_string(),
                done: perk_has(&sim.state, c),
                reward: false,
            })
            .collect(),
    }
}

/// A block header (`___struct___33`).
#[derive(Clone, Debug, PartialEq)]
pub struct BlockHead {
    pub id: Block,
    pub title: String,
    pub y: f64,
    pub k: f64,
}

// port: tasks_blocks
/// The visible blocks with their titles, y and fold: the tasks block in stage 0, nothing while
/// the "tasks done" popup is up, then the two goal blocks.
pub fn tasks_blocks(sim: &Sim, loc: &Locale) -> Vec<BlockHead> {
    let t = &sim.state.state.tasks;
    let ids: &[Block] = if t.stage == 0 {
        &[Block::Tasks]
    } else if t.popup == "tasks" {
        &[]
    } else {
        &[Block::Migration, Block::Magic]
    };
    let mut y = vis("tasks_y");
    let mut out = Vec::new();
    for &id in ids {
        let k = sim
            .progress
            .tasks
            .fold_k
            .get(id.as_str())
            .copied()
            .unwrap_or(1.0);
        let n = match id {
            Block::Tasks => tasks::tasks_defs().len() as f64,
            Block::Migration => 3.0,
            Block::Magic => 5.0,
        };
        let title = match id {
            Block::Tasks => "tasks_title".to_string(),
            _ => format!("goal_{}_label", id.as_str()),
        };
        out.push(BlockHead {
            id,
            title: loc.loc(&title).to_string(),
            y,
            k,
        });
        y += (vis("tasks_first_gap") + (n - 1.0) * vis("tasks_line_px")) * k
            + vis("tasks_block_gap");
    }
    out
}

/// `tasks_header_rect` result (`___struct___34`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeaderRect {
    pub r: R,
    pub text_w: f64,
}

// port: tasks_header_rect
/// The clickable header: the title (ui font) plus the fold arrow, padded.
pub fn tasks_header_rect(
    assets: &GameAssets,
    uf: &UiFonts,
    ui_ox: f64,
    b: &BlockHead,
) -> HeaderRect {
    let x = vis("tasks_x") + ui_ox;
    let scale = vis("tasks_title_scale");
    let mut m = Gm::new(assets);
    m.font = uf.font.clone();
    let k = super::hud::eff_scale(&m, scale);
    let tw = m.string_width(&b.title) * k;
    let th = m.string_height("A") * k;
    HeaderRect {
        r: R::new(
            x - 8.0,
            b.y - th * 0.5 - 6.0,
            x + tw + vis("tasks_arrow_gap") + vis("tasks_arrow_px") + 8.0,
            b.y + th * 0.5 + 6.0,
        ),
        text_w: tw,
    }
}

// port: tasks_mouse_step
/// A click on a header toggles its block (`Command::ToggleTaskBlock`); the cursor on a header
/// takes the mouse.
#[allow(clippy::too_many_arguments)]
pub fn tasks_mouse_step(
    sim: &Sim,
    hud: &HudState,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
    ui_ox: f64,
    mx: f64,
    my: f64,
    click: bool,
    cmds: &mut Vec<Command>,
) -> bool {
    if !visible(sim, hud) || super::hud::ui_blocking(sim, hud) {
        return false;
    }
    for b in tasks_blocks(sim, loc) {
        let r = tasks_header_rect(assets, uf, ui_ox, &b).r;
        if r.contains(mx, my) {
            if click {
                cmds.push(Command::ToggleTaskBlock { block: b.id });
            }
            return true;
        }
    }
    false
}

/// `tasks_visible()` with the HUD's flags.
pub fn visible(sim: &Sim, hud: &HudState) -> bool {
    let input = myco_core::sim::StepInput {
        hud_hidden: hud.hud_hidden,
        ..Default::default()
    };
    tasks::tasks_visible(sim, &input)
}

// port: tasks_draw
#[allow(clippy::too_many_arguments)]
/// The tasks panel (all visible blocks).
pub fn tasks_draw(
    g: &mut Gm,
    sim: &Sim,
    hud: &HudState,
    uf: &UiFonts,
    loc: &Locale,
    ui_ox: f64,
    gw: f64,
    assets: &GameAssets,
) {
    if !visible(sim, hud) {
        return;
    }
    for b in tasks_blocks(sim, loc) {
        let items = tasks_block_items(sim, loc, b.id);
        tasks_block_draw(g, sim, hud, uf, ui_ox, gw, assets, &b, &items);
    }
    g.set_alpha(1.0);
    g.font = uf.small.clone();
    g.halign = 0;
    g.valign = 0;
}

// port: tasks_block_draw
/// One block: shadowed title (hover colour), fold arrow, the fill animation of every item, then
/// (unfolded) circles with their fill, the shadowed texts and the "+N" reward with its icon.
#[allow(clippy::too_many_arguments)]
pub fn tasks_block_draw(
    g: &mut Gm,
    sim: &Sim,
    hud: &HudState,
    uf: &UiFonts,
    ui_ox: f64,
    gw: f64,
    assets: &GameAssets,
    b: &BlockHead,
    items: &[Item],
) {
    use super::hud::{
        COLOR_DIM as C_REWARD, COLOR_GOOD as C_HOVER, COLOR_INK as C_TEXT, COLOR_PANEL as C_BOX,
    };
    let a = sim.progress.tasks.alpha;
    let hr = tasks_header_rect(assets, uf, ui_ox, b);
    let x = vis("tasks_x") + ui_ox;
    let y = b.y;
    let ts = vis("tasks_title_scale");
    let mut xs = vis("tasks_text_scale");
    let lp = vis("tasks_line_px");
    let bx = vis("tasks_box_px") * 0.5;
    let ap = vis("tasks_arrow_px");
    let (mx, my) = hud.mouse_gui;
    let hov = hr.r.contains(mx, my);
    let k = b.k;
    g.font = uf.font.clone();
    g.valign = 1;
    g.halign = 0;
    g.set_colour(0.0);
    g.set_alpha(a * 0.6);
    text(g, x + 3.0, y + 3.0, &b.title, ts, ts);
    g.set_colour(if hov { C_HOVER } else { C_TEXT });
    g.set_alpha(a);
    text(g, x, y, &b.title, ts, ts);
    let ax = x + hr.text_w + vis("tasks_arrow_gap") + ap * 0.5;
    let off = ap * 0.45 * gml::lerp(1.0, -1.0, 1.0 - k);
    triangle(
        g,
        ax - ap * 0.5,
        y + off,
        ax + ap * 0.5,
        y + off,
        ax,
        y - off,
    );
    // the check-mark fill advances per step in `tasks_fill_step`
    if k <= 0.0 {
        return;
    }
    let tx = x + bx * 2.0 + vis("tasks_box_gap");
    let up_h = assets.sprite("UPGRADE").size.y as f64;
    let icon = vis("tasks_reward_icon_px") / up_h.max(1.0);
    let ia = a * k * k;
    let maxw = gw * 0.34 - (tx - x) - 60.0;
    for it in items {
        xs = goal_fit(&GmMeasure(g), &it.text, xs, maxw, 0.0, -1.0);
    }
    let reward_points = bal(sim.state.state.mode, "task_reward_points");
    for (i, it) in items.iter().enumerate() {
        let f = sim.progress.tasks.fill.get(&it.key).copied().unwrap_or(0.0);
        let iy = y + (vis("tasks_first_gap") + i as f64 * lp) * k;
        let cx = x + bx;
        g.set_alpha(ia);
        g.set_colour(C_TEXT);
        g.circle(cx, iy, bx, false);
        g.set_colour(C_BOX);
        g.circle(cx, iy, bx - 2.0, false);
        if f > 0.0 {
            g.set_colour(C_HOVER);
            g.circle(cx, iy, ((bx - 2.0) * f).max(1.0), false);
        }
        g.set_colour(0.0);
        g.set_alpha(ia * 0.6);
        text(g, tx + 2.0, iy + 2.0, &it.text, xs, xs);
        g.set_colour(if it.done && it.reward {
            C_REWARD
        } else {
            C_TEXT
        });
        g.set_alpha(ia);
        text(g, tx, iy, &it.text, xs, xs);
        if it.done && it.reward {
            let rx = tx
                + g.string_width(&it.text) * super::hud::eff_scale(g, xs)
                + vis("tasks_reward_gap");
            let pulse = (f * std::f64::consts::PI).sin() * 0.35 + 1.0;
            let rs = xs * pulse;
            let s = format!("+{}", super::hud::fmt_num(reward_points));
            g.set_alpha(ia * f);
            g.set_colour(C_HOVER);
            text(g, rx, iy, &s, rs, rs);
            let isc = icon * pulse;
            let sx = rx + g.string_width(&s) * super::hud::eff_scale(g, rs) + 4.0;
            g.sprite_ext(
                "UPGRADE",
                0,
                sx,
                iy - up_h * isc * 0.5,
                isc,
                isc,
                C_WHITE,
                ia * f,
            );
        }
    }
}

// port: goal_sprite_fit
/// A sprite fitted into a w×h box centred at (cx, cy), respecting its origin.
#[allow(clippy::too_many_arguments)]
pub fn goal_sprite_fit(
    g: &mut Gm,
    assets: &GameAssets,
    spr: &str,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
) {
    let Some(def) = assets.try_sprite(spr) else {
        return;
    };
    let (sw, sh) = (def.size.x as f64, def.size.y as f64);
    let s = (w / sw.max(1.0)).min(h / sh.max(1.0));
    let dy = (sh * 0.5 - def.origin.y as f64) * s;
    let dx = (sw * 0.5 - def.origin.x as f64) * s;
    g.sprite_ext(spr, 0, cx - dx, cy - dy, s, s, super::hud::icon_tint(spr), 1.0);
}

// port: goal_sprite_part_fit
/// A part of a sprite fitted into a w×h box centred at (cx, cy).
#[allow(clippy::too_many_arguments)]
pub fn goal_sprite_part_fit(
    g: &mut Gm,
    _assets: &GameAssets,
    spr: &str,
    left: f64,
    top: f64,
    pw: f64,
    ph: f64,
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
) {
    let s = (w / pw.max(1.0)).min(h / ph.max(1.0));
    let x = cx - pw * s * 0.5;
    let y = cy - ph * s * 0.5;
    g.sprite_part_ext(spr, 0, left, top, pw, ph, x, y, s, s, C_WHITE, 1.0);
}

// port: tasks_popup_draw
/// The tasks / goal popup: dimmed screen and rounded panel (not for `migration_end`), the
/// "tasks done" page with both goals and the migration needs, or the goal text; then the button.
#[allow(clippy::too_many_arguments)]
pub fn tasks_popup_draw(
    g: &mut Gm,
    sim: &Sim,
    uf: &UiFonts,
    loc: &Locale,
    assets: &GameAssets,
    c_text: f64,
    c_btn_text: f64,
    c_btn: f64,
    gw: f64,
    gh: f64,
    mouse: (f64, f64),
) {
    if !tasks::tasks_popup_open(sim) {
        return;
    }
    let r = tasks::tasks_popup_rect(sim, gw, gh);
    let cx = r.cx;
    let (mx, my) = mouse;
    g.font = uf.font.clone();
    g.halign = 1;
    g.valign = 1;
    let popup = sim.state.state.tasks.popup.as_str();
    let is_end = popup == "migration_end";
    if !is_end {
        g.set_colour(0.0);
        g.set_alpha(0.55);
        g.rectangle(0.0, 0.0, gw, gh, false);
        g.set_alpha(1.0);
        g.set_colour(0x2a3132 as f64);
        g.roundrect_fill(r.x1, r.y1, r.x2, r.y2, 90.0, 90.0);
    }
    g.set_colour(c_text);
    if !is_end {
        if popup == "tasks" {
            let title = loc.loc("goal_tasks_title");
            let mut s = goal_text_scale(&GmMeasure(g), 36.0);
            s = goal_fit(&GmMeasure(g), title, s, 1240.0, 0.0, -1.0);
            text(g, cx, r.y1 + 58.0, title, s, s);
            let body = loc.loc("goal_tasks_text");
            s = goal_text_scale(&GmMeasure(g), 22.0);
            let sep = 40.0 / s;
            s = goal_fit(&GmMeasure(g), body, s, 1240.0, 160.0, sep);
            text_ext(g, cx, r.y1 + 162.0, body, sep, 1240.0 / s, s, s);
            let ly = r.y1 + 266.0;
            g.rectangle(cx - 3.0, ly, cx + 3.0, r.y2 - 44.0, false);
            s = goal_text_scale(&GmMeasure(g), 32.0);
            let (lx, rx) = (cx - 323.0, cx + 320.0);
            let sep = 38.0 / s;
            let (mig, mag) = (loc.loc("goal_migration_title"), loc.loc("goal_magic_title"));
            let ts = goal_fit(&GmMeasure(g), mig, s, 600.0, 120.0, sep).min(goal_fit(
                &GmMeasure(g),
                mag,
                s,
                600.0,
                120.0,
                sep,
            ));
            text_ext(g, lx, ly + 55.0, mig, sep, 600.0 / ts, ts, ts);
            text_ext(g, rx, ly + 55.0, mag, sep, 600.0 / ts, ts, ts);
            let icons = ["", "apple_bite", "water65"];
            let needs = [
                sim.bal("migration_spore_mushrooms"),
                sim.bal("migration_food"),
                sim.bal("migration_water"),
            ];
            let xs = [cx - 542.0, cx - 373.0, cx - 152.0];
            let ns = goal_text_scale(&GmMeasure(g), 30.0);
            for i in 0..3 {
                if i == 0 {
                    goal_sprite_part_fit(
                        g,
                        assets,
                        "sporeshroom",
                        4.0,
                        15.0,
                        38.0,
                        49.0,
                        xs[i],
                        ly + 207.0,
                        76.0,
                        88.0,
                    );
                } else {
                    goal_sprite_fit(g, assets, icons[i], xs[i], ly + 207.0, 80.0, 84.0);
                }
                text(g, xs[i], ly + 287.0, &goal_number_text(needs[i]), ns, ns);
            }
            goal_sprite_fit(g, assets, "mush5", rx, ly + 235.0, 490.0, 184.0);
        } else {
            let mut s = goal_text_scale(&GmMeasure(g), 26.0);
            let txt = loc.loc(if popup == "migration_ready" {
                "goal_migration_done"
            } else {
                "goal_magic_done"
            });
            let sep = 52.0 / s;
            s = goal_fit(
                &GmMeasure(g),
                txt,
                s,
                (r.x2 - r.x1) - 40.0,
                (r.y2 - r.y1) * 0.55,
                sep,
            );
            text_ext(
                g,
                cx,
                (r.y1 + r.y2) * 0.5,
                txt,
                sep,
                ((r.x2 - r.x1) - 40.0) / s,
                s,
                s,
            );
        }
    }
    for b in tasks::tasks_popup_buttons(sim, gw, gh) {
        let label = loc.loc(b.label_key);
        let hov = gml::point_in_rectangle(mx, my, b.x1, b.y1, b.x2, b.y2);
        let bs = goal_text_scale(&GmMeasure(g), 20.0)
            .min((b.x2 - b.x1 - 32.0) / g.string_width(label).max(1.0));
        g.set_colour(c_btn);
        g.set_alpha(0.95);
        g.rectangle(b.x1, b.y1, b.x2, b.y2, false);
        g.set_alpha(1.0);
        g.set_colour(if hov { c_text } else { c_btn_text });
        g.rectangle(b.x1, b.y1, b.x2, b.y2, true);
        text(g, (b.x1 + b.x2) * 0.5, (b.y1 + b.y2) * 0.5, label, bs, bs);
    }
    g.set_alpha(1.0);
    g.font = uf.small.clone();
    g.halign = 0;
    g.valign = 0;
}

/// `tutor_gribi_1_00000` .. `_00028` that exist (`tutorial_init`'s hand frames).
pub fn hand_frames(assets: &GameAssets) -> Vec<String> {
    (0..=28)
        .map(|i| format!("tutor_gribi_1_{i:05}"))
        .filter(|n| assets.try_sprite(n).is_some())
        .collect()
}

// port: tutorial_draw
/// The tutorial: the "aim here" hand and the camera hint over the mycelium origin, then every
/// visible tip (wrapped, fitted to the GUI width, clamped on screen unless world-anchored) with
/// a shadow.
#[allow(clippy::too_many_arguments)]
pub fn tutorial_draw(
    g: &mut Gm,
    sim: &Sim,
    assets: &GameAssets,
    uf: &UiFonts,
    loc: &Locale,
    box_: R,
    col: f64,
    gw: f64,
    gh: f64,
    to_gui: &dyn Fn(f64, f64) -> (f64, f64),
    myc: (f64, f64),
) {
    let rt = &sim.progress.tutorial;
    // the mycelium origin on screen (the original: `myc - camera_view_x/y` at 1×)
    let (mgx, mgy) = to_gui(myc.0, myc.1);
    let pad = 24.0;
    let frames = hand_frames(assets);
    let nf = frames.len();
    if nf > 0 && rt.hand_alpha > 0.0 {
        let spr = &frames[gml::gmod(rt.hand_frame.floor(), nf as f64) as usize];
        let sc = vis("tutorial_hand_scale");
        // the sprite's origin is the centre of its "drag from here" square: the original shifts
        // it `tutorial_hand_offset_x` (48 px) right of the node; here (the owner's request) the
        // square sits on the node
        g.sprite_ext(
            spr,
            0,
            mgx,
            mgy + vis("tutorial_hand_offset_y"),
            sc,
            sc,
            C_WHITE,
            rt.hand_alpha,
        );
    }
    if let Some(cs) = assets.try_sprite("CAMERA_TUTOR") {
        if rt.camera_alpha > 0.0 {
            let sc = vis("tutorial_camera_scale");
            let (sw, sh) = (cs.size.x as f64, cs.size.y as f64);
            let cx = mgx - (sw * 0.5 - cs.origin.x as f64) * sc;
            let cy = mgy - vis("tutorial_camera_offset_y") - (sh - cs.origin.y as f64) * sc;
            let n = cs.frames.len().max(1) as f64;
            g.sprite_ext(
                "CAMERA_TUTOR",
                (gml::gmod(rt.camera_frame.floor(), n)) as usize,
                cx,
                cy,
                sc,
                sc,
                C_WHITE,
                rt.camera_alpha,
            );
        }
    }
    g.font = uf.font.clone();
    g.halign = 1;
    g.valign = 1;
    for tip in Tip::ORDER {
        let name = tip.as_str();
        let Some(t) = sim.state.state.tutorial.tips.get(name) else {
            continue;
        };
        if t.alpha <= 0.0 {
            continue;
        }
        let txt = tutorial_wrap(loc.loc(&format!("tut_{name}")));
        let mut sc = vis(if tip == Tip::Final {
            "tutorial_final_scale"
        } else {
            "tutorial_text_scale"
        });
        let (mut tx, mut ty) = (t.x, t.y);
        match t.anchor.as_str() {
            "world" => {
                (tx, ty) = to_gui(tx, ty);
            }
            "screen" => {
                tx *= gw;
                ty *= gh;
            }
            "upgrades" => {
                tx = (box_.x1 + box_.x2) * 0.5;
                ty = box_.y2 + 40.0;
            }
            _ => {}
        }
        sc = goal_fit(&GmMeasure(g), &txt, sc, gw - pad * 2.0, 0.0, -1.0);
        if t.anchor != "world" {
            let k = super::hud::eff_scale(g, sc);
            let hw = g.string_width(&txt) * k * 0.5;
            let hh = g.string_height(&txt) * k * 0.5;
            tx = gml::clamp(tx, pad + hw, gw - pad - hw);
            ty = gml::clamp(ty, pad + hh, gh - pad - hh);
        }
        g.set_colour(0.0);
        g.set_alpha(t.alpha * 0.6);
        text(g, tx + 3.0, ty + 3.0, &txt, sc, sc);
        g.set_colour(col);
        g.set_alpha(t.alpha);
        text(g, tx, ty, &txt, sc, sc);
    }
    g.set_alpha(1.0);
    g.font = uf.small.clone();
    g.halign = 0;
    g.valign = 0;
}

/// The fill animation of `tasks_block_draw` (each item's check mark eases toward done by
/// `1 / tasks_fill_frames`). The original runs it in Draw GUI, once per 60 Hz frame; here it runs
/// once per fixed step, for the blocks `tasks_draw` would draw.
pub fn tasks_fill_step(sim: &mut Sim, hud: &HudState, loc: &Locale) {
    if !visible(sim, hud) {
        return;
    }
    let step = 1.0 / vis("tasks_fill_frames").max(1.0);
    for b in tasks_blocks(sim, loc) {
        let items = tasks_block_items(sim, loc, b.id);
        let fill = &mut sim.progress.tasks.fill;
        for it in items {
            let target = if it.done { 1.0 } else { 0.0 };
            let cur = fill.get(&it.key).copied().unwrap_or(target);
            let v = if cur >= target {
                target.max(cur - step)
            } else {
                target.min(cur + step)
            };
            fill.insert(it.key, v);
        }
    }
}
