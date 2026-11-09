//! Weather (scr_weather): the rain cycle (dry gap → fade in → hold → fade out), the gloom that
//! leads it, the drops/splashes/spray that the render side draws, and wind.
//!
//! Wind itself lives in the save: `state.state.wind = [wind_dir, wind_speed, wind_timer]`
//! (`global.wind_dir/speed/timer`), rerolled by `game_init`/`game_step` (Task 10). Weather is not
//! saved: `weather_init` runs on a new game and on every load.
//!
//! The rain functions run on [`WeatherRt`] with the simulation's RNG passed in, so they can be
//! driven without a whole `Sim`; the `Sim`-level hooks ([`init`], [`step`], [`force_rain`]) turn
//! the thunder cue into `SimEvent::Sound("thunder")`. No weather function changes resources (rain
//! affects water through other modules that ask [`is_raining`]), so nothing here emits
//! `SimEvent::Resource`.
//!
//! Drawing (rain_streak, rain_bands_draw, rain_gloom_draw, rain_splashes_draw, weather_draw_*) is
//! Task 16's (ruling R7); it reads the public fields here and [`WeatherRt::gloom_shade`].

use crate::balance::{vis, wld};
use crate::events::{SimEvent, SoundCue};
use crate::gml;
use crate::rng::Rng;
use crate::sim::Sim;
use crate::state::GameState;

/// `weather.phase`: 0 dry gap, 1 fade in, 2 hold, 3 fade out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum RainPhase {
    #[default]
    Dry,
    FadeIn,
    Hold,
    FadeOut,
}

impl RainPhase {
    pub fn code(self) -> f64 {
        match self {
            RainPhase::Dry => 0.0,
            RainPhase::FadeIn => 1.0,
            RainPhase::Hold => 2.0,
            RainPhase::FadeOut => 3.0,
        }
    }

    pub fn from_code(c: f64) -> Option<RainPhase> {
        match c {
            0.0 => Some(RainPhase::Dry),
            1.0 => Some(RainPhase::FadeIn),
            2.0 => Some(RainPhase::Hold),
            3.0 => Some(RainPhase::FadeOut),
            _ => None,
        }
    }
}

/// One depth band of rain (far, middle, near).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainBand {
    pub speed: f64,
    pub drift: f64,
    pub len: f64,
    pub width: f64,
    pub alpha: f64,
    pub splash: f64,
}

/// A falling drop; `band` indexes [`WeatherRt::bands`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainDrop {
    pub band: usize,
    pub x: f64,
    pub y: f64,
    pub ground: f64,
    pub speed_scale: f64,
    pub len_scale: f64,
    pub alpha_scale: f64,
}

/// A ground splash ring; `t` counts up to `life` frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainSplash {
    pub x: f64,
    pub y: f64,
    pub t: f64,
    pub life: f64,
    pub size: f64,
}

/// A spray droplet thrown up by a splash; it dies at `life` frames or on reaching `ground`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RainSpray {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub ground: f64,
    pub t: f64,
    pub life: f64,
}

/// `global.weather`.
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherRt {
    pub phase: RainPhase,
    /// frames left in the current phase
    pub timer: f64,
    /// 0..1, how hard it rains
    pub intensity: f64,
    /// drops `0..active` are in flight (the rest are scattered afresh when they join)
    pub active: usize,
    /// horizontal drop push this frame (wind × gust × rain_wind_push)
    pub wind: f64,
    pub gust: f64,
    pub gust_target: f64,
    pub gust_timer: f64,
    /// 0..1, the darkening that leads and follows rain
    pub gloom: f64,
    /// rain colour, GML BGR integer (15521990 = #C6D8EC)
    pub colour: u32,
    /// gloom colour, GML BGR integer (11701370 = #7A8BB2)
    pub gloom_colour: u32,
    pub bands: [RainBand; 3],
    pub drops: Vec<RainDrop>,
    pub splashes: Vec<RainSplash>,
    pub spray: Vec<RainSpray>,
}

