-- Body of stones_s31607.lua / stones_s7.lua (run with the matching MYCO_SEED): a fresh world
-- with stones, deposits, surface walls, mazes and rock clusters live (caves neutral, R19):
-- chunks -3..3 through layers 0..40, the first wall and maze on each side, late clusters, tile
-- and stone-cell queries, stones shoved and stepped, and the obstacle state, for tests/stones.rs.
local gm = require("gm")
local S = dofile("rust/golden/scripts/stones_lib.lua")
local L = S.L
S.setup()

local w = global.world
local out = { seed = w.seed, init_chunks = S.chunks() }

local layers = 41
for cx = -3, 3 do
  local ch = world_get_chunk(global, global, cx)
  world_chunk_reach(global, global, ch, layers * w.layer_rows - 1)
end
-- the first wall and maze on each side, three layers deep
local far = {}
for _, dir in ipairs({ 1, -1 }) do
  local list = world_obstacles_reach(global, global, dir, w.start_x + dir * 30000)
  local got = {}
  for i = 0, gm.array_length(list) - 1 do
    local o = list[i]
    if not got[o.kind] then
      got[o.kind] = true
      local c1 = math.floor(o.x1 / w.chunk_w)
      local c2 = math.floor((o.x2 - 1) / w.chunk_w)
      for cx = c1, c2 do far[#far + 1] = cx end
    end
  end
end
out.far = far
for _, cx in ipairs(far) do
  local ch = world_get_chunk(global, global, cx)
  world_chunk_reach(global, global, ch, 3 * w.layer_rows - 1)
end
-- chunks near the first clusters on each side: layer 0 only, then the draw-time late pass
local late = { 8, 9, -9, -10 }
out.late = late
for _, cx in ipairs(late) do
  world_paint_clusters_late(global, global, world_get_chunk(global, global, cx))
end
for _, cx in ipairs(late) do
  local ch = world_get_chunk(global, global, cx)
  world_chunk_reach(global, global, ch, 20 * w.layer_rows - 1)
  world_paint_clusters_late(global, global, ch)
end
out.chunks = S.chunks()
out.chunk_index = L.plain(w.chunk_index)

out.tile_grid = {}
for _, g in ipairs(L.tile_grid(-3, 3, 200)) do
  g.codes = S.rle(g.codes)
  out.tile_grid[#out.tile_grid + 1] = g
end

-- stone_solid_at / world_stone_covers inside (and around) the first stone cells of each chunk
out.cell_queries = {}
for cx = -3, 3 do
  local ch = world_get_chunk(global, global, cx)
  local q = {}
  local n = math.min(25, gm.array_length(ch.stone_cells))
  for i = 0, n - 1 do
    local c = ch.stone_cells[i * math.floor(gm.array_length(ch.stone_cells) / n)]
    for y = c.row * 48 - 3, c.row * 48 + 50, 5 do
      for x = c.col * 48 - 2, c.col * 48 + 50, 6 do
        q[#q + 1] = (stone_solid_at(global, global, ch, x + 0.5, y + 0.25) and 1 or 0)
          + (world_stone_covers(global, global, ch, x + 0.5, y + 0.25) and 2 or 0)
      end
    end
  end
  out.cell_queries[#out.cell_queries + 1] = { cx = cx, n = n, q = S.rle(q) }
end

-- shove stones near the start and let them roll
gm.random_set_seed(4242)
local function snapshot()
  local s = {}
  for cx = -2, 2 do
    local list = world_get_chunk(global, global, cx).stones
    for i = 0, gm.array_length(list) - 1 do
      local t = list[i]
      if t.moving or t.vx ~= 0 or t.angle ~= 0 then
        s[#s + 1] = { cx, i, t.x, t.y, t.angle, t.vx, t.vy, t.spin, t.moving and 1 or 0 }
      end
    end
  end
  return s
end
out.shoves = {}
local k = 0
for cx = -2, 2 do
  local list = world_get_chunk(global, global, cx).stones
  for i = 0, gm.array_length(list) - 1, 4 do
    local t = list[i]
    k = k + 1
    local args = { t.x + 5 - (k % 3) * 4, t.y - 3 + (k % 5), (k * 47) % 360, 6 + k % 7 }
    stones_shove(global, global, args[1], args[2], args[3], args[4])
    out.shoves[#out.shoves + 1] = args
  end
end
out.frames = {}
for f = 1, 150 do
  stones_step(global, global)
  if f % 10 == 0 or f <= 3 then
    out.frames[#out.frames + 1] = { f, gm.array_length(global.stones_moving), snapshot() }
  end
  if f == 40 then
    -- a second shove while stones still move
    stones_shove(global, global, 300, 700, 135, 9)
  end
end
out.rng_after = gm.random(1)

-- cells cut out of the first right maze's second chunk next to its left border (as
-- world_clear_caves does), then stone_redress_after_cut: thin cells go, tiles are re-picked,
-- and the border-crossing cells are the same structs in both chunks and in the maze data
local maze_o
do
  local list = w.obstacles.right.list
  for i = 0, gm.array_length(list) - 1 do
    if list[i].kind == "maze" then maze_o = list[i] break end
  end
end
-- the first chunk border inside the maze that cuts through a cell (chunk_w is not a multiple of 48)
local rc = math.floor(maze_o.x1 / w.chunk_w) + 1
while (rc * w.chunk_w) % 48 == 0 do rc = rc + 1 end
local rch = world_get_chunk(global, global, rc)
local bcol = math.floor(rc * w.chunk_w / 48)
local top = math.floor(w.surface_y / 48)
local cut = gm.struct({})
local taken = 0
for row = top + 3, top + 25 do
  for col = bcol + 1, bcol + 2 do
    local key = stone_cell_key(global, global, col, row)
    if taken < 6 and not gm.is_undefined(gm.variable_struct_get(rch.stone_map, key)) then
      taken = taken + 1
      gm.variable_struct_set(cut, key, true)
      gm.variable_struct_remove(rch.stone_map, key)
    end
  end
end
local keep = gm.array()
for i = 0, gm.array_length(rch.stone_cells) - 1 do
  local c = rch.stone_cells[i]
  if gm.is_undefined(gm.variable_struct_get(cut, stone_cell_key(global, global, c.col, c.row))) then
    gm.array_push(keep, c)
  end
end
rch.stone_cells = keep
stone_redress_after_cut(global, global, rch, cut)
local cut_keys = L.plain(gm.variable_struct_get_names(cut))
table.sort(cut_keys)
out.redress = {
  cx = rc, bcol = bcol, cut = cut_keys,
  cells = L.plain(rch.stone_cells), map_n = gm.array_length(gm.variable_struct_get_names(rch.stone_map)),
  left_cells = L.plain(world_get_chunk(global, global, rc - 1).stone_cells),
}

out.obstacles = L.plain(w.obstacles)
return out
