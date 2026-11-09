//! Weather (scr_weather): rain phase/gloom cycle, drops, splashes and spray, forced rain, gloom
//! shade and wind, against `rust/golden/data/weather.json` (rust/golden/scripts/weather.lua).

mod common;

use common::{f, golden, i};
use myco_core::events::SimEvent;
use myco_core::rng::Rng;
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use myco_core::weather::{self, RainPhase, WeatherRt};
use serde_json::Value;

const TOL: f64 = 1e-9;

fn close(a: f64, b: f64, what: &str) {
    let scale = 1f64.max(a.abs()).max(b.abs());
    assert!(
        (a - b).abs() <= TOL * scale,
        "{what}: rust {a:?} vs love {b:?}"
    );
}

fn arr(v: &Value) -> &Vec<Value> {
    v.as_array().unwrap_or_else(|| panic!("not an array: {v}"))
}

fn summary(w: &WeatherRt) -> Vec<f64> {
    let sx: f64 = w.drops.iter().map(|d| d.x).sum();
    let sy: f64 = w.drops.iter().map(|d| d.y).sum();
    vec![
        w.phase.code(),
        w.timer,
        w.intensity,
        w.active as f64,
        w.wind,
        w.gust,
        w.gust_target,
        w.gust_timer,
        w.gloom,
        w.gloom_shade(),
        w.splashes.len() as f64,
        w.spray.len() as f64,
        sx,
        sy,
        if w.is_raining() { 1.0 } else { 0.0 },
    ]
}

fn check_summary(w: &WeatherRt, want: &Value, what: &str) {
    let got = summary(w);
    let want = arr(want);
    assert_eq!(got.len(), want.len(), "{what}: summary length");
    const NAMES: [&str; 15] = [
        "phase",
        "timer",
        "intensity",
        "active",
        "wind",
        "gust",
        "gust_target",
        "gust_timer",
        "gloom",
        "shade",
        "splashes",
        "spray",
        "sum x",
        "sum y",
        "raining",
    ];
    for (k, (g, e)) in got.iter().zip(want).enumerate() {
        // sums over 560 drops accumulate rounding in a different order: relative 1e-9 still holds
        close(*g, f(e), &format!("{what}: {}", NAMES[k]));
    }
}

fn check_full(w: &WeatherRt, want: &Value, what: &str) {
    let drops = arr(&want["drops"]);
    assert_eq!(w.drops.len(), drops.len(), "{what}: drop count");
    for (n, (d, e)) in w.drops.iter().zip(drops).enumerate() {
        let e = arr(e);
        assert_eq!(d.band as i64, i(&e[0]), "{what}: drop {n} band");
        let got = [
            d.x,
            d.y,
            d.ground,
            d.speed_scale,
            d.len_scale,
            d.alpha_scale,
        ];
        for (k, g) in got.iter().enumerate() {
            close(
                *g,
                f(&e[k + 1]),
                &format!("{what}: drop {n} field {}", k + 1),
            );
        }
    }
    let splashes = arr(&want["splashes"]);
    assert_eq!(w.splashes.len(), splashes.len(), "{what}: splash count");
    for (n, (s, e)) in w.splashes.iter().zip(splashes).enumerate() {
        let got = [s.x, s.y, s.t, s.life, s.size];
        for (k, g) in got.iter().enumerate() {
            close(*g, f(&arr(e)[k]), &format!("{what}: splash {n} field {k}"));
        }
    }
    let spray = arr(&want["spray"]);
    assert_eq!(w.spray.len(), spray.len(), "{what}: spray count");
    for (n, (p, e)) in w.spray.iter().zip(spray).enumerate() {
        let got = [p.x, p.y, p.vx, p.vy, p.ground, p.t, p.life];
        for (k, g) in got.iter().enumerate() {
            close(*g, f(&arr(e)[k]), &format!("{what}: spray {n} field {k}"));
        }
    }
    check_summary(w, &want["summary"], what);
}

/// The view each golden run used (must mirror rust/golden/scripts/weather.lua).
fn view(run: usize, fr: u64) -> (f64, f64, f64, f64) {
    let fr = fr as f64;
    match run {
        0 => (1000.0 + (fr % 900.0) * 2.0, 0.0, 1920.0, 1080.0),
        1 => {
            let vy = if (900.0..1100.0).contains(&fr) {
                600.0
            } else {
                -120.0 + fr % 300.0
            };
            (4000.0 - (fr % 1200.0) * 3.0, vy, 1600.0, 900.0)
        }
        2 => (200.5 + fr % 50.0, 100.0, 640.0, 360.0),
        _ => unreachable!(),
    }
}

#[test]
fn weather_init_matches_love() {
    let g = golden("weather");
    let w = WeatherRt::new();
    assert_eq!(w.drops.len() as f64, f(&g["vis"]["rain_max_drops"]));
    assert_eq!(w.phase, RainPhase::Dry);
    check_full(&w, &g["runs"][0]["init"], "init");
    assert_eq!(w.colour, 15521990);
    assert_eq!(w.gloom_colour, 11701370);
    let b = &w.bands;
    assert_eq!((b[0].speed, b[1].speed, b[2].speed), (13.0, 19.0, 26.0));
    assert_eq!((b[0].splash, b[1].splash, b[2].splash), (0.55, 0.85, 1.2));
}

