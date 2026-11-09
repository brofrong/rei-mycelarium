-- Shared setup for the Task 7 golden scripts (not a topic itself): a world with the Task 6
-- caves neutralized (R19) and the stone/maze/obstacle code live.
--
-- neutralize_caves() leaves world_cave_rects running (it is not one of the generation hooks the
-- Task 5 scripts replace); mazes and rock clusters read it for their cave hollows, so it is made
-- neutral here too, matching myco_core's `caves::world_cave_rects` hook until Task 6 fills it.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
local S = { L = L }

function S.setup()
  L.neutralize_caves()
  world_cave_rects = function() return gm.array() end
  L.preload()
  -- the pixel-derived stone data the Rust port always has (world_draw builds it in LÖVE)
  stones_init(global, global)
  stone_tiles_init(global, global)
  L.new_world()
end

-- chunk_data plus the Task 7 lists
function S.chunk_data(ch)
  local d = L.chunk_data(ch)
  d.stones = L.plain(ch.stones)
  d.stone_cells = L.plain(ch.stone_cells)
  d.stone_map_n = gm.array_length(gm.variable_struct_get_names(ch.stone_map))
  if gm.variable_struct_exists(ch, "clusters_done") then
    d.clusters_done = gm.variable_struct_get_names(ch.clusters_done)
    d.clusters_done = L.plain(d.clusters_done)
    table.sort(d.clusters_done)
  end
  return d
end

function S.rle(t)
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

-- every generated chunk, tiles/amount run-length encoded
function S.chunks()
  local out = {}
  local keys = gm.variable_struct_get_names(global.world.chunks)
  for i = 0, gm.array_length(keys) - 1 do
    local d = S.chunk_data(gm.variable_struct_get(global.world.chunks, keys[i]))
    d.tiles = S.rle(d.tiles)
    d.amount = S.rle(d.amount)
    out[keys[i]] = d
  end
  return out
end

-- an obstacle / cluster without its cached data
function S.part(p)
  local t = L.plain(p)
  t.data = nil
  return t
end

return S
