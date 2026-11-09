-- Weather (scr_weather) reference for rust/crates/myco_core/tests/weather.rs: weather_init, the rain
-- phase/gloom cycle, drops/splashes/spray under weather_step with a moving view, weather_force_rain
-- and rain_gloom_shade. Every scenario reseeds the gm RNG, so the Rust side can replay it with
-- Rng::new(seed). sfx_play is replaced to count thunder cues (and play nothing).
local gm = require("gm")

local thunder_count = 0
local real_sfx_play = sfx_play
sfx_play = function(self, other, snd)
  if snd == thunder then thunder_count = thunder_count + 1 end
  return -1
end

local function list(arr, fields)
  local t = {}
  for i = 0, gm.array_length(arr) - 1 do
    local s = arr[i]
    local row = {}
    for k, f in ipairs(fields) do row[k] = s[f] end
    t[#t + 1] = row
  end
  return t
end

local DROP = { "band", "x", "y", "ground", "speed_scale", "len_scale", "alpha_scale" }
local SPLASH = { "x", "y", "t", "life", "size" }
local SPRAY = { "x", "y", "vx", "vy", "ground", "t", "life" }

local function summary(w)
  local sx, sy = 0, 0
  for i = 0, gm.array_length(w.drops) - 1 do
    sx = sx + w.drops[i].x
    sy = sy + w.drops[i].y
  end
  return { w.phase, w.timer, w.intensity, w.active, w.wind, w.gust, w.gust_target, w.gust_timer,
    w.gloom, rain_gloom_shade(global, global, w), gm.array_length(w.splashes),
    gm.array_length(w.spray), sx, sy, weather_is_raining(global, global) and 1 or 0 }
end

local function full(w)
  return { drops = list(w.drops, DROP), splashes = list(w.splashes, SPLASH),
    spray = list(w.spray, SPRAY), summary = summary(w) }
end

-- view(f) -> vx, vy, vw, vh for frame f (1-based)
local function run(seed, wind_dir, wind_speed, timer, frames, every, view)
  gm.random_set_seed(seed)
  weather_init(global, global)
  local w = global.weather
  if timer then w.timer = timer end
  global.wind_dir, global.wind_speed = wind_dir, wind_speed
  thunder_count = 0
  local init = full(w)
  local rows, thunders = {}, {}
  for f = 1, frames do
    local before = thunder_count
    weather_step(global, global, view(f))
    if thunder_count ~= before then thunders[#thunders + 1] = f end
    if f % every == 0 or f == frames then rows[#rows + 1] = { f, summary(w) } end
  end
  return { seed = seed, wind_dir = wind_dir, wind_speed = wind_speed, timer = timer or -1,
    frames = frames, every = every, init = init, rows = rows, thunders = thunders,
    final = full(w), rng_after = gm.random(1) }
end

local out = {}
out.vis = { rain_max_drops = vis(global, global, "rain_max_drops") }

out.runs = {}
-- A full natural cycle from weather_init (first gap, fade in, hold, fade out, next gap start)
-- with the view sliding right so drops leave on the left and get reset.
out.runs[1] = run(31607, 1, 2.5, nil, 12500, 7, function(f)
  return 1000 + (f % 900) * 2, 0, 1920, 1080
end)
-- Short gap, wind to the left, the view sliding left and dipping below the surface for a while
-- (weather_step returns early, active = 0, then drops are re-scattered anywhere).
out.runs[2] = run(7, -1, 3.0, 40, 3600, 3, function(f)
  local vy = (f >= 900 and f < 1100) and 600 or -120 + (f % 300)
  return 4000 - (f % 1200) * 3, vy, 1600, 900
end)
-- Fractional wind, small view, high frame count through two cycles at a coarse sample.
out.runs[3] = run(123456, 1, 1.37, 5, 26000, 50, function(f)
  return 200.5 + (f % 50), 100, 640, 360
end)

-- weather_force_rain from each phase.
out.force = {}
for _, case in ipairs({
  { phase = 0, timer = 500, intensity = 0, gust = 1.3 },
  { phase = 1, timer = 100, intensity = 0.4, gust = 0.8 },
  { phase = 2, timer = 900, intensity = 1, gust = 1.6 },
  { phase = 3, timer = 50, intensity = 0.25, gust = 0.6 },
}) do
  weather_init(global, global)
  local w = global.weather
  w.phase, w.timer, w.intensity, w.gust, w.gust_target, w.gust_timer = case.phase, case.timer,
    case.intensity, case.gust, case.gust, 33
  local r = weather_force_rain(global, global)
  out.force[#out.force + 1] = { case.phase, case.timer, case.intensity, case.gust,
    r and 1 or 0, w.phase, w.timer, w.intensity, w.gust, w.gust_target, w.gust_timer }
end

-- rain_gloom_shade on a grid.
out.shade = {}
weather_init(global, global)
for i = 0, 40 do
  global.weather.gloom = i / 40
  out.shade[#out.shade + 1] = { i / 40, rain_gloom_shade(global, global, global.weather) }
end

-- rain_splash_add directly: caps and RNG order.
gm.random_set_seed(99)
weather_init(global, global)
global.weather.wind = -1.7
for i = 1, 130 do
  rain_splash_add(global, global, global.weather, i * 3.5, 480 + (i % 4), 0.55 + (i % 3) * 0.3)
end
out.splash_add = full(global.weather)
out.splash_add.rng_after = gm.random(1)

sfx_play = real_sfx_play
weather_init(global, global)
return out
