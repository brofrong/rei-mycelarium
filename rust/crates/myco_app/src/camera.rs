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
//! Zoom scales the world around the centre of `game_view`; logic always gets `game_view` (1×).

use crate::display::{self, Fit, SafeArea, ViewLayout};
use bevy::camera::{ScalingMode, Viewport};
use bevy::math::DVec2;
use bevy::prelude::*;
use myco_core::sim::ViewRect;

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

    /// The world rectangle the render camera shows (game view scaled by 1/zoom about its centre).
    pub fn render_view(&self) -> ViewRect {
        let (cx, cy) = self.game_view_center();
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
        let (cx, cy) = self.game_view_center();
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
        let (cx, cy) = self.game_view_center();
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

    /// World camera transform: the game view centre, y flipped.
    pub fn world_transform(&self) -> Transform {
        let (cx, cy) = self.game_view_center();
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

/// Render layer of everything drawn by the GUI camera.
pub const GUI_LAYER: usize = 1;

#[derive(Component)]
pub struct WorldCamera;

#[derive(Component)]
pub struct GuiCamera;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cameras>()
            .add_systems(Startup, spawn_cameras)
            .add_systems(First, track_window)
            .add_systems(
                PostUpdate,
                apply_cameras.before(TransformSystems::Propagate),
            );
    }
}

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
    cams: Res<Cameras>,
    mut world: Query<CamParts, (With<WorldCamera>, Without<GuiCamera>)>,
    mut gui: Query<CamParts, (With<GuiCamera>, Without<WorldCamera>)>,
) {
    let vp = cams.viewport();
    for (mut cam, mut proj, mut tf) in &mut world {
        cam.viewport = vp.clone();
        *proj = Projection::Orthographic(cams.world_projection());
        *tf = cams.world_transform();
    }
    for (mut cam, mut proj, mut tf) in &mut gui {
        cam.viewport = vp.clone();
        *proj = Projection::Orthographic(cams.gui_projection());
        *tf = cams.gui_transform();
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
}
