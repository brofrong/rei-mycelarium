//! Camera and cut-scene state the logic depends on (ruling R12c). Task 21 ports
//! `camera_bounds`, `boar_shake` and the `cine_*` logic into these hooks; drawing and zoom
//! stay in the app.

use crate::sim::Sim;

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

/// `camera_bounds(vw, vh)`: the camera box (`state.cam_box`, grown by the mycelium's reach)
/// less the view, clipped to the world bottom; a box narrower than the view centres it.
/// (Task 21's row, ported here unmarked because the animals' world span needs it; Task 21
/// marks it.)
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

/// `boar_shake(vx, vw)` (Task 21): camera shake offset `[dx, dy]`. Until then: none.
pub fn boar_shake(_sim: &mut Sim, _vx: f64, _vw: f64) -> [f64; 2] {
    [0.0, 0.0]
}

/// `cine_step` (Task 21).
pub fn cine_step(_sim: &mut Sim) {}

/// `cine_blocking` (Task 21): a cut-scene holds the input. Until then: never.
pub fn cine_blocking(_sim: &Sim) -> bool {
    false
}

/// `cine_showing` (Task 21): a cut-scene is in its "show" phase (camera input frozen). Until
/// then: never.
pub fn cine_showing(_sim: &Sim) -> bool {
    false
}

/// `forest_fit_view` (Task 21, ruling R12b): re-centre the camera on a new view size. Called by
/// the forest step when the camera size differs from the logical view.
pub fn forest_fit_view(_sim: &mut Sim, _view_w: f64, _view_h: f64) {}

/// `cine_show_start` (Task 21): start the migration spore cut-scene (the tasks popup's
/// "spores" button). Until then: nothing.
pub fn cine_show_start(_sim: &mut Sim) {}

/// The cut-scene part of `tasks_popup_close` (Task 21): `if cine_active() and
/// tasks.cine.phase == "hold" then phase = "out"; t = 0`. Until then: nothing (no cut-scene
/// can be active).
pub fn cine_release_hold(_sim: &mut Sim) {}

/// `camera_jump_edge(dir)` (Task 21): the HUD's left/right camera buttons. Until then: nothing.
pub fn camera_jump_edge(_sim: &mut Sim, _dir: f64) {}

/// `camera_jump_origin()` (Task 21): the HUD's origin button. Until then: nothing.
pub fn camera_jump_origin(_sim: &mut Sim) {}

// ---------------------------------------------------------------------------------------------
// The intro (scr_life `intro_*`; Task 21's rows, ported here unmarked because `game_init` starts
// every game with `intro_active` and obj_life Step_0 runs `intro_step`; Task 21 marks them and
// adds `intro_draw`).

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

/// `intro_init()`: the hedgehog starts half a screen and a frame left of the start.
pub fn intro_init(sim: &mut Sim) {
    use crate::balance::{vis, wld};
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

/// `intro_step()`: true while the intro holds the rest of obj_life's step. The hedgehog walks to
/// the start and drops the origin mushroom; when it lands, `intro_finish`.
pub fn intro_step(sim: &mut Sim) -> bool {
    use crate::balance::{vis, wld};
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

/// `intro_finish()`: the intro ends; its hedgehog walks on as a normal one and the origin knot
/// is placed at the mycelium controller.
pub fn intro_finish(sim: &mut Sim) {
    use crate::balance::vis;
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
