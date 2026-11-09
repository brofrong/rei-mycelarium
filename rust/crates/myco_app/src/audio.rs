//! Port of `scr_audio` (game/myco/gen/meta_1.lua): one-shot sounds placed relative to the camera,
//! loops that follow a world point, the ambience, rain and forest beds, hyphae growth sounds, the
//! music queue and the volumes stored in `settings.ini`.
//!
//! [`Audio`] is the engine. It is written against [`AudioOut`], a mirror of the GameMaker audio
//! builtins scr_audio calls (`audio_play_sound[_on]`, `audio_sound_gain`, emitters, loop points),
//! so the port reads like the original and the tests run it against a recording fake (see
//! `tests/audio.rs` and the golden script `rust/golden/scripts/audio.lua`). The Bevy implementation
//! of [`AudioOut`] is [`backend::BevyOut`].
//!
//! Inputs: `SimEvent::Sound` / `SimEvent::Audio` from the simulation (logic decides *what* sounds,
//! including the original's random choices, see `myco_core::audio`), the weather intensity from
//! `sim.weather`, and the 1× game view (`view_camera[0]`) from `Cameras`. [`AudioRequest`] is how
//! the settings UI changes and saves the volumes.
//!
//! Gain: each voice gets `asset volume × audio_sound_gain` once, like GameMaker. (The LÖVE port
//! applies the asset gain twice; that is not reproduced.)

pub mod backend;

use std::collections::HashMap;
use std::io;
use std::path::Path;

use bevy::prelude::*;
use myco_core::audio::{hyphae_plan, HyphaeStart, FPS};
use myco_core::balance::vis;
use myco_core::events::{AudioCall, SimEvent, SoundCue};
use myco_core::gml::{clamp, lerp, point_distance, round, sign};
use myco_core::rng::Rng;
use myco_core::save;
use myco_core::sim::ViewRect;

use crate::camera::Cameras;
use crate::display::SaveDir;
use crate::sim_bridge::{SimEventMsg, SimRes, SimSet};

pub use backend::BevyOut;

/// A playing sound instance (GameMaker voice id).
pub type Voice = u64;
/// An audio emitter id.
pub type Emitter = u64;

/// The GameMaker audio builtins scr_audio uses. Sounds are named as in `sounds.json`.
pub trait AudioOut {
    /// `audio_play_sound(snd, prio, loop)` or, with an emitter, `audio_play_sound_on`.
    fn play(&mut self, snd: &str, looping: bool, emitter: Option<Emitter>) -> Voice;
    fn is_playing(&self, v: Voice) -> bool;
    /// `audio_stop_sound(voice)`.
    fn stop(&mut self, v: Voice);
    /// `audio_stop_sound(asset)`: every instance of the sound.
    fn stop_sound(&mut self, snd: &str);
    /// `audio_sound_gain(voice, gain, ms)`: linear fade over `ms` (0 = at once).
    fn gain(&mut self, v: Voice, gain: f64, ms: f64);
    /// `audio_sound_pitch(voice, pitch)` (only for `SoundCue::pitch`; scr_audio never changes it).
    fn pitch(&mut self, v: Voice, pitch: f64);
    fn loop_start(&mut self, v: Voice, pos: f64);
    fn loop_end(&mut self, v: Voice, pos: f64);
    /// `audio_sound_length(snd)`, seconds.
    fn sound_length(&self, snd: &str) -> f64;
    fn emitter_create(&mut self) -> Emitter;
    fn emitter_free(&mut self, e: Emitter);
    /// `audio_emitter_position`. scr_audio puts emitters on the unit circle around the listener,
    /// `(pan, 0, sqrt(1 - pan²))`, with falloff off: `x` is the stereo pan.
    fn emitter_position(&mut self, e: Emitter, x: f64, y: f64, z: f64);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeKind {
    Sfx,
    Music,
}

pub const MUSIC_TRACKS: [&str; 3] = ["hang_guitar_01", "guitar", "mushrooms"];
const SOUND_EMITTERS: usize = 8;

/// `sound_world_place` result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Place {
    pub gain: f64,
    pub pan: f64,
}

struct Ambience {
    sound: &'static str,
    key: &'static str,
    panned: bool,
    emitter: Emitter,
    timer: f64,
}

/// The struct `hyphae_sound_begin` returns.
#[derive(Clone, Debug, PartialEq)]
pub struct HyphaeSound {
    pub total: f64,
    pub placed: bool,
    pub x: f64,
    pub y: f64,
    start_voice: Option<Voice>,
    middle_voice: Option<Voice>,
    middle_allowed: bool,
    middle_at: f64,
    end_at: f64,
    end_sound: &'static str,
    end_voice: Option<Voice>,
    fade_until: f64,
    pub ended: bool,
    pub done: bool,
}

