mod common;

use common::{f, golden, i};
use myco_core::rng::Rng;

fn nums(v: &serde_json::Value) -> Vec<f64> {
    v.as_array().expect("array").iter().map(f).collect()
}

fn ints(v: &serde_json::Value) -> Vec<i64> {
    v.as_array().expect("array").iter().map(i).collect()
}

/// Bitwise equality: the same f64 bits, not just approximately equal.
fn assert_bits(got: &[f64], want: &[f64], what: &str) {
    assert_eq!(got.len(), want.len(), "{what}: length");
    for (k, (g, w)) in got.iter().zip(want).enumerate() {
        assert_eq!(g.to_bits(), w.to_bits(), "{what}[{k}]: {g:?} vs {w:?}");
    }
}

#[test]
fn matches_love_sequence() {
    let g = golden("rng");
    let mut r = Rng::new(12345);
    assert_eq!(r.seed(), i(&g["seed"]));
    let random: Vec<f64> = (0..50).map(|_| r.random(1.0)).collect();
    assert_bits(&random, &nums(&g["random"]), "random");
    let irandom: Vec<i64> = (0..20).map(|_| r.irandom(10.0)).collect();
    assert_eq!(irandom, ints(&g["irandom"]), "irandom");
    let range: Vec<i64> = (0..20).map(|_| r.irandom_range(-5.0, 5.0)).collect();
    assert_eq!(range, ints(&g["irandom_range"]), "irandom_range");
    let choose: Vec<i64> = (0..20).map(|_| r.choose(&[1, 2, 3])).collect();
    assert_eq!(choose, ints(&g["choose"]), "choose");
}

#[test]
fn matches_love_extra_seeds() {
    let g = golden("rng");
    let extra = g["extra"].as_array().expect("extra");
    assert!(!extra.is_empty());
    for row in extra {
        let seed = f(&row["seed"]);
        let mut r = Rng::new(seed.floor() as i64);
        assert_eq!(r.seed(), i(&row["got_seed"]), "seed {seed}");
        let what = |s: &str| format!("seed {seed} {s}");
        let v: Vec<f64> = (0..5).map(|_| r.random(1.0)).collect();
        assert_bits(&v, &nums(&row["random"]), &what("random"));
        let v: Vec<f64> = (0..5).map(|_| r.random_range(-3.5, 7.25)).collect();
        assert_bits(&v, &nums(&row["random_range"]), &what("random_range"));
        let v: Vec<i64> = (0..5).map(|_| r.irandom(4.7)).collect();
        assert_eq!(v, ints(&row["irandom_frac"]), "{}", what("irandom_frac"));
        let v: Vec<i64> = (0..5).map(|_| r.irandom_range(9.9, -2.2)).collect();
        assert_eq!(
            v,
            ints(&row["irandom_range_swapped"]),
            "{}",
            what("irandom_range_swapped")
        );
        let v: Vec<i64> = (0..5).map(|_| r.choose(&[10, 20, 30, 40, 50])).collect();
        assert_eq!(v, ints(&row["choose5"]), "{}", what("choose5"));
    }
}

#[test]
fn randomise_reseeds() {
    let mut r = Rng::new(1);
    let s = r.randomise();
    assert_eq!(r.seed(), s);
    assert!((0..2147483647).contains(&s));
    let mut again = Rng::new(s);
    assert_eq!(r.random(1.0).to_bits(), again.random(1.0).to_bits());
}
