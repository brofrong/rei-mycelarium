//! Save model and slot/settings I/O against files written by the LÖVE port.
//! Fixtures: `tests/fixtures/*.json` (exploration/survival captured with MYCO_DUMP at
//! MYCO_SEED=31607; old_minimal/truncated derived from exploration). Defaults: golden save_fresh.
// values below are copied verbatim from LÖVE output (%.17g), so they carry 17 digits
#![allow(clippy::excessive_precision)]
mod common;

use myco_core::save::{
    list_slots, read_settings, read_slot, write_settings, write_slot, SaveError, SlotStatus,
};
use myco_core::state::{DisplaySettings, GameMode, GameState, SlotMeta};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Fresh temporary directory per test (never the real save folder).
fn temp_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("myco_save_test_{}_{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Put a fixture into `dir` as slot `slot` (save_<slot+1>.json) with a valid meta file.
fn install(dir: &Path, fixture_name: &str, slot: usize) {
    std::fs::copy(
        fixture(fixture_name),
        dir.join(format!("save_{}.json", slot + 1)),
    )
    .unwrap();
    let meta = r#"{"length_cm":10.5,"mode":"exploration","name":"Fixture","saved":46335.4}"#;
    let mut bytes = meta.as_bytes().to_vec();
    bytes.push(0);
    std::fs::write(dir.join(format!("save_{}_meta.json", slot + 1)), bytes).unwrap();
}

fn parse_file(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap();
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    serde_json::from_slice(&bytes[..end]).unwrap()
}

/// Structural equality; numbers compare as f64 with relative tolerance 1e-9 (an integer written
/// as `5` and a double `5.0` are equal, as in GML).
fn assert_values_close(a: &Value, b: &Value, path: &str) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            let scale = 1f64.max(x.abs()).max(y.abs());
            assert!((x - y).abs() <= 1e-9 * scale, "{path}: {x} != {y}");
        }
        (Value::Array(x), Value::Array(y)) => {
            assert_eq!(x.len(), y.len(), "{path}: array length");
            for (i, (p, q)) in x.iter().zip(y).enumerate() {
                assert_values_close(p, q, &format!("{path}[{i}]"));
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            let kx: Vec<_> = x.keys().collect();
            let mut ky: Vec<_> = y.keys().collect();
            let mut kxs = kx.clone();
            kxs.sort();
            ky.sort();
            assert_eq!(kxs, ky, "{path}: keys");
            for k in kx {
                assert_values_close(&x[k], &y[k], &format!("{path}.{k}"));
            }
        }
        _ => assert_eq!(a, b, "{path}"),
    }
}

fn meta() -> SlotMeta {
    SlotMeta {
        length_cm: 59.057214604018057,
        mode: GameMode::Exploration,
        name: "R".into(),
        saved: 46335.456388888888,
    }
}

fn roundtrip(fixture_name: &str, mode: GameMode) {
    let src = temp_dir(&format!("src_{fixture_name}"));
    install(&src, fixture_name, 0);
    let state = read_slot(&src, 0).expect("fixture loads");
    assert_eq!(state.version, 1);
    assert_eq!(state.name, "Fixture");
    assert_eq!(state.state.mode, mode);
    assert_eq!(state.world.seed, 31607);
    assert_eq!(
        state.mycelium.knots.len(),
        2,
        "two knots: start + the placed node"
    );
    assert_eq!(state.mycelium.growths.len(), 2, "two shots");

    let out = temp_dir(&format!("out_{fixture_name}"));
    write_slot(&out, 1, &state, &meta()).unwrap();
    let again = read_slot(&out, 1).expect("written slot loads");

    // read -> write -> read keeps every value ...
    let v1 = serde_json::to_value(&state).unwrap();
    let v2 = serde_json::to_value(&again).unwrap();
    assert_values_close(&v1, &v2, "$");
    // ... and the written file holds exactly what LÖVE wrote (nothing dropped or added)
    assert_values_close(
        &parse_file(&fixture(fixture_name)),
        &parse_file(&out.join("save_2.json")),
        "$",
    );
    let m = parse_file(&out.join("save_2_meta.json"));
    assert_eq!(
        m,
        json!({"length_cm": 59.057214604018057, "mode": "exploration", "name": "R", "saved": 46335.456388888888})
    );
}

#[test]
fn roundtrip_exploration_preserves_values() {
    roundtrip("exploration.json", GameMode::Exploration);
}

#[test]
fn roundtrip_survival_preserves_values() {
    roundtrip("survival.json", GameMode::Survival);
}

