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
//!   prints which frame it took, and fails after [`SHOT_GIVE_UP_FRAMES`]. With
//!   `MYCO_SHOT_EXACT=1` a late capture is not written and the app exits with an error, so
//!   comparison runs never compare different frames.
//! - `MYCO_SEED=<int>` fixes the world seed (read in `main`, see [`seed_from_env`]).
//! - `MYCO_FOCUS` treats the window as focused (no pause on focus loss).
//!
//! A frame here is one fixed simulation step (1-based), which is one rendered frame in the LÖVE
//! port. LÖVE runs a frame's actions in `love.draw`, after that frame's update (main.lua), so an
//! action at frame N is seen by step N + 1; here the actions of frame N are applied at the start
//! of step N + 1 ([`run_script`], `SimSet::Prepare`), so frame numbers mean the same step in both
//! ports. A dump at frame N is the state after step N in both.

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
    /// Rust only: `zoom:<factor>` multiplies the zoom target at the cursor, like the wheel
    Zoom(f64),
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
            "zoom" => args.trim().parse().ok().map(Action::Zoom),
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
    /// `MYCO_SHOT_EXACT=1`: only the requested frame is acceptable
    pub shot_exact: bool,
}

/// How many frames past the requested one a screenshot may wait for a rendered frame.
pub const SHOT_GIVE_UP_FRAMES: u64 = 600;

/// Whether a capture taken at frame `at` may stand for the requested `frame`: always by default,
/// only the exact frame with `MYCO_SHOT_EXACT=1`.
pub fn shot_frame_acceptable(exact: bool, at: u64, frame: u64) -> bool {
    !exact || at == frame
}

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
            shot_exact: var("MYCO_SHOT_EXACT").is_some_and(|v| v == "1"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Development hooks for render comparisons. Each one acts only when its variable is set; with
// none set they change nothing.

/// `MYCO_CAMERA=x,y` (camera top-left when the game starts), `MYCO_CAVE_TAKE=<cave id>` (take
/// that cave when the game starts), `MYCO_FORCE_RAIN=1` (start the rain cycle at boot, the
/// original's `weather_force_rain` debug hook), `MYCO_BW=1` (BLACK METAL post effect on; the
/// shipped menus cannot turn it on), `MYCO_UI_DEMO=1` (synthetic floating numbers). The LÖVE side
/// does the same through `MYCO_PROBE`.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct DevHooks {
    pub camera: Option<(f64, f64)>,
    pub cave_take: Option<String>,
    pub force_rain: bool,
    pub bw: bool,
    pub ui_demo: bool,
}

impl DevHooks {
    pub fn from_vars(get: impl Fn(&str) -> Option<String>) -> DevHooks {
        let on = |k: &str| get(k).is_some_and(|v| v == "1");
        DevHooks {
            camera: get("MYCO_CAMERA").and_then(|v| {
                let (x, y) = v.split_once(',')?;
                Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
            }),
            cave_take: get("MYCO_CAVE_TAKE").filter(|v| !v.is_empty()),
            force_rain: on("MYCO_FORCE_RAIN"),
            bw: on("MYCO_BW"),
            ui_demo: on("MYCO_UI_DEMO"),
        }
    }

    pub fn from_env() -> DevHooks {
        DevHooks::from_vars(|k| std::env::var(k).ok())
    }

    /// Boot-time hooks on the title's sim (`MYCO_FORCE_RAIN`).
    pub fn apply_at_boot(&self, sim: &mut myco_core::sim::Sim) {
        if self.force_rain {
            myco_core::weather::force_rain(sim);
        }
    }

