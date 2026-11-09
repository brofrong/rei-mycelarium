-- GameMaker audio on love.audio: sounds, instances, gains with fades, emitters,
-- loop points (emulated) and the listener.
local assets = require("gm.assets")

local M = {}
local sources = {}        -- asset id -> prototype Source
local asset_gain = {}     -- asset id -> gain set by audio_sound_gain on the asset
local instances, next_inst = {}, 200000
local emitters, next_emitter = {}, 0
local loops = {}          -- asset id -> {start, finish}

local function proto(snd)
  local s = sources[snd]
  if not s then
    local def = assets.sounds[snd]
    s = love.audio.newSource(def.path, def.streamed and "stream" or "static")
    sources[snd] = s
  end
  return s
end

local function base_volume(snd) return (assets.sounds[snd].volume or 1) * (asset_gain[snd] or 1) end

local function play(snd, loop, gain, offset, pitch, emitter)
  local src = proto(snd):clone()
  local inst = { id = next_inst, snd = snd, src = src, gain = gain or 1, fade = nil, emitter = emitter }
  next_inst = next_inst + 1
  src:setLooping(loop and loop ~= 0 and not loops[snd])
  inst.loop = loop and loop ~= 0
  if pitch and pitch > 0 then src:setPitch(pitch) end
  if emitter and src:getChannelCount() == 1 then
    src:setRelative(false)
    src:setPosition(emitter.x, emitter.y, emitter.z)
    src:setAttenuationDistances(emitter.ref or 1, emitter.max or 100000)
    src:setRolloff(emitter.factor or 1)
  end
  src:setVolume(base_volume(snd) * inst.gain * (emitter and emitter.gain or 1))
  if offset and offset > 0 then src:seek(offset) end
  src:play()
  instances[inst.id] = inst
  return inst.id
end

function M.audio_play_sound(snd, prio, loop, gain, offset, pitch) return play(snd, loop, gain, offset, pitch) end
function M.audio_play_sound_on(em, snd, loop, prio, gain, offset, pitch) return play(snd, loop, gain, offset, pitch, emitters[em]) end

local function targets(id)
  if instances[id] then return { instances[id] } end
  local out = {}
  for _, inst in pairs(instances) do if inst.snd == id then out[#out + 1] = inst end end
  return out
end

function M.audio_is_playing(id)
  for _, inst in ipairs(targets(id)) do if inst.src:isPlaying() then return true end end
  return false
end

function M.audio_stop_sound(id)
  for _, inst in ipairs(targets(id)) do inst.src:stop(); instances[inst.id] = nil end
end

function M.audio_sound_gain(id, gain, ms)
  if not instances[id] and assets.sounds[id] then asset_gain[id] = gain end
  for _, inst in ipairs(targets(id)) do
    if ms and ms > 0 then
      inst.fade = { from = inst.gain, to = gain, t = 0, dur = ms / 1000 }
    else
      inst.gain, inst.fade = gain, nil
    end
  end
end

function M.audio_sound_length(id)
  local snd = instances[id] and instances[id].snd or id
  return proto(snd):getDuration("seconds")
end

function M.audio_sound_loop_start(id, pos)
  local snd = instances[id] and instances[id].snd or id
  loops[snd] = loops[snd] or { start = 0, finish = 0 }
  loops[snd].start = pos
end
function M.audio_sound_loop_end(id, pos)
  local snd = instances[id] and instances[id].snd or id
  loops[snd] = loops[snd] or { start = 0, finish = 0 }
  loops[snd].finish = pos
end

function M.audio_emitter_create()
  local id = next_emitter
  next_emitter = next_emitter + 1
  emitters[id] = { x = 0, y = 0, z = 0, gain = 1 }
  return id
end
function M.audio_emitter_free(id) emitters[id] = nil end
function M.audio_emitter_position(id, x, y, z)
  local e = emitters[id]
  e.x, e.y, e.z = x, y, z
  for _, inst in pairs(instances) do
    if inst.emitter == e and inst.src:getChannelCount() == 1 then inst.src:setPosition(x, y, z) end
  end
end
function M.audio_emitter_falloff(id, ref, max, factor) local e = emitters[id]; e.ref, e.max, e.factor = ref, max, factor end
function M.audio_emitter_gain(id, g) emitters[id].gain = g end
function M.audio_listener_position(x, y, z) love.audio.setPosition(x, y, z) end

M.audio_falloff_none, M.audio_falloff_inverse_distance, M.audio_falloff_inverse_distance_clamped = 0, 1, 2
M.audio_falloff_linear_distance, M.audio_falloff_linear_distance_clamped = 3, 4
M.audio_falloff_exponent_distance, M.audio_falloff_exponent_distance_clamped = 5, 6
local MODELS = { [0] = "none", "inverse", "inverseclamped", "linear", "linearclamped", "exponent", "exponentclamped" }
function M.audio_falloff_set_model(m) love.audio.setDistanceModel(MODELS[m] or "inverseclamped") end

function M.update(dt)
  for id, inst in pairs(instances) do
    local src = inst.src
    if inst.fade then
      local f = inst.fade
      f.t = f.t + dt
      local k = math.min(1, f.t / f.dur)
      inst.gain = f.from + (f.to - f.from) * k
      if k >= 1 then inst.fade = nil end
    end
    local lp = loops[inst.snd]
    if inst.loop and lp then
      local finish = lp.finish > 0 and lp.finish or src:getDuration("seconds")
      if src:tell() >= finish or not src:isPlaying() then
        src:seek(lp.start)
        if not src:isPlaying() then src:play() end
      end
    end
    src:setVolume(base_volume(inst.snd) * inst.gain * (inst.emitter and inst.emitter.gain or 1))
    if not src:isPlaying() and not inst.loop then instances[id] = nil end
  end
end

return M
