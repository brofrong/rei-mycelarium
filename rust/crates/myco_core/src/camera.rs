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
