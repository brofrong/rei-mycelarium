//! Stones, mazes, rock clusters and surface obstacles (Task 7) against the LÖVE port.
//! Golden topics: stones_art (pixel-derived stone data), stones_maze (maze_rng, obstacle bands,
//! rock clusters, built mazes/clusters, blob/cut/maze primitives), stones_s31607 / stones_s7
//! (worlds with everything live, R19; shoving and rolling stones, a redress at a maze border).
mod common;

use common::{f, golden, i};
use myco_core::rng::Rng;
use myco_core::sim::Sim;
use myco_core::state::{GameMode, WorldChunkSave};
use myco_core::world::{maze, obstacles, stones, WorldRt};
use serde_json::{json, Value};
use std::collections::HashSet;

/// Decode `[[value, count], ...]`.
fn unrle(v: &Value) -> Vec<f64> {
    let mut out = Vec::new();
    for run in v.as_array().unwrap() {
        let r = run.as_array().unwrap();
        for _ in 0..i(&r[1]) {
            out.push(f(&r[0]));
        }
    }
    out
}

/// Structural JSON equality: numbers as f64, "undefined" equals null, key order ignored.
fn json_eq(path: &str, got: &Value, want: &Value) {
    match (got, want) {
        (Value::Number(a), Value::Number(b)) => {
            assert_eq!(a.as_f64().unwrap(), b.as_f64().unwrap(), "{path}")
        }
        (Value::Null, Value::String(s)) | (Value::String(s), Value::Null) if s == "undefined" => {}
        (Value::Bool(a), Value::Number(b)) | (Value::Number(b), Value::Bool(a)) => {
            assert_eq!(*a as i64 as f64, b.as_f64().unwrap(), "{path}")
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: length");
            for (k, (x, y)) in a.iter().zip(b).enumerate() {
                json_eq(&format!("{path}[{k}]"), x, y);
            }
        }
        (Value::Object(a), Value::Array(b)) if a.is_empty() && b.is_empty() => {}
        (Value::Array(a), Value::Object(b)) if a.is_empty() && b.is_empty() => {}
        (Value::Object(a), Value::Object(b)) => {
            let mut ka: Vec<_> = a.keys().collect();
            let mut kb: Vec<_> = b.keys().collect();
            ka.sort();
            kb.sort();
            assert_eq!(ka, kb, "{path}: keys");
            for (k, x) in a {
                json_eq(&format!("{path}.{k}"), x, &b[k]);
            }
        }
        _ => assert_eq!(got, want, "{path}"),
    }
}

fn to_json<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap()
}

fn rle(v: &[f64]) -> Value {
    let mut out: Vec<Value> = Vec::new();
    let mut k = 0;
    while k < v.len() {
        let mut n = 1;
        while k + n < v.len() && v[k + n] == v[k] {
            n += 1;
        }
        out.push(json!([v[k], n]));
        k += n;
    }
    Value::Array(out)
}

// ---------------------------------------------------------------------------------------------
// pixel-derived data

#[test]
fn stone_art_matches_love() {
    let g = golden("stones_art");
    let art = stones::stone_art();
    assert_eq!(art.reach, f(&g["reach"]));
    assert_eq!(stones::stone_reach(&WorldRt::default()), f(&g["reach"]));
    for (k, want) in g["kinds"].as_array().unwrap().iter().enumerate() {
        let kd = &art.kinds[k];
        for (key, got) in [
            ("left", kd.left),
            ("w", kd.w),
            ("ox", kd.ox),
            ("oy", kd.oy),
            ("radius", kd.radius),
            ("body", kd.body),
        ] {
            assert_eq!(got, f(&want[key]), "kind {k}.{key}");
        }
    }
    let mut n = 0;
    for t in 0..49i64 {
        for y in -1..=48 {
            let mut x = -1.0;
            while x <= 49.0 {
                let want = i(&g["solid"][n]) == 1;
                assert_eq!(
                    stones::stone_tile_solid_at(t % 7, t / 7, x, y as f64 + 0.5),
                    want,
                    "tile {t} ({x},{y})"
                );
                n += 1;
                x += 7.0;
            }
        }
    }
    for r in g["roles"].as_array().unwrap() {
        let mask = i(&r[0]);
        let depth = f(&r[1]);
        let got = stones::stone_role_tiles(
            mask & 1 != 0,
            mask & 2 != 0,
            mask & 4 != 0,
            mask & 8 != 0,
            depth,
        );
        let got: Vec<Value> = got.iter().map(|&(a, b)| json!([a, b])).collect();
        json_eq(&format!("roles {r}"), &Value::Array(got), &r[2]);
    }
    let inner = [(true, true), (true, false), (false, true), (false, false)];
    for (k, &(up, left)) in inner.iter().enumerate() {
        let (a, b) = stones::stone_inner_corner_tile(up, left);
        json_eq("inner", &json!([a, b]), &g["inner"][k]);
    }
}

