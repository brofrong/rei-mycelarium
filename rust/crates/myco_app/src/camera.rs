//! The 1× game view the logic sees, the zoomed render camera, and screen ↔ world/GUI mapping.
//!
//! Coordinate spaces:
//! - **screen**: logical window pixels, origin top-left (what `Window::cursor_position` returns);
//!   physical pixels are `screen × scale_factor`;
//! - **game**: the application surface in view units (`view_w × view_h`), which is also the GUI
//!   space (`display_set_gui_size(view_w, view_h)` in the original; the 1920-wide UI sits at
//!   `ui_ox`, bottom-anchored with `ui_by`);
//! - **world**: GameMaker room pixels, y down. Bevy draws world point `(x, y)` at `(x, -y)`.
//!
//! The picture occupies [`Cameras::picture`] (the `display_math.fit` rectangle inside the safe
//! area, rounded to whole pixels so the Bevy viewport and the mouse mapping agree exactly).
//! Zoom scales the world around the centre of `game_view` ([`Cameras::render_center`]); logic
//! always gets `game_view` (1×, `camera_view_x/y`: floored, shake applied).
//!
//! **Zoom** (Task 21, an addition to the original): the mouse wheel and the trackpad pinch move
//! [`ZoomState::target`]; [`zoom_advance`] eases the zoom toward it (exponential, 0.12 s) and
//! moves the logic camera so the world point under the cursor stays put. Limits: 2× in; out to
//! the explored colony's bounding box plus 256 px, but 1× is always allowed (Review Focus 4).
//! The render camera stays between the sky top (`surface_y − view_h`) and the world bottom.
//! The intro, the cut-scenes and the title lock the zoom to 1×.

use crate::display::{self, Fit, SafeArea, ViewLayout};
use bevy::camera::{ScalingMode, Viewport};
use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::math::DVec2;
use bevy::prelude::*;
use myco_core::camera::CameraRt;
use myco_core::sim::{Sim, ViewRect};

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Cameras {
    /// 1× game view in world pixels; logic moves its centre (`StepInput.view`)
    pub game_view: ViewRect,
    /// render zoom: 2 = twice as close, 0.5 = twice as far
    pub zoom: f64,
    pub scale_factor: f64,
    /// window size, physical pixels
    pub window_px: DVec2,
    pub safe: SafeArea,
    pub layout: ViewLayout,
    /// picture rectangle in physical window pixels (rounded fit)
    pub picture: Fit,
    /// sub-pixel part of the logic camera's shaken position (`x + shake − view_x`): the zoomed
    /// render camera adds it (at 1× the world is drawn on whole pixels, like `camera_view_x`)
    pub sub: DVec2,
    /// world y range the render camera stays in: (sky top `surface_y − view_h`, world bottom)
    pub y_limits: Option<(f64, f64)>,
}

impl Default for Cameras {
    fn default() -> Self {
        Cameras::new(
            1920.0,
            1080.0,
            1.0,
            SafeArea {
                x: 0.0,
                y: 0.0,
                w: 1920.0,
                h: 1080.0,
            },
        )
    }
}

impl Cameras {
    pub fn new(win_w: f64, win_h: f64, scale_factor: f64, safe: SafeArea) -> Cameras {
        let mut c = Cameras {
            game_view: ViewRect {
                x: 0.0,
                y: 0.0,
                w: 1920.0,
                h: 1080.0,
            },
            zoom: 1.0,
            scale_factor: 1.0,
            window_px: DVec2::ZERO,
            safe,
            layout: ViewLayout::default(),
            picture: Fit::default(),
            sub: DVec2::ZERO,
            y_limits: None,
        };
        c.set_window(win_w, win_h, scale_factor, safe);
        c
    }