/// The struct `sound_loop_start` returns; `key` is the logic id of its source.
struct SoundLoop {
    key: u64,
    sound: &'static str,
    emitter: Emitter,
    voice: Option<Voice>,
    fade: f64,
    idle: f64,
    x: f64,
    y: f64,
    volume: f64,
    reach: f64,
    fade_in: f64,
    fade_out: f64,
    done: bool,
}

/// Engine state: the `global.*` variables of scr_audio.
#[derive(Resource)]
pub struct Audio {
    pub volume_sfx: f64,
    pub volume_music: f64,
    /// `view_camera[0]` (the 1× game view); `None` = no camera (full gain, centred).
    pub view: Option<ViewRect>,
    music_bag: Vec<&'static str>,
    music_last: Option<&'static str>,
    pub music_voice: Option<Voice>,
    pub rain_voice: Option<Voice>,
    pub forest_voice: Option<Voice>,
    pub forest_duck: f64,
    pub water_touch_cooldown: f64,
    ambience: Vec<Ambience>,
    sound_emitters: Vec<Emitter>,
    sound_emitter_next: usize,
    sound_loops: Vec<SoundLoop>,
    hyphae: HashMap<u64, HyphaeSound>,
    /// The original rolls ambience timers, pans and the music shuffle with the global RNG; here
    /// they have their own (sequences need not match, see the spec).
    rng: Rng,
}

/// `max(a, b, c)` as GameMaker's variadic `max`.
fn max3(a: f64, b: f64, c: f64) -> f64 {
    a.max(b).max(c)
}

/// `ini_read_real`: the value parsed as a number, else `def` (Lua `tonumber` semantics: no
/// `inf`/`nan` words).
fn ini_read_real(text: &str, section: &str, key: &str, def: f64) -> f64 {
    save::ini_read(text, section, key)
        .and_then(|s| {
            let t = s.trim();
            if t.chars()
                .any(|c| c.is_ascii_alphabetic() && c != 'e' && c != 'E')
            {
                return None;
            }
            t.parse::<f64>().ok()
        })
        .unwrap_or(def)
}

// port: sound_camera_gain
/// Gain of a sound at world `(x, y)`: 1 inside the view, falling linearly to 0 at
/// `view_w * sound_camera_fade_views` from its edge.
pub fn sound_camera_gain(view: Option<&ViewRect>, x: f64, y: f64) -> f64 {
    let Some(v) = view else { return 1.0 };
    let dx = max3(v.x - x, 0.0, x - (v.x + v.w));
    let dy = max3(v.y - y, 0.0, y - (v.y + v.h));
    let fade = (v.w * vis("sound_camera_fade_views")).max(1.0);
    clamp(1.0 - point_distance(0.0, 0.0, dx, dy) / fade, 0.0, 1.0)
}

// port: sound_world_place
/// Gain and pan of a sound at world `(x, y)`: distance from the view in view sizes against
/// `fade_views`; off to the side it pans from `sound_pan_edge` to `sound_pan_far`.
pub fn sound_world_place(view: Option<&ViewRect>, x: f64, y: f64, fade_views: f64) -> Place {
    let Some(v) = view else {
        return Place {
            gain: 1.0,
            pan: 0.0,
        };
    };
    let cw = v.w.max(1.0);
    let ch = v.h.max(1.0);
    let dx = if x < v.x {
        x - v.x
    } else {
        (x - (v.x + cw)).max(0.0)
    };
    let nx = dx.abs() / cw;
    let ny = max3(v.y - y, 0.0, y - (v.y + ch)) / ch;
    let radius = fade_views.max(0.01);
    let gain = clamp(1.0 - point_distance(0.0, 0.0, nx, ny) / radius, 0.0, 1.0);
    let mut pan = 0.0;
    if dx != 0.0 {
        let edge = vis("sound_pan_edge");
        let far = vis("sound_pan_far");
        pan = sign(dx) * lerp(edge, far, clamp(nx / radius, 0.0, 1.0));
    }
    Place {
        gain,
        pan: clamp(pan, -1.0, 1.0),
    }
}