#[test]
fn stones_slice_sheet_finds_seven_columns() {
    // seven filled columns separated by gaps
    let (w, h) = (40usize, 3usize);
    let filled = |x: usize| x % 6 < 4 && x < 41;
    let sl = stones::stones_slice_sheet(|x, _y| if filled(x) { 255 } else { 0 }, w, h);
    assert_eq!(sl.len(), 7);
    assert_eq!((sl[0].left, sl[0].width), (0.0, 4.0));
    assert_eq!((sl[6].left, sl[6].width), (36.0, 4.0));
    // anything but seven runs falls back to even sevenths
    let sl = stones::stones_slice_sheet(|_, _| 255, 70, 2);
    assert_eq!(sl.len(), 7);
    assert_eq!((sl[3].left, sl[3].width), (30.0, 10.0));
}

// ---------------------------------------------------------------------------------------------
// maze, obstacle and cluster generators

fn new_world(seed: i64) -> Sim {
    Sim::new_game(GameMode::Exploration, "Golden", Some(seed))
}

fn cells_json(blob: &stones::StoneBlob) -> Value {
    let list: Vec<Value> = blob.list.iter().map(|&(c, r)| json!([c, r])).collect();
    let mut keys: Vec<String> = blob
        .keys
        .iter()
        .map(|&(c, r)| stones::stone_cell_key(c, r))
        .collect();
    keys.sort();
    json!({ "list": list, "keys": keys })
}

fn open_json(m: &maze::Maze) -> Value {
    let v: Vec<f64> = m.open.iter().map(|&o| o as i64 as f64).collect();
    rle(&v)
}

fn rooms_json(rooms: &[maze::MazeRoom], full: bool) -> Value {
    Value::Array(
        rooms
            .iter()
            .map(|r| {
                if full {
                    json!({ "cx": r.cx, "cy": r.cy, "rx": r.rx, "ry": r.ry, "water": r.water,
                            "food_min": r.food_min, "food_max": r.food_max })
                } else {
                    json!({ "cx": r.cx, "cy": r.cy, "rx": r.rx, "ry": r.ry })
                }
            })
            .collect(),
    )
}

fn pts_json(p: &[(f64, f64)]) -> Value {
    Value::Array(p.iter().map(|&(x, y)| json!([x, y])).collect())
}

fn part_json(p: &myco_core::state::ObstacleSave) -> Value {
    let mut v = to_json(p);
    v.as_object_mut().unwrap().remove("data");
    v
}

