-- Mycelarium (LÖVE port). Runs the translated game; MYCO_DEMO=draw|display runs a test scene.
io.stdout:setvbuf("no")
-- MYCO_SHOT=<path>:<frame> saves a screenshot at that frame and quits.
local frame = 0
local shot_path, shot_frame
local demo_name = os.getenv("MYCO_DEMO")
local demo = (demo_name == "draw" and require("demo")) or (demo_name == "display" and require("demo_display")) or nil
local gm

function love.load()
  local shot = os.getenv("MYCO_SHOT")
  if shot then
    shot_path, shot_frame = shot:match("^(.*):(%d+)$")
    shot_frame = tonumber(shot_frame)
  end
  if demo then
    demo.load()
  else
    gm = require("gm")
    require("myco").boot()
  end
end

function love.resize() if demo then demo.resize() end end
function love.update(dt) if demo then demo.update(dt) else gm.update(dt) end end

function love.keypressed(key)
  if key == "escape" and love.keyboard.isDown("lgui", "rgui") then love.event.quit() end
  if demo then if demo.keypressed then demo.keypressed(key) end else gm.keypressed(key) end
end
function love.textinput(t) if gm then gm.textinput(t) end end
function love.keyreleased(key) if gm then gm.keyreleased(key) end end
function love.mousepressed(x, y, b) if gm then gm.mousepressed(x, y, b) end end
function love.mousereleased(x, y, b) if gm then gm.mousereleased(x, y, b) end end
function love.wheelmoved(x, y) if gm then gm.wheelmoved(x, y) end end
function love.quit() if gm then return gm.quit() end end

-- MYCO_INPUT="60:move:0.5,0.6;61:down:1;63:up:1;120:key:space" (frame:action:args; move takes
-- fractions of the application surface rectangle) drives the game for automated checks.
local script = {}
for frame_s, action, args in (os.getenv("MYCO_INPUT") or ""):gmatch("(%d+):(%a+):([^;]*)") do
  script[#script + 1] = { tonumber(frame_s), action, args }
end
-- MYCO_PROBE="frame|lua chunk" prints debug state at a frame (development aid).
local probe_frame, probe_code = (os.getenv("MYCO_PROBE") or ""):match("^(%d+)|(.*)$")
probe_frame = tonumber(probe_frame)

local function run_script()
  if probe_frame == frame then
    local fn, err = loadstring(probe_code)
    if fn then local ok, res = pcall(fn); print("PROBE", ok, res) else print("PROBE syntax", err) end
  end
  for _, s in ipairs(script) do
    if s[1] == frame then
      local display = require("gm.display")
      if s[2] == "move" then
        local fx, fy = s[3]:match("([%d.]+),([%d.]+)")
        local f = display.fit
        display.mouse_override = { f.x + f.w * tonumber(fx), f.y + f.h * tonumber(fy) }
      elseif s[2] == "gmove" then
        -- GUI coordinates (display_set_gui_size space) to window pixels
        local gx, gy = s[3]:match("([-%d.]+),([-%d.]+)")
        local f = display.fit
        display.mouse_override = { f.x + tonumber(gx) * f.w / display.gui_w, f.y + tonumber(gy) * f.h / display.gui_h }
      elseif s[2] == "wmove" then
        -- world offset from the mycelium controller, through view 0 to window pixels
        local w = require("gm.window")
        local dx, dy = s[3]:match("([-%d.]+),([-%d.]+)")
        local m = gm.instance_find(obj_mycelium_controller, 0)
        local cam, port = w.cameras[gm.view_camera[0]], w.ports[0]
        local sx = port.x + (m.x + tonumber(dx) - cam.x) * port.w / cam.w
        local sy = port.y + (m.y + tonumber(dy) - cam.y) * port.h / cam.h
        local f, d = display.fit, display
        display.mouse_override = { f.x + sx * f.w / d.surf_w, f.y + sy * f.h / d.surf_h }
      elseif s[2] == "down" then gm.mousepressed(0, 0, tonumber(s[3]))
      elseif s[2] == "up" then gm.mousereleased(0, 0, tonumber(s[3]))
      elseif s[2] == "key" then gm.keypressed(s[3]); gm.keyreleased(s[3])
      elseif s[2] == "quit" then love.event.quit()
      elseif s[2] == "kdown" then gm.keypressed(s[3])
      elseif s[2] == "kup" then gm.keyreleased(s[3])
      elseif s[2] == "text" then gm.textinput(s[3]) end
    end
  end
end

function love.draw()
  frame = frame + 1
  if gm then run_script() end
  if demo then demo.draw() else gm.render() end
  if shot_frame and frame == shot_frame then
    love.graphics.captureScreenshot(function(img)
      local f = assert(io.open(shot_path, "wb"))
      f:write(img:encode("png"):getString())
      f:close()
      love.event.quit()
    end)
  end
end
