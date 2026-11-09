-- Window, fullscreen and the application surface on macOS (Retina + notch).
-- All sizes here are drawable pixels; the safe area excludes the notch.
local dm = require("gm.display_math")

local M = { settings = nil, fit = dm.fit(0, 0, 0, 0, 0, 0, 1, 1), surf_w = 1, surf_h = 1, gui_w = 1920, gui_h = 1080 }
local SETTINGS_FILE = "settings.ini"

function M.load_settings()
  local text = love.filesystem.getInfo(SETTINGS_FILE) and love.filesystem.read(SETTINGS_FILE) or ""
  return dm.ini_to_settings(text)
end

function M.save_settings(t)
  love.filesystem.write(SETTINGS_FILE, dm.settings_to_ini(t))
end

local function safe_area()
  local sx, sy, sw, sh = love.window.getSafeArea()
  local ww, wh = love.graphics.getPixelDimensions()
  -- getSafeArea is in window units; convert to pixels
  local k = love.window.getDPIScale()
  if love.graphics.getDimensions() == ww then k = 1 end
  sx, sy, sw, sh = sx * k, sy * k, sw * k, sh * k
  if sw <= 0 or sh <= 0 then sx, sy, sw, sh = 0, 0, ww, wh end
  return ww, wh, sx, sy, sw, sh
end

-- Size the game should treat as "the window" (window_get_width/height).
function M.window_size()
  local _, _, _, _, sw, sh = safe_area()
  return math.floor(sw + 0.5), math.floor(sh + 0.5)
end

function M.set_fullscreen(on)
  love.window.setFullscreen(on, "desktop")
  if M.settings then
    M.settings.fullscreen = on
    M.save_settings(M.settings)
  end
end

function M.get_fullscreen() return (love.window.getFullscreen()) end

function M.set_window_size(w, h)
  if love.window.getFullscreen() then return end
  local k = love.window.getDPIScale()
  love.window.updateMode(math.floor(w / k), math.floor(h / k), {})
end

function M.init(settings)
  M.settings = settings or M.load_settings()
  if M.settings.fullscreen then
    love.window.setFullscreen(true, "desktop")
  else
    M.set_window_size(M.settings.window_w, M.settings.window_h)
  end
end

-- Recompute where the application surface (surf_w x surf_h) lands in the window.
function M.update(surf_w, surf_h, gui_w, gui_h)
  M.surf_w, M.surf_h = surf_w, surf_h
  M.gui_w, M.gui_h = gui_w or surf_w, gui_h or surf_h
  local ww, wh, sx, sy, sw, sh = safe_area()
  M.fit = dm.fit(ww, wh, sx, sy, sw, sh, surf_w, surf_h)
  return M.fit
end

-- application_get_position: x1, y1, x2, y2 in window pixels
function M.application_position()
  local f = M.fit
  return f.x, f.y, f.x + f.w, f.y + f.h
end

-- Test autopilot can pin the cursor (window pixels) without moving the real mouse.
M.mouse_override = nil

local function mouse_px()
  if M.mouse_override then return M.mouse_override[1], M.mouse_override[2] end
  local mx, my = love.mouse.getPosition()
  if love.graphics.getDimensions() ~= love.graphics.getPixelDimensions() then
    local k = love.window.getDPIScale()
    mx, my = mx * k, my * k
  end
  return mx, my
end

-- Mouse in application-surface pixels.
function M.mouse_surface()
  local mx, my = mouse_px()
  return dm.screen_to_game(M.fit, mx, my)
end

-- Mouse in GUI coordinates (display_set_gui_size space).
function M.mouse_gui()
  local sx, sy = M.mouse_surface()
  if M.fit.scale == 0 then return -1, -1 end
  return sx * M.gui_w / M.surf_w, sy * M.gui_h / M.surf_h
end

-- Draw a canvas the size of the application surface at its window position.
function M.present(canvas)
  local f = M.fit
  if f.scale == 0 then return end
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(canvas, f.x, f.y, 0, f.w / canvas:getWidth(), f.h / canvas:getHeight())
end

-- Draw a GUI-sized canvas over the same rectangle.
function M.present_gui(canvas)
  M.present(canvas)
end

return M
