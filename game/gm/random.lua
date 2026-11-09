-- GML random numbers on a seedable xorshift64* generator (no love dependency).
-- Sequences differ from GameMaker's, but are deterministic for a given seed.
local bit = require("bit")
local ffi = require("ffi")

local M = {}
local state = ffi.new("uint64_t[1]")
local seed = 0

function M.random_set_seed(s)
  seed = math.floor(s)
  state[0] = ffi.cast("uint64_t", seed) * 6364136223846793005ULL + 1442695040888963407ULL
  if state[0] == 0 then state[0] = 88172645463325252ULL end
end

function M.random_get_seed() return seed end

-- The whole generator state (seed and xorshift word), to set aside and restore later
-- (conformance runs keep audio draws off the game's sequence, see gm/conform.lua).
function M.get_state() return { seed = seed, word = ffi.new("uint64_t", state[0]) } end
function M.set_state(st) seed, state[0] = st.seed, st.word end

-- M.on_draw: conformance debugging hook called on every draw (gm/conform.lua MYCO_TRACE_DRAWS)
M.on_draw = nil
local function next01()
  if M.on_draw then M.on_draw() end
  local x = state[0]
  x = bit.bxor(x, bit.rshift(x, 12))
  x = bit.bxor(x, bit.lshift(x, 25))
  x = bit.bxor(x, bit.rshift(x, 27))
  state[0] = x
  local v = x * 2685821657736338717ULL
  return tonumber(bit.rshift(v, 11)) / 9007199254740992
end

function M.random(n) return next01() * n end
function M.random_range(a, b) return a + next01() * (b - a) end
function M.irandom(n) return math.floor(next01() * (math.floor(n) + 1)) end
function M.irandom_range(a, b)
  a, b = math.floor(math.min(a, b)), math.floor(math.max(a, b))
  return a + math.floor(next01() * (b - a + 1))
end
function M.choose(...)
  local n = select("#", ...)
  return (select(math.floor(next01() * n) + 1, ...))
end
-- randomise reseeds from the clock; with `fixed` (MYCO_RNG_SEED, conformance runs) it reseeds
-- like random_set_seed(fixed), the same as the Rust port does with that variable
function M.randomise_from(fixed)
  if fixed then
    M.random_set_seed(fixed)
  else
    M.random_set_seed(math.floor((os.time() * 1000 + os.clock() * 1e6) % 2147483647))
  end
  return seed
end
function M.randomise() return M.randomise_from(tonumber(os.getenv("MYCO_RNG_SEED") or "")) end
M.randomize = M.randomise

M.random_set_seed(0)
return M
