//! Lakes (scr_world): the three lake shapes, their placement per layer, coverage tests, water
//! level, and the sparkle ("flare") positions the renderer animates.

use super::art::{art, LakeKind};
use super::hash::world_hash01;
use super::WorldRt;
use crate::balance::{vis, wld};
use crate::gml;
use crate::state::{LakeSave, WorldChunkSave};

/// One entry of `lake_layout()`: where the lake body sits inside its sprite cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LakeLayout {
    pub left: f64,
    pub width: f64,
    pub body_x: f64,
    pub body_y: f64,
    pub body_w: f64,
    pub body_h: f64,
}

// port: lake_layout
pub fn lake_layout() -> &'static [LakeLayout; 3] {
    const L: [LakeLayout; 3] = [
        LakeLayout {
            left: 0.0,
            width: 351.0,
            body_x: 42.0,
            body_y: 27.0,
            body_w: 247.0,
            body_h: 143.0,
        },
        LakeLayout {
            left: 351.0,
            width: 274.0,
            body_x: 41.0,
            body_y: 55.0,
            body_w: 154.0,
            body_h: 85.0,
        },
        LakeLayout {
            left: 625.0,
            width: 176.0,
            body_x: 38.0,
            body_y: 65.0,
            body_w: 93.0,
            body_h: 51.0,
        },
    ];
    &L
}

/// `global.lake_kinds` (built by `lakes_init` from the water sprite, Task 13's; the data is
/// embedded, see `art.rs`).
pub fn lake_kinds() -> &'static [LakeKind] {
    &art().lake_kinds
}

/// The lake kind for a lake's `kind` field.
pub fn lake_kind(kind: i64) -> &'static LakeKind {
    &lake_kinds()[kind.clamp(0, 2) as usize]
}

// port: lakes_ready
/// The masks are always available in the port (the original builds them on the first step).
pub fn lakes_ready() -> bool {
    true
}

// port: lake_solid_at
pub fn lake_solid_at(kind: &LakeKind, x: f64, y: f64) -> bool {
    if y < 0.0 || y >= kind.height {
        return false;
    }
    let Some(spans) = kind.spans.get(y.trunc() as usize) else {
        return false;
    };
    spans
        .as_chunks::<2>()
        .0
        .iter()
        .any(|s| x >= s[0] && x <= s[1])
}

// port: lake_pick_kind
/// 0 big, 1 medium, 2 small; bigger lakes get likelier with depth `t`.
pub fn lake_pick_kind(t: f64, r: f64) -> i64 {
    let big = gml::lerp(
        wld("water_lake_big_chance_top"),
        wld("water_lake_big_chance_bottom"),
        t,
    );
    let small = gml::lerp(
        wld("water_lake_small_chance_top"),
        wld("water_lake_small_chance_bottom"),
        t,
    );
    if r < big {
        return 0;
    }
    if r < big + small {
        return 2;
    }
    1
}

// port: lake_amount
pub fn lake_amount(kind: i64) -> f64 {
    match kind {
        0 => wld("water_lake_big_amount"),
        1 => wld("water_lake_medium_amount"),
        _ => wld("water_lake_small_amount"),
    }
}

// port: lake_flare_count
pub fn lake_flare_count(kind: i64) -> f64 {
    match kind {
        0 => vis("water_flares_big"),
        1 => vis("water_flares_medium"),
        _ => vis("water_flares_small"),
    }
}

/// A new lake struct (`___struct___343/349`): not connected, no flares yet.
pub(crate) fn new_lake(
    cx: i64,
    seed: i64,
    kind: i64,
    x: f64,
    y: f64,
    body: [f64; 4],
    amount: f64,
) -> LakeSave {
    LakeSave {
        amount,
        body_h: body[3],
        body_w: body[2],
        body_x: body[0],
        body_y: body[1],
        connected: false,
        cx,
        flare_seq: 0,
        flares: None,
        full: amount,
        kind,
        seed,
        sv: None,
        x,
        y,
        extra: Default::default(),
    }
}

