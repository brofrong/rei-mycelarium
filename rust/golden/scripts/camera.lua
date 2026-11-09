-- Camera and cut-scene reference for rust/crates/myco_core/tests/camera.rs: boar_shake (repeller
-- kick, boars near the view, ui_blocking), and the migration cut-scene from cine_show_start through
-- every cine_step frame (stops, camera, shake, white, spores, the planet: stars and mycelium
-- segments), the hold on the "migration_end" popup, tasks_popup_close and the fade out.
-- gm.current_time is driven like the Rust sim clock: floor(frame * 1000 / 60).
local gm = require("gm")

local function now(frame) return math.floor(frame * 1000 / 60) end
local function num(v, d)
  if v == nil or gm.is_undefined(v) then return d end
  return v
end

local fc = gm.instance_find(obj_forest_controller, 0)
local mc = gm.instance_find(obj_mycelium_controller, 0)

local saved = {
  title = global.title_open, intro = global.intro_active, pause = global.pause_open,
  menu = global.menu_open, logo = global.logo_open, debug = global.debug_open,
  boars = global.boars, kick = global.repel_kick, mushrooms = global.mushrooms,
  wind_dir = global.wind_dir, wind_speed = global.wind_speed,
}
global.title_open, global.intro_active, global.pause_open = false, false, false
global.menu_open, global.logo_open, global.debug_open = false, false, false
global.tasks.popup = ""
global.tasks.cine = nil

local out = {}

