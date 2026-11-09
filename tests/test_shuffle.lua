-- array_shuffle draws from the game RNG (GML's array_shuffle uses the seeded generator), so a
-- seeded run shuffles the same way every time and the Rust port can repeat it.
local gm = require("gm.core")
local random = require("gm.random")

return {
  shuffle_uses_game_rng = function()
    local a = gm.array(1, 2, 3, 4, 5, 6, 7, 8)
    random.random_set_seed(42)
    local s1 = gm.array_shuffle(a)
    random.random_set_seed(42)
    local s2 = gm.array_shuffle(a)
    for i = 0, 7 do assert(s1[i] == s2[i], "shuffle not reproducible under one seed") end
    -- Fisher-Yates from the end with j = floor(random(1) * (i + 1)), as myco_core::life does
    random.random_set_seed(42)
    local want = { 1, 2, 3, 4, 5, 6, 7, 8 }
    for i = 8, 2, -1 do
      local j = math.floor(random.random(1) * i) + 1
      want[i], want[j] = want[j], want[i]
    end
    for i = 0, 7 do assert(s1[i] == want[i + 1], "unexpected order at " .. i) end
  end,
}
