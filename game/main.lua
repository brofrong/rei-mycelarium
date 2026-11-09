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

function love.draw()
  frame = frame + 1
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