#[test]
fn maze_matches_love() {
    let g = golden("stones_maze");
    let mut sim = new_world(31607);
    let seed = sim.world.params.seed;
    assert_eq!(seed as f64, f(&g["seed"]));
    assert_eq!(sim.world.params.start_x, f(&g["start_x"]));

    // maze_rng sequences
    for r in g["rng"].as_array().unwrap() {
        let mut m = maze::Maze::rng_only(seed, f(&r["key"]));
        for (k, want) in r["seq"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                maze::maze_rng(&mut m),
                f(want),
                "maze_rng key {} #{k}",
                r["key"]
            );
        }
        for (k, want) in r["ints"].as_array().unwrap().iter().enumerate() {
            assert_eq!(
                maze::maze_rng_int(&mut m, -1.0, 3.0),
                f(want),
                "maze_rng_int key {} #{k}",
                r["key"]
            );
        }
        assert_eq!(m.n, f(&r["n"]));
    }

    // obstacle bands
    let w = &mut sim.world;
    let right: Vec<Value> = obstacles::world_obstacles_reach(w, 1.0, 60000.0)
        .iter()
        .map(part_json)
        .collect();
    json_eq("right", &Value::Array(right), &g["right"]);
    let left: Vec<Value> = obstacles::world_obstacles_reach(w, -1.0, -60000.0)
        .iter()
        .map(part_json)
        .collect();
    json_eq("left", &Value::Array(left), &g["left"]);
    let o = w.params.obstacles.as_ref().unwrap();
    assert_eq!(
        [
            o.right.cursor,
            o.right.count as f64,
            o.left.cursor,
            o.left.count as f64
        ]
        .to_vec(),
        g["cursors"]
            .as_array()
            .unwrap()
            .iter()
            .map(f)
            .collect::<Vec<_>>()
    );
    for r in g["at_chunk"].as_array().unwrap() {
        let ch = WorldChunkSave {
            cx: i(&r[0]),
            ..Default::default()
        };
        let keys: Vec<i64> = obstacles::world_obstacles_at_chunk(w, &ch)
            .iter()
            .map(|o| o.key)
            .collect();
        json_eq(&format!("at_chunk {}", r[0]), &to_json(&keys), &r[1]);
    }

    // rock clusters
    let got: Vec<Value> = maze::world_rock_clusters(w)
        .iter()
        .map(|c| c.to_json())
        .collect();
    json_eq("clusters", &Value::Array(got), &g["clusters"]);

    // built mazes and clusters
    for m in g["mazes"].as_array().unwrap() {
        let d = maze::world_maze_build(w, i(&m["key"]));
        json_eq(&format!("maze {}", m["key"]), &to_json(&d), &m["data"]);
    }
    for c in g["cluster_data"].as_array().unwrap() {
        let idx = maze::world_rock_clusters(w)
            .iter()
            .position(|cl| cl.id == c["id"].as_str().unwrap())
            .unwrap();
        let d = maze::world_cluster_build(w, idx);
        json_eq(&format!("cluster {}", c["id"]), &to_json(&d), &c["data"]);
    }

    // blob primitives
    for b in g["blobs"].as_array().unwrap() {
        let a: Vec<i64> = b[0].as_array().unwrap().iter().map(i).collect();
        let blob = stones::stone_blob_cells(w, a[0], a[1], a[2], a[3], a[4]);
        json_eq(&format!("blob {}", b[0]), &cells_json(&blob), &b[1]);
    }
    let mut grid = HashSet::new();
    for y in 0..=9 {
        for x in 0..=9 {
            if (!(4..=6).contains(&x)) && !(y == 5 && x < 2) {
                grid.insert((x, y));
            }
        }
    }
    grid.insert((20, 20));
    let blob = stones::stone_blob_finish(&mut grid, -1, 21, -1, 21, 6);
    json_eq("blob_protect", &cells_json(&blob), &g["blob_protect"]);

    // cuts
    let full = || {
        let mut k = HashSet::new();
        for y in 0..30 {
            for x in 0..30 {
                k.insert((x, y));
            }
        }
        k
    };
    let left_of = |k: &HashSet<(i64, i64)>| {
        let mut t: Vec<String> = k
            .iter()
            .map(|&(c, r)| stones::stone_cell_key(c, r))
            .collect();
        t.sort();
        to_json(&t)
    };
    let mut k1 = full();
    stones::stone_cut_round(w, &mut k1, 12.3, 14.8, 5.4, 3.9, 17.0);
    json_eq("cut_round", &left_of(&k1), &g["cuts"][0]);
    let mut k2 = full();
    stones::stone_cut_tunnel(w, &mut k2, 3.5, 4.2, 25.5, 21.7, 1.6, 23.0);
    json_eq("cut_tunnel", &left_of(&k2), &g["cuts"][1]);

    // maze primitives on a synthetic maze
    let mut m = maze::Maze {
        seed,
        key: 12345.0,
        n: 0.0,
        c1: 0.0,
        c2: 39.0,
        top: 10.0,
        w: 40,
        h: 30,
        keep_top: 3,
        xmin: 2.0,
        xmax: 37.0,
        ymin: 14.0,
        ymax: 33.0,
        open: vec![false; 1200],
        inside: None,
    };
    maze::maze_disc(&mut m, 5.5, 12.25, 2.6);
    let room = maze::maze_room(&mut m, 20.0, 20.0, 3.2, 2.1);
    json_eq("room", &rooms_json(&[room], false)[0], &g["room"]);
    let pts = maze::maze_tunnel(
        &mut m,
        0.0,
        20.0,
        10.0,
        80.0,
        1.0,
        2.2,
        Some((39.0, 25.0)),
        true,
    );
    json_eq("tunnel", &pts_json(&pts), &g["tunnel"]);
    let pts2 = maze::maze_tunnel(&mut m, 30.0, 30.0, 200.0, 30.0, 0.8, 1.6, None, false);
    json_eq("tunnel2", &pts_json(&pts2), &g["tunnel2"]);
    json_eq("open1", &open_json(&m), &g["open1"]);
    assert_eq!(m.n, f(&g["n1"]));
    let dist = maze::maze_distance(&m, None);
    let dv: Vec<f64> = dist.iter().map(|&d| d as f64).collect();
    json_eq("dist", &rle(&dv), &g["dist"]);
    maze::maze_dig_out(&mut m, &dist, 20, 25);
    let mut rooms = Vec::new();
    maze::maze_hollows(&mut m, &mut rooms);
    json_eq("hollows", &rooms_json(&rooms, true), &g["hollows"]);
    let mut rooms2 = Vec::new();
    maze::maze_branches(&mut m, &pts, &mut rooms2);
    json_eq("branches", &rooms_json(&rooms2, true), &g["branches"]);
    json_eq("open2", &open_json(&m), &g["open2"]);
    assert_eq!(m.n, f(&g["n2"]));
    let mut cut = HashSet::new();
    for c in 0..=39 {
        for r in 0..=25 {
            if !maze::maze_is_open(&m, c as f64, (10 + r) as f64) {
                cut.insert((c, 10 + r));
            }
        }
    }
    let fin = maze::maze_finish(w, &mut m, &mut cut, &rooms, -1, 40, 9, 41, true);
    json_eq("finish", &to_json(&fin), &g["finish"]);
    assert_eq!(m.n, f(&g["n3"]));
}

