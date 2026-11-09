//! Bones (scr_world): small animal skeletons painted into chunk layers (`chunk.bones`, saved)
//! and the two big skeletons deep below the start (the T-rex and the UFO), which double as
//! relics (`relics.rs`) and as cave clear zones.

// `!(a >= b)` and friends are kept as the original writes them (same result for NaN).
#![allow(clippy::neg_cmp_op_on_partial_ord)]

use super::caves::{self, cave_range, Zone};
use super::relics::RelicKind;
use super::{gen, lakes, stones, WorldRt};
use crate::balance::{vis, wld, wld_arr};
use crate::gml;
use crate::state::{BoneSave, WorldChunkSave};

/// A small skeleton kind (`___struct___307..312__bone_kinds`).
#[derive(Clone, Debug, PartialEq)]
pub struct BoneKind {
    pub id: &'static str,
    pub sprite: &'static str,
    pub weight: f64,
    /// depth (screens below the surface) where its weight starts to fade
    pub near: f64,
    /// depth where the weight reaches `weight * deep_share`
    pub far: f64,
    pub deep_share: f64,
}

impl BoneKind {
    /// `[sprite_get_width, sprite_get_height]` of its sprite.
    pub fn size(&self) -> [f64; 2] {
        caves::bone_sprite_size(self.id)
    }
}

// port: bone_kinds
/// `bone_kinds()`: squirrel, hedgehog, crow, pig, wolf, elk.
pub fn bone_kinds() -> &'static [BoneKind; 6] {
    const fn k(
        id: &'static str,
        sprite: &'static str,
        weight: f64,
        near: f64,
        far: f64,
        deep_share: f64,
    ) -> BoneKind {
        BoneKind {
            id,
            sprite,
            weight,
            near,
            far,
            deep_share,
        }
    }
    static KINDS: [BoneKind; 6] = [
        k("squirrel", "spr_skeleton_squirrel", 30.0, 0.8, 2.4, 0.0),
        k("hedgehog", "spr_skeleton_hedgehog", 30.0, 0.8, 2.4, 0.0),
        k("crow", "spr_skeleton_crow", 24.0, 0.6, 2.0, 0.0),
        k("pig", "spr_skeleton_pig", 10.0, 1.5, 3.5, 0.2),
        k("wolf", "spr_skeleton_wolf", 8.0, 1.5, 3.5, 0.2),
        k("elk", "spr_skeleton_elk", 5.0, 1.5, 3.5, 0.2),
    ];
    &KINDS
}

// port: bone_kind_weight
/// `bone_kind_weight(k, depth)`: full weight above `near`, fading to `deep_share` at `far`.
pub fn bone_kind_weight(k: &BoneKind, depth: f64) -> f64 {
    let t = gml::clamp((depth - k.near) / 0.01f64.max(k.far - k.near), 0.0, 1.0);
    k.weight * gml::lerp(1.0, k.deep_share, t)
}

/// Result of [`bone_kind_pick`] (`___struct___313..315`): a `bone_kinds()` index (-1: none)
/// and the total weight at this depth relative to the base weights.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BonePick {
    pub kind: i64,
    pub share: f64,
}

// port: bone_kind_pick
/// `bone_kind_pick(r, depth)`: the kind a roll `r` in [0, 1) picks by depth-weighted weights.
pub fn bone_kind_pick(r: f64, depth: f64) -> BonePick {
    let kinds = bone_kinds();
    let mut total = 0.0;
    let mut base = 0.0;
    for k in kinds {
        total += bone_kind_weight(k, depth);
        base += k.weight;
    }
    if !(total > 0.0) {
        return BonePick {
            kind: -1,
            share: 0.0,
        };
    }
    let mut roll = r * total;
    for (i, k) in kinds.iter().enumerate() {
        roll -= bone_kind_weight(k, depth);
        if roll < 0.0 {
            return BonePick {
                kind: i as i64,
                share: total / base,
            };
        }
    }
    BonePick {
        kind: kinds.len() as i64 - 1,
        share: total / base,
    }
}

