//! [`AudioOut`] on bevy_audio (rodio).
//!
//! bevy_audio has no stereo pan or loop points, so every voice is its own [`VoiceSound`] asset
//! (a custom `Decodable`) whose [`VoiceSource`] reads gain, pan, loop points and stop from a
//! shared [`VoiceCtl`] the engine updates. bevy_kira_audio would have those built in, but it needs
//! bevy's own audio disabled and would replace the `Handle<AudioSource>` sounds `GameAssets`
//! already loads; a ~200-line source keeps the stock stack.
//!
//! Voice gain = `sounds.json` volume × `audio_sound_gain` (fades run per frame, like GameMaker's).
//! Pan applies to mono sounds only (as OpenAL positions only mono sources): left `min(1, 1 - pan)`,
//! right `min(1, 1 + pan)`, so a centred emitter sounds like a plain `audio_play_sound`.

use std::collections::HashMap;
use std::num::NonZero;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

use bevy::asset::LoadState;
use bevy::audio::{AudioPlayer, AudioSource, Decodable, PlaybackSettings, Source};
use bevy::prelude::*;

use super::{AudioOut, Emitter, Voice};
use crate::assets::GameAssets;

/// Sounds longer than this stream from the compressed bytes instead of being decoded up front
/// (music, rain, forest, leaves); loop points need the decoded form.
const STREAM_OVER_SECS: f64 = 20.0;

fn load_f32(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}
fn store_f32(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::Relaxed)
}

/// Parameters shared between the engine (main thread) and a playing voice (audio thread).
#[derive(Debug, Default)]
pub struct VoiceCtl {
    gain: AtomicU32,
    pan: AtomicU32,
    loop_start: AtomicU32,
    loop_end: AtomicU32,
    stop: AtomicBool,
    finished: AtomicBool,
}

impl VoiceCtl {
    pub fn new(gain: f32, pan: f32) -> VoiceCtl {
        let c = VoiceCtl::default();
        store_f32(&c.gain, gain);
        store_f32(&c.pan, pan);
        c
    }
    pub fn gain(&self) -> f32 {
        load_f32(&self.gain)
    }
    pub fn pan(&self) -> f32 {
        load_f32(&self.pan)
    }
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }
}

/// Decoded samples, interleaved.
#[derive(Debug)]
pub struct Pcm {
    pub channels: u16,
    pub rate: u32,
    pub samples: Vec<f32>,
}

impl Pcm {
    pub fn decode(src: &AudioSource) -> Pcm {
        let dec = src.decoder();
        let channels = dec.channels().get();
        let rate = dec.sample_rate().get();
        Pcm {
            channels,
            rate,
            samples: dec.collect(),
        }
    }
}

enum Data {
    Pcm(Arc<Pcm>),
    Stream(AudioSource),
    Missing,
}

/// One voice as a bevy_audio asset.
#[derive(Asset, TypePath)]
pub struct VoiceSound {
    data: Data,
    ctl: Arc<VoiceCtl>,
    looping: bool,
    pitch: f32,
}

impl VoiceSound {
    pub fn from_pcm(pcm: Arc<Pcm>, ctl: Arc<VoiceCtl>, looping: bool, pitch: f32) -> VoiceSound {
        VoiceSound {
            data: Data::Pcm(pcm),
            ctl,
            looping,
            pitch,
        }
    }
}

type StreamDecoder = <AudioSource as Decodable>::Decoder;

enum Input {
    Pcm {
        pcm: Arc<Pcm>,
        pos: usize,
    },
    Stream {
        src: AudioSource,
        dec: Box<StreamDecoder>,
    },
    Empty,
}

/// The rodio source of a voice: always stereo, gain and pan from [`VoiceCtl`].
pub struct VoiceSource {
    input: Input,
    channels_in: u16,
    rate: u32,
    ctl: Arc<VoiceCtl>,
    looping: bool,
    gain_now: f32,
    right: Option<f32>,
}

impl Decodable for VoiceSound {
    type Decoder = VoiceSource;