// port: world_lake_covers
/// Inside the body rectangle of one of the chunk's lakes.
pub fn world_lake_covers(chunk: &WorldChunkSave, x: f64, y: f64) -> bool {
    chunk.lakes.iter().any(|l| {
        x >= l.body_x && x < l.body_x + l.body_w && y >= l.body_y && y < l.body_y + l.body_h
    })
}

// port: lake_flare_life_ms
pub fn lake_flare_life_ms() -> f64 {
    ((vis("water_flare_fade_in_seconds").max(0.0)
        + vis("water_flare_hold_seconds").max(0.0)
        + vis("water_flare_fade_out_seconds").max(0.0))
        * 1000.0)
        .max(1.0)
}

// port: lake_flare_alpha
pub fn lake_flare_alpha(age: f64) -> f64 {
    let fin = vis("water_flare_fade_in_seconds").max(0.0) * 1000.0;
    let hold = vis("water_flare_hold_seconds").max(0.0) * 1000.0;
    let fout = vis("water_flare_fade_out_seconds").max(0.0) * 1000.0;
    if age < 0.0 {
        return 0.0;
    }
    if age < fin {
        return age / fin;
    }
    if age < fin + hold {
        return 1.0;
    }
    if age < fin + hold + fout {
        return 1.0 - (age - fin - hold) / fout;
    }
    0.0
}

/// One sparkle of a lake (`___struct___351`); runtime only, the renderer keeps the list.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LakeFlare {
    pub x: f64,
    pub y: f64,
    pub born: f64,
    pub placed: bool,
}

// port: lake_flare_spot
/// A free spot for flare `skip` inside the lake mask below `min_y` (12 tries), advancing the
/// lake's saved `flare_seq`.
pub fn lake_flare_spot(
    world_seed: i64,
    lake: &mut LakeSave,
    flares: &[LakeFlare],
    kind: &LakeKind,
    skip: usize,
    min_y: f64,
) -> Option<(f64, f64)> {
    let [sw, fh] = art().sprites.water_flare;
    let fw = (sw / 3.0).trunc();
    let ox0 = (fw * 0.1).floor();
    let ox1 = (fw * 0.9).ceil() - 1.0;
    let oy0 = (fh * 0.2).floor();
    let oy1 = (fh * 0.8).ceil() - 1.0;
    let xmin = -ox0;
    let xmax = kind.width - 1.0 - ox1;
    let ymin = min_y.max(kind.top) - oy0;
    let ymax = kind.bottom - oy1;
    if xmax < xmin || ymax < ymin {
        return None;
    }
    let seed = (lake.cx * 64 + lake.seed) as f64;
    for _ in 0..12 {
        lake.flare_seq += 1;
        let q = lake.flare_seq as f64;
        let x = xmin + (world_hash01(world_seed, seed, q, 91.0) * (xmax - xmin + 1.0)).floor();
        let y = ymin + (world_hash01(world_seed, seed, q, 92.0) * (ymax - ymin + 1.0)).floor();
        if lake_solid_at(kind, x + ox0, y + oy0)
            && lake_solid_at(kind, x + ox1, y + oy0)
            && lake_solid_at(kind, x + ox0, y + oy1)
            && lake_solid_at(kind, x + ox1, y + oy1)
        {
            let hit = flares.iter().enumerate().any(|(i, f)| {
                i != skip && f.placed && (f.x - x).abs() < fw * 0.75 && (f.y - y).abs() < fh * 0.5
            });
            if !hit {
                return Some((x, y));
            }
        }
    }
    None
}

