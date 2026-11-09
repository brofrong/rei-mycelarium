-- Shared helpers for the world golden scripts (not a topic itself). Since ruling R19's
-- re-capture everything is live: caves, bones, relics, stones, deposits, walls, mazes and rock
-- clusters.
local gm = require("gm")
local L = {}

-- the file-local caches (statics) of the cave/bone/relic/cluster functions and the cave
-- globals, emptied so a new world computes them afresh like a new process (the boot world may
-- share the seed); myco_core keeps these caches in WorldRt, new with every world
local function reset_static(fn, name, fields)
  local i = 1
  while true do
    local n, v = debug.getupvalue(fn, i)
    if n == nil then error("no upvalue " .. name) end
    if n == name then
      for k, x in pairs(fields) do v[k] = x end
      return
    end
    i = i + 1
  end
end

function L.reset()
  reset_static(world_cave_rects, "world_cave_rects_static", { _cache = gm.undefined })
  reset_static(world_big_bones, "world_big_bones_static", { _cache = gm.undefined, _key = "" })
  reset_static(world_clear_zones, "world_clear_zones_static", { _cache = gm.undefined, _key = "" })
  reset_static(world_relics, "world_relics_static", { _for = gm.undefined, _out = gm.array() })
  reset_static(world_rock_clusters, "world_rock_clusters_static", { _init = false })
  global.cave_yellow = gm.undefined
  global.cave_blue_state = gm.undefined
  global.cave_perks = gm.struct({})
  global.cave_taken = gm.struct({})
  global.relic_taken = gm.struct({})
  global.cave_flash = gm.struct({})
  global.cave_wrap = gm.struct({})
  global.cave_popups = gm.array()
  global.cave_burst_pending = false
  global.play_frames = 0
end

-- the asset caches the Rust port always has (art_preload_step / world_draw build them in LÖVE),
-- then fresh caches
function L.preload()
  lakes_init(global, global)
  leaf_fg_art_init(global, global)
  stones_init(global, global)
  stone_tiles_init(global, global)
  caves_mask_init(global, global)
  L.reset()
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