    fn decoder(&self) -> VoiceSource {
        let (input, channels_in, rate) = match &self.data {
            Data::Pcm(p) => (
                Input::Pcm {
                    pcm: p.clone(),
                    pos: 0,
                },
                p.channels,
                p.rate,
            ),
            Data::Stream(src) => {
                let dec = src.decoder();
                let (c, r) = (dec.channels().get(), dec.sample_rate().get());
                (
                    Input::Stream {
                        src: src.clone(),
                        dec: Box::new(dec),
                    },
                    c,
                    r,
                )
            }
            Data::Missing => (Input::Empty, 1, 44100),
        };
        let rate = ((rate as f32 * self.pitch.max(0.01)).round() as u32).max(1);
        VoiceSource {
            input,
            channels_in: channels_in.max(1),
            rate,
            ctl: self.ctl.clone(),
            looping: self.looping,
            gain_now: self.ctl.gain(),
            right: None,
        }
    }
}

impl VoiceSource {
    fn finish(&mut self) -> Option<f32> {
        self.ctl.finished.store(true, Ordering::Relaxed);
        self.input = Input::Empty;
        None
    }

    /// Next input frame as (left, right) and whether the input is mono.
    fn frame(&mut self) -> Option<(f32, f32)> {
        let ch = self.channels_in as usize;
        match &mut self.input {
            Input::Pcm { pcm, pos } => {
                let frames = pcm.samples.len() / ch;
                if frames == 0 {
                    return None;
                }
                let rate = pcm.rate as f32;
                let ls = load_f32(&self.ctl.loop_start);
                let le = load_f32(&self.ctl.loop_end);
                let end = if self.looping && le > 0.0 {
                    ((le * rate) as usize).clamp(1, frames)
                } else {
                    frames
                };
                if *pos >= end {
                    if !self.looping {
                        return None;
                    }
                    *pos = ((ls * rate) as usize).min(end - 1);
                }
                let i = *pos * ch;
                *pos += 1;
                let l = pcm.samples[i];
                let r = if ch > 1 { pcm.samples[i + 1] } else { l };
                Some((l, r))
            }
            Input::Stream { src, dec } => {
                let mut first = dec.next();
                if first.is_none() && self.looping {
                    **dec = src.decoder();
                    first = dec.next();
                }
                let l = first?;
                let mut r = l;
                for k in 1..ch {
                    let s = dec.next().unwrap_or(0.0);
                    if k == 1 {
                        r = s;
                    }
                }
                Some((l, r))
            }
            Input::Empty => None,
        }
    }
}

impl Iterator for VoiceSource {
    type Item = bevy::audio::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(r) = self.right.take() {
            return Some(r as _);
        }
        if matches!(self.input, Input::Empty) && self.ctl.finished() {
            return None;
        }
        if self.ctl.stop.load(Ordering::Relaxed) {
            return self.finish().map(|s| s as _);
        }
        let Some((l, r)) = self.frame() else {
            return self.finish().map(|s| s as _);
        };
        // ~5 ms smoothing so gain steps do not click
        self.gain_now += (self.ctl.gain() - self.gain_now) * 0.005;
        let g = self.gain_now;
        let (l, r) = if self.channels_in == 1 {
            let pan = self.ctl.pan();
            (l * g * (1.0 - pan).min(1.0), r * g * (1.0 + pan).min(1.0))
        } else {
            (l * g, r * g)
        };
        self.right = Some(r);
        Some(l as _)
    }
}

impl Source for VoiceSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> bevy::audio::ChannelCount {
        NonZero::new(2).unwrap()
    }
    fn sample_rate(&self) -> bevy::audio::SampleRate {
        NonZero::new(self.rate).unwrap()
    }
    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

struct SoundSlot {
    handle: Handle<AudioSource>,
    volume: f64,
}

struct Fade {
    from: f64,
    to: f64,
    t: f64,
    dur: f64,
}

struct VoiceState {
    snd: String,
    ctl: Arc<VoiceCtl>,
    /// `audio_sound_gain` value.
    gain: f64,
    fade: Option<Fade>,
    /// `sounds.json` volume of the sound.
    base: f64,
    emitter: Option<Emitter>,
    looping: bool,
    pitch: f64,
    loop_set: bool,
}

