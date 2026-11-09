//! Scripted runs, as in `game/main.lua`:
//!
//! - `MYCO_INPUT="60:move:0.5,0.6;61:down:1;63:up:1;120:key:space"` (`frame:action:args`).
//!   Actions: `move fx,fy` (fractions of the picture rectangle), `gmove gx,gy` (GUI coordinates),
//!   `wmove dx,dy` (world offset from the mycelium controller), `down n` / `up n` (mouse button,
//!   1 left, 2 right), `key name` (press and release), `kdown name`, `kup name`, `text s`, `quit`.
//! - `MYCO_DUMP=<frame>:<file.json>` writes what a save would contain (`Sim::to_save`, the save
//!   JSON text without the trailing NUL) and quits.
//! - `MYCO_SHOT=<file.png>:<frame>` saves a screenshot at that frame and quits. A frame macOS
//!   did not let us render (window occluded: no swap-chain texture, so Bevy's capture comes back
//!   all zeros, alpha included) is never written; the shot waits for the next rendered frame,
//!   prints which frame it took, and fails after [`SHOT_GIVE_UP_FRAMES`].
//! - `MYCO_SEED=<int>` fixes the world seed (read in `main`, see [`seed_from_env`]).
//! - `MYCO_FOCUS` treats the window as focused (no pause on focus loss).
//!
//! A frame here is one fixed simulation step (1-based), which is one rendered frame in the LÖVE
//! port. Actions run before that step's simulation, so a `down` at frame N is the `pressed` of
//! step N.

use crate::camera::Cameras;
use crate::sim_bridge::{key_from_love_name, ForceFocus, PendingInput, SimRes, SimSet};
use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Move(f64, f64),
    GMove(f64, f64),
    WMove(f64, f64),
    Down(u8),
    Up(u8),
    Key(String),
    KDown(String),
    KUp(String),
    Text(String),
    Quit,
}

/// Parse `MYCO_INPUT`. Unknown actions and malformed entries are skipped like `main.lua` does.
pub fn parse_script(s: &str) -> Vec<(u64, Action)> {
    let pair = |a: &str| -> Option<(f64, f64)> {
        let (x, y) = a.split_once(',')?;
        Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
    };
    let mut out = Vec::new();
    for entry in s.split(';') {
        let mut parts = entry.splitn(3, ':');
        let (Some(f), Some(action), Some(args)) = (parts.next(), parts.next(), parts.next()) else {
            continue;
        };
        let Ok(frame) = f.trim().parse::<u64>() else {
            continue;
        };
        let a = match action {
            "move" => pair(args).map(|(x, y)| Action::Move(x, y)),
            "gmove" => pair(args).map(|(x, y)| Action::GMove(x, y)),
            "wmove" => pair(args).map(|(x, y)| Action::WMove(x, y)),
            "down" => args.trim().parse().ok().map(Action::Down),
            "up" => args.trim().parse().ok().map(Action::Up),
            "key" => Some(Action::Key(args.to_string())),
            "kdown" => Some(Action::KDown(args.to_string())),
            "kup" => Some(Action::KUp(args.to_string())),
            "text" => Some(Action::Text(args.to_string())),
            "quit" => Some(Action::Quit),
            _ => None,
        };
        if let Some(a) = a {
            out.push((frame, a));
        }
    }
    out
}

/// `<frame>:<path>` (MYCO_DUMP).
pub fn parse_dump(s: &str) -> Option<(u64, PathBuf)> {
    let (f, p) = s.split_once(':')?;
    if p.is_empty() {
        return None;
    }
    Some((f.parse().ok()?, PathBuf::from(p)))
}

/// `<path>:<frame>` (MYCO_SHOT); the path may itself contain colons.
pub fn parse_shot(s: &str) -> Option<(PathBuf, u64)> {
    let (p, f) = s.rsplit_once(':')?;
    if p.is_empty() {
        return None;
    }
    Some((PathBuf::from(p), f.parse().ok()?))
}

/// `MYCO_SEED` as a world seed.
pub fn seed_from_env() -> Option<i64> {
    std::env::var("MYCO_SEED").ok()?.trim().parse().ok()
}

/// World position of the mycelium controller for `wmove` (room position until the mycelium task
/// keeps it in sync with the controller).
#[derive(Resource, Clone, Copy, Debug)]
pub struct MyceliumAnchor(pub DVec2);

impl Default for MyceliumAnchor {
    fn default() -> Self {
        MyceliumAnchor(DVec2::new(640.0, 480.0)) // Room1 obj_mycelium_controller
    }
}

