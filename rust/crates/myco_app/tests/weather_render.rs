//! Task 16: rain drawing (scr_weather: rain_streak, rain_bands_draw, rain_gloom_draw,
//! rain_splashes_draw, weather_draw_back/front).

use myco_app::render::weather::{
    rain_bands_draw, rain_gloom_draw, rain_splashes_draw, rain_streak, weather_draw_back,
    weather_draw_front, Vert,
};
use myco_core::weather::{RainDrop, RainSplash, RainSpray, WeatherRt};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn rain_streak_is_a_tapered_quad_fading_to_the_tail() {
    let mut v = Vec::new();
    // straight down, 10 long, 2 wide: head at (5, 20), tail at (5, 10)
    rain_streak(&mut v, 5.0, 20.0, 0.0, 10.0, 2.0, 0x123456 as f64, 0.5);
    assert_eq!(v.len(), 6);
    let want = [
        (4.0, 20.0, 0.5),
        (6.0, 20.0, 0.5),
        (4.7, 10.0, 0.0),
        (4.7, 10.0, 0.0),
        (6.0, 20.0, 0.5),
        (5.3, 10.0, 0.0),
    ];
    for (got, (x, y, a)) in v.iter().zip(want) {
        assert!(
            close(got.x, x) && close(got.y, y) && close(got.alpha, a),
            "{got:?}"
        );
        assert_eq!(got.col, 0x123456 as f64);
    }
    // zero length draws nothing
    let mut e = Vec::new();
    rain_streak(&mut e, 0.0, 0.0, 0.0, 0.0, 2.0, 0.0, 1.0);
    assert!(e.is_empty());
}

fn drop(band: usize, x: f64, y: f64) -> RainDrop {
    RainDrop {
        band,
        x,
        y,
        ground: 480.0,
        speed_scale: 1.0,
        len_scale: 1.0,
        alpha_scale: 1.0,
    }
}

fn raining() -> WeatherRt {
    let mut w = WeatherRt::new();
    w.intensity = 1.0;
    w.wind = 0.0;
    w.drops = vec![
        drop(0, 100.0, 100.0),
        drop(1, 120.0, 100.0),
        drop(2, 140.0, 100.0),
        drop(0, -500.0, 100.0), // left of the view
        drop(1, 160.0, 900.0),  // below the ground line
    ];
    w
}

#[test]
fn bands_draw_only_their_bands_inside_the_view() {
    let w = raining();
    let back = rain_bands_draw(&w, 0.0, 0.0, 0.0, 0.0, 1920.0, 1080.0);
    assert_eq!(back.len(), 6, "one streak: band 0 in view");
    let front = rain_bands_draw(&w, 1.0, 2.0, 0.0, 0.0, 1920.0, 1080.0);
    assert_eq!(
        front.len(),
        12,
        "bands 1 and 2 in view, the low drop is culled"
    );
    // streak heads sit on the drop; alpha = band alpha * intensity
    let b = &w.bands[0];
    assert!(close(back[0].y, 100.0) && close(back[0].alpha, b.alpha));
    // only round(len * intensity) drops are active
    let mut half = raining();
    half.intensity = 0.4; // round(5 * 0.4) = 2 drops: bands 0 and 1
    assert_eq!(
        rain_bands_draw(&half, 0.0, 2.0, 0.0, 0.0, 1920.0, 1080.0).len(),
        12
    );
}

#[test]
fn gloom_multiplies_by_a_tint_towards_the_gloom_colour() {
    let mut w = WeatherRt::new();
    w.gloom = 0.0;
    assert!(rain_gloom_draw(&w).is_none(), "no gloom: nothing drawn");
    w.gloom = 1.0; // shade 1, strength 0.8
    let col = rain_gloom_draw(&w).unwrap() as u32;
    let gc = w.gloom_colour;
    let ch = |c: u32, s: u32| ((c >> s) & 255) as f64;
    let lerp = |c: f64| (255.0 + (c - 255.0) * 0.8).floor() as u32;
    let want = lerp(ch(gc, 16)) << 16 | lerp(ch(gc, 8)) << 8 | lerp(ch(gc, 0));
    assert_eq!(col, want);
}

#[test]
fn splashes_draw_ellipse_rings_and_spray_streaks() {
    let mut w = WeatherRt::new();
    w.splashes = vec![
        RainSplash {
            x: 100.0,
            y: 470.0,
            t: 5.0,
            life: 10.0,
            size: 4.0,
        },
        RainSplash {
            x: 5000.0,
            y: 470.0,
            t: 5.0,
            life: 10.0,
            size: 4.0,
        },
    ];
    w.spray = vec![RainSpray {
        x: 200.0,
        y: 470.0,
        vx: 1.0,
        vy: -2.0,
        ground: 480.0,
        t: 2.0,
        life: 8.0,
    }];
    let v: Vec<Vert> = rain_splashes_draw(&w, 0.0, 1920.0);
    let rings = v.len() - 6;
    assert!(
        rings > 0 && rings.is_multiple_of(6),
        "ring is made of quads"
    );
    // ring: r = 4 * (0.25 + 0.5 * 1.6) = 4.2, alpha (1 - 0.5) * 0.45
    let ring = &v[..rings];
    assert!(ring.iter().all(|p| close(p.alpha, 0.225)));
    let max_x = ring.iter().map(|p| p.x).fold(f64::MIN, f64::max);
    assert!((max_x - (100.0 + 4.2 + 0.5)).abs() < 1e-6, "{max_x}");
    let max_y = ring.iter().map(|p| p.y).fold(f64::MIN, f64::max);
    assert!((max_y - (470.0 + 4.2 * 0.3 + 0.5)).abs() < 1e-6, "{max_y}");
    // spray: streak with alpha 0.5 * (1 - t/life)
    assert!(close(v[rings].alpha, 0.5 * 0.75));
}

#[test]
fn weather_draw_skips_rain_below_the_surface() {
    let mut w = raining();
    w.gloom = 1.0;
    w.splashes.push(RainSplash {
        x: 100.0,
        y: 470.0,
        t: 1.0,
        life: 10.0,
        size: 4.0,
    });
    assert_eq!(weather_draw_back(&w, 0.0, 0.0, 1920.0, 1080.0).len(), 6);
    assert!(weather_draw_back(&w, 0.0, 480.0, 1920.0, 1080.0).is_empty());
    let f = weather_draw_front(&w, 0.0, 0.0, 1920.0, 1080.0);
    assert!(f.gloom.is_some() && f.bands.len() == 12 && !f.splashes.is_empty());
    // underground: only the gloom remains
    let u = weather_draw_front(&w, 0.0, 600.0, 1920.0, 1080.0);
    assert!(u.gloom.is_some() && u.bands.is_empty() && u.splashes.is_empty());
    // dry: no bands
    w.intensity = 0.0;
    assert!(weather_draw_back(&w, 0.0, 0.0, 1920.0, 1080.0).is_empty());
}
