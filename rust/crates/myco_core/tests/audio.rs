//! Logic-side audio helpers (`myco_core::audio`) against the LÖVE reference
//! (`rust/golden/data/audio.json`, script `rust/golden/scripts/audio.lua`).
mod common;

use common::{assert_close, f, golden};
use myco_core::audio::{self, HyphaeStart};
use myco_core::events::{AudioCall, SimEvent};
use myco_core::rng::Rng;

#[test]
fn sound_lengths_match_love() {
    let g = golden("audio");
    let lengths = g["lengths"].as_object().unwrap();
    assert_eq!(lengths.len(), 24);
    for (name, v) in lengths {
        let len = audio::sound_length(name).unwrap_or_else(|| panic!("no length for {name}"));
        assert_close(len, f(v), 1e-12);
    }
    assert_eq!(audio::sound_length("nope"), None);
}

/// For a non-positional hyphae sound (mycelium growth) the frame on which the original sets
/// `s.done` follows from the plan alone: the growth stays active until then.
#[test]
fn hyphae_done_frame_matches_love() {
    let g = golden("audio");
    let mut checked = 0;
    for h in g["hyphae"].as_array().unwrap() {
        let c = h["case"].as_array().unwrap();
        if !c[2].is_string() || f(&c[6]) >= 0.0 {
            continue; // positioned (end voice decides) or stopped early
        }
        let start = if f(&c[4]) == 1.0 {
            HyphaeStart::One
        } else {
            HyphaeStart::Two
        };
        let plan = audio::hyphae_plan(f(&c[0]), f(&c[1]), start);
        let sfx_on = f(&c[7]) > 0.0;
        assert_eq!(plan.done_at(sfx_on), f(&h["done_at"]), "case {c:?}");
        checked += 1;
    }
    assert_eq!(checked, 6);
}

#[test]
fn hyphae_plan_picks_end_sound_by_length() {
    // vis: hyphae_len_tiny 80, mid 128, long 224; end_at = round(total * 0.85)
    let p = audio::hyphae_plan(250.0, 200.0, HyphaeStart::One);
    assert_eq!(
        (p.end_sound, p.end_at, p.tiny),
        ("hyphae_end_long", 170.0, false)
    );
    assert_eq!(p.middle_at, 22.0); // round(0.72 * 0.5 * 60)
    let p = audio::hyphae_plan(130.0, 200.0, HyphaeStart::Two);
    assert_eq!((p.end_sound, p.middle_at), ("hyphae_end_mid", 23.0));
    assert_eq!(
        audio::hyphae_plan(100.0, 200.0, HyphaeStart::One).end_sound,
        "hyphae_end_short"
    );
    // tiny: too short, or the middle would start after the end
    assert!(audio::hyphae_plan(79.0, 200.0, HyphaeStart::One).tiny);
    let p = audio::hyphae_plan(300.0, 20.0, HyphaeStart::One);
    assert!(p.tiny);
    assert_eq!(p.end_sound, "hyphae_end_short");
}

#[test]
fn logic_rolls_choices_with_the_game_rng() {
    let mut a = Rng::new(5);
    let mut b = Rng::new(5);
    let (e, plan) = audio::hyphae_begin(&mut a, 7, 100.0, 90.0, None);
    let want = b.choose(&[HyphaeStart::One, HyphaeStart::Two]);
    assert_eq!(
        e,
        SimEvent::Audio(AudioCall::HyphaeBegin {
            id: 7,
            len: 100.0,
            total: 90.0,
            at: None,
            start: want
        })
    );
    assert_eq!(plan, audio::hyphae_plan(100.0, 90.0, want));
    let e = audio::mushroom_grow(&mut a, Some((1.0, 2.0)), Some(0.5));
    let want = b.choose(&["mush_grow_1", "mush_grow_2", "mush_grow_3"]);
    assert_eq!(
        e,
        SimEvent::Audio(AudioCall::MushroomGrow {
            sound: want,
            at: Some((1.0, 2.0)),
            volume: Some(0.5)
        })
    );
    assert_eq!(a.next01(), b.next01());
}
