-- Mycelium (obj_mycelium_controller) on the Task 2 Exploration fixture, loaded through load_game
-- with everything live: shot parameters per upgrade level and mode, playability probes, five
-- seeded shots, a mouse-driven Step_0 session (aim, LMB shot, growth scan, harvest, length),
-- quick nodes (alt+click path, myc_node_quick), node costs, a node burst, a digger burst, spores,
-- placement checks and the save view of growths/knots. For tests/mycelium.rs.
-- Determinism: gm.get_timer is frozen (time budgets never run out: an infinitely fast machine,
-- as in myco_core), gm.current_time = floor(frame * 1000 / 60) like Sim::now_ms, mouse and
-- keyboard are scripted, the camera view is fixed.
-- Run: MYCO_IDENTITY=MycelariumTest-t8 tools/golden.sh mycelium
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.preload()
loc = function(self, other, key) return key end

local f = assert(io.open("rust/crates/myco_core/tests/fixtures/exploration.json", "rb"))
local text = f:read("*a"):gsub("%z+$", "")
f:close()
local slot = 4
save_write_text(global, global, save_path(global, global, slot), text)
assert(load_game(global, global, slot), "load_game failed")
global.title_open, global.intro_active, global.pause_open = false, false, false
global.menu_open, global.debug_open, global.logo_open = false, false, false
global.ui_capture_mouse = false
global.right_click_used = false

local mc = gm.instance_find(obj_mycelium_controller, 0)
local cam = gm.view_camera[0]
local VIEW = { x = -960, y = 0, w = 1920, h = 1080 }
gm.camera_set_view_pos(cam, VIEW.x, VIEW.y)
gm.camera_set_view_size(cam, VIEW.w, VIEW.h)

gm.get_timer = function() return 0 end
local mouse = { x = 0, y = 0, down = false, pressed = false, alt = false }
gm.mouse_check_button = function(b) return b == gm.mb_left and mouse.down end
gm.mouse_check_button_pressed = function(b) return b == gm.mb_left and mouse.pressed end
gm.keyboard_check = function(k) return k == gm.vk_alt and mouse.alt end
gm.keyboard_check_pressed = function(k) return false end
gm.device_mouse_x_to_gui = function() return -100000 end
gm.device_mouse_y_to_gui = function() return -100000 end

local frame = 0
local function now() gm.current_time = math.floor(frame * 1000 / 60) end
now()

local out = { view = VIEW }

