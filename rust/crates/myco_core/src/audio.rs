//! The logic side of `scr_audio`. The audio engine itself (voices, gains, music, ambience, loops,
//! hyphae sounds) lives in the app (`myco_app::audio`); logic only tells it what happened through
//! [`SimEvent::Audio`] / [`SimEvent::Sound`]. This module holds what both sides need:
//!
//! - the sound lengths (`audio_sound_length`, measured by the LÖVE port), so the hyphae plan is the
//!   same in logic and in the engine;
//! - [`hyphae_plan`], the deterministic part of `hyphae_sound_begin`, and [`HyphaePlan::done_at`],
//!   the frame on which the original's `s.done` turns true for a non-positional sound (mycelium
//!   growths stay in `myc_active` until then);
//! - event builders that roll the original's `choose(...)` with the game RNG.

use crate::balance::vis;
use crate::events::{AudioCall, SimEvent, SoundCue};
use crate::gml;
use crate::rng::Rng;

/// The game's fixed step (`game_get_speed(gamespeed_fps)`).
pub const FPS: f64 = 60.0;

/// `audio_sound_length` of every sound, seconds, as the LÖVE port decodes them
/// (golden `audio.json` → `lengths`).
pub const SOUND_LENGTHS: &[(&str, f64)] = &[
    ("bird", 2.2559863945578233),
    ("boar_tremor", 9.43201814058957),
    ("creak", 4.008004535147392),
    ("explosion", 1.392018140589569),
    ("forest", 176.66401360544216),
    ("guitar", 87.84),
    ("hang_guitar_01", 71.23201814058957),
    ("hyphae_end_long", 0.8880045351473923),
    ("hyphae_end_mid", 0.5280045351473923),
    ("hyphae_end_short", 0.36),
    ("hyphae_middle", 1.7280045351473923),
    ("hyphae_midlle", 3.0),
    ("hyphae_start_1", 0.72),
    ("hyphae_start_2", 0.7680045351473923),
    ("leaves", 31.224013605442178),
    ("mush_grow_1", 0.45598639455782314),
    ("mush_grow_2", 0.45598639455782314),
    ("mush_grow_3", 0.45598639455782314),
    ("mushrooms", 161.23201814058956),
    ("rain01", 86.41732426303855),
    ("root", 2.562517006802721),
    ("thunder", 13.4418820861678),
    ("water_touch", 1.322403628117914),
    ("woodpecker", 3.3840136054421768),
];

/// `audio_sound_length(name)` in seconds; `None` for an unknown sound.
pub fn sound_length(name: &str) -> Option<f64> {
    SOUND_LENGTHS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|&(_, l)| l)
}

/// The two start sounds of a hyphae growth (`choose(hyphae_start_1, hyphae_start_2)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HyphaeStart {
    One,
    Two,
}

impl HyphaeStart {
    pub fn sound(self) -> &'static str {
        match self {
            HyphaeStart::One => "hyphae_start_1",
            HyphaeStart::Two => "hyphae_start_2",
        }
    }
}

/// What `hyphae_sound_begin` decides before playing anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HyphaePlan {
    pub start: HyphaeStart,
    pub end_sound: &'static str,
    /// Frame (of `t`) at which the looping middle starts: half the start sound.
    pub middle_at: f64,
    /// Frame at which the end sound plays: `round(total * hyphae_end_at)`.
    pub end_at: f64,
    /// Too short for a middle part (then the short end sound).
    pub tiny: bool,
}

/// The deterministic part of `hyphae_sound_begin(len, total, ...)` with the start sound chosen.
pub fn hyphae_plan(len: f64, total: f64, start: HyphaeStart) -> HyphaePlan {
    let mut end_sound = "hyphae_end_short";
    if len >= vis("hyphae_len_long") {
        end_sound = "hyphae_end_long";
    } else if len >= vis("hyphae_len_mid") {
        end_sound = "hyphae_end_mid";
    }
    let start_len = sound_length(start.sound()).unwrap_or(0.0);
    let middle_at = gml::round(start_len * 0.5 * FPS);
    let end_at = gml::round(total * gml::clamp(vis("hyphae_end_at"), 0.0, 1.0));
    let tiny = len < vis("hyphae_len_tiny") || middle_at >= end_at;
    if tiny {
        end_sound = "hyphae_end_short";
    }
    HyphaePlan {
        start,
        end_sound,
        middle_at,
        end_at,
        tiny,
    }
}

impl HyphaePlan {
    /// The first `t` (steps run with t = 1, 2, ...) on which a non-positional hyphae sound is
    /// done in the original. `sfx_on`: `volume_sfx > 0` when the middle would start (muted, the
    /// middle never plays and the sound ends with the end cue). Logic has no volume, so it passes
    /// `true`: muted, a growth then stays active `hyphae_middle_fade_frames` longer than in the
    /// original, which only affects its sound.
    pub fn done_at(&self, sfx_on: bool) -> f64 {
        let ended = self.end_at.max(1.0);
        if sfx_on && !self.tiny {
            ended + vis("hyphae_middle_fade_frames").max(1.0)
        } else {
            ended
        }
    }
}

/// `hyphae_sound_begin(len, total, x, y)` as an event; rolls the start sound with `rng`. Also
/// returns the plan, so logic can keep a finished growth active until `plan.done_at(true)`.
pub fn hyphae_begin(
    rng: &mut Rng,
    id: u64,
    len: f64,
    total: f64,
    at: Option<(f64, f64)>,
) -> (SimEvent, HyphaePlan) {
    let start = rng.choose(&[HyphaeStart::One, HyphaeStart::Two]);
    let event = SimEvent::Audio(AudioCall::HyphaeBegin {
        id,
        len,
        total,
        at,
        start,
    });
    (event, hyphae_plan(len, total, start))
}

/// `mushroom_grow_sound(x, y, vol)` as an event; rolls the sound with `rng`.
pub fn mushroom_grow(rng: &mut Rng, at: Option<(f64, f64)>, volume: Option<f64>) -> SimEvent {
    let sound = rng.choose(&["mush_grow_1", "mush_grow_2", "mush_grow_3"]);
    SimEvent::Audio(AudioCall::MushroomGrow { sound, at, volume })
}

/// `water_touch_sound(x, y)` as an event.
pub fn water_touch(at: Option<(f64, f64)>) -> SimEvent {
    SimEvent::Audio(AudioCall::WaterTouch { at })
}

/// `sfx_play_at(snd, x, y, vol, reach)` as an event (`reach: None` = `sound_world_reach_views`).
pub fn sfx_at(name: &str, x: f64, y: f64, vol: f64, reach: Option<f64>) -> SimEvent {
    let mut cue = SoundCue::new(name).at(x, y).gain(vol);
    cue.reach = reach;
    SimEvent::Sound(cue)
}
