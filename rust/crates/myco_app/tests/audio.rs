//! The audio engine (`myco_app::audio`, port of `scr_audio`) against the LÖVE port.
//!
//! `rust/golden/scripts/audio.lua` replaces GameMaker's audio builtins with a recorder and runs
//! scenarios through the real scr_audio functions; here the engine runs the same scenarios against
//! [`Fake`], which records the same calls under the same rules (voice ids from 1000, emitter ids
//! from 0, a voice plays until stopped and a non-looping one ends ceil(length * 60) frames after
//! it started).
use std::collections::HashMap;
use std::path::PathBuf;

use myco_app::audio::{
    sound_camera_gain, sound_world_place, Audio, AudioOut, Emitter, Voice, VolumeKind,
};
use myco_core::audio::{sound_length, HyphaeStart};
use myco_core::events::AudioCall;
use myco_core::rng::Rng;
use myco_core::sim::ViewRect;
use serde_json::{json, Value};

fn golden() -> Value {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../golden/data/audio.json");
    let text = std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{}: {e} (tools/golden.sh audio)", p.display()));
    serde_json::from_str(&text).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64) -> bool {
    a == b || (a - b).abs() <= 1e-9 * 1f64.max(a.abs()).max(b.abs())
}

/// Compare JSON values, numbers within 1e-9 relative.
fn same(got: &Value, want: &Value) -> bool {
    match (got, want) {
        (Value::Number(_), Value::Number(_)) => close(f(got), f(want)),
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(x, y)| same(x, y))
        }
        _ => got == want,
    }
}

fn assert_same(got: &Value, want: &Value, what: &str) {
    assert!(same(got, want), "{what}:\n got  {got}\n want {want}");
}

struct FakeVoice {
    stopped: bool,
    looping: bool,
    ends: f64,
}

#[derive(Default)]
struct Fake {
    log: Vec<Value>,
    voices: HashMap<Voice, FakeVoice>,
    next_voice: Voice,
    next_em: Emitter,
    now: f64,
}

impl Fake {
    fn new() -> Fake {
        Fake {
            next_voice: 1000,
            ..Default::default()
        }
    }
    fn take(&mut self) -> Value {
        Value::Array(std::mem::take(&mut self.log))
    }
}

impl AudioOut for Fake {
    fn play(&mut self, snd: &str, looping: bool, emitter: Option<Emitter>) -> Voice {
        let id = self.next_voice;
        self.next_voice += 1;
        let ends = self.now + (sound_length(snd).unwrap() * 60.0).ceil();
        self.voices.insert(
            id,
            FakeVoice {
                stopped: false,
                looping,
                ends,
            },
        );
        let em = emitter.map_or(-1, |e| e as i64);
        self.log.push(json!(["play", id, snd, looping as i32, em]));
        id
    }
    fn is_playing(&self, v: Voice) -> bool {
        self.voices
            .get(&v)
            .is_some_and(|fv| !fv.stopped && (fv.looping || self.now < fv.ends))
    }
    fn stop(&mut self, v: Voice) {
        if let Some(fv) = self.voices.get_mut(&v) {
            fv.stopped = true;
            self.log.push(json!(["stop", v]));
        }
    }
    fn stop_sound(&mut self, snd: &str) {
        self.log.push(json!(["stop_asset", snd]));
    }
    fn gain(&mut self, v: Voice, gain: f64, ms: f64) {
        self.log.push(json!(["gain", v, gain, ms]));
    }
    fn pitch(&mut self, v: Voice, pitch: f64) {
        self.log.push(json!(["pitch", v, pitch]));
    }
    fn loop_start(&mut self, v: Voice, pos: f64) {
        self.log.push(json!(["loop_start", v, pos]));
    }
    fn loop_end(&mut self, v: Voice, pos: f64) {
        self.log.push(json!(["loop_end", v, pos]));
    }
    fn sound_length(&self, snd: &str) -> f64 {
        sound_length(snd).unwrap()
    }
    fn emitter_create(&mut self) -> Emitter {
        let id = self.next_em;
        self.next_em += 1;
        id
    }
    fn emitter_free(&mut self, e: Emitter) {
        self.log.push(json!(["em_free", e]));
    }
    fn emitter_position(&mut self, e: Emitter, x: f64, _y: f64, z: f64) {
        self.log.push(json!(["em_pos", e, x, z]));
    }
}

