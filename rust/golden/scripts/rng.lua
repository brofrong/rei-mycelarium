-- Reference sequence of game/gm/random.lua (xorshift64*) for rust/crates/myco_core/tests/rng.rs.
local gm = require("gm")

local function seq(n, f)
  local t = {}
  for i = 1, n do t[i] = f() end
  return t
end

local out = {}
gm.random_set_seed(12345)
out.seed = gm.random_get_seed()
out.random = seq(50, function() return gm.random(1) end)
out.irandom = seq(20, function() return gm.irandom(10) end)
out.irandom_range = seq(20, function() return gm.irandom_range(-5, 5) end)
out.choose = seq(20, function() return gm.choose(1, 2, 3) end)

-- extra seeds and argument shapes: negative and zero seeds, fractional n, swapped ranges
out.extra = {}
for _, s in ipairs({ 0, 1, -1, -12345, 99999, 2147483646, 31607.7 }) do
  gm.random_set_seed(s)
  local row = { seed = s, got_seed = gm.random_get_seed() }
  row.random = seq(5, function() return gm.random(1) end)
  row.random_range = seq(5, function() return gm.random_range(-3.5, 7.25) end)
  row.irandom_frac = seq(5, function() return gm.irandom(4.7) end)
  row.irandom_range_swapped = seq(5, function() return gm.irandom_range(9.9, -2.2) end)
  row.choose5 = seq(5, function() return gm.choose(10, 20, 30, 40, 50) end)
  out.extra[#out.extra + 1] = row
end
return out