impl Audio {
    /// The engine before `audio_init` (volumes given, nothing created).
    pub fn new(volume_sfx: f64, volume_music: f64, rng: Rng) -> Audio {
        Audio {
            volume_sfx,
            volume_music,
            view: None,
            music_bag: Vec::new(),
            music_last: None,
            music_voice: None,
            rain_voice: None,
            forest_voice: None,
            forest_duck: 0.0,
            water_touch_cooldown: 0.0,
            ambience: Vec::new(),
            sound_emitters: Vec::new(),
            sound_emitter_next: 0,
            sound_loops: Vec::new(),
            hyphae: HashMap::new(),
            rng,
        }
    }

    // port: audio_init
    /// Volumes from `settings` (the text of settings.ini), then everything reset.
    pub fn audio_init(&mut self, out: &mut dyn AudioOut, settings: &str) {
        self.volume_sfx = clamp(
            ini_read_real(settings, "audio", "volume_sfx", vis("volume_sfx")),
            0.0,
            1.0,
        );
        self.volume_music = clamp(
            ini_read_real(settings, "audio", "volume_music", vis("volume_music")),
            0.0,
            1.0,
        );
        self.music_bag.clear();
        self.music_last = None;
        self.music_voice = None;
        for t in MUSIC_TRACKS {
            out.stop_sound(t);
        }
        out.stop_sound("rain01");
        self.rain_voice = None;
        out.stop_sound("forest");
        self.forest_voice = None;
        self.forest_duck = 0.0;
        self.water_touch_cooldown = 0.0;
        self.ambience_init(out);
        self.sound_emitters_init(out);
        self.sound_loops_init(out);
    }

    // port: audio_save
    /// settings.ini text with `[audio] volume_sfx/volume_music` written (`ini_write_real`, `%f`),
    /// other sections kept.
    pub fn audio_save_text(&self, settings: &str) -> String {
        let t = save::ini_write(
            settings,
            "audio",
            "volume_sfx",
            &format!("{:.6}", self.volume_sfx),
        );
        save::ini_write(
            &t,
            "audio",
            "volume_music",
            &format!("{:.6}", self.volume_music),
        )
    }

