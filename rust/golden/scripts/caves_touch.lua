-- Cave and relic rewards on the Task 2 Exploration fixture (loaded through load_game, stones
-- neutralized per R19): touches across every cave and relic, the perks, popups, wraps, burst
-- and yellow timer cut they leave, wrap hits, hover tips, perk_debug_next and the perk
-- multipliers. loc() is replaced by the identity so texts come out as their keys (myco_core
-- keeps keys; the app localizes). For tests/caves.rs. Run: tools/golden.sh caves_touch
local gm = require("gm")
local C = dofile("rust/golden/scripts/caves_lib.lua")
local L = C.L
C.setup()
loc = function(self, other, key) return key end
local tips = {}
mycelium_extreme_tip = function(self, other, dir)
  local t = tips[dir]
  if t == nil then return gm.undefined end
  return gm.struct({ x = t, y = 600 })
end

local f = assert(io.open("rust/crates/myco_core/tests/fixtures/exploration.json", "rb"))
local text = f:read("*a"):gsub("%z+$", "")
f:close()
local slot = 4
save_write_text(global, global, save_path(global, global, slot), text)
assert(load_game(global, global, slot), "load_game failed")
global.title_open, global.intro_active, global.pause_open = false, false, false

-- mushrooms with timers for the yellow perk's cut
global.mushrooms = gm.array(gm.struct({ timer = 100 }), gm.struct({ timer = 37.5 }), gm.struct({ timer = 0 }))

local out = {}
out.rects = C.rects()
out.obstacles = L.plain(global.world.obstacles)
-- the yellow cave appears (no mycelium tips)
global.cave_yellow = cave_yellow_pick(global, global)
out.rects_yellow = C.rects()

