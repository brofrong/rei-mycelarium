-- INI files in GameMaker format: [section] key="value".
local M = {}

function M.parse(text)
  local d = { order = {}, sections = {} }
  local cur
  for line in (text or ""):gmatch("[^\r\n]+") do
    local s = line:match("^%s*%[(.-)%]%s*$")
    if s then
      cur = s
      if not d.sections[s] then
        d.sections[s] = { order = {}, values = {} }
        d.order[#d.order + 1] = s
      end
    elseif cur then
      local k, v = line:match('^%s*([^=]-)%s*=%s*(.-)%s*$')
      if k and k ~= "" then
        v = v:match('^"(.*)"$') or v
        M.write(d, cur, k, v)
      end
    end
  end
  return d
end

function M.read(d, section, key, default)
  local s = d.sections[section]
  local v = s and s.values[key]
  if v == nil then return default end
  return v
end

function M.write(d, section, key, value)
  local s = d.sections[section]
  if not s then
    s = { order = {}, values = {} }
    d.sections[section] = s
    d.order[#d.order + 1] = section
  end
  if s.values[key] == nil then s.order[#s.order + 1] = key end
  s.values[key] = tostring(value)
end

function M.serialize(d)
  local out = {}
  for _, name in ipairs(d.order) do
    out[#out + 1] = "[" .. name .. "]"
    local s = d.sections[name]
    for _, k in ipairs(s.order) do out[#out + 1] = k .. '="' .. s.values[k] .. '"' end
  end
  return table.concat(out, "\r\n") .. "\r\n"
end

return M