#[test]
fn file_ends_with_nul_and_tmp_removed() {
    let src = temp_dir("nul_src");
    install(&src, "exploration.json", 0);
    let state = read_slot(&src, 0).unwrap();
    let out = temp_dir("nul_out");
    write_slot(&out, 0, &state, &meta()).unwrap();
    for f in ["save_1.json", "save_1_meta.json"] {
        let bytes = std::fs::read(out.join(f)).unwrap();
        assert_eq!(*bytes.last().unwrap(), 0, "{f} ends with NUL");
        assert_eq!(
            bytes.iter().filter(|&&b| b == 0).count(),
            1,
            "{f}: single NUL"
        );
    }
    let names: Vec<String> = std::fs::read_dir(&out)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(
        names.iter().all(|n| !n.ends_with(".tmp")),
        "temp files left: {names:?}"
    );
    assert_eq!(names.len(), 2, "{names:?}");
}

/// A LÖVE-written slot file read and written back by Rust is byte-identical.
fn assert_bytes_roundtrip(original_path: &Path, tag: &str) {
    let src = temp_dir(&format!("bytes_src_{tag}"));
    std::fs::copy(original_path, src.join("save_1.json")).unwrap();
    let out = temp_dir(&format!("bytes_out_{tag}"));
    write_slot(&out, 0, &read_slot(&src, 0).unwrap(), &meta()).unwrap();
    let written = std::fs::read(out.join("save_1.json")).unwrap();
    let original = std::fs::read(original_path).unwrap();
    if written != original {
        let at = written
            .iter()
            .zip(&original)
            .position(|(a, b)| a != b)
            .unwrap_or(written.len().min(original.len()));
        let ctx = |b: &[u8]| {
            String::from_utf8_lossy(&b[at.saturating_sub(80)..(at + 40).min(b.len())]).into_owned()
        };
        panic!(
            "{tag}: save text differs from LÖVE's at byte {at}:\n rust: {}\n love: {}",
            ctx(&written),
            ctx(&original)
        );
    }
}

#[test]
fn written_text_matches_love_encoder() {
    // LÖVE's gm.json: sorted keys, integral numbers as integers, others %.17g, NUL at the end.
    assert_bytes_roundtrip(&fixture("exploration.json"), "exploration");
    assert_bytes_roundtrip(&fixture("survival.json"), "survival");
    let out = temp_dir("encoder");
    write_slot(
        &out,
        2,
        &read_slot(
            &{
                let d = temp_dir("encoder_src");
                install(&d, "exploration.json", 0);
                d
            },
            0,
        )
        .unwrap(),
        &meta(),
    )
    .unwrap();
    let m = std::fs::read(out.join("save_3_meta.json")).unwrap();
    assert_eq!(
        String::from_utf8(m).unwrap(),
        "{\"length_cm\":59.057214604018057,\"mode\":\"exploration\",\"name\":\"R\",\"saved\":46335.456388888888}\0"
    );
}

#[test]
fn unknown_fields_survive() {
    let mut v = parse_file(&fixture("exploration.json"));
    v["future_field"] = json!({"a": 1});
    v["state"]["future_field"] = json!({"a": 1});
    v["world"]["chunks"]["c0"]["future_chunk_field"] = json!([1, 2.5, "x"]);
    let state: GameState = serde_json::from_value(v).unwrap();
    let out = temp_dir("unknown");
    write_slot(&out, 0, &state, &meta()).unwrap();
    let back = parse_file(&out.join("save_1.json"));
    assert_eq!(back["future_field"], json!({"a": 1}));
    assert_eq!(back["state"]["future_field"], json!({"a": 1}));
    assert_eq!(
        back["world"]["chunks"]["c0"]["future_chunk_field"],
        json!([1, 2.5, "x"])
    );
}

#[test]
fn mushrooms_and_water_roundtrip() {
    // Shapes from mushroom_add (scr_life) via save_copy_struct, and both water source forms.
    let mut v = parse_file(&fixture("exploration.json"));
    v["mushrooms"] = json!([{
        "active": true, "born": 123456, "carried_by": -1, "charges": 3, "crow_target": false,
        "flash": 0, "flies": [{"angle": 12.5, "bob": 200.25, "radius": 30.5, "speed": 1.75}],
        "hold_angle": 0, "hold_xscale": 1, "hold_yscale": 1, "host": 4, "side": 1, "stench": [],
        "stench_due": 0, "stolen": false, "timer": 312.5, "type": 1, "variant": 2, "x": -44.5,
        "y": 480, "regen_timer": 7
    }]);
    v["water"] = json!([{"lake": 2, "lake_cx": -1}, {"cx": 0, "idx": 17}]);
    let state: GameState = serde_json::from_value(v.clone()).unwrap();
    let m = &state.mushrooms[0];
    assert_eq!((m.kind, m.variant, m.host, m.carried_by), (1, 2, 4, -1));
    assert_eq!(m.flies.len(), 1);
    assert_eq!(m.regen_timer, Some(7.0)); // typed since Task 9
    let out = temp_dir("mush");
    write_slot(&out, 0, &state, &meta()).unwrap();
    assert_values_close(&v, &parse_file(&out.join("save_1.json")), "$");
}

