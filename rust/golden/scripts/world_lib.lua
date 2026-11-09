-- Shared helpers for the Task 5 world golden scripts (not a topic itself).
--
-- `neutralize()` replaces the cave (Task 6) and stone/maze/obstacle (Task 7) entry points that
-- world generation and the tile queries call with neutral versions: no caves, no stones, no
-- bones, no obstacles. myco_core's world hooks for those tasks have exactly these neutral
-- bodies until Tasks 6/7 port them, so the Task 5 references compare the Task 5 logic alone.
-- Tasks 6/7 re-capture these topics without neutralizing.
local gm = require("gm")
local L = {}

function L.neutralize()
  local function noop() return gm.undefined end
  local function no() return false end
  caves_extend = noop
  world_paint_bones = noop
  world_clear_caves = noop
  world_cave_hits = no
  world_cave_solid_any = no
  world_paint_clusters = noop
  world_paint_stone_clusters = noop
  world_paint_stone_pillar = noop
  world_obstacles_at_chunk = function() return gm.array() end
  world_paint_maze_part = noop
  world_paint_surface_wall = noop
  world_paint_stone_deposits = noop
  world_paint_stones = noop
  stone_solid_at = no
  world_stone_covers = no
end

-- the asset caches the Rust port always has (art_preload_step builds them on the first step)
function L.preload()
  lakes_init(global, global)
  leaf_fg_art_init(global, global)
end

-- gm arrays (0-based, __n) and structs -> plain Lua tables for main.lua's JSON writer;
-- undefined -> "undefined"
function L.plain(v)
  if v == gm.undefined then return "undefined" end
  if type(v) ~= "table" then return v end
  local n = rawget(v, "__n")
  if n then
    local t = {}
    for i = 0, n - 1 do t[i + 1] = L.plain(rawget(v, i)) end
    return t
  end
  local t = {}
  for k, x in pairs(v) do
    if type(k) == "string" then t[k] = L.plain(x) end
  end
  return t
end

-- a fresh world exactly as game_init makes it (world_init honours MYCO_SEED)
function L.new_world()
  world_init(global, global, global.start_x)
end

local function chunk_data(ch)
  return {
    cx = ch.cx,
    rows = ch.rows,
    tiles = L.plain(ch.tiles),
    amount = L.plain(ch.amount),
    foods = L.plain(ch.foods),
    lakes = L.plain(ch.lakes),
  }
end
L.chunk_data = chunk_data

-- world params (global.world without chunks)
function L.params()
  local w = global.world
  return {
    tile = w.tile, surface_y = w.surface_y, rows = w.rows, cols = w.cols, chunk_w = w.chunk_w,
    bottom_y = w.bottom_y, water_row = w.water_row, hard_row = w.hard_row,
    layer_rows = w.layer_rows, seed = w.seed, turf = w.turf, start_x = w.start_x,
    chunk_index = L.plain(w.chunk_index),
  }
end

-- every chunk currently generated, keyed by chunk key
function L.chunks()
  local out = {}
  local keys = gm.variable_struct_get_names(global.world.chunks)
  for i = 0, gm.array_length(keys) - 1 do
    out[keys[i]] = chunk_data(gm.variable_struct_get(global.world.chunks, keys[i]))
  end
  return out
end

-- world_tile_at at every tile centre of chunks c0..c1, rows 0..rows-1 (row-major per chunk)
function L.tile_grid(c0, c1, rows)
  local w = global.world
  local out = {}
  for cx = c0, c1 do
    local codes = {}
    for row = 0, rows - 1 do
      local y = w.surface_y + row * w.tile + w.tile * 0.5
      for col = 0, w.cols - 1 do
        local x = (cx * w.cols + col) * w.tile + w.tile * 0.5
        codes[#codes + 1] = world_tile_at(global, global, x, y)
      end
    end
    out[#out + 1] = { cx = cx, rows = rows, codes = codes }
  end
  return out
end

return L