const VIEW: ViewRect = ViewRect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
};

/// The golden script's `reset()`: volumes 0.8 / 0.55, emitters and loops initialised.
fn reset() -> (Audio, Fake) {
    let mut out = Fake::new();
    let mut a = Audio::new(0.8, 0.55, Rng::new(1));
    a.view = Some(VIEW);
    a.sound_emitters_init(&mut out);
    a.sound_loops_init(&mut out);
    (a, out)
}

fn voice_json(v: Option<Voice>) -> Value {
    v.map_or(json!(-1), |v| json!(v))
}

#[test]
fn sound_volume_falls_off_like_love() {
    let g = golden();
    let mut n = 0;
    for case in g["place"].as_array().unwrap() {
        let v = case["view"].as_array().unwrap();
        let view = ViewRect {
            x: f(&v[0]),
            y: f(&v[1]),
            w: f(&v[2]),
            h: f(&v[3]),
        };
        for r in case["rows"].as_array().unwrap() {
            let r: Vec<f64> = r.as_array().unwrap().iter().map(f).collect();
            let (x, y) = (r[0], r[1]);
            let got = [
                sound_camera_gain(Some(&view), x, y),
                sound_world_place(Some(&view), x, y, 1.0).gain,
                sound_world_place(Some(&view), x, y, 1.0).pan,
                sound_world_place(Some(&view), x, y, 0.5).gain,
                sound_world_place(Some(&view), x, y, 0.5).pan,
                sound_world_place(Some(&view), x, y, 0.005).gain,
                sound_world_place(Some(&view), x, y, 0.005).pan,
            ];
            for (k, want) in r[2..].iter().enumerate() {
                assert!(
                    close(got[k], *want),
                    "view {view:?} point ({x}, {y}) value {k}: {} vs {want}",
                    got[k]
                );
            }
            n += 1;
        }
    }
    assert_eq!(n, 3 * 25 * 16);
    // no camera: full gain, centred
    assert_eq!(sound_camera_gain(None, 1e6, 1e6), 1.0);
    let p = sound_world_place(None, 1e6, 1e6, 1.0);
    assert_eq!((p.gain, p.pan), (1.0, 0.0));
}

#[test]
fn rain_and_forest_follow_weather_like_love() {
    let g = golden();
    let (mut a, mut out) = reset();
    for row in g["weather"].as_array().unwrap() {
        let fr = f(&row[0]);
        out.now = fr;
        if fr == 200.0 {
            out.voices
                .get_mut(&a.forest_voice.unwrap())
                .unwrap()
                .stopped = true;
        }
        let w = f(&row[1]);
        a.rain_sound_step(&mut out, w);
        a.forest_sound_step(&mut out, w);
        assert!(close(a.forest_duck, f(&row[2])), "frame {fr} duck");
        assert_same(&out.take(), &row[3], &format!("frame {fr}"));
    }
}

#[test]
fn hyphae_sounds_match_love() {
    let g = golden();
    for h in g["hyphae"].as_array().unwrap() {
        let c = h["case"].as_array().unwrap();
        let (mut a, mut out) = reset();
        a.volume_sfx = f(&c[7]);
        let at = c[2].as_f64().map(|x| (x, f(&c[3])));
        let start = if f(&c[4]) == 1.0 {
            HyphaeStart::One
        } else {
            HyphaeStart::Two
        };
        let stop_at = f(&c[6]);
        let mut s = a.hyphae_sound_begin(&mut out, f(&c[0]), f(&c[1]), at, start);
        let mut frames = vec![json!([0, out.take()])];
        let mut done_at = -1.0;
        for t in 1..=(f(&c[5]) as i64) {
            let t = t as f64;
            out.now = t;
            if t == stop_at {
                a.hyphae_sound_stop(&mut out, &mut s);
            } else {
                a.hyphae_sound_step(&mut out, &mut s, t);
            }
            let log = out.take();
            if !log.as_array().unwrap().is_empty() {
                frames.push(json!([t, log]));
            }
            if s.done && done_at < 0.0 {
                done_at = t;
            }
        }
        assert_same(&Value::Array(frames), &h["frames"], &format!("case {c:?}"));
        assert_eq!(done_at, f(&h["done_at"]), "case {c:?}");
    }
}

