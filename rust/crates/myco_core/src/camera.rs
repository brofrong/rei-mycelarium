//! Camera state the logic depends on (ruling R12c): the 1× game camera, its bounds, the HUD's
//! camera jumps, the fly-to-marker, the shake and the intro. The cut-scene logic is in
//! [`crate::cine`] (re-exported here for the callers); drawing and zoom stay in the app.

use crate::balance::vis;
use crate::gml;
use crate::sim::Sim;

pub use crate::cine::{
    cine_active, cine_blocking, cine_release_hold, cine_show_start, cine_showing, cine_step,
};

/// The game camera (`obj_forest_controller.camera_*`), logical 1× view in world pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraRt {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub width: f64,
    pub height: f64,
    /// `camera_view_x/y`: the drawn top-left corner (floored, shake applied; Task 5's forest
    /// step writes it)
    pub view_x: f64,
    pub view_y: f64,
    /// this step's `boar_shake` offset (`view_x = floor(clamp(x + shake[0]))`); the app's zoomed
    /// render camera uses the unfloored position
    pub shake: [f64; 2],
    /// `global.camera_fly`: easing toward an animal marker (Task 21's `camera_fly_to_marker`
    /// starts it and keeps the target in sync with the marker; Task 5's forest step runs it)
    pub fly: Option<CameraFly>,
}

/// `global.camera_fly` (`___struct___113`): seconds flown and the marker position.
#[derive(Clone, Debug, PartialEq)]
pub struct CameraFly {
    pub t: f64,
    pub x: f64,
    pub y: f64,
}

impl Default for CameraRt {
    fn default() -> Self {
        CameraRt {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            width: 1920.0,
            height: 1080.0,
            view_x: 0.0,
            view_y: 0.0,
            shake: [0.0, 0.0],
            fly: None,
        }
    }
}

/// `camera_bounds` result: allowed range of the camera's top-left corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CamBounds {
    pub x1: f64,
    pub x2: f64,
    pub y1: f64,
    pub y2: f64,
}

// port: camera_bounds
/// `camera_bounds(vw, vh)`: the camera box (`state.cam_box`, grown by the mycelium's reach)
/// less the view, clipped to the world bottom; a box narrower than the view centres it.
pub fn camera_bounds(sim: &Sim, vw: f64, vh: f64) -> CamBounds {
    let [bx1, by1, bx2, by2] = sim.state.state.cam_box;
    let mut x1 = bx1;
    let mut x2 = bx2 - vw;
    let y1 = by1;
    let mut y2 = by2.min(sim.world.params.bottom_y) - vh;
    if x2 < x1 {
        x1 = (bx1 + bx2) * 0.5 - vw * 0.5;
        x2 = x1;
    }
    if y2 < y1 {
        y2 = y1;
    }
    CamBounds { x1, x2, y1, y2 }
}

// port: camera_jump_to
/// Centre the camera on `(x, y)` within the bounds and stop it.
pub fn camera_jump_to(sim: &mut Sim, x: f64, y: f64) {
    let (w, h) = (sim.camera.width, sim.camera.height);
    let b = camera_bounds(sim, w, h);
    let cam = &mut sim.camera;
    cam.x = gml::clamp(x - w * 0.5, b.x1, b.x2);
    cam.y = gml::clamp(y - h * 0.5, b.y1, b.y2);
    cam.vx = 0.0;
    cam.vy = 0.0;
}

/// The render zoom (Rust only: the original has no zoom) keeps the world point under the cursor
/// fixed by moving the logic camera: top-left to `(x, y)` within `camera_bounds`, the view on
/// whole pixels of the shaken position (as the forest step floors it). Sent as
/// `Command::CameraPan` and applied inside the step.
pub fn camera_pan_to(sim: &mut Sim, x: f64, y: f64) {
    let (w, h) = (sim.camera.width, sim.camera.height);
    let b = camera_bounds(sim, w, h);
    let cam = &mut sim.camera;
    cam.x = gml::clamp(x, b.x1, b.x2);
    cam.y = gml::clamp(y, b.y1, b.y2);
    cam.view_x = (cam.x + cam.shake[0]).floor();
    cam.view_y = (cam.y + cam.shake[1]).floor();
}

