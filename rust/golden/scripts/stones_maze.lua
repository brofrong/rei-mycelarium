-- Maze, rock-cluster and obstacle generators (seed 31607): maze_rng sequences, the obstacle bands
-- and rock clusters, built mazes and clusters, blob and cut primitives, and the maze primitives
-- on a synthetic maze, for tests/stones.rs. Run: MYCO_SEED=31607 tools/golden.sh stones_maze
assert(os.getenv("MYCO_SEED") == "31607", "run with MYCO_SEED=31607")
local gm = require("gm")
local S = dofile("rust/golden/scripts/stones_lib.lua")
local L = S.L
S.setup()

local out = { seed = global.world.seed, start_x = global.world.start_x }

-- maze_rng / maze_rng_int sequences
out.rng = {}
for _, key in ipairs({ 7, 131, 4 * 31 + 7, 90000 * 31 + 11, -5 * 31 + 7, 123456789 }) do
  local m = { key = key, n = 0 }
  local seq, ints = {}, {}
  for i = 1, 40 do seq[i] = maze_rng(global, global, m) end
  for i = 1, 20 do ints[i] = maze_rng_int(global, global, m, -1, 3) end
  out.rng[#out.rng + 1] = { key = key, seq = seq, ints = ints, n = m.n }
end

-- obstacle bands both ways (built lazily from start_x on the first call)
out.right = {}
for _, p in ipairs(L.plain(world_obstacles_reach(global, global, 1, 60000))) do
  p.data = nil
  out.right[#out.right + 1] = p
end
out.left = {}
for _, p in ipairs(L.plain(world_obstacles_reach(global, global, -1, -60000))) do
  p.data = nil
  out.left[#out.left + 1] = p
end
out.cursors = { global.world.obstacles.right.cursor, global.world.obstacles.right.count,
  global.world.obstacles.left.cursor, global.world.obstacles.left.count }
out.at_chunk = {}
for _, cx in ipairs({ -60, -12, -5, 0, 5, 6, 17, 18, 22, 40 }) do
  local list = world_obstacles_at_chunk(global, global, { cx = cx })
  local keys = {}
  for i = 0, gm.array_length(list) - 1 do keys[#keys + 1] = list[i].key end
  out.at_chunk[#out.at_chunk + 1] = { cx, keys }
end

-- rock clusters
local clusters = world_rock_clusters(global, global)
out.clusters = {}
for i = 0, gm.array_length(clusters) - 1 do out.clusters[#out.clusters + 1] = S.part(clusters[i]) end

-- built mazes (first two each side) and clusters near the start
local function data(d)
  return { entries = L.plain(d.entries), foods = L.plain(d.foods), lakes = L.plain(d.lakes) }
end
out.mazes = {}
local function mazes_of(dir)
  local list = global.world.obstacles[dir > 0 and "right" or "left"].list
  local n = 0
  for i = 0, gm.array_length(list) - 1 do
    if list[i].kind == "maze" and n < 2 then
      n = n + 1
      out.mazes[#out.mazes + 1] = { key = list[i].key, data = data(world_maze_build(global, global, list[i])) }
    end
  end
end
mazes_of(1)
mazes_of(-1)
out.cluster_data = {}
for i = 0, gm.array_length(clusters) - 1 do
  local c = clusters[i]
  local s = tonumber(c.id:match("_(%-?%d+)$"))
  if s >= -1 and s <= 1 then
    out.cluster_data[#out.cluster_data + 1] = { id = c.id, data = data(world_cluster_build(global, global, c)) }
  end
end

-- blob primitives (stone_blob_cells -> stone_blob_finish -> stone_blob_largest_piece)
local function cells(blob)
  local t = {}
  for i = 0, gm.array_length(blob.list) - 1 do t[#t + 1] = { blob.list[i].col, blob.list[i].row } end
  local keys = L.plain(gm.variable_struct_get_names(blob.keys))
  table.sort(keys)
  return { list = t, keys = keys }
end
out.blobs = {}
for _, a in ipairs({ { 10, 20, 3, 2, 2 }, { -7, 33, 7, 5, 5 }, { 120, 60, 9, 9, 4 }, { 0, 12, 1, 1, 2 }, { 55, 41, 5, 7, 3 } }) do
  out.blobs[#out.blobs + 1] = { a, cells(stone_blob_cells(global, global, a[1], a[2], a[3], a[4], a[5])) }
end
-- protect > 0 keeps every piece of at least that size
local g = gm.struct({})
for y = 0, 9 do
  for x = 0, 9 do
    if (x < 4 or x > 6) and not (y == 5 and x < 2) then gm.variable_struct_set(g, stone_cell_key(global, global, x, y), true) end
  end
end
gm.variable_struct_set(g, stone_cell_key(global, global, 20, 20), true)
out.blob_protect = cells(stone_blob_finish(global, global, g, -1, 21, -1, 21, 6))

-- cuts on a full grid
local function full()
  local k = gm.struct({})
  for y = 0, 29 do for x = 0, 29 do gm.variable_struct_set(k, stone_cell_key(global, global, x, y), true) end end
  return k
end
local function left(k)
  local t = L.plain(gm.variable_struct_get_names(k))
  table.sort(t)
  return t
end
local k1 = full()
stone_cut_round(global, global, k1, 12.3, 14.8, 5.4, 3.9, 17)
local k2 = full()
stone_cut_tunnel(global, global, k2, 3.5, 4.2, 25.5, 21.7, 1.6, 23)
out.cuts = { left(k1), left(k2) }

-- maze primitives on a synthetic maze
local function maze()
  return { key = 12345, n = 0, c1 = 0, c2 = 39, top = 10, w = 40, h = 30, keep_top = 3,
    xmin = 2, xmax = 37, ymin = 14, ymax = 33, open = gm.array_create(1200, false) }
end
local function open_of(m)
  local t = {}
  for i = 0, m.w * m.h - 1 do t[#t + 1] = m.open[i] and 1 or 0 end
  return S.rle(t)
end
local m = maze()
maze_disc(global, global, m, 5.5, 12.25, 2.6)
local room = maze_room(global, global, m, 20, 20, 3.2, 2.1)
out.room = L.plain(room)
local pts = maze_tunnel(global, global, m, 0, 20, 10, 80, 1, 2.2, gm.array(39, 25), true)
out.tunnel = L.plain(pts)
local pts2 = maze_tunnel(global, global, m, 30, 30, 200, 30, 0.8, 1.6, gm.undefined, false)
out.tunnel2 = L.plain(pts2)
out.open1 = open_of(m)
out.n1 = m.n
local dist = maze_distance(global, global, m, gm.undefined)
out.dist = S.rle(L.plain(dist))
maze_dig_out(global, global, m, dist, 20, 25)
local rooms = gm.array()
maze_hollows(global, global, m, rooms)
out.hollows = L.plain(rooms)
local rooms2 = gm.array()
maze_branches(global, global, m, pts, rooms2)
out.branches = L.plain(rooms2)
out.open2 = open_of(m)
out.n2 = m.n
local fin = maze_finish(global, global, m, (function()
  local cut = gm.struct({})
  for c = 0, 39 do
    for r = 0, 25 do
      if not maze_is_open(global, global, m, c, 10 + r) then
        gm.variable_struct_set(cut, stone_cell_key(global, global, c, 10 + r), true)
      end
    end
  end
  return cut
end)(), rooms, -1, 40, 9, 41, true)
out.finish = data(fin)
out.n3 = m.n
return out