#[test]
fn sound_loop_follows_and_fades_like_love() {
    let g = golden();
    let (mut a, mut out) = reset();
    for row in g["loop"].as_array().unwrap() {
        let fr = f(&row[0]);
        out.now = fr;
        a.sound_loops_step(&mut out);
        if fr <= 120.0 {
            a.audio_call(
                &mut out,
                &AudioCall::LoopFollow {
                    id: 1,
                    sound: "boar_tremor",
                    x: -1500.0 + fr * 40.0,
                    y: 600.0,
                    volume: 1.0,
                    reach: 1.0,
                    fade_in: 45.0,
                    fade_out: 60.0,
                },
            );
        }
        assert_eq!(a.sound_loops_len() as f64, f(&row[1]), "frame {fr} loops");
        assert_same(&out.take(), &row[2], &format!("frame {fr}"));
    }
}

#[test]
fn one_shots_water_and_mushrooms_match_love() {
    let g = golden();
    let (mut a, mut out) = reset();
    for r in g["oneshot"].as_array().unwrap() {
        let reach = r[3].as_f64();
        let v = a.sfx_play_at(
            &mut out,
            "explosion",
            f(&r[0]),
            f(&r[1]),
            Some(f(&r[2])),
            reach,
        );
        assert_eq!(voice_json(v), r[4], "shot {r}");
        assert_same(&out.take(), &r[5], &format!("shot {r}"));
    }
    let mut rows = Vec::new();
    for fr in 1..=100 {
        out.now = fr as f64;
        if a.water_touch_cooldown > 0.0 {
            a.water_touch_cooldown -= 1.0;
        }
        if fr % 7 == 0 {
            a.water_touch_sound(&mut out, Some((100.0 + fr as f64 * 30.0, 300.0)));
        }
        if fr == 50 || fr == 51 {
            a.water_touch_sound(&mut out, None);
        }
        let log = out.take();
        if !log.as_array().unwrap().is_empty() {
            rows.push(json!([fr, a.water_touch_cooldown, log]));
        }
    }
    assert_same(&Value::Array(rows), &g["water"], "water");
    let mut rows = Vec::new();
    for (k, snd) in ["mush_grow_1", "mush_grow_2", "mush_grow_3"]
        .iter()
        .enumerate()
    {
        a.mushroom_grow_sound(&mut out, snd, None, None);
        rows.push(json!([k + 1, "none", out.take()]));
        a.mushroom_grow_sound(&mut out, snd, Some((2200.0, 500.0)), Some(0.6));
        rows.push(json!([k + 1, "at", out.take()]));
    }
    assert_same(&Value::Array(rows), &g["mush"], "mush");
    a.volume_sfx = 0.0;
    let muted = json!([
        voice_json(a.sfx_play(&mut out, "thunder")),
        voice_json(a.sfx_play_at(&mut out, "explosion", 10.0, 10.0, None, None)),
        voice_json(a.hyphae_sound_play(&mut out, "hyphae_start_1", false, 1.0)),
    ]);
    assert_eq!(muted, g["muted"]);
    assert!(out.log.is_empty());
}