// ---------------------------------------------------------------------------------------------
// worlds with stones

fn chunk_eq(ch: &WorldChunkSave, want: &Value, what: &str) {
    for (key, got) in [
        ("foods", to_json(&ch.foods)),
        ("lakes", to_json(&ch.lakes)),
        ("stones", to_json(&ch.stones)),
        ("stone_cells", to_json(&ch.stone_cells)),
    ] {
        json_eq(&format!("{what}.{key}"), &got, &want[key]);
    }
    assert_eq!(ch.rows as f64, f(&want["rows"]), "{what}.rows");
    assert_eq!(
        ch.stone_map.len() as f64,
        f(&want["stone_map_n"]),
        "{what}.stone_map"
    );
    for c in &ch.stone_cells {
        let m = &ch.stone_map[&stones::stone_cell_key(c.col, c.row)];
        assert_eq!(m, c, "{what}: stone_map entry of {},{}", c.col, c.row);
    }
    match (&ch.clusters_done, want.get("clusters_done")) {
        (None, None) => {}
        (Some(m), Some(w)) => {
            let keys: Vec<&String> = m.keys().collect();
            json_eq(&format!("{what}.clusters_done"), &to_json(&keys), w);
        }
        (g, w) => panic!("{what}.clusters_done: {g:?} vs {w:?}"),
    }
    let tiles: Vec<f64> = ch.tiles.iter().map(|&t| t as f64).collect();
    assert_eq!(tiles, unrle(&want["tiles"]), "{what}.tiles");
    assert_eq!(ch.amount, unrle(&want["amount"]), "{what}.amount");
}

fn chunks_eq(w: &WorldRt, want: &Value, what: &str) {
    let want = want.as_object().unwrap();
    assert_eq!(w.chunks.len(), want.len(), "{what}: chunk count");
    for (key, wc) in want {
        let cx: i64 = key[1..].parse().unwrap();
        let ch = w
            .chunk(cx)
            .unwrap_or_else(|| panic!("{what}: missing {key}"));
        chunk_eq(ch, wc, &format!("{what}.{key}"));
    }
}