    /// Recompute layout and picture for a window (resize, fullscreen switch, DPI change).
    /// Keeps the centre of `game_view`; its size follows the new layout.
    pub fn set_window(&mut self, win_w: f64, win_h: f64, scale_factor: f64, safe: SafeArea) {
        self.window_px = DVec2::new(win_w, win_h);
        self.scale_factor = if scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        self.safe = safe;
        // display.window_size() is the safe area's size
        if let Some(layout) = display::view_for_window(safe.w.round(), safe.h.round()) {
            self.layout = layout;
        }
        let f = display::fit(
            win_w,
            win_h,
            safe.x,
            safe.y,
            safe.w,
            safe.h,
            self.layout.surf_w,
            self.layout.surf_h,
        );
        self.picture = if f.scale == 0.0 {
            f
        } else {
            let x = f.x.round();
            let y = f.y.round();
            Fit {
                x,
                y,
                w: (f.x + f.w).round() - x,
                h: (f.y + f.h).round() - y,
                scale: f.scale,
            }
        };
        let (cx, cy) = self.game_view_center();
        self.game_view.w = self.layout.view_w;
        self.game_view.h = self.layout.view_h;
        self.set_game_view_center(cx, cy);
    }

    pub fn game_view_center(&self) -> (f64, f64) {
        (
            self.game_view.x + self.game_view.w / 2.0,
            self.game_view.y + self.game_view.h / 2.0,
        )
    }

    pub fn set_game_view_center(&mut self, cx: f64, cy: f64) {
        self.game_view.x = cx - self.game_view.w / 2.0;
        self.game_view.y = cy - self.game_view.h / 2.0;
    }

    /// The 1× game view follows the logic camera (`camera_view_x/y`, after each step).
    pub fn sync_game_view(&mut self, cam: &CameraRt) {
        self.game_view.x = cam.view_x;
        self.game_view.y = cam.view_y;
        let fx = cam.x + cam.shake[0];
        let fy = cam.y + cam.shake[1];
        // the shaken position was clamped to the bounds: no sub-pixel part then
        self.sub = DVec2::new(
            if fx.floor() == cam.view_x {
                fx - cam.view_x
            } else {
                0.0
            },
            if fy.floor() == cam.view_y {
                fy - cam.view_y
            } else {
                0.0
            },
        );
    }

    /// Centre of the render camera: the game view's centre (plus the sub-pixel part when
    /// zoomed), kept vertically inside [`y_limits`](Self::y_limits).
    pub fn render_center(&self) -> (f64, f64) {
        let (mut cx, mut cy) = self.game_view_center();
        if self.zoom != 1.0 {
            cx += self.sub.x;
            cy += self.sub.y;
        }
        if let Some((top, bottom)) = self.y_limits {
            let h = self.game_view.h / self.zoom;
            cy = if h >= bottom - top {
                (top + bottom) / 2.0
            } else {
                cy.clamp(top + h / 2.0, bottom - h / 2.0)
            };
        }
        (cx, cy)
    }

    /// The world rectangle the render camera shows (game view scaled by 1/zoom about the render
    /// centre).
    pub fn render_view(&self) -> ViewRect {
        let (cx, cy) = self.render_center();
        let (w, h) = (self.game_view.w / self.zoom, self.game_view.h / self.zoom);
        ViewRect {
            x: cx - w / 2.0,
            y: cy - h / 2.0,
            w,
            h,
        }
    }

    fn degenerate(&self) -> bool {
        self.picture.w <= 0.0 || self.picture.h <= 0.0
    }

    /// Screen (logical px) → GUI / game view units (`display.mouse_gui`). A degenerate window
    /// maps everything to (-1, -1) like `screen_to_game`.
    pub fn screen_to_gui(&self, p: Vec2) -> DVec2 {
        if self.degenerate() {
            return DVec2::new(-1.0, -1.0);
        }
        let px = p.as_dvec2() * self.scale_factor;
        DVec2::new(
            (px.x - self.picture.x) / self.picture.w * self.layout.view_w,
            (px.y - self.picture.y) / self.picture.h * self.layout.view_h,
        )
    }