    /// `audio_save` on `<dir>/settings.ini` (through a temporary file).
    pub fn audio_save(&self, dir: &Path) -> io::Result<()> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join("settings.ini");
        let text = String::from_utf8_lossy(&std::fs::read(&path).unwrap_or_default()).into_owned();
        let tmp = dir.join("settings.ini.tmp");
        std::fs::write(&tmp, self.audio_save_text(&text))?;
        std::fs::rename(&tmp, &path)
    }

    // port: audio_get_volume
    pub fn audio_get_volume(&self, kind: VolumeKind) -> f64 {
        match kind {
            VolumeKind::Music => self.volume_music,
            VolumeKind::Sfx => self.volume_sfx,
        }
    }

    // port: audio_set_volume
    pub fn audio_set_volume(&mut self, out: &mut dyn AudioOut, kind: VolumeKind, value: f64) {
        let v = clamp(value, 0.0, 1.0);
        match kind {
            VolumeKind::Music => {
                self.volume_music = v;
                if let Some(mv) = self.music_voice {
                    if out.is_playing(mv) {
                        out.gain(mv, v, 0.0);
                    }
                }
            }
            VolumeKind::Sfx => self.volume_sfx = v,
        }
    }

    // port: audio_step
    /// Once per game step (obj_game Step). `weather` is `global.weather.intensity`.
    pub fn audio_step(&mut self, out: &mut dyn AudioOut, weather: f64) {
        self.rain_sound_step(out, weather);
        self.forest_sound_step(out, weather);
        self.ambience_step(out);
        self.sound_loops_step(out);
        if self.water_touch_cooldown > 0.0 {
            self.water_touch_cooldown -= 1.0;
        }
        if let Some(v) = self.music_voice {
            if out.is_playing(v) {
                return;
            }
        }
        let track = self.music_next_track();
        self.music_last = Some(track);
        let v = out.play(track, false, None);
        self.music_voice = Some(v);
        out.gain(v, self.volume_music, 0.0);
    }

    // port: music_next_track
    /// The first track first, then shuffled bags of all tracks; a new bag never starts with the
    /// track that just played.
    pub fn music_next_track(&mut self) -> &'static str {
        let Some(last) = self.music_last else {
            return MUSIC_TRACKS[0];
        };
        if self.music_bag.is_empty() {
            // array_shuffle (game/gm/value.lua): Fisher-Yates from the end
            let mut bag = MUSIC_TRACKS.to_vec();
            for i in (1..bag.len()).rev() {
                let j = (self.rng.next01() * (i + 1) as f64).floor() as usize;
                bag.swap(i, j);
            }
            if bag.len() > 1 && bag[0] == last {
                let j = (self.rng.irandom((bag.len() - 2) as f64) + 1) as usize;
                bag.swap(0, j);
            }
            self.music_bag = bag;
        }
        self.music_bag.remove(0)
    }

    // port: rain_sound_step
    pub fn rain_sound_step(&mut self, out: &mut dyn AudioOut, weather: f64) {
        let w = weather;
        let playing = self.rain_voice.is_some_and(|v| out.is_playing(v));
        if w <= 0.0 {
            if playing {
                out.stop(self.rain_voice.unwrap());
            }
            self.rain_voice = None;
            return;
        }
        if !playing {
            self.rain_voice = Some(out.play("rain01", true, None));
        }
        let g = w * self.volume_sfx * clamp(vis("rain_sound_volume"), 0.0, 1.0);
        out.gain(self.rain_voice.unwrap(), g, 0.0);
    }

    // port: forest_sound_step
    /// The forest bed, ducked toward `forest_rain_level` while it rains.
    pub fn forest_sound_step(&mut self, out: &mut dyn AudioOut, weather: f64) {
        let w = weather;
        let step = 1.0 / vis("forest_duck_frames").max(1.0);
        self.forest_duck += clamp(w - self.forest_duck, -step, step);
        if !self.forest_voice.is_some_and(|v| out.is_playing(v)) {
            self.forest_voice = Some(out.play("forest", true, None));
        }
        let duck_lerp = lerp(1.0, vis("forest_rain_level"), self.forest_duck);
        let v = vis("forest_volume") * duck_lerp;
        out.gain(
            self.forest_voice.unwrap(),
            self.volume_sfx * clamp(v, 0.0, 1.0),
            0.0,
        );
    }

    // port: ambience_init
    pub fn ambience_init(&mut self, out: &mut dyn AudioOut) {
        // audio_falloff_set_model(audio_falloff_none), listener at the origin: the backend's
        // emitters are pure pans
        for a in self.ambience.drain(..) {
            out.emitter_free(a.emitter);
        }
        self.ambience_add(out, "bird", "ambience_bird", true);
        self.ambience_add(out, "creak", "ambience_creak", true);
        self.ambience_add(out, "woodpecker", "ambience_woodpecker", true);
        self.ambience_add(out, "leaves", "ambience_leaves", false);
    }

    // port: ambience_add
    fn ambience_add(
        &mut self,
        out: &mut dyn AudioOut,
        sound: &'static str,
        key: &'static str,
        panned: bool,
    ) {
        let mut a = Ambience {
            sound,
            key,
            panned,
            emitter: out.emitter_create(),
            timer: 0.0,
        };
        self.ambience_rearm(&mut a);
        self.ambience.push(a);
    }

    // port: ambience_rearm
    fn ambience_rearm(&mut self, a: &mut Ambience) {
        let max_sec = vis(&format!("{}_max_sec", a.key));
        let min_sec = vis(&format!("{}_min_sec", a.key));
        a.timer = round(self.rng.random_range(min_sec, max_sec) * FPS);
    }

    /// Frames left on each ambience timer (bird, creak, woodpecker, leaves).
    pub fn ambience_timers(&self) -> Vec<f64> {
        self.ambience.iter().map(|a| a.timer).collect()
    }

    // port: ambience_step
    /// Each ambience sound plays every `<key>_min_sec..<key>_max_sec` seconds at a random pan.
    pub fn ambience_step(&mut self, out: &mut dyn AudioOut) {
        let mut list = std::mem::take(&mut self.ambience);
        for a in &mut list {
            a.timer -= 1.0;
            if a.timer <= 0.0 {
                self.ambience_rearm(a);
                if self.volume_sfx > 0.0 {
                    let pan = if a.panned {
                        clamp(vis("ambience_pan"), 0.0, 1.0) * self.rng.random_range(-1.0, 1.0)
                    } else {
                        0.0
                    };
                    out.emitter_position(a.emitter, pan, 0.0, (1.0 - pan * pan).sqrt());
                    let voice = out.play(a.sound, false, Some(a.emitter));
                    let vol = clamp(vis(&format!("{}_volume", a.key)), 0.0, 1.0);
                    out.gain(voice, self.volume_sfx * vol, 0.0);
                }
            }
        }
        self.ambience = list;
    }

    // port: water_touch_sound
    /// Hypha touching water; at most once per `water_touch_cooldown_frames`.
    pub fn water_touch_sound(&mut self, out: &mut dyn AudioOut, at: Option<(f64, f64)>) {
        if self.water_touch_cooldown > 0.0 {
            return;
        }
        let voice = match at {
            None => {
                let v = self.sfx_play(out, "water_touch");
                if let Some(v) = v {
                    let g = self.volume_sfx * clamp(vis("water_touch_volume"), 0.0, 1.0);
                    out.gain(v, g, 0.0);
                }
                v
            }
            Some((x, y)) => self.sfx_play_at(
                out,
                "water_touch",
                x,
                y,
                Some(vis("water_touch_volume")),
                None,
            ),
        };
        if voice.is_some() {
            self.water_touch_cooldown = vis("water_touch_cooldown_frames");
        }
    }

    // port: hyphae_sound_begin
    /// Start a hyphae growth sound: the start cue now, a looping middle from half the start cue,
    /// the end cue (short/mid/long by length) at `hyphae_end_at` of `total` frames. `at` places
    /// it in the world (leaves, worms); `None` is non-positional (mycelium growth). `start` is the
    /// original's `choose(hyphae_start_1, hyphae_start_2)`, rolled by logic.
    pub fn hyphae_sound_begin(
        &self,
        out: &mut dyn AudioOut,
        len: f64,
        total: f64,
        at: Option<(f64, f64)>,
        start: HyphaeStart,
    ) -> HyphaeSound {
        let plan = hyphae_plan(len, total, start);
        let (sx, sy) = at.unwrap_or((0.0, 0.0));
        let mut s = HyphaeSound {
            total,
            placed: at.is_some(),
            x: sx,
            y: sy,
            start_voice: None,
            middle_voice: None,
            middle_allowed: !plan.tiny,
            middle_at: plan.middle_at,
            end_at: plan.end_at,
            end_sound: plan.end_sound,
            end_voice: None,
            fade_until: -1.0,
            ended: false,
            done: false,
        };
        let place_gain = self.hyphae_sound_place_gain(&s);
        s.start_voice =
            self.hyphae_sound_play(out, start.sound(), false, vis("hyphae_volume") * place_gain);
        s
    }

    // port: hyphae_sound_place_gain
    fn hyphae_sound_place_gain(&self, s: &HyphaeSound) -> f64 {
        if s.placed {
            sound_camera_gain(self.view.as_ref(), s.x, s.y)
        } else {
            1.0
        }
    }

    // port: hyphae_sound_step
    /// Advance a hyphae sound to frame `t` of its growth.
    pub fn hyphae_sound_step(&self, out: &mut dyn AudioOut, s: &mut HyphaeSound, t: f64) {
        if s.done {
            return;
        }
        let place_gain = self.hyphae_sound_place_gain(s);
        let start_middle =
            !s.ended && s.middle_allowed && s.middle_voice.is_none() && t >= s.middle_at;
        if start_middle {
            s.middle_voice = self.hyphae_sound_play(out, "hyphae_midlle", true, 0.0);
            s.middle_allowed = false;
            if let Some(mv) = s.middle_voice {
                let len = out.sound_length("hyphae_midlle");
                let loop_start = clamp(vis("hyphae_loop_start"), 0.0, len * 0.5);
                let loop_end = (loop_start + 0.1).max(len - vis("hyphae_loop_tail"));
                out.loop_start(mv, loop_start);
                out.loop_end(mv, loop_end);
            }
        }
        if s.placed {
            let g = self.volume_sfx * clamp(vis("hyphae_volume") * place_gain, 0.0, 1.0);
            if let Some(v) = s.start_voice.filter(|&v| out.is_playing(v)) {
                out.gain(v, g, 0.0);
            }
            if let Some(v) = s.end_voice.filter(|&v| out.is_playing(v)) {
                out.gain(v, g, 0.0);
            }
        }
        if !s.ended {
            if let Some(mv) = s.middle_voice {
                let from = s.total * vis("hyphae_taper_from");
                let k = clamp((t - from) / (s.end_at - from).max(1.0), 0.0, 1.0);
                let taper = lerp(1.0, vis("hyphae_taper_to"), k);
                let m = vis("hyphae_middle_volume") * taper * place_gain;
                out.gain(mv, self.volume_sfx * clamp(m, 0.0, 1.0), 0.0);
            }
        }
        if !s.ended && t >= s.end_at {
            s.ended = true;
            if let Some(mv) = s.middle_voice {
                let fade = vis("hyphae_middle_fade_frames").max(1.0);
                out.gain(mv, 0.0, fade / FPS * 1000.0);
                s.fade_until = t + fade;
            }
            s.end_voice =
                self.hyphae_sound_play(out, s.end_sound, false, vis("hyphae_volume") * place_gain);
        }
        if s.ended && t >= s.fade_until {
            if let Some(mv) = s.middle_voice.take() {
                out.stop(mv);
            }
        }
        let end_playing = s.placed && s.end_voice.is_some_and(|v| out.is_playing(v));
        s.done = s.ended && s.middle_voice.is_none() && !end_playing;
    }

    // port: hyphae_sound_stop
    pub fn hyphae_sound_stop(&self, out: &mut dyn AudioOut, s: &mut HyphaeSound) {
        if let Some(mv) = s.middle_voice.take() {
            out.stop(mv);
        }
        s.done = true;
    }

    // port: hyphae_sound_play
    pub fn hyphae_sound_play(
        &self,
        out: &mut dyn AudioOut,
        snd: &str,
        looping: bool,
        gain: f64,
    ) -> Option<Voice> {
        if self.volume_sfx <= 0.0 {
            return None;
        }
        let voice = out.play(snd, looping, None);
        out.gain(voice, self.volume_sfx * clamp(gain, 0.0, 1.0), 0.0);
        Some(voice)
    }

    // port: mushroom_grow_sound
    /// `snd` is the original's `choose(mush_grow_1, _2, _3)`, rolled by logic; `vol: None` =
    /// `mush_grow_volume`.
    pub fn mushroom_grow_sound(
        &mut self,
        out: &mut dyn AudioOut,
        snd: &str,
        at: Option<(f64, f64)>,
        vol: Option<f64>,
    ) {
        let vol = vol.unwrap_or_else(|| vis("mush_grow_volume"));
        let Some((x, y)) = at else {
            if let Some(v) = self.sfx_play(out, snd) {
                out.gain(v, self.volume_sfx * clamp(vol, 0.0, 1.0), 0.0);
            }
            return;
        };
        self.sfx_play_at(
            out,
            snd,
            x,
            y,
            Some(vol),
            Some(vis("sound_camera_fade_views")),
        );
    }

    // port: sfx_play
    pub fn sfx_play(&mut self, out: &mut dyn AudioOut, snd: &str) -> Option<Voice> {
        if self.volume_sfx <= 0.0 {
            return None;
        }
        let voice = out.play(snd, false, None);
        out.gain(voice, self.volume_sfx, 0.0);
        Some(voice)
    }

    // port: sound_emitters_init
    /// The 8 emitters `sfx_play_at` cycles through.
    pub fn sound_emitters_init(&mut self, out: &mut dyn AudioOut) {
        for e in self.sound_emitters.drain(..) {
            out.emitter_free(e);
        }
        self.sound_emitter_next = 0;
        for _ in 0..SOUND_EMITTERS {
            let e = out.emitter_create();
            self.sound_emitters.push(e);
        }
    }

    // port: sfx_play_at
    /// A one-shot at world `(x, y)`: gain and pan from `sound_world_place` (`reach: None` =
    /// `sound_world_reach_views`); nothing when out of reach. `vol: None` = 1.
    pub fn sfx_play_at(
        &mut self,
        out: &mut dyn AudioOut,
        snd: &str,
        x: f64,
        y: f64,
        vol: Option<f64>,
        reach: Option<f64>,
    ) -> Option<Voice> {
        let vol = vol.unwrap_or(1.0);
        if self.volume_sfx <= 0.0 {
            return None;
        }
        let reach = reach.unwrap_or_else(|| vis("sound_world_reach_views"));
        let place = sound_world_place(self.view.as_ref(), x, y, reach);
        if place.gain <= 0.0 {
            return None;
        }
        let em = *self.sound_emitters.get(self.sound_emitter_next)?;
        self.sound_emitter_next = (self.sound_emitter_next + 1) % SOUND_EMITTERS;
        let pan = place.pan;
        out.emitter_position(em, pan, 0.0, (1.0 - pan * pan).sqrt());
        let voice = out.play(snd, false, Some(em));
        out.gain(
            voice,
            self.volume_sfx * clamp(vol * place.gain, 0.0, 1.0),
            0.0,
        );
        Some(voice)
    }

    // port: sound_loops_init
    pub fn sound_loops_init(&mut self, out: &mut dyn AudioOut) {
        for mut l in std::mem::take(&mut self.sound_loops) {
            sound_loop_free(out, &mut l);
        }
    }

    // port: sound_loop_start
    fn sound_loop_start(&mut self, out: &mut dyn AudioOut, key: u64, sound: &'static str) -> usize {
        self.sound_loops.push(SoundLoop {
            key,
            sound,
            emitter: out.emitter_create(),
            voice: None,
            fade: 0.0,
            idle: 0.0,
            x: 0.0,
            y: 0.0,
            volume: 1.0,
            reach: 1.0,
            fade_in: 1.0,
            fade_out: 1.0,
            done: false,
        });
        self.sound_loops.len() - 1
    }

    // port: sound_loop_follow
    /// Called every frame by the loop's source: move it, fade it in.
    #[allow(clippy::too_many_arguments)]
    fn sound_loop_follow(
        &mut self,
        out: &mut dyn AudioOut,
        i: usize,
        x: f64,
        y: f64,
        volume: f64,
        reach: f64,
        fade_in: f64,
        fade_out: f64,
    ) {
        let (view, vol_sfx) = (self.view, self.volume_sfx);
        let l = &mut self.sound_loops[i];
        if l.done {
            return;
        }
        l.x = x;
        l.y = y;
        l.volume = volume;
        l.reach = reach;
        l.fade_in = fade_in.max(1.0);
        l.fade_out = fade_out.max(1.0);
        l.idle = 0.0;
        l.fade = (l.fade + 1.0 / l.fade_in).min(1.0);
        sound_loop_apply(out, view.as_ref(), vol_sfx, l);
    }

    // port: sound_loops_step
    /// Loops not followed for more than 2 frames fade out over `fade_out` frames and are freed.
    pub fn sound_loops_step(&mut self, out: &mut dyn AudioOut) {
        let (view, vol_sfx) = (self.view, self.volume_sfx);
        let mut keep = Vec::with_capacity(self.sound_loops.len());
        for mut l in std::mem::take(&mut self.sound_loops) {
            l.idle += 1.0;
            let mut skip = false;
            if l.idle > 2.0 {
                l.fade -= 1.0 / l.fade_out;
                if l.fade <= 0.0 {
                    sound_loop_free(out, &mut l);
                    skip = true;
                } else {
                    sound_loop_apply(out, view.as_ref(), vol_sfx, &mut l);
                }
            }
            if !skip {
                keep.push(l);
            }
        }
        self.sound_loops = keep;
    }

    pub fn sound_loops_len(&self) -> usize {
        self.sound_loops.len()
    }

    /// Live hyphae sounds (begun and neither done nor stopped).
    pub fn hyphae_len(&self) -> usize {
        self.hyphae.len()
    }

    /// A stateful call from logic.
    pub fn audio_call(&mut self, out: &mut dyn AudioOut, call: &AudioCall) {
        match *call {
            AudioCall::WaterTouch { at } => self.water_touch_sound(out, at),
            AudioCall::MushroomGrow { sound, at, volume } => {
                self.mushroom_grow_sound(out, sound, at, volume)
            }
            AudioCall::HyphaeBegin {
                id,
                len,
                total,
                at,
                start,
            } => {
                let s = self.hyphae_sound_begin(out, len, total, at, start);
                self.hyphae.insert(id, s);
            }
            AudioCall::HyphaeStep { id, t } => {
                if let Some(mut s) = self.hyphae.remove(&id) {
                    self.hyphae_sound_step(out, &mut s, t);
                    if !s.done {
                        self.hyphae.insert(id, s);
                    }
                }
            }
            AudioCall::HyphaeStop { id } => {
                if let Some(mut s) = self.hyphae.remove(&id) {
                    self.hyphae_sound_stop(out, &mut s);
                }
            }
            AudioCall::LoopFollow {
                id,
                sound,
                x,
                y,
                volume,
                reach,
                fade_in,
                fade_out,
            } => {
                let i = match self.sound_loops.iter().position(|l| l.key == id && !l.done) {
                    Some(i) => i,
                    None => self.sound_loop_start(out, id, sound),
                };
                self.sound_loop_follow(out, i, x, y, volume, reach, fade_in, fade_out);
            }
        }
    }

    /// A one-shot cue: `sfx_play` (+ gain override) or, positioned, `sfx_play_at`.
    pub fn sound_cue(&mut self, out: &mut dyn AudioOut, cue: &SoundCue) {
        let voice = match (cue.x, cue.y) {
            (Some(x), Some(y)) => self.sfx_play_at(out, &cue.name, x, y, Some(cue.gain), cue.reach),
            _ => {
                let v = self.sfx_play(out, &cue.name);
                if let Some(v) = v.filter(|_| cue.gain != 1.0) {
                    out.gain(v, self.volume_sfx * clamp(cue.gain, 0.0, 1.0), 0.0);
                }
                v
            }
        };
        if let Some(v) = voice.filter(|_| cue.pitch != 1.0) {
            out.pitch(v, cue.pitch);
        }
    }

    /// Route a simulation event; non-audio events are ignored.
    pub fn sim_event(&mut self, out: &mut dyn AudioOut, e: &SimEvent) {
        match e {
            SimEvent::Sound(cue) => self.sound_cue(out, cue),
            SimEvent::Audio(call) => self.audio_call(out, call),
            _ => {}
        }
    }
}

