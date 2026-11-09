-- The Task 2 Exploration fixture loaded through load_game: world chunks and tiles (saved rows
-- and newly reached ones), forest chunks and host queries, and what save_game writes back. Task
-- 6/7 features neutralized (world_lib.lua). For tests/world.rs and tests/forest.rs.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.neutralize_caves()
L.neutralize_stones()
L.preload()

local function rle(t)
  local out = {}
  local i = 1
  while i <= #t do
    local v, n = t[i], 1
    while t[i + n] == v do n = n + 1 end
    out[#out + 1] = { v, n }
    i = i + n
  end
  return out
end

local f = assert(io.open("rust/crates/myco_core/tests/fixtures/exploration.json", "rb"))
local text = f:read("*a"):gsub("%z+$", "")
f:close()
local slot = 4
save_write_text(global, global, save_path(global, global, slot), text)
assert(load_game(global, global, slot), "load_game failed")
global.title_open, global.intro_active = false, false

local out = { params = L.params() }
local w = global.world
local saved = {}
local keys = gm.variable_struct_get_names(w.chunks)
for i = 0, gm.array_length(keys) - 1 do
  local ch = gm.variable_struct_get(w.chunks, keys[i])
  saved[#saved + 1] = ch.cx
end
out.saved_cx = saved
out.tile_grid_saved = {}
for _, g in ipairs(L.tile_grid(-3, 2, 31)) do
  g.codes = rle(g.codes)
  out.tile_grid_saved[#out.tile_grid_saved + 1] = g
end
-- reach two more layers in every saved chunk, then a new chunk
for _, cx in ipairs(saved) do
  world_chunk_reach(global, global, world_get_chunk(global, global, cx), 3 * w.layer_rows - 1)
end
world_get_chunk(global, global, 3)
out.chunks = {}
keys = gm.variable_struct_get_names(w.chunks)
for i = 0, gm.array_length(keys) - 1 do
  local d = L.chunk_data(gm.variable_struct_get(w.chunks, keys[i]))
  d.tiles = rle(d.tiles)
  d.amount = rle(d.amount)
  out.chunks[keys[i]] = d
end
out.chunk_index = L.plain(w.chunk_index)
out.tile_grid = {}
for _, g in ipairs(L.tile_grid(-3, 3, 93)) do
  g.codes = rle(g.codes)
  out.tile_grid[#out.tile_grid + 1] = g
end

-- forest
local fc = gm.instance_find(obj_forest_controller, 0)
local function host_id(t)
  if t == gm.undefined or t == nil then return "none" end
  if gm.variable_struct_exists(t, "part") then return { "stump", t.x, t.part.left } end
  return { "tree", t.x, gm.sprite_get_name(t.sprite) }
end
out.forest = {
  camera = { fc.camera_x, fc.camera_y },
  start_stump = { fc.start_stump.x, fc.start_stump.part.left },
  chunk_keys = L.plain(gm.variable_struct_get_names(fc.surface_chunks)),
}
out.host_near = {}
out.tree_near = {}
for x = -800, 800, 23 do -- keeps every query inside the saved surface chunks s-2..s1
  for _, r in ipairs({ 10, 60, 200 }) do
    out.host_near[#out.host_near + 1] = { x, r, host_id(fc.forest_find_host_near(fc, fc, x, r)) }
    out.tree_near[#out.tree_near + 1] = { x, r, host_id(fc.forest_find_tree_near(fc, fc, x, r)) }
  end
end
out.slots = {}
for i = 0, gm.array_length(fc.start_stump.part.slots) - 1 do
  local s = fc.stump_slot_at(fc, fc, fc.start_stump, i)
  out.slots[#out.slots + 1] = { s.x, s.y, s.face }
end
out.clearing = {}
for _, x in ipairs({ -200, -100, -99, 0, 99, 100, 101, 150, 400 }) do
  out.clearing[#out.clearing + 1] = { x, fc.surface_in_start_clearing(fc, fc, x) and 1 or 0 }
end
out.near_stump = {}
for _, x in ipairs({ -900, -700, -650, -600, -559, -400, -300, 0 }) do
  for _, hw in ipairs({ 5, 40, 80 }) do
    out.near_stump[#out.near_stump + 1] = { x, hw, fc.surface_near_stump(fc, fc, fc.start_stumps, x, hw) and 1 or 0 }
  end
end

-- what save_game writes right after the load (world/forest/water parts)
local captured
local saved_write = save_write_text
save_write_text = function(self, other, path, t)
  if not captured then captured = t end
end
global.save_slot = slot
local ok, err = pcall(save_game, global, global)
save_write_text = saved_write
assert(ok and captured, "save_game failed: " .. tostring(err))
out.save_json = captured
return out
