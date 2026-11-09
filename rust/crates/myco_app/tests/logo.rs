//! scr_logo against the LÖVE golden (`rust/golden/data/logo.json`, captured with
//! `MYCO_IDENTITY=MycelariumTest-t18 tools/golden.sh logo`).

use myco_app::logo::*;
use serde_json::Value;

fn golden() -> Value {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/../../golden/data/logo.json");
    serde_json::from_str(&std::fs::read_to_string(p).expect("logo golden")).expect("json")
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 1e-9 * (1.0 + b.abs()),
        "{what}: rust {a} vs love {b}"
    );
}

fn pts_eq(r: &[[f64; 2]], g: &Value, what: &str) {
    let g = g.as_array().unwrap();
    assert_eq!(r.len(), g.len(), "{what}: point count");
    for (i, (p, q)) in r.iter().zip(g).enumerate() {
        close(p[0], f(&q[0]), &format!("{what}[{i}].x"));
        close(p[1], f(&q[1]), &format!("{what}[{i}].y"));
    }
}

fn model_eq(m: &LogoModel, g: &Value, what: &str) {
    assert_eq!(m.letters, g["letters"].as_i64().unwrap(), "{what}: letters");
    for k in ["x1", "x2", "y1", "y2", "letter_x1", "letter_x2", "letter_y1", "letter_y2", "duration"] {
        let v = match k {
            "x1" => m.x1,
            "x2" => m.x2,
            "y1" => m.y1,
            "y2" => m.y2,
            "letter_x1" => m.letter_x1,
            "letter_x2" => m.letter_x2,
            "letter_y1" => m.letter_y1,
            "letter_y2" => m.letter_y2,
            _ => m.duration,
        };
        close(v, f(&g[k]), &format!("{what}.{k}"));
    }
    let gr = g["ribbons"].as_array().unwrap();
    assert_eq!(m.ribbons.len(), gr.len(), "{what}: ribbon count");
    for (i, (r, q)) in m.ribbons.iter().zip(gr).enumerate() {
        let w = format!("{what}.ribbons[{i}]");
        pts_eq(&r.pts, &q["pts"], &w);
        close(r.total, f(&q["total"]), &format!("{w}.total"));
        close(r.nib, f(&q["nib"]), &format!("{w}.nib"));
        close(r.letter, f(&q["letter"]), &format!("{w}.letter"));
        assert_eq!(r.tip_start, q["tip_start"].as_bool().unwrap(), "{w}.tip_start");
        assert_eq!(r.tip_end, q["tip_end"].as_bool().unwrap(), "{w}.tip_end");
        let gl = q["len"].as_array().unwrap();
        assert_eq!(r.len.len(), gl.len());
        for (a, b) in r.len.iter().zip(gl) {
            close(*a, f(b), &format!("{w}.len"));
        }
    }
    let gb = g["blades"].as_array().unwrap();
    assert_eq!(m.blades.len(), gb.len(), "{what}: blade count");
    for (i, (b, q)) in m.blades.iter().zip(gb).enumerate() {
        let w = format!("{what}.blades[{i}]");
        pts_eq(&b.pts, &q["pts"], &w);
        close(b.width, f(&q["width"]), &format!("{w}.width"));
        close(b.root, f(&q["root"]), &format!("{w}.root"));
        close(b.birth, f(&q["birth"]), &format!("{w}.birth"));
        close(b.span, f(&q["span"]), &format!("{w}.span"));
    }
}

/// The brief's test: the logo for "MYCELARIUM" with seed 1 (and the seed `logo_grow_start`
/// actually uses) has the same geometry as in LÖVE, as do a few other texts.
#[test]
fn logo_geometry_matches_love() {
    let g = golden();
    for b in g["builds"].as_array().unwrap() {
        let text = b["text"].as_str().unwrap();
        let seed = f(&b["seed"]);
        let m = logo_build(text, seed);
        model_eq(&m, &b["model"], &format!("{text:?}/{seed}"));
    }
}

#[test]
fn logo_paint_vertices_match_love() {
    let g = golden();
    let m = logo_build("MYCELARIUM", 1.0 + logo_text_seed("MYCELARIUM"));
    for p in g["paint"].as_array().unwrap() {
        let progress = f(&p["progress"]);
        let strips = logo_paint(&m, progress, 960.0, 590.0, 1720.0, 800.0);
        let gs = p["strips"].as_array().unwrap();
        assert_eq!(strips.len(), gs.len(), "strip count at {progress}");
        for (i, (s, q)) in strips.iter().zip(gs).enumerate() {
            let flat = q.as_array().unwrap();
            assert_eq!(s.len() * 2, flat.len(), "strip {i} at {progress}");
            for (k, v) in s.iter().enumerate() {
                close(v[0], f(&flat[2 * k]), &format!("p{progress} s{i} v{k}.x"));
                close(v[1], f(&flat[2 * k + 1]), &format!("p{progress} s{i} v{k}.y"));
            }
        }
    }
}

#[test]
fn logo_helpers_match_love() {
    let g = golden();
    for n in g["noise"].as_array().unwrap() {
        close(logo_noise(f(&n[0]), f(&n[1])), f(&n[2]), "noise");
    }
    for t in g["text_seed"].as_array().unwrap() {
        close(logo_text_seed(t[0].as_str().unwrap()), f(&t[1]), "text_seed");
    }
    for r in g["reach"].as_array().unwrap() {
        close(logo_reach(f(&r[0]), f(&r[1])), f(&r[2]), "reach");
    }
    for t in g["typing"].as_array().unwrap() {
        let mut st = LogoState::logo_init();
        let mut kb = t[0].as_str().unwrap().to_string();
        st.logo_read_typing(&mut kb);
        assert_eq!(st.text, t[1].as_str().unwrap());
        assert_eq!(kb, t[2].as_str().unwrap());
    }
}

#[test]
fn logo_flow_type_grow_reroll_edit() {
    let mut st = LogoState::logo_init();
    let mut kb = String::from("old");
    st.logo_open(&mut kb);
    assert!(st.open && st.phase == LogoPhase::Type && kb.is_empty());
    kb.push_str("  ");
    st.logo_step(&mut kb, true);
    assert_eq!(st.phase, LogoPhase::Type, "only spaces: Enter does nothing");
    kb.push_str("myco");
    st.logo_step(&mut kb, true);
    assert_eq!(st.phase, LogoPhase::Grow);
    assert_eq!(st.text, "  MYCO");
    let dur = st.model.as_ref().unwrap().duration;
    for _ in 0..171 {
        st.logo_step(&mut kb, false);
    }
    assert_eq!(st.progress, dur);
    let a = st.logo_draw(960.0, 590.0, 1720.0, 800.0);
    assert!(st.surface.is_some(), "finished logo is cached");
    assert_eq!(a, st.logo_draw(960.0, 590.0, 1720.0, 800.0));
    let before = st.model.clone();
    st.logo_reroll();
    assert_eq!(st.seed, 2.0);
    assert_ne!(st.model, before);
    assert!(st.surface.is_none() && st.progress == 0.0);
    st.logo_edit(&mut kb);
    assert_eq!((st.phase, kb.as_str()), (LogoPhase::Type, "  MYCO"));
}
