-- Data the life logic derives from art (ruling R7: the render task owns the pixel readers,
-- logic gets their output): hedgehog_back_init's back points, squirrel_trunk_edges of the three
-- tree sprites, leaf_art_init's frame bounds/offsets, and the sprite metadata the animals
-- measure. myco_core embeds this as src/life/art.json; tests/life.rs checks the copy.
-- Run: MYCO_IDENTITY=MycelariumTest-t10 tools/golden.sh life_assets
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")

local out = {}
local function meta(spr)
  return { gm.sprite_get_width(spr), gm.sprite_get_height(spr), gm.sprite_get_xoffset(spr),
    gm.sprite_get_yoffset(spr) }
end
out.sprites = {}
for _, n in ipairs({ "hedgehog", "boar", "squirrel", "crow", "tree01", "tree02", "tree03", "waterdrop" }) do
  out.sprites[n] = meta(assert(_G[n], n))
end

global.hedgehog_back = gm.undefined
hedgehog_back_init(global, global)
out.hedgehog_back = {}
for f = 0, 7 do
  local p = global.hedgehog_back[f]
  out.hedgehog_back[f + 1] = { p.x, p.y }
end

global.squirrel_trunks = gm.undefined
out.trunks = {}
for _, n in ipairs({ "tree01", "tree02", "tree03" }) do
  local c = squirrel_trunk_edges(global, global, _G[n])
  out.trunks[n] = { left = L.plain(c.left), right = L.plain(c.right), origin_y = c.origin_y, h = c.h }
end

global.leaf_art = gm.undefined
leaf_art_init(global, global)
local function frames(arr)
  local t = {}
  for i = 0, gm.array_length(arr) - 1 do
    local f = arr[i]
    t[i + 1] = { f.x1, f.y1, f.x2, f.y2, f.mid_x, f.mid_y, f.off_x, f.off_y }
  end
  return t
end
out.leaf_art = { faces = {}, turns = {} }
for k = 0, 1 do
  out.leaf_art.faces[k + 1] = frames(global.leaf_art.faces[k])
  out.leaf_art.turns[k + 1] = frames(global.leaf_art.turns[k])
end
return out
