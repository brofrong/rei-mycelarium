-- GML values: 0-based arrays, structs, methods, undefined, UTF-8 strings, math helpers.
-- Calling convention for translated GML functions: fn(self, other, ...).
local M = {}

local UNDEFINED = setmetatable({}, { __tostring = function() return "undefined" end })

-- In GML booleans are the reals 1 and 0. Give Lua booleans arithmetic so
-- translated code like `px - r` with r = true behaves as in GameMaker.
local function num(v) if v == true then return 1 elseif v == false then return 0 end return v end
debug.setmetatable(true, {
  __add = function(a, b) return num(a) + num(b) end,
  __sub = function(a, b) return num(a) - num(b) end,
  __mul = function(a, b) return num(a) * num(b) end,
  __div = function(a, b) return num(a) / num(b) end,
  __mod = function(a, b) return math.fmod(num(a), num(b)) end,
  __pow = function(a, b) return num(a) ^ num(b) end,
  __unm = function(a) return -num(a) end,
})
M.undefined = UNDEFINED

local METHOD = {} -- method metatable (declared early: array_sort checks for it)

-- Arrays ----------------------------------------------------------------------
-- Elements live at raw integer keys 0..n-1; the length is kept in a.__n.
local ARRAY = {}
-- GML floors array indices; missing integer keys fall through to here.
ARRAY.__index = function(a, k)
  if type(k) == "number" and k ~= math.floor(k) then return rawget(a, math.floor(k)) end
  return nil
end
ARRAY.__newindex = function(a, k, v)
  if type(k) == "number" then
    k = math.floor(k)
    if rawget(a, k) ~= nil then rawset(a, k, v); return end
    local n = rawget(a, "__n")
    for i = n, k - 1 do rawset(a, i, 0) end
    rawset(a, k, v)
    if k >= n then rawset(a, "__n", k + 1) end
  else
    rawset(a, k, v)
  end
end

local function new_array(n)
  return setmetatable({ __n = n or 0 }, ARRAY)
end

function M.array(...)
  local a = new_array(0)
  local args = { n = select("#", ...), ... }
  for i = 1, args.n do rawset(a, i - 1, args[i]) end
  rawset(a, "__n", args.n)
  return a
end

function M.array_create(n, v)
  local a = new_array(n)
  if v == nil then v = 0 end
  for i = 0, n - 1 do rawset(a, i, v) end
  return a
end

function M.is_array(v) return getmetatable(v) == ARRAY end
function M.array_length(a) return rawget(a, "__n") end

function M.array_resize(a, n)
  local old = rawget(a, "__n")
  for i = old, n - 1 do rawset(a, i, 0) end
  for i = n, old - 1 do rawset(a, i, nil) end
  rawset(a, "__n", n)
end

function M.array_push(a, ...)
  for i = 1, select("#", ...) do
    local n = rawget(a, "__n")
    rawset(a, n, (select(i, ...)))
    rawset(a, "__n", n + 1)
  end
end

function M.array_pop(a)
  local n = rawget(a, "__n")
  if n == 0 then return UNDEFINED end
  local v = rawget(a, n - 1)
  rawset(a, n - 1, nil)
  rawset(a, "__n", n - 1)
  return v
end

function M.array_delete(a, index, count)
  local n = rawget(a, "__n")
  count = math.min(count, n - index)
  if count <= 0 then return end
  for i = index, n - count - 1 do rawset(a, i, rawget(a, i + count)) end
  for i = n - count, n - 1 do rawset(a, i, nil) end
  rawset(a, "__n", n - count)
end

function M.array_insert(a, index, ...)
  local k = select("#", ...)
  local n = rawget(a, "__n")
  for i = n - 1, index, -1 do rawset(a, i + k, rawget(a, i)) end
  for i = 1, k do rawset(a, index + i - 1, (select(i, ...))) end
  rawset(a, "__n", math.max(n, index) + k)
end

function M.array_copy(dest, dest_index, src, src_index, length)
  local tmp = {}
  for i = 0, length - 1 do tmp[i] = rawget(src, src_index + i) end
  for i = 0, length - 1 do dest[dest_index + i] = tmp[i] end
end

function M.array_sort(a, asc_or_fn)
  local n = rawget(a, "__n")
  local t = {}
  for i = 0, n - 1 do t[i + 1] = rawget(a, i) end
  if type(asc_or_fn) == "function" or getmetatable(asc_or_fn) == METHOD then
    table.sort(t, function(x, y) return asc_or_fn(nil, nil, x, y) < 0 end)
  elseif asc_or_fn == false then
    table.sort(t, function(x, y) return x > y end)
  else
    table.sort(t)
  end
  for i = 1, n do rawset(a, i - 1, t[i]) end
end

