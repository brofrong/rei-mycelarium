-- Audio (scr_audio) reference for rust/crates/myco_app/tests/audio.rs.
-- The gm audio builtins are replaced by a recorder, so every scr_audio call turns into a log of
-- the GameMaker audio calls it makes (play / gain / stop / loop points / emitter positions).
-- The Rust engine runs the same scenarios against a fake backend with the same rules:
--   * voice ids start at 1000, emitter ids at 0 (in creation order);
--   * a voice plays until stopped; a non-looping one also ends ceil(length * 60) frames after it
--     started (frames counted by the scenario through `now`);
--   * gm.choose is pinned per scenario (the Rust side passes the chosen sound in).
-- Also captured: every sound's length (audio_sound_length) and sound_camera_gain /
-- sound_world_place on a grid of points around several views.
local gm = require("gm")
local assets = require("gm.assets")

local real_len = gm.audio_sound_length
local function sname(id) return assets.sounds[id].name end

local out = { lengths = {} }
for id = 0, 23 do
  local s = assets.sounds[id]
  if s then out.lengths[s.name] = real_len(id) end
end

-- recorder -------------------------------------------------------------------------------------
local log, voices, next_voice, next_em, now = {}, {}, 1000, 0, 0
local function L(...) log[#log + 1] = { ... } end
local function new_voice(snd, loop)
  local id = next_voice
  next_voice = next_voice + 1
  local loops = loop and loop ~= 0
  voices[id] = { snd = snd, stopped = false, loop = loops,
    ends = now + math.ceil(real_len(snd) * 60) }
  return id
end
gm.audio_play_sound = function(snd, prio, loop)
  local id = new_voice(snd, loop)
  L("play", id, sname(snd), (loop and loop ~= 0) and 1 or 0, -1)
  return id
end
gm.audio_play_sound_on = function(em, snd, loop, prio)
  local id = new_voice(snd, loop)
  L("play", id, sname(snd), (loop and loop ~= 0) and 1 or 0, em)
  return id
end
gm.audio_is_playing = function(id)
  local v = voices[id]
  if not v then return false end
  return not v.stopped and (v.loop or now < v.ends)
end
gm.audio_stop_sound = function(id)
  if voices[id] then
    voices[id].stopped = true
    L("stop", id)
  else
    L("stop_asset", sname(id))
  end
end
gm.audio_sound_gain = function(id, g, ms) L("gain", id, g, ms) end
gm.audio_sound_loop_start = function(id, pos) L("loop_start", id, pos) end
gm.audio_sound_loop_end = function(id, pos) L("loop_end", id, pos) end
gm.audio_emitter_create = function()
  local id = next_em
  next_em = next_em + 1
  return id
end
gm.audio_emitter_free = function(id) L("em_free", id) end
gm.audio_emitter_position = function(id, x, y, z) L("em_pos", id, x, z) end
gm.audio_falloff_set_model = function() end
gm.audio_listener_position = function() end

local real_choose = gm.choose
local function pin_choose(k) gm.choose = function(...) return (select(k, ...)) end end

local function reset()
  log, voices, next_voice, next_em, now = {}, {}, 1000, 0, 0
  global.volume_sfx = 0.8
  global.volume_music = 0.55
  global.rain_voice = -1
  global.forest_voice = -1
  global.forest_duck = 0
  global.water_touch_cooldown = 0
  global.sound_emitters = nil
  global.sound_loops = nil
  global.ambience = nil
  sound_emitters_init(global, global)   -- emitters 0..7
  sound_loops_init(global, global)
end

-- camera ---------------------------------------------------------------------------------------
local cam = gm.view_camera[0]
if cam == -1 then
  cam = gm.camera_create_view(0, 0, 1920, 1080)
  gm.view_camera[0] = cam
end
local function view(x, y, w, h)
  gm.camera_set_view_pos(cam, x, y)
  gm.camera_set_view_size(cam, w, h)
end

-- A. placement grid ----------------------------------------------------------------------------
local VIEWS = { { 0, 0, 1920, 1080 }, { 1000, 500, 2560, 1440 }, { -300, 200, 1920, 1200 } }
out.place = {}
for _, v in ipairs(VIEWS) do
  view(v[1], v[2], v[3], v[4])
  local rows = {}
  for ix = -10, 14 do
    for iy = -6, 9 do
      local x = v[1] + ix * v[3] / 4 + 0.5
      local y = v[2] + iy * v[4] / 3 - 0.25
      local p1 = sound_world_place(global, global, x, y, 1.0)
      local p2 = sound_world_place(global, global, x, y, 0.5)
      local p3 = sound_world_place(global, global, x, y, 0.005)
      rows[#rows + 1] = { x, y, sound_camera_gain(global, global, x, y),
        p1.gain, p1.pan, p2.gain, p2.pan, p3.gain, p3.pan }
    end
  end
  out.place[#out.place + 1] = { view = v, rows = rows }
end
view(0, 0, 1920, 1080)

-- B. rain + forest under an intensity schedule --------------------------------------------------
local function intensity(f)
  if f < 20 then return 0 end
  if f < 120 then return (f - 20) / 100 end
  if f < 260 then return 1 end
  if f < 360 then return (360 - f) / 100 end
  return 0
end
reset()
global.weather = global.weather or {}
local saved_intensity = global.weather.intensity
out.weather = {}
for f = 1, 420 do
  now = f
  log = {}
  global.weather.intensity = intensity(f)
  if f == 200 then voices[global.forest_voice].stopped = true end -- voice dropped: restarts
  rain_sound_step(global, global)
  forest_sound_step(global, global)
  out.weather[#out.weather + 1] = { f, global.weather.intensity, global.forest_duck, log }
end
global.weather.intensity = saved_intensity

-- C. hyphae sounds -----------------------------------------------------------------------------
-- { len, total, x|nil, y|nil, start choice, frames, stop_at|nil, volume_sfx }
local HYPHAE = {
  { 50, 60, nil, nil, 1, 80 },
  { 100, 90, nil, nil, 1, 120 },
  { 150, 120, nil, nil, 2, 160 },
  { 250, 200, nil, nil, 2, 240 },
  { 250, 20, nil, nil, 1, 60 },
  { 180, 150, 2500, 400, 1, 260 },
  { 180, 150, 900, 300, 2, 260 },
  { 300, 180, -700, 1500, 1, 200, 100 },
  { 200, 140, nil, nil, 1, 160, nil, 0 },
}
out.hyphae = {}
for _, c in ipairs(HYPHAE) do
  reset()
  if c[8] then global.volume_sfx = c[8] end
  pin_choose(c[5])
  now = 0
  local s = hyphae_sound_begin(global, global, c[1], c[2], c[3], c[4])
  local frames = { { 0, log } }
  local done_at = -1
  for t = 1, c[6] do
    now = t
    log = {}
    if c[7] and t == c[7] then
      hyphae_sound_stop(global, global, s)
    else
      hyphae_sound_step(global, global, s, t)
    end
    if #log > 0 then frames[#frames + 1] = { t, log } end
    if s.done and done_at < 0 then done_at = t end
  end
  out.hyphae[#out.hyphae + 1] = { case = { c[1], c[2], c[3] or "none", c[4] or "none", c[5], c[6],
    c[7] or -1, c[8] or 0.8 }, frames = frames, done_at = done_at }
end
gm.choose = real_choose

-- D. a sound loop following a boar across the view, then fading out ----------------------------
reset()
out.loop = {}
local l
for f = 1, 200 do
  now = f
  log = {}
  sound_loops_step(global, global)
  if f <= 120 then
    if l == nil or l.done then l = sound_loop_start(global, global, boar_tremor) end
    sound_loop_follow(global, global, l, -1500 + f * 40, 600, 1.0, 1.0, 45, 60)
  end
  out.loop[#out.loop + 1] = { f, gm.array_length(global.sound_loops), log }
end

-- E. positioned one-shots, water touch cooldown, mushroom growth --------------------------------
reset()
out.oneshot = {}
local SHOTS = { { 960, 540, 1, nil }, { -100, 540, 1, nil }, { 2100, 900, 0.7, nil },
  { 4000, 540, 1, nil }, { 2500, -600, 1, 2.0 }, { -2000, 2000, 0.5, 3 }, { 1000, 1200, 1, 0.5 },
  { 3000, 540, 1, 0.5 }, { 500, 500, 1.4, nil } }
for i, s in ipairs(SHOTS) do
  log = {}
  local v = sfx_play_at(global, global, explosion, s[1], s[2], s[3], s[4])
  out.oneshot[#out.oneshot + 1] = { s[1], s[2], s[3], s[4] or "none", v, log }
end
out.water = {}
for f = 1, 100 do
  now = f
  log = {}
  if global.water_touch_cooldown > 0 then global.water_touch_cooldown = global.water_touch_cooldown - 1 end
  if f % 7 == 0 then water_touch_sound(global, global, 100 + f * 30, 300) end
  if f == 50 or f == 51 then water_touch_sound(global, global) end
  if #log > 0 then out.water[#out.water + 1] = { f, global.water_touch_cooldown, log } end
end
out.mush = {}
for k = 1, 3 do
  pin_choose(k)
  log = {}
  mushroom_grow_sound(global, global)
  out.mush[#out.mush + 1] = { k, "none", log }
  log = {}
  mushroom_grow_sound(global, global, 2200, 500, 0.6)
  out.mush[#out.mush + 1] = { k, "at", log }
end
gm.choose = real_choose
log = {}
global.volume_sfx = 0
out.muted = { sfx_play(global, global, thunder), sfx_play_at(global, global, explosion, 10, 10),
  hyphae_sound_play(global, global, hyphae_start_1, false, 1) }

return out