    /// GUI units → screen (logical px).
    pub fn gui_to_screen(&self, g: DVec2) -> DVec2 {
        DVec2::new(
            self.picture.x + g.x / self.layout.view_w * self.picture.w,
            self.picture.y + g.y / self.layout.view_h * self.picture.h,
        ) / self.scale_factor
    }

    /// Screen (logical px) → world px through the zoomed render camera.
    pub fn screen_to_world(&self, p: Vec2) -> DVec2 {
        self.gui_to_world(self.screen_to_gui(p))
    }

    /// World px → screen (logical px); inverse of [`screen_to_world`](Self::screen_to_world).
    pub fn world_to_screen(&self, w: DVec2) -> DVec2 {
        self.gui_to_screen(self.world_to_gui(w))
    }

    pub fn gui_to_world(&self, g: DVec2) -> DVec2 {
        let (cx, cy) = self.render_center();
        DVec2::new(
            cx + (g.x - self.layout.view_w / 2.0) * self.game_view.w
                / self.layout.view_w
                / self.zoom,
            cy + (g.y - self.layout.view_h / 2.0) * self.game_view.h
                / self.layout.view_h
                / self.zoom,
        )
    }

    pub fn world_to_gui(&self, w: DVec2) -> DVec2 {
        let (cx, cy) = self.render_center();
        DVec2::new(
            self.layout.view_w / 2.0
                + (w.x - cx) * self.zoom * self.layout.view_w / self.game_view.w,
            self.layout.view_h / 2.0
                + (w.y - cy) * self.zoom * self.layout.view_h / self.game_view.h,
        )
    }

    /// Bevy viewport of the picture (both cameras draw only there).
    pub fn viewport(&self) -> Option<Viewport> {
        if self.degenerate() {
            return None;
        }
        let max = self.window_px.max(DVec2::ONE);
        let x = self.picture.x.clamp(0.0, max.x - 1.0);
        let y = self.picture.y.clamp(0.0, max.y - 1.0);
        Some(Viewport {
            physical_position: UVec2::new(x as u32, y as u32),
            physical_size: UVec2::new(
                self.picture.w.min(max.x - x).max(1.0) as u32,
                self.picture.h.min(max.y - y).max(1.0) as u32,
            ),
            ..default()
        })
    }

    /// Orthographic projection of the world camera: the render view's size.
    pub fn world_projection(&self) -> OrthographicProjection {
        let v = self.render_view();
        OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: v.w as f32,
                height: v.h as f32,
            },
            ..OrthographicProjection::default_2d()
        }
    }

    /// World camera transform: the render centre, y flipped.
    pub fn world_transform(&self) -> Transform {
        let (cx, cy) = self.render_center();
        Transform::from_xyz(cx as f32, -cy as f32, 0.0)
    }

    /// GUI camera: `view_w × view_h` units, GUI point (x, y) drawn at Bevy (x, -y).
    pub fn gui_projection(&self) -> OrthographicProjection {
        OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: self.layout.view_w as f32,
                height: self.layout.view_h as f32,
            },
            ..OrthographicProjection::default_2d()
        }
    }

    pub fn gui_transform(&self) -> Transform {
        Transform::from_xyz(
            (self.layout.view_w / 2.0) as f32,
            -(self.layout.view_h / 2.0) as f32,
            0.0,
        )
    }
}

/// World point in Bevy coordinates (y up).
pub fn world_to_bevy(x: f64, y: f64) -> Vec2 {
    Vec2::new(x as f32, -y as f32)
}

/// Blend space of every camera (R21b): GameMaker and LÖVE blend gamma-encoded sRGB values, so
/// the cameras composite in sRGB space (Bevy 0.20 `CompositingSpace::Srgb`: an `Rgba8Unorm` main
/// texture, the 2D shaders encode their output with `linear_to_srgb` under `SRGB_OUTPUT`).
/// All cameras share the window target and must use the same space. Custom 2D fragment shaders
/// must honour `SRGB_OUTPUT` the same way (Bevy's mesh2d / sprite / colour-material shaders do).
pub const COMPOSITING: bevy::camera::CompositingSpace = bevy::camera::CompositingSpace::Srgb;