function M.array_concat(...)
  local out = new_array(0)
  for k = 1, select("#", ...) do
    local a = select(k, ...)
    for i = 0, rawget(a, "__n") - 1 do M.array_push(out, rawget(a, i)) end
  end
  return out
end

function M.array_contains(a, v)
  for i = 0, rawget(a, "__n") - 1 do
    if rawget(a, i) == v then return true end
  end
  return false
end

-- Callbacks follow the translated-GML convention fn(self, other, element, index).
function M.array_filter(a, fn)
  local out = new_array(0)
  for i = 0, rawget(a, "__n") - 1 do
    local v = rawget(a, i)
    if fn(nil, nil, v, i) then M.array_push(out, v) end
  end
  return out
end

-- GML's array_shuffle draws from the game's seeded RNG (random_set_seed), not from Lua's.
function M.array_shuffle(a, rng)
  rng = rng or function() return require("gm.random").random(1) end
  local n = rawget(a, "__n")
  local out = new_array(0)
  for i = 0, n - 1 do M.array_push(out, rawget(a, i)) end
  for i = n - 1, 1, -1 do
    local j = math.floor(rng() * (i + 1))
    local t = rawget(out, i)
    rawset(out, i, rawget(out, j))
    rawset(out, j, t)
  end
  return out
end

-- Iterate as for i, v in gm.ipairs0(a)
function M.ipairs0(a)
  local n = rawget(a, "__n")
  return function(_, i)
    i = i + 1
    if i < n then return i, rawget(a, i) end
  end, a, -1
end

-- Structs and methods -----------------------------------------------------------
local STRUCT = { __name = "struct" }
-- Called like any translated function: m(self, other, ...). The bound struct
-- becomes self and the caller's self becomes other.
METHOD.__call = function(m, caller, _, ...)
  return m.fn(m.self, caller, ...)
end

function M.struct(t)
  return setmetatable(t or {}, STRUCT)
end

function M.is_struct(v) return getmetatable(v) == STRUCT end

-- new Ctor(...) for GML constructor functions translated as fn(self, other, ...).
local constructors = setmetatable({}, { __mode = "k" })
function M.new(ctor, ...)
  local s = M.struct({})
  constructors[s] = ctor
  ctor(s, nil, ...)
  return s
end
function M.instanceof(s) return constructors[s] end

function M.method(self, fn)
  if getmetatable(fn) == METHOD then fn = fn.fn end
  return setmetatable({ self = self, fn = fn }, METHOD)
end

function M.is_method(v) return getmetatable(v) == METHOD or type(v) == "function" end
function M.method_get_self(m) return getmetatable(m) == METHOD and m.self or UNDEFINED end

function M.variable_struct_exists(s, k) return rawget(s, k) ~= nil end
function M.variable_struct_get(s, k)
  local v = rawget(s, k)
  if v == nil then return UNDEFINED end
  return v