impl VoiceState {
    fn push_gain(&self) {
        store_f32(&self.ctl.gain, (self.base * self.gain) as f32);
    }
}

/// [`AudioOut`] for the app. Engine calls only change state; [`spawn_voices`] starts queued
/// voices and [`update_voices`] runs fades and forgets finished voices.
#[derive(Resource, Default)]
pub struct BevyOut {
    sounds: HashMap<String, SoundSlot>,
    voices: HashMap<Voice, VoiceState>,
    pending: Vec<Voice>,
    emitters: HashMap<Emitter, f64>,
    next_voice: Voice,
    next_emitter: Emitter,
    pcm: HashMap<String, Arc<Pcm>>,
    ready: bool,
}

impl BevyOut {
    pub fn from_assets(assets: &GameAssets) -> BevyOut {
        let mut out = BevyOut::default();
        for s in &assets.sounds {
            out.add_sound(&s.name, s.handle.clone(), s.volume);
        }
        out
    }

    pub fn add_sound(&mut self, name: &str, handle: Handle<AudioSource>, volume: f64) {
        self.sounds
            .insert(name.to_string(), SoundSlot { handle, volume });
    }

    /// Every sound finished loading (or failed): the engine waits for this, as the original
    /// loads all sounds before its first step.
    pub fn ready(&self) -> bool {
        self.ready
    }

    /// The level the voice is sent at: asset volume × gain (`None` once it is gone).
    pub fn voice_level(&self, v: Voice) -> Option<f64> {
        self.voices.get(&v).map(|s| s.base * s.gain)
    }

    pub fn voice_ctl(&self, v: Voice) -> Option<Arc<VoiceCtl>> {
        self.voices.get(&v).map(|s| s.ctl.clone())
    }

    /// Advance fades by `dt` seconds and drop finished voices.
    pub fn advance(&mut self, dt: f64) {
        self.voices.retain(|_, s| {
            if let Some(f) = &mut s.fade {
                f.t += dt;
                let k = (f.t / f.dur).min(1.0);
                s.gain = f.from + (f.to - f.from) * k;
                if k >= 1.0 {
                    s.fade = None;
                }
            }
            s.push_gain();
            !s.ctl.finished()
        });
    }

    fn make_voice(&mut self, v: Voice, sources: &Assets<AudioSource>) -> Option<VoiceSound> {
        let s = self.voices.get(&v)?;
        let slot = self.sounds.get(&s.snd);
        let src = slot.and_then(|sl| sources.get(&sl.handle));
        let len = myco_core::audio::sound_length(&s.snd).unwrap_or(0.0);
        let data = match src {
            None => Data::Missing,
            Some(src) if s.loop_set || len <= STREAM_OVER_SECS => {
                let pcm = self
                    .pcm
                    .entry(s.snd.clone())
                    .or_insert_with(|| Arc::new(Pcm::decode(src)))
                    .clone();
                Data::Pcm(pcm)
            }
            Some(src) => Data::Stream(src.clone()),
        };
        Some(VoiceSound {
            data,
            ctl: s.ctl.clone(),
            looping: s.looping,
            pitch: s.pitch as f32,
        })
    }
}

impl AudioOut for BevyOut {
    fn play(&mut self, snd: &str, looping: bool, emitter: Option<Emitter>) -> Voice {
        let id = self.next_voice;
        self.next_voice += 1;
        let base = self.sounds.get(snd).map_or(1.0, |s| s.volume);
        let pan = emitter
            .and_then(|e| self.emitters.get(&e))
            .copied()
            .unwrap_or(0.0);
        let state = VoiceState {
            snd: snd.to_string(),
            ctl: Arc::new(VoiceCtl::new(base as f32, pan as f32)),
            gain: 1.0,
            fade: None,
            base,
            emitter,
            looping,
            pitch: 1.0,
            loop_set: false,
        };
        self.voices.insert(id, state);
        self.pending.push(id);
        id
    }

    fn is_playing(&self, v: Voice) -> bool {
        self.voices.get(&v).is_some_and(|s| !s.ctl.finished())
    }

