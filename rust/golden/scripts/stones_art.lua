-- Stone data derived from pixels (R7: stones_init / stone_tiles_init are T13's, logic reads
-- their output): the seven loose-stone kinds (centre, radius, body), global.stone_reach and the
-- solid spans of the 7x7 stone_tile atlas, plus the tile-role tables, for tests/stones.rs.
-- Run: tools/golden.sh stones_art
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")

stones_init(global, global)
stone_tiles_init(global, global)

local out = { reach = global.stone_reach, kinds = {}, tiles = {} }
for i = 0, gm.array_length(global.stone_kinds) - 1 do
  local k = global.stone_kinds[i]
  out.kinds[#out.kinds + 1] = { left = k.left, w = k.w, ox = k.ox, oy = k.oy, radius = k.radius, body = k.body }
end
for t = 0, gm.array_length(global.stone_tiles) - 1 do
  out.tiles[#out.tiles + 1] = L.plain(global.stone_tiles[t])
end

-- stone_tile_solid_at over every tile at a few columns per row
out.solid = {}
for t = 0, 48 do
  local tx, ty = t % 7, math.floor(t / 7)
  for y = -1, 48, 1 do
    for x = -1, 49, 7 do
      out.solid[#out.solid + 1] = stone_tile_solid_at(global, global, tx, ty, x, y + 0.5) and 1 or 0
    end
  end
end

-- role tables for every edge combination and depth
out.roles = {}
for mask = 0, 15 do
  for _, depth in ipairs({ 0, 0.5, 1, 2 }) do
    local up, down, left, right = mask % 2 == 1, math.floor(mask / 2) % 2 == 1,
      math.floor(mask / 4) % 2 == 1, math.floor(mask / 8) % 2 == 1
    out.roles[#out.roles + 1] = { mask, depth, L.plain(stone_role_tiles(global, global, up, down, left, right, depth)) }
  end
end
out.inner = {
  L.plain(stone_inner_corner_tile(global, global, true, true)),
  L.plain(stone_inner_corner_tile(global, global, true, false)),
  L.plain(stone_inner_corner_tile(global, global, false, true)),
  L.plain(stone_inner_corner_tile(global, global, false, false)),
}
return out