    /// Game-start hooks (`MYCO_CAMERA`, `MYCO_CAVE_TAKE`), once the title has closed.
    pub fn apply_at_game_start(&self, sim: &mut myco_core::sim::Sim) {
        if let Some((x, y)) = self.camera {
            sim.camera.x = x;
            sim.camera.y = y;
            // camera_bounds is real (Task 21): grow the camera box so the forest step keeps the
            // requested view (the LÖVE side neutralises camera_bounds through MYCO_PROBE)
            let (w, h) = (sim.camera.width, sim.camera.height);
            let b = &mut sim.state.state.cam_box;
            b[0] = b[0].min(x);
            b[1] = b[1].min(y);
            b[2] = b[2].max(x + w);
            b[3] = b[3].max(y + h);
        }
        if let Some(id) = &self.cave_take {
            let cave = sim.world.cave_rects().iter().find(|c| &c.id == id).cloned();
            if let Some(c) = cave {
                myco_core::world::caves::cave_take(sim, &c, None, None);
            }
        }
    }
}

fn dev_hooks_boot(
    hooks: Res<DevHooks>,
    mut sim: ResMut<SimRes>,
    mut bw: Option<ResMut<crate::render::post::BwMode>>,
) {
    hooks.apply_at_boot(&mut sim.0);
    if let Some(bw) = bw.as_mut() {
        bw.0 |= hooks.bw;
    }
}

fn dev_hooks_game_start(hooks: Res<DevHooks>, mut sim: ResMut<SimRes>, mut done: Local<bool>) {
    if *done || sim.0.title_open {
        return;
    }
    *done = true;
    hooks.apply_at_game_start(&mut sim.0);
}

/// `MYCO_UI_DEMO=1` (dev hook for screenshots): every 0.5 s synthetic resource events around the
/// view centre and a refused node, for the floating numbers and tips.
pub fn ui_demo(
    sim: Res<crate::sim_bridge::SimRes>,
    cams: Res<crate::camera::Cameras>,
    hooks: Res<DevHooks>,
    mut out: MessageWriter<crate::sim_bridge::SimEventMsg>,
) {
    use myco_core::events::{NodeReject, ResourceKind, SimEvent};
    if !hooks.ui_demo || !sim.0.frame.is_multiple_of(30) {
        return;
    }
    let (cx, cy) = cams.game_view_center();
    let k = (sim.0.frame / 30) as f64;
    let ev = |kind, delta, dx: f64, dy: f64, source| SimEvent::Resource {
        kind,
        delta,
        x: cx + dx,
        y: cy + dy,
        source,
    };
    for e in [
        ev(ResourceKind::Food, 5.0, -300.0 + 20.0 * (k % 5.0), 80.0, 1),
        ev(ResourceKind::Food, 3.0, -300.0, 80.0, 1),
        ev(ResourceKind::Water, 12.0, 200.0, 140.0, 2),
        ev(ResourceKind::Food, -15.0, -40.0, 220.0, 3),
        ev(ResourceKind::UpgradePoints, 1.0, 420.0, 40.0, 4),
        SimEvent::NodeRejected {
            reason: NodeReject::NotEnoughFood { need: 12 },
            x: cx,
            y: cy,
        },
    ] {
        out.write(crate::sim_bridge::SimEventMsg(e));
    }
}

/// `MYCO_FPS_LOG=<steps>` (long runs): every that many fixed steps print the frame rate, the
/// fixed step rate, the mean and worst frame time and the mean CPU time of a frame (First to
/// Last, without waiting for the display), e.g. `FPS step 600: 60.0 frames/s, 60.0 steps/s,
/// frame 16.67 ms, worst 20.1 ms, cpu 4.2 ms`; and the window's size once.
#[derive(Default)]
pub struct FpsLog {
    every: Option<u64>,
    frames: u64,
    span_start: Option<std::time::Instant>,
    steps_at_start: u64,
    worst_ms: f64,
    last: Option<std::time::Instant>,
    busy_ms: f64,
    window_logged: bool,
}

/// When this frame started (`First`), for [`FpsLog`]'s CPU time.
#[derive(Resource, Default)]
pub struct FrameStart(pub Option<std::time::Instant>);

fn frame_start(mut s: ResMut<FrameStart>) {
    s.0 = Some(std::time::Instant::now());
}

