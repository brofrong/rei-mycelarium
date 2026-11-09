local r = require("gm.random")

return {
  seeded_is_deterministic = function()
    r.random_set_seed(1234)
    local a = { r.random(1), r.irandom(100), r.random_range(5, 6) }
    r.random_set_seed(1234)
    local b = { r.random(1), r.irandom(100), r.random_range(5, 6) }
    assert(a[1] == b[1] and a[2] == b[2] and a[3] == b[3] and r.random_get_seed() == 1234)
  end,
  ranges = function()
    r.random_set_seed(7)
    for _ = 1, 2000 do
      local x, i, j = r.random(10), r.irandom(3), r.irandom_range(-2, 2)
      assert(x >= 0 and x < 10 and i >= 0 and i <= 3 and i == math.floor(i) and j >= -2 and j <= 2)
      local f = r.random_range(-1, 1)
      assert(f >= -1 and f < 1)
    end
  end,
  choose_picks_argument = function()
    r.random_set_seed(3)
    local seen = {}
    for _ = 1, 200 do seen[r.choose("a", "b", "c")] = true end
    assert(seen.a and seen.b and seen.c)
  end,
}
