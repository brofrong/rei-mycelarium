//! Task 19: the upgrade-tree screen (scr_skilltree tree_*, obj_hud hud_tree_*) against the LÖVE
//! golden `rust/golden/data/tree.json` (script `rust/golden/scripts/tree.lua`).

use myco_app::ui::tree::{
    tree_bend_point, tree_noise, tree_seed, tree_strand_path, tree_strand_point, HyphaCache,
    TreeInput, TreeView,
};
use myco_core::progress::skilltree::{layout, upg_def, upg_tree_via};
use myco_core::progress::upgrades;
use myco_core::sim::Sim;
use myco_core::state::GameMode;
use serde_json::Value;
use std::path::Path;

fn golden() -> Value {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../golden/data/tree.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn close(a: f64, b: f64, what: &str) {
    let tol = 1e-9 * (1.0 + b.abs());
    assert!((a - b).abs() <= tol, "{what}: rust {a} != love {b}");
}

#[test]
fn tree_helpers_match_love() {
    let g = golden();
    for (id, v) in g["seed"].as_object().unwrap() {
        if id == "__obj" {
            continue;
        }
        close(tree_seed(id), f(v), &format!("seed {id}"));
    }
    for r in g["noise"].as_array().unwrap() {
        close(tree_noise(f(&r[0]), f(&r[1])), f(&r[2]), "noise");
    }
    for r in g["strand"].as_array().unwrap() {
        let (x, y) = tree_strand_point(10.0, 20.0, 300.0, 400.0, 1.7, 12.0, f(&r[0]));
        close(x, f(&r[1]), "strand x");
        close(y, f(&r[2]), "strand y");
    }
    let path = tree_strand_path(10.0, 20.0, 300.0, 400.0, 1.7, 12.0, 6.0, 0.7);
    let gp = g["strand_path"].as_array().unwrap();
    assert_eq!(path.len(), gp.len());
    for (p, r) in path.iter().zip(gp) {
        close(p.0, f(&r[0]), "path x");
        close(p.1, f(&r[1]), "path y");
        close(p.2, f(&r[2]), "path t");
    }
    for r in g["bend"].as_array().unwrap() {
        let (x, y) = tree_bend_point(0.0, 0.0, 50.0, -80.0, 120.0, 30.0, f(&r[0]));
        close(x, f(&r[1]), "bend x");
        close(y, f(&r[2]), "bend y");
    }
}

#[test]
fn tree_hyphae_match_love() {
    let g = golden();
    let mut cache = HyphaCache::default();
    let view = TreeView::default();
    for e in g["hyphae"].as_array().unwrap() {
        let id = e["id"].as_str().unwrap();
        let d = upg_def(id).unwrap();
        let p = upg_def(d.parent).unwrap();
        let wa = view.world(p);
        let wb = view.world(d);
        let h = cache.get(id, wa.0, wa.1, wb.0, wb.1, upg_tree_via(d));
        close(h.main_len, f(&e["main_len"]), &format!("{id} main_len"));
        close(h.total, f(&e["total"]), &format!("{id} total"));
        let gb = e["branches"].as_array().unwrap();
        assert_eq!(h.branches.len(), gb.len(), "{id} branches");
        for (bi, (b, r)) in h.branches.iter().zip(gb).enumerate() {
            let what = format!("{id} branch {bi}");
            assert_eq!(b.cells.len() as f64, f(&r["n"]), "{what} cells");
            close(b.start, f(&r["start"]), &format!("{what} start"));
            close(b.span, f(&r["span"]), &format!("{what} span"));
            let (sx, sy, sw, sd) = b.cells.iter().fold((0.0, 0.0, 0.0, 0.0), |a, c| {
                (a.0 + c.x, a.1 + c.y, a.2 + c.w, a.3 + c.d)
            });
            close(sx, f(&r["sx"]), &format!("{what} sum x"));
            close(sy, f(&r["sy"]), &format!("{what} sum y"));
            close(sw, f(&r["sw"]), &format!("{what} sum w"));
            close(sd, f(&r["sd"]), &format!("{what} sum d"));
            if let Some(cells) = r.get("cells").and_then(Value::as_array) {
                for (c, gc) in b.cells.iter().zip(cells) {
                    close(c.x, f(&gc[0]), &format!("{what} cell x"));
                    close(c.y, f(&gc[1]), &format!("{what} cell y"));
                    close(c.w, f(&gc[2]), &format!("{what} cell w"));
                    close(c.d, f(&gc[3]), &format!("{what} cell d"));
                }
            }
        }
        close(cache.length(id), f(&e["length"]), &format!("{id} length"));
    }
}

#[test]
fn tree_node_positions_match_love() {
    let g = golden();
    for l in g["layouts"].as_array().unwrap() {
        let (gw, gh) = (f(&l["gui"][0]), f(&l["gui"][1]));
        let mut view = TreeView::default();
        view.set_gui_size(gw, gh);
        view.fit();
        let what = format!("gui {gw}x{gh}");
        let c = view.canvas();
        for (k, v) in [c.x1, c.y1, c.x2, c.y2].iter().enumerate() {
            close(*v, f(&l["canvas"][k]), &format!("{what} canvas {k}"));
        }
        close(view.zoom, f(&l["zoom"]), &format!("{what} zoom"));
        close(
            view.fit_zoom,
            f(&l["fit_zoom"]),
            &format!("{what} fit_zoom"),
        );
        close(view.zoom_lo, f(&l["zoom_lo"]), &format!("{what} zoom_lo"));
        close(view.cx, f(&l["cx"]), &format!("{what} cx"));
        close(view.cy, f(&l["cy"]), &format!("{what} cy"));
        let b = view.bounds;
        for (k, v) in [b.x1, b.y1, b.x2, b.y2].iter().enumerate() {
            close(*v, f(&l["bounds"][k]), &format!("{what} bounds {k}"));
        }
        let nodes = l["nodes"].as_array().unwrap();
        assert_eq!(nodes.len(), layout().len());
        for (d, n) in layout().iter().zip(nodes) {
            assert_eq!(d.id, n[0].as_str().unwrap());
            let (x, y) = view.screen(d);
            close(x, f(&n[1]), &format!("{what} {} x", d.id));
            close(y, f(&n[2]), &format!("{what} {} y", d.id));
            close(
                view.radius(d) * view.zoom,
                f(&n[3]),
                &format!("{what} {} r", d.id),
            );
        }
    }
}

/// The tree branch of obj_hud Step_1, frame by frame: hover, buy on click (as
/// `Command::BuyUpgrade`, applied like `hud_begin_step` does), rejection, wheel zoom, drag,
/// recentre, close.
#[test]
fn tree_step_matches_love() {
    let g = golden();
    let mut sim = Sim::new_game(GameMode::Exploration, "", Some(31607));
    sim.state.state.upgrade_points = 3;
    sim.state.state.upgrade_levels = [("origin".to_string(), 1)].into();
    let mut view = TreeView::default();
    view.set_gui_size(1920.0, 1080.0);
    view.closed_step();
    let mut open = true;
    for s in g["steps"].as_array().unwrap() {
        let i = &s["input"];
        let key = i[7].as_str().unwrap();
        let input = TreeInput {
            mx: f(&i[0]),
            my: f(&i[1]),
            down: i[2].as_bool().unwrap(),
            pressed: i[3].as_bool().unwrap(),
            released: i[4].as_bool().unwrap(),
            wheel: (i[5].as_bool().unwrap() as i32 - i[6].as_bool().unwrap() as i32) as f64,
            recentre_key: key == "R",
        };
        let out = view.step(&input);
        if let Some(id) = &out.buy {
            upgrades::buy(&mut sim, id);
        }
        if out.close {
            open = false;
        }
        let label = s["label"].as_str().unwrap();
        assert_eq!(open, s["menu_open"].as_bool().unwrap(), "{label}: open");
        assert_eq!(
            view.hover_id,
            s["hover"].as_str().unwrap(),
            "{label}: hover"
        );
        assert_eq!(
            view.press_id,
            s["press_id"].as_str().unwrap(),
            "{label}: press_id"
        );
        assert_eq!(view.drag, s["drag"].as_bool().unwrap(), "{label}: drag");
        assert_eq!(
            view.dragged,
            s["dragged"].as_bool().unwrap(),
            "{label}: dragged"
        );
        close(view.zoom, f(&s["zoom"]), &format!("{label}: zoom"));
        close(view.cx, f(&s["cx"]), &format!("{label}: cx"));
        close(view.cy, f(&s["cy"]), &format!("{label}: cy"));
        assert_eq!(
            sim.state.state.upgrade_points as f64,
            f(&s["points"]),
            "{label}: points"
        );
        for d in layout() {
            assert_eq!(
                upgrades::upg_level(&sim.state, d.id) as f64,
                f(&s["levels"][d.id]),
                "{label}: level {}",
                d.id
            );
        }
    }
}