// port: lake_init_flares
/// Fresh flares for a lake (`now_ms` = `current_time`): count by size, placed where they fit,
/// births staggered over half a life. Resets the lake's `flare_seq`.
pub fn lake_init_flares(
    world_seed: i64,
    lake: &mut LakeSave,
    kind: &LakeKind,
    min_y: f64,
    now_ms: f64,
) -> Vec<LakeFlare> {
    let count = lake_flare_count(lake.kind);
    let life = lake_flare_life_ms();
    lake.flare_seq = 0;
    let n = count.max(0.0).ceil() as usize;
    let mut flares = vec![
        LakeFlare {
            x: 0.0,
            y: 0.0,
            born: now_ms,
            placed: false
        };
        n
    ];
    for i in 0..n {
        if let Some((x, y)) = lake_flare_spot(world_seed, lake, &flares, kind, i, min_y) {
            let f = &mut flares[i];
            f.x = x;
            f.y = y;
            f.placed = true;
            let seed = (lake.cx * 64 + lake.seed) as f64;
            f.born = now_ms + (world_hash01(world_seed, seed, i as f64, 93.0) * life * 0.5).floor();
        }
    }
    flares
}

// port: lake_cut_row
/// First mask row still holding water: the level sinks as the lake empties.
pub fn lake_cut_row(lake: &LakeSave, kind: &LakeKind) -> f64 {
    let t = 1.0 - gml::clamp(lake.amount / lake.full.max(1.0), 0.0, 1.0);
    gml::lerp(kind.top, kind.bottom + 1.0, t).floor()
}

impl WorldRt {
    // port: world_paint_lakes
    /// Try `water_lake_tries` lakes in this layer of the chunk.
    pub(crate) fn paint_lakes(&mut self, chunk: &mut WorldChunkSave, layer: i64) {
        let p = &self.params;
        let key = super::gen::layer_key(chunk.cx, layer);
        let lay = lake_layout();
        let x0 = (chunk.cx * p.chunk_w) as f64;
        let chunk_w = p.chunk_w as f64;
        let top = self.layer_top(layer);
        let bottom = self.layer_bottom(layer);
        let gap = wld("water_lake_gap");
        let clr = wld("stone_water_clearance_px");
        let tries = wld("water_lake_tries").max(0.0);
        let chance = wld("water_lake_chance");
        let mut t = 0.0;
        while t < tries {
            if self.hash01(key, t, 81.0) < chance {
                let depth = self.hash01(key, t, 82.0);
                let kind = lake_pick_kind(depth, self.hash01(key, t, 83.0));
                let l = lay[kind as usize];
                let bx = (x0
                    + gap
                    + self.hash01(key, t, 84.0) * (chunk_w - l.body_w - gap * 2.0).max(0.0))
                .floor();
                let by = gml::lerp(top, top.max(bottom - l.body_h), depth).floor();
                let mut ok = !chunk.lakes.iter().any(|o| {
                    bx < o.body_x + o.body_w + gap
                        && bx + l.body_w + gap > o.body_x
                        && by < o.body_y + o.body_h + gap
                        && by + l.body_h + gap > o.body_y
                });
                if ok
                    && super::caves::world_cave_hits(
                        self,
                        bx,
                        by,
                        bx + l.body_w,
                        by + l.body_h,
                        None,
                    )
                {
                    ok = false;
                }
                let mut yy = by - clr;
                while yy <= by + l.body_h + clr && ok {
                    let mut xx = bx - clr;
                    while xx <= bx + l.body_w + clr && ok {
                        ok = !super::stones::world_stone_covers(self, chunk, xx, yy);
                        xx += 16.0;
                    }
                    yy += 16.0;
                }
                if ok {
                    let amount = lake_amount(kind);
                    chunk.lakes.push(new_lake(
                        chunk.cx,
                        t as i64 + layer * 8,
                        kind,
                        bx - l.body_x,
                        by - l.body_y,
                        [bx, by, l.body_w, l.body_h],
                        amount,
                    ));
                }
            }
            t += 1.0;
        }
    }
}
