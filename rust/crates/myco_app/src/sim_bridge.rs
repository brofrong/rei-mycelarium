//! The simulation as a Bevy resource, stepped in `FixedUpdate` at exactly 60 Hz.
//!
//! Input is gathered every rendered frame (`Update`) into [`PendingInput`]. Edges (`pressed`,
//! `released`, typed text, key presses) are latched there until the next fixed step consumes them,
//! so a click in a frame without a fixed step is not lost, and one with several fixed steps is
//! seen once. Held state (`down`) is the current state at every step. The cursor is mapped to the
//! world and GUI at step time through [`Cameras`]. Events the simulation emits are re-sent as
//! [`SimEventMsg`] messages (Bevy's buffered events).

use crate::camera::Cameras;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::input::ButtonState as BevyButton;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use myco_core::events::SimEvent;
use myco_core::sim::{ButtonState, Command, Key, Sim, StepInput};

#[derive(Resource)]
pub struct SimRes(pub Sim);

/// One simulation event, re-sent after each fixed step.
#[derive(Message, Clone, Debug)]
pub struct SimEventMsg(pub SimEvent);

/// Input waiting for the next fixed step. Other systems (UI, autopilot) add commands and flags.
#[derive(Resource, Default, Clone, Debug)]
pub struct PendingInput {
    /// cursor in screen coordinates (logical window px); `None` outside the window
    pub cursor: Option<Vec2>,
    /// autopilot cursor (screen px) used instead of the real mouse
    pub cursor_override: Option<Vec2>,
    pub lmb: ButtonState,
    pub rmb: ButtonState,
    pub keys_pressed: Vec<Key>,
    pub keys_down: Vec<Key>,
    pub text: String,
    pub commands: Vec<Command>,
    /// set by the UI each frame: the cursor is over the HUD / a menu
    pub ui_captured: bool,
    pub tasks_panel_open: bool,
    pub tasks_popup_open: bool,
    /// `global.menu_open`: the upgrade-tree screen is open (set by `ui::tree`)
    pub menu_open: bool,
}

impl PendingInput {
    pub fn button(&mut self, b: MouseButton) -> Option<&mut ButtonState> {
        match b {
            MouseButton::Left => Some(&mut self.lmb),
            MouseButton::Right => Some(&mut self.rmb),
            _ => None,
        }
    }

    /// GameMaker semantics (`gm/input.lua mouse_event`): press latches `pressed` and sets
    /// `down`, release latches `released` and clears `down`.
    pub fn mouse_event(&mut self, b: MouseButton, down: bool) {
        if let Some(s) = self.button(b) {
            if down {
                s.pressed = true;
                s.down = true;
            } else {
                s.down = false;
                s.released = true;
            }
        }
    }

    /// `gm/input.lua key_event`: `pressed` only on the transition to down.
    pub fn key_event(&mut self, k: Key, down: bool) {
        if down {
            if !self.keys_down.contains(&k) {
                if !self.keys_pressed.contains(&k) {
                    self.keys_pressed.push(k);
                }
                self.keys_down.push(k);
            }
        } else {
            self.keys_down.retain(|d| *d != k);
        }
    }

    /// Build the step input and clear the latched edges.
    pub fn take_step(&mut self, cams: &Cameras) -> StepInput {
        let screen = self.cursor_override.or(self.cursor);
        let input = StepInput {
            cursor: screen.map(|p| cams.screen_to_world(p).into()),
            cursor_gui: screen.map(|p| cams.screen_to_gui(p).into()),
            lmb: self.lmb,
            rmb: self.rmb,
            keys_pressed: std::mem::take(&mut self.keys_pressed),
            keys_down: self.keys_down.clone(),
            text: std::mem::take(&mut self.text),
            ui_captured: self.ui_captured,
            view: cams.game_view,
            commands: std::mem::take(&mut self.commands),
            tasks_panel_open: self.tasks_panel_open,
            tasks_popup_open: self.tasks_popup_open,
            // HUD/menu state the HUD tasks (T17) will provide
            menu_open: self.menu_open,
            debug_open: false,
            logo_open: false,
            hud_hidden: false,
        };
        for b in [&mut self.lmb, &mut self.rmb] {
            b.pressed = false;
            b.released = false;
        }
        input
    }
}