    fn stop(&mut self, v: Voice) {
        if let Some(s) = self.voices.remove(&v) {
            s.ctl.stop.store(true, Ordering::Relaxed);
            s.ctl.finished.store(true, Ordering::Relaxed);
        }
        self.pending.retain(|&p| p != v);
    }

    fn stop_sound(&mut self, snd: &str) {
        let ids: Vec<Voice> = self
            .voices
            .iter()
            .filter(|(_, s)| s.snd == snd)
            .map(|(&id, _)| id)
            .collect();
        for id in ids {
            self.stop(id);
        }
    }

    fn gain(&mut self, v: Voice, gain: f64, ms: f64) {
        if let Some(s) = self.voices.get_mut(&v) {
            if ms > 0.0 {
                s.fade = Some(Fade {
                    from: s.gain,
                    to: gain,
                    t: 0.0,
                    dur: ms / 1000.0,
                });
            } else {
                s.gain = gain;
                s.fade = None;
            }
            s.push_gain();
        }
    }

    fn pitch(&mut self, v: Voice, pitch: f64) {
        if let Some(s) = self.voices.get_mut(&v) {
            s.pitch = pitch;
        }
    }

    fn loop_start(&mut self, v: Voice, pos: f64) {
        if let Some(s) = self.voices.get_mut(&v) {
            s.loop_set = true;
            store_f32(&s.ctl.loop_start, pos as f32);
        }
    }

    fn loop_end(&mut self, v: Voice, pos: f64) {
        if let Some(s) = self.voices.get_mut(&v) {
            s.loop_set = true;
            store_f32(&s.ctl.loop_end, pos as f32);
        }
    }

    fn sound_length(&self, snd: &str) -> f64 {
        myco_core::audio::sound_length(snd).unwrap_or(0.0)
    }

    fn emitter_create(&mut self) -> Emitter {
        let id = self.next_emitter;
        self.next_emitter += 1;
        self.emitters.insert(id, 0.0);
        id
    }

    fn emitter_free(&mut self, e: Emitter) {
        self.emitters.remove(&e);
    }

    fn emitter_position(&mut self, e: Emitter, x: f64, _y: f64, _z: f64) {
        let pan = x.clamp(-1.0, 1.0);
        if let Some(p) = self.emitters.get_mut(&e) {
            *p = pan;
        }
        for s in self.voices.values() {
            if s.emitter == Some(e) {
                store_f32(&s.ctl.pan, pan as f32);
            }
        }
    }
}

