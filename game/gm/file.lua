-- Files, ini, buffers, JSON and system info. Paths are relative to the LÖVE
-- save directory (writes) with fallback to the game's bundled files (reads),
-- like GameMaker's sandbox.
local assets = require("gm.assets")
local buffer = require("gm.buffer")
local ini = require("gm.ini")
local json = require("gm.json")
local value = require("gm.value")

local M = {}

local function read_any(name)
  if love.filesystem.getInfo(name) then return love.filesystem.read(name) end
  if love.filesystem.getInfo("assets/" .. name) then return love.filesystem.read("assets/" .. name) end
  return nil
end
M.read_any = read_any

function M.file_exists(name) return love.filesystem.getInfo(name) ~= nil or love.filesystem.getInfo("assets/" .. name) ~= nil end
function M.file_delete(name) return love.filesystem.remove(name) end
function M.file_rename(a, b)
  local data = love.filesystem.read(a)
  if not data then return false end
  love.filesystem.write(b, data)
  love.filesystem.remove(a)
  return true
end

-- Text files
local files, next_file = {}, 0
function M.file_text_open_read(name)
  local data = read_any(name)
  if not data then return -1 end
  local lines = {}
  for line in (data:gsub("\r\n", "\n") .. "\n"):gmatch("(.-)\n") do lines[#lines + 1] = line end
  if lines[#lines] == "" then lines[#lines] = nil end
  local id = next_file
  next_file = next_file + 1
  files[id] = { lines = lines, line = 1, col = 1, mode = "r" }
  return id
end
function M.file_text_open_write(name)
  local id = next_file
  next_file = next_file + 1
  files[id] = { name = name, out = {}, mode = "w" }
  return id
end
function M.file_text_read_string(id)
  local f = files[id]
  local line = f.lines[f.line] or ""
  local s = line:sub(f.col)
  f.col = #line + 1
  return s
end
function M.file_text_readln(id)
  local f = files[id]
  local line = (f.lines[f.line] or ""):sub(f.col)
  f.line, f.col = f.line + 1, 1
  return line
end
function M.file_text_eof(id) local f = files[id]; return f.line > #f.lines end
function M.file_text_write_string(id, s) local f = files[id]; f.out[#f.out + 1] = s end
function M.file_text_writeln(id) local f = files[id]; f.out[#f.out + 1] = "\r\n" end
function M.file_text_close(id)
  local f = files[id]
  if f and f.mode == "w" then love.filesystem.write(f.name, table.concat(f.out)) end
  files[id] = nil
end

-- INI
local cur_ini
function M.ini_open(name) cur_ini = { name = name, data = ini.parse(read_any(name) or ""), dirty = false } end
function M.ini_open_from_string(s) cur_ini = { data = ini.parse(s), dirty = false } end
function M.ini_close()
  local c = cur_ini
  cur_ini = nil
  if not c then return "" end
  local text = ini.serialize(c.data)
  if c.name and c.dirty then love.filesystem.write(c.name, text) end
  return text
end
function M.ini_read_string(sec, key, def) return ini.read(cur_ini.data, sec, key, def) end
function M.ini_read_real(sec, key, def) return tonumber(ini.read(cur_ini.data, sec, key, nil)) or def end
function M.ini_write_string(sec, key, v) ini.write(cur_ini.data, sec, key, v); cur_ini.dirty = true end
function M.ini_write_real(sec, key, v) ini.write(cur_ini.data, sec, key, string.format("%f", v)); cur_ini.dirty = true end
function M.ini_section_exists(sec) return cur_ini.data.sections[sec] ~= nil end
function M.ini_key_exists(sec, key) return ini.read(cur_ini.data, sec, key, nil) ~= nil end

-- Buffers (ids are numbers, like in GameMaker)
for _, k in ipairs({ "fixed", "grow", "wrap", "fast", "u8", "s8", "u16", "s16", "u32", "s32", "f16", "f32", "f64", "bool",
  "string", "u64", "text", "seek_start", "seek_relative", "seek_end" }) do M["buffer_" .. k] = buffer[k] end
function M.buffer_create(size, kind, align) return buffer.create(size, kind, align).id end
function M.buffer_exists(id) return buffer.exists(id) end
function M.buffer_delete(id) buffer.delete(id) end
function M.buffer_read(id, t) return buffer.read(id, t) end
function M.buffer_write(id, t, v) return buffer.write(id, t, v) end
function M.buffer_peek(id, off, t) return buffer.peek(id, off, t) end
function M.buffer_poke(id, off, t, v) return buffer.poke(id, off, t, v) end
function M.buffer_seek(id, base, off) buffer.seek(id, base, off) end
function M.buffer_tell(id) return buffer.tell(id) end
function M.buffer_get_size(id) return buffer.get_size(id) end
function M.buffer_resize(id, n) buffer.resize(id, n) end
function M.buffer_copy(src, off, size, dest, doff) buffer.copy(src, off, size, dest, doff) end
function M.buffer_load(name)
  local data = read_any(name)
  if not data then return -1 end
  return buffer.from_string(data, buffer.grow).id
end
function M.buffer_save(id, name) love.filesystem.write(name, buffer.to_string(id)) end
function M.buffer_get_address(id) return buffer.get(id).data end

function M.buffer_get_surface(id, surf, offset, draw)
  local canvas = draw.surface_get_canvas(surf)
  local img = canvas:newImageData()
  local bytes = img:getString()
  local b = buffer.get(id)
  if (offset or 0) + #bytes > b.size then buffer.resize(id, (offset or 0) + #bytes) end
  require("ffi").copy(b.data + (offset or 0), bytes, #bytes)
end

function M.buffer_set_surface(id, surf, offset, draw)
  local canvas = draw.surface_get_canvas(surf)
  local w, h = canvas:getDimensions()
  local b = buffer.get(id)
  local data = love.image.newImageData(w, h, "rgba8", require("ffi").string(b.data + (offset or 0), w * h * 4))
  local img = love.graphics.newImage(data)
  love.graphics.push("all")
  love.graphics.setCanvas(canvas)
  love.graphics.origin()
  love.graphics.setBlendMode("replace", "premultiplied")
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(img)
  love.graphics.pop()
  img:release()
end

-- JSON, time, system
function M.json_parse(s) return json.decode(s, json.gml) end
function M.json_stringify(v) return json.encode(v) end

local t0 = love.timer.getTime()
function M.get_timer() return math.floor((love.timer.getTime() - t0) * 1e6) end
function M.date_current_datetime()
  -- GameMaker datetime: days since 1899-12-30, local time.
  local now = os.time()
  local utc = os.time(os.date("!*t", now))
  return (now + (now - utc)) / 86400 + 25569
end

local function apple_locale()
  local p = io.popen("defaults read -g AppleLocale 2>/dev/null")
  local s = p and p:read("*l") or nil
  if p then p:close() end
  return s or os.getenv("LANG") or "en_US"
end
local locale = apple_locale()
function M.os_get_language() return (locale:match("^(%a%a)") or "en"):lower() end
function M.os_get_region() return (locale:match("^%a%a[_-](%a%a)") or "US"):upper() end

function M.show_debug_message(...)
  local parts = {}
  for i = 1, select("#", ...) do parts[i] = value.string((select(i, ...))) end
  print(table.concat(parts, " "))
end
function M.show_error(msg, abort) error(value.string(msg), 2) end
M.gamespeed_fps, M.gamespeed_microseconds = 0, 1
function M.game_get_speed(kind)
  local fps = assets.game.speed or 60
  if kind == M.gamespeed_microseconds then return 1e6 / fps end
  return fps
end

return M