#[derive(Resource, Default, Debug)]
pub struct Autopilot {
    pub script: Vec<(u64, Action)>,
    /// fixed steps started so far (1-based frame of the current step)
    pub frame: u64,
    pub dump: Option<(u64, PathBuf)>,
    pub shot: Option<(PathBuf, u64)>,
    shot_requested: bool,
    /// the window is occluded (`WindowOccluded`): frames are not rendered
    occluded: bool,
    occluded_logged: bool,
}

/// How many frames past the requested one a screenshot may wait for a rendered frame.
pub const SHOT_GIVE_UP_FRAMES: u64 = 600;

/// Whether a window capture holds a rendered frame. Every camera on the window clears or
/// draws opaque pixels; an all-transparent capture means the frame was never rendered (Bevy
/// skips the swap-chain texture of an occluded window, `CurrentSurfaceTexture::Occluded`, yet
/// still delivers the untouched capture buffer).
pub fn capture_rendered(img: &Image) -> bool {
    img.data
        .as_ref()
        .is_some_and(|d| d.as_chunks::<4>().0.iter().any(|p| p[3] != 0))
}

impl Autopilot {
    pub fn from_env() -> Autopilot {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Autopilot {
            script: var("MYCO_INPUT")
                .map(|s| parse_script(&s))
                .unwrap_or_default(),
            frame: 0,
            dump: var("MYCO_DUMP").and_then(|s| parse_dump(&s)),
            shot: var("MYCO_SHOT").and_then(|s| parse_shot(&s)),
            shot_requested: false,
            occluded: false,
            occluded_logged: false,
        }
    }
}

pub struct AutopilotPlugin;

impl Plugin for AutopilotPlugin {
    fn build(&self, app: &mut App) {
        let force_focus = std::env::var_os("MYCO_FOCUS").is_some();
        app.insert_resource(Autopilot::from_env())
            .insert_resource(ForceFocus(force_focus))
            .init_resource::<MyceliumAnchor>()
            .add_systems(
                FixedUpdate,
                (
                    run_script.in_set(SimSet::Prepare),
                    dump_state.in_set(SimSet::After),
                ),
            )
            .add_systems(Update, (track_occlusion, take_screenshot).chain());
    }
}

fn mouse_button(n: u8) -> Option<MouseButton> {
    match n {
        1 => Some(MouseButton::Left),
        2 => Some(MouseButton::Right),
        3 => Some(MouseButton::Middle),
        _ => None,
    }
}

/// Apply one action to the pending input (screen positions through `cams`).
pub fn apply_action(a: &Action, pending: &mut PendingInput, cams: &Cameras, anchor: DVec2) -> bool {
    let sf = cams.scale_factor;
    let pic = cams.picture;
    match a {
        Action::Move(fx, fy) => {
            let p = DVec2::new(pic.x + pic.w * fx, pic.y + pic.h * fy) / sf;
            pending.cursor_override = Some(p.as_vec2());
        }
        Action::GMove(gx, gy) => {
            pending.cursor_override = Some(cams.gui_to_screen(DVec2::new(*gx, *gy)).as_vec2())
        }
        Action::WMove(dx, dy) => {
            pending.cursor_override = Some(
                cams.world_to_screen(anchor + DVec2::new(*dx, *dy))
                    .as_vec2(),
            )
        }
        Action::Down(b) | Action::Up(b) => {
            if let Some(mb) = mouse_button(*b) {
                pending.mouse_event(mb, matches!(a, Action::Down(_)));
            }
        }
        Action::Key(k) => {
            if let Some(k) = key_from_love_name(k) {
                pending.key_event(k, true);
                pending.key_event(k, false);
            }
        }
        Action::KDown(k) | Action::KUp(k) => {
            if let Some(k) = key_from_love_name(k) {
                pending.key_event(k, matches!(a, Action::KDown(_)));
            }
        }
        Action::Text(t) => pending.text.push_str(t),
        Action::Quit => return true,
    }
    false
}

/// Runs the `MYCO_INPUT` actions due this fixed step (`SimSet::Prepare`); UI steps that read
/// the pending input order themselves after it.
pub fn run_script(
    mut ap: ResMut<Autopilot>,
    mut pending: ResMut<PendingInput>,
    cams: Res<Cameras>,
    anchor: Res<MyceliumAnchor>,
    mut exit: MessageWriter<AppExit>,
) {
    ap.frame += 1;
    let frame = ap.frame;
    for (f, a) in &ap.script {
        if *f == frame && apply_action(a, &mut pending, &cams, anchor.0) {
            exit.write(AppExit::Success);
        }
    }
}