#[test]
fn old_minimal_loads_with_defaults() {
    let fresh_golden = common::golden("save_fresh");
    let fresh: Value = serde_json::from_str(fresh_golden["save_json"].as_str().unwrap()).unwrap();
    let dir = temp_dir("old");
    install(&dir, "old_minimal.json", 0);
    let state = read_slot(&dir, 0).expect("old save loads");
    let st = serde_json::to_value(&state.state).unwrap();

    // load_game defaults: tasks/caves/relics/mode from a fresh game
    assert_values_close(&st["tasks"], &fresh["state"]["tasks"], "tasks");
    assert_values_close(&st["cave_perks"], &json!({}), "cave_perks");
    assert_values_close(&st["cave_taken"], &json!({}), "cave_taken");
    assert_values_close(&st["relic_taken"], &json!({}), "relic_taken");
    assert_eq!(st["mode"], json!("exploration"));
    assert_eq!(st["play_frames"], json!(0));
    assert_eq!(st["water_infinite"], json!(false));
    assert_eq!(st["cave_yellow"], Value::Null);
    // missing tutorial tips are filled from the fresh tutorial (load_game merges tut.order)
    assert_values_close(
        &st["tutorial"]["tips"]["final"],
        &fresh["state"]["tutorial"]["tips"]["final"],
        "tips.final",
    );
    // timers: jitter-free intervals (boar as load_game: boar_spawn_interval * fps)
    assert_values_close(&st["timers"], &fresh_golden["timer_defaults"], "timers");
    // values that were present are kept
    let orig = parse_file(&fixture("old_minimal.json"));
    assert_values_close(&st["food"], &orig["state"]["food"], "food");
    assert_values_close(
        &st["tutorial"]["tips"]["node"],
        &orig["state"]["tutorial"]["tips"]["node"],
        "tips.node",
    );
}

#[test]
fn defaults_match_fresh_save() {
    // Default for whole missing objects equals the fresh-game values (wind and timers jitter-free).
    let g = common::golden("save_fresh");
    let fresh: Value = serde_json::from_str(g["save_json"].as_str().unwrap()).unwrap();
    let mut v = fresh.clone();
    for k in [
        "tasks",
        "timers",
        "tutorial",
        "wind",
        "cam_box",
        "upgrade_levels",
        "food",
        "water_free",
    ] {
        v["state"].as_object_mut().unwrap().remove(k);
    }
    let mut state: GameState = serde_json::from_value(v).unwrap();
    // the boar timer has no plain default: load_game's mode-aware fallback fills it
    state.apply_load_defaults();
    let st = serde_json::to_value(&state.state).unwrap();
    for k in [
        "tasks",
        "tutorial",
        "cam_box",
        "upgrade_levels",
        "food",
        "water_free",
    ] {
        assert_values_close(&st[k], &fresh["state"][k], k);
    }
    assert_values_close(&st["wind"], &g["wind_default"], "wind");
    assert_values_close(&st["timers"], &g["timer_defaults"], "timers");
}

#[test]
fn truncated_is_corrupt_not_panic() {
    let dir = temp_dir("trunc");
    install(&dir, "exploration.json", 0);
    install(&dir, "truncated.json", 1);
    // slot 3: meta itself is garbage
    std::fs::write(dir.join("save_3_meta.json"), b"{\"length_cm\":1").unwrap();
    std::fs::write(dir.join("save_3.json"), b"").unwrap();
    let slots = list_slots(&dir, 4);
    assert_eq!(slots.len(), 4);
    match &slots[0] {
        SlotStatus::Ok(m) => {
            assert_eq!(m.name, "Fixture");
            assert_eq!(m.mode, GameMode::Exploration);
            assert_eq!(m.length_cm, 10.5);
        }
        other => panic!("slot 1: {other:?}"),
    }
    assert!(
        matches!(slots[1], SlotStatus::Corrupt(_)),
        "slot 2: {:?}",
        slots[1]
    );
    assert!(
        matches!(slots[2], SlotStatus::Corrupt(_)),
        "slot 3: {:?}",
        slots[2]
    );
    assert!(
        matches!(slots[3], SlotStatus::Empty),
        "slot 4: {:?}",
        slots[3]
    );
    assert!(matches!(read_slot(&dir, 1), Err(SaveError::Json(_))));
    assert!(matches!(read_slot(&dir, 3), Err(SaveError::Missing)));
}