impl Default for WeatherRt {
    fn default() -> Self {
        WeatherRt::new()
    }
}

/// Band of drop `i`: the original cycles `[0, 1, 2, 0, 1, 0]`, so far drops are the most common.
const BAND_PATTERN: [usize; 6] = [0, 1, 2, 0, 1, 0];

impl WeatherRt {
    // port: weather_init
    /// A fresh dry weather state: the first rain comes after `rain_first_gap_frames`.
    pub fn new() -> WeatherRt {
        let n = 1f64.max(gml::round(vis("rain_max_drops"))) as usize;
        let drops = (0..n)
            .map(|i| RainDrop {
                band: BAND_PATTERN[i % BAND_PATTERN.len()],
                x: 0.0,
                y: 0.0,
                ground: 0.0,
                speed_scale: 1.0,
                len_scale: 1.0,
                alpha_scale: 1.0,
            })
            .collect();
        WeatherRt {
            phase: RainPhase::Dry,
            timer: vis("rain_first_gap_frames"),
            intensity: 0.0,
            active: 0,
            wind: 0.0,
            gust: 1.0,
            gust_target: 1.0,
            gust_timer: 0.0,
            gloom: 0.0,
            colour: 15521990,
            gloom_colour: 11701370,
            bands: [
                RainBand {
                    speed: 13.0,
                    drift: 0.55,
                    len: 13.0,
                    width: 1.0,
                    alpha: 0.2,
                    splash: 0.55,
                },
                RainBand {
                    speed: 19.0,
                    drift: 0.85,
                    len: 22.0,
                    width: 1.5,
                    alpha: 0.32,
                    splash: 0.85,
                },
                RainBand {
                    speed: 26.0,
                    drift: 1.15,
                    len: 34.0,
                    width: 2.3,
                    alpha: 0.46,
                    splash: 1.2,
                },
            ],
            drops,
            splashes: Vec::new(),
            spray: Vec::new(),
        }
    }

    // port: weather_is_raining
    pub fn is_raining(&self) -> bool {
        self.intensity > 0.0
    }

    // port: weather_force_rain
    /// Debug toggle: start rain when dry or fading out (returns true), otherwise start fading out
    /// from the current intensity (returns false).
    pub fn force_rain(&mut self) -> bool {
        if self.phase == RainPhase::Dry || self.phase == RainPhase::FadeOut {
            self.begin();
            return true;
        }
        self.phase = RainPhase::FadeOut;
        self.timer = vis("rain_fade_out_frames") * self.intensity;
        false
    }

    // port: rain_begin
    /// Enter the fade-in, continuing from the current intensity.
    pub fn begin(&mut self) {
        self.phase = RainPhase::FadeIn;
        let fade = 1f64.max(vis("rain_fade_in_frames"));
        self.timer = fade * (1.0 - self.intensity);
        self.gust = 1.0;
        self.gust_target = 1.0;
        self.gust_timer = 0.0;
    }

    // port: rain_phase_step
    /// Advance the rain cycle one frame and the gloom after it. Returns true when thunder sounds.
    pub fn phase_step(&mut self, rng: &mut Rng) -> bool {
        self.timer -= 1.0;
        match self.phase {
            RainPhase::Dry => {
                self.intensity = 0.0;
                if self.timer <= 0.0 {
                    self.begin();
                }
            }
            RainPhase::FadeIn => {
                let fade = 1f64.max(vis("rain_fade_in_frames"));
                self.intensity = gml::clamp(1.0 - self.timer / fade, 0.0, 1.0);
                if self.timer <= 0.0 {
                    self.phase = RainPhase::Hold;
                    let r = rng.random_range(0.7, 1.4);
                    self.timer = vis("rain_hold_frames") * r;
                }
            }
            RainPhase::Hold => {
                self.intensity = 1.0;
                if self.timer <= 0.0 {
                    self.phase = RainPhase::FadeOut;
                    self.timer = 1f64.max(vis("rain_fade_out_frames"));
                }
            }
            RainPhase::FadeOut => {
                let fade = 1f64.max(vis("rain_fade_out_frames"));
                self.intensity = gml::clamp(self.timer / fade, 0.0, 1.0);
                if self.timer <= 0.0 {
                    self.phase = RainPhase::Dry;
                    let gmax = vis("rain_gap_max_frames");
                    let gmin = vis("rain_gap_min_frames");
                    self.timer = rng.irandom_range(gmin, gmax) as f64;
                }
            }
        }
        self.gloom_step()
    }

