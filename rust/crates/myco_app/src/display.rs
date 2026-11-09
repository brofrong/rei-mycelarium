//! Window, fullscreen below the MacBook notch, `view_for_window`, settings.ini.
//!
//! Ports `game/gm/display_math.lua` (`fit`, `screen_to_game`, `view_for_window`) and the window
//! side of `game/gm/display.lua`. All sizes are physical (drawable) pixels unless a name says
//! otherwise.
//!
//! Fullscreen is `WindowMode::BorderlessFullscreen`, which winit implements on macOS with
//! `toggleFullScreen:` (a native fullscreen Space). macOS then places the drawable below the
//! notch itself (3024×1898 on a 14" MacBook Pro, the same as the LÖVE port measured in
//! `re/notes/display.md`). If a window ever covers the whole screen including the notch, the
//! `NSScreen.safeAreaInsets` top inset is cut off by [`safe_area_in_window`].

use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode, WindowPosition, WindowResolution};
use myco_core::save;
use myco_core::state::DisplaySettings;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

/// `display_math.view_for_window` result: logical view and application-surface size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewLayout {
    pub view_w: f64,
    pub view_h: f64,
    pub surf_w: f64,
    pub surf_h: f64,
    /// left edge of the 1920-wide UI inside the view (`global.ui_ox`)
    pub ui_ox: f64,
    /// UI bottom anchoring offset (`global.ui_by`)
    pub ui_by: f64,
}

impl Default for ViewLayout {
    fn default() -> Self {
        ViewLayout {
            view_w: 1920.0,
            view_h: 1080.0,
            surf_w: 1920.0,
            surf_h: 1080.0,
            ui_ox: 0.0,
            ui_by: 0.0,
        }
    }
}

// port: display_fit
/// `display_fit` (core_3.lua:370; `display_math.view_for_window` in the LÖVE host): the view
/// keeps 1920×1080 as a minimum and grows along the window aspect (clamped to 1.25..32/9); the
/// surface follows the window size, capped at 4K pixels. Rounding is the game's `round`
/// (banker's, `gml::round`), as the decompiled display_fit does. `None` for a degenerate window.
/// [`crate::camera::Cameras`] applies it whenever the window changes (`track_window`), which is
/// what display_fit's "changed" check guards in the original.
pub fn view_for_window(ww: f64, wh: f64) -> Option<ViewLayout> {
    if ww <= 0.0 || wh <= 0.0 {
        return None;
    }
    let round = myco_core::gml::round;
    let aspect = 1.25_f64.max((32.0 / 9.0_f64).min(ww / wh));
    let (vw, vh) = if aspect >= 16.0 / 9.0 {
        (round(1080.0 * aspect), 1080.0)
    } else {
        (1920.0, round(1920.0 / aspect))
    };
    let k = (ww / vw)
        .min(wh / vh)
        .min((3840.0 * 2160.0 / (vw * vh)).sqrt());
    Some(ViewLayout {
        view_w: vw,
        view_h: vh,
        surf_w: round(vw * k).max(1.0),
        surf_h: round(vh * k).max(1.0),
        ui_ox: (vw - 1920.0) * 0.5,
        ui_by: vh - 1080.0,
    })
}

/// `display_math.fit`: where the application surface lands in the window (window pixels).
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Fit {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub scale: f64,
}

/// Largest aspect-preserving `base_w×base_h` box inside the safe area, centred.
#[allow(clippy::too_many_arguments)]
pub fn fit(
    win_w: f64,
    win_h: f64,
    safe_x: f64,
    safe_y: f64,
    safe_w: f64,
    safe_h: f64,
    base_w: f64,
    base_h: f64,
) -> Fit {
    if win_w <= 0.0 || win_h <= 0.0 || safe_w <= 0.0 || safe_h <= 0.0 {
        return Fit::default();
    }
    let scale = (safe_w / base_w).min(safe_h / base_h);
    let (w, h) = (base_w * scale, base_h * scale);
    Fit {
        x: safe_x + (safe_w - w) / 2.0,
        y: safe_y + (safe_h - h) / 2.0,
        w,
        h,
        scale,
    }
}