/// Render layer of everything drawn by the GUI camera.
pub const GUI_LAYER: usize = 1;

#[derive(Component)]
pub struct WorldCamera;

/// Task 15's BLACK METAL post effect: while `Some`, the world and GUI cameras render into this
/// offscreen frame (full image, no viewport) instead of the window; `render::post` draws it.
#[derive(Resource, Clone, Debug, Default)]
pub struct PostRedirect(pub Option<Handle<Image>>);

#[derive(Component)]
pub struct GuiCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cameras>()
            .init_resource::<ZoomState>()
            .add_systems(Startup, spawn_cameras)
            .add_systems(First, track_window)
            .add_systems(
                PreUpdate,
                zoom_system.after(crate::sim_bridge::collect_input),
            )
            .add_systems(
                PostUpdate,
                apply_cameras
                    .before(TransformSystems::Propagate)
                    .before(bevy::camera::CameraUpdateSystems),
            );
    }
}

// port: gml_Object_obj_forest_controller_CleanUp_0
/// The cameras (obj_forest_controller Create_0's `forest_camera` is the world camera here). Its
/// CleanUp (`camera_destroy(forest_camera)` when the room ends) has nothing to free: the room
/// lasts as long as the app (a `game_restart` rebuilds the `Sim`, not the cameras), and Bevy
/// despawns the camera entities with the world.
fn spawn_cameras(mut commands: Commands, cams: Res<Cameras>) {
    use bevy::camera::visibility::RenderLayers;
    // clears the whole window (bars around the picture) and draws nothing
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderLayers::none(),
        COMPOSITING,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            viewport: cams.viewport(),
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Projection::Orthographic(cams.world_projection()),
        cams.world_transform(),
        COMPOSITING,
        WorldCamera,
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            viewport: cams.viewport(),
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Projection::Orthographic(cams.gui_projection()),
        cams.gui_transform(),
        RenderLayers::layer(GUI_LAYER),
        COMPOSITING,
        GuiCamera,
    ));
}

/// Window size, DPI and safe area → `Cameras` (every frame; cheap, and catches resizes,
/// fullscreen switches and display changes).
fn track_window(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    screen: Option<Res<display::ScreenInfo>>,
    mut cams: ResMut<Cameras>,
) {
    let Ok(win) = windows.single() else { return };
    let (w, h) = (win.physical_width() as f64, win.physical_height() as f64);
    let sf = win.scale_factor() as f64;
    let safe = display::safe_area_in_window(w, h, screen.as_deref());
    if cams.window_px != DVec2::new(w, h) || cams.scale_factor != sf || cams.safe != safe {
        cams.set_window(w, h, sf, safe);
    }
}

type CamParts = (
    &'static mut Camera,
    &'static mut Projection,
    &'static mut Transform,
);

