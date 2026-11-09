-- Mushrooms (scr_life mushroom functions) on the Task 2 Exploration fixture, loaded through
-- load_game. For tests/mushrooms.rs:
-- 1. sprite metadata the logic measures (hit boxes, cells, hedgehog/boar frame widths);
-- 2. type catalogue and costs over perks, upgrade levels and experience-mushroom counts; the
--    period/price helpers over upgrade levels and the yellow perk;
-- 3. mushroom_add rolls (variant, flies, timers, grow sound, stats) for a seeded build of every
--    type, then hosts, timers and consumption set up and the whole game saved to
--    tests/fixtures/mushrooms.json and loaded back (load_game) — the fixture with mushrooms;
-- 4. 600 frames of mushrooms_step + drops_step + repeller_clouds_step from that fixture with a
--    hedgehog (frame 100) and a boar (frame 300) walking into repellers: per-frame food, water,
--    points, and stinky pools, clouds, drops, hosts, mushrooms along the way and at the end;
-- 5. pure helpers: hit boxes, progress bars, stinky shape/funnel/beam/guards, shade table.
-- Determinism: gm.current_time = floor(frame * 1000 / 60) like Sim::now_ms, camera view fixed.
-- Run: MYCO_IDENTITY=MycelariumTest-t9 tools/golden.sh mushrooms
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.preload()
loc = function(self, other, key) return key end

local function read(path)
  local f = assert(io.open(path, "rb"))
  local t = f:read("*a")
  f:close()
  return t
end

local text = read("rust/crates/myco_core/tests/fixtures/exploration.json"):gsub("%z+$", "")
local slot = 4
save_write_text(global, global, save_path(global, global, slot), text)
assert(load_game(global, global, slot), "load_game failed")
global.title_open, global.intro_active, global.pause_open = false, false, false
global.menu_open, global.debug_open, global.logo_open = false, false, false

local cam = gm.view_camera[0]
local VIEW = { x = -960, y = 0, w = 1920, h = 1080 }
local function set_view()
  gm.camera_set_view_pos(cam, VIEW.x, VIEW.y)
  gm.camera_set_view_size(cam, VIEW.w, VIEW.h)
end
set_view()
gm.get_timer = function() return 0 end

local frame = 0
local function now() gm.current_time = math.floor(frame * 1000 / 60) end
now()

