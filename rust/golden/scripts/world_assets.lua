-- Data the world/forest logic derives from art (lake masks, food sheet cells, sprite sizes, leaf
-- frame bounds) plus the soil-row helpers, for myco_core (it embeds this file: the masks are
-- the only logic input that comes from pixels) and tests/world.rs.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.preload()

local out = {}
-- lakes_init: per lake kind left/width/height/top/bottom and the solid spans of every row
out.lake_kinds = {}
for i = 0, gm.array_length(global.lake_kinds) - 1 do
  local k = global.lake_kinds[i]
  local spans = {}
  for y = 0, k.height - 1 do
    spans[y + 1] = L.plain(k.spans[y])
  end
  out.lake_kinds[i + 1] = { left = k.left, width = k.width, height = k.height, top = k.top,
    bottom = k.bottom, spans = spans }
end
out.lake_layout = L.plain(lake_layout(global, global))

local function size(spr)
  return { gm.sprite_get_width(spr), gm.sprite_get_height(spr) }
end
out.sprites = {
  food_small = size(food_small), food_medium = size(food_medium), food_big = size(food_big),
  water_flare = size(water_flare), originshroom = size(originshroom), stump = size(stump),
}
out.food_cells = {}
out.food_value = {}
for k = -1, 3 do
  out.food_cells[#out.food_cells + 1] = world_food_cells(global, global, k)
  out.food_value[#out.food_value + 1] = world_food_value(global, global, k)
end
out.food_kinds = world_food_kinds(global, global)
out.turf = gm.asset_get_index("trintrava1")

-- leaf_fg_art frame bounds (sprites themselves are the renderer's)
local function frame(f)
  return { f.x1, f.y1, f.x2, f.y2, f.mid_x, f.mid_y }
end
local art = global.leaf_fg_art
out.leaf_fg = { rest = frame(art.rest), turned = frame(art.turned), carry_x = art.carry_x,
  carry_y = art.carry_y, turn = {} }
for i = 0, gm.array_length(art.turn) - 1 do
  out.leaf_fg.turn[i + 1] = frame(art.turn[i])
end

-- soil rows and sprites
out.dense_row = world_dense_row(global, global)
out.deep_row = world_deep_row(global, global)
out.ocean_row = world_ocean_row(global, global)
out.ocean_y = world_ocean_y(global, global)
out.deep_y = world_deep_y(global, global)
out.dense_y = world_dense_y(global, global)
out.seam = {}
for _, s in ipairs({ 0, 0.5, 1, 2.25, 3, 7.5, 12 }) do
  out.seam[#out.seam + 1] = { s, world_seam_row_at_screens(global, global, s) }
end
out.rows = {}
for row = -2, 320 do
  out.rows[#out.rows + 1] = {
    row, world_soil_band(global, global, row), world_soil_tier(global, global, row),
    gm.sprite_get_name(world_soil_sprite(global, global, row)),
    gm.sprite_get_name(world_dirt_sprite(global, global, row)),
    gm.sprite_get_name(world_soil_edge_sprite(global, global, world_soil_band(global, global, row), row)),
  }
end
out.is_ocean = {}
for _, y in ipairs({ out.ocean_y - 1, out.ocean_y, out.ocean_y + 0.5, 0, 480 }) do
  out.is_ocean[#out.is_ocean + 1] = { y, world_is_ocean_at(global, global, y) }
end

-- lake helpers
out.lake_pick = {}
for _, t in ipairs({ 0, 0.25, 0.5, 0.9, 1 }) do
  for _, r in ipairs({ 0, 0.05, 0.1, 0.2, 0.3, 0.45, 0.6, 0.99 }) do
    out.lake_pick[#out.lake_pick + 1] = { t, r, lake_pick_kind(global, global, t, r) }
  end
end
out.lake_amount = { lake_amount(global, global, 0), lake_amount(global, global, 1), lake_amount(global, global, 2) }
out.lake_flare_count = { lake_flare_count(global, global, 0), lake_flare_count(global, global, 1),
  lake_flare_count(global, global, 2) }
out.flare_life = lake_flare_life_ms(global, global)
out.flare_alpha = {}
for _, age in ipairs({ -5, 0, 10, 100, 250, 400, 800, 1200, 1700, 2500, 4000, 9000 }) do
  out.flare_alpha[#out.flare_alpha + 1] = { age, lake_flare_alpha(global, global, age) }
end
out.cut_row = {}
for k = 0, 2 do
  local kd = global.lake_kinds[k]
  for _, a in ipairs({ -3, 0, 1, 20, 33.5, 67, 100, 200 }) do
    local lake = { amount = a, full = lake_amount(global, global, k) }
    out.cut_row[#out.cut_row + 1] = { k, a, lake_cut_row(global, global, lake, kd) }
  end
end
out.solid = {}
for k = 0, 2 do
  local kd = global.lake_kinds[k]
  for y = -2, kd.height + 1, 3 do
    for x = -2, kd.width + 1, 5 do
      out.solid[#out.solid + 1] = lake_solid_at(global, global, kd, x, y) and 1 or 0
    end
  end
end
return out
