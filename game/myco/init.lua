-- Loads the translated game (game/myco/gen/*.lua), registers objects with their
-- event functions and runs the global scripts, like the GameMaker runner does.
local gm = require("gm")

local M = {}

local EVENT_NAMES = { [0] = "Create", [1] = "Destroy", [2] = "Alarm", [3] = "Step", [4] = "Collision", [5] = "Keyboard",
  [6] = "Mouse", [7] = "Other", [8] = "Draw", [9] = "KeyPress", [10] = "KeyRelease", [12] = "CleanUp" }

function M.load_code()
  local files = love.filesystem.getDirectoryItems("myco/gen")
  table.sort(files)
  for _, f in ipairs(files) do
    if f:match("%.lua$") then
      local chunk = assert(love.filesystem.load("myco/gen/" .. f))
      chunk()
    end
  end
end

-- Objects come from objects.json; each event maps to the global function
-- gml_Object_<obj>_<Event>_<sub> produced by the translation.
function M.register_objects(missing)
  for name, def in pairs(gm.assets.objects) do
    local spec = { parent = def.parent, visible = def.visible, persistent = def.persistent,
      sprite = def.sprite and gm.assets.sprite_by_name[def.sprite] or -1 }
    for _, ev in ipairs(def.events) do
      local ev_name = (EVENT_NAMES[ev[1]] or ("Event" .. ev[1])) .. "_" .. ev[2]
      local fn = rawget(_G, "gml_Object_" .. name .. "_" .. ev_name)
      if fn then spec[ev_name] = fn else missing[#missing + 1] = name .. "." .. ev_name end
    end
    _G[name] = gm.object(name, spec)
  end
end

function M.run_global_scripts()
  for _, name in ipairs(require("myco.global_scripts")) do
    local fn = rawget(_G, name)
    -- top-level script code runs in the global context
    if fn then fn(gm.global, gm.global) end
  end
end

local TRACE = os.getenv("MYCO_TRACE")
local function trace(msg) if TRACE then print(string.format("[%.3f] %s", love.timer.getTime(), msg)) end end
M.trace = trace

-- GMEXT-Steamworks GML wrappers the game calls as scripts (the port runs without Steam).
local function define_steam_scripts()
  steam_step = function() end
  steam_userstats_get_achievement = function() return false end
  steam_userstats_set_achievement = function() return false end
  steam_userstats_store_stats = function() return false end
  steam_api_is_initialized = function() return false end
end

function M.boot()
  define_steam_scripts()
  _G.global = gm.global
  trace("load code")
  M.load_code()
  -- conformance runs: music/ambience draws stay off the game's RNG (gm/conform.lua)
  local conform = require("gm.conform")
  if conform.enabled then conform.install_audio(_G, require("gm.random")) end
  trace("assets")
  local missing = {}
  gm.boot_assets()
  M.register_objects(missing)
  for _, m in ipairs(missing) do print("myco: missing event " .. m) end
  trace("global scripts")
  M.run_global_scripts()
  trace("room")
  gm.boot_room()
  trace("booted")
end

return M