fn apply_cameras(
    mut commands: Commands,
    cams: Res<Cameras>,
    redirect: Option<Res<PostRedirect>>,
    mut applied: Local<Option<AssetId<Image>>>,
    mut world: Query<(Entity, CamParts), (With<WorldCamera>, Without<GuiCamera>)>,
    mut gui: Query<(Entity, CamParts), (With<GuiCamera>, Without<WorldCamera>)>,
) {
    use bevy::camera::CameraProjection;
    let target = redirect.and_then(|r| r.0.clone());
    let vp = if target.is_some() {
        None
    } else {
        cams.viewport()
    };
    let size = match (&target, &vp) {
        (Some(_), _) => Vec2::new(
            cams.picture.w.floor().max(1.0) as f32,
            cams.picture.h.floor().max(1.0) as f32,
        ),
        (None, Some(v)) => v.physical_size.as_vec2(),
        (None, None) => Vec2::ONE,
    };
    // render target: the window, or the post frame (written only when it changes)
    if *applied != target.as_ref().map(|t| t.id()) {
        let rt = match &target {
            Some(img) => bevy::camera::RenderTarget::Image(img.clone().into()),
            None => bevy::camera::RenderTarget::Window(bevy::window::WindowRef::Primary),
        };
        for (e, _) in world.iter().chain(gui.iter()) {
            commands.entity(e).insert(rt.clone());
        }
        *applied = target.as_ref().map(|t| t.id());
    }
    // The projection is written with its `area` already computed (`update`), and only when it
    // changes: a fresh `OrthographicProjection` has the unit area until Bevy's `camera_system`
    // updates it, and a frustum built from that culls every sprite (R21a, see task-13-report).
    let fit = |mut p: OrthographicProjection| {
        p.update(size.x, size.y);
        Projection::Orthographic(p)
    };
    let apply = |cam: &mut Mut<Camera>, proj: &mut Mut<Projection>, p: Projection| {
        let key = |v: &Option<Viewport>| v.as_ref().map(|v| (v.physical_position, v.physical_size));
        if key(&cam.viewport) != key(&vp) {
            cam.viewport = vp.clone();
        }
        // Projection has no PartialEq; its Debug form covers every field
        if format!("{:?}", **proj) != format!("{p:?}") {
            **proj = p;
        }
    };
    for (_, (mut cam, mut proj, mut tf)) in &mut world {
        apply(&mut cam, &mut proj, fit(cams.world_projection()));
        tf.set_if_neq(cams.world_transform());
    }
    for (_, (mut cam, mut proj, mut tf)) in &mut gui {
        apply(&mut cam, &mut proj, fit(cams.gui_projection()));
        tf.set_if_neq(cams.gui_transform());
    }
}

// ---------------------------------------------------------------------------------------------
// Zoom (Task 21)

/// Closest zoom: twice as near.
pub const ZOOM_MAX: f64 = 2.0;
/// Time constant of the zoom's exponential easing.
pub const ZOOM_SMOOTH_SECONDS: f64 = 0.12;
/// Margin around the explored colony at the farthest zoom.
pub const ZOOM_PAD_PX: f64 = 256.0;
/// Zoom factor per wheel notch.
pub const ZOOM_WHEEL_STEP: f64 = 1.15;

/// The zoom the render camera eases toward.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct ZoomState {
    pub target: f64,
}

impl Default for ZoomState {
    fn default() -> Self {
        ZoomState { target: 1.0 }
    }
}

/// World y range of the render camera: from the sky top (`surface_y − view_h`, but not above
/// y = 0, where the forest backdrop starts and the 1× camera never goes) to the world bottom.
pub fn render_y_limits(sim: &Sim, view_h: f64) -> (f64, f64) {
    let p = &sim.world.params;
    ((p.surface_y - view_h).max(0.0), p.bottom_y)
}

/// The zoom is held at 1×: the intro, a cut-scene (until it is gone) and the title.
pub fn zoom_locked(sim: &Sim) -> bool {
    sim.intro_active || myco_core::cine::cine_active(sim) || sim.title_open
}

/// Bounding box `[x1, y1, x2, y2]` of what the colony has explored: hyphae, knots, mushrooms.
pub fn explored_bbox(sim: &Sim) -> Option<[f64; 4]> {
    let mut bb: Option<[f64; 4]> = None;
    let mut add = |x1: f64, y1: f64, x2: f64, y2: f64| {
        if !(x1.is_finite() && y1.is_finite() && x2.is_finite() && y2.is_finite()) {
            return;
        }
        bb = Some(match bb {
            None => [x1, y1, x2, y2],
            Some(b) => [b[0].min(x1), b[1].min(y1), b[2].max(x2), b[3].max(y2)],
        });
    };
    for g in &sim.mycelium.growths {
        for b in &g.branches {
            if !b.points.is_empty() {
                add(b.bx1, b.by1, b.bx2, b.by2);
            }
        }
    }
    for k in &sim.mycelium.knots {
        add(k.x, k.y, k.x, k.y);
    }
    for m in &sim.state.mushrooms {
        add(m.x, m.y, m.x, m.y);
    }
    bb
}

