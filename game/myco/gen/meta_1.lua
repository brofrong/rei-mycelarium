local gm = require("gm")

-- gml_Script_audio_init @ 0x14002f640
function audio_init(self, other)
  gm.ini_open("settings.ini")
  global.volume_sfx = gm.clamp(gm.ini_read_real("audio", "volume_sfx", gm.real(vis(self, other, "volume_sfx"))), 0, 1.0)
  global.volume_music = gm.clamp(gm.ini_read_real("audio", "volume_music", gm.real(vis(self, other, "volume_music"))), 0, 1.0)
  gm.ini_close()
  global.music_tracks = gm.array(hang_guitar_01, guitar, mushrooms)
  global.music_bag = gm.array()
  global.music_last = -1
  global.music_voice = -1
  local i = 0
  while i < gm.array_length(global.music_tracks) do
    gm.audio_stop_sound(global.music_tracks[i])
    i = i + 1
  end
  gm.audio_stop_sound(rain01)
  global.rain_voice = -1
  gm.audio_stop_sound(forest)
  global.forest_voice = -1
  global.forest_duck = 0
  global.water_touch_cooldown = 0
  ambience_init(self, other)
  sound_emitters_init(self, other)
  sound_loops_init(self, other)
end

-- gml_Script_audio_save @ 0x140030230
function audio_save(self, other)
  gm.ini_open("settings.ini")
  gm.ini_write_real("audio", "volume_sfx", gm.real(global.volume_sfx))
  gm.ini_write_real("audio", "volume_music", gm.real(global.volume_music))
  gm.ini_close()
end

-- gml_Script_audio_get_volume @ 0x140030480
function audio_get_volume(self, other, kind)
  if kind == "music" then
    return global.volume_music
  else
    return global.volume_sfx
  end
end

-- gml_Script_audio_set_volume @ 0x1400306a0
function audio_set_volume(self, other, kind, value)
  local v = gm.clamp(gm.real(value), 0, 1.0)
  if kind == "music" then
    global.volume_music = v
    if global.music_voice ~= -1 then
      if gm.bool(gm.audio_is_playing(global.music_voice)) then
        gm.audio_sound_gain(global.music_voice, v, 0)
      end
    end
  else
    global.volume_sfx = v
  end
end

-- gml_Script_audio_step @ 0x140030bc0
function audio_step(self, other)
  rain_sound_step(self, other)
  forest_sound_step(self, other)
  ambience_step(self, other)
  sound_loops_step(self, other)
  if global.water_touch_cooldown > 0 then
    global.water_touch_cooldown = global.water_touch_cooldown - 1.0
  end
  if global.music_voice ~= -1 then
    if gm.bool(gm.audio_is_playing(global.music_voice)) then
      return
    end
  end
  local track = music_next_track(self, other)
  global.music_last = track
  global.music_voice = gm.audio_play_sound(track, 1.0, false)
  gm.audio_sound_gain(global.music_voice, global.volume_music, 0)
end

-- gml_Script_music_next_track @ 0x1400312b0
function music_next_track(self, other)
  if global.music_last == -1 then
    return global.music_tracks[0]
  end
  if not gm.is_array(global.music_bag) or gm.array_length(global.music_bag) == 0 then
    global.music_bag = gm.array_shuffle(global.music_tracks)
    if gm.is_array(global.music_bag) and gm.array_length(global.music_bag) > 1 then
      if global.music_bag[0] == global.music_last then
        local j = gm.irandom(gm.array_length(global.music_bag) - 2) + 1.0
        local first = global.music_bag[0]
        global.music_bag[0] = global.music_bag[j]
        global.music_bag[j] = first
      end
    end
  end
  local track = global.music_bag[0]
  gm.array_delete(global.music_bag, 0, 1.0)
  return track
end

-- gml_Script_rain_sound_step @ 0x140031be0
function rain_sound_step(self, other)
  local w
  if gm.bool(gm.variable_global_exists("weather")) then
    w = global.weather.intensity
  else
    w = 0
  end
  local playing = global.rain_voice ~= -1 and gm.bool(gm.audio_is_playing(global.rain_voice))
  if w <= 0 then
    if playing then
      gm.audio_stop_sound(global.rain_voice)
    end
    global.rain_voice = -1
    return
  end
  if not playing then
    global.rain_voice = gm.audio_play_sound(rain01, 2.0, true)
  end
  gm.audio_sound_gain(global.rain_voice, w * global.volume_sfx * gm.clamp(gm.real(vis(self, other, "rain_sound_volume")), 0, 1.0), 0)
end

-- gml_Script_forest_sound_step @ 0x1400325d0
function forest_sound_step(self, other)
  local w
  if gm.bool(gm.variable_global_exists("weather")) then
    w = global.weather.intensity
  else
    w = 0
  end
  local step = 1.0 / gm.max(1.0, vis(self, other, "forest_duck_frames"))
  global.forest_duck = global.forest_duck + gm.clamp(gm.real(w - global.forest_duck), -gm.real(step), gm.real(step))
  if not (global.forest_voice ~= -1 and gm.bool(gm.audio_is_playing(global.forest_voice))) then
    global.forest_voice = gm.audio_play_sound(forest, 3.0, true)
  end
  local duck_lerp = gm.lerp(1.0, vis(self, other, "forest_rain_level"), global.forest_duck)
  local v = vis(self, other, "forest_volume") * duck_lerp
  gm.audio_sound_gain(global.forest_voice, global.volume_sfx * gm.clamp(gm.real(v), 0, 1.0), 0)
end

-- gml_Script_ambience_init @ 0x140033320
function ambience_init(self, other)
  gm.audio_falloff_set_model(0)
  gm.audio_listener_position(0, 0, 0)
  if gm.bool(gm.variable_global_exists("ambience")) then
    local i = 0
    while i < gm.array_length(global.ambience) do
      gm.audio_emitter_free(global.ambience[i].emitter)
      i = i + 1
    end
  end
  global.ambience = gm.array()
  ambience_add(self, other, bird, "ambience_bird", true)
  ambience_add(self, other, creak, "ambience_creak", true)
  ambience_add(self, other, woodpecker, "ambience_woodpecker", true)
  ambience_add(self, other, leaves, "ambience_leaves", false)
end

-- gml_Script____struct___3__ambience_add__scr_audio @ 0x140033d50
function ___struct___3__ambience_add__scr_audio(self, other, sound, key, panned, emitter)
  self.sound = sound
  self.key = key
  self.panned = panned
  self.emitter = emitter
  self.timer = 0
end

-- gml_Script_ambience_add @ 0x140034030
function ambience_add(self, other, sound, key, panned)
  local a = gm.new(___struct___3__ambience_add__scr_audio, sound, key, panned, gm.audio_emitter_create())
  ambience_rearm(self, other, a)
  gm.array_push(global.ambience, a)
end

