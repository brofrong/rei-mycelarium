-- Life (scr_life: hedgehog, boar, squirrel, crow, worm, leaves, far life) and game_step against
-- the LÖVE port. For tests/life.rs:
-- 1. funcs: the deterministic behaviour functions on input grids (frames, offsets, claws, trunk
--    edges, zones, cell checks, thresholds, takes/can-take checks) on the colony fixture
--    (tests/fixtures/mushrooms.json, loaded through load_game);
-- 2. timers: the Exploration fixture (no mushrooms: animals far away) run through life_step
--    for 3600 frames, state.timers at checkpoints;
-- 3. run: the colony fixture, seeded, short animal timers, absorption and protein owned, with
--    summoned hedgehog/squirrel/crow/boar, a leaf gust and worms, 1800 frames of life_step;
--    per-frame counts and resources, snapshots of every animal, leaf and worm;
-- 4. game: the Exploration fixture, seeded, 1500 frames of game_step (wind, gusts, points,
--    play frames).
-- Determinism: gm.current_time = floor(frame * 1000 / 60) like Sim::now_ms, gm.get_timer = 0,
-- camera view fixed (-960, 0, 1920, 1080).
-- Run: MYCO_IDENTITY=MycelariumTest-t10 tools/golden.sh life
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
local slot = 4
local function load_fixture(name)
  local text = read("rust/crates/myco_core/tests/fixtures/" .. name):gsub("%z+$", "")
  save_write_text(global, global, save_path(global, global, slot), text)
  assert(load_game(global, global, slot), "load_game failed: " .. name)
  global.title_open, global.intro_active, global.pause_open = false, false, false
  global.menu_open, global.debug_open, global.logo_open = false, false, false
end

local cam = gm.view_camera[0]
local VIEW = { x = -960, y = 0, w = 1920, h = 1080 }
local function set_view()
  gm.camera_set_view_pos(cam, VIEW.x, VIEW.y)
  gm.camera_set_view_size(cam, VIEW.w, VIEW.h)
  global.view_w, global.view_h = VIEW.w, VIEW.h
end
gm.get_timer = function() return 0 end
local frame = 0
local function now() gm.current_time = math.floor(frame * 1000 / 60) end

local function U(v) if gm.is_undefined(v) then return "undefined" end return v end
local function B(v) return gm.bool(v) and 1 or 0 end
local function set_upg(id, lvl) gm.variable_struct_set(global.upgrade_levels, id, lvl) end
local function set_perk(id, on)
  if on then gm.variable_struct_set(global.cave_perks, id, true)
  elseif gm.variable_struct_exists(global.cave_perks, id) then gm.variable_struct_remove(global.cave_perks, id) end
end
local function S(t) return gm.struct(t) end

local out = { view = VIEW }
load_fixture("mushrooms.json")
set_view()
now()
gm.random_set_seed(1)
local SY = wld(global, global, "surface_height_px")

-- 1. funcs ----------------------------------------------------------------------------------
local F = {}
F.hedgehog = { hedgehog_frame_w(global, global), hedgehog_frame_h(global, global), hedgehog_back_y(global, global),
  hedgehog_border_margin(global, global) }