// port: camera_jump_origin
/// The HUD's origin button: centre on the mycelium controller, top of the bounds, stopped.
pub fn camera_jump_origin(sim: &mut Sim) {
    let ox = sim.mycelium.x;
    let (w, h) = (sim.camera.width, sim.camera.height);
    let b = camera_bounds(sim, w, h);
    let cam = &mut sim.camera;
    cam.x = gml::clamp(ox - w * 0.5, b.x1, b.x2);
    cam.y = gml::clamp(0.0, b.y1, b.y2);
    cam.vx = 0.0;
    cam.vy = 0.0;
}

// port: camera_jump_edge
/// The HUD's left (-1) / right (1) buttons: jump to the mycelium's extreme tip on that side, or
/// to the origin without growths.
pub fn camera_jump_edge(sim: &mut Sim, dir: f64) {
    match crate::mycelium::mycelium_extreme_tip(sim, dir) {
        Some((x, y)) => camera_jump_to(sim, x, y),
        None => camera_jump_origin(sim),
    }
}

// port: camera_fly_to_marker
/// `global.camera_fly = { marker, t = 0 }`: ease the camera toward a clicked animal marker
/// (the forest step flies it). The marker lives in the app; [`camera_fly_track`] keeps the
/// target on it while it moves.
pub fn camera_fly_to_marker(sim: &mut Sim, x: f64, y: f64) {
    sim.camera.fly = Some(CameraFly { t: 0.0, x, y });
}

/// The flown marker moved (the original reads `fly.marker.x/y` live every step): update the
/// target of a running fly; nothing when none runs.
pub fn camera_fly_track(sim: &mut Sim, x: f64, y: f64) {
    if let Some(f) = sim.camera.fly.as_mut() {
        f.x = x;
        f.y = y;
    }
}

// port: boar_shake
/// Camera shake offset `[dx, dy]` for the view `[vx, vx + vw]`: the cut-scene's shake while it
/// tours; otherwise none while the UI blocks; otherwise the decaying repeller kick (random, so it
/// draws from the game RNG every step) plus a rumble from the nearest boar, fading over
/// `boar_shake_falloff_px` outside the view. `ui_blocking` is `ui_blocking()` (its menu / debug
/// / logo / popup parts come from the app's `StepInput`).
pub fn boar_shake(sim: &mut Sim, vx: f64, vw: f64, ui_blocking: bool) -> [f64; 2] {
    if let Some(c) = crate::cine::cine_shake(sim) {
        return c;
    }
    if ui_blocking {
        return [0.0, 0.0];
    }
    let k = sim.mushrooms.repel_kick;
    sim.mushrooms.repel_kick = k * 0.82;
    if sim.mushrooms.repel_kick < 0.2 {
        sim.mushrooms.repel_kick = 0.0;
    }
    let bx = k * sim.rng.random_range(-0.5, 0.5);
    let by = k * sim.rng.random_range(-1.0, 1.0);
    let base = [bx, by];
    if sim.life.boars.is_empty() {
        return base;
    }
    let mut best: f64 = 0.0;
    let fall = vis("boar_shake_falloff_px").max(1.0);
    for b in &sim.life.boars {
        let d = 0f64.max(vx - b.x).max(b.x - (vx + vw));
        best = best.max(1.0 - d / fall);
    }
    if best <= 0.0 {
        return base;
    }
    let amp = vis("boar_shake_px") * best * best;
    let t = sim.now_ms();
    [
        base[0] + amp * 0.45 * (gml::sin(t * 0.031) * 0.6 + gml::sin(t * 0.057 + 1.3) * 0.4),
        base[1] + amp * (gml::sin(t * 0.043 + 0.7) * 0.6 + gml::sin(t * 0.089 + 2.1) * 0.4),
    ]
}