#[test]
fn rain_phase_matches_love() {
    let g = golden("weather");
    for (run, r) in arr(&g["runs"]).iter().enumerate() {
        let seed = i(&r["seed"]);
        let wind = f(&r["wind_dir"]) * f(&r["wind_speed"]);
        let mut rng = Rng::new(seed);
        let mut w = WeatherRt::new();
        let timer = f(&r["timer"]);
        if timer >= 0.0 {
            w.timer = timer;
        }
        let frames = i(&r["frames"]) as u64;
        let mut rows = arr(&r["rows"]).iter();
        let mut thunders = Vec::new();
        for fr in 1..=frames {
            let (vx, vy, vw, _vh) = view(run, fr);
            if w.step(&mut rng, wind, vx, vy, vw) {
                thunders.push(fr as i64);
            }
            let every = i(&r["every"]) as u64;
            if fr % every == 0 || fr == frames {
                let row = arr(rows.next().expect("golden row"));
                assert_eq!(i(&row[0]) as u64, fr);
                check_summary(&w, &row[1], &format!("run {run} frame {fr}"));
            }
        }
        assert!(rows.next().is_none(), "run {run}: unused golden rows");
        let want_thunders: Vec<i64> = arr(&r["thunders"]).iter().map(i).collect();
        assert_eq!(thunders, want_thunders, "run {run}: thunder frames");
        check_full(&w, &r["final"], &format!("run {run} final"));
        close(
            rng.random(1.0),
            f(&r["rng_after"]),
            &format!("run {run}: rng after"),
        );
    }
}

#[test]
fn force_rain_matches_love() {
    let g = golden("weather");
    for row in arr(&g["force"]) {
        let row: Vec<f64> = arr(row).iter().map(f).collect();
        let mut w = WeatherRt::new();
        w.phase = RainPhase::from_code(row[0]).unwrap();
        w.timer = row[1];
        w.intensity = row[2];
        w.gust = row[3];
        w.gust_target = row[3];
        w.gust_timer = 33.0;
        let r = w.force_rain();
        let what = format!("force from phase {}", row[0]);
        assert_eq!(r, row[4] == 1.0, "{what}: result");
        assert_eq!(w.phase.code(), row[5], "{what}: phase");
        let got = [w.timer, w.intensity, w.gust, w.gust_target, w.gust_timer];
        for (k, gv) in got.iter().enumerate() {
            close(*gv, row[6 + k], &format!("{what}: field {}", 6 + k));
        }
    }
}

#[test]
fn gloom_shade_matches_love() {
    let g = golden("weather");
    let mut w = WeatherRt::new();
    for row in arr(&g["shade"]) {
        let row = arr(row);
        w.gloom = f(&row[0]);
        close(
            w.gloom_shade(),
            f(&row[1]),
            &format!("shade at {}", w.gloom),
        );
    }
}

#[test]
fn splash_add_matches_love() {
    let g = golden("weather");
    let want = &g["splash_add"];
    let mut rng = Rng::new(99);
    let mut w = WeatherRt::new();
    w.wind = -1.7;
    for n in 1..=130 {
        let nf = n as f64;
        w.splash_add(
            &mut rng,
            nf * 3.5,
            480.0 + (n % 4) as f64,
            0.55 + (n % 3) as f64 * 0.3,
        );
    }
    check_full(&w, want, "splash_add");
    close(
        rng.random(1.0),
        f(&want["rng_after"]),
        "splash_add: rng after",
    );
}

#[test]
fn wind_matches_love() {
    // wind = wind_dir * wind_speed (state.state.wind = [dir, speed, timer]); weather_step scales it by
    // the gust and rain_wind_push, which rain_phase_matches_love checks through w.wind.
    let mut sim = Sim::new_game(GameMode::Exploration, "Test", Some(31607));
    sim.state.state.wind = [-1.0, 2.75, 100.0];
    assert_eq!(weather::wind(&sim.state), -2.75);
    sim.state.state.wind = [1.0, 1.25, 3.0];
    assert_eq!(weather::wind(&sim.state), 1.25);
}

#[test]
fn sim_hook_uses_state_wind_and_emits_thunder() {
    let g = golden("weather");
    let r = &g["runs"][1];
    let mut sim = Sim::new_game(GameMode::Exploration, "Test", Some(31607));
    sim.drain_events();
    sim.rng = Rng::new(i(&r["seed"]));
    weather::init(&mut sim);
    sim.weather.timer = f(&r["timer"]);
    sim.state.state.wind[0] = f(&r["wind_dir"]);
    sim.state.state.wind[1] = f(&r["wind_speed"]);
    let thunders: Vec<u64> = arr(&r["thunders"]).iter().map(|v| i(v) as u64).collect();
    // run 1 samples every 3rd frame; step the hook to frame 600 and compare that row
    for fr in 1..=600u64 {
        let (vx, vy, vw, vh) = view(1, fr);
        weather::step(&mut sim, vx, vy, vw, vh);
        let thunder = sim
            .drain_events()
            .iter()
            .any(|e| matches!(e, SimEvent::Sound(c) if c.name == "thunder"));
        assert_eq!(thunder, thunders.contains(&fr), "thunder at frame {fr}");
    }
    let row = arr(&arr(&r["rows"])[199]);
    assert_eq!(i(&row[0]), 600);
    check_summary(&sim.weather, &row[1], "hook frame 600");
    assert_eq!(weather::is_raining(&sim), sim.weather.intensity > 0.0);
    // force_rain from the dry phase starts the fade-in at once
    sim.weather = WeatherRt::new();
    assert!(weather::force_rain(&mut sim));
    assert_eq!(sim.weather.phase, RainPhase::FadeIn);
}