// port: sound_loop_apply
fn sound_loop_apply(
    out: &mut dyn AudioOut,
    view: Option<&ViewRect>,
    volume_sfx: f64,
    l: &mut SoundLoop,
) {
    let place = sound_world_place(view, l.x, l.y, l.reach);
    let playing = l.voice.is_some_and(|v| out.is_playing(v));
    if !playing {
        l.voice = Some(out.play(l.sound, true, Some(l.emitter)));
    }
    let pan = place.pan;
    let depth = (1.0 - pan * pan).sqrt();
    out.emitter_position(l.emitter, pan, 0.0, depth);
    let level = clamp(l.volume * place.gain * l.fade, 0.0, 1.0);
    out.gain(l.voice.unwrap(), volume_sfx * level, 0.0);
}

// port: sound_loop_free
fn sound_loop_free(out: &mut dyn AudioOut, l: &mut SoundLoop) {
    if let Some(v) = l.voice.take() {
        out.stop(v);
    }
    if !l.done {
        out.emitter_free(l.emitter);
    }
    l.done = true;
}

// ---------------------------------------------------------------------------------------------
// Bevy side

/// What the settings UI asks of the audio (`audio_set_volume` while dragging a slider,
/// `audio_save` on release).
#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum AudioRequest {
    SetVolume(VolumeKind, f64),
    Save,
}

