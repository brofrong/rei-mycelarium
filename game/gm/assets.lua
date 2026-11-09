-- Loads build/assets (extracted by tools/extract.py) and exposes GameMaker-style
-- numeric asset ids: sprites, fonts, sounds, shaders and rooms are indexed in
-- data.win order (0-based), and their names become globals (apple, fnt_gm_20, ...).
local json = require("gm.json")

local M = { sprites = {}, sprite_by_name = {}, fonts = {}, font_by_name = {}, sounds = {}, sound_by_name = {},
  shaders = {}, shader_by_name = {}, rooms = {}, room_by_name = {}, pages = {}, game = {} }

local ROOT = "assets/"

local function read_json(name)
  local text = assert(love.filesystem.read(ROOT .. name), "missing " .. ROOT .. name)
  return json.decode(text)
end

-- JSON objects lose order; data.win order is restored from each file's key order on disk.
local function ordered_keys(name)
  local text = love.filesystem.read(ROOT .. name)
  local keys, depth, i = {}, 0, 1
  while true do
    local c = text:sub(i, i)
    if c == "" then break end
    if c == '"' then
      local j = i + 1
      while text:sub(j, j) ~= '"' do
        if text:sub(j, j) == "\\" then j = j + 1 end
        j = j + 1
      end
      if depth == 1 and text:sub(j + 1):match("^%s*:") then keys[#keys + 1] = text:sub(i + 1, j - 1) end
      i = j
    elseif c == "{" or c == "[" then depth = depth + 1
    elseif c == "}" or c == "]" then depth = depth - 1 end
    i = i + 1
  end
  return keys
end

local function frame_quad(fr)
  local page = M.pages[fr.page + 1]
  return love.graphics.newQuad(fr.src[1], fr.src[2], fr.src[3], fr.src[4], page:getDimensions())
end

function M.load(globals)
  globals = globals or _G
  M.game = read_json("game.json")
  local i = 0
  while love.filesystem.getInfo(ROOT .. "pages/page" .. i .. ".png") do
    local img = love.graphics.newImage(ROOT .. "pages/page" .. i .. ".png")
    img:setFilter("linear", "linear")
    M.pages[i + 1] = img
    i = i + 1
  end

  local sprites = read_json("sprites.json")
  for idx, name in ipairs(ordered_keys("sprites.json")) do
    local s = sprites[name]
    s.name, s.index = name, idx - 1
    for _, fr in ipairs(s.frames) do
      fr.image = M.pages[fr.page + 1]
      fr.quad = frame_quad(fr)
    end
    M.sprites[idx - 1] = s
    M.sprite_by_name[name] = idx - 1
    globals[name] = idx - 1
  end

  local fonts = read_json("fonts.json")
  for idx, name in ipairs(ordered_keys("fonts.json")) do
    local f = fonts[name]
    f.name, f.index, f.kind = name, idx - 1, "gm"
    f.frame.image = M.pages[f.frame.page + 1]
    M.fonts[idx - 1] = f
    M.font_by_name[name] = idx - 1
    globals[name] = idx - 1
  end

  local sounds = read_json("sounds.json")
  for idx, name in ipairs(ordered_keys("sounds.json")) do
    local s = sounds[name]
    s.name, s.index, s.path = name, idx - 1, ROOT .. "sounds/" .. s.file
    M.sounds[idx - 1] = s
    M.sound_by_name[name] = idx - 1
    globals[name] = idx - 1
  end

  local idx = 0
  for _, name in ipairs({ "sh_blackmetal", "sh_glow_blur", "sh_stinky" }) do
    if love.filesystem.getInfo(ROOT .. "shaders/" .. name .. ".fsh") then
      M.shaders[idx] = { name = name, index = idx, vsh = love.filesystem.read(ROOT .. "shaders/" .. name .. ".vsh"),
        fsh = love.filesystem.read(ROOT .. "shaders/" .. name .. ".fsh") }
      M.shader_by_name[name] = idx
      globals[name] = idx
      idx = idx + 1
    end
  end

  local rooms = read_json("rooms.json")
  for i2, name in ipairs(ordered_keys("rooms.json")) do
    rooms[name].name, rooms[name].index = name, i2 - 1
    M.rooms[i2 - 1] = rooms[name]
    M.room_by_name[name] = i2 - 1
    globals[name] = name
  end
  M.objects = read_json("objects.json")
  return M
end

function M.sprite(id)
  local s = M.sprites[id]
  if not s then error("gm: unknown sprite " .. tostring(id), 2) end
  return s
end

-- Add a runtime sprite (sprite_create_from_surface etc.); returns its id.
function M.add_sprite(s)
  local id = #M.sprites + 1
  while M.sprites[id] do id = id + 1 end
  s.index = id
  M.sprites[id] = s
  return id
end

function M.read_text(name) return love.filesystem.read(ROOT .. name) end

return M