    // port: rain_gloom_step
    /// Ease the gloom toward 1 from `rain_gloom_lead_frames` before rain until it ends, else
    /// toward 0. Returns true (thunder) when the target turns on while the sky is fully clear.
    pub fn gloom_step(&mut self) -> bool {
        let target = match self.phase {
            RainPhase::Dry => {
                if self.timer <= vis("rain_gloom_lead_frames") {
                    1.0
                } else {
                    0.0
                }
            }
            RainPhase::FadeIn | RainPhase::Hold => 1.0,
            RainPhase::FadeOut => 0.0,
        };
        let thunder = target > 0.0 && self.gloom <= 0.0;
        let rate = 1.0 / 1f64.max(vis("rain_gloom_fade_frames"));
        let step = gml::clamp(target - self.gloom, -rate, rate);
        self.gloom = gml::clamp(self.gloom + step, 0.0, 1.0);
        thunder
    }

    // port: rain_drop_reset
    /// Respawn a drop around the view: anywhere between the view top and its ground when
    /// `anywhere` (rain starting), else just above the view.
    pub fn drop_reset(d: &mut RainDrop, rng: &mut Rng, vx: f64, vy: f64, vw: f64, anywhere: bool) {
        let surf = wld("surface_height_px");
        d.x = (vx - 140.0) + rng.random(vw + 280.0);
        d.ground = (surf - 4.0) + rng.random(7.0);
        d.speed_scale = rng.random_range(0.85, 1.15);
        d.len_scale = rng.random_range(0.75, 1.35);
        d.alpha_scale = rng.random_range(0.6, 1.3);
        if anywhere {
            d.y = (vy - 60.0) + rng.random(80f64.max((d.ground - vy) + 60.0));
        } else {
            d.y = (vy - 40.0) - rng.random(220.0);
        }
    }

    // port: rain_splash_add
    /// A splash ring at (x, y) plus 1–2 spray droplets, within the `rain_max_splashes` /
    /// `rain_max_spray` caps.
    pub fn splash_add(&mut self, rng: &mut Rng, x: f64, y: f64, scale: f64) {
        if self.splashes.len() as f64 >= vis("rain_max_splashes") {
            return;
        }
        let life = rng.irandom_range(15.0, 26.0) as f64;
        let size = rng.random_range(4.0, 9.0) * scale;
        self.splashes.push(RainSplash {
            x,
            y,
            t: 0.0,
            life,
            size,
        });
        if self.spray.len() as f64 >= vis("rain_max_spray") {
            return;
        }
        let n = rng.irandom_range(1.0, 2.0);
        for _ in 0..n {
            let vx = rng.random_range(-1.3, 1.3) + self.wind * 0.12;
            let vy = -rng.random_range(1.1, 2.7) * scale;
            let ground = y + rng.random(3.0);
            let life = rng.irandom_range(10.0, 20.0) as f64;
            self.spray.push(RainSpray {
                x,
                y,
                vx,
                vy,
                ground,
                t: 0.0,
                life,
            });
        }
    }

    // port: rain_splashes_step
    /// Age splashes; move spray under gravity (0.17/frame²) and drop the finished ones.
    pub fn splashes_step(&mut self) {
        self.splashes.retain_mut(|s| {
            s.t += 1.0;
            s.t < s.life
        });
        self.spray.retain_mut(|p| {
            p.t += 1.0;
            p.vy += 0.17;
            p.x += p.vx;
            p.y += p.vy;
            p.t < p.life && p.y < p.ground
        });
    }

