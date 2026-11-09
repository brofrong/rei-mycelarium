//! Water sources (scr_world): connecting lakes, water cells and the ocean to the mycelium,
//! taking water from them, and the shown-water counter. `sim.state.water` is the live
//! `global.water_sources` in its saved form (lake = chunk + index in its lakes, cell = chunk +
//! tile index); lakes are never removed from a chunk after generation, so the index is stable.

use super::lakes;
use crate::balance::vis;
use crate::progress::{self, tutorial};
use crate::sim::Sim;
use crate::state::WaterSave;

// port: world_connect_lake_at
/// Connect the first unconnected, non-empty lake whose mask is at (or within 4 px of) the
/// point.
pub fn connect_lake_at(sim: &mut Sim, x: f64, y: f64) -> bool {
    if !lakes::lakes_ready() {
        return false;
    }
    let cx = (x / sim.world.params.chunk_w as f64).floor() as i64;
    let chunk = sim.world.get_chunk(cx);
    let hit = chunk.lakes.iter().position(|l| {
        if l.connected || l.amount <= 0.0 {
            return false;
        }
        let kd = lakes::lake_kind(l.kind);
        let lx = (x - l.x).floor();
        let ly = (y - l.y).floor();
        if lx < -4.0 || lx > kd.width + 4.0 || ly < -4.0 || ly > kd.height + 4.0 {
            return false;
        }
        lakes::lake_solid_at(kd, lx, ly)
            || lakes::lake_solid_at(kd, lx - 4.0, ly)
            || lakes::lake_solid_at(kd, lx + 4.0, ly)
            || lakes::lake_solid_at(kd, lx, ly - 4.0)
            || lakes::lake_solid_at(kd, lx, ly + 4.0)
    });
    let Some(i) = hit else {
        return false;
    };
    let l = &mut chunk.lakes[i];
    l.connected = true;
    let (amount, lake_cx) = (l.amount, l.cx);
    // save_game finds the lake in world_get_chunk(lake.cx).lakes: its own chunk
    debug_assert_eq!(lake_cx, cx);
    sim.state.water.push(WaterSave::Lake {
        lake_cx,
        lake: i as i64,
        extra: Default::default(),
    });
    sim.world.dirty_chunks.insert(cx);
    progress::tasks_stat_add(sim, "water", amount);
    tutorial::event(sim, "water", None, None);
    true
}

// port: world_connect_water_at
/// Connect whatever water is at the point: the ocean (infinite water, not in survival), a
/// lake, or a water cell.
pub fn connect_water_at(sim: &mut Sim, x: f64, y: f64) -> bool {
    let w = &mut sim.world;
    if y < w.params.surface_y || y >= w.params.bottom_y {
        return false;
    }
    if y >= w.ocean_y() {
        if w.bottom_is_rock() || sim.state.state.water_infinite {
            return false;
        }
        sim.state.state.water_infinite = true;
        return true;
    }
    if connect_lake_at(sim, x, y) {
        return true;
    }
    let w = &mut sim.world;
    let loc = w.locate(x, y);
    w.chunk_reach(loc.cx, loc.row);
    let chunk = w.chunks.get_mut(&loc.cx).unwrap();
    let i = loc.idx as usize;
    if chunk.tiles[i] == 3 && !chunk.connected[i] {
        chunk.connected[i] = true;
        let amount = chunk.amount[i];
        w.dirty_chunks.insert(loc.cx);
        progress::tasks_stat_add(sim, "water", amount);
        sim.state.water.push(WaterSave::Cell {
            cx: loc.cx,
            idx: loc.idx,
            extra: Default::default(),
        });
        return true;
    }
    false
}

// port: water_source_amount
/// Water left in a connected source (0 if its chunk is not generated, which a connected
/// source's chunk always is).
pub fn water_source_amount(sim: &Sim, src: &WaterSave) -> f64 {
    match src {
        WaterSave::Lake { lake_cx, lake, .. } => sim
            .world
            .chunk(*lake_cx)
            .and_then(|c| c.lakes.get(*lake as usize))
            .map_or(0.0, |l| l.amount),
        WaterSave::Cell { cx, idx, .. } => sim
            .world
            .chunk(*cx)
            .and_then(|c| c.amount.get(*idx as usize).copied())
            .unwrap_or(0.0),
    }
}

// port: water_source_take
/// Take up to `want` from source `k` of `sim.state.water`; an emptied water cell becomes soil.
pub fn water_source_take(sim: &mut Sim, k: usize, want: f64) -> f64 {
    let src = sim.state.water[k].clone();
    let take = water_source_amount(sim, &src).min(want);
    let w = &mut sim.world;
    match src {
        WaterSave::Lake { lake_cx, lake, .. } => {
            let l = &mut w.get_chunk(lake_cx).lakes[lake as usize];
            l.amount -= take;
            w.dirty_chunks.insert(lake_cx);
        }
        WaterSave::Cell { cx, idx, .. } => {
            let chunk = w.get_chunk(cx);
            let i = idx as usize;
            chunk.amount[i] -= take;
            if chunk.amount[i] <= 0.0 {
                chunk.tiles[i] = 0;
            }
            w.dirty_chunks.insert(cx);
        }
    }
    take
}

// port: water_spend
/// Spend water: free water first, then the oldest sources (dropped when empty). With
/// infinite water only the shown counter dips.
pub fn water_spend(sim: &mut Sim, n: f64) {
    if sim.state.state.water_infinite {
        sim.world.water_dip += n;
        sim.world.water_dip_hold = vis("water_dip_hold_frames");
        return;
    }
    let mut need = n;
    let take = sim.state.state.water_free.min(need);
    sim.state.state.water_free -= take;
    need -= take;
    while need > 0.0 && !sim.state.water.is_empty() {
        need -= water_source_take(sim, 0, need);
        let src = sim.state.water[0].clone();
        if water_source_amount(sim, &src) <= 0.0 {
            sim.state.water.remove(0);
        }
    }
}

// port: water_shown
/// The water amount the HUD shows.
pub fn water_shown(sim: &Sim) -> f64 {
    if sim.state.state.water_infinite {
        return (1_000_000.0 - sim.world.water_dip).max(0.0);
    }
    sim.water_total()
}

// port: water_counter_step
/// obj_hud Begin Step: after a hold, the infinite-water dip refills.
pub fn water_counter_step(sim: &mut Sim) {
    let w = &mut sim.world;
    if w.water_dip <= 0.0 {
        return;
    }
    if w.water_dip_hold > 0.0 {
        w.water_dip_hold -= 1.0;
    } else {
        w.water_dip = (w.water_dip * vis("water_dip_refill_keep") - 1.0).max(0.0);
    }
}