-- boar_shake ----------------------------------------------------------------------------------
local function boar(x) return gm.struct({ x = x, y = 480, dir = 1, frame = 0 }) end
gm.random_set_seed(3)
global.repel_kick = 4
global.boars = gm.array()
local rows = {}
-- frames 1..40: kick only (no boars); 41..80: one boar 300 px right of the view; 81..120: a boar
-- inside the view; 121..130: blocked by the pause (no RNG draws, kick kept)
for f = 1, 130 do
  if f == 41 then global.boars = gm.array(boar(3800 + 1920 + 300)) end
  if f == 81 then global.boars = gm.array(boar(4500), boar(-9000)) end
  if f == 121 then global.repel_kick = 3; global.pause_open = true end
  gm.current_time = now(f)
  local s = boar_shake(global, global, 3800, 1920)
  rows[#rows + 1] = { f, s[0], s[1], global.repel_kick }
end
global.pause_open = false
out.boar_shake = { rows = rows, rng_after = gm.random(1) }

-- the migration cut-scene ---------------------------------------------------------------------
local function shroom(type, x, stolen, carried)
  return gm.struct({ type = type, x = x, y = 480, stolen = stolen, carried_by = carried })
end
global.boars = gm.array()
global.repel_kick = 0
global.wind_dir, global.wind_speed = 1, 2
mc.x, mc.y = 5000, 480
mc.myc_spores = gm.array()
fc.camera_x, fc.camera_y, fc.camera_width, fc.camera_height = 3800, 0, 1920, 1080
fc.camera_vx, fc.camera_vy = 3, -1
global.mushrooms = gm.array(
  shroom(0, 4000, false, -1), shroom(0, 5600, false, -1), shroom(1, 4400, false, -1),
  shroom(0, 7000, false, -1), shroom(0, 6100, true, -1), shroom(0, 2500, false, -1),
  shroom(0, 4800, false, 3))
local stat0 = num(gm.variable_struct_get(global.tasks.stats, "ach_migrated"), 0)

gm.random_set_seed(21)
gm.current_time = now(0)
cine_show_start(global, global)
local c = global.tasks.cine
local stops = {}
for i = 0, gm.array_length(c.stops) - 1 do
  local s = c.stops[i]
  stops[#stops + 1] = { s.x, s.y, s.seconds, s.count }
end
out.cine = { stops = stops, blocking = gm.bool(cine_blocking(global, global)), showing = gm.bool(cine_showing(global, global)) }

local frames = {}
local planet
local f = 0
local function spores() return gm.array_length(mc.myc_spores) end
local function spore_sum()
  local sx, sy = 0, 0
  for i = 0, gm.array_length(mc.myc_spores) - 1 do
    sx = sx + mc.myc_spores[i].x + mc.myc_spores[i].vx
    sy = sy + mc.myc_spores[i].y + mc.myc_spores[i].vy
  end
  return sx, sy
end
while gm.is_struct(global.tasks.cine) and global.tasks.cine.phase == "show" and f < 2000 do
  f = f + 1
  gm.current_time = now(f)
  cine_step(global, global)
  c = global.tasks.cine
  local sh = cine_shake(global, global)
  local dx, dy = -999, -999
  if not gm.is_undefined(sh) and sh ~= nil then dx, dy = sh[0], sh[1] end
  frames[#frames + 1] = { f, c.phase == "show" and 1 or 0, c.stop, c.t, fc.camera_x, fc.camera_y,
    fc.camera_vx, fc.camera_vy, num(c.shake_k, -1), num(c.white, -1), spores(), dx, dy }
end
out.cine.frames = frames
local sx, sy = spore_sum()
out.cine.spores_after_show = { spores(), sx, sy }
-- the planet built by cine_planet_start on the last stop
c = global.tasks.cine
local stars = {}
for i = 0, gm.array_length(c.stars) - 1 do
  local s = c.stars[i]
  stars[#stars + 1] = { s.x, s.y, s.s, s.a, s.ph, s.sp }
end
local segs, tsum, wsum = {}, 0, 0
for i = 0, gm.array_length(c.segs) - 1 do
  local s = c.segs[i]
  tsum = tsum + s.time
  wsum = wsum + s.w
  if i % 37 == 0 or i == gm.array_length(c.segs) - 1 then
    segs[#segs + 1] = { i, s.x1, s.y1, s.x2, s.y2, s.time, s.w }
  end
end
out.cine.planet = { phase = c.phase, t = c.t, white = c.white, shake_k = c.shake_k,
  stars = stars, seg_count = gm.array_length(c.segs), seg_tsum = tsum, seg_wsum = wsum, segs = segs,
  rng_probe_state = gm.random(1) }
-- play until the hold
local play = 0
while global.tasks.cine.phase == "play" and play < 2000 do
  play = play + 1
  f = f + 1
  gm.current_time = now(f)
  cine_step(global, global)
end
out.cine.play_frames = play
out.cine.hold = { phase = global.tasks.cine.phase, t = global.tasks.cine.t, popup = global.tasks.popup,
  stat = num(gm.variable_struct_get(global.tasks.stats, "ach_migrated"), 0) - stat0,
  blocking = gm.bool(cine_blocking(global, global)), showing = gm.bool(cine_showing(global, global)) }
-- a few frames on hold, then the popup closes: out, fade, gone
for _ = 1, 5 do f = f + 1; cine_step(global, global) end
out.cine.hold.t_after5 = global.tasks.cine.t
tasks_popup_close(global, global)
local fade = {}
while gm.is_struct(global.tasks.cine) and #fade < 500 do
  f = f + 1
  local ph = global.tasks.cine.phase
  cine_step(global, global)
  local a = gm.is_struct(global.tasks.cine) and global.tasks.cine.alpha or -1
  fade[#fade + 1] = { ph == "out" and 1 or 0, a, gm.bool(cine_blocking(global, global)) and 1 or 0 }
end
out.cine.fade = fade
out.cine.rng_after = gm.random(1)

global.title_open, global.intro_active = saved.title, saved.intro
global.menu_open, global.logo_open, global.debug_open = saved.menu, saved.logo, saved.debug
global.boars, global.repel_kick, global.mushrooms = saved.boars, saved.kick, saved.mushrooms
global.wind_dir, global.wind_speed = saved.wind_dir, saved.wind_speed
global.tasks.cine = nil
global.tasks.popup = ""
mc.myc_spores = gm.array()
return out