fn dump_state(ap: Res<Autopilot>, sim: Res<SimRes>, mut exit: MessageWriter<AppExit>) {
    let Some((frame, path)) = &ap.dump else {
        return;
    };
    if ap.frame != *frame {
        return;
    }
    let (state, _meta) = sim.0.to_save();
    let ok = myco_core::save::to_love_json(&state).and_then(|text| std::fs::write(path, text));
    match ok {
        Ok(()) => println!("DUMP wrote {}", path.display()),
        Err(e) => {
            eprintln!("DUMP failed: {e}");
            std::process::exit(1);
        }
    }
    exit.write(AppExit::Success);
}

fn track_occlusion(
    mut msgs: MessageReader<bevy::window::WindowOccluded>,
    mut ap: ResMut<Autopilot>,
) {
    for m in msgs.read() {
        ap.occluded = m.occluded;
    }
}

fn take_screenshot(mut ap: ResMut<Autopilot>, mut commands: Commands) {
    let Some((path, frame)) = ap.shot.clone() else {
        return;
    };
    if ap.shot_requested || ap.frame < frame {
        return;
    }
    if ap.frame > frame + SHOT_GIVE_UP_FRAMES {
        eprintln!(
            "SHOT failed: no rendered frame between {frame} and {} (window occluded)",
            ap.frame
        );
        commands.write_message(AppExit::error());
        ap.shot = None;
        return;
    }
    if ap.occluded {
        if !ap.occluded_logged {
            eprintln!("SHOT waiting: window occluded at frame {}", ap.frame);
            ap.occluded_logged = true;
        }
        return;
    }
    ap.shot_requested = true;
    let mut save = save_to_disk(path.clone());
    commands.spawn(Screenshot::primary_window()).observe(
        move |ev: On<ScreenshotCaptured>,
              mut ap: ResMut<Autopilot>,
              mut exit: MessageWriter<AppExit>| {
            if !capture_rendered(&ev.image) {
                eprintln!(
                    "SHOT discarded: frame {} was not rendered (window occluded), retrying",
                    ap.frame
                );
                ap.shot_requested = false;
                return;
            }
            let at = ap.frame;
            save(ev);
            if at == frame {
                println!("SHOT wrote {}", path.display());
            } else {
                println!(
                    "SHOT wrote {} (frame {at}, requested {frame})",
                    path.display()
                );
            }
            exit.write(AppExit::Success);
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::SafeArea;

    #[test]
    fn unrendered_capture_is_detected() {
        use bevy::asset::RenderAssetUsages;
        use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
        let img = |px: [u8; 4]| {
            Image::new_fill(
                Extent3d {
                    width: 4,
                    height: 2,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &px,
                TextureFormat::Bgra8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD,
            )
        };
        assert!(!capture_rendered(&img([0, 0, 0, 0])), "never rendered");
        assert!(capture_rendered(&img([0, 0, 0, 255])), "rendered black");
    }

    #[test]
    fn parses_main_lua_format() {
        let s = parse_script(
            "60:move:0.5,0.6;61:down:1;63:up:1;120:key:space;5:gmove:-10,20.5;7:bogus:1;9:quit:",
        );
        assert_eq!(
            s,
            vec![
                (60, Action::Move(0.5, 0.6)),
                (61, Action::Down(1)),
                (63, Action::Up(1)),
                (120, Action::Key("space".into())),
                (5, Action::GMove(-10.0, 20.5)),
                (9, Action::Quit),
            ]
        );
        assert_eq!(
            parse_shot("/tmp/r.png:30"),
            Some((PathBuf::from("/tmp/r.png"), 30))
        );
        assert_eq!(
            parse_dump("120:/tmp/d.json"),
            Some((120, PathBuf::from("/tmp/d.json")))
        );
    }

    #[test]
    fn move_and_wmove_land_on_their_points() {
        let mut cams = Cameras::new(
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
        cams.zoom = 0.5;
        let mut p = PendingInput::default();
        apply_action(
            &Action::WMove(10.0, -20.0),
            &mut p,
            &cams,
            DVec2::new(640.0, 480.0),
        );
        let w = cams.screen_to_world(p.cursor_override.unwrap());
        assert!((w - DVec2::new(650.0, 460.0)).length() < 1e-2, "{w:?}");
        apply_action(&Action::Move(0.0, 0.0), &mut p, &cams, DVec2::ZERO);
        assert_eq!(cams.screen_to_gui(p.cursor_override.unwrap()), DVec2::ZERO);
    }
}