/// Events from logic reach the same functions: hyphae sounds are kept per id and dropped once
/// done, positioned cues go through sfx_play_at, plain cues through sfx_play with a gain override.
#[test]
fn audio_calls_drive_the_engine() {
    let (mut a, mut out) = reset();
    a.audio_call(
        &mut out,
        &AudioCall::HyphaeBegin {
            id: 9,
            len: 50.0,
            total: 60.0,
            at: None,
            start: HyphaeStart::One,
        },
    );
    assert_eq!(a.hyphae_len(), 1);
    for t in 1..=51 {
        out.now = t as f64;
        a.audio_call(&mut out, &AudioCall::HyphaeStep { id: 9, t: t as f64 });
    }
    assert_eq!(a.hyphae_len(), 0, "done sound dropped");
    out.take();
    a.audio_call(
        &mut out,
        &AudioCall::HyphaeBegin {
            id: 3,
            len: 200.0,
            total: 100.0,
            at: None,
            start: HyphaeStart::One,
        },
    );
    a.audio_call(&mut out, &AudioCall::HyphaeStop { id: 3 });
    assert_eq!(a.hyphae_len(), 0);

    out.take();
    let cue = myco_core::events::SoundCue::new("thunder");
    a.sound_cue(&mut out, &cue);
    assert_same(
        &out.take(),
        &json!([["play", 1003, "thunder", 0, -1], ["gain", 1003, 0.8, 0]]),
        "plain cue",
    );
    a.sound_cue(&mut out, &cue.clone().gain(0.5));
    assert_same(
        &out.take(),
        &json!([
            ["play", 1004, "thunder", 0, -1],
            ["gain", 1004, 0.8, 0],
            ["gain", 1004, 0.4, 0]
        ]),
        "cue with gain",
    );
    a.sound_cue(&mut out, &cue.clone().at(960.0, 540.0).gain(0.5));
    assert_same(
        &out.take(),
        &json!([
            ["em_pos", 0, 0, 1],
            ["play", 1005, "thunder", 0, 0],
            ["gain", 1005, 0.4, 0]
        ]),
        "positioned cue",
    );
}

#[test]
fn music_plays_tracks_without_repeats() {
    let (mut a, mut out) = reset();
    let mut tracks = Vec::new();
    for _ in 0..30 {
        a.audio_step(&mut out, 0.0);
        let log = out.take();
        let play = log
            .as_array()
            .unwrap()
            .iter()
            .find(|e| e[0] == "play" && e[3] == 0 && e[4] == -1)
            .expect("a track starts")
            .clone();
        let v = play[1].as_u64().unwrap();
        assert!(log
            .as_array()
            .unwrap()
            .iter()
            .any(|e| same(e, &json!(["gain", v, 0.55, 0]))));
        tracks.push(play[2].as_str().unwrap().to_string());
        // the track still plays next frame: no new one
        a.audio_step(&mut out, 0.0);
        assert!(!out
            .take()
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e[0] == "play" && e[2] == play[2] && e[4] == -1));
        out.voices.get_mut(&v).unwrap().stopped = true; // track over
    }
    assert_eq!(tracks[0], "hang_guitar_01");
    for w in tracks.windows(2) {
        assert_ne!(w[0], w[1], "{tracks:?}");
    }
    for bag in [&tracks[1..4], &tracks[4..7], &tracks[7..10]] {
        let mut b = bag.to_vec();
        b.sort();
        assert_eq!(b, ["guitar", "hang_guitar_01", "mushrooms"], "{tracks:?}");
    }
}

#[test]
fn ambience_rearms_and_pans_within_limits() {
    let (mut a, mut out) = reset();
    a.ambience_init(&mut out);
    // emitters: 8 sound emitters first (reset), then the 4 ambience ones
    let timers = a.ambience_timers();
    let limits = [(60.0, 120.0), (60.0, 120.0), (60.0, 120.0), (90.0, 150.0)];
    for (t, (lo, hi)) in timers.iter().zip(limits) {
        assert!(
            *t >= lo * 60.0 && *t <= hi * 60.0 && t.fract() == 0.0,
            "{t}"
        );
    }
    let mut plays: HashMap<String, usize> = HashMap::new();
    let mut last_pos: HashMap<u64, f64> = HashMap::new();
    for _ in 0..(150 * 60) {
        a.ambience_step(&mut out);
        for e in out.take().as_array().unwrap() {
            match e[0].as_str().unwrap() {
                "em_pos" => {
                    let (x, z) = (f(&e[2]), f(&e[3]));
                    assert!(close(x * x + z * z, 1.0));
                    last_pos.insert(e[1].as_u64().unwrap(), x);
                }
                "play" => {
                    let em = e[4].as_u64().unwrap();
                    let x = last_pos[&em];
                    if e[2] == "leaves" {
                        assert_eq!(x, 0.0);
                    } else {
                        assert!(x.abs() <= 0.4);
                    }
                    *plays.entry(e[2].as_str().unwrap().into()).or_default() += 1;
                }
                "gain" => assert!(close(f(&e[2]), 0.8)),
                other => panic!("unexpected {other}"),
            }
        }
    }
    for s in ["bird", "creak", "woodpecker", "leaves"] {
        assert!(plays.get(s).copied().unwrap_or(0) >= 1, "{s}: {plays:?}");
    }
    // muted: timers still re-arm, nothing plays
    a.volume_sfx = 0.0;
    for _ in 0..(150 * 60) {
        a.ambience_step(&mut out);
    }
    assert!(out.log.is_empty());
}