/// Safe area inside the window, physical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct SafeArea {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// The screen the game starts on (`NSScreen.mainScreen`), points plus backing scale.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct ScreenInfo {
    pub width_pt: f64,
    pub height_pt: f64,
    pub scale: f64,
    /// `safeAreaInsets.top` (the notch); 0 on screens without one
    pub inset_top_pt: f64,
}

/// The window's safe area: the whole window, minus the notch inset when the window covers the
/// full screen height (the notch then overlaps its top rows).
pub fn safe_area_in_window(win_w: f64, win_h: f64, screen: Option<&ScreenInfo>) -> SafeArea {
    let mut s = SafeArea {
        x: 0.0,
        y: 0.0,
        w: win_w.max(0.0),
        h: win_h.max(0.0),
    };
    if let Some(sc) = screen {
        let full_h = (sc.height_pt * sc.scale).round();
        let inset = (sc.inset_top_pt * sc.scale).round();
        if inset > 0.0 && win_h >= full_h && win_h > inset {
            s.y = inset;
            s.h = win_h - inset;
        }
    }
    s
}

/// `NSScreen.mainScreen` metrics. Must run on the main thread (call from `main` before the app
/// starts); `None` when unavailable.
#[cfg(target_os = "macos")]
pub fn main_screen_info() -> Option<ScreenInfo> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;
    let mtm = MainThreadMarker::new()?;
    let screen = NSScreen::mainScreen(mtm)?;
    let frame = screen.frame();
    let insets = screen.safeAreaInsets();
    Some(ScreenInfo {
        width_pt: frame.size.width,
        height_pt: frame.size.height,
        scale: screen.backingScaleFactor(),
        inset_top_pt: insets.top,
    })
}

#[cfg(not(target_os = "macos"))]
pub fn main_screen_info() -> Option<ScreenInfo> {
    None
}

/// Save folder: `MYCO_SAVE_DIR` if set; else, for an automated run (`MYCO_INPUT`, `MYCO_DUMP` or
/// `MYCO_SHOT` set), a fresh temp folder; else the LÖVE port's folder (`save::save_dir`).
pub fn save_dir() -> PathBuf {
    let automated = ["MYCO_INPUT", "MYCO_DUMP", "MYCO_SHOT"]
        .iter()
        .any(|k| std::env::var_os(k).is_some_and(|v| !v.is_empty()));
    save_dir_for(
        std::env::var_os("MYCO_SAVE_DIR").map(PathBuf::from),
        automated,
    )
}

/// [`save_dir`] from its inputs: an automated run never falls back to the real save folder.
pub fn save_dir_for(env: Option<PathBuf>, automated: bool) -> PathBuf {
    match env {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ if automated => {
            std::env::temp_dir().join(format!("myco-auto-save-{}", std::process::id()))
        }
        _ => save::save_dir(),
    }
}

pub fn save_dir_from(env: Option<PathBuf>) -> PathBuf {
    match env {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => save::save_dir(),
    }
}

// port: display_save
/// `display_save`: write `[display]` (fullscreen, the chosen window size) into settings.ini
/// through `save::ini_write`, keeping every other section (`[game] language`). Values in
/// `ini_write_real`'s format (`%f`).
pub fn write_display_settings(dir: &Path, s: &DisplaySettings) -> io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("settings.ini");
    let mut text = String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_default()).into_owned();
    let fs = if s.fullscreen { 1.0 } else { 0.0 };
    text = save::ini_write(&text, "display", "fullscreen", &format!("{fs:.6}"));
    text = save::ini_write(
        &text,
        "display",
        "window_w",
        &format!("{:.6}", s.window_w as f64),
    );
    text = save::ini_write(
        &text,
        "display",
        "window_h",
        &format!("{:.6}", s.window_h as f64),
    );
    let tmp = dir.join("settings.ini.tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, &path)
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// Folder holding saves and settings.ini for this run.
#[derive(Resource, Clone, Debug)]
pub struct SaveDir(pub PathBuf);

/// Display settings in effect (as read from settings.ini at startup, updated on toggles).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Display(pub DisplaySettings);

/// Ask the display to step through the window sizes (`display_cycle_size(dir)`, settings menu).
#[derive(Message, Clone, Copy, Debug)]
pub struct CycleWindowSize(pub i32);

/// The window sizes offered in the settings (`global.display_sizes`) and the chosen one.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct DisplaySizes {
    pub sizes: Vec<(u32, u32)>,
    pub index: usize,
}