/// Start the voices queued since the last call.
pub fn spawn_voices(
    mut commands: Commands,
    out: Option<ResMut<BevyOut>>,
    sources: Res<Assets<AudioSource>>,
    mut voices: ResMut<Assets<VoiceSound>>,
) {
    let Some(mut out) = out else { return };
    for v in std::mem::take(&mut out.pending) {
        if let Some(sound) = out.make_voice(v, &sources) {
            commands.spawn((
                AudioPlayer::<VoiceSound>(voices.add(sound)),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

pub fn update_voices(time: Res<Time>, out: Option<ResMut<BevyOut>>) {
    if let Some(mut out) = out {
        out.advance(time.delta_secs_f64());
    }
}

/// Mark the backend ready once every sound has loaded or failed.
pub fn check_ready(server: Res<AssetServer>, out: Option<ResMut<BevyOut>>) {
    let Some(mut out) = out else { return };
    if out.ready {
        return;
    }
    out.ready = out.sounds.values().all(|s| {
        matches!(
            server.load_state(&s.handle),
            LoadState::Loaded | LoadState::Failed(_)
        )
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pcm(channels: u16, samples: Vec<f32>) -> Arc<Pcm> {
        Arc::new(Pcm {
            channels,
            rate: 10,
            samples,
        })
    }

    fn take(src: &mut VoiceSource, n: usize) -> Vec<f32> {
        src.by_ref().take(n).collect()
    }

    #[test]
    fn mono_is_panned_and_gained_stereo_is_not() {
        let ctl = Arc::new(VoiceCtl::new(0.5, 0.0));
        let mut s = VoiceSound::from_pcm(pcm(1, vec![1.0; 4]), ctl.clone(), false, 1.0).decoder();
        assert_eq!(take(&mut s, 2), [0.5, 0.5]);
        store_f32(&ctl.pan, 0.6);
        let v = take(&mut s, 2);
        assert!(
            (v[0] - 0.2).abs() < 1e-6 && (v[1] - 0.5).abs() < 1e-6,
            "{v:?}"
        );
        store_f32(&ctl.pan, -1.0);
        let v = take(&mut s, 2);
        assert!((v[0] - 0.5).abs() < 1e-6 && v[1].abs() < 1e-6, "{v:?}");

        let ctl = Arc::new(VoiceCtl::new(1.0, 0.9));
        let mut s = VoiceSound::from_pcm(pcm(2, vec![0.25, 0.75]), ctl, false, 1.0).decoder();
        assert_eq!(take(&mut s, 4), [0.25, 0.75]);
        assert_eq!(s.channels().get(), 2);
    }

    #[test]
    fn one_shot_ends_and_reports_finished() {
        let ctl = Arc::new(VoiceCtl::new(1.0, 0.0));
        let mut s = VoiceSound::from_pcm(pcm(1, vec![1.0; 3]), ctl.clone(), false, 1.0).decoder();
        assert_eq!(take(&mut s, 100).len(), 6);
        assert!(ctl.finished());
    }

    #[test]
    fn loop_points_wrap_inside_the_region() {
        // rate 10: loop 0.2 s .. 0.4 s = frames 2..4
        let ctl = Arc::new(VoiceCtl::new(1.0, 0.0));
        store_f32(&ctl.loop_start, 0.2);
        store_f32(&ctl.loop_end, 0.4);
        let samples: Vec<f32> = (0..10).map(|i| i as f32).collect();
        let mut s = VoiceSound::from_pcm(pcm(1, samples), ctl.clone(), true, 1.0).decoder();
        let left: Vec<f32> = take(&mut s, 16).into_iter().step_by(2).collect();
        assert_eq!(left, [0.0, 1.0, 2.0, 3.0, 2.0, 3.0, 2.0, 3.0]);
        ctl.stop.store(true, Ordering::Relaxed);
        assert_eq!(s.next(), None);
        assert!(ctl.finished());
    }

    #[test]
    fn whole_sound_loops_without_points_and_pitch_scales_rate() {
        let ctl = Arc::new(VoiceCtl::new(1.0, 0.0));
        let mut s = VoiceSound::from_pcm(pcm(1, vec![1.0, 2.0]), ctl, true, 2.0).decoder();
        let left: Vec<f32> = take(&mut s, 10).into_iter().step_by(2).collect();
        assert_eq!(left, [1.0, 2.0, 1.0, 2.0, 1.0]);
        assert_eq!(s.sample_rate().get(), 20);
    }

    #[test]
    fn asset_volume_is_applied_once() {
        let mut out = BevyOut::default();
        out.add_sound("rain01", Handle::default(), 0.96);
        let v = out.play("rain01", true, None);
        out.gain(v, 0.5, 0.0);
        assert!((out.voice_level(v).unwrap() - 0.48).abs() < 1e-12);
        assert!((out.voice_ctl(v).unwrap().gain() - 0.48).abs() < 1e-6);
    }

    #[test]
    fn fades_run_over_time_and_emitters_move_their_voices() {
        let mut out = BevyOut::default();
        out.add_sound("bird", Handle::default(), 1.0);
        let e = out.emitter_create();
        out.emitter_position(e, -0.3, 0.0, 0.95);
        let v = out.play("bird", false, Some(e));
        assert!((out.voice_ctl(v).unwrap().pan() + 0.3).abs() < 1e-6);
        out.emitter_position(e, 0.4, 0.0, 0.9);
        assert!((out.voice_ctl(v).unwrap().pan() - 0.4).abs() < 1e-6);
        out.gain(v, 0.0, 1000.0);
        out.advance(0.25);
        assert!((out.voice_level(v).unwrap() - 0.75).abs() < 1e-12);
        out.advance(1.0);
        assert_eq!(out.voice_level(v), Some(0.0));
        assert!(out.is_playing(v));
        out.stop(v);
        assert!(!out.is_playing(v));
        assert!(out.pending.is_empty());
    }
}