local function keys(s)
  local names = gm.variable_struct_get_names(s)
  local t = {}
  for i = 0, gm.array_length(names) - 1 do t[#t + 1] = names[i] end
  table.sort(t)
  return t
end
local function state()
  local wraps = {}
  for _, k in ipairs(keys(global.cave_wrap)) do
    local wr = gm.variable_struct_get(global.cave_wrap, k)
    wraps[#wraps + 1] = { k, wr.from_x, wr.from_y }
  end
  local pops = {}
  for i = 0, gm.array_length(global.cave_popups) - 1 do
    local p = global.cave_popups[i]
    pops[#pops + 1] = { p.text, p.x, p.y }
  end
  local timers = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do timers[#timers + 1] = global.mushrooms[i].timer end
  local burst = "false"
  if gm.is_struct(global.cave_burst_pending) then
    burst = { global.cave_burst_pending.x, global.cave_burst_pending.y }
  end
  return {
    taken = keys(global.cave_taken), perks = keys(global.cave_perks), relics = keys(global.relic_taken),
    flash = keys(global.cave_flash), wraps = wraps, popups = pops, timers = timers, burst = burst,
    time_mult = perk_time_mult(global, global), cost_mult = perk_cost_mult(global, global),
  }
end

-- touches: a row of points through each cave (and past it), then each relic
out.touches = {}
local function sweep(fn, r)
  local y = (r.y1 + r.y2) * 0.5 + 0.5
  for x = r.x1 - 60, r.x2 + 60, 11 do
    fn(global, global, x, y)
    out.touches[#out.touches + 1] = { x, y, #keys(global.cave_taken), #keys(global.relic_taken) }
  end
end
-- a miss far away first
caves_touch_at(global, global, 123456, 7890)
relics_touch_at(global, global, 123456, 7890)
for _, r in ipairs(out.rects_yellow) do sweep(caves_touch_at, r) end
out.after_caves = state()
local rs = world_relics(global, global)
out.relic_rects = {}
for i = 0, gm.array_length(rs) - 1 do
  local r = rs[i]
  out.relic_rects[#out.relic_rects + 1] = { r.id, r.x1, r.y1, r.x2, r.y2 }
  sweep(relics_touch_at, r)
end
out.after_relics = state()
-- second touch of a taken cave changes nothing
local r0 = out.rects_yellow[1]
caves_touch_at(global, global, (r0.x1 + r0.x2) * 0.5, (r0.y1 + r0.y2) * 0.5)
out.after_again = state()

-- relic_wrapped: fresh wraps are still wrapping; old ones are done
out.relic_wrapped = {}
for _, id in ipairs({ "relic_t_rex", "relic_ufo", "t_rex", "ufo", "nope" }) do
  out.relic_wrapped[#out.relic_wrapped + 1] = { id, relic_wrapped(global, global, id) and 1 or 0 }
end
for _, k in ipairs(keys(global.cave_wrap)) do gm.variable_struct_get(global.cave_wrap, k).born = -1e9 end
for _, id in ipairs({ "relic_t_rex", "relic_ufo", "t_rex", "ufo", "nope" }) do
  out.relic_wrapped[#out.relic_wrapped + 1] = { id, relic_wrapped(global, global, id) and 1 or 0 }
end

-- wrap hits around the taken caves (wraps finished)
out.wrap_hit = {}
for _, r in ipairs(out.rects_yellow) do
  for _, p in ipairs(C.grid(r.x1 - 70, r.y1 - 70, r.x2 + 70, r.y2 + 70, 41, 37)) do
    local h = caves_wrap_hit(global, global, p[1], p[2], 40)
    out.wrap_hit[#out.wrap_hit + 1] = { p[1], p[2], h.hit and 1 or 0, h.x, h.y, h.dist }
  end
end

-- hover tips: icons row at (24, 64), view at the first cave
out.hover = {}
local vx, vy = r0.x1 - 300, r0.y1 - 200
for _, p in ipairs({ { 30, 70 }, { 80, 70 }, { 130, 70 }, { 180, 70 }, { 230, 70 }, { 280, 70 }, { 30, 120 }, { 10, 10 } }) do
  out.hover[#out.hover + 1] = { p[1], p[2], cave_hover_tip(global, global, p[1], p[2], 24, 64, vx, vy) }
end
for _, p in ipairs(C.grid(250, 150, 860, 520, 31, 29)) do
  out.hover[#out.hover + 1] = { p[1], p[2], cave_hover_tip(global, global, p[1], p[2], 24, 64, vx, vy) }
end

-- perk_debug_next from no perks: takes the caves in cave_defs order, then blue ones
global.cave_perks = gm.struct({})
global.cave_taken = gm.struct({})
global.cave_popups = gm.array()
global.mushrooms = gm.array(gm.struct({ timer = 90 }))
out.debug_next = {}
for _ = 1, 8 do out.debug_next[#out.debug_next + 1] = perk_debug_next(global, global) end
out.after_debug = state()

-- re-taking caves whose perk is already held: no second yellow cut, no second purple glow
global.cave_taken = gm.struct({})
global.mushrooms = gm.array(gm.struct({ timer = 80 }))
for _, r in ipairs(out.rects_yellow) do
  if r.def == "yellow" or r.def == "purple" then
    caves_touch_at(global, global, (r.x1 + r.x2) * 0.5 + 0.25, (r.y1 + r.y2) * 0.5 + 0.5)
  end
end
out.after_retake = state()

-- where exactly a touch starts to count: vertical and horizontal scans into every cave and
-- relic from outside, each point on an untaken state
out.touch_probe = {}
local function probe_at(fn, x, y)
  global.cave_taken = gm.struct({})
  global.relic_taken = gm.struct({})
  fn(global, global, x, y)
  local n = #keys(global.cave_taken) + #keys(global.relic_taken)
  out.touch_probe[#out.touch_probe + 1] = { x, y, n }
end
local function scan(fn, r)
  local mx, my = (r[1] + r[3]) * 0.5 + 0.25, (r[2] + r[4]) * 0.5 + 0.5
  for d = -35, 70, 1.5 do
    probe_at(fn, mx, r[2] + d)
    probe_at(fn, r[1] + d, my)
    probe_at(fn, mx + 37, r[4] - d)
  end
end
for _, r in ipairs(out.rects_yellow) do scan(caves_touch_at, { r.x1, r.y1, r.x2, r.y2 }) end
for _, r in ipairs(out.relic_rects) do scan(relics_touch_at, { r[2], r[3], r[4], r[5] }) end

-- perk_has truthiness and the multipliers in both modes
out.perk_has = {}
for _, v in ipairs({ { "a", true }, { "b", false }, { "c", 1 }, { "d", 0 }, { "e", 0.7 }, { "f", 0.3 } }) do
  gm.variable_struct_set(global.cave_perks, v[1], v[2])
end
for _, id in ipairs({ "a", "b", "c", "d", "e", "f", "zz" }) do
  out.perk_has[#out.perk_has + 1] = { id, perk_has(global, global, id) and 1 or 0 }
end
out.mult = {}
for _, mode in ipairs({ "exploration", "survival" }) do
  game_mode_apply(global, global, mode)
  for _, set in ipairs({ {}, { "green" }, { "yellow" }, { "green", "yellow" } }) do
    global.cave_perks = gm.struct({})
    for _, id in ipairs(set) do gm.variable_struct_set(global.cave_perks, id, true) end
    out.mult[#out.mult + 1] = { mode, table.concat(set, "+"), perk_time_mult(global, global), perk_cost_mult(global, global) }
  end
end
game_mode_apply(global, global, "exploration")
return out