/// When present, every `StepInput` given to the simulation is recorded (tests, debugging).
#[derive(Resource, Default, Debug)]
pub struct StepHistory(pub Vec<StepInput>);

/// The window is treated as focused (`MYCO_FOCUS`): no pause on focus loss.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct ForceFocus(pub bool);

/// Set by the UI (`ui_blocking`, ruling R9): a menu is open, focus loss does not pause.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct UiBlocking(pub bool);

/// Fixed-step system sets, in order.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    /// before the step: autopilot script, focus check
    Prepare,
    Step,
    /// after the step: dumps, render-side syncing
    After,
}

pub struct SimBridgePlugin;

impl Plugin for SimBridgePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0))
            .init_resource::<PendingInput>()
            .init_resource::<ForceFocus>()
            .init_resource::<UiBlocking>()
            .add_message::<SimEventMsg>()
            .configure_sets(
                FixedUpdate,
                (SimSet::Prepare, SimSet::Step, SimSet::After).chain(),
            )
            .add_systems(PreUpdate, collect_input.after(bevy::input::InputSystems))
            .add_systems(
                FixedUpdate,
                (
                    focus_step.in_set(SimSet::Prepare),
                    step_sim.in_set(SimSet::Step),
                ),
            );
    }
}

/// `KeyCode` → the game's key (`gm/input.lua KEYS`, physical layout).
pub fn key_from_code(code: KeyCode) -> Option<Key> {
    use KeyCode::*;
    Some(match code {
        ArrowLeft => Key::Left,
        ArrowRight => Key::Right,
        ArrowUp => Key::Up,
        ArrowDown => Key::Down,
        ShiftLeft | ShiftRight => Key::Shift,
        AltLeft | AltRight => Key::Alt,
        Enter | NumpadEnter => Key::Enter,
        Escape => Key::Escape,
        Backspace => Key::Backspace,
        F1 => Key::F1,
        F2 => Key::F2,
        Space => Key::Char(' '),
        KeyA => Key::Char('A'),
        KeyB => Key::Char('B'),
        KeyC => Key::Char('C'),
        KeyD => Key::Char('D'),
        KeyE => Key::Char('E'),
        KeyF => Key::Char('F'),
        KeyG => Key::Char('G'),
        KeyH => Key::Char('H'),
        KeyI => Key::Char('I'),
        KeyJ => Key::Char('J'),
        KeyK => Key::Char('K'),
        KeyL => Key::Char('L'),
        KeyM => Key::Char('M'),
        KeyN => Key::Char('N'),
        KeyO => Key::Char('O'),
        KeyP => Key::Char('P'),
        KeyQ => Key::Char('Q'),
        KeyR => Key::Char('R'),
        KeyS => Key::Char('S'),
        KeyT => Key::Char('T'),
        KeyU => Key::Char('U'),
        KeyV => Key::Char('V'),
        KeyW => Key::Char('W'),
        KeyX => Key::Char('X'),
        KeyY => Key::Char('Y'),
        KeyZ => Key::Char('Z'),
        Digit0 => Key::Char('0'),
        Digit1 => Key::Char('1'),
        Digit2 => Key::Char('2'),
        Digit3 => Key::Char('3'),
        Digit4 => Key::Char('4'),
        Digit5 => Key::Char('5'),
        Digit6 => Key::Char('6'),
        Digit7 => Key::Char('7'),
        Digit8 => Key::Char('8'),
        Digit9 => Key::Char('9'),
        Numpad0 => Key::Numpad(0),
        Numpad1 => Key::Numpad(1),
        Numpad2 => Key::Numpad(2),
        Numpad3 => Key::Numpad(3),
        Numpad4 => Key::Numpad(4),
        Numpad5 => Key::Numpad(5),
        Numpad6 => Key::Numpad(6),
        Numpad7 => Key::Numpad(7),
        Numpad8 => Key::Numpad(8),
        Numpad9 => Key::Numpad(9),
        _ => return None,
    })
}

