-- Body of world_s31607.lua / world_s7.lua (run with the matching MYCO_SEED): a fresh world with
-- caves, stones and mazes live, its chunks -2..2 reached through
-- layers 0..40, tile queries, then the water/food mutations, for tests/world.rs.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.preload()
L.new_world()

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

local w = global.world
local out = { seed = w.seed, params = L.params(), init_chunks = L.chunks() }

-- reach layers 0..40 of chunks -2..2
local layers = 41
for cx = -2, 2 do
  local ch = world_get_chunk(global, global, cx)
  world_chunk_reach(global, global, ch, layers * w.layer_rows - 1)
end
out.chunks = {}
local keys = gm.variable_struct_get_names(w.chunks)
for i = 0, gm.array_length(keys) - 1 do
  local d = L.chunk_data(gm.variable_struct_get(w.chunks, keys[i]))
  d.tiles = rle(d.tiles)
  d.amount = rle(d.amount)
  out.chunks[keys[i]] = d
end
out.chunk_index = L.plain(w.chunk_index)

-- world_tile_at at tile centres, rows 0..199 (dense, deep, ocean rows inside)
out.tile_grid = {}
for _, g in ipairs(L.tile_grid(-2, 2, 200)) do
  g.codes = rle(g.codes)
  out.tile_grid[#out.tile_grid + 1] = g
end

-- point queries: around every lake of chunks -2..2 and on a coarse grid
local pts = {}
for cx = -2, 2 do
  local ch = world_get_chunk(global, global, cx)
  for i = 0, gm.array_length(ch.lakes) - 1 do
    local l = ch.lakes[i]
    for y = l.body_y - 20, l.body_y + l.body_h + 20, 13 do
      for x = l.body_x - 20, l.body_x + l.body_w + 20, 17 do
        pts[#pts + 1] = { x + 0.25, y + 0.5 }
      end
    end
  end
end
for y = -100, 12000, 377 do
  for x = -2100, 2100, 233 do
    pts[#pts + 1] = { x, y }
  end
end
pts[#pts + 1] = { 0 / 0, 500 }
local function queries()
  local q = {}
  for i, p in ipairs(pts) do
    local x, y = p[1], p[2]
    q[i] = {
      world_tile_at(global, global, x, y),
      world_water_at(global, global, x, y) and 1 or 0,
      world_water_shown_at(global, global, x, y) and 1 or 0,
      world_is_passable(global, global, x, y) and 1 or 0,
      world_is_deep_soil(global, global, x, y) and 1 or 0,
      world_soil_water_factor(global, global, x, y),
      world_travel_cost(global, global, x, y, 0.1),
    }
  end
  return q
end
local plain_pts = {}
for i, p in ipairs(pts) do plain_pts[i] = { p[1] == p[1] and p[1] or "nan", p[2] } end
out.points = plain_pts
out.queries = queries()
out.locate = {}
for _, p in ipairs({ { 0, 480 }, { -1, 543 }, { 1023.9, 1000 }, { -1024, 479 }, { 5000.5, 9000 } }) do
  local l = world_locate(global, global, p[1], p[2])
  out.locate[#out.locate + 1] = { p[1], p[2], l.cx, l.row, l.idx }
end
out.food_value_in = {}
for _, r in ipairs({ { -960, 0, 960, 1080 }, { -2048, 480, 2048, 3000 }, { 0, 0, 100, 100 }, { -3000, 2000, 3000, 6000 } }) do
  out.food_value_in[#out.food_value_in + 1] = { r[1], r[2], r[3], r[4], world_food_value_in(global, global, r[1], r[2], r[3], r[4]) }
end

-- mutations: connect water at lake centres (and one miss), spend water, collect food
local function sources()
  local s = {}
  for i = 0, gm.array_length(global.water_sources) - 1 do
    local src = global.water_sources[i]
    if gm.is_struct(src.lake) then
      s[#s + 1] = { "lake", src.lake.cx, src.lake.body_x, src.lake.amount }
    else
      s[#s + 1] = { "cell", src.cx, src.idx, water_source_amount(global, global, src) }
    end
  end
  return s
end
out.connect = {}
for cx = -2, 2 do
  local ch = world_get_chunk(global, global, cx)
  for i = 0, gm.array_length(ch.lakes) - 1 do
    local l = ch.lakes[i]
    local x, y = l.body_x + l.body_w * 0.5, l.body_y + l.body_h * 0.5
    out.connect[#out.connect + 1] = { x, y, world_connect_water_at(global, global, x, y) and 1 or 0 }
    -- second call on the same lake: already connected
    out.connect[#out.connect + 1] = { x, y, world_connect_water_at(global, global, x, y) and 1 or 0 }
  end
end
out.connect[#out.connect + 1] = { 0, 300, world_connect_water_at(global, global, 0, 300) and 1 or 0 }
out.sources = sources()
out.water_total = water_total(global, global)
out.spend = {}
for _, n in ipairs({ 30, 60, 5.5, 400, 1000, 2345.5 }) do
  water_spend(global, global, n)
  out.spend[#out.spend + 1] = { n, global.water_free, water_total(global, global), water_shown(global, global), sources() }
end
out.shown_after_spend = {}
for i, p in ipairs(pts) do
  out.shown_after_spend[i] = world_water_shown_at(global, global, p[1], p[2]) and 1 or 0
end
out.collect = {}
for cx = -2, 2 do
  local ch = world_get_chunk(global, global, cx)
  for i = 0, gm.array_length(ch.foods) - 1, 2 do
    local f = ch.foods[i]
    out.collect[#out.collect + 1] = { f.x + 3, f.y - 2, 40, world_collect_food_at(global, global, f.x + 3, f.y - 2, 40) }
  end
end
out.collect[#out.collect + 1] = { 1023, 1500, 300, world_collect_food_at(global, global, 1023, 1500, 300) }
out.final_chunks = {}
for cx = -2, 2 do
  out.final_chunks[#out.final_chunks + 1] = { foods = L.plain(world_get_chunk(global, global, cx).foods),
    lakes = L.plain(world_get_chunk(global, global, cx).lakes) }
end
return out