impl DisplaySizes {
    /// `display_init`'s list: the standard sizes that fit the screen (`display_get_width()`,
    /// `display_get_height() - 80`, in points as the LÖVE port reports them), at least 1280×720;
    /// the size saved in settings.ini is selected, else the largest.
    pub fn new(screen: Option<(f64, f64)>, saved: Option<(u32, u32)>) -> DisplaySizes {
        const ALL: [(u32, u32); 7] = [
            (1280, 720),
            (1366, 768),
            (1600, 900),
            (1920, 1080),
            (2560, 1440),
            (3200, 1800),
            (3840, 2160),
        ];
        let (dw, dh) = screen.unwrap_or((1920.0, 1080.0 + 80.0));
        let mut sizes: Vec<(u32, u32)> = ALL
            .iter()
            .copied()
            .filter(|&(w, h)| w as f64 <= dw && h as f64 <= dh - 80.0)
            .collect();
        if sizes.is_empty() {
            sizes.push(ALL[0]);
        }
        let index = saved
            .and_then(|want| sizes.iter().rposition(|&s| s == want))
            .unwrap_or(sizes.len() - 1);
        DisplaySizes { sizes, index }
    }

    // port: display_window_size
    pub fn display_window_size(&self) -> (u32, u32) {
        self.sizes[self.index]
    }

    // port: display_cycle_size
    /// Step to the next/previous size (wrapping); the caller applies and saves it.
    pub fn display_cycle_size(&mut self, dir: i32) -> (u32, u32) {
        let n = self.sizes.len() as i64;
        self.index = (self.index as i64 + dir as i64 + n).rem_euclid(n) as usize;
        self.display_window_size()
    }
}

// port: display_init
/// `display_init`: the window sizes that fit the screen and the saved one picked
/// ([`DisplaySizes::new`]); the saved fullscreen flag (default on) already chose the window mode
/// in [`primary_window`]; then `display_apply` (the countdown) and `display_fit` (the camera's
/// `track_window`).
fn init_display_sizes(
    mut commands: Commands,
    screen: Option<Res<ScreenInfo>>,
    dir: Option<Res<SaveDir>>,
) {
    let pt = screen.map(|s| (s.width_pt, s.height_pt));
    // ini_read_real("display", "window_w"/"window_h", 0)
    let text = dir
        .and_then(|d| std::fs::read(d.0.join("settings.ini")).ok())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let num = |k: &str| {
        save::ini_read(&text, "display", k)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    let saved = Some((num("window_w") as u32, num("window_h") as u32));
    commands.insert_resource(DisplaySizes::new(pt, saved));
    let fullscreen = save::ini_read(&text, "display", "fullscreen")
        .and_then(|v| v.trim().parse::<f64>().ok())
        .is_none_or(|v| v != 0.0);
    commands.insert_resource(DisplayPending(display_apply_pending(fullscreen)));
}

/// `global.display_pending`: steps left in the window-mode switch (see [`display_step_tick`]).
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct DisplayPending(pub u32);

/// What `display_step` does to the window on a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayAction {
    /// `window_set_size(display_window_size())`
    SetSize,
    /// `window_center()`
    Centre,
}

/// `display_apply`'s countdown: none in fullscreen, 4 steps for a window.
pub fn display_apply_pending(fullscreen: bool) -> u32 {
    if fullscreen {
        0
    } else {
        4
    }
}

/// One step of `display_step`'s countdown.
pub fn display_step_tick(pending: &mut u32) -> Option<DisplayAction> {
    if *pending == 0 {
        return None;
    }
    *pending -= 1;
    match *pending {
        2 => Some(DisplayAction::SetSize),
        0 => Some(DisplayAction::Centre),
        _ => None,
    }
}