/// The engine in the app: `audio_init` at startup (settings.ini from [`SaveDir`]), `audio_step`
/// after every simulation step followed by that step's sound events.
pub struct GameAudioPlugin;

impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        use bevy::audio::AddAudioSource;
        app.add_audio_source::<backend::VoiceSound>()
            .add_message::<AudioRequest>()
            .add_systems(Startup, init_audio)
            .add_systems(
                FixedUpdate,
                // after every `SimSet::After` system: events sent there (the tiles step's
                // hyphae stops) play in their own step
                (audio_fixed_step, backend::spawn_voices)
                    .chain()
                    .after(SimSet::After),
            )
            .add_systems(
                Update,
                (
                    backend::check_ready,
                    handle_requests,
                    backend::update_voices,
                    backend::spawn_voices,
                )
                    .chain(),
            );
    }
}

fn init_audio(
    mut commands: Commands,
    assets: Res<crate::assets::GameAssets>,
    save_dir: Option<Res<SaveDir>>,
) {
    let mut out = BevyOut::from_assets(&assets);
    let mut rng = Rng::new(0);
    rng.randomise();
    let mut audio = Audio::new(vis("volume_sfx"), vis("volume_music"), rng);
    let text = save_dir
        .and_then(|d| std::fs::read(d.0.join("settings.ini")).ok())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    audio.audio_init(&mut out, &text);
    commands.insert_resource(audio);
    commands.insert_resource(out);
}

