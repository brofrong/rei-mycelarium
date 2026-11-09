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

-- Reference capture for the Rust port.
-- MYCO_GOLDEN=<script.lua> MYCO_GOLDEN_OUT=<file.json>: on the first frame (game booted, menu or
-- Room1 shown) run the script; it returns a plain Lua table, written as JSON (numbers as %.17g),
-- then quit. MYCO_DUMP=<frame>:<file.json> writes what save_game would save at that frame (same
-- JSON, no slot written) and quits. MYCO_SEED=<int> fixes the world seed (see world_init).
-- MYCO_IDENTITY=<name> (conf.lua) moves the LÖVE save folder: use it for any automated session
-- that starts a game, so autosaves never touch the real slots in LOVE/Mycelarium.
local golden_script, golden_out = os.getenv("MYCO_GOLDEN"), os.getenv("MYCO_GOLDEN_OUT")
local dump_frame, dump_path = (os.getenv("MYCO_DUMP") or ""):match("^(%d+):(.+)$")
dump_frame = tonumber(dump_frame)

local function json_encode(v)
  local t = type(v)
  if t == "number" then
    if v ~= v or v == math.huge or v == -math.huge then return "null" end
    return string.format("%.17g", v)
  elseif t == "boolean" then return tostring(v)
  elseif t == "string" then
    return '"' .. v:gsub('[%c"\\]', function(c)
      return ({ ['"'] = '\\"', ["\\"] = "\\\\", ["\n"] = "\\n", ["\r"] = "\\r", ["\t"] = "\\t" })[c]
        or string.format("\\u%04x", c:byte())
    end) .. '"'
  elseif t == "table" then
    local n = #v
    local count = 0
    for _ in pairs(v) do count = count + 1 end
    local parts = {}
    if count == n then
      for i = 1, n do parts[i] = json_encode(v[i]) end
      return "[" .. table.concat(parts, ",") .. "]"
    end
    local keys = {}
    for k in pairs(v) do keys[#keys + 1] = tostring(k) end
    table.sort(keys)
    for _, k in ipairs(keys) do
      local val = v[k]
      if val == nil then val = v[tonumber(k)] end
      parts[#parts + 1] = json_encode(k) .. ":" .. json_encode(val)
    end
    return "{" .. table.concat(parts, ",") .. "}"
  elseif v == nil then return "null" end
  error("json_encode: unsupported type " .. t)
end

local function write_file(path, text)
  local f = assert(io.open(path, "wb"))
  f:write(text)
  f:close()
end

local function run_golden()
  local fn = assert(loadfile(golden_script))
  local ok, res = xpcall(fn, debug.traceback)
  if not ok then
    print("GOLDEN error: " .. tostring(res))
    os.exit(1)
  end
  write_file(assert(golden_out, "MYCO_GOLDEN_OUT not set"), json_encode(res) .. "\n")
  print("GOLDEN wrote " .. golden_out)
  gm.__ended = true -- skip Game End (it autosaves)
  love.event.quit()
end

-- save_game builds the save table and writes it through save_write_text; capture that text
-- instead, lifting the title/intro/slot guards for the duration of the call.
local function run_dump()
  local captured
  local saved_write, saved_timer = save_write_text, global.save_timer
  local saved_slot, saved_title, saved_intro = global.save_slot, global.title_open, global.intro_active
  save_write_text = function(self, other, path, text)
    if not captured then captured = text end
  end
  if gm.is_undefined(global.save_slot) or global.save_slot < 0 then global.save_slot = 0 end
  global.title_open, global.intro_active = false, false
  local ok, err = pcall(save_game, global, global)
  save_write_text, global.save_timer = saved_write, saved_timer
  global.save_slot, global.title_open, global.intro_active = saved_slot, saved_title, saved_intro
  if not ok or not captured then
    print("DUMP failed: " .. tostring(err or "save_game wrote nothing"))
    os.exit(1)
  end
  write_file(dump_path, captured)
  print("DUMP wrote " .. dump_path)
  gm.__ended = true -- skip Game End (it autosaves)
  love.event.quit()
end

function love.draw()
  frame = frame + 1
  if gm then run_script() end
  if demo then demo.draw() else gm.render() end
  if gm and golden_script and frame == 1 then run_golden() end
  if gm and dump_frame and frame == dump_frame then run_dump() end
  if shot_frame and frame == shot_frame then
    love.graphics.captureScreenshot(function(img)
      local f = assert(io.open(shot_path, "wb"))
      f:write(img:encode("png"):getString())
      f:close()
      love.event.quit()
    end)
  end
end