    // port: weather_step
    /// One frame of weather for the game view at (vx, vy) of width vw; `wind` is
    /// `wind_dir * wind_speed`. Returns true when thunder sounds.
    pub fn step(&mut self, rng: &mut Rng, wind: f64, vx: f64, vy: f64, vw: f64) -> bool {
        let thunder = self.phase_step(rng);
        self.splashes_step();
        if self.intensity <= 0.0 {
            self.active = 0;
            return thunder;
        }
        self.gust_timer -= 1.0;
        if self.gust_timer <= 0.0 {
            self.gust_timer = rng.irandom_range(90.0, 260.0) as f64;
            self.gust_target = rng.random_range(0.55, 1.7);
        }
        self.gust += (self.gust_target - self.gust) * 0.018;
        self.wind = wind * self.gust * vis("rain_wind_push");
        // `not (vy < surface)` as in the original (also true for a NaN view)
        #[allow(clippy::neg_cmp_op_on_partial_ord)]
        let below = !(vy < wld("surface_height_px"));
        if below {
            self.active = 0;
            return thunder;
        }
        let n = gml::round(self.drops.len() as f64 * self.intensity) as usize;
        let left = vx - 160.0;
        let right = vx + vw + 160.0;
        let chance = vis("rain_splash_chance");
        for i in 0..n {
            let mut d = self.drops[i];
            if i >= self.active {
                Self::drop_reset(&mut d, rng, vx, vy, vw, true);
            } else {
                let b = self.bands[d.band];
                d.x += self.wind * b.drift;
                d.y += b.speed * d.speed_scale;
                if d.y < d.ground {
                    if d.x < left || d.x > right || d.y < vy - 520.0 {
                        Self::drop_reset(&mut d, rng, vx, vy, vw, false);
                    }
                } else {
                    if d.x > vx - 32.0 && d.x < vx + vw + 32.0 && rng.random(1.0) < chance {
                        self.splash_add(rng, d.x, d.ground, b.splash);
                    }
                    Self::drop_reset(&mut d, rng, vx, vy, vw, false);
                }
            }
            self.drops[i] = d;
        }
        self.active = n;
        thunder
    }

    // port: rain_gloom_shade
    /// Smootherstep of the gloom (6g⁵ − 15g⁴ + 10g³), the darkening the render applies.
    pub fn gloom_shade(&self) -> f64 {
        let g = self.gloom;
        let k = g * (g * 6.0 - 15.0) + 10.0;
        g * g * g * k
    }
}

fn thunder(sim: &mut Sim) {
    sim.emit(SimEvent::Sound(SoundCue::new("thunder")));
}

/// `weather_init`; called by `game_init` (Task 10) and after a load.
pub fn init(sim: &mut Sim) {
    sim.weather = WeatherRt::new();
}

/// `weather_step(vx, vy, vw)`; obj_forest_controller Step_0 calls it with the game view unless the
/// UI blocks. `_vh` is unused, as in the original.
pub fn step(sim: &mut Sim, vx: f64, vy: f64, vw: f64, _vh: f64) {
    let w = wind(&sim.state);
    if sim.weather.step(&mut sim.rng, w, vx, vy, vw) {
        thunder(sim);
    }
}

/// `weather_is_raining`.
pub fn is_raining(sim: &Sim) -> bool {
    sim.weather.is_raining()
}

/// `weather_force_rain` (debug).
pub fn force_rain(sim: &mut Sim) -> bool {
    sim.weather.force_rain()
}

/// The wind force `global.wind_dir * global.wind_speed` from `state.state.wind`; positive blows right.
pub fn wind(state: &GameState) -> f64 {
    state.state.wind[0] * state.state.wind[1]
}

/// `rain_gloom_shade(global.weather)`.
pub fn gloom_shade(sim: &Sim) -> f64 {
    sim.weather.gloom_shade()
}

/// Weather is not part of the save (`weather_init` runs on load); wind is already in `state.state.wind`.
pub fn before_save(_sim: &Sim, _out: &mut GameState) {}