local out = { view = VIEW }
local SY = wld(global, global, "surface_height_px")
local function U(v) if gm.is_undefined(v) then return "undefined" end return v end
local function B(v) return gm.bool(v) and 1 or 0 end
local function keys(s)
  local names = gm.variable_struct_get_names(s)
  local t = {}
  for i = 0, gm.array_length(names) - 1 do t[#t + 1] = names[i] end
  table.sort(t)
  return t
end
local function stats()
  local o = {}
  for _, k in ipairs(keys(global.tasks.stats)) do o[#o + 1] = { k, gm.variable_struct_get(global.tasks.stats, k) } end
  return o
end
local function probe() return gm.random(1.0) end
local function set_upg(id, lvl) gm.variable_struct_set(global.upgrade_levels, id, lvl) end
local function set_perk(id, on)
  if on then gm.variable_struct_set(global.cave_perks, id, true)
  elseif gm.variable_struct_exists(global.cave_perks, id) then gm.variable_struct_remove(global.cave_perks, id) end
end

-- 1. sprites ------------------------------------------------------------------------------
local SPR = { "sporeshroom", "predatorshroom", "explosiveshroom", "explosiveshroom_exploded", "treeshroom",
  "diggershroom", "stinkyshroom", "experienceshroom", "consumptorshroom", "hedgehog", "boar" }
local sprites = {}
for _, n in ipairs(SPR) do
  local s = assert(_G[n], n)
  sprites[#sprites + 1] = { n, gm.sprite_get_width(s), gm.sprite_get_height(s), gm.sprite_get_xoffset(s),
    gm.sprite_get_yoffset(s), gm.sprite_get_bbox_left(s), gm.sprite_get_bbox_top(s), gm.sprite_get_bbox_right(s),
    gm.sprite_get_bbox_bottom(s), shroom_sheet_cells(global, global, s), mushroom_cell_width(global, global, s) }
end
out.sprites = sprites
out.frame_w = { hedgehog_frame_w(global, global), boar_frame_w(global, global) }
local types = {}
for t = 0, 7 do
  local cl = {}
  local spr = mushroom_type_sprite(global, global, t)
  for v = 0, 2 do cl[#cl + 1] = mushroom_cell_left(global, global, spr, v) end
  types[#types + 1] = { gm.sprite_get_name(spr), mushroom_type_variants(global, global, t),
    mushroom_type_unlock_id(global, global, t), mushroom_type_name_key(global, global, t), cl }
end
out.types = types

-- 2. costs -----------------------------------------------------------------------------------
local UNLOCKS = { "unlock_spore", "unlock_predator", "unlock_repeller", "unlock_tree", "unlock_digger",
  "unlock_stinky", "unlock_experience", "unlock_parasitism" }
local function cost_rows()
  local rows = {}
  for t = 0, 7 do
    rows[#rows + 1] = { mushroom_type_cost_base(global, global, t), mushroom_type_water_cost_base(global, global, t),
      mushroom_type_cost(global, global, t), mushroom_type_water_cost(global, global, t),
      B(mushroom_type_affordable(global, global, t)), B(mushroom_type_unlocked(global, global, t)) }
  end
  return rows
end
local LEVELED = { "digger_length", "repeller_regen_cost", "repeller_autoregen", "predator_digestion", "predator_yield",
  "tree_drop_rate", "tree_water_yield", "parasitism_speed", "parasitism_yield", "stinky_width" }
local maxl = {}
for _, id in ipairs(LEVELED) do maxl[#maxl + 1] = { id, upg_max_level(global, global, id) } end
out.max_levels = maxl

local function helpers()
  return { repeller_regen_price(global, global), repeller_autoregen_frames(global, global),
    repeller_max_charges(global, global), experience_period(global, global), predator_period(global, global),
    predator_yield(global, global), tree_drop_period(global, global), tree_drop_water(global, global) }
end

local costs = {}
local function cost_phase(tag)
  for _, perk in ipairs({ false, true }) do
    set_perk("green", perk)
    local dmax = upg_max_level(global, global, "digger_length")
    for lvl = 0, dmax do
      set_upg("digger_length", lvl)
      for _, food in ipairs({ 0, 45, 100000 }) do
        global.food = food
        costs[#costs + 1] = { tag = tag, perk = perk and 1 or 0, digger = lvl, food = food, rows = cost_rows() }
      end
    end
  end
  set_perk("green", false)
  set_upg("digger_length", 0)
end
cost_phase("locked")
for _, id in ipairs(UNLOCKS) do set_upg(id, 1) end
cost_phase("unlocked")

local helper_rows = {}
for _, yellow in ipairs({ false, true }) do
  set_perk("yellow", yellow)
  for _, id in ipairs(LEVELED) do
    for lvl = 0, upg_max_level(global, global, id) do
      set_upg(id, lvl)
      local h = helpers()
      helper_rows[#helper_rows + 1] = { yellow and 1 or 0, id, lvl, h }
    end
    set_upg(id, 0)
  end
end
set_perk("yellow", false)
out.helpers = helper_rows

-- 3. building the colony ------------------------------------------------------------------
local LEVELS = { digger_length = 2, repeller_regen_cost = 1, repeller_autoregen = 5, predator_digestion = 1,
  predator_yield = 1, tree_drop_rate = 1, tree_water_yield = 1, parasitism_speed = 1, parasitism_yield = 1,
  stinky_width = 2 }
local lv = {}
for _, id in ipairs(LEVELED) do
  local l = math.min(LEVELS[id], upg_max_level(global, global, id))
  set_upg(id, l)
  lv[#lv + 1] = { id, l }
end
out.levels = lv
global.food = 100000
global.water_free = 5000

local fc = gm.instance_find(obj_forest_controller, 0)
local function find_tree(x)
  local ch = gm.variable_struct_get(fc.surface_chunks, "s" .. math.floor(x / fc.chunk_width))
  for i = 0, gm.array_length(ch.trees) - 1 do
    if ch.trees[i].x == x then return ch.trees[i] end
  end
  error("no tree at " .. x)
end
local function find_stump(x)
  local ch = gm.variable_struct_get(fc.surface_chunks, "s" .. math.floor(x / fc.chunk_width))
  for i = 0, gm.array_length(ch.stumps) - 1 do
    if ch.stumps[i].x == x then return ch.stumps[i] end
  end
  error("no stump at " .. x)
end
local TOFF = vis(global, global, "tree_mushroom_offset_x")
local TH = vis(global, global, "tree_mushroom_height_px")
-- type, x, y (nil = surface)
local BUILD = {
  { 0, -300 }, { 1, -200 }, { 1, 600 }, { 2, -400 }, { 2, 400 }, { 2, 700 }, { 3, 230 + TOFF, SY - TH },
  { 3, -933 + TOFF, SY - TH }, { 4, 100 }, { 5, -100 }, { 5, 300 }, { 5, 3000 }, { 6, 500 }, { 6, -500 },
  { 7, -950, SY - 120 }, { 7, -559, SY - 30 }, { 7, 250, SY - 200 },
}
gm.random_set_seed(4242)
local added = {}
for _, b in ipairs(BUILD) do
  local idx = mushroom_add(global, global, b[1], b[2], b[3] or SY)
  local m = global.mushrooms[idx]
  local flies = {}
  for j = 0, gm.array_length(m.flies) - 1 do
    local f = m.flies[j]
    flies[#flies + 1] = { f.angle, f.radius, f.speed, f.bob }
  end
  added[#added + 1] = { idx = idx, type = m.type, variant = m.variant, x = m.x, y = m.y, timer = m.timer,
    charges = m.charges, born = m.born, flies = flies }
end
out.added = { list = added, probe = probe(), stats = stats() }
local cost_after = {}
for _, perk in ipairs({ false, true }) do
  set_perk("green", perk)
  cost_after[#cost_after + 1] = cost_rows()
end
set_perk("green", false)
out.costs = costs
out.costs_after = cost_after

-- hosts, timers, consumption
local T_EAT = find_tree(-933)
local T_KEEP = find_tree(230)
local STUMP = find_stump(-559)
local M = global.mushrooms
M[14].host, M[14].side, M[14].variant = T_EAT, -1, consumptor_variant(global, global, -1)
M[15].host, M[15].side, M[15].variant = STUMP, 1, consumptor_variant(global, global, 1)
M[16].host, M[16].side, M[16].variant = T_KEEP, 1, consumptor_variant(global, global, 1)
T_EAT.consume = 0.999
STUMP.consume = 0.99
T_KEEP.consume = 0.25
M[1].timer = 30
M[2].timer = 75
M[5].active = false
M[5].charges = 0
M[5].regen_timer = 3500
M[6].timer = 50
M[7].timer = 20
M[12].timer = 30
M[13].timer = 41

-- save the colony, then load it back: the fixture
global.save_slot = slot
global.save_name = "mushrooms"
assert(save_game(global, global), "save_game failed")
local saved = save_read_text(global, global, save_path(global, global, slot))
do
  local f = assert(io.open("rust/crates/myco_core/tests/fixtures/mushrooms.json", "wb"))
  f:write(saved)
  f:close()
end
assert(load_game(global, global, slot), "load_game (mushrooms) failed")
global.title_open, global.intro_active, global.pause_open = false, false, false
set_view()
frame = 0
now()

-- 5a. pure helpers on the loaded colony -----------------------------------------------------
local function boxes()
  local o = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do
    local m = global.mushrooms[i]
    local b = mushroom_hit_box(global, global, m)
    local u = mushroom_hit_box_unmirrored(global, global, m)
    o[#o + 1] = { b.x1, b.y1, b.x2, b.y2, u.x1, u.y1, u.x2, u.y2, mushroom_variant(global, global, m),
      gm.sprite_get_name(mushroom_sprite_now(global, global, m)), shroom_grow_scale(global, global, m),
      mushroom_index_of(global, global, m) }
  end
  return o
end
local function progress()
  local o = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do
    local m = global.mushrooms[i]
    o[#o + 1] = { U(tree_shroom_progress(global, global, m)), U(experience_progress(global, global, m)),
      U(consumptor_progress(global, global, m)), U(repeller_regen_progress(global, global, m)),
      U(predator_progress(global, global, m)) }
  end
  return o
end
out.boxes0 = boxes()
out.progress0 = progress()
local hosts0 = {}
for i = 0, gm.array_length(global.mushrooms) - 1 do
  local m = global.mushrooms[i]
  if gm.is_undefined(m.host) then hosts0[#hosts0 + 1] = "undefined" else hosts0[#hosts0 + 1] = m.host.x end
end
out.hosts0 = hosts0
local cf = {}
for _, h in ipairs({ find_tree(-933), find_tree(230), find_stump(-559) }) do
  cf[#cf + 1] = { consume_frames(global, global, h), consume_food(global, global, h) }
  local list = consumptors_on(global, global, h)
  cf[#cf] [3] = gm.array_length(list)
end
out.consume = cf

-- 4. 600 frames -----------------------------------------------------------------------------
gm.random_set_seed(99)
local hreach = hedgehog_frame_w(global, global) * 0.4
local breach = boar_frame_w(global, global) * vis(global, global, "boar_snout_dx")
local frames = {}
local snaps = {}
local function pool_snap(m)
  local s = m.stench
  if not gm.is_struct(s) then return { n = -1, due = m.stench_due, live = U(gm.variable_struct_get(m, "stench_live")) } end
  local first = {}
  for i = 0, math.min(s.n, 4) - 1 do
    first[#first + 1] = { s.side[i], s.rise[i], s.offset[i], s.speed[i], s.age[i], s.phase[i], s.size[i], s.heat[i],
      s.drift[i], s.px[i], s.py[i], s.pix[i], s.colour[i], s.alpha[i] }
  end
  local sx, sy, sc, sa = 0, 0, 0, 0
  for i = 0, s.n - 1 do
    sx = sx + s.px[i]
    sy = sy + s.py[i]
    sc = sc + s.colour[i]
    sa = sa + s.alpha[i]
  end
  return { n = s.n, due = m.stench_due, live = U(gm.variable_struct_get(m, "stench_live")), first = first,
    sums = { sx, sy, sc, sa } }
end
local function mushrooms_snap()
  local o = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do
    local m = global.mushrooms[i]
    o[#o + 1] = { type = m.type, x = m.x, y = m.y, timer = m.timer, active = B(m.active), charges = m.charges,
      flash = m.flash, regen = U(gm.variable_struct_get(m, "regen_timer")),
      fly0 = gm.array_length(m.flies) > 0 and { m.flies[0].angle, m.flies[0].bob } or {} }
  end
  return o
end
local function clouds_snap()
  local o = {}
  for i = 0, gm.array_length(global.repel_clouds) - 1 do
    local c = global.repel_clouds[i]
    local px, py, ps = 0, 0, 0
    for k = 0, gm.array_length(c.puffs) - 1 do
      px = px + c.puffs[k].x
      py = py + c.puffs[k].y
      ps = ps + c.puffs[k].size
    end
    local sx, sy = 0, 0
    for k = 0, gm.array_length(c.specks) - 1 do
      sx = sx + c.specks[k].x
      sy = sy + c.specks[k].y
    end
    local p0 = c.puffs[0]
    o[#o + 1] = { x = c.x, y = c.y, ground = c.ground, dir = c.dir, flash = c.flash, puffs = gm.array_length(c.puffs),
      specks = gm.array_length(c.specks), sums = { px, py, ps, sx, sy },
      p0 = p0 and { p0.x, p0.y, p0.vx, p0.vy, p0.size, p0.grow, p0.life, p0.total, p0.colour, B(p0.dust) } or {} }
  end
  return o
end
local function animals_snap()
  local o = {}
  for i = 0, gm.array_length(global.animals) - 1 do o[#o + 1] = { global.animals[i].x, global.animals[i].dir } end
  local b = {}
  for i = 0, gm.array_length(global.boars) - 1 do b[#b + 1] = { global.boars[i].x, global.boars[i].dir, B(global.boars[i].fleeing) } end
  return { animals = o, boars = b }
end
local function drops_snap()
  local o = {}
  for i = 0, gm.array_length(global.drops) - 1 do
    local d = global.drops[i]
    o[#o + 1] = { d.x, d.y, d.target_y, d.speed }
  end
  return o
end
local function hosts_snap()
  local o = {}
  for _, k in ipairs(keys(fc.surface_chunks)) do
    local ch = gm.variable_struct_get(fc.surface_chunks, k)
    for i = 0, gm.array_length(ch.trees) - 1 do
      local t = ch.trees[i]
      o[#o + 1] = { "t", t.x, U(t.consume), B(gm.variable_struct_get(t, "consumed")) }
    end
    for i = 0, gm.array_length(ch.stumps) - 1 do
      local t = ch.stumps[i]
      o[#o + 1] = { "s", t.x, U(t.consume), B(gm.variable_struct_get(t, "consumed")) }
    end
  end
  return o
end
local STINKY = {}
for i = 0, gm.array_length(global.mushrooms) - 1 do
  if global.mushrooms[i].type == 5 then STINKY[#STINKY + 1] = global.mushrooms[i] end
end

for k = 0, 599 do
  frame = k
  now()
  if k == 100 then
    local r = global.mushrooms[3]
    gm.array_push(global.animals, gm.struct({ x = r.x - hreach - 30, y = SY, dir = 1, speed = 0, frame = 0,
      frame_timer = 4, carrying = -1 }))
  end
  if k == 300 then
    local r = global.mushrooms[4]
    gm.array_push(global.boars, gm.struct({ x = r.x + breach + 40, y = SY, dir = -1, fleeing = false }))
  end
  mushrooms_step(global, global)
  drops_step(global, global)
  repeller_clouds_step(global, global)
  frames[#frames + 1] = { global.food, global.water_free, global.upgrade_points, gm.array_length(global.mushrooms),
    gm.array_length(global.drops), gm.array_length(global.repel_clouds), global.stinky_live_count, global.stinky_live_last }
  if k % 50 == 0 or k == 599 or k == 101 or k == 301 then
    local pools = {}
    for _, m in ipairs(STINKY) do pools[#pools + 1] = pool_snap(m) end
    snaps[#snaps + 1] = { frame = k, pools = pools, mushrooms = mushrooms_snap(), clouds = clouds_snap(),
      animals = animals_snap(), drops = drops_snap(), kick = U(global.repel_kick) }
  end
end
out.frames = frames
out.snaps = snaps
out.final = { hosts = hosts_snap(), stats = stats(), probe = probe(), boxes = boxes(), progress = progress() }

-- 5b. stinky helpers ------------------------------------------------------------------------
local shapes = {}
for lvl = 0, 4 do
  set_upg("stinky_width", lvl)
  local sh = stinky_shape(global, global)
  local fr = {}
  for _, dy in ipairs({ -10, 0, 33.5, 120, 400, 2000 }) do fr[#fr + 1] = stinky_funnel_radius(global, global, sh, dy) end
  local hits = {}
  for _, px in ipairs({ -160, -120, -100, -80, -40, 0, 290, 300, 330, 2990 }) do
    for _, py in ipairs({ SY + 20, SY - 20, SY - 60, SY - 200, SY - 600 }) do
      hits[#hits + 1] = B(stinky_beam_hits(global, global, px, py, 6))
    end
  end
  local guards = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do guards[#guards + 1] = B(stinky_guards(global, global, global.mushrooms[i])) end
  shapes[#shapes + 1] = { base = sh.base, open = sh.open, flare = sh.flare, top = sh.top, top_at = stinky_top_at(global, global, lvl),
    funnel = fr, hits = hits, reach = stinky_squirrel_reach(global, global), guards = guards }
end
set_upg("stinky_width", 2)
out.stinky_shapes = shapes
local caps = {}
for v = 0, 3 do
  for _, born in ipairs({ "none", 0, -300, -1000 }) do
    local m = gm.struct({ type = 5, variant = v })
    if born ~= "none" then m.born = gm.current_time + born end
    caps[#caps + 1] = { v, born, stinky_cap_height(global, global, m), shroom_grow_scale(global, global, m) }
  end
end
out.caps = caps
local hot = gm.make_colour_rgb(vis(global, global, "stinky_hot_r"), vis(global, global, "stinky_hot_g"), vis(global, global, "stinky_hot_b"))
local deep = gm.make_colour_rgb(vis(global, global, "stinky_deep_r"), vis(global, global, "stinky_deep_g"), vis(global, global, "stinky_deep_b"))
local tab = stinky_shade_table(global, global, hot, deep)
local shade = {}
for i = 0, 256 do shade[#shade + 1] = tab[i] end
out.shade = { hot = hot, deep = deep, table = shade }
out.wind = { stinky_wind_target(global, global), stinky_wind_shift(global, global, 50), stinky_wind_shift(global, global, 0) }
out.front = repeller_animal_front(global, global, 100, -1, 30)
return out
