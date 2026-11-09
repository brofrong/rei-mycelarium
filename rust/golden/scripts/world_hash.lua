-- world_hash01 / cave_hash on 10 000 argument triples (several world seeds) for
-- rust/crates/myco_core/tests/world.rs::hash_matches_love.
local s = 20261109
local function rnd(n) -- deterministic LCG in [0, n)
  s = (s * 1103515245 + 12345) % 2147483648
  return s % n
end

local salts = { 0, 1, 7, 11, 12, 13, 21, 22, 23, 31, 32, 33, 34, 35, 36, 37, 38, 39, 81, 82, 83, 84,
  91, 92, 93, 95, 96, 97, 98, 99, 100, 120, 121, 130, 140, 150, 611 }
local seeds = { 31607, 7, 0, 99999, 12345 }

local saved_seed = global.world.seed
local rows = {}
for i = 1, 10000 do
  local kind = i % 5
  local a, b, c
  if kind == 0 then -- chunk / layer keys: cx + layer * 0x250000
    a = (rnd(81) - 40) + rnd(60) * 0x250000
    b = rnd(400)
  elseif kind == 1 then -- pixel positions, fractional
    a = (rnd(400000) - 200000) + rnd(1000) / 1000
    b = (rnd(40000) - 20000) / 7
  elseif kind == 2 then -- small integers, negatives
    a = rnd(2001) - 1000
    b = rnd(201) - 100
  elseif kind == 3 then -- large magnitudes
    a = (rnd(2000000) - 1000000) * 37
    b = rnd(70000) - 35000
  else -- loop counters like i * 16 + j
    a = rnd(5000) * 16 + rnd(16)
    b = rnd(12)
  end
  c = salts[rnd(#salts) + 1]
  if i % 7 == 0 then c = rnd(1000) end
  local seed = seeds[(i % #seeds) + 1]
  global.world.seed = seed
  rows[i] = { seed, a, b, c, world_hash01(global, global, a, b, c), cave_hash(global, global, a, b, c) }
end
global.world.seed = saved_seed
return { rows = rows }
