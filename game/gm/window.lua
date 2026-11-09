-- Window, display, cameras/views and mouse coordinates (GameMaker API).
local display = require("gm.display")
local value = require("gm.value")

local M = {}
-- Flags the game writes as builtin variables (gm.view_enabled = true) live in a
-- shared table; init.lua points it at the gm table.
local S = M
function M.share(t) S = t end

-- Cameras and views -----------------------------------------------------------
local cameras, next_camera = {}, 0
M.cameras = cameras
M.view_camera = value.array_create(8, -1)
M.view_enabled = false
M.view_visible = value.array_create(8, false)
M.ports = {}
for i = 0, 7 do M.ports[i] = { x = 0, y = 0, w = 1920, h = 1080 } end

function M.camera_create_view(x, y, w, h, angle, obj, xs, ys, xb, yb)
  local id = next_camera
  next_camera = next_camera + 1
  cameras[id] = { x = x, y = y, w = w, h = h, angle = angle or 0 }
  return id
end
function M.camera_create() return M.camera_create_view(0, 0, 1920, 1080) end
function M.camera_destroy(id) cameras[id] = nil end
function M.camera_get_view_x(id) return cameras[id].x end
function M.camera_get_view_y(id) return cameras[id].y end
function M.camera_get_view_width(id) return cameras[id].w end
function M.camera_get_view_height(id) return cameras[id].h end
function M.camera_set_view_pos(id, x, y) local c = cameras[id]; c.x, c.y = x, y end
function M.camera_set_view_size(id, w, h) local c = cameras[id]; c.w, c.h = w, h end
function M.camera_set_view_angle(id, a) cameras[id].angle = a end

function M.view_set_camera(i, cam) M.view_camera[i] = cam end
function M.view_get_camera(i) return M.view_camera[i] end
function M.view_set_visible(i, v) M.view_visible[i] = v end
function M.view_set_xport(i, v) M.ports[i].x = v end
function M.view_set_yport(i, v) M.ports[i].y = v end
function M.view_set_wport(i, v) M.ports[i].w = v end
function M.view_set_hport(i, v) M.ports[i].h = v end
function M.view_get_xport(i) return M.ports[i].x end
function M.view_get_yport(i) return M.ports[i].y end
function M.view_get_wport(i) return M.ports[i].w end
function M.view_get_hport(i) return M.ports[i].h end

-- Set up views from a room definition (rooms.json).
function M.room_views(def)
  S.view_enabled = def.views_enabled
  for i, v in ipairs(def.views or {}) do
    local idx = i - 1
    if v.enabled then
      if M.view_camera[idx] == -1 or cameras[M.view_camera[idx]] == nil then
        M.view_camera[idx] = M.camera_create_view(v.view[1], v.view[2], v.view[3], v.view[4])
      else
        local c = cameras[M.view_camera[idx]]
        c.x, c.y, c.w, c.h = v.view[1], v.view[2], v.view[3], v.view[4]
      end
      M.view_visible[idx] = true
      M.ports[idx] = { x = v.port[1], y = v.port[2], w = v.port[3], h = v.port[4] }
    end
  end
end

-- Transform for drawing view 0 into the application surface.
function M.world_transform()
  local t = love.math.newTransform()
  local cam = cameras[M.view_camera[0]]
  if not (S.view_enabled and cam) then return t end
  local p = M.ports[0]
  t:translate(p.x, p.y)
  t:scale(p.w / cam.w, p.h / cam.h)
  t:translate(-cam.x, -cam.y)
  return t
end

-- Display, GUI and window --------------------------------------------------------
function M.display_get_width() local w = love.window.getDesktopDimensions(); return w end
function M.display_get_height() local _, h = love.window.getDesktopDimensions(); return h end
function M.display_set_gui_size(w, h) display.gui_w, display.gui_h = w, h end
function M.display_get_gui_width() return display.gui_w end
function M.display_get_gui_height() return display.gui_h end

function M.window_get_width() local w = display.window_size(); return w end
function M.window_get_height() local _, h = display.window_size(); return h end
function M.window_set_size(w, h) display.set_window_size(w, h) end
function M.window_center()
  if love.window.getFullscreen() then return end
  local dw, dh = love.window.getDesktopDimensions()
  local w, h = love.window.getMode()
  love.window.setPosition(math.floor((dw - w) / 2), math.floor((dh - h) / 2))
end
function M.window_get_fullscreen() return display.get_fullscreen() end
function M.window_set_fullscreen(on) display.set_fullscreen(on and on ~= 0) end
-- MYCO_FOCUS=1 treats the window as focused (scripted runs start unfocused and the game auto-pauses).
local force_focus = os.getenv("MYCO_FOCUS") ~= nil
function M.window_has_focus() return force_focus or love.window.hasFocus() end
M.cr_none, M.cr_default = -1, 0
function M.window_set_cursor(cr) love.mouse.setVisible(cr ~= M.cr_none) end

function M.application_get_position() return value.array(display.application_position()) end
M.surface_draw_enabled = true
function M.application_surface_draw_enable(on) S.surface_draw_enabled = on and on ~= 0 end

-- Mouse coordinates -------------------------------------------------------------
function M.device_mouse_x_to_gui() local x = display.mouse_gui(); return x end
function M.device_mouse_y_to_gui() local _, y = display.mouse_gui(); return y end

-- Room-space mouse through view 0 (refreshed by the host every step).
function M.update_mouse(gm)
  local sx, sy = display.mouse_surface()
  local cam = cameras[M.view_camera[0]]
  if S.view_enabled and cam then
    local p = M.ports[0]
    gm.mouse_x = cam.x + (sx - p.x) * cam.w / p.w
    gm.mouse_y = cam.y + (sy - p.y) * cam.h / p.h
  else
    gm.mouse_x, gm.mouse_y = sx, sy
  end
end

return M
