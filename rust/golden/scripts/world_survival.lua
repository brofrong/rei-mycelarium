-- Survival layout (MYCO_SEED=31607): game_mode_start("survival") thins the existing world, new
-- chunks are thinned as they generate, the bottom is rock. Plus the infinite-water path of an
-- exploration world. Task 6/7 features neutralized (world_lib.lua). For tests/world.rs.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
assert(os.getenv("MYCO_SEED") == "31607", "run with MYCO_SEED=31607")
L.neutralize_caves()
L.neutralize_stones()
L.preload()

local out = {}
-- exploration: the ocean gives infinite water; spending dips the shown amount, which refills
L.new_world()
local oy = world_ocean_y(global, global)
out.ocean = { oy, world_tile_at(global, global, 10, oy + 5), world_water_at(global, global, 10, oy + 5) and 1 or 0 }
out.infinite = {}
out.infinite[1] = world_connect_water_at(global, global, 10, oy + 5) and 1 or 0
out.infinite[2] = world_connect_water_at(global, global, 10, oy + 5) and 1 or 0
out.infinite[3] = global.water_infinite and 1 or 0
water_spend(global, global, 50)
out.dip = {}
for i = 1, 45 do
  water_counter_step(global, global)
  out.dip[i] = { global.water_dip, global.water_dip_hold, water_shown(global, global) }
end
water_spend(global, global, 7)
out.dip[#out.dip + 1] = { global.water_dip, global.water_dip_hold, water_shown(global, global) }
global.water_infinite = false
global.water_dip, global.water_dip_hold = 0, 0
global.water_sources = gm.array()

-- survival
L.new_world()
game_mode_start(global, global, "survival")
out.thinned = L.chunks()
for _, cx in ipairs({ 3, -4, 6 }) do
  world_get_chunk(global, global, cx)
end
world_chunk_reach(global, global, world_get_chunk(global, global, 0), 4 * global.world.layer_rows - 1)
world_chunk_reach(global, global, world_get_chunk(global, global, 6), 3 * global.world.layer_rows - 1)
out.chunks = {}
local keys = gm.variable_struct_get_names(global.world.chunks)
for i = 0, gm.array_length(keys) - 1 do
  local ch = gm.variable_struct_get(global.world.chunks, keys[i])
  out.chunks[keys[i]] = { rows = ch.rows, foods = L.plain(ch.foods), lakes = L.plain(ch.lakes) }
end
out.chunk_index = L.plain(global.world.chunk_index)
out.full_at = {}
for _, p in ipairs({ { 0, 480 }, { 1920, 1000 }, { 1921, 1000 }, { -1920, 2640 }, { -1920, 2641 }, { 5000, 600 } }) do
  out.full_at[#out.full_at + 1] = { p[1], p[2], world_survival_full_at(global, global, p[1], p[2]) and 1 or 0 }
end
out.rock = { world_bottom_is_rock(global, global) and 1 or 0, world_tile_at(global, global, 10, oy + 5),
  world_water_at(global, global, 10, oy + 5) and 1 or 0, world_water_shown_at(global, global, 10, oy + 5) and 1 or 0,
  world_connect_water_at(global, global, 10, oy + 5) and 1 or 0 }
game_mode_apply(global, global, "exploration")
return out
