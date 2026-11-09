//! Shared helpers for golden tests: reference values captured from the LÖVE port with
//! `tools/golden.sh <topic>` live in `rust/golden/data/<topic>.json`.
#![allow(dead_code)]

use std::path::PathBuf;

pub fn golden_path(topic: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../golden/data")
        .join(format!("{topic}.json"))
}

/// Load `rust/golden/data/<topic>.json`; panics with the capture command when it is missing.
pub fn golden(topic: &str) -> serde_json::Value {
    let path = golden_path(topic);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "{}: {e} (capture it with tools/golden.sh {topic})",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: bad JSON: {e}", path.display()))
}

/// Relative closeness: |a-b| <= rel * max(1, |a|, |b|) (so values near zero compare absolutely).
pub fn assert_close(a: f64, b: f64, rel: f64) {
    if a == b || (a.is_nan() && b.is_nan()) {
        return;
    }
    let scale = 1f64.max(a.abs()).max(b.abs());
    assert!(
        (a - b).abs() <= rel * scale,
        "assert_close failed: {a:?} vs {b:?} (rel {rel})"
    );
}

pub fn f(v: &serde_json::Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

pub fn i(v: &serde_json::Value) -> i64 {
    let x = f(v);
    assert_eq!(x, x.trunc(), "not an integer: {v}");
    x as i64
}