-- gml_Script_ambience_rearm @ 0x140034520
function ambience_rearm(self, other, a)
  local fps = gm.game_get_speed(0)
  local max_sec = gm.real(vis(self, other, a.key .. "_max_sec"))
  local min_sec = gm.real(vis(self, other, a.key .. "_min_sec"))
  a.timer = gm.round(gm.random_range(min_sec, max_sec) * fps)
end

-- gml_Script_ambience_step @ 0x140034b80
function ambience_step(self, other)
  local i = 0
  while i < gm.array_length(global.ambience) do
    local a = global.ambience[i]
    a.timer = a.timer - 1.0
    if a.timer <= 0 then
      ambience_rearm(self, other, a)
      if global.volume_sfx > 0 then
        local pan
        if gm.bool(a.panned) then
          pan = gm.clamp(gm.real(vis(self, other, "ambience_pan")), 0, 1.0) * gm.random_range(-1.0, 1.0)
        else
          pan = 0
        end
        gm.audio_emitter_position(a.emitter, pan, 0, gm.sqrt(1.0 - pan * pan))
        local voice = gm.audio_play_sound_on(a.emitter, a.sound, false, 4.0)
        gm.audio_sound_gain(voice, global.volume_sfx * gm.clamp(gm.real(vis(self, other, a.key .. "_volume")), 0, 1.0), 0)
      end
    end
    i = i + 1
  end
end

