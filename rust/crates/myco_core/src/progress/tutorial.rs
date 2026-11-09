//! Tutorial (scr_game `tutorial_*`): tips that appear as the player reaches each step. Saved
//! state: `state.state.tutorial` (tips, pending events, ate/drank/hand_done); the hand and
//! camera animations live in [`TutorialRt`]. Drawing is the app's (Task 17); a tip that
//! appears is also announced as `SimEvent::TutorialTip`.

use crate::balance::{vis, wld};
use crate::events::SimEvent;
use crate::gml;
use crate::mycelium::soil::mycelium_cm;
use crate::sim::{Sim, StepInput};
use crate::state::{TipSave, TutorialSave};
use serde_json::{json, Value};

/// A tutorial tip (`tut.order`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tip {
    Shoot,
    Node,
    Salt,
    Water,
    Xp,
    Upgrade,
    Mushrooms,
    Rain,
    Clay,
    Final,
}

impl Tip {
    /// `tut.order`: the order tips are updated (and drawn) in.
    pub const ORDER: [Tip; 10] = [
        Tip::Shoot,
        Tip::Node,
        Tip::Salt,
        Tip::Water,
        Tip::Xp,
        Tip::Upgrade,
        Tip::Mushrooms,
        Tip::Rain,
        Tip::Clay,
        Tip::Final,
    ];

    /// The key in `tutorial.tips` (and the `tutorial_<key>` locale key).
    pub fn as_str(self) -> &'static str {
        match self {
            Tip::Shoot => "shoot",
            Tip::Node => "node",
            Tip::Salt => "salt",
            Tip::Water => "water",
            Tip::Xp => "xp",
            Tip::Upgrade => "upgrade",
            Tip::Mushrooms => "mushrooms",
            Tip::Rain => "rain",
            Tip::Clay => "clay",
            Tip::Final => "final",
        }
    }

    pub fn from_key(s: &str) -> Option<Tip> {
        Tip::ORDER.into_iter().find(|t| t.as_str() == s)
    }
}

/// `tip.state`: 0 not shown yet, 1 showing, 2 done (fading out / gone).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TipState {
    Hidden,
    Shown,
    Done,
}

impl TipState {
    pub fn code(self) -> i64 {
        match self {
            TipState::Hidden => 0,
            TipState::Shown => 1,
            TipState::Done => 2,
        }
    }

    pub fn from_code(code: i64) -> Option<TipState> {
        Some(match code {
            0 => TipState::Hidden,
            1 => TipState::Shown,
            2 => TipState::Done,
            _ => return None,
        })
    }
}

/// `tip.anchor`: what the tip's position is relative to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// world px
    World,
    /// fractions of the GUI size
    Screen,
    /// the upgrades button
    Upgrades,
}

impl Anchor {
    pub fn as_str(self) -> &'static str {
        match self {
            Anchor::World => "world",
            Anchor::Screen => "screen",
            Anchor::Upgrades => "upgrades",
        }
    }
}

/// Runtime part of `global.tut` (the sprites `hand_frames` / `camera_sprite` are the app's).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TutorialRt {
    /// frame of the "aim here" hand animation (fractional, at `tutorial_hand_fps`)
    pub hand_frame: f64,
    pub hand_alpha: f64,
    /// alpha / frame of the camera hint shown with the salt and water tips
    pub camera_alpha: f64,
    pub camera_frame: f64,
}

// port: tutorial_init
/// Every tip hidden, no events, nothing eaten or drunk.
pub fn init(sim: &mut Sim) {
    sim.state.state.tutorial = TutorialSave::default();
    sim.progress.tutorial = TutorialRt::default();
}

// port: tutorial_event
/// Report a tutorial event (`aim`, `shot`, `node`, `clay`, `food`, `water`, `mushroom`) at
/// a point (missing coordinates count as 0); the next [`step`] consumes it.
pub fn event(sim: &mut Sim, name: &str, x: Option<f64>, y: Option<f64>) {
    let (x, y) = (x.unwrap_or(0.0), y.unwrap_or(0.0));
    sim.state
        .state
        .tutorial
        .events
        .insert(name.to_string(), json!({ "x": x, "y": y }));
}

fn tip_mut(sim: &mut Sim, tip: Tip) -> &mut TipSave {
    sim.state
        .state
        .tutorial
        .tips
        .entry(tip.as_str().to_string())
        .or_default()
}

fn tip_alpha(sim: &Sim, tip: Tip) -> f64 {
    sim.state
        .state
        .tutorial
        .tips
        .get(tip.as_str())
        .map_or(0.0, |t| t.alpha)
}