// port: display_apply
/// `display_apply`: switch the window mode when it differs from the setting, start the countdown.
pub fn display_apply(win: &mut Window, fullscreen: bool) -> u32 {
    let is_full = !matches!(win.mode, WindowMode::Windowed);
    if is_full != fullscreen {
        win.mode = if fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Current)
        } else {
            WindowMode::Windowed
        };
    }
    display_apply_pending(fullscreen)
}

// port: display_step
/// obj_game Step's `display_step`: two steps into a switch to a window it takes the chosen size
/// (logical points, ruling R23), at the end it is centred.
pub fn display_step(
    mut pending: Option<ResMut<DisplayPending>>,
    sizes: Option<Res<DisplaySizes>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(pending) = pending.as_mut() else {
        return;
    };
    let Some(action) = display_step_tick(&mut pending.0) else {
        return;
    };
    let Ok(mut win) = windows.single_mut() else {
        return;
    };
    match action {
        DisplayAction::SetSize => {
            if let Some(s) = sizes {
                let (w, h) = s.display_window_size();
                win.resolution.set(w as f32, h as f32);
            }
        }
        DisplayAction::Centre => {
            win.position = WindowPosition::Centered(MonitorSelection::Current);
        }
    }
}

/// `display_cycle_size(dir)`: pick the size, `display_apply` (a window takes the size two steps
/// later and is centred at the end of the countdown, [`display_step`]), `display_save`.
pub fn apply_cycle_size(
    mut msgs: MessageReader<CycleWindowSize>,
    sizes: Option<ResMut<DisplaySizes>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut display: ResMut<Display>,
    mut pending: Option<ResMut<DisplayPending>>,
    dir: Res<SaveDir>,
) {
    let Some(mut sizes) = sizes else {
        msgs.clear();
        return;
    };
    let mut changed = false;
    for m in msgs.read() {
        sizes.display_cycle_size(m.0);
        changed = true;
    }
    if !changed {
        return;
    }
    let (w, h) = sizes.display_window_size();
    display.0.window_w = w as _;
    display.0.window_h = h as _;
    if let Ok(mut win) = windows.single_mut() {
        let p = display_apply(&mut win, display.0.fullscreen);
        if let Some(pending) = pending.as_mut() {
            pending.0 = p;
        }
    }
    if let Err(e) = write_display_settings(&dir.0, &display.0) {
        warn!("settings.ini: {e}");
    }
}

/// Ask the display to switch fullscreen on or off; the choice is written to settings.ini.
#[derive(Message, Clone, Copy, Debug)]
pub struct SetFullscreen(pub bool);

/// The primary window for the given settings (`display.init`): fullscreen below the notch, or a
/// window of `window_w×window_h` logical points, centred. Ruling R23: LÖVE (highdpi, no DPI
/// scaling) applies the settings.ini size as points (`gm/display.lua`), so a 1366×768 window is
/// 2732×1536 pixels on Retina; the size is not divided by the scale factor. (Before the window
/// exists Bevy's scale factor is 1, and winit creates it with this logical size.)
pub fn primary_window(settings: &DisplaySettings, _screen: Option<&ScreenInfo>) -> Window {
    let mut w = Window {
        title: "Mycelarium".into(),
        resolution: WindowResolution::new(settings.window_w.max(1), settings.window_h.max(1)),
        position: WindowPosition::Centered(MonitorSelection::Primary),
        ..default()
    };
    if settings.fullscreen {
        w.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Primary);
    }
    w
}

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SetFullscreen>()
            .add_message::<CycleWindowSize>()
            .add_systems(Startup, (init_display_sizes, install_quit_hook))
            .add_systems(FixedUpdate, display_step)
            .add_systems(
                Update,
                (
                    display_quit_requests,
                    display_keys,
                    apply_set_fullscreen,
                    apply_cycle_size,
                )
                    .chain(),
            );
    }
}

/// Cmd+Esc quits (as in the LÖVE port, main.lua); so does Cmd+Q when it reaches the window as a
/// key (normally the app menu's Quit takes it first, see [`install_quit_hook`]). Both go through
/// `AppExit`, so `ui::save_on_exit` saves (obj_game Other_3). The original has no fullscreen
/// hotkey: the screen mode changes only from the settings menu.
pub fn display_keys(keys: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    let cmd = keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]);
    if cmd && keys.any_just_pressed([KeyCode::Escape, KeyCode::KeyQ]) {
        exit.write(AppExit::Success);
    }
}