fn snapshot(w: &mut WorldRt) -> Value {
    let mut s = Vec::new();
    for cx in -2..=2 {
        for (k, t) in w.get_chunk(cx).stones.iter().enumerate() {
            if t.moving || t.vx != 0.0 || t.angle != 0.0 {
                s.push(json!([
                    cx,
                    k,
                    t.x,
                    t.y,
                    t.angle,
                    t.vx,
                    t.vy,
                    t.spin,
                    t.moving as i64
                ]));
            }
        }
    }
    Value::Array(s)
}

fn stone_world_matches(topic: &str, seed: i64) {
    let g = golden(topic);
    let mut sim = new_world(seed);
    chunks_eq(&sim.world, &g["init_chunks"], "init");

    let lr = sim.world.params.layer_rows;
    for cx in -3..=3 {
        sim.world.chunk_reach(cx, 41 * lr - 1);
    }
    for dir in [1.0, -1.0] {
        let sx = sim.world.params.start_x;
        obstacles::world_obstacles_reach(&mut sim.world, dir, sx + dir * 30000.0);
    }
    for cx in g["far"].as_array().unwrap().iter().map(i) {
        sim.world.chunk_reach(cx, 3 * lr - 1);
    }
    let late: Vec<i64> = g["late"].as_array().unwrap().iter().map(i).collect();
    for &cx in &late {
        sim.world.get_chunk(cx);
        maze::world_paint_clusters_late(&mut sim.world, cx);
    }
    for &cx in &late {
        sim.world.chunk_reach(cx, 20 * lr - 1);
        maze::world_paint_clusters_late(&mut sim.world, cx);
    }
    chunks_eq(&sim.world, &g["chunks"], "reached");
    json_eq(
        "chunk_index",
        &to_json(&sim.world.params.chunk_index),
        &g["chunk_index"],
    );

    // tile grid
    let p = sim.world.params.clone();
    for gr in g["tile_grid"].as_array().unwrap() {
        let cx = i(&gr["cx"]);
        let mut got = Vec::new();
        for row in 0..i(&gr["rows"]) {
            let y = p.surface_y + (row * p.tile) as f64 + p.tile as f64 * 0.5;
            for col in 0..p.cols {
                let x = ((cx * p.cols + col) * p.tile) as f64 + p.tile as f64 * 0.5;
                got.push(sim.world.tile_at(x, y).code() as f64);
            }
        }
        assert_eq!(got, unrle(&gr["codes"]), "{topic}: tile grid of chunk {cx}");
    }

    // stone cell queries
    for cq in g["cell_queries"].as_array().unwrap() {
        let cx = i(&cq["cx"]);
        let n = i(&cq["n"]) as usize;
        let w = &sim.world;
        let ch = w.chunk(cx).unwrap();
        let mut q = Vec::new();
        for k in 0..n {
            let c = &ch.stone_cells[k * (ch.stone_cells.len() / n)];
            let mut y = (c.row * 48 - 3) as f64;
            while y <= (c.row * 48 + 50) as f64 {
                let mut x = (c.col * 48 - 2) as f64;
                while x <= (c.col * 48 + 50) as f64 {
                    let (px, py) = (x + 0.5, y + 0.25);
                    q.push(
                        stones::stone_solid_at(w, ch, px, py) as i64 as f64
                            + 2.0 * stones::world_stone_covers(w, ch, px, py) as i64 as f64,
                    );
                    x += 6.0;
                }
                y += 5.0;
            }
        }
        assert_eq!(q, unrle(&cq["q"]), "{topic}: cell queries of chunk {cx}");
    }

    // shove and roll
    sim.rng = Rng::new(4242);
    for a in g["shoves"].as_array().unwrap() {
        stones::stones_shove(&mut sim, f(&a[0]), f(&a[1]), f(&a[2]), f(&a[3]));
    }
    let frames = g["frames"].as_array().unwrap();
    let mut fi = 0;
    for fr in 1..=150 {
        stones::stones_step(&mut sim);
        if fr % 10 == 0 || fr <= 3 {
            let want = &frames[fi];
            fi += 1;
            assert_eq!(i(&want[0]), fr);
            assert_eq!(
                sim.world.stones_moving.len() as i64,
                i(&want[1]),
                "{topic}: moving after frame {fr}"
            );
            json_eq(
                &format!("{topic}: stones after frame {fr}"),
                &snapshot(&mut sim.world),
                &want[2],
            );
        }
        if fr == 40 {
            stones::stones_shove(&mut sim, 300.0, 700.0, 135.0, 9.0);
        }
    }
    assert_eq!(sim.rng.random(1.0), f(&g["rng_after"]), "RNG draws");

    // cells cut next to a border of the first right maze, then the redress (as caves do)
    let w = &mut sim.world;
    let cw = w.params.chunk_w;
    let x1 = w
        .params
        .obstacles
        .as_ref()
        .unwrap()
        .right
        .list
        .iter()
        .find(|o| o.kind == "maze")
        .unwrap()
        .x1;
    let mut rc = (x1 / cw as f64).floor() as i64 + 1;
    while (rc * cw) % 48 == 0 {
        rc += 1;
    }
    let r = &g["redress"];
    assert_eq!(rc, i(&r["cx"]));
    let bcol = (rc * cw).div_euclid(48);
    let top = (w.params.surface_y / 48.0).floor() as i64;
    let mut ch = w.chunks.remove(&rc).unwrap();
    let mut cut = HashSet::new();
    for row in top + 3..=top + 25 {
        for col in bcol + 1..=bcol + 2 {
            if cut.len() < 6
                && ch
                    .stone_map
                    .remove(&stones::stone_cell_key(col, row))
                    .is_some()
            {
                cut.insert((col, row));
            }
        }
    }
    ch.stone_cells.retain(|c| !cut.contains(&(c.col, c.row)));
    stones::stone_redress_after_cut(w, &mut ch, &mut cut);
    let mut keys: Vec<String> = cut
        .iter()
        .map(|&(c, r)| stones::stone_cell_key(c, r))
        .collect();
    keys.sort();
    json_eq("redress.cut", &to_json(&keys), &r["cut"]);
    json_eq("redress.cells", &to_json(&ch.stone_cells), &r["cells"]);
    assert_eq!(ch.stone_map.len() as f64, f(&r["map_n"]));
    w.chunks.insert(rc, ch);
    json_eq(
        "redress.left_cells",
        &to_json(&w.chunk(rc - 1).unwrap().stone_cells),
        &r["left_cells"],
    );

    // obstacles as save_game writes them (painted mazes carry their data)
    json_eq(
        "obstacles",
        &to_json(&sim.world.params.obstacles),
        &g["obstacles"],
    );
}