// port: tutorial_show
/// Show a tip that has not been shown yet; `timer` frames until it is done (-1: until hidden).
pub fn show(sim: &mut Sim, tip: Tip, anchor: Anchor, x: f64, y: f64, timer: f64) {
    let t = tip_mut(sim, tip);
    if t.state != TipState::Hidden.code() {
        return;
    }
    t.state = TipState::Shown.code();
    t.anchor = anchor.as_str().to_string();
    t.x = x;
    t.y = y;
    t.timer = timer;
    sim.emit(SimEvent::TutorialTip {
        key: tip.as_str().to_string(),
    });
}

// port: tutorial_hide
/// A showing tip is done (it fades out).
pub fn hide(sim: &mut Sim, tip: Tip) {
    let t = tip_mut(sim, tip);
    if t.state == TipState::Shown.code() {
        t.state = TipState::Done.code();
    }
}

// port: tutorial_show_nearest
/// After the first node: point the salt tip at the nearest uncollected food and the water tip
/// at the nearest unconnected lake within two chunks (or at the node when there is none).
pub fn show_nearest(sim: &mut Sim, px: f64, py: f64) {
    let r = 2;
    let mut food: Option<(f64, f64)> = None;
    let mut food_d = f64::INFINITY;
    let mut lake: Option<(f64, f64, f64)> = None;
    let mut lake_d = f64::INFINITY;
    let c = (px / sim.world.params.chunk_w as f64).floor() as i64;
    for ci in c - r..=c + r {
        // world_get_chunk: a missing chunk is generated, as in the original
        let ch = sim.world.get_chunk(ci);
        for f in &ch.foods {
            let d = gml::point_distance(px, py, f.x, f.y);
            if !f.collected && d < food_d {
                food_d = d;
                food = Some((f.x, f.y));
            }
        }
        for l in &ch.lakes {
            let lx = l.body_x + l.body_w * 0.5;
            let ly = l.body_y + l.body_h * 0.5;
            let d = gml::point_distance(px, py, lx, ly);
            if !l.connected && d < lake_d {
                lake_d = d;
                lake = Some((l.body_x, l.body_w, l.body_y));
            }
        }
    }
    let oy = vis("tutorial_world_offset_y");
    if !sim.state.state.tutorial.ate {
        match food {
            None => show(sim, Tip::Salt, Anchor::World, px, py - oy, -1.0),
            Some((fx, fy)) => show(sim, Tip::Salt, Anchor::World, fx, fy - oy, -1.0),
        }
    }
    if !sim.state.state.tutorial.drank {
        match lake {
            None => show(sim, Tip::Water, Anchor::World, px, py + oy, -1.0),
            Some((bx, bw, by)) => {
                show(sim, Tip::Water, Anchor::World, bx + bw * 0.5, by - oy, -1.0)
            }
        }
    }
}

// port: tutorial_wrap
/// Break a long tip into two lines at the space nearest its middle.
pub fn tutorial_wrap(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if len as f64 <= vis("tutorial_wrap_chars") {
        return s.to_string();
    }
    let mid = len as f64 * 0.5;
    // 1-based index of the chosen space, like string_char_at
    let mut best: Option<usize> = None;
    for (i, &ch) in chars.iter().enumerate() {
        let pos = i + 1;
        if ch == ' ' && best.is_none_or(|b| (pos as f64 - mid).abs() < (b as f64 - mid).abs()) {
            best = Some(pos);
        }
    }
    let Some(best) = best else {
        return s.to_string();
    };
    let head: String = chars[..best - 1].iter().collect();
    let tail: String = chars[best..].iter().collect();
    format!("{head}\n{tail}")
}

// port: tutorial_state
pub fn tutorial_state(sim: &Sim, tip: Tip) -> TipState {
    sim.state
        .state
        .tutorial
        .tips
        .get(tip.as_str())
        .and_then(|t| TipState::from_code(t.state))
        .unwrap_or(TipState::Hidden)
}

/// `mushroom_type_unlocked(t)` for any type 0..7.
fn any_mushroom_unlocked(sim: &Sim) -> bool {
    crate::mushrooms::MushroomKind::ALL
        .iter()
        .any(|t| crate::mushrooms::mushroom_type_unlocked(&sim.state, *t))
}

fn event_xy(ev: &serde_json::Map<String, Value>, name: &str) -> Option<(f64, f64)> {
    let e = ev.get(name)?;
    let n = |k: &str| e.get(k).and_then(Value::as_f64).unwrap_or(0.0);
    Some((n("x"), n("y")))
}

