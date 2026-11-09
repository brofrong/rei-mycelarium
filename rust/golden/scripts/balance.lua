-- Balance tables as the LÖVE port holds them, for rust/crates/myco_core/tests/balance.rs:
-- raw balance_defaults(), global.balance after game_mode_apply for each mode, global.visual,
-- global.worldgen, and every literal key passed to bal/vis/wld in game/myco/gen/*.lua.
local gm = require("gm")

local function plain(v)
  if gm.is_array(v) then
    local t = {}
    for i = 0, gm.array_length(v) - 1 do t[#t + 1] = plain(v[i]) end
    return t
  end
  return v
end

local function dump(s)
  local t = {}
  local names = gm.variable_struct_get_names(s)
  for i = 0, gm.array_length(names) - 1 do
    local k = names[i]
    t[k] = plain(gm.variable_struct_get(s, k))
  end
  return t
end

local used = { bal = {}, vis = {}, wld = {} }
local seen = { bal = {}, vis = {}, wld = {} }
local dir = "myco/gen"
for _, f in ipairs(love.filesystem.getDirectoryItems(dir)) do
  if f:match("%.lua$") then
    local src = love.filesystem.read(dir .. "/" .. f)
    for fn, key in src:gmatch("(%a%a%a)%(self, other, \"([%w_]+)\"%)") do
      if used[fn] and not seen[fn][key] then
        seen[fn][key] = true
        used[fn][#used[fn] + 1] = key
      end
    end
  end
end
for _, list in pairs(used) do table.sort(list) end

local mode_before = game_mode(global, global)
local out = {
  defaults = dump(balance_defaults(global, global)),
  visual = dump(global.visual),
  worldgen = dump(global.worldgen),
  used = used,
}
game_mode_apply(global, global, "exploration")
out.exploration = dump(global.balance)
game_mode_apply(global, global, "survival")
out.survival = dump(global.balance)
game_mode_apply(global, global, mode_before)
return out