// port: world_paint_bones
/// `world_paint_bones(chunk, layer)`: up to `bones_tries_per_layer` skeletons in the layer, on
/// soil (not lakes, stones or rock), clear of caves, of each other, and of the start screen.
pub fn world_paint_bones(w: &mut WorldRt, ch: &mut WorldChunkSave, layer: i64) {
    if caves::neutral() {
        return;
    }
    let key = gen::layer_key(ch.cx, layer);
    let p = &w.params;
    let (tile, sy, chunk_w, start_x) = (p.tile as f64, p.surface_y, p.chunk_w as f64, p.start_x);
    let x0 = ch.cx as f64 * chunk_w;
    let top = w.layer_top(layer) + tile;
    let bot = w
        .layer_bottom(layer)
        .min(w.ocean_y() - wld("ocean_clear_margin_px"));
    let kinds = bone_kinds();
    let sw = vis("screen_width_px");
    let sh = vis("screen_height_px");
    let mut t = 0.0;
    while t < wld("bones_tries_per_layer") {
        let y = gml::lerp(top, bot, w.hash01(key, t, 404.0));
        let pick = bone_kind_pick(w.hash01(key, t, 402.0), (y - sy) / sh);
        if pick.kind >= 0 && !(w.hash01(key, t, 401.0) >= wld("bones_chance") * pick.share) {
            let k = pick.kind as usize;
            let [bw, bh] = kinds[k].size();
            if bot - top >= bh + 16.0 {
                let bx = (x0 + w.hash01(key, t, 403.0) * 1f64.max(chunk_w - bw)).floor();
                let by = gml::clamp(y - bh * 0.5, top, bot - bh).floor();
                let skip = (bx + bw * 0.5 - start_x).abs() < sw * 0.5 && by < sy + sh;
                if !skip && !caves::world_cave_hits(w, bx, by, bx + bw, by + bh, None) {
                    let mut ok = true;
                    let mut i = 0.0;
                    while i < 5.0 && ok {
                        let fx = if i == 4.0 { 0.5 } else { i % 2.0 };
                        let px = bx + fx * bw;
                        let fy = if i == 4.0 { 0.5 } else { (i / 2.0f64).trunc() };
                        let py = by + fy * bh;
                        ok = !lakes::world_lake_covers(ch, px, py)
                            && !stones::world_stone_covers(w, ch, px, py)
                            && w.chunk_get(
                                ch,
                                (px / tile).floor() as i64,
                                ((py - sy) / tile).floor() as i64,
                            ) != 2;
                        i += 1.0;
                    }
                    for b in &ch.bones {
                        if !ok {
                            break;
                        }
                        ok = bx + bw + 32.0 <= b.x
                            || bx >= b.x + b.w + 32.0
                            || by + bh + 32.0 <= b.y
                            || by >= b.y + b.h + 32.0;
                    }
                    if ok {
                        let flip = if w.hash01(key, t, 405.0) < 0.5 {
                            -1.0
                        } else {
                            1.0
                        };
                        ch.bones.push(BoneSave {
                            kind: k as i64,
                            x: bx,
                            y: by,
                            w: bw,
                            h: bh,
                            flip,
                            extra: Default::default(),
                        });
                    }
                }
            }
        }
        t += 1.0;
    }
}

/// A big skeleton (`___struct___319__world_big_bones`): the T-rex or the UFO.
#[derive(Clone, Debug, PartialEq)]
pub struct BigBone {
    pub kind: RelicKind,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl BigBone {
    pub fn zone(&self) -> Zone {
        Zone {
            id: self.kind.id().to_string(),
            x1: self.x1,
            y1: self.y1,
            x2: self.x2,
            y2: self.y2,
        }
    }
}

// port: world_big_bones
/// `world_big_bones()`: the T-rex and the UFO on opposite sides, `bones_big_x` screens out and
/// `bones_big_depth` screens down (inside the deep soil), stepped outward off the caves. Cached
/// per start_x/seed from the first call (later caves do not move them).
pub fn world_big_bones(w: &mut WorldRt) -> &[BigBone] {
    let key = (w.params.start_x, w.params.seed);
    if w.caves.big_bones_key == Some(key) {
        return &w.caves.big_bones;
    }
    w.caves.big_bones_key = Some(key);
    w.caves.big_bones.clear();
    w.caves.bones_gen += 1;
    let sw = vis("screen_width_px");
    let sh = vis("screen_height_px");
    let ylo = w.deep_y() + wld("caves_margin_px");
    let yhi = w.ocean_y() - wld("caves_margin_px");
    let rects: Vec<(bool, [f64; 4])> = w
        .cave_rects()
        .iter()
        .map(|r| (r.id == "yellow", [r.x1, r.y1, r.x2, r.y2]))
        .collect();
    let dir = if w.hash01(6.0, 1.0, 406.0) < 0.5 {
        -1.0
    } else {
        1.0
    };
    let defs = [(RelicKind::TRex, dir), (RelicKind::Ufo, -dir)];
    let pad = wld("caves_clearance_px") * 2.0;
    for (i, (kind, side)) in defs.into_iter().enumerate() {
        let i = i as f64;
        let [bw, bh] = caves::cave_mask(kind.id()).unwrap().sprite;
        let cx = w.params.start_x
            + side * sw * cave_range(wld_arr("bones_big_x"), w.hash01(7.0 + i, 2.0, 406.0));
        let cy = w.params.surface_y
            + sh * cave_range(wld_arr("bones_big_depth"), w.hash01(7.0 + i, 3.0, 406.0));
        let by = gml::clamp(cy - bh * 0.5, ylo, ylo.max(yhi - bh)).floor();
        let mut bx = (cx - bw * 0.5).floor();
        for _ in 0..12 {
            let ok = rects.iter().all(|(yellow, r)| {
                *yellow
                    || bx + bw + pad <= r[0]
                    || bx >= r[2] + pad
                    || by + bh + pad <= r[1]
                    || by >= r[3] + pad
            });
            if ok {
                break;
            }
            bx += side * (bw + pad);
        }
        w.caves.big_bones.push(BigBone {
            kind,
            x1: bx,
            y1: by,
            x2: bx + bw,
            y2: by + bh,
        });
    }
    &w.caves.big_bones
}
