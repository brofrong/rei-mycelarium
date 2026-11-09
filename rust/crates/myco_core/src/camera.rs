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

/// `camera_bounds(vw, vh)` (Task 21). Until then: unbounded.
pub fn camera_bounds(_sim: &Sim, _vw: f64, _vh: f64) -> CamBounds {
    CamBounds {
        x1: f64::NEG_INFINITY,
        x2: f64::INFINITY,
        y1: f64::NEG_INFINITY,
        y2: f64::INFINITY,
    }
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
