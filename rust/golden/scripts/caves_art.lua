-- Cave and relic art data the logic reads (ruling R7): the alpha masks caves_mask_init builds
-- from the cave/relic sprites (alpha >= 250 as solid spans per row), the outlines, branches and
-- stems cave_outline_build derives from them, and the sprite sizes caves/bones/relics measure.
-- myco_core embeds the masks and sizes (src/world/cave_art.json); tests/caves.rs checks the
-- copy and rebuilds the outlines from the masks. Run: tools/golden.sh caves_art
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")

caves_mask_init(global, global)
assert(gm.is_struct(global.cave_masks), "caves_mask_init made no masks")

local function spans(m)
  local rows = {}
  for y = 0, m.h - 1 do
    local row, x = {}, 0
    while x < m.w do
      if gm.buffer_peek(m.buffer, (y * m.w + x) * 4 + 3, gm.buffer_u8) >= 250 then
        local x0 = x
        while x + 1 < m.w and gm.buffer_peek(m.buffer, (y * m.w + x + 1) * 4 + 3, gm.buffer_u8) >= 250 do
          x = x + 1
        end
        row[#row + 1] = x0
        row[#row + 1] = x
      end
      x = x + 1
    end
    rows[#rows + 1] = row
  end
  return rows
end

local function mask_data(id, sprite)
  local m = gm.variable_struct_get(global.cave_masks, id)
  local branches = {}
  for b = 0, gm.array_length(m.branches) - 1 do
    local br = m.branches[b]
    branches[#branches + 1] = { at = br.at, delay = br.delay or 0, points = L.plain(br.points) }
  end
  return {
    id = id,
    w = m.w,
    h = m.h,
    sprite = { gm.sprite_get_width(sprite), gm.sprite_get_height(sprite) },
    spans = spans(m),
    outline = L.plain(m.outline),
    branches = branches,
    stems = L.plain(m.stems),
  }
end

local out = { caves = {}, relics = {}, bones = {} }
local defs = cave_defs(global, global)
for i = 0, gm.array_length(defs) - 1 do
  local d = defs[i]
  local c = mask_data(d.id, d.sprite)
  c.left = d.left
  c.def_w = d.w
  c.feet = L.plain(d.feet)
  out.caves[#out.caves + 1] = c
end
for _, id in ipairs({ "t_rex", "ufo" }) do
  local d = relic_def(global, global, id)
  local r = mask_data(id, d.sprite)
  r.def_w = d.w
  out.relics[#out.relics + 1] = r
end
local kinds = bone_kinds(global, global)
for i = 0, gm.array_length(kinds) - 1 do
  local k = kinds[i]
  out.bones[#out.bones + 1] = {
    id = k.id, weight = k.weight, near = k.near, far = k.far, deep_share = k.deep_share,
    size = { gm.sprite_get_width(k.sprite), gm.sprite_get_height(k.sprite) },
  }
end
return out
