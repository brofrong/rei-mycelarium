-- Shared helpers for the Task 6 cave golden scripts (not a topic itself). Everything is live
-- (ruling R19 re-capture): caves sit on real mazes and rock clusters.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
local C = { L = L }

C.reset = L.reset

function C.setup()
  L.preload()
end

function C.rect(r)
  return { id = r.id, def = r.def.id, k = r.k, x1 = r.x1, y1 = r.y1, x2 = r.x2, y2 = r.y2 }
end

function C.rects()
  local out = {}
  local rs = world_cave_rects(global, global)
  for i = 0, gm.array_length(rs) - 1 do out[#out + 1] = C.rect(rs[i]) end
  return out
end

function C.boxes(list)
  local out = {}
  for i = 0, gm.array_length(list) - 1 do
    local r = list[i]
    out[#out + 1] = { r.id or "", r.x1, r.y1, r.x2, r.y2 }
  end
  return out
end

local function hosts(list)
  local out = {}
  for i = 0, gm.array_length(list) - 1 do
    local h = list[i]
    out[#out + 1] = { key = h.key, x1 = h.x1, y1 = h.y1, x2 = h.x2, y2 = h.y2, used = h.used and 1 or 0 }
  end
  return out
end

function C.blue_state()
  local st = global.cave_blue_state
  if not gm.is_struct(st) then return "undefined" end
  local function lane(l) return { dir = l.dir, n = l.n, gap = l.gap, cursor = l.cursor } end
  return { mazes = hosts(st.mazes), walls = hosts(st.walls), left = lane(st.left), right = lane(st.right) }
end

function C.clusters()
  local out = {}
  local cs = world_rock_clusters(global, global)
  for i = 0, gm.array_length(cs) - 1 do
    local c = cs[i]
    out[#out + 1] = {
      id = c.id, tier = c.tier, key = c.key, col = c.col, row = c.row, r = c.r, c1 = c.c1, c2 = c.c2,
      x1 = c.x1, y1 = c.y1, x2 = c.x2, y2 = c.y2,
    }
  end
  return out
end

function C.rle(t)
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

function C.chunk(ch)
  local d = L.chunk_data(ch)
  d.tiles = C.rle(d.tiles)
  d.amount = C.rle(d.amount)
  d.bones = L.plain(ch.bones)
  return d
end

function C.chunks()
  local out = {}
  local keys = gm.variable_struct_get_names(global.world.chunks)
  for i = 0, gm.array_length(keys) - 1 do
    out[keys[i]] = C.chunk(gm.variable_struct_get(global.world.chunks, keys[i]))
  end
  return out
end

-- probe points over a box (step sx, sy), offset so they fall between pixels and tiles
function C.grid(x1, y1, x2, y2, sx, sy)
  local pts = {}
  for y = y1, y2, sy do
    for x = x1, x2, sx do pts[#pts + 1] = { x + 0.37, y + 0.61 } end
  end
  return pts
end

return C
