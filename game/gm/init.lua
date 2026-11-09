-- The full GML compatibility layer (`local gm = require("gm")`) and the host loop
-- that runs the game like the GameMaker runner: fixed 60 steps per second,
-- draw phases pre -> world (application surface through view 0) -> post -> gui.
local gm = require("gm.core")
local value = require("gm.value")
local assets = require("gm.assets")
local display = require("gm.display")
local draw = require("gm.draw")
local window = require("gm.window")
local input = require("gm.input")
local audio = require("gm.audio")
local file = require("gm.file")
local steam = require("gm.steam")
local mathx = require("gm.mathx")
local json = require("gm.json")
local random = require("gm.random")
local vertex = require("gm.vertex")

for _, mod in ipairs({ mathx, draw, window, input, audio, file, steam, random, vertex }) do
  for k, v in pairs(mod) do
    if gm[k] == nil then gm[k] = v end
  end
end
gm.assets, gm.json, gm.display = assets, json, display
window.share(gm)
gm.draw_self = draw.draw_self
gm.mouse_x, gm.mouse_y = 0, 0

function gm.buffer_get_surface(buf, surf, offset) file.buffer_get_surface(buf, surf, offset, draw) end
function gm.buffer_set_surface(buf, surf, offset) file.buffer_set_surface(buf, surf, offset, draw) end
function gm.string_byte_length(s) return #s end

function gm.variable_global_exists(name) return gm.global[name] ~= nil end
function gm.variable_global_get(name)
  local v = gm.global[name]
  if v == nil then return value.undefined end
  return v
end
function gm.variable_global_set(name, v) gm.global[name] = v end

function gm.asset_get_index(name)
  return assets.sprite_by_name[name] or assets.sound_by_name[name] or assets.font_by_name[name]
    or assets.shader_by_name[name] or (assets.room_by_name[name] and name) or (gm.object_get(name)) or -1
end

-- Sprite animation after End Step (image_speed is in sprite frames per step or per second).
local game_speed = 60
function gm.animate(instances)
  for _, inst in ipairs(instances) do
    local spr = inst.sprite_index
    if not inst.__destroyed and spr and spr >= 0 and assets.sprites[spr] then
      local s = assets.sprites[spr]
      local per_step = s.speed_type == 1 and s.speed or s.speed / game_speed
      inst.image_index = inst.image_index + inst.image_speed * per_step
      local n = #s.frames
      if inst.image_index >= n then
        inst.image_index = inst.image_index - n
        gm.event_perform(inst, "Other_7") -- Animation End
      end
    end
  end
end

-- Host ---------------------------------------------------------------------------
local lg = love.graphics
local acc = 0
local t_start = love.timer.getTime()
gm.current_time = 0

local function refresh_display()
  local app = draw.surface_get_canvas(gm.application_surface)
  display.update(app:getWidth(), app:getHeight(), display.gui_w, display.gui_h)
end

function gm.boot(start_room)
  gm.boot_assets()
  gm.boot_room(start_room)
end

function gm.boot_assets()
  assets.load(_G)
  game_speed = assets.game.speed or 60
  for _, room in pairs(assets.rooms) do gm.define_room(room.name, room) end
  display.init()
  local ww, wh = display.window_size()
  gm.application_surface = draw.surface_create(math.max(ww, 1), math.max(wh, 1))
  _G.application_surface = gm.application_surface
  refresh_display()
  gm.on_room_start = function(_, def) window.room_views(def) end
  gm.on_game_end = function()
    gm.__ended = true
    love.event.quit()
  end
end

function gm.boot_room(start_room)
  gm.apply_room(start_room or assets.game.room_order[1])
end

function gm.update(dt)
  local step = 1 / game_speed
  acc = math.min(acc + dt, step * 5)
  while acc >= step do
    refresh_display()
    gm.current_time = math.floor((love.timer.getTime() - t_start) * 1000)
    window.update_mouse(gm)
    gm.step()
    input.end_step()
    audio.update(step)
    acc = acc - step
  end
end

local function fill_background_layers()
  for _, l in ipairs(gm.room_def and gm.room_def.layers or {}) do
    local bg = l.background
    if bg and bg.visible and l.visible and bg.sprite == nil then
      local a = math.floor(bg.color / 0x1000000) % 256 / 255
      if a > 0 then
        local r, g, b = draw.bgr_to_rgb(bg.color % 0x1000000)
        local cam = window.cameras[window.view_camera[0]]
        lg.setColor(r, g, b, a)
        if cam then lg.rectangle("fill", cam.x, cam.y, cam.w, cam.h) else lg.rectangle("fill", 0, 0, gm.room_width, gm.room_height) end
      end
    end
  end
end

function gm.render()
  refresh_display()
  local app = draw.surface_get_canvas(gm.application_surface)
  local identity = love.math.newTransform()
  gm.draw(function(phase)
    if phase == "pre" then
      lg.setCanvas()
      lg.origin()
      draw.world_base = identity
    elseif phase == "world" then
      lg.setCanvas({ app, stencil = true })
      local t = window.world_transform()
      lg.replaceTransform(t)
      draw.world_base = t
      fill_background_layers()
    elseif phase == "post" then
      lg.setCanvas()
      lg.origin()
      draw.world_base = identity
      if gm.surface_draw_enabled then display.present(app) end
    elseif phase == "gui" then
      lg.setCanvas()
      local f = display.fit
      local t = love.math.newTransform(f.x, f.y, 0, f.w / display.gui_w, f.h / display.gui_h)
      lg.replaceTransform(t)
      draw.world_base = t
    end
  end)
  lg.setCanvas()
  lg.origin()
  lg.setColor(1, 1, 1, 1)
end

-- love callbacks the host forwards
gm.keyboard_string = ""
function gm.keypressed(key)
  input.key_event(key, true)
  if key == "backspace" then
    local chars = value.utf8_chars(gm.keyboard_string)
    chars[#chars] = nil
    gm.keyboard_string = table.concat(chars)
  end
end
function gm.textinput(t) gm.keyboard_string = string.sub(gm.keyboard_string .. t, -1024) end
function gm.keyreleased(key) input.key_event(key, false) end
function gm.mousepressed(_, _, button) input.mouse_event(button, true) end
function gm.mousereleased(_, _, button) input.mouse_event(button, false) end
function gm.wheelmoved(_, dy) input.wheel_event(dy) end
function gm.quit()
  if not gm.__ended then
    gm.__ended = true
    gm.game_end()
  end
  return false
end

return gm