/// `(farthest, closest)` zoom: out until the explored bounding box plus [`ZOOM_PAD_PX`] fits the
/// screen, but never past 1× in (1× is always allowed) and never so far that the view is taller
/// than the sky top .. world bottom range; in to [`ZOOM_MAX`].
pub fn zoom_limits(sim: &Sim, cams: &Cameras) -> (f64, f64) {
    let (vw, vh) = (cams.game_view.w, cams.game_view.h);
    let mut lo: f64 = 1.0;
    if let Some([x1, y1, x2, y2]) = explored_bbox(sim) {
        let fit = (vw / (x2 - x1 + 2.0 * ZOOM_PAD_PX)).min(vh / (y2 - y1 + 2.0 * ZOOM_PAD_PX));
        lo = lo.min(fit);
    }
    let (top, bottom) = render_y_limits(sim, vh);
    let range = bottom - top;
    if range > 0.0 {
        lo = lo.max((vh / range).min(1.0));
    }
    (lo, ZOOM_MAX)
}

/// A wheel notch / pinch: multiply the target zoom (limits apply in [`zoom_advance`]).
pub fn zoom_wheel(zs: &mut ZoomState, factor: f64) {
    if factor.is_finite() && factor > 0.0 {
        zs.target *= factor;
    }
}

/// One frame of zoom: ease toward the target (clamped to [`zoom_limits`]) and keep the world
/// point under `anchor` (screen, logical px) there. The render side (`cams`: zoom, 1× game view,
/// sub-pixel part) changes at once; the logic camera does not: the returned top-left is sent as
/// `Command::CameraPan` and applied inside the next step, so the logic camera moves only in
/// steps (deterministic, replayable). `base` is the logic camera's top-left this pan builds on
/// (a pan still waiting for its step, else `sim.camera`). The pan stays inside
/// `camera_bounds`, and the 1× game view keeps the render camera's centre (except where the
/// render camera is held below the sky top / above the world bottom).
pub fn zoom_advance(
    cams: &mut Cameras,
    sim: &Sim,
    zs: &mut ZoomState,
    dt: f64,
    anchor: Vec2,
    base: (f64, f64),
) -> Option<(f64, f64)> {
    cams.y_limits = Some(render_y_limits(sim, cams.game_view.h));
    if zoom_locked(sim) {
        cams.zoom = 1.0;
        zs.target = 1.0;
        return None;
    }
    let (lo, hi) = zoom_limits(sim, cams);
    zs.target = zs.target.clamp(lo, hi);
    let z0 = cams.zoom;
    let (l0, lt) = (z0.ln(), zs.target.ln());
    let z1 = if (lt - l0).abs() < 1e-4 {
        zs.target
    } else {
        (l0 + (lt - l0) * (1.0 - (-dt / ZOOM_SMOOTH_SECONDS).exp())).exp()
    };
    if z1 == z0 {
        return None;
    }
    let before = cams.screen_to_world(anchor);
    cams.zoom = z1;
    let after = cams.screen_to_world(anchor);
    let d = before - after;
    let (w, h) = (sim.camera.width, sim.camera.height);
    let b = myco_core::camera::camera_bounds(sim, w, h);
    let nx = myco_core::gml::clamp(base.0 + d.x, b.x1, b.x2);
    let ny = myco_core::gml::clamp(base.1 + d.y, b.y1, b.y2);
    let (dx, dy) = (nx - base.0, ny - base.1);
    let fx = cams.game_view.x + cams.sub.x + dx;
    let fy = cams.game_view.y + cams.sub.y + dy;
    cams.game_view.x = fx.floor();
    cams.game_view.y = fy.floor();
    cams.sub = DVec2::new(fx - cams.game_view.x, fy - cams.game_view.y);
    Some((nx, ny))
}