/// LÖVE key name (`MYCO_INPUT key:<name>`) → the game's key (`gm/input.lua keycode`).
pub fn key_from_love_name(name: &str) -> Option<Key> {
    Some(match name {
        "left" => Key::Left,
        "right" => Key::Right,
        "up" => Key::Up,
        "down" => Key::Down,
        "lshift" | "rshift" => Key::Shift,
        "lalt" | "ralt" => Key::Alt,
        "return" | "kpenter" => Key::Enter,
        "escape" => Key::Escape,
        "backspace" => Key::Backspace,
        "f1" => Key::F1,
        "f2" => Key::F2,
        "space" => Key::Char(' '),
        _ => {
            if let Some(d) = name.strip_prefix("kp") {
                let n: u8 = d.parse().ok().filter(|n| *n <= 9)?;
                return Some(Key::Numpad(n));
            }
            let mut cs = name.chars();
            let c = cs.next()?;
            if cs.next().is_some() || !c.is_ascii_alphanumeric() {
                return None;
            }
            Key::Char(c.to_ascii_uppercase())
        }
    })
}

/// Every frame: mouse/key edges and text into `PendingInput`.
fn collect_input(
    mut pending: ResMut<PendingInput>,
    mut mouse: MessageReader<MouseButtonInput>,
    mut keys: MessageReader<KeyboardInput>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    for ev in mouse.read() {
        pending.mouse_event(ev.button, ev.state == BevyButton::Pressed);
    }
    for ev in keys.read() {
        let down = ev.state == BevyButton::Pressed;
        if down {
            if let Some(t) = &ev.text {
                if !t.chars().any(char::is_control) {
                    pending.text.push_str(t);
                }
            }
        }
        if ev.repeat {
            continue;
        }
        if let Some(k) = key_from_code(ev.key_code) {
            pending.key_event(k, down);
        }
    }
    pending.cursor = windows.single().ok().and_then(Window::cursor_position);
}

/// `focus_step` (the app's half of obj_game Step): losing focus opens the pause menu.
fn focus_step(
    windows: Query<&Window, With<PrimaryWindow>>,
    force: Res<ForceFocus>,
    blocking: Res<UiBlocking>,
    sim: Res<SimRes>,
    mut pending: ResMut<PendingInput>,
) {
    let focused = windows.single().map(|w| w.focused).unwrap_or(true);
    if focused || force.0 || myco_core::balance::vis("pause_on_focus_loss") <= 0.0 || blocking.0 {
        return;
    }
    if !sim.0.pause_open && !pending.commands.contains(&Command::SetPaused(true)) {
        pending.commands.push(Command::SetPaused(true));
    }
}

/// One simulation step per fixed tick.
fn step_sim(
    mut sim: ResMut<SimRes>,
    mut pending: ResMut<PendingInput>,
    mut cams: ResMut<Cameras>,
    history: Option<ResMut<StepHistory>>,
    mut out: MessageWriter<SimEventMsg>,
) {
    let input = pending.take_step(&cams);
    sim.0.step(&input);
    if let Some(mut h) = history {
        h.0.push(input);
    }
    // the logic's camera (top-left corner) moves the 1× game view
    let (x, y) = (sim.0.camera.x, sim.0.camera.y);
    cams.game_view.x = x;
    cams.game_view.y = y;
    out.write_batch(sim.0.drain_events().into_iter().map(SimEventMsg));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn love_key_names() {
        assert_eq!(key_from_love_name("space"), Some(Key::Char(' ')));
        assert_eq!(key_from_love_name("a"), Some(Key::Char('A')));
        assert_eq!(key_from_love_name("7"), Some(Key::Char('7')));
        assert_eq!(key_from_love_name("kp3"), Some(Key::Numpad(3)));
        assert_eq!(key_from_love_name("return"), Some(Key::Enter));
        assert_eq!(key_from_love_name("tab"), None);
    }

    #[test]
    fn key_pressed_only_on_transition() {
        let mut p = PendingInput::default();
        p.key_event(Key::Left, true);
        p.key_event(Key::Left, true);
        let s = p.take_step(&Cameras::default());
        assert_eq!(s.keys_pressed, vec![Key::Left]);
        assert_eq!(s.keys_down, vec![Key::Left]);
        let s = p.take_step(&Cameras::default());
        assert!(s.keys_pressed.is_empty());
        assert_eq!(s.keys_down, vec![Key::Left]);
    }
}
