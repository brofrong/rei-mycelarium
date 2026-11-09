-- Body of caves_s31607.lua / caves_s7.lua (run with the matching MYCO_SEED): a fresh world with
-- everything live (caves, bones, relics, stones, mazes), then the cave layout, the chunks
-- that hold caves and big bones, point queries around them, the pure cave/bone helpers, and
-- the yellow cave appearing. For tests/caves.rs (which replays the same calls in order: the
-- cave caches fill lazily, so the order matters).
local gm = require("gm")
local C = dofile("rust/golden/scripts/caves_lib.lua")
local L = C.L
C.setup()
L.new_world()

local w = global.world
local out = { seed = w.seed }
out.init_chunks = C.chunks()
out.obstacles = L.plain(w.obstacles)
out.rects_init = C.rects()
out.blue_state = C.blue_state()
out.clusters = C.clusters()
out.big_bones = C.boxes(world_big_bones(global, global))
local function relics()
  local rs = world_relics(global, global)
  local o = {}
  for i = 0, gm.array_length(rs) - 1 do
    local r = rs[i]
    o[#o + 1] = { id = r.id, relic = r.relic, def = r.def.id, k = r.k, x1 = r.x1, y1 = r.y1, x2 = r.x2, y2 = r.y2 }
  end
  return o
end
out.relics = relics()
out.zones = C.boxes(world_clear_zones(global, global))

-- generate the chunks under every cave and big bone down to its bottom
local function plan_for(boxes)
  local plan = {}
  for _, b in ipairs(boxes) do
    local x1, y1, x2, y2 = b[1], b[2], b[3], b[4]
    local row = math.floor((y2 + 100 - w.surface_y) / w.tile)
    row = math.min(row, math.floor((world_ocean_y(global, global) - w.surface_y) / w.tile) - 1)
    for cx = math.floor((x1 - 100) / w.chunk_w), math.floor((x2 + 100) / w.chunk_w) do
      plan[#plan + 1] = { cx, row }
    end
  end
  return plan
end
local function run_plan(plan)
  for _, p in ipairs(plan) do
    world_chunk_reach(global, global, world_get_chunk(global, global, p[1]), p[2])
  end
end
local boxes = {}
for _, r in ipairs(out.rects_init) do boxes[#boxes + 1] = { r.x1, r.y1, r.x2, r.y2 } end
for _, b in ipairs(out.big_bones) do boxes[#boxes + 1] = { b[2], b[3], b[4], b[5] } end
out.plan = plan_for(boxes)
run_plan(out.plan)
out.chunks = C.chunks()
out.chunk_index = L.plain(w.chunk_index)

-- point queries around every cave and big bone
local function probe(boxes_)
  local pts, q = {}, {}
  for _, b in ipairs(boxes_) do
    for _, p in ipairs(C.grid(b[1] - 40, b[2] - 40, b[3] + 40, b[4] + 40, 23, 19)) do pts[#pts + 1] = p end
  end
  for i, p in ipairs(pts) do
    local x, y = p[1], p[2]
    q[i] = {
      x, y,
      world_tile_at(global, global, x, y),
      world_cave_solid_any(global, global, x, y) and 1 or 0,
      world_cave_hits(global, global, x, y, x, y, gm.undefined) and 1 or 0,
      world_cave_hits(global, global, x - 30, y - 30, x + 30, y + 30, 10) and 1 or 0,
    }
  end
  return q
end
out.probe = probe(boxes)

-- pure helpers
out.cave_range = {}
for _, t in ipairs({ 0, 0.25, 0.5, 0.999 }) do
  out.cave_range[#out.cave_range + 1] = { t, cave_range(global, global, wld(global, global, "caves_purple_x"), t) }
end
out.bone_weight = {}
out.bone_pick = {}
local kinds = bone_kinds(global, global)
for _, depth in ipairs({ -1, 0, 0.6, 0.8, 1.1, 1.5, 2.0, 2.4, 3.0, 3.5, 5 }) do
  for k = 0, gm.array_length(kinds) - 1 do
    out.bone_weight[#out.bone_weight + 1] = { k, depth, bone_kind_weight(global, global, kinds[k], depth) }
  end
  for _, r in ipairs({ 0, 0.1, 0.3, 0.5, 0.7, 0.9, 0.99, 0.999999 }) do
    local p = bone_kind_pick(global, global, r, depth)
    out.bone_pick[#out.bone_pick + 1] = { r, depth, p.kind, p.share }
  end
end
out.branch_points = {}
for _, a in ipairs({ { { 10, 20 }, 0, 18, 3 }, { { 100.5, 40.25 }, 135, 55, 77 }, { { 0, 0 }, 359, 4, 9 } }) do
  local pts = cave_branch_points(global, global, gm.array(a[1][1], a[1][2]), a[2], a[3], a[4])
  out.branch_points[#out.branch_points + 1] = L.plain(pts)
end
local function host(key, x1, y1, x2, y2, used)
  return gm.struct({ key = key, x1 = x1, y1 = y1, x2 = x2, y2 = y2, used = used })
end
local hs = gm.array(
  host(1, 3000, 480, 6000, 3000, false), host(2, -9000, 480, -5000, 3000, false),
  host(3, 12000, 480, 15000, 3000, true), host(4, 20000, 480, 26000, 3000, false),
  host(5, -2000, 480, -1000, 3000, false))
out.host_pick = {}
for _, side in ipairs({ -1, 0, 1 }) do
  for _, dist in ipairs({ 0, 4000, 9000, 13000, 30000 }) do
    local h = cave_host_pick(global, global, hs, side, dist)
    out.host_pick[#out.host_pick + 1] = { side, dist, gm.is_undefined(h) and -1 or h.key }
  end
end
out.place = {}
local function place(kind, cx, cy, lo, hi, list, extra)
  local l = gm.array()
  for _, r in ipairs(list) do gm.array_push(l, gm.struct({ x1 = r[1], y1 = r[2], x2 = r[3], y2 = r[4] })) end
  local e = gm.undefined
  if extra then
    e = gm.array()
    for _, r in ipairs(extra) do gm.array_push(e, gm.struct({ x1 = r[1], y1 = r[2], x2 = r[3], y2 = r[4] })) end
  end
  local p = cave_place(global, global, l, kind, "t", cx, cy, lo, hi, e)
  out.place[#out.place + 1] = gm.is_undefined(p) and "undefined"
    or { p.x1, p.y1, p.x2, p.y2, p.k, gm.array_length(l) }
end
place("blue", 5000, 1500, 600, 3000, {})
place("purple", 5000.5, 100, 600, 3000, { { 4700, 1000, 5300, 1800 } })
place("red", 5000, 2900, 600, 3000, { { 4700, 600, 5300, 3000 } }, { { 4000, 600, 4800, 3000 } })
place("green", 0, 1500, 600, 700, {}, { { -5000, 0, 5000, 4000 } })
place("yellow", 100, 1500, 600, 3000, { { -10000, 0, 10000, 4000 } })

-- the yellow cave: picks for mycelium tips (stubbed), then cave_yellow_step at the threshold
local tips = {}
mycelium_extreme_tip = function(self, other, dir)
  local t = tips[dir]
  if t == nil then return gm.undefined end
  return gm.struct({ x = t, y = 600 })
end
local function spot_data(s)
  if gm.is_undefined(s) then return "undefined" end
  return { s.x, s.y, s.top, s.bottom, s.cluster }
end
out.untouched = {}
for _, a in ipairs({ { 0, 600, 100, 700, 0 }, { 0, 50000, 100, 50100, 80 }, { 9000, 3000, 9500, 3500, 152 },
  { -40000, 9000, -39000, 9500, 152 }, { 30000, 600, 31000, 900, 0 } }) do
  out.untouched[#out.untouched + 1] = { world_area_untouched(global, global, a[1], a[2], a[3], a[4], a[5]) and 1 or 0 }
end
out.yellow_pick = {}
for _, t in ipairs({ {}, { [1] = 3000 }, { [-1] = -5000 }, { [1] = 40000, [-1] = -40000 }, { [1] = 1e6, [-1] = -1e6 } }) do
  tips = t
  out.yellow_pick[#out.yellow_pick + 1] = spot_data(cave_yellow_pick(global, global))
end
tips = {}
global.title_open, global.intro_active, global.pause_open = true, false, false
local need = wld(global, global, "caves_yellow_after_minutes") * 60 * 60
global.play_frames = need - 3
out.yellow_step = {}
local function step()
  cave_yellow_step(global, global)
  out.yellow_step[#out.yellow_step + 1] = { global.play_frames, spot_data(cave_yellow_spot(global, global)) }
end
step() -- title open: nothing
global.title_open = false
for _ = 1, 6 do step() end
out.rects_yellow = C.rects()
out.big_bones_yellow = C.boxes(world_big_bones(global, global))
out.zones_yellow = C.boxes(world_clear_zones(global, global))
out.blue_state_yellow = C.blue_state()
local yb = {}
for _, r in ipairs(out.rects_yellow) do
  if r.def == "yellow" then yb[#yb + 1] = { r.x1, r.y1, r.x2, r.y2 } end
end
out.plan_yellow = plan_for(yb)
run_plan(out.plan_yellow)
local yc = {}
for _, p in ipairs(out.plan_yellow) do
  local key = world_chunk_key(global, global, p[1])
  yc[key] = C.chunk(gm.variable_struct_get(w.chunks, key))
end
out.chunks_yellow = yc
out.probe_yellow = probe(yb)

-- a fresh process loading this world with the yellow spot saved: the caches start empty and
-- fill with the spot already set (world_big_bones runs inside world_cave_rects)
local spot = global.cave_yellow
C.reset()
global.cave_yellow = spot
out.rects_reload = C.rects()
out.big_bones_reload = C.boxes(world_big_bones(global, global))
out.zones_reload = C.boxes(world_clear_zones(global, global))
out.relics_reload = relics()

-- the same with a (synthetic) yellow spot right on the T-rex: the big bones ignore the yellow
-- cave, the yellow cave is placed before the bones exist
local tb = out.big_bones[1]
local sy = (tb[3] + tb[5]) * 0.5
C.reset()
global.cave_yellow = gm.struct({ x = (tb[2] + tb[4]) * 0.5, y = sy, top = sy - 1500, bottom = sy + 1500, cluster = "synthetic" })
out.rects_on_bones = C.rects()
out.big_bones_on_bones = C.boxes(world_big_bones(global, global))
out.zones_on_bones = C.boxes(world_clear_zones(global, global))
return out
