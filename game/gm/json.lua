-- Small JSON codec. decode(text) gives plain Lua tables (1-based arrays);
-- decode(text, json.gml) gives GML structs and 0-based gm arrays.
local value = require("gm.value")

local M = {}

M.gml = { array = function(items, n)
  local a = value.array()
  for i = 1, n do a[i - 1] = items[i] end
  return a
end, object = value.struct, null = value.undefined }

local plain = { array = function(items) return items end, object = function(t) return t end, null = nil }

local escapes = { ['"'] = '"', ["\\"] = "\\", ["/"] = "/", b = "\b", f = "\f", n = "\n", r = "\r", t = "\t" }

local function utf8_char(cp)
  return value.chr(cp)
end

function M.decode(text, mode)
  mode = mode or plain
  local pos = 1
  local function ws() pos = text:find("[^ \t\r\n]", pos) or #text + 1 end
  local function err(msg) error(string.format("json: %s at %d", msg, pos), 0) end
  local parse

  local function str()
    local out, i = {}, pos + 1
    while true do
      local c = text:sub(i, i)
      if c == "" then err("unterminated string") end
      if c == '"' then break end
      if c == "\\" then
        local e = text:sub(i + 1, i + 1)
        if e == "u" then
          local cp = tonumber(text:sub(i + 2, i + 5), 16)
          if cp >= 0xD800 and cp <= 0xDBFF and text:sub(i + 6, i + 7) == "\\u" then
            local lo = tonumber(text:sub(i + 8, i + 11), 16)
            cp = 0x10000 + (cp - 0xD800) * 0x400 + (lo - 0xDC00)
            i = i + 6
          end
          out[#out + 1] = utf8_char(cp)
          i = i + 6
        else
          out[#out + 1] = escapes[e] or e
          i = i + 2
        end
      else
        local j = text:find('["\\]', i) or #text + 1
        out[#out + 1] = text:sub(i, j - 1)
        i = j
      end
    end
    pos = i + 1
    return table.concat(out)
  end

  function parse()
    ws()
    local c = text:sub(pos, pos)
    if c == "{" then
      pos = pos + 1
      local t = {}
      ws()
      if text:sub(pos, pos) == "}" then pos = pos + 1; return mode.object(t) end
      while true do
        ws()
        if text:sub(pos, pos) ~= '"' then err("expected key") end
        local k = str()
        ws()
        if text:sub(pos, pos) ~= ":" then err("expected ':'") end
        pos = pos + 1
        t[k] = parse()
        ws()
        local d = text:sub(pos, pos)
        pos = pos + 1
        if d == "}" then return mode.object(t) end
        if d ~= "," then err("expected ',' or '}'") end
      end
    elseif c == "[" then
      pos = pos + 1
      local items, n = {}, 0
      ws()
      if text:sub(pos, pos) == "]" then pos = pos + 1; return mode.array(items, 0) end
      while true do
        n = n + 1
        items[n] = parse()
        ws()
        local d = text:sub(pos, pos)
        pos = pos + 1
        if d == "]" then return mode.array(items, n) end
        if d ~= "," then err("expected ',' or ']'") end
      end
    elseif c == '"' then
      return str()
    elseif text:sub(pos, pos + 3) == "true" then pos = pos + 4; return true
    elseif text:sub(pos, pos + 4) == "false" then pos = pos + 5; return false
    elseif text:sub(pos, pos + 3) == "null" then pos = pos + 4; return mode.null
    else
      local num = text:match("^-?%d+%.?%d*[eE]?[-+]?%d*", pos)
      if not num or num == "" then err("unexpected character") end
      pos = pos + #num
      return tonumber(num)
    end
  end

  local ok, result = pcall(parse)
  if not ok then error(result, 2) end
  return result
end

local function encode_string(s)
  return '"' .. s:gsub('[%c"\\]', function(c)
    local map = { ['"'] = '\\"', ["\\"] = "\\\\", ["\n"] = "\\n", ["\r"] = "\\r", ["\t"] = "\\t" }
    return map[c] or string.format("\\u%04x", c:byte())
  end) .. '"'
end

local function encode_number(n)
  if n ~= n or n == math.huge or n == -math.huge then return "0" end
  if n == math.floor(n) and math.abs(n) < 1e15 then return string.format("%d", n) end
  return string.format("%.17g", n)
end

function M.encode(v)
  local t = type(v)
  if t == "string" then return encode_string(v) end
  if t == "number" then return encode_number(v) end
  if t == "boolean" then return tostring(v) end
  if v == nil or v == value.undefined then return "null" end
  if value.is_array(v) then
    local out = {}
    for i = 0, value.array_length(v) - 1 do out[#out + 1] = M.encode(v[i]) end
    return "[" .. table.concat(out, ",") .. "]"
  end
  if value.is_method(v) then return "null" end
  local keys = {}
  for k in pairs(v) do
    if type(k) == "string" then keys[#keys + 1] = k end
  end
  table.sort(keys)
  local out = {}
  for _, k in ipairs(keys) do out[#out + 1] = encode_string(k) .. ":" .. M.encode(v[k]) end
  return "{" .. table.concat(out, ",") .. "}"
end

return M