F.back_point = {}
for _, x in ipairs({ 0, 13.7, -250.2 }) do
  for _, y in ipairs({ 480, 500.5 }) do
    for _, dir in ipairs({ -1, 1, 0 }) do
      for _, fr in ipairs({ -1, 0, 3.6, 7, 9 }) do
        local p = hedgehog_back_point(global, global, x, y, dir, fr)
        F.back_point[#F.back_point + 1] = { x, y, dir, fr, p[0], p[1] }
      end
    end
  end
end
local span = hedgehog_world_span(global, global)
F.span = { span.left, span.right }
F.boar = { boar_frame_w(global, global) }
F.squirrel = { squirrel_frame_w(global, global), squirrel_frame_h(global, global), squirrel_run_y(global, global),
  squirrel_tail_drop(global, global) }
F.offset = {}
for _, face in ipairs({ -1, 1 }) do
  for _, ang in ipairs({ 0, 37, 90, 180, -90, 270.5 }) do
    for _, pt in ipairs({ { 0, 0 }, { 40, 77 }, { 95.5, 12 } }) do
      local o = squirrel_offset(global, global, face, ang, pt[1], pt[2])
      local sq = S({ mirror = face, angle = ang, x = 120.25, y = 300 })
      local p = squirrel_point(global, global, sq, pt[1], pt[2])
      F.offset[#F.offset + 1] = { face, ang, pt[1], pt[2], o[0], o[1], p[0], p[1] }
    end
  end
end
F.bite = {}
for t = 0, 7 do
  local b = squirrel_shroom_bite(global, global, t)
  F.bite[#F.bite + 1] = { t, b[0], b[1], squirrel_bite_y(global, global, S({ type = t, y = 470.5 })),
    crow_cap_dy(global, global, S({ type = t })) }
end
local TREES = { "tree01", "tree02", "tree03" }
F.edges = {}
for _, tn in ipairs(TREES) do
  for _, flip in ipairs({ -1, 1 }) do
    local tr = S({ sprite = _G[tn], x = 333.5, y = SY, flip = flip })
    for _, side in ipairs({ -1, 0, 1 }) do
      for _, py in ipairs({ -200, SY - 600, SY - 400.4, SY - 250, SY - 120, SY - 40, SY, SY + 90 }) do
        F.edges[#F.edges + 1] = { tn, flip, side, py, squirrel_edge_x(global, global, tr, side, py),
          squirrel_climb_x(global, global, -side, 90 * side, tr, side, py),
          squirrel_climb_x(global, global, side, 0, tr, side, py) }
      end
    end
  end
end
F.sky = {}
for _, tn in ipairs(TREES) do
  for _, tx in ipairs({ -3000, -900, 0, 1000, 5000 }) do
    for _, vy in ipairs({ -300, 0, 200 }) do
      local tr = S({ sprite = _G[tn], x = tx, y = SY, flip = 1 })
      local v = S({ x = -960, y = vy, w = 1920, h = 1080 })
      F.sky[#F.sky + 1] = { tn, tx, vy, squirrel_sky_y(global, global, tr, v) }
    end
  end
end
F.turn = {}
do
  local s = S({ x = 10, y = 20, angle = 0 })
  squirrel_turn_start(global, global, s, 3, 180, 50, -40)
  for k = 1, 12 do
    local done = squirrel_turn_step(global, global, s)
    F.turn[#F.turn + 1] = { s.turn_t, s.angle, s.x, s.y, B(done), s.state }
  end
end
F.hold = {}
for _, mir in ipairs({ -1, 1 }) do
  for _, ang in ipairs({ 0, 90, -90, 180 }) do
    for _, t in ipairs({ 3, 7 }) do
      local s = S({ mirror = mir, angle = ang, x = 50, y = 250, mushroom = S({ type = t }) })
      local p = squirrel_hold_pose(global, global, s)
      F.hold[#F.hold + 1] = { mir, ang, t, p.x, p.y, p.xscale, p.yscale, p.angle }
    end
  end
end
F.crow = { crow_frame_w(global, global), crow_below_claws(global, global) }
F.claw = {}
for _, fr in ipairs({ -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 4.5 }) do
  local c = crow_claw(global, global, fr)
  for _, dir in ipairs({ -1, 1 }) do
    local o = crow_claw_offset(global, global, fr, dir)
    F.claw[#F.claw + 1] = { fr, dir, c[0], c[1], o[0], o[1] }
  end
end
F.crow_sky = {}
for _, vy in ipairs({ -500, 0, 300 }) do
  for _, h in ipairs({ 0, 40.5, 120 }) do
    F.crow_sky[#F.crow_sky + 1] = { vy, h, crow_sky_y(global, global, S({ x = 0, y = vy, w = 1920, h = 1080 }), h) }
  end
end
F.worm = { worm_zone_top(global, global), worm_zone_bottom(global, global), worm_segment_px(global, global) }
F.keep = {}
for _, vy in ipairs({ -500, 0, 480, 2000, 9000 }) do
  for _, vh in ipairs({ 600, 1080, 2400 }) do
    F.keep[#F.keep + 1] = { vy, vh, worm_keep_bottom(global, global, vy, vh) }
  end
end
F.cell = {}
for x = -2000, 2000, 137 do
  for y = SY - 20, worm_zone_bottom(global, global) + 40, 53 do
    F.cell[#F.cell + 1] = { x, y, B(worm_cell_ok(global, global, x, y, gm.undefined)),
      B(worm_cell_ok(global, global, x, y, SY + 4)) }
  end
end
F.body_at = {}
do
  local body = gm.array(S({ x = 0, y = 0 }), S({ x = 3, y = 4 }), S({ x = 6, y = 4 }), S({ x = 6, y = 10 }))
  local w = S({ body = body })
  for _, s in ipairs({ -1, 0, 0.5, 1, 1.25, 2, 2.9, 3, 4 }) do
    local p = worm_body_at(global, global, w, s)
    F.body_at[#F.body_at + 1] = { s, p.x, p.y, p.dir }
  end
end
F.leaf = {}
for _, w in ipairs({ -2, -0.0, 0, 1.5 }) do
  F.leaf[#F.leaf + 1] = { w, leaf_face_for_wind(global, global, w) }
end
F.levels = {}
for lvl = 0, 3 do
  set_upg("absorption_speed", lvl)
  set_upg("absorption_yield", lvl)
  F.levels[#F.levels + 1] = { lvl, leaf_decay_time(global, global), leaf_food_value(global, global),
    worm_absorb_frames(global, global) }
end
set_upg("absorption_speed", 0)
set_upg("absorption_yield", 0)
F.leaf_art = {}
for _, l in ipairs({ S({}), S({ face = 1 }), S({ face = 0, change_from = 0, change_frame = 3 }),
  S({ face = 1, change_from = 1, change_frame = 7.5 }), S({ change_from = -1 }) }) do
  local a = leaf_art_frame(global, global, l)
  F.leaf_art[#F.leaf_art + 1] = { a.x1, a.y1, a.x2, a.y2, a.mid_x, a.mid_y, a.off_x, a.off_y }
end
-- checks over the colony's mushrooms with flags varied
F.checks = {}
local ms = global.mushrooms
for i = 0, gm.array_length(ms) - 1 do
  local m = ms[i]
  local row = {}
  for _, flags in ipairs({ { false, false, -1 }, { true, false, -1 }, { false, true, -1 }, { false, false, 0 } }) do
    m.stolen, m.crow_target, m.carried_by = flags[1], flags[2], flags[3]
    for _, red in ipairs({ false, true }) do
      set_perk("red", red)
      row[#row + 1] = { B(hedgehog_takes(global, global, m)), B(boar_hits_mushroom(global, global, m)),
        B(crow_can_take(global, global, m)), B(squirrel_takes(global, global, m)) }
    end
  end
  m.stolen, m.crow_target, m.carried_by = false, false, -1
  set_perk("red", false)
  F.checks[#F.checks + 1] = row
end
F.counts = {}
local saved = global.mushrooms
for n = 0, 40, 5 do
  local arr = gm.array()
  for k = 1, n do gm.array_push(arr, S({ type = 0, carried_by = (k % 7 == 0) and 0 or -1, stolen = (k % 11 == 0) })) end
  global.mushrooms = arr
  F.counts[#F.counts + 1] = { n, animal_mushroom_count(global, global), animal_timer_step(global, global) }
end
global.mushrooms = saved
local z = life_near_zone(global, global)
F.zone = { z.x1, z.x2, z.w }
F.outside = {}
for _, a in ipairs({ -5000, -2000, -1024, 0, 900, 4000 }) do
  for _, b in ipairs({ 0, 1024, 3000 }) do
    F.outside[#F.outside + 1] = { a, a + b, life_far_outside(global, global, a, a + b, z) }
  end
end
F.any = { B(boar_any_target(global, global)), B(squirrel_any_target(global, global)) }
F.tree_has = {}
F.near = {}
local fc = gm.instance_find(obj_forest_controller, 0)
for c = -2, 2 do
  local trees = fc.surface_get_chunk(fc, fc, c).trees
  for i = 0, gm.array_length(trees) - 1 do
    local t = trees[i]
    F.tree_has[#F.tree_has + 1] = { t.x, gm.sprite_get_name(t.sprite), B(squirrel_tree_has_mushroom(global, global, t)) }
  end
end
for _, x in ipairs({ -3000, -1200, -400, 0, 350, 1500, 4200 }) do
  for _, rng in ipairs({ 80, 600, 2000 }) do
    for _, free in ipairs({ false, true }) do
      local t = squirrel_tree_near(global, global, x, rng, free, gm.undefined)
      F.near[#F.near + 1] = { x, rng, B(free), t and U(t) ~= "undefined" and t.x or "undefined" }
    end
  end
  for _, dir in ipairs({ -1, 0, 1 }) do
    local t = squirrel_tree03_find(global, global, x, dir, 200, false, gm.undefined)
    local t2 = squirrel_tree03_find(global, global, x, dir, 0, true, gm.undefined)
    F.near[#F.near + 1] = { x, dir, U(t) ~= "undefined" and t.x or "undefined", U(t2) ~= "undefined" and t2.x or "undefined" }
  end
end
-- boar_spawn_pack in survival (the pack follows)
game_mode_apply(global, global, "survival")
global.boars, global.boar_pack = gm.array(), gm.array()
boar_spawn_pack(global, global, -3000, 1)
F.pack = {}
for i = 0, gm.array_length(global.boar_pack) - 1 do
  local p = global.boar_pack[i]
  F.pack[#F.pack + 1] = { p.x, p.dir, p.wait }
end
F.pack_targets = gm.array_length(global.boars[0].targets)
game_mode_apply(global, global, "exploration")
global.boars, global.boar_pack = gm.array(), gm.array()
out.funcs = F

-- 2. timers ---------------------------------------------------------------------------------
local function timers()
  return { global.hedgehog_timer, global.boar_timer, global.squirrel_timer, global.crow_timer,
    global.experience_crow_timers[0], global.experience_crow_timers[1], global.leaf_timer, global.worm_timer }
end
load_fixture("exploration.json")
set_view()
gm.random_set_seed(7)
frame = 0
local T = { start = timers(), at = {} }
for k = 0, 3599 do
  frame = k
  now()
  set_view()
  life_step(global, global)
  if (k + 1) % 600 == 0 then T.at[#T.at + 1] = { k + 1, timers() } end
end
out.timers = T

-- 3. run ------------------------------------------------------------------------------------
load_fixture("mushrooms.json")
set_view()
gm.random_set_seed(99)
set_upg("unlock_absorption", 1)
set_upg("unlock_protein", 1)
global.hedgehog_timer = 240
global.squirrel_timer = 420
global.crow_timer = 330
global.experience_crow_timers[0] = 520
global.experience_crow_timers[1] = 700
global.boar_timer = 610
global.leaf_timer = 90
global.worm_timer = 30

local function idx(m)
  if gm.is_undefined(m) then return -1 end
  return mushroom_index_of(global, global, m)
end
local function tx(t) if gm.is_undefined(t) then return "undefined" end return t.x end
local function snap()
  local s = {}
  s.animals = {}
  for i = 0, gm.array_length(global.animals) - 1 do
    local a = global.animals[i]
    s.animals[#s.animals + 1] = { a.x, a.y, a.dir, a.speed, a.frame, a.frame_timer, a.carrying }
  end
  s.boars = {}
  for i = 0, gm.array_length(global.boars) - 1 do
    local b = global.boars[i]
    local tg = {}
    for j = 0, gm.array_length(b.targets) - 1 do tg[#tg + 1] = idx(b.targets[j]) end
    s.boars[#s.boars + 1] = { b.x, b.y, b.dir, b.frame, b.frame_timer, b.hits, B(b.fleeing), tg }
  end
  s.pack = {}
  for i = 0, gm.array_length(global.boar_pack) - 1 do
    local p = global.boar_pack[i]
    s.pack[#s.pack + 1] = { p.x, p.dir, p.wait }
  end
  s.flung = {}
  for i = 0, gm.array_length(global.boar_flung) - 1 do
    local f = global.boar_flung[i]
    s.flung[#s.flung + 1] = { f.type, gm.sprite_get_name(f.sprite), f.variant, f.x, f.y, f.vx, f.vy, f.angle, f.spin, f.life }
  end
  s.squirrels = {}
  for i = 0, gm.array_length(global.squirrels) - 1 do
    local q = global.squirrels[i]
    s.squirrels[#s.squirrels + 1] = { tx(q.tree), tx(q.home), tx(q.target_tree), tx(q.far_tree), B(q.direct), q.side,
      q.dir, q.mirror, q.angle, q.x, q.y, q.stop_y, q.run_line, q.state, q.turn_t, q.frame, q.frame_timer, q.timer,
      idx(q.mushroom), B(q.empty_run), q.grab_x, q.grab_y }
  end
  s.crows = {}
  for i = 0, gm.array_length(global.crows) - 1 do
    local c = global.crows[i]
    s.crows[#s.crows + 1] = { c.state, c.dir, c.x, c.y, c.x0, c.y0, c.grab_x, c.grab_y, c.t, c.frame, c.frame_timer,
      c.timer, c.speed, B(c.flee), idx(c.mushroom) }
  end
  s.pending = {}
  for i = 0, 2 do
    local p = global.crow_pending[i]
    if gm.is_undefined(p) then s.pending[#s.pending + 1] = "undefined"
    else s.pending[#s.pending + 1] = { idx(p.mushroom), p.dir, p.x, p.y } end
  end
  s.leaves = {}
  for i = 0, gm.array_length(global.leaves) - 1 do
    local l = global.leaves[i]
    local tn = (gm.variable_struct_exists(l, "tendril") and gm.is_array(l.tendril)) and gm.array_length(l.tendril) or -1
    s.leaves[#s.leaves + 1] = { l.x, l.y, l.phase, l.mirror, l.turn, B(l.turned), l.turn_delay, l.angle, l.drift,
      B(l.landed), B(l.absorbing), l.timer, l.life, U(l.total), tn }
  end
  s.worms = {}
  for i = 0, gm.array_length(global.worms) - 1 do
    local w = global.worms[i]
    local body = {}
    for j = 0, gm.array_length(w.body) - 1 do body[#body + 1] = { w.body[j].x, w.body[j].y } end
    local tn = gm.variable_struct_exists(w, "tendril") and gm.array_length(w.tendril) or -1
    s.worms[#s.worms + 1] = { w.dir, w.turn, w.side, w.phase, w.check, w.trail_walk, B(w.caught), w.timer, w.total,
      w.struggle, U(w.touch_frames), tn, body }
  end
  s.trail = {}
  for i = 0, gm.array_length(global.worm_trail) - 1 do
    local t = global.worm_trail[i]
    s.trail[#s.trail + 1] = { t.x, t.y, t.life }
  end
  s.tendrils = {}
  for i = 0, gm.array_length(global.leaf_tendrils_out) - 1 do
    local t = global.leaf_tendrils_out[i]
    s.tendrils[#s.tendrils + 1] = { gm.array_length(t.cells), t.timer, t.total }
  end
  s.mushrooms = {}
  for i = 0, gm.array_length(global.mushrooms) - 1 do
    local m = global.mushrooms[i]
    s.mushrooms[#s.mushrooms + 1] = { m.type, m.x, m.y, B(m.stolen), B(m.crow_target), m.carried_by, m.hold_xscale,
      m.hold_yscale, m.hold_angle, U(m.carry_angle) }
  end
  s.far = { global.far_rate.leaves, global.far_rate.worms, global.far_rate.food, global.far_food_due,
    global.far_timer, global.far_scan_from }
  return s
end

local R = { frames = {}, snaps = {} }
for k = 0, 1799 do
  frame = k
  now()
  set_view()
  local note
  if k == 5 then note = hedgehog_summon(global, global) end
  if k == 15 then note = squirrel_summon(global, global) end
  if k == 25 then note = crow_summon(global, global) end
  if k == 35 then note = boar_summon(global, global) end
  if k == 45 then leaf_gust_spawn(global, global) end
  if k == 55 or k == 56 or k == 57 then worm_spawn(global, global, true) end
  if k == 65 or k == 300 or k == 301 then leaf_spawn(global, global, true, gm.undefined, gm.undefined) end
  if k == 900 then note = squirrel_summon(global, global) end
  if k == 950 then note = crow_summon(global, global) end
  life_step(global, global)
  R.frames[#R.frames + 1] = { global.food, global.water_free, gm.array_length(global.mushrooms),
    gm.array_length(global.animals), gm.array_length(global.boars), gm.array_length(global.boar_flung),
    gm.array_length(global.squirrels), gm.array_length(global.crows), gm.array_length(global.leaves),
    gm.array_length(global.worms), gm.array_length(global.worm_trail), gm.array_length(global.leaf_tendrils_out),
    global.hedgehog_timer, global.boar_timer, global.squirrel_timer, global.crow_timer,
    global.experience_crow_timers[0], global.experience_crow_timers[1], global.leaf_timer, global.worm_timer,
    note or "" }
  if k % 25 == 24 then
    local s = snap()
    s.frame = k
    R.snaps[#R.snaps + 1] = s
  end
end
local stats = {}
local names = gm.variable_struct_get_names(global.tasks.stats)
for i = 0, gm.array_length(names) - 1 do stats[#stats + 1] = { names[i], gm.variable_struct_get(global.tasks.stats, names[i]) } end
table.sort(stats, function(a, b) return a[1] < b[1] end)
R.stats = stats
R.probe = gm.random(1.0)
out.run = R

-- 4. game_step ------------------------------------------------------------------------------
load_fixture("exploration.json")
set_view()
gm.random_set_seed(5)
local G = { frames = {} }
for k = 0, 1499 do
  frame = k
  now()
  set_view()
  game_step(global, global)
  if k % 10 == 9 then
    G.frames[#G.frames + 1] = { k, global.wind_dir, global.wind_speed, global.wind_timer, global.upgrade_points,
      global.mycelium_points_awarded, global.play_frames, gm.array_length(global.leaves) }
  end
end
G.probe = gm.random(1.0)
out.game = G
return out
