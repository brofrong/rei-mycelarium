//! What the simulation tells the app each frame: resource changes (for floating numbers),
//! refused node placements, sounds, saves and tutorial tips. Logic tasks extend [`SimEvent`]
//! with their own variants; the app drains them with `Sim::drain_events`.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKind {
    Food,
    Water,
    UpgradePoints,
}

/// Why a node (right click / quick node) was not placed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeReject {
    NotOnMycelium,
    InWater,
    TooCloseToNode,
    NotEnoughFood { need: i64 },
    Blocked,
}

/// A one-shot sound. `name` is the sound's name in `sounds.json` (as the original passes it to
/// `sfx_play` and friends); `x`/`y` are world coordinates for positional sounds.
///
/// The app plays it like the original: without a position `sfx_play(name)` (then, for `gain` other
/// than 1, `audio_sound_gain(voice, volume_sfx * clamp(gain, 0, 1))`); with a position
/// `sfx_play_at(name, x, y, gain, reach)`, where `reach: None` is `sound_world_reach_views`.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundCue {
    pub name: String,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub gain: f64,
    pub pitch: f64,
    pub reach: Option<f64>,
}

impl SoundCue {
    /// A non-positional sound at full gain and normal pitch.
    pub fn new(name: impl Into<String>) -> SoundCue {
        SoundCue {
            name: name.into(),
            x: None,
            y: None,
            gain: 1.0,
            pitch: 1.0,
            reach: None,
        }
    }

    pub fn at(mut self, x: f64, y: f64) -> SoundCue {
        self.x = Some(x);
        self.y = Some(y);
        self
    }

    /// The `vol` argument of `sfx_play_at` (or the gain override after `sfx_play`).
    pub fn gain(mut self, gain: f64) -> SoundCue {
        self.gain = gain;
        self
    }

    /// The `reach` argument of `sfx_play_at`, in views.
    pub fn reach(mut self, reach: f64) -> SoundCue {
        self.reach = Some(reach);
        self
    }
}

/// Calls into `scr_audio` that logic makes and that keep state in the audio engine (the app's
/// `audio.rs`). Logic emits them with [`SimEvent::Audio`]; random choices the original makes
/// inside these functions are rolled by logic with `sim.rng` (see `crate::audio`), so the gameplay
/// RNG advances as in the original.
#[derive(Clone, Debug, PartialEq)]
pub enum AudioCall {
    /// `water_touch_sound(x, y)`; `at: None` is the non-positional form.
    WaterTouch { at: Option<(f64, f64)> },
    /// `mushroom_grow_sound(x, y, vol)`; `sound` is the `choose(mush_grow_1, _2, _3)` result,
    /// `volume: None` is `vis("mush_grow_volume")`.
    MushroomGrow {
        sound: &'static str,
        at: Option<(f64, f64)>,
        volume: Option<f64>,
    },
    /// `hyphae_sound_begin(len, total, x, y)` stored under `id` (the original keeps the struct on
    /// the growth / leaf / worm). `start` is the `choose(hyphae_start_1, hyphae_start_2)` result.
    HyphaeBegin {
        id: u64,
        len: f64,
        total: f64,
        at: Option<(f64, f64)>,
        start: crate::audio::HyphaeStart,
    },
    /// `hyphae_sound_step(s, t)` for the sound `id`.
    HyphaeStep { id: u64, t: f64 },
    /// `hyphae_sound_stop(s)` for the sound `id`.
    HyphaeStop { id: u64 },
    /// `sound_loop_start(sound)` when `id` has no live loop (never started or already faded
    /// out), then `sound_loop_follow(l, x, y, volume, reach, fade_in, fade_out)`. Sent every frame
    /// the source exists (the boar's tremor); a loop not followed for 3 frames fades out.
    LoopFollow {
        id: u64,
        sound: &'static str,
        x: f64,
        y: f64,
        volume: f64,
        reach: f64,
        fade_in: f64,
        fade_out: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum SimEvent {
    /// `source`: a stable id of what produced the change (one floating number per source/frame).
    Resource {
        kind: ResourceKind,
        delta: f64,
        x: f64,
        y: f64,
        source: u64,
    },
    NodeRejected {
        reason: NodeReject,
        x: f64,
        y: f64,
    },
    Sound(SoundCue),
    /// Stateful audio calls (hyphae growth sounds, loops, water touch, mushroom growth).
    Audio(AudioCall),
    Saved,
    GameOver,
    TutorialTip {
        key: String,
    },
}