// port: anon__799__gml_Object_obj_forest_controller_Create_0
/// `forest_fit_view` (ruling R12b): the view size changed (window resize, fullscreen); keep the
/// camera centre and take the new size. (Its `camera_set_view_size` / `view_set_wport/hport`
/// half is the app's: `Cameras::set_window` sizes the Bevy world camera and viewport.)
pub fn forest_fit_view(sim: &mut Sim, view_w: f64, view_h: f64) {
    let cam = &mut sim.camera;
    cam.x += (cam.width - view_w) * 0.5;
    cam.y += (cam.height - view_h) * 0.5;
    cam.width = view_w;
    cam.height = view_h;
}

// ---------------------------------------------------------------------------------------------
// The intro (scr_life `intro_*`; ported by Task 10 for `game_init`, which starts every game with
// `intro_active`, and obj_life Step_0, which runs `intro_step`). `intro_draw` is the app's.

/// `global.intro` (`___struct___105`): the hedgehog that brings the origin mushroom.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Intro {
    pub x: f64,
    pub y: f64,
    pub frame: f64,
    pub frame_timer: f64,
    pub dropped: bool,
    pub shroom_y: f64,
    pub shroom_speed: f64,
}

// port: intro_init
/// `intro_init()`: the hedgehog starts half a screen and a frame left of the start.
pub fn intro_init(sim: &mut Sim) {
    use crate::balance::wld;
    sim.life.intro = Intro {
        x: sim.state.state.start_x - vis("screen_width_px") * 0.5 - crate::life::hedgehog_frame_w(),
        y: wld("surface_height_px"),
        frame: 0.0,
        frame_timer: vis("hedgehog_frame_interval"),
        dropped: false,
        shroom_y: 0.0,
        shroom_speed: 0.0,
    };
}

// port: intro_step
/// `intro_step()`: true while the intro holds the rest of obj_life's step. The hedgehog walks to
/// the start and drops the origin mushroom; when it lands, `intro_finish`.
pub fn intro_step(sim: &mut Sim) -> bool {
    use crate::balance::wld;
    if !sim.intro_active {
        return false;
    }
    if sim.pause_open || sim.title_open {
        return true;
    }
    let start_x = sim.state.state.start_x;
    let floor_y = wld("surface_height_px");
    let spd = vis("intro_hedgehog_speed");
    let it = &mut sim.life.intro;
    it.frame_timer -= 1.0;
    if it.frame_timer <= 0.0 {
        it.frame_timer = vis("hedgehog_frame_interval");
        it.frame = crate::gml::gmod(it.frame + 1.0, 8.0);
    }
    it.x += spd;
    if !it.dropped {
        if it.x >= start_x {
            it.dropped = true;
            it.shroom_y = floor_y + crate::life::hedgehog::hedgehog_back_y();
            it.shroom_speed = 0.0;
        }
        true
    } else {
        it.shroom_speed = (it.shroom_speed + vis("intro_drop_gravity")).min(16.0);
        it.shroom_y += it.shroom_speed;
        if it.shroom_y < floor_y {
            return true;
        }
        intro_finish(sim);
        false
    }
}

// port: intro_finish
/// `intro_finish()`: the intro ends; its hedgehog walks on as a normal one and the origin knot
/// is placed at the mycelium controller.
pub fn intro_finish(sim: &mut Sim) {
    sim.intro_active = false;
    let it = sim.life.intro.clone();
    let id = sim.life.new_id(crate::mycelium::source::ANIMAL);
    sim.life.animals.push(crate::life::Hedgehog {
        id,
        x: it.x,
        y: it.y,
        dir: 1.0,
        speed: vis("intro_hedgehog_speed"),
        frame: it.frame,
        frame_timer: it.frame_timer,
        carrying: -1,
    });
    let (x, y) = (sim.mycelium.x, sim.mycelium.y);
    crate::mycelium::knots::add_knot(sim, x, y);
}