/// A Quit asked for by the OS (app menu Quit / Cmd+Q, Dock Quit) and not yet turned into
/// `AppExit`.
static QUIT_PENDING: AtomicBool = AtomicBool::new(false);
/// Some Quit was asked for since launch.
static QUIT_ASKED: AtomicBool = AtomicBool::new(false);

/// Ask the app to quit through `AppExit` on its next update, so the Game End save runs (winit's
/// own `terminate:` path ends the app in `applicationWillTerminate:`, where Bevy only clears the
/// world without running a schedule). Returns whether a Quit was asked before: the
/// `applicationShouldTerminate:` hook then lets the terminate through, so a stuck app still
/// quits on the second Cmd+Q.
pub fn request_quit() -> bool {
    QUIT_PENDING.store(true, Ordering::SeqCst);
    QUIT_ASKED.swap(true, Ordering::SeqCst)
}

/// Turns a [`request_quit`] into `AppExit` (once per request).
pub fn display_quit_requests(mut exit: MessageWriter<AppExit>) {
    if QUIT_PENDING.swap(false, Ordering::SeqCst) {
        exit.write(AppExit::Success);
    }
}

/// macOS: Cmd+Q is the key equivalent of the Quit item in winit's default app menu, which sends
/// `terminate:` and never reaches the window as a key. Add `applicationShouldTerminate:` to
/// winit's app delegate: the first terminate is cancelled and becomes [`request_quit`] (then
/// `AppExit`, the save, and winit's normal exit); a second one while the first is pending
/// terminates at once. Logging out also asks the app to terminate: the first request cancels
/// that logout while the game saves and quits, a retried logout goes through.
#[cfg(target_os = "macos")]
pub fn install_quit_hook() {
    use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
    use objc2::sel;

    // NSApplicationTerminateReply: NSTerminateCancel = 0, NSTerminateNow = 1
    unsafe extern "C-unwind" fn should_terminate(
        _this: *mut AnyObject,
        _cmd: Sel,
        _sender: *mut AnyObject,
    ) -> usize {
        if request_quit() {
            1
        } else {
            0
        }
    }

    let Some(cls) = AnyClass::get(c"WinitApplicationDelegate") else {
        // no winit event loop (tests, headless runs)
        return;
    };
    let f: unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> usize =
        should_terminate;
    // SAFETY: the function matches the `Q@:@` signature of
    // `-(NSApplicationTerminateReply)applicationShouldTerminate:(NSApplication *)sender`, and
    // the class is winit's registered delegate class, which does not implement that method.
    let added = unsafe {
        let imp: Imp = std::mem::transmute::<_, Imp>(f);
        objc2::ffi::class_addMethod(
            cls as *const AnyClass as *mut AnyClass,
            sel!(applicationShouldTerminate:),
            imp,
            c"Q@:@".as_ptr(),
        )
    };
    if !added.as_bool() {
        warn!("applicationShouldTerminate: already defined; Cmd+Q may quit without saving");
    }
}

#[cfg(not(target_os = "macos"))]
pub fn install_quit_hook() {}

// port: display_set_fullscreen
/// `display_set_fullscreen(on)` (the settings menu): the setting, `display_apply` (the window
/// takes its size two steps later, `display_step`), `display_save`.
fn apply_set_fullscreen(
    mut msgs: MessageReader<SetFullscreen>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut display: ResMut<Display>,
    mut pending: Option<ResMut<DisplayPending>>,
    dir: Res<SaveDir>,
) {
    let Some(SetFullscreen(on)) = msgs.read().last().copied() else {
        return;
    };
    if let Ok(mut win) = windows.single_mut() {
        let p = display_apply(&mut win, on);
        if let Some(pending) = pending.as_mut() {
            pending.0 = p;
        }
    }
    display.0.fullscreen = on;
    if let Err(e) = write_display_settings(&dir.0, &display.0) {
        warn!("settings.ini: {e}");
    }
}
