//! Relics (scr_world): the two big skeletons (`bones::world_big_bones`) can be touched by the
//! mycelium like caves; that marks them taken (`state.relic_taken`, saved, keyed "t_rex" /
//! "ufo") and starts their wrap (runtime `cave_flash` / `cave_wrap`, keyed "relic_<id>").
//! Relics grant no perk.

use super::caves::{self, CaveWrap};
use super::{bones, WorldRt};
use crate::balance::{gml_truthy, vis};
use crate::sim::Sim;
use crate::state::GameState;

/// The relic skeletons.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelicKind {
    TRex,
    Ufo,
}

impl RelicKind {
    pub const ALL: [RelicKind; 2] = [RelicKind::TRex, RelicKind::Ufo];

    /// "t_rex" / "ufo" (the big bone id, the `relic_taken` key and the mask id).
    pub fn id(self) -> &'static str {
        match self {
            RelicKind::TRex => "t_rex",
            RelicKind::Ufo => "ufo",
        }
    }

    pub fn sprite(self) -> &'static str {
        match self {
            RelicKind::TRex => "spr_skeleton_T_rex",
            RelicKind::Ufo => "spr_UFO",
        }
    }

    pub fn from_id(id: &str) -> Option<RelicKind> {
        RelicKind::ALL.into_iter().find(|k| k.id() == id)
    }
}

/// A relic definition (`___struct___321/322__relic_def`): the whole sprite, no stems.
#[derive(Clone, Debug, PartialEq)]
pub struct RelicDef {
    pub kind: RelicKind,
    pub sprite: &'static str,
    pub left: f64,
    /// sprite width
    pub w: f64,
}

// port: relic_def
/// `relic_def(id)`: the definition for "t_rex" or "ufo".
pub fn relic_def(id: &str) -> Option<RelicDef> {
    let kind = RelicKind::from_id(id)?;
    Some(RelicDef {
        kind,
        sprite: kind.sprite(),
        left: 0.0,
        w: caves::cave_mask(id)?.sprite[0],
    })
}

/// A touchable relic (`___struct___323__world_relics`): `id` is "relic_<relic>".
#[derive(Clone, Debug, PartialEq)]
pub struct Relic {
    pub id: String,
    pub relic: RelicKind,
    pub k: f64,
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

impl Relic {
    /// `cave_solid_at(self, x, y)` on the relic's mask.
    pub fn solid_at(&self, x: f64, y: f64) -> bool {
        caves::cave_solid_at(self.relic.id(), self.k, self.x1, self.y1, x, y)
    }
}

// port: world_relics
/// `world_relics()`: one relic per big bone (rebuilt when the big bones are).
pub fn world_relics(w: &mut WorldRt) -> &[Relic] {
    bones::world_big_bones(w);
    let gen = w.caves.bones_gen;
    if w.caves.relics_gen != Some(gen) {
        w.caves.relics_gen = Some(gen);
        w.caves.relics = w
            .caves
            .big_bones
            .iter()
            .filter_map(|b| {
                relic_def(b.kind.id()).map(|d| Relic {
                    id: format!("relic_{}", b.kind.id()),
                    relic: d.kind,
                    k: 1.0,
                    x1: b.x1,
                    y1: b.y1,
                    x2: b.x2,
                    y2: b.y2,
                })
            })
            .collect();
    }
    &w.caves.relics
}

// port: relic_taken
/// `relic_taken(r)`: the relic was touched (`state.relic_taken[$ r.relic]`).
pub fn relic_taken(state: &GameState, r: &Relic) -> bool {
    gml_truthy(state.state.relic_taken.get(r.relic.id()))
}

// port: relic_wrapped
/// `relic_wrapped(id)`: `relic_taken[$ id]` is set and the wrap under the same key (if any) has
/// run `cave_wrap_ms`. (The original mixes keys: `relic_taken` uses "t_rex", `cave_wrap`
/// "relic_t_rex"; kept as is.)
pub fn relic_wrapped(sim: &Sim, id: &str) -> bool {
    if gml_truthy(sim.state.state.relic_taken.get(id)) {
        // TODO(port) in the Lua: value from inlined FUN_1402f02e0, assumed global.cave_wrap[$ id]
        if let Some(wr) = sim.world.caves.wrap.get(id) {
            return sim.now_ms() - wr.born >= vis("cave_wrap_ms");
        }
        return true;
    }
    false
}

// port: relics_touch_at
/// `relics_touch_at(x, y)` (a growth tip reached x, y): mark every untouched relic whose
/// skeleton is within `cave_touch_px` taken, flash it and start its wrap. Returns the relics
/// taken.
pub fn relics_touch_at(sim: &mut Sim, x: f64, y: f64) -> Vec<RelicKind> {
    let rs = world_relics(&mut sim.world).to_vec();
    let pad = vis("cave_touch_px");
    let mut taken = Vec::new();
    for r in &rs {
        if x < r.x1 - pad
            || x > r.x2 + pad
            || y < r.y1 - pad
            || y > r.y2 + pad
            || relic_taken(&sim.state, r)
        {
            continue;
        }
        if caves::touch_hits(|px, py| r.solid_at(px, py), x, y, pad) {
            sim.state
                .state
                .relic_taken
                .insert(r.relic.id().to_string(), true.into());
            let now = sim.now_ms();
            let c = &mut sim.world.caves;
            c.flash.insert(r.id.clone(), now);
            c.wrap.insert(
                r.id.clone(),
                CaveWrap {
                    born: now,
                    from_x: (x - r.x1) / r.k,
                    from_y: (y - r.y1) / r.k,
                },
            );
            caves::water_touch_sound(sim, x, y);
            taken.push(r.relic);
        }
    }
    taken
}