fn fps_log(
    ap: Res<Autopilot>,
    start_of_frame: Res<FrameStart>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut log: Local<FpsLog>,
    mut init: Local<bool>,
) {
    if !*init {
        *init = true;
        log.every = std::env::var("MYCO_FPS_LOG")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|n| *n > 0);
    }
    let Some(every) = log.every else { return };
    let now = std::time::Instant::now();
    if let Some(t0) = start_of_frame.0 {
        log.busy_ms += (now - t0).as_secs_f64() * 1000.0;
    }
    if !log.window_logged {
        if let Ok(w) = windows.single() {
            if w.physical_width() > 0 && ap.frame > 10 {
                log.window_logged = true;
                println!(
                    "WINDOW {}x{} px, scale {}, {:?}",
                    w.physical_width(),
                    w.physical_height(),
                    w.scale_factor(),
                    w.mode
                );
            }
        }
    }
    if let Some(last) = log.last {
        let ms = (now - last).as_secs_f64() * 1000.0;
        if ms > log.worst_ms {
            log.worst_ms = ms;
        }
    }
    log.last = Some(now);
    log.frames += 1;
    let start = *log.span_start.get_or_insert(now);
    let steps = ap.frame - log.steps_at_start;
    if steps >= every {
        let secs = (now - start).as_secs_f64().max(1e-9);
        println!(
            "FPS step {}: {:.1} frames/s, {:.1} steps/s, frame {:.2} ms, worst {:.1} ms, cpu {:.2} ms",
            ap.frame,
            log.frames as f64 / secs,
            steps as f64 / secs,
            secs * 1000.0 / log.frames as f64,
            log.worst_ms,
            log.busy_ms / log.frames as f64
        );
        log.frames = 0;
        log.busy_ms = 0.0;
        log.span_start = Some(now);
        log.steps_at_start = ap.frame;
        log.worst_ms = 0.0;
    }
}

pub struct AutopilotPlugin;

impl Plugin for AutopilotPlugin {
    fn build(&self, app: &mut App) {
        let force_focus = std::env::var_os("MYCO_FOCUS").is_some();
        app.insert_resource(Autopilot::from_env())
            .insert_resource(ForceFocus(force_focus))
            .insert_resource(DevHooks::from_env())
            .init_resource::<MyceliumAnchor>()
            .add_systems(Startup, dev_hooks_boot)
            .add_systems(
                FixedUpdate,
                (
                    run_script.in_set(SimSet::Prepare),
                    dev_hooks_game_start
                        .in_set(SimSet::Prepare)
                        .after(run_script),
                    ui_demo.in_set(SimSet::After),
                    dump_state.in_set(SimSet::After),
                ),
            )
            .add_systems(Update, (track_occlusion, take_screenshot).chain())
            .init_resource::<FrameStart>()
            .add_systems(First, frame_start)
            .add_systems(Last, fps_log);
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
            pending.cursor_override = Some(p);
        }
        Action::GMove(gx, gy) => {
            pending.cursor_override = Some(cams.gui_to_screen(DVec2::new(*gx, *gy)))
        }
        Action::WMove(dx, dy) => {
            pending.cursor_override = Some(cams.world_to_screen(anchor + DVec2::new(*dx, *dy)))
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
        Action::Zoom(f) => {
            if *f > 0.0 {
                pending.zoom_log += f.ln();
            }
        }
        Action::Quit => return true,
    }
    false
}

/// The script entries applied at the start of step `step` (1-based): those of the frame before
/// it, as LÖVE runs frame N's actions after step N (frame 0 never runs, as in main.lua).
pub fn actions_before_step(script: &[(u64, Action)], step: u64) -> impl Iterator<Item = &Action> {
    script
        .iter()
        .filter(move |(f, _)| *f >= 1 && *f + 1 == step)
        .map(|(_, a)| a)
}