#[test]
fn wrong_version_is_rejected() {
    let dir = temp_dir("ver");
    let mut v = parse_file(&fixture("exploration.json"));
    v.as_object_mut().unwrap().remove("version");
    std::fs::write(dir.join("save_1.json"), serde_json::to_vec(&v).unwrap()).unwrap();
    assert!(matches!(read_slot(&dir, 0), Err(SaveError::Version(0))));
}

#[test]
fn settings_ini_matches_love() {
    let text = "[display]\r\nfullscreen=\"1.000000\"\r\nwindow_w=\"1280.000000\"\r\nwindow_h=\"720.000000\"\r\n";
    let dir = temp_dir("ini");
    std::fs::write(dir.join("settings.ini"), text).unwrap();
    let s = read_settings(&dir);
    assert_eq!(
        s,
        DisplaySettings {
            fullscreen: true,
            window_w: 1280,
            window_h: 720
        }
    );
    let out = temp_dir("ini_out");
    write_settings(&out, &s).unwrap();
    assert_eq!(
        std::fs::read_to_string(out.join("settings.ini")).unwrap(),
        text
    );

    write_settings(
        &out,
        &DisplaySettings {
            fullscreen: false,
            window_w: 1600,
            window_h: 900,
        },
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(out.join("settings.ini")).unwrap(),
        "[display]\r\nfullscreen=\"0.000000\"\r\nwindow_w=\"1600.000000\"\r\nwindow_h=\"900.000000\"\r\n"
    );
    // missing file / bad values -> ini_to_settings defaults
    assert_eq!(
        read_settings(&temp_dir("ini_none")),
        DisplaySettings {
            fullscreen: true,
            window_w: 1280,
            window_h: 720
        }
    );
    std::fs::write(
        dir.join("settings.ini"),
        "[other]\nwindow_w=\"5\"\n[display]\nwindow_w=\"-3\"\nwindow_h = 801.7\nfullscreen=\"0\"\n",
    )
    .unwrap();
    assert_eq!(
        read_settings(&dir),
        DisplaySettings {
            fullscreen: false,
            window_w: 1280,
            window_h: 801
        }
    );
}

#[test]
fn save_dir_is_love_folder() {
    let d = myco_core::save::save_dir();
    assert!(
        d.ends_with("Library/Application Support/LOVE/Mycelarium"),
        "{}",
        d.display()
    );
}

/// The owner's real LÖVE slots (read only; copies go to a temp dir) survive a Rust round trip
/// byte for byte. Run with `cargo test -p myco_core --test save -- --ignored`.
#[test]
#[ignore]
fn real_saves_roundtrip_bytes() {
    for slot in 0..3 {
        let p = myco_core::save::save_dir().join(format!("save_{}.json", slot + 1));
        if p.exists() {
            assert_bytes_roundtrip(&p, &format!("real{slot}"));
        }
    }
}

/// load_game runs in the live room (meta_3.lua:820): `current_time` keeps counting from boot and
/// the game RNG continues from the title, nothing reseeds it. A loaded game continues the title
/// sim's clock and RNG; a restart to the title (`game_restart`) keeps the clock too.
#[test]
fn load_continues_the_live_clock_and_rng() {
    use myco_core::sim::{Sim, StepInput};
    let dir = std::env::temp_dir().join(format!("myco-t22-live-load-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::copy(fixture("exploration.json"), dir.join("save_1.json")).unwrap();
    let mut title = Sim::title_screen(Some(31607));
    for _ in 0..90 {
        title.step(&StepInput::default());
    }
    let (frame, seed) = (title.frame, title.rng.seed());
    let loaded = title.load_game_live(&dir, 0).unwrap();
    assert_eq!(loaded.frame, frame, "current_time continues");
    assert_eq!(loaded.rng.seed(), seed, "the game RNG is the title's");
    assert_eq!(loaded.save.slot, 0);
    assert!(!loaded.title_open);
    assert_ne!(loaded.session, title.session);
    let restarted = title.restart_title(Some(31607));
    assert_eq!(restarted.frame, frame);
    assert!(restarted.title_open);
    // a GameMaker global no Create event resets survives game_restart
    assert!(title.forest.backdrop_leaves.is_some());
    assert_eq!(restarted.forest.backdrop_leaves, title.forest.backdrop_leaves);
    let _ = std::fs::remove_dir_all(&dir);
}
