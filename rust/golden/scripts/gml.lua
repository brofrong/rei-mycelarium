-- Reference outputs of the GML math layer (game/gm/mathx.lua, gm.mod) for tests/gml.rs.
-- Each entry is { inputs..., output }.
local gm = require("gm")

local xs = { -3.5, -2.5, -1.5, -0.5, 0, 0.5, 1.5, 2.5, 3.5, 0.49999999999999994, -0.49999999999999994,
  2.4, 2.6, -2.4, -2.6, 7, -7, 1e15 + 0.5, 123456.5, -123456.5, 0.1, -0.1 }
local pts = { { 0, 0, 1, 0 }, { 0, 0, 0, 1 }, { 0, 0, -1, 0 }, { 0, 0, 0, -1 }, { 0, 0, 1, 1 },
  { 5, 5, 5, 5 }, { 10, -3, -7.5, 12.25 }, { -100, 40, 33, -2 }, { 1.5, 2.5, 1.5000001, 2.4 },
  { 0, 0, -1, -1e-9 }, { 0, 0, -1, 1e-9 }, { 320, 240, 319.999, 900 } }
local dirs = { -720, -405, -90, -45.5, 0, 1e-7, 30, 45, 89.999, 90, 135, 180, 225, 270, 359.99, 360, 450, 1000.25 }
local lens = { 0, 1, -2.5, 17.3, 1000 }
local angles = { { 0, 0 }, { 10, 350 }, { 350, 10 }, { 180, 0 }, { 0, 180 }, { -190, 170 }, { 720, -1 },
  { 45.5, -45.5 }, { 1e-9, -1e-9 }, { 359, 1 }, { -540, 0 }, { 0.5, 180.5 } }
local mods = { { 7, 3 }, { -7, 3 }, { 7, -3 }, { -7, -3 }, { 7.5, 2 }, { -7.5, 2 }, { 0, 5 }, { 5, 7 },
  { -0.5, 1 }, { 1e10 + 0.25, 360 }, { 359.999, 360 }, { -360, 360 } }

local out = { round = {}, frac = {}, sign = {}, point_direction = {}, point_distance = {},
  lengthdir_x = {}, lengthdir_y = {}, angle_difference = {}, mod = {}, lerp = {}, clamp = {} }
for _, x in ipairs(xs) do
  out.round[#out.round + 1] = { x, gm.round(x) }
  out.frac[#out.frac + 1] = { x, gm.frac(x) }
  out.sign[#out.sign + 1] = { x, gm.sign(x) }
end
for _, p in ipairs(pts) do
  out.point_direction[#out.point_direction + 1] = { p[1], p[2], p[3], p[4], gm.point_direction(p[1], p[2], p[3], p[4]) }
  out.point_distance[#out.point_distance + 1] = { p[1], p[2], p[3], p[4], gm.point_distance(p[1], p[2], p[3], p[4]) }
end
for _, l in ipairs(lens) do
  for _, d in ipairs(dirs) do
    out.lengthdir_x[#out.lengthdir_x + 1] = { l, d, gm.lengthdir_x(l, d) }
    out.lengthdir_y[#out.lengthdir_y + 1] = { l, d, gm.lengthdir_y(l, d) }
  end
end
for _, a in ipairs(angles) do
  out.angle_difference[#out.angle_difference + 1] = { a[1], a[2], gm.angle_difference(a[1], a[2]) }
end
for _, m in ipairs(mods) do
  out.mod[#out.mod + 1] = { m[1], m[2], gm.mod(m[1], m[2]) }
end
for _, t in ipairs({ -0.5, 0, 0.25, 0.5, 1, 1.75 }) do
  out.lerp[#out.lerp + 1] = { -3, 11.5, t, gm.lerp(-3, 11.5, t) }
end
for _, x in ipairs({ -5, -1, 0, 0.5, 1, 2, 10 }) do
  out.clamp[#out.clamp + 1] = { x, 0, 1, gm.clamp(x, 0, 1) }
end
return out