end
function M.variable_struct_set(s, k, v) rawset(s, k, v) end
function M.variable_struct_remove(s, k) rawset(s, k, nil) end
function M.variable_struct_get_names(s)
  local names = {}
  for k in pairs(s) do
    if type(k) == "string" then names[#names + 1] = k end
  end
  table.sort(names)
  return M.array(unpack(names))
end
M.struct_exists, M.struct_get, M.struct_set, M.struct_remove, M.struct_get_names =
  M.variable_struct_exists, M.variable_struct_get, M.variable_struct_set, M.variable_struct_remove, M.variable_struct_get_names

function M.variable_clone(v)
  local mt = getmetatable(v)
  if mt == ARRAY then
    local out = new_array(0)
    for i = 0, rawget(v, "__n") - 1 do M.array_push(out, M.variable_clone(rawget(v, i))) end
    return out
  end
  if mt == STRUCT then
    local out = M.struct({})
    for k, x in pairs(v) do rawset(out, k, M.variable_clone(x)) end
    return out
  end
  return v
end

function M.is_undefined(v) return v == UNDEFINED or v == nil end
function M.is_string(v) return type(v) == "string" end
function M.is_real(v) return type(v) == "number" end
function M.is_numeric(v) return type(v) == "number" or type(v) == "boolean" end
function M.is_bool(v) return type(v) == "boolean" end

function M.typeof(v)
  local t = type(v)
  if t == "number" then return "number" end
  if t == "string" then return "string" end
  if t == "boolean" then return "bool" end
  if t == "function" then return "method" end
  if v == nil or v == UNDEFINED then return "undefined" end
  local mt = getmetatable(v)
  if mt == ARRAY then return "array" end
  if mt == METHOD then return "method" end
  if mt == STRUCT then return "struct" end
  return "struct"
end

-- Strings (UTF-8, 1-based like GML) --------------------------------------------
local CHAR = "[%z\1-\127\194-\244][\128-\191]*"

local function chars(s)
  local t = {}
  for c in s:gmatch(CHAR) do t[#t + 1] = c end
  return t
end
M.utf8_chars = chars

function M.string_length(s) local n = 0; for _ in s:gmatch(CHAR) do n = n + 1 end; return n end

function M.string_char_at(s, i)
  local t = chars(s)
  return t[math.max(1, math.floor(i))] or ""
end

function M.string_copy(s, index, count)
  local t = chars(s)
  index = math.max(1, math.floor(index))
  return table.concat(t, "", index, math.min(#t, index + math.floor(count) - 1))
end

function M.string_delete(s, index, count)
  local t = chars(s)
  for _ = 1, count do table.remove(t, index) end
  return table.concat(t)
end

function M.string_insert(sub, s, index)
  local t = chars(s)
  table.insert(t, math.min(index, #t + 1), sub)
  return table.concat(t)
end

function M.string_pos(sub, s)
  local b = s:find(sub, 1, true)
  if not b then return 0 end
  return M.string_length(s:sub(1, b - 1)) + 1
end

function M.string_replace_all(s, sub, new)
  if sub == "" then return s end
  local out, i = {}, 1
  while true do
    local b, e = s:find(sub, i, true)
    if not b then break end
    out[#out + 1] = s:sub(i, b - 1)
    out[#out + 1] = new
    i = e + 1
  end
  out[#out + 1] = s:sub(i)
  return table.concat(out)
end

function M.string_replace(s, sub, new)
  local b, e = s:find(sub, 1, true)
  if not b then return s end
  return s:sub(1, b - 1) .. new .. s:sub(e + 1)
end

function M.string_upper(s) return s:upper() end
function M.string_lower(s) return s:lower() end
function M.string_repeat(s, n) return s:rep(n) end
function M.string_count(sub, s)
  local n, i = 0, 1
  while true do
    local b, e = s:find(sub, i, true)
    if not b then return n end
    n, i = n + 1, e + 1
  end
end
function M.string_trim(s) return (s:gsub("^%s+", ""):gsub("%s+$", "")) end
function M.ord(c)
  local b1 = c:byte(1)
  if not b1 then return 0 end
  if b1 < 0x80 then return b1 end
  if b1 < 0xE0 then return (b1 - 0xC0) * 64 + c:byte(2) - 0x80 end
  if b1 < 0xF0 then return ((b1 - 0xE0) * 64 + c:byte(2) - 0x80) * 64 + c:byte(3) - 0x80 end
  return (((b1 - 0xF0) * 64 + c:byte(2) - 0x80) * 64 + c:byte(3) - 0x80) * 64 + c:byte(4) - 0x80
end
function M.chr(n)
  if n < 0x80 then return string.char(n) end
  if n < 0x800 then return string.char(0xC0 + math.floor(n / 64), 0x80 + n % 64) end
  if n < 0x10000 then return string.char(0xE0 + math.floor(n / 4096), 0x80 + math.floor(n / 64) % 64, 0x80 + n % 64) end
  return string.char(0xF0 + math.floor(n / 262144), 0x80 + math.floor(n / 4096) % 64, 0x80 + math.floor(n / 64) % 64, 0x80 + n % 64)
end

function M.string_split(s, delim)
  local out, i = {}, 1
  while true do
    local b, e = s:find(delim, i, true)
    if not b then break end
    out[#out + 1] = s:sub(i, b - 1)
    i = e + 1
  end
  out[#out + 1] = s:sub(i)
  return M.array(unpack(out))
end

-- GML string(): integers print without decimals, other reals with 2 decimals.
function M.string(v)
  local t = type(v)
  if t == "string" then return v end
  if t == "number" then
    if v == math.floor(v) and math.abs(v) < 1e15 then return string.format("%d", v) end
    return string.format("%.2f", v)
  end
  if t == "boolean" then return v and "1" or "0" end
  if v == nil or v == UNDEFINED then return "undefined" end
  return tostring(v)
end

function M.string_format(v, total, dec)
  return string.format("%" .. total .. "." .. dec .. "f", v)
end

function M.real(v)
  if type(v) == "number" then return v end
  if type(v) == "boolean" then return v and 1 or 0 end
  return tonumber(v) or 0
end

-- GML truthiness: numbers are true above 0.5; undefined is false.
function M.bool(v)
  if type(v) == "number" then return v > 0.5 end
  return v ~= nil and v ~= false and v ~= UNDEFINED
end

-- Math with GML semantics -------------------------------------------------------
local function trunc(x) return x >= 0 and math.floor(x) or math.ceil(x) end
M.trunc = trunc
function M.div(a, b) return trunc(a / b) end
function M.mod(a, b) return math.fmod(a, b) end

return M
