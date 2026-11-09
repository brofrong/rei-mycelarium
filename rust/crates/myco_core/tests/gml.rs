mod common;

use common::{assert_close, f, golden};
use myco_core::gml;

fn rows(g: &serde_json::Value, key: &str) -> Vec<Vec<f64>> {
    let rows = g[key]
        .as_array()
        .unwrap_or_else(|| panic!("golden gml: no {key}"));
    assert!(!rows.is_empty(), "golden gml: empty {key}");
    rows.iter()
        .map(|r| r.as_array().expect("row").iter().map(f).collect())
        .collect()
}

/// Exact (bitwise) for arithmetic-only functions.
fn exact(got: f64, want: f64, what: String) {
    assert!(
        got.to_bits() == want.to_bits() || (got == 0.0 && want == 0.0),
        "{what}: {got:?} vs {want:?}"
    );
}

#[test]
fn matches_love_table() {
    let g = golden("gml");
    for r in rows(&g, "round") {
        exact(gml::round(r[0]), r[1], format!("round({})", r[0]));
    }
    for r in rows(&g, "frac") {
        exact(gml::frac(r[0]), r[1], format!("frac({})", r[0]));
    }
    for r in rows(&g, "sign") {
        exact(gml::sign(r[0]), r[1], format!("sign({})", r[0]));
    }
    for r in rows(&g, "mod") {
        exact(
            gml::gmod(r[0], r[1]),
            r[2],
            format!("mod({}, {})", r[0], r[1]),
        );
    }
    for r in rows(&g, "lerp") {
        exact(gml::lerp(r[0], r[1], r[2]), r[3], format!("lerp{r:?}"));
    }
    for r in rows(&g, "clamp") {
        exact(gml::clamp(r[0], r[1], r[2]), r[3], format!("clamp{r:?}"));
    }
    for r in rows(&g, "angle_difference") {
        exact(
            gml::angle_difference(r[0], r[1]),
            r[2],
            format!("angle_difference({}, {})", r[0], r[1]),
        );
    }
    for r in rows(&g, "point_distance") {
        assert_close(gml::point_distance(r[0], r[1], r[2], r[3]), r[4], 1e-12);
    }
    for r in rows(&g, "point_direction") {
        assert_close(gml::point_direction(r[0], r[1], r[2], r[3]), r[4], 1e-12);
    }
    for r in rows(&g, "lengthdir_x") {
        assert_close(gml::lengthdir_x(r[0], r[1]), r[2], 1e-12);
    }
    for r in rows(&g, "lengthdir_y") {
        assert_close(gml::lengthdir_y(r[0], r[1]), r[2], 1e-12);
    }
}

#[test]
fn round_is_half_even() {
    assert_eq!(gml::round(0.5), 0.0);
    assert_eq!(gml::round(1.5), 2.0);
    assert_eq!(gml::round(2.5), 2.0);
    assert_eq!(gml::round(-0.5), 0.0);
    assert_eq!(gml::round(-1.5), -2.0);
    assert_eq!(gml::round(-2.5), -2.0);
}
