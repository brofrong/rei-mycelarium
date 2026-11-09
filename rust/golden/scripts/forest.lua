-- Forest generation on a seeded RNG (gm.random is bit-identical to myco_core's Rng):
-- obj_forest_controller Create_0 on a fresh instance table, then surface chunks in order; the
-- pickers; backdrop leaves over a few hundred steps; leaf_fg_drop. For tests/forest.rs.
local gm = require("gm")
local L = dofile("rust/golden/scripts/world_lib.lua")
L.preload()

local function host(t)
  local roots = t.roots
  return { x = t.x, y = t.y, flip = t.flip, roots = L.plain(roots) }
end
local function chunk_data(ch)
  local d = { trees = {}, small_trees = {}, bushes = L.plain(ch.bushes), plants = L.plain(ch.plants),
    decor_trees = L.plain(ch.decor_trees), stumps = {} }
  for i = 0, gm.array_length(ch.trees) - 1 do
    local t = ch.trees[i]
    local h = host(t)
    h.sprite = gm.sprite_get_name(t.sprite)
    d.trees[#d.trees + 1] = h
  end
  for i = 0, gm.array_length(ch.small_trees) - 1 do
    local t = ch.small_trees[i]
    local h = host(t)
    h.sprite = gm.sprite_get_name(t.sprite)
    d.small_trees[#d.small_trees + 1] = h
  end
  for i = 0, gm.array_length(ch.stumps) - 1 do
    local s = ch.stumps[i]
    local h = host(s)
    h.part = s.part.left
    d.stumps[#d.stumps + 1] = h
  end
  return d
end

local out = { start_x = global.start_x, chunk_w = global.world.chunk_w, runs = {} }
for _, seed in ipairs({ 4242, 7, 123456 }) do
  gm.random_set_seed(seed)
  local fc = {}
  gml_Object_obj_forest_controller_Create_0(fc, fc)
  local run = { seed = seed, start_stump = chunk_data({ trees = gm.array(), small_trees = gm.array(),
    bushes = gm.array(), plants = gm.array(), decor_trees = gm.array(), stumps = gm.array(fc.start_stump) }).stumps[1],
    camera = { fc.camera_x, fc.camera_y }, chunks = {} }
  for _, cx in ipairs({ 0, -1, 1, -2, 2, 7, -9 }) do
    run.chunks[#run.chunks + 1] = { cx = cx, data = chunk_data(fc.surface_get_chunk(fc, fc, cx)) }
  end
  -- the pickers on their own
  run.picks = {}
  for i = 1, 20 do
    run.picks[#run.picks + 1] = { fc.bush_pick(fc, fc), fc.part_pick(fc, fc, fc.plant_parts),
      fc.part_pick(fc, fc, fc.decor_tree_parts), L.plain(fc.tree_roots_pick(fc, fc, fc.tree_sprites[i % 3])),
      L.plain(fc.tree_roots_pick(fc, fc, fc.small_tree_sprites[i % 3])) }
  end
  run.stump_make = {}
  for q = 0, 3 do
    run.stump_make[#run.stump_make + 1] = host(fc.stump_make(fc, fc, fc.stump_parts[q], 100 * q - 150))
  end
  out.runs[#out.runs + 1] = run
end

-- backdrop leaves: 400 steps, the wind turns at step 150 and becomes strong at step 260
gm.random_set_seed(99)
global.backdrop_leaves = nil
global.wind_dir, global.wind_speed = 1, 2.5
out.leaves = {}
local cam = -960
for step = 1, 400 do
  if step == 150 then global.wind_dir = -1 end
  if step == 260 then global.wind_speed = vis(global, global, "wind_max_strength") end
  cam = cam + ((step > 200) and 7 or -3)
  backdrop_leaves_step(global, global, cam, 1920)
  if step % 50 == 0 or step == 1 then
    out.leaves[#out.leaves + 1] = { step = step, cam = cam, bands = L.plain(global.backdrop_leaves) }
  end
end
global.backdrop_leaves = nil

out.drop = {}
for _, l in ipairs({
  {}, { turn = -1, angle = 0 }, { turn = 0, angle = 10, mirror = 1 }, { turn = 5, angle = 30, mirror = 0 },
  { turn = 13.7, angle = -45, mirror = 1 }, { turn = 20, angle = 90 }, { turned = true, angle = 200 },
  { turned = false, angle = 1, mirror = 1 }, { turn = -3, turned = true, angle = -170, mirror = 2 },
}) do
  local s = gm.struct({})
  for k, v in pairs(l) do s[k] = v end
  out.drop[#out.drop + 1] = { l.turn or "none", l.turned == nil and "none" or (l.turned and 1 or 0),
    l.angle or "none", l.mirror or "none", leaf_fg_drop(global, global, s), leaf_fg_mirror(global, global, s) }
end
return out