// port: tutorial_step
/// One frame of the tutorial: consume the pending events, show and hide tips, run the tip
/// timers and fades. `raining` is `weather_is_raining()` (Task 12); `input` carries the
/// app's `menu_open` (upgrade tree), `debug_open` and `logo_open`.
pub fn step(sim: &mut Sim, input: &StepInput, raining: bool) {
    if sim.title_open || sim.intro_active {
        sim.state.state.tutorial.events.clear();
        return;
    }
    if sim.pause_open || input.debug_open || input.logo_open {
        return;
    }
    let ev: serde_json::Map<String, Value> = std::mem::take(&mut sim.state.state.tutorial.events)
        .into_iter()
        .collect();
    let fps = sim.fps;
    let tip_frames = vis("tutorial_tip_frames");
    let start_x = sim.state.state.start_x;
    show(
        sim,
        Tip::Shoot,
        Anchor::World,
        start_x,
        wld("surface_height_px") - vis("tutorial_shoot_offset_y"),
        -1.0,
    );
    if ev.contains_key("aim") {
        sim.state.state.tutorial.hand_done = true;
    }
    let shoot_alpha = tip_alpha(sim, Tip::Shoot);
    let hand_done = sim.state.state.tutorial.hand_done;
    let rt = &mut sim.progress.tutorial;
    if hand_done {
        rt.hand_alpha = (rt.hand_alpha - 1.0 / vis("tutorial_hand_fade_frames").max(1.0)).max(0.0);
    } else {
        rt.hand_alpha = shoot_alpha;
    }
    if rt.hand_alpha > 0.0 {
        rt.hand_frame += vis("tutorial_hand_fps") / fps;
    }
    if let Some((x, y)) = event_xy(&ev, "shot") {
        hide(sim, Tip::Shoot);
        show(
            sim,
            Tip::Node,
            Anchor::World,
            x,
            y + vis("tutorial_world_offset_y"),
            -1.0,
        );
    }
    if let Some((x, y)) = event_xy(&ev, "node") {
        hide(sim, Tip::Node);
        show_nearest(sim, x, y);
    }
    if ev.contains_key("food") {
        sim.state.state.tutorial.ate = true;
    }
    if ev.contains_key("water") {
        sim.state.state.tutorial.drank = true;
    }
    let (ate, drank) = (sim.state.state.tutorial.ate, sim.state.state.tutorial.drank);
    if ate {
        hide(sim, Tip::Salt);
    }
    let mut cam_show = false;
    if tutorial_state(sim, Tip::Salt) != TipState::Hidden
        || tutorial_state(sim, Tip::Water) != TipState::Hidden
    {
        cam_show = !(ate && drank);
    }
    let cam_fade = 1.0 / vis("tutorial_fade_frames").max(1.0);
    let rt = &mut sim.progress.tutorial;
    let step = if cam_show { cam_fade } else { -cam_fade };
    rt.camera_alpha = gml::clamp(rt.camera_alpha + step, 0.0, 1.0);
    rt.camera_frame += vis("tutorial_camera_fps") / fps;
    if drank {
        hide(sim, Tip::Water);
    }
    if mycelium_cm(sim) >= vis("tutorial_xp_cm") {
        show(
            sim,
            Tip::Xp,
            Anchor::Screen,
            0.5,
            0.22,
            vis("tutorial_short_frames"),
        );
    }
    if tutorial_state(sim, Tip::Xp) == TipState::Done && tip_alpha(sim, Tip::Xp) <= 0.0 {
        show(sim, Tip::Upgrade, Anchor::Upgrades, 0.0, 0.0, tip_frames);
    }
    if input.menu_open {
        hide(sim, Tip::Upgrade);
    } else if any_mushroom_unlocked(sim) {
        show(sim, Tip::Mushrooms, Anchor::Screen, 0.5, 0.22, tip_frames);
    }
    if ev.contains_key("mushroom") {
        hide(sim, Tip::Mushrooms);
    }
    if let Some((x, y)) = event_xy(&ev, "clay") {
        let timer = vis("tutorial_clay_seconds") * fps;
        show(
            sim,
            Tip::Clay,
            Anchor::World,
            x,
            y + vis("tutorial_world_offset_y"),
            timer,
        );
    }
    if raining {
        show(sim, Tip::Rain, Anchor::Screen, 0.5, 0.3, -1.0);
    } else {
        hide(sim, Tip::Rain);
    }
    let all_done = Tip::ORDER
        .into_iter()
        .filter(|t| *t != Tip::Final && *t != Tip::Clay)
        .all(|t| tutorial_state(sim, t) == TipState::Done && tip_alpha(sim, t) <= 0.0);
    if all_done {
        show(
            sim,
            Tip::Final,
            Anchor::Screen,
            0.5,
            0.4,
            vis("tutorial_final_frames"),
        );
    }
    let fade = 1.0 / vis("tutorial_fade_frames").max(1.0);
    for tip in Tip::ORDER {
        let t = tip_mut(sim, tip);
        if t.state == TipState::Shown.code() {
            if t.timer > 0.0 {
                t.timer -= 1.0;
                if t.timer <= 0.0 {
                    t.state = TipState::Done.code();
                }
            }
            t.alpha = (t.alpha + fade).min(1.0);
        } else {
            t.alpha = (t.alpha - fade).max(0.0);
        }
    }
}