#[test]
fn stone_layout_matches_love_seed_31607() {
    stone_world_matches("stones_s31607", 31607);
}

#[test]
fn stone_layout_matches_love_seed_7() {
    stone_world_matches("stones_s7", 7);
}

// ---------------------------------------------------------------------------------------------
// Rust-only behaviour

#[test]
fn obstacles_built_lazily_from_start_x_when_absent_in_a_save() {
    let sim = new_world(31607);
    let (mut st, _) = sim.to_save();
    assert!(
        st.world.obstacles.is_some(),
        "generating the start chunks built it"
    );
    st.world.obstacles = None;
    let text = serde_json::to_string(&st).unwrap();
    assert!(!text.contains("\"obstacles\""), "absent key stays absent");
    st.world.start_x = 500.0;
    let mut sim = Sim::from_save(st).unwrap();
    obstacles::world_obstacles_reach(&mut sim.world, 1.0, -1e9);
    let o = sim.world.params.obstacles.as_ref().unwrap();
    let min = myco_core::balance::wld("surface_wall_min_start_px");
    assert_eq!(o.right.cursor, 500.0 + min);
    assert_eq!(o.left.cursor, 500.0 - min);
    assert!(o.right.list.is_empty() && o.left.list.is_empty());
}

#[test]
fn maze_data_survives_a_save_round_trip() {
    let mut sim = new_world(7);
    let key = obstacles::world_obstacles_reach(&mut sim.world, 1.0, 30000.0)
        .iter()
        .find(|o| o.kind == "maze")
        .unwrap()
        .key;
    let built = maze::world_maze_build(&mut sim.world, key);
    // through the save file encoder (save_N.json with its trailing NUL) and back
    let dir = std::env::temp_dir().join(format!("myco_t7_maze_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (st, meta) = sim.to_save();
    myco_core::save::write_slot(&dir, 0, &st, &meta).unwrap();
    let path = myco_core::save::save_path(&dir, 0);
    let text1 = std::fs::read(&path).unwrap();
    let back = myco_core::save::read_slot(&dir, 0).unwrap();
    let mut sim2 = Sim::from_save(back.clone()).unwrap();
    assert_eq!(maze::world_maze_build(&mut sim2.world, key), built);
    // the world section (built maze data included) re-encodes byte for byte
    let enc = |w: &myco_core::state::WorldSave| myco_core::save::to_love_json(w).unwrap();
    assert_eq!(enc(&back.world), enc(&st.world));
    assert!(String::from_utf8_lossy(&text1).contains("\"entries\""));
    std::fs::remove_dir_all(&dir).unwrap();
    // an unbuilt maze is saved as null and a wall has no data key
    let o = sim2.world.params.obstacles.as_ref().unwrap();
    let v = to_json(o);
    let list = v["right"]["list"].as_array().unwrap();
    assert!(list
        .iter()
        .any(|o| o["kind"] == "wall" && o.get("data").is_none()));
    assert!(list
        .iter()
        .any(|o| o["kind"] == "maze" && o["data"].is_null()));
}

#[test]
fn stone_cell_band_is_optional() {
    // an old cell without "band" stays without it, and the redress hashes one
    let c: myco_core::state::StoneCellSave =
        serde_json::from_str(r#"{"col":3,"row":20,"tc":3,"tr":3,"deep":false}"#).unwrap();
    assert_eq!(c.band, None);
    assert!(!serde_json::to_string(&c).unwrap().contains("band"));
    let c: myco_core::state::StoneCellSave =
        serde_json::from_str(r#"{"band":1,"col":3,"row":20,"tc":3,"tr":3,"deep":false}"#).unwrap();
    assert_eq!(c.band, Some(1));
    assert!(serde_json::to_string(&c).unwrap().contains("\"band\":1"));
}

#[test]
fn stone_cell_deep_is_optional() {
    // an old cell without "deep" stays without it (the renderer falls back to the cell's row);
    // a present value round-trips byte for byte
    let old = r#"{"col":3,"row":20,"tc":3,"tr":3}"#;
    let c: myco_core::state::StoneCellSave = serde_json::from_str(old).unwrap();
    assert_eq!(c.deep, None);
    assert_eq!(serde_json::to_string(&c).unwrap(), old);
    for text in [
        r#"{"band":1,"col":3,"deep":false,"row":20,"tc":3,"tr":3}"#,
        r#"{"band":0,"col":3,"deep":true,"row":20,"tc":3,"tr":3}"#,
    ] {
        let c: myco_core::state::StoneCellSave = serde_json::from_str(text).unwrap();
        assert!(c.deep.is_some());
        assert_eq!(serde_json::to_string(&c).unwrap(), text);
    }
}

#[test]
fn loading_stops_moving_stones() {
    let mut sim = new_world(31607);
    let s = sim.world.get_chunk(0).stones[0].clone();
    stones::stones_shove(&mut sim, s.x, s.y, 0.0, 8.0);
    assert!(!sim.world.stones_moving.is_empty());
    let (st, _) = sim.to_save();
    let sim2 = Sim::from_save(st).unwrap();
    assert!(sim2.world.stones_moving.is_empty());
    for ch in sim2.world.chunks.values() {
        for t in &ch.stones {
            assert!(!t.moving && t.vx == 0.0 && t.vy == 0.0 && t.spin == 0.0);
            assert_ne!(t.uid, 0, "loaded stones get runtime ids");
        }
    }
}

#[test]
fn point_distance_squares_through_pow_like_luajit() {
    // two stones touching at frame 15 of stones_s31607: macOS pow(dy, 2) is one ulp above
    // dy * dy, and LuaJIT's `^` (gm.point_distance) uses pow
    let dy = 8400.647101910525 - 8381.575123296032;
    assert_eq!(myco_core::gml::power(dy, 2.0), 363.7403682716693);
    assert_ne!(dy * dy, 363.7403682716693);
    let d = myco_core::gml::point_distance(0.0, 0.0, 4.192331650958589, dy);
    assert_eq!(
        d,
        (4.192331650958589f64.powf(std::hint::black_box(2.0)) + 363.7403682716693).sqrt()
    );
}