/// Wheel and pinch (and the autopilot's `zoom:` actions) → the zoom target, then
/// [`zoom_advance`] at the cursor. Input is ignored while a menu, the tree, a popup or the pause
/// is up.
#[allow(clippy::too_many_arguments)]
fn zoom_system(
    time: Res<Time>,
    mut wheel: MessageReader<MouseWheel>,
    mut pinch: MessageReader<PinchGesture>,
    mut zs: ResMut<ZoomState>,
    mut cams: ResMut<Cameras>,
    sim: Option<Res<crate::sim_bridge::SimRes>>,
    blockers: Option<Res<crate::sim_bridge::UiBlockers>>,
    pending: Option<ResMut<crate::sim_bridge::PendingInput>>,
) {
    let mut log = 0.0;
    for ev in wheel.read() {
        let notches = match ev.unit {
            MouseScrollUnit::Line => ev.y as f64,
            MouseScrollUnit::Pixel => ev.y as f64 / 40.0,
        };
        log += notches * ZOOM_WHEEL_STEP.ln();
    }
    for ev in pinch.read() {
        log += ev.0 as f64;
    }
    let (Some(sim), Some(mut pending)) = (sim, pending) else {
        return;
    };
    log += std::mem::take(&mut pending.zoom_log);
    let blocked = blockers.is_some_and(|b| b.any()) || sim.0.pause_open;
    if log != 0.0 && !blocked {
        zoom_wheel(&mut zs, log.exp());
    }
    let pic = cams.picture;
    let centre = Vec2::new(
        ((pic.x + pic.w / 2.0) / cams.scale_factor) as f32,
        ((pic.y + pic.h / 2.0) / cams.scale_factor) as f32,
    );
    let anchor = pending.cursor_override.or(pending.cursor).unwrap_or(centre);
    let dt = time.delta_secs_f64();
    let base = pending
        .camera_pan
        .unwrap_or((sim.0.camera.x, sim.0.camera.y));
    if let Some(pan) = zoom_advance(&mut cams, &sim.0, &mut zs, dt, anchor, base) {
        pending.camera_pan = Some(pan);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::CameraProjection;

    /// Retina ×2, safe area 64 px from the top (window covering the notch), zoom 0.37.
    fn notch_cameras() -> Cameras {
        let mut c = Cameras::new(
            3024.0,
            1964.0,
            2.0,
            SafeArea {
                x: 0.0,
                y: 64.0,
                w: 3024.0,
                h: 1900.0,
            },
        );
        c.zoom = 0.37;
        c.set_game_view_center(5000.0, 1234.5);
        c
    }

    const POINTS: [Vec2; 5] = [
        Vec2::new(0.0, 32.0),
        Vec2::new(756.0, 507.0),
        Vec2::new(1511.5, 981.5),
        Vec2::new(100.25, 900.75),
        Vec2::new(1400.0, 40.0),
    ];

    #[test]
    fn screen_to_world_roundtrip_with_zoom() {
        let c = notch_cameras();
        for p in POINTS {
            let w = c.screen_to_world(p);
            let back = c.world_to_screen(w);
            assert!(
                (back - p.as_dvec2()).length() < 1e-6,
                "{p:?} -> {w:?} -> {back:?}"
            );
        }
        // the centre of the picture maps to the centre of game_view
        let pc = Vec2::new(
            ((c.picture.x + c.picture.w / 2.0) / 2.0) as f32,
            ((c.picture.y + c.picture.h / 2.0) / 2.0) as f32,
        );
        let w = c.screen_to_world(pc);
        assert!((w - DVec2::new(5000.0, 1234.5)).length() < 1e-6, "{w:?}");
        // the picture starts below the notch
        assert!(c.picture.y >= 64.0);
        // logic sees the 1× view with the same centre
        assert_eq!(c.game_view.w, c.layout.view_w);
        let rv = c.render_view();
        assert!((rv.w - c.game_view.w / 0.37).abs() < 1e-9);
    }

    /// The Bevy objects built from `Cameras` show exactly what `screen_to_world` says.
    #[test]
    fn bevy_projection_agrees_with_mapping() {
        let c = notch_cameras();
        let vp = c.viewport().unwrap();
        let mut proj = c.world_projection();
        proj.update(vp.physical_size.x as f32, vp.physical_size.y as f32);
        let tf = c.world_transform();
        for p in POINTS {
            let phys = p * 2.0;
            let t = (phys - vp.physical_position.as_vec2()) / vp.physical_size.as_vec2();
            let bx = tf.translation.x + proj.area.min.x + t.x * proj.area.width();
            let by = tf.translation.y + proj.area.max.y - t.y * proj.area.height();
            let w = c.screen_to_world(p);
            // f32 world coordinates near x = 5000: allow a few ulps
            assert!(
                (bx as f64 - w.x).abs() < 0.01 && (-(by as f64) - w.y).abs() < 0.01,
                "{p:?}"
            );
        }
    }

    #[test]
    fn gui_mapping_matches_display_lua() {
        // notch_fill_and_mouse in tests/test_display.lua: bottom-right corner -> view size
        let c = Cameras::new(
            3024.0,
            1964.0,
            1.0,
            SafeArea {
                x: 0.0,
                y: 74.0,
                w: 3024.0,
                h: 1890.0,
            },
        );
        let g = c.screen_to_gui(Vec2::new(3024.0, 1964.0));
        assert_eq!(g, DVec2::new(1920.0, 1200.0));
        assert_eq!(c.screen_to_gui(Vec2::new(0.0, 74.0)), DVec2::ZERO);
    }

    #[test]
    fn resize_keeps_game_view_centre() {
        let mut c = notch_cameras();
        c.set_window(
            1280.0,
            720.0,
            1.0,
            SafeArea {
                x: 0.0,
                y: 0.0,
                w: 1280.0,
                h: 720.0,
            },
        );
        assert_eq!(c.game_view_center(), (5000.0, 1234.5));
        assert_eq!((c.game_view.w, c.game_view.h), (1920.0, 1080.0));
    }

    /// Regression (R21a): `apply_cameras` used to write a fresh projection whose `area` is the
    /// unit rect and relied on Bevy's `camera_system` running afterwards; when the (ambiguous)
    /// schedule order put it after `camera_system`, the frustum was 2×2 world units and every
    /// sprite was culled (all-black world, flipping with the set of registered plugins).
    #[test]
    fn applied_projection_area_matches_the_view_without_camera_system() {
        use bevy::ecs::system::RunSystemOnce;
        let cams = notch_cameras();
        let rv = cams.render_view();
        let mut world = World::new();
        world.spawn((
            Camera::default(),
            Projection::Orthographic(OrthographicProjection::default_2d()),
            Transform::default(),
            WorldCamera,
        ));
        world.spawn((
            Camera::default(),
            Projection::Orthographic(OrthographicProjection::default_2d()),
            Transform::default(),
            GuiCamera,
        ));
        world.insert_resource(cams);
        world.run_system_once(apply_cameras).unwrap();
        let mut q = world.query_filtered::<&Projection, With<WorldCamera>>();
        let Projection::Orthographic(p) = q.single(&world).unwrap() else {
            panic!("not orthographic")
        };
        assert!(
            (p.area.width() as f64 - rv.w).abs() < 1e-3
                && (p.area.height() as f64 - rv.h).abs() < 1e-3,
            "world projection area {:?}, view {}x{}",
            p.area,
            rv.w,
            rv.h
        );
        let mut q = world.query_filtered::<&Projection, With<GuiCamera>>();
        let Projection::Orthographic(p) = q.single(&world).unwrap() else {
            panic!("not orthographic")
        };
        assert!(p.area.width() > 2.5, "gui projection area {:?}", p.area);
    }
}