/// Starts fixed step `frame` and applies the `MYCO_INPUT` actions of the previous frame
/// (`SimSet::Prepare`; see the module docs); UI steps that read the pending input order
/// themselves after it.
pub fn run_script(
    mut ap: ResMut<Autopilot>,
    mut pending: ResMut<PendingInput>,
    cams: Res<Cameras>,
    anchor: Res<MyceliumAnchor>,
    mut exit: MessageWriter<AppExit>,
) {
    ap.frame += 1;
    let frame = ap.frame;
    #[cfg(feature = "trace-draws")]
    {
        // MYCO_TRACE_STEPS=a-b: print the game-RNG draw callers from step a to b
        let on = std::env::var("MYCO_TRACE_STEPS")
            .ok()
            .and_then(|v| {
                let (a, b) = v.split_once('-')?;
                Some(frame >= a.parse().ok()? && frame <= b.parse().ok()?)
            })
            .unwrap_or(false);
        myco_core::rng::draw_trace::set_range(if on { 0 } else { u64::MAX }, u64::MAX);
        if on {
            println!("STEP {frame}");
        }
    }
    for a in actions_before_step(&ap.script, frame) {
        if apply_action(a, &mut pending, &cams, anchor.0) {
            exit.write(AppExit::Success);
        }
    }
}

/// `MYCO_DUMP`: the save JSON after the step of the requested frame, once every `SimSet::After`
/// system that touches the sim has run (LÖVE dumps after that frame's draw).
pub fn dump_state(ap: Res<Autopilot>, sim: Res<SimRes>, mut exit: MessageWriter<AppExit>) {
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
    // the capture is of this frame; the observer runs a few frames later (async readback)
    let at = ap.frame;
    let exact = ap.shot_exact;
    let mut save = save_to_disk(path.clone());
    commands.spawn(Screenshot::primary_window()).observe(
        move |ev: On<ScreenshotCaptured>,
              mut ap: ResMut<Autopilot>,
              mut exit: MessageWriter<AppExit>| {
            if !capture_rendered(&ev.image) {
                eprintln!(
                    "SHOT discarded: frame {at} was not rendered (window occluded), retrying"
                );
                ap.shot_requested = false;
                return;
            }
            if !shot_frame_acceptable(exact, at, frame) {
                eprintln!(
                    "SHOT failed: frame {frame} was not rendered (window occluded); the first \
                     rendered frame is {at} and MYCO_SHOT_EXACT=1 forbids a late capture"
                );
                exit.write(AppExit::error());
                return;
            }
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
    fn exact_shots_refuse_a_later_frame() {
        assert!(
            shot_frame_acceptable(false, 605, 600),
            "default: a late capture is kept"
        );
        assert!(shot_frame_acceptable(true, 600, 600));
        assert!(
            !shot_frame_acceptable(true, 605, 600),
            "MYCO_SHOT_EXACT=1: late fails"
        );
    }

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
    fn dev_hooks_are_off_without_their_variables() {
        assert_eq!(DevHooks::from_vars(|_| None), DevHooks::default());
        let h = DevHooks::from_vars(|k| match k {
            "MYCO_CAMERA" => Some("100,-20.5".into()),
            "MYCO_CAVE_TAKE" => Some("purple".into()),
            "MYCO_FORCE_RAIN" | "MYCO_BW" | "MYCO_UI_DEMO" => Some("1".into()),
            _ => None,
        });
        assert_eq!(h.camera, Some((100.0, -20.5)));
        assert_eq!(h.cave_take.as_deref(), Some("purple"));
        assert!(h.force_rain && h.bw && h.ui_demo);
        // with no hook set, nothing in the game state changes
        let mut sim =
            myco_core::sim::Sim::new_game(myco_core::state::GameMode::Exploration, "", Some(7));
        let before = myco_core::save::to_love_json(&sim.to_save().0).unwrap();
        DevHooks::default().apply_at_boot(&mut sim);
        DevHooks::default().apply_at_game_start(&mut sim);
        assert_eq!(
            myco_core::save::to_love_json(&sim.to_save().0).unwrap(),
            before
        );
    }

    #[test]
    fn frame_n_actions_reach_step_n_plus_one() {
        let s = parse_script("0:key:a;1:down:1;3:up:1;3:key:space");
        let at = |step| actions_before_step(&s, step).cloned().collect::<Vec<_>>();
        assert!(
            at(1).is_empty(),
            "frame 0 never runs; nothing precedes step 1"
        );
        assert_eq!(at(2), vec![Action::Down(1)]);
        assert!(at(3).is_empty());
        assert_eq!(at(4), vec![Action::Up(1), Action::Key("space".into())]);
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
