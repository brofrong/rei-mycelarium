-- Mycelarium (LÖVE port). Phase 1 skeleton: display + mouse test scene.
local display = require("gm.display")
local dm = require("gm.display_math")

local view, canvas
local frame = 0
local shot_path, shot_frame

local function refit()
  local ww, wh = display.window_size()
  local v = dm.view_for_window(ww, wh)
  if not v then return end
  if not view or v.surf_w ~= view.surf_w or v.surf_h ~= view.surf_h then
    canvas = love.graphics.newCanvas(v.surf_w, v.surf_h)
  end
  view = v
  display.update(v.surf_w, v.surf_h, v.view_w, v.view_h)
end

local demo_name = os.getenv("MYCO_DEMO")
local demo = demo_name == "draw" and require("demo") or nil

function love.load()
  if demo then demo.load() end
  local shot = os.getenv("MYCO_SHOT")
  if shot then
    shot_path, shot_frame = shot:match("^(.*):(%d+)$")
    shot_frame = tonumber(shot_frame)
  end
  if not demo then
    display.init()
    refit()
  end
end

function love.resize()
  if demo then demo.resize() else refit() end
end

function love.update(dt)
  if demo then demo.update(dt) end
end

function love.keypressed(key)
  local cmd = love.keyboard.isDown("lgui", "rgui")
  local ctrl = love.keyboard.isDown("lctrl", "rctrl")
  local alt = love.keyboard.isDown("lalt", "ralt")
  if key == "f11" or (key == "f" and cmd and ctrl) or (key == "return" and alt) then
    display.set_fullscreen(not display.get_fullscreen())
    refit()
  elseif key == "escape" then
    love.event.quit()
  end
end

local function draw_scene()
  local gx, gy = display.mouse_gui()
  love.graphics.push()
  love.graphics.scale(view.surf_w / view.view_w, view.surf_h / view.view_h)
  love.graphics.clear(0.08, 0.06, 0.05, 1)
  love.graphics.setLineWidth(4)
  love.graphics.setColor(0.9, 0.5, 0.2, 1)
  love.graphics.rectangle("line", 2, 2, view.view_w - 4, view.view_h - 4)
  love.graphics.setColor(0.4, 0.8, 0.4, 1)
  love.graphics.rectangle("line", view.ui_ox, 0, 1920, 1080)
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.line(gx - 30, gy, gx + 30, gy)
  love.graphics.line(gx, gy - 30, gx, gy + 30)
  local ww, wh = display.window_size()
  local sx, sy, sw, sh = love.window.getSafeArea()
  love.graphics.print(string.format(
    "window(px) %dx%d  pixel dims %dx%d  dims %dx%d  dpi %.2f\nsafe(units) %d,%d %dx%d  fullscreen %s\nview %dx%d surf %dx%d ui_ox %.0f ui_by %d\nmouse gui %.1f, %.1f",
    ww, wh, love.graphics.getPixelWidth(), love.graphics.getPixelHeight(), love.graphics.getWidth(), love.graphics.getHeight(),
    love.window.getDPIScale(), sx, sy, sw, sh, tostring(display.get_fullscreen()),
    view.view_w, view.view_h, view.surf_w, view.surf_h, view.ui_ox, view.ui_by, gx, gy), 40, 40, 0, 2, 2)
  love.graphics.pop()
end

function love.draw()
  frame = frame + 1
  if demo then
    demo.draw()
  elseif view then
    love.graphics.setCanvas(canvas)
    draw_scene()
    love.graphics.setCanvas()
    love.graphics.clear(0, 0, 0, 1)
    display.present(canvas)
  end
  if shot_frame and frame == shot_frame then
    love.graphics.captureScreenshot(function(img)
      local data = img:encode("png")
      local f = assert(io.open(shot_path, "wb"))
      f:write(data:getString())
      f:close()
      love.event.quit()
    end)
  end
end