fn audio_fixed_step(
    audio: Option<ResMut<Audio>>,
    out: Option<ResMut<BevyOut>>,
    sim: Res<SimRes>,
    cams: Option<Res<Cameras>>,
    mut events: MessageReader<SimEventMsg>,
) {
    let (Some(mut audio), Some(mut out)) = (audio, out) else {
        events.clear();
        return;
    };
    if !out.ready() {
        events.clear();
        return;
    }
    audio.view = cams.map(|c| c.game_view);
    let out = &mut *out;
    audio.audio_step(out, sim.0.weather.intensity);
    for SimEventMsg(e) in events.read() {
        audio.sim_event(out, e);
    }
}

fn handle_requests(
    mut requests: MessageReader<AudioRequest>,
    audio: Option<ResMut<Audio>>,
    out: Option<ResMut<BevyOut>>,
    save_dir: Option<Res<SaveDir>>,
) {
    let (Some(mut audio), Some(mut out)) = (audio, out) else {
        requests.clear();
        return;
    };
    for r in requests.read() {
        match *r {
            AudioRequest::SetVolume(kind, v) => audio.audio_set_volume(&mut *out, kind, v),
            AudioRequest::Save => {
                if let Some(d) = &save_dir {
                    if let Err(e) = audio.audio_save(&d.0) {
                        warn!("audio_save: {e}");
                    }
                }
            }
        }
    }
}