#[test]
fn volumes_come_from_settings_and_save_keeps_other_sections() {
    let mut out = Fake::new();
    let mut a = Audio::new(0.0, 0.0, Rng::new(1));
    a.audio_init(
        &mut out,
        "[display]\r\nfullscreen=\"1.000000\"\r\n[audio]\r\nvolume_sfx=\"0.250000\"\r\nvolume_music=\"7\"\r\n",
    );
    assert_eq!(a.audio_get_volume(VolumeKind::Sfx), 0.25);
    assert_eq!(a.audio_get_volume(VolumeKind::Music), 1.0, "clamped");
    // audio_init stops leftovers, then builds 4 ambience emitters and 8 sound emitters
    let log = out.take();
    assert_same(
        &log,
        &json!([
            ["stop_asset", "hang_guitar_01"],
            ["stop_asset", "guitar"],
            ["stop_asset", "mushrooms"],
            ["stop_asset", "rain01"],
            ["stop_asset", "forest"]
        ]),
        "init",
    );
    assert_eq!(a.ambience_timers().len(), 4);

    let mut b = Audio::new(0.0, 0.0, Rng::new(1));
    b.audio_init(&mut out, "[audio]\r\nvolume_sfx=\"x\"\r\n");
    assert_eq!(b.audio_get_volume(VolumeKind::Sfx), 0.8, "vis default");
    assert_eq!(b.audio_get_volume(VolumeKind::Music), 0.55);

    a.audio_set_volume(&mut out, VolumeKind::Sfx, 1.5);
    a.audio_set_volume(&mut out, VolumeKind::Music, 0.3);
    assert_eq!(a.audio_get_volume(VolumeKind::Sfx), 1.0);
    let text =
        a.audio_save_text("[display]\r\nfullscreen=\"1.000000\"\r\n[game]\r\nlanguage=\"de\"\r\n");
    assert_eq!(
        text,
        "[display]\r\nfullscreen=\"1.000000\"\r\n[game]\r\nlanguage=\"de\"\r\n[audio]\r\nvolume_sfx=\"1.000000\"\r\nvolume_music=\"0.300000\"\r\n"
    );

    let dir = std::env::temp_dir().join(format!("myco-audio-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("settings.ini"), "[game]\r\nlanguage=\"fr\"\r\n").unwrap();
    a.audio_save(&dir).unwrap();
    let saved = std::fs::read_to_string(dir.join("settings.ini")).unwrap();
    assert!(saved.contains("language=\"fr\"") && saved.contains("volume_music=\"0.300000\""));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn music_volume_change_reaches_the_playing_track() {
    let (mut a, mut out) = reset();
    a.audio_step(&mut out, 0.0);
    let v = a.music_voice.unwrap();
    out.take();
    a.audio_set_volume(&mut out, VolumeKind::Music, 0.2);
    assert_same(&out.take(), &json!([["gain", v, 0.2, 0]]), "music gain");
    out.voices.get_mut(&v).unwrap().stopped = true;
    a.audio_set_volume(&mut out, VolumeKind::Music, 0.4);
    assert!(out.take().as_array().unwrap().is_empty());
}