local function wt() return water_total(global, global) end
local function pts(list)
  local o = {}
  for i = 0, gm.array_length(list) - 1 do o[#o + 1] = { list[i].x, list[i].y } end
  return o
end
local function cells(list)
  if gm.is_undefined(list) then return "undefined" end
  local o = {}
  for i = 0, gm.array_length(list) - 1 do o[#o + 1] = { list[i].x, list[i].y, list[i].w } end
  return o
end
local function num0(v)
  if gm.is_undefined(v) then return 0 end
  return v
end
local function branch(b)
  return {
    points = pts(b.points), cells = cells(b.cells), start = b.start, width = b.width,
    shoved = b.shoved and 1 or 0, swept = b.swept, bx1 = b.bx1, by1 = b.by1, bx2 = b.bx2, by2 = b.by2,
    scan_done = num0(gm.variable_struct_get(b, "scan_done")),
    scan_done_len = num0(gm.variable_struct_get(b, "scan_done_len")),
  }
end
local function growth(g, with_cells)
  local brs = {}
  for i = 0, gm.array_length(g.branches) - 1 do
    local b = branch(g.branches[i])
    if not with_cells then b.cells = nil end
    brs[#brs + 1] = b
  end
  return {
    branches = brs, duration = g.duration, finish_age = g.finish_age,
    finished = g.finished and 1 or 0, budding = g.budding and 1 or 0,
    no_length = gm.variable_struct_exists(g, "no_length") and 1 or 0,
    scanned_length = g.scanned_length, bx1 = g.bx1, by1 = g.by1, bx2 = g.bx2, by2 = g.by2,
    sound = gm.variable_struct_exists(g, "sound") and 1 or 0,
  }
end
local function knots()
  local o = {}
  for i = 0, gm.array_length(mc.myc_knots) - 1 do
    local k = mc.myc_knots[i]
    local e = { x = k.x, y = k.y }
    if gm.variable_struct_exists(k, "shroom") then
      e.kind = k.shroom.kind
      e.cell = k.shroom.cell
      e.born = k.born
    end
    o[#o + 1] = e
  end
  return o
end
local function keys(s)
  local names = gm.variable_struct_get_names(s)
  local t = {}
  for i = 0, gm.array_length(names) - 1 do t[#t + 1] = names[i] end
  table.sort(t)
  return t
end
local function tut_events()
  local o = {}
  for _, k in ipairs(keys(global.tut.events)) do
    local e = gm.variable_struct_get(global.tut.events, k)
    o[#o + 1] = { k, e.x, e.y }
  end
  return o
end
local function stats()
  local o = {}
  for _, k in ipairs(keys(global.tasks.stats)) do o[#o + 1] = { k, gm.variable_struct_get(global.tasks.stats, k) } end
  return o
end
local function probe() return gm.random(1.0) end
local function set_upg(id, lvl) gm.variable_struct_set(global.upgrade_levels, id, lvl) end
local function upg(id)
  local v = gm.variable_struct_get(global.upgrade_levels, id)
  if gm.is_undefined(v) then return 0 end
  return v
end

-- after load_game: the rebuilt growths, grid and knots
out.loaded = { growths = {}, knots = knots(), far_max = mc.myc_far_max, x = mc.x, y = mc.y,
  unbaked = gm.array_length(mc.myc_unbaked), active = gm.array_length(mc.myc_active) }
for i = 0, gm.array_length(mc.myc_growths) - 1 do
  local g = mc.myc_growths[i]
  local e = growth(g, false)
  e.age = g.age
  out.loaded.growths[#out.loaded.growths + 1] = e
end
out.blocking = ui_blocking(global, global) and 1 or 0
out.consts = {
  surface_y = mc.myc_surface_y, tile = mc.myc_tile, sample_step = mc.myc_sample_step,
  min_growth_len = mc.myc_min_growth_len, growth_duration = mc.myc_growth_duration,
  food_pickup_radius = mc.myc_food_pickup_radius, knot_radius = mc.myc_knot_radius,
  pick_radius = mc.myc_mycelium_pick_radius,
}

-- ---------------------------------------------------------------------------------------------
-- parameters per mode, upgrade level and rain
local saved_levels = {}
for _, id in ipairs({ "shot_range", "digger_length", "shot_width", "water_efficiency", "node_efficiency",
  "budding_chance", "soil_penetration", "spore_quality" }) do saved_levels[id] = upg(id) end
out.params = {}
for _, mode in ipairs({ "exploration", "survival" }) do
  game_mode_apply(global, global, mode)
  for lvl = 0, 6 do
    for id, _ in pairs(saved_levels) do set_upg(id, lvl) end
    for _, rain in ipairs({ 0, 0.5 }) do
      global.weather.intensity = rain
      out.params[#out.params + 1] = {
        mode = mode, lvl = lvl, rain = rain,
        shot_range = myc_shot_range(global, global), shot_range_max = myc_shot_range_max(global, global),
        digger_range = myc_digger_range(global, global), shot_spread = myc_shot_spread(global, global),
        water_cost = myc_shot_water_cost(global, global), water_rate = myc_shot_water_rate(global, global),
        node_cost = myc_node_food_cost(global, global), budding = myc_budding_chance(global, global),
        pen = myc_soil_penetration(global, global), root_chance = mc.myc_spore_root_chance(mc, mc),
      }
    end
  end
end
game_mode_apply(global, global, "exploration")
global.weather.intensity = 0
for id, v in pairs(saved_levels) do set_upg(id, v) end

out.cm = {}
for _, px in ipairs({ 0, 1, 37.5, 526.4976212684751, 12345.678, 1e6 }) do
  global.mycelium_length_px = px
  global.units_imperial = false
  local t1 = length_text(global, global, mycelium_cm(global, global))
  global.units_imperial = true
  local t2 = length_text(global, global, mycelium_cm(global, global))
  out.cm[#out.cm + 1] = { px, pixels_to_cm(global, global, px), mycelium_cm(global, global), t1, t2 }
end
global.units_imperial = false
global.mycelium_length_px = 526.4976212684751
local la = {}
for _, d in ipairs({ 10, -5, 0, 2.5 }) do
  mycelium_add_length(global, global, d)
  la[#la + 1] = global.mycelium_length_px
end
out.add_length = la
global.mycelium_length_px = 526.4976212684751

-- ---------------------------------------------------------------------------------------------
-- playability probes
local P = {}
P.points = {}
for y = 430, 2500, 47 do
  for x = -420, 420, 61 do
    P.points[#P.points + 1] = {
      x, y,
      mc.myc_is_blocked_point(mc, mc, x, y) and 1 or 0,
      mc.myc_is_playable_point(mc, mc, x, y, false) and 1 or 0,
      mc.myc_is_playable_point(mc, mc, x, y, true) and 1 or 0,
      mc.myc_is_playable_clear_point(mc, mc, x, y, 1, false) and 1 or 0,
      mc.myc_is_playable_clear_point(mc, mc, x, y, 2, false) and 1 or 0,
      mc.myc_is_playable_clear_point(mc, mc, x, y, 2, true) and 1 or 0,
    }
  end
end
-- segments with r = 1 (myc_place_check passes the GML true as the radius; baff3b6) and r = 2
P.segments = {}
local seg_from = { { 0, 480 }, { -44.58447299418679, 596.7521495073507 }, { 150, 900 }, { -300, 1400 }, { 60, 2200 } }
for _, a in ipairs(seg_from) do
  for dir = 0, 359, 30 do
    for _, len in ipairs({ 0, 40, 260 }) do
      local bx = a[1] + gm.lengthdir_x(len, dir)
      local by = a[2] + gm.lengthdir_y(len, dir)
      P.segments[#P.segments + 1] = {
        a[1], a[2], bx, by,
        mc.myc_segment_is_playable(mc, mc, a[1], a[2], bx, by, true) and 1 or 0,
        mc.myc_segment_is_playable(mc, mc, a[1], a[2], bx, by, 1, false) and 1 or 0,
        mc.myc_segment_is_playable(mc, mc, a[1], a[2], bx, by, 2, false) and 1 or 0,
        mc.myc_segment_is_playable(mc, mc, a[1], a[2], bx, by, 2, true) and 1 or 0,
      }
    end
  end
end
-- allowed length, water and food cost of shots
P.measure = {}
local from = { { 0, 480 + 48 }, { -44.58447299418679, 596.7521495073507 }, { 150, 1800 }, { -200, 2600 }, { 40, 4200 } }
for _, a in ipairs(from) do
  for dir = 0, 359, 15 do
    for _, maxlen in ipairs({ 120, 300, 900 }) do
      local l0 = mc.myc_measure_allowed_len_from(mc, mc, a[1], a[2], dir, maxlen)
      local l1 = mc.myc_measure_allowed_len_from(mc, mc, a[1], a[2], dir, maxlen, true)
      P.measure[#P.measure + 1] = {
        a[1], a[2], dir, maxlen, l0, l1,
        mc.myc_shot_cost_for(mc, mc, a[1], a[2], dir, l0),
        mc.myc_shot_food_for(mc, mc, a[1], a[2], dir, l0),
        mc.myc_shot_cost_for(mc, mc, a[1], a[2], dir, maxlen),
        mc.myc_shot_food_for(mc, mc, a[1], a[2], dir, maxlen),
      }
    end
  end
end
out.play = P

-- ---------------------------------------------------------------------------------------------
-- five seeded shots straight through myc_add_growth_from (state restored after each)
local function snapshot_growths()
  local n = gm.array_length(mc.myc_growths)
  return n
end
out.shots = {}
local shot_cases = {
  { seed = 101, x = 0, y = 480, dir = 270, upg = {}, opts = nil },
  { seed = 102, x = -44.58447299418679, y = 596.7521495073507, dir = 200, upg = { shot_range = 3, shot_width = 2 }, opts = nil },
  { seed = 103, x = 0, y = 480, dir = 315, upg = { shot_range = 6, budding_chance = 4 }, opts = nil },
  { seed = 104, x = 150, y = 1800, dir = 250, upg = { soil_penetration = 2 }, opts = { free = true, pierce = true, width_mult = 1.4, kinks_per_base = 2 } },
  { seed = 105, x = -44.58447299418679, y = 596.7521495073507, dir = 120, upg = { shot_width = 5 }, opts = { free = true, silent = true, instant = true } },
}
for _, c in ipairs(shot_cases) do
  for id, lvl in pairs(c.upg) do set_upg(id, lvl) end
  gm.random_set_seed(c.seed)
  local range = myc_shot_range(global, global)
  local len = mc.myc_measure_allowed_len_from(mc, mc, c.x, c.y, c.dir, range, c.opts and c.opts.pierce or false)
  local food0, water0 = global.food, wt()
  local n0 = snapshot_growths()
  local opts = gm.undefined
  if c.opts then opts = gm.struct(c.opts) end
  local ok = mc.myc_add_growth_from(mc, mc, c.x, c.y, c.dir, len, opts)
  local rec = { seed = c.seed, len = len, ok = ok and 1 or 0, food0 = food0, water0 = water0,
    food = global.food, water = wt(), water_free = global.water_free }
  if gm.array_length(mc.myc_growths) > n0 then
    rec.growth = growth(mc.myc_growths[gm.array_length(mc.myc_growths) - 1], true)
  end
  rec.probe = probe()
  out.shots[#out.shots + 1] = rec
  for id, _ in pairs(c.upg) do set_upg(id, saved_levels[id] or 0) end
end
-- undo the shots: back to the loaded mycelium (load the fixture again)
assert(load_game(global, global, slot), "reload failed")
global.title_open, global.intro_active, global.pause_open = false, false, false
mc = gm.instance_find(obj_mycelium_controller, 0)
gm.camera_set_view_pos(cam, VIEW.x, VIEW.y)
gm.camera_set_view_size(cam, VIEW.w, VIEW.h)

-- ---------------------------------------------------------------------------------------------
-- a mouse-driven session through Step_0
local function step(inp)
  now()
  mouse.x, mouse.y = inp[1], inp[2]
  gm.mouse_x, gm.mouse_y = inp[1], inp[2]
  mouse.down, mouse.pressed, mouse.alt = inp[3] == 1, inp[4] == 1, inp[5] == 1
  gml_Object_obj_mycelium_controller_Step_0(mc, mc)
  frame = frame + 1
end
local function unfinished()
  local n = 0
  for i = 0, gm.array_length(mc.myc_growths) - 1 do
    if not mc.myc_growths[i].finished then n = n + 1 end
  end
  return n
end
local function digest(tag)
  return {
    tag, global.food, wt(), global.mycelium_length_px, gm.array_length(mc.myc_growths), unfinished(),
    gm.array_length(mc.myc_knots), mc.myc_dragging and 1 or 0, mc.myc_has_aim and 1 or 0,
    mc.myc_aim_dir, mc.myc_aim_len, mc.myc_aim_reach, mc.myc_aim_cost, mc.myc_aim_food,
    gm.array_length(mc.myc_spores),
  }
end
-- the input script, shared with the Rust test: {x, y, down, pressed, alt}
local inputs = {}
local function add(n, x, y, down, pressed, alt, list)
  list = list or inputs
  for _ = 1, n do list[#list + 1] = { x, y, down, pressed, alt or 0 } end
end
add(2, 300, 300, 0, 0)
add(1, 2, 482, 1, 1)                -- press on the origin knot: aim starts
for k = 1, 12 do add(1, 2 + k * 9, 482 + k * 22, 1, 0) end  -- drag
add(1, 110, 746, 0, 0)              -- release: shot
add(140, 110, 746, 0, 0)            -- the growth runs
add(1, -44, 597, 1, 1)              -- press on the second knot
for k = 1, 10 do add(1, -44 - k * 21, 597 + k * 15, 1, 0) end
add(1, -254, 747, 0, 0)             -- release
add(160, -254, 747, 0, 0)
add(1, 500, 300, 1, 1)              -- a click on nothing (clears prompts)
add(1, 500, 300, 0, 0)
out.session_inputs = inputs
gm.random_set_seed(4242)
local D = {}
for i, inp in ipairs(inputs) do
  step(inp)
  D[#D + 1] = digest(i)
end
out.session = { digests = D, growths = {}, knots = knots(), cam_box = { global.cam_box_x1, global.cam_box_y1,
  global.cam_box_x2, global.cam_box_y2 }, far_max = mc.myc_far_max, events = tut_events(), probe = probe() }
for i = 0, gm.array_length(mc.myc_growths) - 1 do
  out.session.growths[#out.session.growths + 1] = growth(mc.myc_growths[i], false)
end

-- ---------------------------------------------------------------------------------------------
-- quick nodes (myc_node_quick; the Rust port's right click) along the session's growths
gm.random_set_seed(777)
local Q = {}
local qpts = { { 30, 560 }, { 30, 560 }, { 31, 562 }, { 60, 640 }, { 500, 300 }, { -100, 640 }, { -150, 680 },
  { -150, 690 }, { 0, 482 }, { 85, 700 } }
for _, p in ipairs(qpts) do
  local hit = mc.myc_get_mycelium_hit(mc, mc, p[1], p[2], mc.myc_mycelium_pick_radius)
  local ok = mc.myc_node_quick(mc, mc, p[1], p[2])
  Q[#Q + 1] = { p[1], p[2], ok and 1 or 0, hit.hit and 1 or 0, hit.x, hit.y, global.food, gm.array_length(mc.myc_knots) }
end
global.food = 3
local ok = mc.myc_node_quick(mc, mc, 110, 746)
Q[#Q + 1] = { 110, 746, ok and 1 or 0, 0, 0, 0, global.food, gm.array_length(mc.myc_knots) }
global.food = 500
out.quick = { rows = Q, knots = knots(), events = tut_events() }

-- a second session: a long shot (shot_range 4) from the quick node at (88.7, 698.4) through food
-- and a lake (harvest: food, water, reach), plus an alt+click quick node
set_upg("shot_range", 4)
local inputs2 = {}
add(1, 89, 699, 1, 1, 0, inputs2)
for k = 1, 14 do add(1, 89 + k * 7, 699 + k * 22, 1, 0, 0, inputs2) end
add(1, 187, 1007, 0, 0, 0, inputs2)
add(150, 187, 1007, 0, 0, 0, inputs2)
add(1, 120, 790, 0, 1, 1, inputs2)   -- alt+click on the growth: quick node
add(1, 120, 790, 0, 0, 0, inputs2)
out.session2_inputs = inputs2
gm.random_set_seed(4343)
local food_s2, water_s2 = global.food, wt()
local D2 = {}
for i, inp in ipairs(inputs2) do
  step(inp)
  D2[#D2 + 1] = digest(i)
end
out.session2 = { food0 = food_s2, water0 = water_s2, digests = D2, growths = {}, knots = knots(),
  cam_box = { global.cam_box_x1, global.cam_box_y1, global.cam_box_x2, global.cam_box_y2 },
  water_free = global.water_free, sources = gm.array_length(global.water_sources), stats = stats(),
  events = tut_events(), probe = probe() }
for i = 0, gm.array_length(mc.myc_growths) - 1 do
  out.session2.growths[#out.session2.growths + 1] = growth(mc.myc_growths[i], false)
end
set_upg("shot_range", saved_levels.shot_range or 0)

-- node cost per node_efficiency level, both modes
out.node_cost = {}
for _, mode in ipairs({ "exploration", "survival" }) do
  game_mode_apply(global, global, mode)
  for lvl = 0, 12 do
    set_upg("node_efficiency", lvl)
    out.node_cost[#out.node_cost + 1] = { mode, lvl, myc_node_food_cost(global, global) }
  end
end
game_mode_apply(global, global, "exploration")
set_upg("node_efficiency", saved_levels.node_efficiency or 0)

-- ---------------------------------------------------------------------------------------------
-- node burst (the blue cave): knots sorted by distance, off-view ones instant, then shown ones
gm.random_set_seed(9001)
mc.myc_node_burst(mc, mc, 40, 900)
local B = { queue = gm.array_length(mc.myc_burst_queue) }
B.digests = {}
for k = 1, 40 do
  step({ -400, 200, 0, 0, 0 })
  B.digests[#B.digests + 1] = digest(k)
end
B.growths = {}
for i = 0, gm.array_length(mc.myc_growths) - 1 do B.growths[#B.growths + 1] = growth(mc.myc_growths[i], false) end
B.knots = knots()
B.probe = probe()
out.burst = B

-- a digger burst
gm.random_set_seed(31337)
set_upg("digger_length", 2)
mc.myc_digger_burst(mc, mc, 120, 480)
local G = { queue = gm.array_length(mc.myc_digger_queue), digests = {} }
for k = 1, 30 do
  step({ -400, 200, 0, 0, 0 })
  G.digests[#G.digests + 1] = digest(k)
end
G.growths = {}
for i = 0, gm.array_length(mc.myc_growths) - 1 do G.growths[#G.growths + 1] = growth(mc.myc_growths[i], false) end
G.probe = probe()
out.digger = G

-- ---------------------------------------------------------------------------------------------
-- spores: released from the origin, shown ones, rays, then 1000 frames of falling and rooting
gm.random_set_seed(2468)
global.food = 500
set_upg("spore_count", 2)
set_upg("spore_quality", 3)
local S = {}
S.ok = mc.myc_spawn_spores_from(mc, mc, 0, 480) and 1 or 0
mc.myc_spawn_spores_show(mc, mc, 300, 420, 5, 1.3)
local rays = gm.array(gm.struct({ a = 80, power = 2 }), gm.struct({ a = 100, power = 1.5 }), gm.struct({ a = 60, power = 3 }))
mc.myc_spawn_spores_rays(mc, mc, -300, 400, 7, rays, 1.7)
local function spores()
  local o = {}
  for i = 0, gm.array_length(mc.myc_spores) - 1 do
    local s = mc.myc_spores[i]
    o[#o + 1] = { s.x, s.y, s.vx, s.vy, s.weight, s.age, s.life, s.phase,
      gm.variable_struct_get(s, "show") == true and 1 or 0, gm.variable_struct_get(s, "ray") == true and 1 or 0 }
  end
  return o
end
S.start = spores()
S.food = global.food
S.snaps = {}
for k = 1, 1000 do
  step({ -400, 200, 0, 0, 0 })
  if k % 100 == 0 then S.snaps[#S.snaps + 1] = { k, spores(), knots() } end
end
S.stats = stats()
S.cam_box = { global.cam_box_x1, global.cam_box_y1, global.cam_box_x2, global.cam_box_y2 }
S.probe = probe()
out.spores = S

-- ---------------------------------------------------------------------------------------------
-- placement checks (unlocked types, enough food) and the build menu
for _, id in ipairs({ "unlock_spore", "unlock_predator", "unlock_repeller", "unlock_tree", "unlock_digger",
  "unlock_stinky", "unlock_experience" }) do set_upg(id, 1) end
global.food = 1000
gm.random_set_seed(1357)
local PL = {}
for t = 0, 6 do
  for x = -900, 900, 75 do
    local p = mc.myc_place_check(mc, mc, t, x, 470)
    PL[#PL + 1] = { t, x, p.ok and 1 or 0, p.x, p.y, p.face, p.root_x, p.from_x, p.from_y,
      mc.myc_can_build_type(mc, mc, t, x, 480) and 1 or 0 }
  end
end
out.place = PL
out.place_probe = probe()
local SB = {}
for _, p in ipairs({ { 0, 480 }, { 10, 470 }, { 30, 520 }, { -44, 500 }, { 200, 480 }, { 0, 700 } }) do
  local b = mc.myc_get_surface_build_at(mc, mc, p[1], p[2])
  SB[#SB + 1] = { p[1], p[2], b.hit and 1 or 0, b.x, b.y, mc.myc_surface_buildable_at(mc, mc, p[1], p[2]) and 1 or 0,
    mc.myc_has_mushroom_near(mc, mc, p[1], p[2], 50) and 1 or 0, mc.myc_surface_block_reason(mc, mc, p[1], p[2]) }
end
out.surface = SB
local MENU = {}
for _, p in ipairs({ { 30, 560 }, { 500, 300 }, { 0, 482 }, { 85, 700 }, { -200, 718 }, { 150, 900 }, { 160, 1000 } }) do
  local ok = mc.myc_open_build_menu_at(mc, mc, p[1], p[2])
  local opts = {}
  for i = 0, gm.array_length(mc.myc_build_options) - 1 do
    local o = mc.myc_build_options[i]
    opts[#opts + 1] = { o.kind, o.type, mc.myc_can_build_option(mc, mc, o) and 1 or 0, mc.myc_build_option_label(mc, mc, o) }
  end
  MENU[#MENU + 1] = { p[1], p[2], ok and 1 or 0, mc.myc_build_node_x, mc.myc_build_node_y, mc.myc_build_surface_x,
    mc.myc_build_surface_y, mc.myc_build_surface_ok and 1 or 0, mc.myc_mushroom_prompt and 1 or 0, mc.myc_build_hint, opts }
end
out.menu = MENU

-- ---------------------------------------------------------------------------------------------
-- pure helpers
local ob = mc.myc_origin_box(mc, mc)
out.origin_box = { ob.x1, ob.y1, ob.x2, ob.y2 }
local SR = {}
for kind = 0, 1 do
  for cell = 0, 8 do
    local r = mc.myc_smallshroom_rect(mc, mc, gm.struct({ kind = kind, cell = cell }))
    SR[#SR + 1] = { kind, cell, r.left, r.top, r.width, r.height }
  end
end
out.smallshroom = SR
local tips = {}
for _, d in ipairs({ -1, 1 }) do
  local t = mycelium_extreme_tip(global, global, d)
  tips[#tips + 1] = { d, t.x, t.y }
end
out.tips = tips
local saved = {}
for i = 0, gm.array_length(mc.myc_growths) - 1 do
  local g = mc.myc_growths[i]
  local brs = {}
  for q = 0, gm.array_length(g.branches) - 1 do
    local b = g.branches[q]
    brs[#brs + 1] = { points = pts(b.points), start = b.start, width = b.width, shoved = b.shoved and 1 or 0, swept = b.swept }
  end
  saved[#saved + 1] = brs
end
out.saved = { growths = saved, knots = knots(), length_px = global.mycelium_length_px, food = global.food }
return out