-- gml_Script_water_touch_sound @ 0x140035b40
function water_touch_sound(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if global.water_touch_cooldown > 0 then
    return
  end
  local voice
  if gm.is_undefined(x) then
    voice = sfx_play(self, other, water_touch)
    if voice ~= -1 then
      gm.audio_sound_gain(voice, global.volume_sfx * gm.clamp(gm.real(vis(self, other, "water_touch_volume")), 0, 1.0), 0)
    end
  else
    voice = sfx_play_at(self, other, water_touch, x, y, vis(self, other, "water_touch_volume"), gm.undefined)
  end
  if voice ~= -1 then
    global.water_touch_cooldown = vis(self, other, "water_touch_cooldown_frames")
  end
end

-- gml_Script____struct___4__hyphae_sound_begin__scr_audio @ 0x140036660
function ___struct___4__hyphae_sound_begin__scr_audio(self, other, total, placed, x, y, start_voice, middle_voice, middle_allowed, middle_at, end_at, end_sound, end_voice, fade_until)
  self.total = total
  self.placed = placed
  self.x = x
  self.y = y
  self.start_voice = start_voice
  self.middle_voice = middle_voice
  self.middle_allowed = middle_allowed
  self.middle_at = middle_at
  self.end_at = end_at
  self.end_sound = end_sound
  self.end_voice = end_voice
  self.fade_until = fade_until
  self.ended = false
  self.done = false
end

-- gml_Script_hyphae_sound_begin @ 0x140036d20
function hyphae_sound_begin(self, other, len, total, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  local start = gm.choose(hyphae_start_1, hyphae_start_2)
  local end_snd = hyphae_end_short
  if len >= vis(self, other, "hyphae_len_long") then
    end_snd = hyphae_end_long
  elseif len >= vis(self, other, "hyphae_len_mid") then
    end_snd = hyphae_end_mid
  end
  local fps = gm.game_get_speed(0)
  local middle_at = gm.round(gm.audio_sound_length(start) * 0.5 * fps)
  local end_at = gm.round(total * gm.clamp(gm.real(vis(self, other, "hyphae_end_at")), 0, 1.0))
  local tiny = (len < vis(self, other, "hyphae_len_tiny")) or (middle_at >= end_at)
  if tiny then
    end_snd = hyphae_end_short
  end
  local sx, sy
  if not gm.is_undefined(x) then sx = x else sx = 0 end
  if not gm.is_undefined(y) then sy = y else sy = 0 end
  local s = gm.new(___struct___4__hyphae_sound_begin__scr_audio, total, not gm.is_undefined(x), sx, sy, -1.0, -1.0, not tiny, middle_at, end_at, end_snd, -1.0, -1.0)
  local place_gain = hyphae_sound_place_gain(self, other, s)
  s.start_voice = hyphae_sound_play(self, other, start, false, vis(self, other, "hyphae_volume") * place_gain)
  return s
end

-- gml_Script_hyphae_sound_place_gain @ 0x1400386a0
function hyphae_sound_place_gain(self, other, s)
  if gm.bool(s.placed) then
    return sound_camera_gain(self, other, s.x, s.y)
  else
    return 1.0
  end
end

-- gml_Script_hyphae_sound_step @ 0x140038b20
function hyphae_sound_step(self, other, s, t)
  if gm.bool(s.done) then
    return
  end
  local place_gain = hyphae_sound_place_gain(self, other, s)
  local start_middle = false
  if not gm.bool(s.ended) and gm.bool(s.middle_allowed) and s.middle_voice == -1 then
    start_middle = t >= s.middle_at
  end
  if start_middle then
    s.middle_voice = hyphae_sound_play(self, other, hyphae_midlle, true, 0)
    s.middle_allowed = false
    if s.middle_voice ~= -1 then
      local len = gm.audio_sound_length(hyphae_midlle)
      local loop_start = gm.clamp(gm.real(vis(self, other, "hyphae_loop_start")), 0, gm.real(len * 0.5))
      local loop_end = gm.max(loop_start + 0.1, len - vis(self, other, "hyphae_loop_tail"))
      gm.audio_sound_loop_start(s.middle_voice, loop_start)
      gm.audio_sound_loop_end(s.middle_voice, loop_end)
    end
  end
  if gm.bool(s.placed) then
    local g = global.volume_sfx * gm.clamp(gm.real(vis(self, other, "hyphae_volume") * place_gain), 0, 1.0)
    if s.start_voice ~= -1 and gm.bool(gm.audio_is_playing(s.start_voice)) then
      gm.audio_sound_gain(s.start_voice, g, 0)
    end
    if s.end_voice ~= -1 and gm.bool(gm.audio_is_playing(s.end_voice)) then
      gm.audio_sound_gain(s.end_voice, g, 0)
    end
  end
  if not gm.bool(s.ended) and s.middle_voice ~= -1 then
    local from = s.total * vis(self, other, "hyphae_taper_from")
    local k = gm.clamp(gm.real((t - from) / gm.max(1.0, s.end_at - from)), 0, 1.0)
    local taper = gm.lerp(1.0, vis(self, other, "hyphae_taper_to"), k)
    local mv = vis(self, other, "hyphae_middle_volume") * taper * place_gain
    gm.audio_sound_gain(s.middle_voice, global.volume_sfx * gm.clamp(gm.real(mv), 0, 1.0), 0)
  end
  if not gm.bool(s.ended) and t >= s.end_at then
    s.ended = true
    if s.middle_voice ~= -1 then
      local fade = gm.max(1.0, vis(self, other, "hyphae_middle_fade_frames"))
      gm.audio_sound_gain(s.middle_voice, 0, fade / gm.game_get_speed(0) * 1000)
      s.fade_until = t + fade
    end
    s.end_voice = hyphae_sound_play(self, other, s.end_sound, false, vis(self, other, "hyphae_volume") * place_gain)
  end
  if gm.bool(s.ended) and s.middle_voice ~= -1 and t >= s.fade_until then
    gm.audio_stop_sound(s.middle_voice)
    s.middle_voice = -1
  end
  local end_playing = false
  if gm.bool(s.placed) and s.end_voice ~= -1 then
    end_playing = gm.bool(gm.audio_is_playing(s.end_voice))
  end
  s.done = gm.bool(s.ended) and s.middle_voice == -1 and not end_playing
end

-- gml_Script_hyphae_sound_stop @ 0x14003bc60
function hyphae_sound_stop(self, other, s)
  if s.middle_voice ~= -1 then
    gm.audio_stop_sound(s.middle_voice)
    s.middle_voice = -1
  end
  s.done = true
end

-- gml_Script_hyphae_sound_play @ 0x14003bfe0
function hyphae_sound_play(self, other, snd, loop, gain)
  if global.volume_sfx <= 0 then
    return -1
  end
  local voice = gm.audio_play_sound(snd, 5.0, loop)
  gm.audio_sound_gain(voice, global.volume_sfx * gm.clamp(gm.real(gain), 0, 1.0), 0)
  return voice
end

-- gml_Script_sound_camera_gain @ 0x14003c4d0
function sound_camera_gain(self, other, x, y)
  local cam = gm.view_camera[0]
  if cam == -1 then
    return 1.0
  end
  local cx = gm.camera_get_view_x(cam)
  local cy = gm.camera_get_view_y(cam)
  local cw = gm.camera_get_view_width(cam)
  local ch = gm.camera_get_view_height(cam)
  local dx = gm.max(cx - x, 0, x - (cx + cw))
  local dy = gm.max(cy - y, 0, y - (cy + ch))
  local fade = gm.max(1.0, cw * vis(self, other, "sound_camera_fade_views"))
  return gm.clamp(gm.real(1.0 - gm.point_distance(0, 0, dx, dy) / fade), 0, 1.0)
end

-- gml_Script_mushroom_grow_sound @ 0x14003d350
function mushroom_grow_sound(self, other, x, y, vol)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if vol == nil then vol = gm.undefined end
  if gm.is_undefined(vol) then
    vol = vis(self, other, "mush_grow_volume")
  end
  local snd = gm.choose(mush_grow_1, mush_grow_2, mush_grow_3)
  if gm.is_undefined(x) then
    local voice = sfx_play(self, other, snd)
    if voice ~= -1 then
      gm.audio_sound_gain(voice, global.volume_sfx * gm.clamp(gm.real(vol), 0, 1.0), 0)
    end
    return
  end
  sfx_play_at(self, other, snd, x, y, vol, vis(self, other, "sound_camera_fade_views"))
end

-- gml_Script_sfx_play @ 0x14003df30
function sfx_play(self, other, snd)
  if global.volume_sfx <= 0 then
    return -1
  end
  local voice = gm.audio_play_sound(snd, 5.0, false)
  gm.audio_sound_gain(voice, global.volume_sfx, 0)
  return voice
end

-- gml_Script_sound_emitters_init @ 0x14003e340
function sound_emitters_init(self, other)
  if gm.bool(gm.variable_global_exists("sound_emitters")) then
    local i = 0
    while i < gm.array_length(global.sound_emitters) do
      gm.audio_emitter_free(global.sound_emitters[i])
      i = i + 1
    end
  end
  global.sound_emitters = gm.array()
  global.sound_emitter_next = 0
  local i = 0
  while i < 8.0 do
    gm.array_push(global.sound_emitters, gm.audio_emitter_create())
    i = i + 1
  end
end

-- gml_Script____struct___5__sound_world_place__scr_audio @ 0x14003e980
function ___struct___5__sound_world_place__scr_audio(self, other)
  self.gain = 1.0
  self.pan = 0
end

-- gml_Script____struct___6__sound_world_place__scr_audio @ 0x14003eae0
function ___struct___6__sound_world_place__scr_audio(self, other, gain, pan)
  self.gain = gain
  self.pan = pan
end

-- gml_Script_sound_world_place @ 0x14003eca0
function sound_world_place(self, other, x, y, fade_views)
  local cam = gm.view_camera[0]
  if cam == -1 then
    return gm.new(___struct___5__sound_world_place__scr_audio)
  end
  local cx = gm.camera_get_view_x(cam)
  local cy = gm.camera_get_view_y(cam)
  local cw = gm.max(1.0, gm.camera_get_view_width(cam))
  local ch = gm.max(1.0, gm.camera_get_view_height(cam))
  local dx
  if x < cx then
    dx = x - cx
  else
    dx = gm.max(0, x - (cx + cw))
  end
  local nx = gm.abs(gm.real(dx)) / cw
  local ny = gm.max(cy - y, 0, y - (cy + ch)) / ch
  local radius = gm.max(0.01, fade_views)
  local gain = gm.clamp(gm.real(1.0 - gm.point_distance(0, 0, nx, ny) / radius), 0, 1.0)
  local pan = 0
  if dx ~= 0 then
    local edge = vis(self, other, "sound_pan_edge")
    local far = vis(self, other, "sound_pan_far")
    pan = gm.sign(gm.real(dx)) * gm.lerp(edge, far, gm.clamp(gm.real(nx / radius), 0, 1.0))
  end
  return gm.new(___struct___6__sound_world_place__scr_audio, gain, gm.clamp(gm.real(pan), -1.0, 1.0))
end

-- gml_Script_sfx_play_at @ 0x140040560
function sfx_play_at(self, other, snd, x, y, vol, reach)
  if vol == nil or gm.is_undefined(vol) then vol = 1.0 end
  if reach == nil then reach = gm.undefined end
  if global.volume_sfx <= 0 then
    return -1
  end
  if gm.is_undefined(reach) then
    reach = vis(self, other, "sound_world_reach_views")
  end
  local place = sound_world_place(self, other, x, y, reach)
  if place.gain <= 0 then
    return -1
  end
  local em = global.sound_emitters[global.sound_emitter_next]
  global.sound_emitter_next = gm.mod(global.sound_emitter_next + 1, 8)
  local pan = place.pan
  gm.audio_emitter_position(em, pan, 0, gm.sqrt(1.0 - pan * pan))
  local voice = gm.audio_play_sound_on(em, snd, false, 5.0)
  gm.audio_sound_gain(voice, global.volume_sfx * gm.clamp(gm.real(vol * place.gain), 0, 1.0), 0)
  return voice
end

-- gml_Script_sound_loops_init @ 0x140041600
function sound_loops_init(self, other)
  if gm.bool(gm.variable_global_exists("sound_loops")) then
    local i = 0
    while i < gm.array_length(global.sound_loops) do
      sound_loop_free(self, other, global.sound_loops[i])
      i = i + 1
    end
  end
  global.sound_loops = gm.array()
end

-- gml_Script____struct___7__sound_loop_start__scr_audio @ 0x1400419e0
function ___struct___7__sound_loop_start__scr_audio(self, other, sound, emitter, voice)
  self.sound = sound
  self.emitter = emitter
  self.voice = voice
  self.fade = 0
  self.idle = 0
  self.x = 0
  self.y = 0
  self.volume = 1.0
  self.reach = 1.0
  self.fade_in = 1.0
  self.fade_out = 1.0
  self.done = false
end

-- gml_Script_sound_loop_start @ 0x140041e90
function sound_loop_start(self, other, sound)
  local l = gm.new(___struct___7__sound_loop_start__scr_audio, sound, gm.audio_emitter_create(), -1.0)
  gm.array_push(global.sound_loops, l)
  return l
end

-- gml_Script_sound_loop_follow @ 0x140042300
function sound_loop_follow(self, other, l, x, y, volume, reach, fade_in, fade_out)
  if gm.bool(l.done) then
    return
  end
  l.x = x
  l.y = y
  l.volume = volume
  l.reach = reach
  l.fade_in = gm.max(1.0, fade_in)
  l.fade_out = gm.max(1.0, fade_out)
  l.idle = 0
  l.fade = gm.min(1.0, l.fade + 1.0 / l.fade_in)
  sound_loop_apply(self, other, l)
end

-- gml_Script_sound_loop_apply @ 0x140042d60
function sound_loop_apply(self, other, l)
  local place = sound_world_place(self, other, l.x, l.y, l.reach)
  local playing = false
  if l.voice ~= -1 then
    playing = gm.bool(gm.audio_is_playing(l.voice))
  end
  if not playing then
    l.voice = gm.audio_play_sound_on(l.emitter, l.sound, true, 4.0)
  end
  local pan = place.pan
  local depth = gm.sqrt(1.0 - pan * pan)
  gm.audio_emitter_position(l.emitter, pan, 0, depth)
  local level = gm.clamp(gm.real(l.volume * place.gain * l.fade), 0, 1.0)
  gm.audio_sound_gain(l.voice, global.volume_sfx * level, 0)
end

-- gml_Script_sound_loop_free @ 0x140043bb0
function sound_loop_free(self, other, l)
  if l.voice ~= -1 then
    gm.audio_stop_sound(l.voice)
    l.voice = -1
  end
  if not gm.bool(l.done) then
    gm.audio_emitter_free(l.emitter)
  end
  l.done = true
end

-- gml_Script_sound_loops_step @ 0x1400440a0
function sound_loops_step(self, other)
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(global.sound_loops) do
    local l = global.sound_loops[i]
    l.idle = l.idle + 1
    local skip = false
    if l.idle > 2.0 then
      l.fade = l.fade - 1.0 / l.fade_out
      if l.fade <= 0 then
        sound_loop_free(self, other, l)
        skip = true
      else
        sound_loop_apply(self, other, l)
      end
    end
    if not skip then
      gm.array_push(keep, l)
    end
    i = i + 1
  end
  global.sound_loops = keep
end

-- gml_GlobalScript_scr_audio @ 0x140044870
function gml_GlobalScript_scr_audio(self, other)
  self.audio_init = gm.method(self, audio_init)
  self.audio_save = gm.method(self, audio_save)
  self.audio_get_volume = gm.method(self, audio_get_volume)
  self.audio_set_volume = gm.method(self, audio_set_volume)
  self.audio_step = gm.method(self, audio_step)
  self.music_next_track = gm.method(self, music_next_track)
  self.rain_sound_step = gm.method(self, rain_sound_step)
  self.forest_sound_step = gm.method(self, forest_sound_step)
  self.ambience_init = gm.method(self, ambience_init)
  self.ambience_add = gm.method(self, ambience_add)
  self.ambience_rearm = gm.method(self, ambience_rearm)
  self.ambience_step = gm.method(self, ambience_step)
  self.water_touch_sound = gm.method(self, water_touch_sound)
  self.hyphae_sound_begin = gm.method(self, hyphae_sound_begin)
  self.hyphae_sound_place_gain = gm.method(self, hyphae_sound_place_gain)
  self.hyphae_sound_step = gm.method(self, hyphae_sound_step)
  self.hyphae_sound_stop = gm.method(self, hyphae_sound_stop)
  self.hyphae_sound_play = gm.method(self, hyphae_sound_play)
  self.sound_camera_gain = gm.method(self, sound_camera_gain)
  self.mushroom_grow_sound = gm.method(self, mushroom_grow_sound)
  self.sfx_play = gm.method(self, sfx_play)
  self.sound_emitters_init = gm.method(self, sound_emitters_init)
  self.sound_world_place = gm.method(self, sound_world_place)
  self.sfx_play_at = gm.method(self, sfx_play_at)
  self.sound_loops_init = gm.method(self, sound_loops_init)
  self.sound_loop_start = gm.method(self, sound_loop_start)
  self.sound_loop_follow = gm.method(self, sound_loop_follow)
  self.sound_loop_apply = gm.method(self, sound_loop_apply)
  self.sound_loop_free = gm.method(self, sound_loop_free)
  self.sound_loops_step = gm.method(self, sound_loops_step)
end

-- gml_Script_locale_font_file @ 0x1401913a0
function locale_font_file(self, other, code)
  if code == "zh-Hans" then
    return "fonts/ch-sim.ttf"
  elseif code == "zh-Hant" then
    return "fonts/ch-sim.ttf"
  elseif code == "ko" then
    return "fonts/korean.otf"
  elseif code == "ja" then
    return "fonts/japan.ttf"
  elseif code == "de" or code == "fr" or code == "es" or code == "it" or code == "pt-BR" or code == "ru" or code == "uk" then
    return "fonts/cyrillic-ger-spa-port.ttf"
  end
  return ""
end

-- gml_Script_ui_font @ 0x140191960
function ui_font(self, other)
  if gm.bool(gm.variable_global_exists("locale_font")) and gm.bool(gm.font_exists(global.locale_font)) then
    return global.locale_font
  end
  return Font1
end

-- gml_Script_ui_font_small @ 0x140191c60
function ui_font_small(self, other)
  if gm.bool(gm.variable_global_exists("locale_font_small")) and gm.bool(gm.font_exists(global.locale_font_small)) then
    return global.locale_font_small
  end
  return -1
end

-- gml_Script_locale_font_ink_h @ 0x140191f60
function locale_font_ink_h(self, other, fnt, str)
  local old_font = gm.draw_get_font()
  local old_color = gm.draw_get_color()
  local old_alpha = gm.draw_get_alpha()
  local old_halign = gm.draw_get_halign()
  local old_valign = gm.draw_get_valign()
  gm.draw_set_font(fnt)
  local w = gm.clamp(gm.ceil(gm.string_width(str)) + 16, 16.0, 1024.0)
  local h = gm.clamp(gm.ceil(gm.string_height(str)) * 2 + 16, 16.0, 1024.0)
  local surf = gm.surface_create(w, h)
  local result = 0
  if gm.bool(gm.surface_exists(surf)) then
    gm.surface_set_target(surf)
    gm.draw_clear_alpha(gm.c_black, 0)
    gm.draw_set_color(gm.c_white)
    gm.draw_set_alpha(1.0)
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
    gm.draw_text(8.0, 8.0, str)
    gm.surface_reset_target()
    local buf = gm.buffer_create(w * h * 4, gm.buffer_fixed, 1.0)
    gm.buffer_get_surface(buf, surf, 0)
    local top = -1
    local bottom = -1
    local yy = 0
    while yy < h do
      local xx = 0
      while xx < w do
        if gm.buffer_peek(buf, (yy * w + xx) * 4 + 3, gm.buffer_u8) > 110.0 then
          if top < 0 then
            top = yy
          end
          bottom = yy
          break
        end
        xx = xx + 1
      end
      yy = yy + 1
    end
    gm.buffer_delete(buf)
    gm.surface_free(surf)
    if top >= 0 then
      result = bottom - top + 1
    else
      result = 0
    end
  end
  gm.draw_set_font(old_font)
  gm.draw_set_color(old_color)
  gm.draw_set_alpha(old_alpha)
  gm.draw_set_halign(old_halign)
  gm.draw_set_valign(old_valign)
  return result
end

-- gml_Script_locale_font_fit @ 0x140193a60
function locale_font_fit(self, other, file, ref_font)
  local probe = "HEXN"
  local size
  if ref_font == -1 then
    size = 12.0
  else
    size = gm.font_get_size(ref_font)
  end
  local f = gm.font_add(file, size, false, false, 32.0, 127.0)
  if f < 0 then
    return -1
  end
  local old_font = gm.draw_get_font()
  gm.draw_set_font(ref_font)
  local ref_h = gm.string_height("Mg")
  gm.draw_set_font(f)
  local new_h = gm.string_height("Mg")
  gm.draw_set_font(old_font)
  if new_h > 0 then
    if gm.abs(gm.real(new_h - ref_h)) > ref_h * 0.05 then
      gm.font_delete(f)
      size = gm.max(4.0, gm.round(size * ref_h / new_h))
      f = gm.font_add(file, size, false, false, 32.0, 127.0)
    end
  end
  local ref_ink = locale_font_ink_h(self, other, ref_font, probe)
  local i = 0
  while true do
    if not (i < 2.0) then break end
    if f < 0 then break end
    if not (ref_ink > 0) then break end
    local ink = locale_font_ink_h(self, other, f, probe)
    if ink <= 0 then break end
    if gm.abs(gm.real(ink - ref_ink)) <= gm.max(1.0, ref_ink * 0.03) then break end
    gm.font_delete(f)
    size = gm.max(4.0, gm.round(size * ref_ink / ink))
    f = gm.font_add(file, size, false, false, 32.0, 127.0)
    i = i + 1
  end
  return f
end

-- gml_Script_locale_font_load @ 0x140195580
function locale_font_load(self, other, code)
  local path = locale_font_file(self, other, code)
  if gm.bool(gm.variable_global_exists("locale_font_path")) and global.locale_font_path == path then
    return
  end
  if gm.bool(gm.variable_global_exists("locale_font")) then
    if global.locale_font ~= Font1 and gm.bool(gm.font_exists(global.locale_font)) then
      gm.font_delete(global.locale_font)
    end
  end
  if gm.bool(gm.variable_global_exists("locale_font_small")) then
    if global.locale_font_small ~= -1 and gm.bool(gm.font_exists(global.locale_font_small)) then
      gm.font_delete(global.locale_font_small)
    end
  end
  global.locale_font = Font1
  global.locale_font_small = -1
  global.locale_font_path = path
  if path == "" or not gm.bool(gm.file_exists(path)) then
    return
  end
  local f = locale_font_fit(self, other, path, Font1)
  if f >= 0 then
    global.locale_font = f
  else
    global.locale_font = Font1
  end
  global.locale_font_small = locale_font_fit(self, other, path, -1.0)
end

-- gml_Script____struct___114__locale_init__scr_locale @ 0x140196210
-- (Ghidra failed to decompile this constructor; fields recovered from the disassembly: one string assignment per GML line L167-L389)
function ___struct___114__locale_init__scr_locale(self, other)
  self.hud_food = "FOOD"
  self.hud_water = "WATER"
  self.hud_length = "LENGTH"
  self.hud_points = "POINTS"
  self.hud_wind = "WIND"
  self.hud_cm = "cm"
  self.hud_in = "in"
  self.wind_tooltip = "Wind strength and direction"
  self.points_tooltip = "Upgrade points"
  self.points_till_next = "till next point"
  self.hud_next_point = "next in"
  self.button_upgrades = "UPGRADES"
  self.button_close = "CLOSE"
  self.button_add_node = "ADD NODE"
  self.button_build_mushroom = "BUILD MUSHROOM"
  self.button_release_spores = "RELEASE SPORES"
  self.button_regenerate = "REGENERATE"
  self.hint_needs_water = "NEEDS WATER"
  self.hint_too_crowded = "TOO CLOSE TO ANOTHER MUSHROOM"
  self.hint_no_nodes_in_water = "NO NODES IN WATER"
  self.perk_red = "POWER OF FAUNA SHROOM"
  self.perk_blue = "POWER OF WATER SHROOM"
  self.perk_green = "POWER OF FLORA SHROOM"
  self.perk_yellow = "POWER OF LIGHT SHROOM"
  self.perk_purple = "POWER OF COSMOS SHROOM"
  self.perk_red_desc = "Scares off squirrels forever"
  self.perk_blue_desc = "Shoots mycelia from every node in a random direction"
  self.perk_green_desc = "Reduces food price for all mushrooms by 25%"
  self.perk_yellow_desc = "Increases all production and restoration speed by 50%"
  self.perk_purple_desc = "Adds glow to your mycelia and sends a purple pulse once in a while.\n\nPress F to turn it on/off"
  self.hint_no_mutations = "UNLOCK A MUSHROOM IN THE UPGRADE TREE"
  self.button_resume = "RESUME"
  self.button_main_menu = "MAIN MENU"
  self.button_quit = "QUIT"
  self.button_black_metal = "BLACK METAL"
  self.button_units = "UNITS"
  self.units_metric = "METRIC"
  self.units_imperial = "IMPERIAL"
  self.toggle_on = "ON"
  self.toggle_off = "OFF"
  self.button_display_mode = "SCREEN"
  self.display_fullscreen = "FULLSCREEN"
  self.display_windowed = "WINDOWED"
  self.button_window_size = "WINDOW"
  self.volume_sfx = "SFX"
  self.volume_music = "MUSIC"
  self.title_new_game = "New Game"
  self.title_exploration = "Exploration Mode"
  self.title_survival = "Survival Mode"
  self.mode_exploration = "Exploration"
  self.mode_survival = "Survival"
  self.title_load_game = "Load Game"
  self.slot_title_new = "CHOOSE A SLOT"
  self.slot_title_load = "LOAD GAME"
  self.slot_label = "Slot"
  self.slot_empty = "Empty"
  self.slot_back = "Back"
  self.slot_name_prompt = "NAME YOUR MYCELIUM"
  self.slot_name_hint = "ENTER to start, ESC to go back"
  self.slot_overwrites = "Overwrites"
  self.slot_default_name = "Mycelium"
  self.title_settings = "Settings"
  self.title_credits = "Credits"
  self.credits_plan_kapkan = "plan kapkan"
  self.credits_art_ui = "art & ui"
  self.credits_smm_qa = "smm & qa"
  self.credits_forest_assets = "forest assets"
  self.title_exit = "Exit"
  self.tut_shoot = "Shoot a hypha from a mushroom."
  self.tut_node = "Right-click a hypha branch to build a node for another shot (Alt + click: at once)."
  self.tut_salt = "Eat salt to build nodes and mushrooms."
  self.tut_water = "Drink water to shoot hyphae."
  self.tut_xp = "Your mycelia size is your experience."
  self.tut_upgrade = "Upgrade your mycelia."
  self.tut_mushrooms = "Build mushrooms on the ground for different purposes."
  self.tut_rain = "Hyphae shots don't cost any water during the rain."
  self.tut_clay = "This is clay. Your shots are shorter here."
  self.tut_final = "EAT. DRINK. GROW."
  self.tasks_title = "EVOLUTION TASKS"
  self.goal_tasks_title = "CONGRATULATION!"
  self.goal_tasks_text = "You've finished all evolution tasks.\nNow you are ready for something bigger.\nAchieving each can result in finishing the game\n(but you can keep playing afterwards nonetheless)."
  self.goal_migration_title = "THE\nGREAT\nMIGRATION"
  self.goal_magic_title = "FIND ALL\nFIVE MAGIC\nMUSHROOMS"
  self.goal_migration_label = "THE GREAT MIGRATION"
  self.goal_magic_label = "FIVE MAGIC MUSHROOMS"
  self.goal_migration_done = "Congrats! You are ready for Great Migration!"
  self.goal_magic_done = "Congrats! You have collected 5 magic mushrooms!\nNow you are a superior organism in the forest!"
  self.goal_button_grow = "Let's grow!"
  self.goal_button_spores = "Shoot spores"
  self.goal_item_spores = "Spore mushrooms"
  self.goal_item_water = "Water"
  self.goal_item_food = "Food"
  self.goal_item_green = "Flora Shroom"
  self.goal_item_yellow = "Light Shroom"
  self.goal_item_purple = "Cosmos Shroom"
  self.goal_item_blue = "Water Shroom"
  self.goal_item_red = "Fauna Shroom"
  self.goal_button_exit = "Main menu"
  self.goal_button_continue = "Continue"
  self.task_absorption = "Upgrade Absorption"
  self.task_shot_range = "Upgrade shot range"
  self.task_water = "Drink 1000 water from underground pools"
  self.task_rain_point = "Gain 1 upgrade point during the rain"
  self.task_spores = "Shoot spores"
  self.task_food = "Produce 30 food"
  self.task_build = "Build 5 Mushrooms"
  self.task_hedgehog = "Scare off the Hedgehog"
  self.task_grow = "Grow 10000 cm"
  self.task_trees = "Build 2 mushrooms on trees"
  self.task_stinky = "Build 2 Stinky mushrooms"
  self.pause_hint = "ESC to resume"
  self.trunk_hyphae = "HYPHAE"
  self.trunk_fruiting = "FRUITING BODIES"
  self.trunk_enzymes = "ENZYMES"
  self.upgrade_level = "LV"
  self.upgrade_max = "MAX"
  self.upgrade_cost = "COST"
  self.upgrade_locked = "LOCKED"
  self.upgrade_coming_later = "COMING LATER"
  self.upgrade_no_points = "NOT ENOUGH POINTS"
  self.build_none = "UNLOCK MUSHROOMS IN THE UPGRADE TREE"
  self.upgrade_demo_locked = "NOT AVAILABLE IN THE DEMO"
  self.upgrade_requires = "NEEDS"
  self.upgrade_grow = "CLICK TO GROW"
  self.tree_hint = "DRAG TO PAN WHEEL TO ZOOM CLICK A NODE TO GROW IT"
  self.tree_recentre = "RECENTRE"
  self.button_logo = "LOGO"
  self.logo_title = "BLACK METAL BAND LOGO GENERATOR"
  self.logo_typing_hint = "ENTER TO GROW IT BACKSPACE TO ERASE LATIN LETTERS ONLY"
  self.logo_grown_hint = "REROLL FOR ANOTHER GROWTH EDIT TO RENAME"
  self.logo_empty = "TYPE SOMETHING UNHOLY"
  self.logo_caret = "_"
  self.button_reroll = "REROLL"
  self.button_grow = "GROW"
  self.button_edit = "EDIT"
  self.button_clear = "CLEAR"
  self.upg_origin = "Origin"
  self.upg_origin_desc = "Where the network started. Everything grows down from here."
  self.upg_symbiosis = "Symbiosis"
  self.upg_symbiosis_desc = "A future trunk. Nothing to grow here yet."
  self.upg_shot_range = "Shot Range"
  self.upg_shot_range_desc = "Mycelium reaches further with every shot."
  self.upg_shot_width = "Shot Width"
  self.upg_shot_width_desc = "Growth fans out wider to the sides."
  self.upg_water_efficiency = "Water Efficiency"
  self.upg_water_efficiency_desc = "Each shot spends less water."
  self.upg_node_efficiency = "Node Efficiency"
  self.upg_node_efficiency_desc = "Building a node costs less food."
  self.upg_budding_chance = "Budding Chance"
  self.upg_budding_chance_desc = "Chance a free node buds at the tip of a shot."
  self.upg_soil_penetration = "Soil Penetration"
  self.upg_soil_penetration_desc = "Push through dense soil with less resistance."
  self.upg_unlock_spore = "Spore Mushroom"
  self.upg_unlock_spore_desc = "Allows building spore mushrooms on the surface."
  self.upg_spore_count = "Spore Count"
  self.upg_spore_count_desc = "More spores released per puff."
  self.upg_spore_lightness = "Spore Lightness"
  self.upg_spore_lightness_desc = "Lighter spores travel further on the wind."
  self.upg_spore_quality = "Spore Quality"
  self.upg_spore_quality_desc = "Higher chance a landed spore roots into a node."
  self.upg_unlock_predator = "Predator Mushroom"
  self.upg_unlock_predator_desc = "Allows building predator mushrooms that trap flies."
  self.upg_predator_digestion = "Digestion Speed"
  self.upg_predator_digestion_desc = "Predator mushrooms bring in food more often."
  self.upg_predator_yield = "Predator Yield"
  self.upg_predator_yield_desc = "More food from every caught fly."
  self.upg_unlock_repeller = "Repeller Mushroom"
  self.upg_unlock_repeller_desc = "Allows building repeller mushrooms that scare animals."
  self.upg_unlock_experience = "Experience Mushroom"
  self.upg_unlock_experience_desc = "Allows building experience mushrooms. Each gives an upgrade point every three minutes; every next mushroom is more expensive. Crows find them a treat and keep coming for them - guard them well."
  self.upg_unlock_stinky = "Stinky Mushroom"
  self.upg_unlock_stinky_desc = "Allows building stinky mushrooms. A red stench rises from them that drives off squirrels and crows."
  self.upg_stinky_width = "Stench Width"
  self.upg_stinky_width_desc = "The stench of stinky mushrooms rises in a wider funnel, and at the top level a wider column, keeping squirrels and crows further off."
  self.upg_repeller_autoregen = "Repeller Auto Regen"
  self.upg_repeller_autoregen_desc = "A spent repeller recharges by itself: in 5 minutes, one less per level, down to 1."
  self.upg_repeller_regen_cost = "Repeller Regen Cost"
  self.upg_repeller_regen_cost_desc = "Regenerating a spent repeller costs less food."
  self.upg_unlock_tree = "Water Mushroom"
  self.upg_unlock_tree_desc = "Allows building water mushrooms on tree trunks."
  self.upg_tree_water_yield = "Water Yield"
  self.upg_tree_water_yield_desc = "More water from every drop."
  self.upg_tree_drop_rate = "Drop Rate"
  self.upg_tree_drop_rate_desc = "Drops fall more often."
  self.upg_unlock_digger = "Digger Mushroom"
  self.upg_unlock_digger_desc = "Allows building digger mushrooms. Each one fires ten thick strands through the soil, breaking through stone."
  self.upg_digger_length = "Digger Length"
  self.upg_digger_length_desc = "Digger mushroom strands grow longer, up to three times the length."
  self.upg_unlock_absorption = "Absorption"
  self.upg_unlock_absorption_desc = "Mycelium reaches out to fallen leaves and decomposes them."
  self.upg_absorption_speed = "Absorption Speed"
  self.upg_absorption_speed_desc = "Decompose leaves faster."
  self.upg_absorption_yield = "Absorption Yield"
  self.upg_absorption_yield_desc = "More food from every leaf."
  self.upg_unlock_protein = "Protein Absorption"
  self.upg_unlock_protein_desc = "Mycelium wraps earthworms that crawl close, holds them still and digests them."
  self.upg_unlock_parasitism = "Consumptor Mushroom"
  self.upg_unlock_parasitism_desc = "Allows building consumptor mushrooms on trees, stumps and fallen trees. They slowly eat them away into food; the more on one, the sooner it is gone."
  self.upg_parasitism_speed = "Consumptor Speed"
  self.upg_parasitism_speed_desc = "Consumptor mushrooms eat faster."
  self.upg_parasitism_yield = "Consumptor Yield"
  self.upg_parasitism_yield_desc = "More food from everything consumptor mushrooms eat."
  self.button_language = "LANGUAGE"
  self.language_name = "ENGLISH"
  self.debug_title = "BALANCE (F1 to close)"
end

-- gml_Script_locale_init @ 0x14019a570
function locale_init(self, other)
  global.locale_id = "en"
  global.locale_en = gm.new(___struct___114__locale_init__scr_locale)
  global.locale_table = locale_load_csv(self, other, "localization.csv")
  global.locale_langs = locale_available(self, other)
  gm.ini_open("settings.ini")
  local lang = gm.ini_read_string("game", "language", "")
  gm.ini_close()
  local code
  if lang == "" then
    code = locale_system(self, other)
  else
    code = lang
  end
  locale_set(self, other, code)
end

-- gml_Script_locale_load_csv @ 0x14019add0
function locale_load_csv(self, other, path)
  local table_ = gm.struct({})
  if not gm.bool(gm.file_exists(path)) then
    return table_
  end
  local f = gm.file_text_open_read(path)
  local rows = gm.array()
  local buf = ""
  local delim = ","
  local first = true
  while not gm.bool(gm.file_text_eof(f)) do
    local line = gm.file_text_read_string(f)
    gm.file_text_readln(f)
    if first then
      if gm.string_length(line) > 0 then
        if gm.ord(gm.string_char_at(line, 1.0)) == 65279.0 then
          line = gm.string_delete(line, 1.0, 1.0)
        end
      end
      if gm.string_count(";", line) > gm.string_count(",", line) then
        delim = ";"
      end
      first = false
    end
    if buf == "" then
      buf = line
    else
      buf = buf .. "\n" .. line
    end
    if math.fmod(gm.real(gm.string_count("\"", buf)), 2) == 0.0 then
      gm.array_push(rows, locale_csv_split(self, other, buf, delim))
      buf = ""
    end
  end
  gm.file_text_close(f)
  if not (gm.is_array(rows) and gm.array_length(rows) ~= 0) then
    return table_
  end
  local header = rows[0]
  local key_col = -1
  local i = 0
  while i < gm.array_length(header) do
    header[i] = gm.string_trim(header[i])
    if header[i] == "key" then
      key_col = i
    end
    i = i + 1
  end
  if not (key_col >= 0) then
    return table_
  end
  i = 0
  while i < gm.array_length(header) do
    if i ~= key_col and header[i] ~= "" and header[i] ~= "context" then
      gm.variable_struct_set(table_, header[i], gm.struct({}))
    end
    i = i + 1
  end
  local r = 1.0
  while r < gm.array_length(rows) do
    local row = rows[r]
    if key_col < gm.array_length(row) then
      local key = gm.string_trim(row[key_col])
      if key ~= "" then
        local c = 0
        while c < gm.min(gm.array_length(row), gm.array_length(header)) do
          local lang = header[c]
          if gm.bool(gm.variable_struct_exists(table_, lang)) and row[c] ~= "" then
            gm.variable_struct_set(gm.variable_struct_get(table_, lang), key, gm.string_replace_all(row[c], "\\n", "\n"))
          end
          c = c + 1
        end
      end
    end
    r = r + 1
  end
  return table_
end

-- gml_Script_locale_csv_split @ 0x14019d270
function locale_csv_split(self, other, line, delim)
  local out = gm.array()
  local n = gm.string_byte_length(line)
  local src = gm.buffer_create(n + 1, gm.buffer_fixed, 1.0)
  local tmp = gm.buffer_create(n + 1, gm.buffer_fixed, 1.0)
  local d = gm.ord(delim)
  local q = 34.0
  local inq = false
  local start = 0
  gm.buffer_write(src, gm.buffer_text, line)
  local i = 0
  while i <= n do
    local c
    if i >= n then
      c = -1
    else
      c = gm.buffer_peek(src, i, gm.buffer_u8)
    end
    if inq then
      if c == q then
        inq = false
      end
    elseif c == q then
      inq = true
    elseif c == d or c == -1 then
      local len = i - start
      local cell = ""
      if len > 0 then
        gm.buffer_copy(src, start, len, tmp, 0)
        gm.buffer_poke(tmp, len, gm.buffer_u8, 0)
        cell = gm.buffer_peek(tmp, 0, gm.buffer_string)
      end
      if gm.string_length(cell) >= 2.0 and gm.string_char_at(cell, 1.0) == "\"" and gm.string_char_at(cell, gm.string_length(cell)) == "\"" then
        cell = gm.string_replace_all(gm.string_copy(cell, 2.0, gm.string_length(cell) - 2), "\"\"", "\"")
      end
      gm.array_push(out, cell)
      start = i + 1
    end
    i = i + 1
  end
  gm.buffer_delete(src)
  gm.buffer_delete(tmp)
  return out
end

-- gml_Script_locale_available @ 0x14019ef30
function locale_available(self, other)
  local out = gm.array("en")
  local names = gm.variable_struct_get_names(global.locale_table)
  local order = gm.array("en", "de", "fr", "es", "it", "pt-BR", "ru", "uk", "zh-Hant", "zh-Hans", "ko", "ja")
  local i = 0
  while i < gm.array_length(names) do
    if not gm.bool(gm.array_contains(order, names[i])) then
      gm.array_push(order, names[i])
    end
    i = i + 1
  end
  i = 1.0
  while i < gm.array_length(order) do
    local t = gm.variable_struct_get(global.locale_table, order[i])
    if gm.bool(gm.is_struct(t)) and gm.array_length(gm.variable_struct_get_names(t)) > 1 then
      gm.array_push(out, order[i])
    end
    i = i + 1
  end
  return out
end

-- gml_Script_locale_system @ 0x14019fad0
function locale_system(self, other)
  local lang = gm.os_get_language()
  local region = gm.string_upper(gm.os_get_region())
  local code = lang
  if lang == "pt" then
    code = "pt-BR"
  elseif lang == "zh" then
    if region == "TW" or region == "HK" or region == "MO" then
      code = "zh-Hant"
    else
      code = "zh-Hans"
    end
  end
  if gm.bool(gm.array_contains(global.locale_langs, code)) then
    return code
  else
    return "en"
  end
end

-- gml_Script_locale_set @ 0x1401a0390
function locale_set(self, other, code)
  if not gm.bool(gm.array_contains(global.locale_langs, code)) then
    code = "en"
  end
  global.locale_id = code
  global.locale_strings = gm.struct({})
  locale_font_load(self, other, code)
  local names = gm.variable_struct_get_names(global.locale_en)
  local i = 0
  while i < gm.array_length(names) do
    gm.variable_struct_set(global.locale_strings, names[i], gm.variable_struct_get(global.locale_en, names[i]))
    i = i + 1
  end
  local chain
  if code == "en" then
    chain = gm.array("en")
  else
    chain = gm.array("en", code)
  end
  local j = 0
  while j < gm.array_length(chain) do
    local t = gm.variable_struct_get(global.locale_table, chain[j])
    if gm.bool(gm.is_struct(t)) then
      local keys = gm.variable_struct_get_names(t)
      local k = 0
      while k < gm.array_length(keys) do
        gm.variable_struct_set(global.locale_strings, keys[k], gm.variable_struct_get(t, keys[k]))
        k = k + 1
      end
    end
    j = j + 1
  end
end

-- gml_Script_locale_cycle @ 0x1401a1670
function locale_cycle(self, other, dir)
  local n = gm.array_length(global.locale_langs)
  local cur = 0
  local i = 0
  while i < n do
    if global.locale_langs[i] == global.locale_id then
      cur = i
    end
    i = i + 1
  end
  locale_set(self, other, global.locale_langs[gm.mod(cur + dir + n, n)])
  gm.ini_open("settings.ini")
  gm.ini_write_string("game", "language", global.locale_id)
  gm.ini_close()
end

-- gml_Script_loc @ 0x1401a1da0
function loc(self, other, key)
  local s = gm.variable_struct_get(global.locale_strings, key)
  if gm.is_undefined(s) then
    return key
  end
  return s
end

-- gml_GlobalScript_scr_locale @ 0x1401a2090
function gml_GlobalScript_scr_locale(self, other)
  self.locale_font_file = gm.method(self, locale_font_file)
  self.ui_font = gm.method(self, ui_font)
  self.ui_font_small = gm.method(self, ui_font_small)
  self.locale_font_ink_h = gm.method(self, locale_font_ink_h)
  self.locale_font_fit = gm.method(self, locale_font_fit)
  self.locale_font_load = gm.method(self, locale_font_load)
  self.locale_init = gm.method(self, locale_init)
  self.locale_load_csv = gm.method(self, locale_load_csv)
  self.locale_csv_split = gm.method(self, locale_csv_split)
  self.locale_available = gm.method(self, locale_available)
  self.locale_system = gm.method(self, locale_system)
  self.locale_set = gm.method(self, locale_set)
  self.locale_cycle = gm.method(self, locale_cycle)
  self.loc = gm.method(self, loc)
end

-- gml_Script____struct___119_____struct___118__logo_init__scr_logo @ 0x1401a2930
function ___struct___119_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.56
  self.s = s
end

-- gml_Script____struct___120_____struct___118__logo_init__scr_logo @ 0x1401a2ad0
function ___struct___120_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.56
  self.s = s
end

-- gml_Script____struct___121_____struct___118__logo_init__scr_logo @ 0x1401a2c70
function ___struct___121_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.56
  self.s = s
end

-- gml_Script____struct___122_____struct___118__logo_init__scr_logo @ 0x1401a2e10
function ___struct___122_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.58
  self.s = s
end

-- gml_Script____struct___123_____struct___118__logo_init__scr_logo @ 0x1401a2fb0
function ___struct___123_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.54
  self.s = s
end

-- gml_Script____struct___124_____struct___118__logo_init__scr_logo @ 0x1401a3150
function ___struct___124_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.52
  self.s = s
end

-- gml_Script____struct___125_____struct___118__logo_init__scr_logo @ 0x1401a32f0
function ___struct___125_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.58
  self.s = s
end

-- gml_Script____struct___126_____struct___118__logo_init__scr_logo @ 0x1401a3490
function ___struct___126_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.58
  self.s = s
end

-- gml_Script____struct___127_____struct___118__logo_init__scr_logo @ 0x1401a3630
function ___struct___127_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.28
  self.s = s
end

-- gml_Script____struct___128_____struct___118__logo_init__scr_logo @ 0x1401a37d0
function ___struct___128_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.48
  self.s = s
end

-- gml_Script____struct___129_____struct___118__logo_init__scr_logo @ 0x1401a3970
function ___struct___129_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.58
  self.s = s
end

-- gml_Script____struct___130_____struct___118__logo_init__scr_logo @ 0x1401a3b10
function ___struct___130_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.52
  self.s = s
end

-- gml_Script____struct___131_____struct___118__logo_init__scr_logo @ 0x1401a3cb0
function ___struct___131_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.78
  self.s = s
end

-- gml_Script____struct___132_____struct___118__logo_init__scr_logo @ 0x1401a3e50
function ___struct___132_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.62
  self.s = s
end

-- gml_Script____struct___133_____struct___118__logo_init__scr_logo @ 0x1401a3ff0
function ___struct___133_____struct___118__logo_init__scr_logo(self, other, s)
  self.w = 0.58
  self.s = s
end
