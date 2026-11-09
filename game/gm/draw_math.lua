-- Pure helpers for GameMaker drawing semantics (no love dependency).
local M = {}

-- GM colours are integers in BGR order: 0xBBGGRR.
function M.bgr_to_rgb(c)
  c = math.floor(c) % 0x1000000
  return (c % 256) / 255, (math.floor(c / 256) % 256) / 255, math.floor(c / 65536) / 255
end

function M.make_colour_rgb(r, g, b)
  return math.floor(b) * 65536 + math.floor(g) * 256 + math.floor(r)
end
M.make_color_rgb = M.make_colour_rgb

function M.colour_get_red(c) return math.floor(c) % 256 end
function M.colour_get_green(c) return math.floor(c / 256) % 256 end
function M.colour_get_blue(c) return math.floor(c / 65536) % 256 end
M.color_get_red, M.color_get_green, M.color_get_blue = M.colour_get_red, M.colour_get_green, M.colour_get_blue

function M.merge_colour(c1, c2, amount)
  local function mix(a, b) return math.floor(a + (b - a) * amount + 0.5) end
  return M.make_colour_rgb(mix(M.colour_get_red(c1), M.colour_get_red(c2)),
    mix(M.colour_get_green(c1), M.colour_get_green(c2)), mix(M.colour_get_blue(c1), M.colour_get_blue(c2)))
end
M.merge_color = M.merge_colour

function M.make_colour_hsv(h, s, v)
  h, s, v = h / 255 * 360, s / 255, v / 255
  local c = v * s
  local x = c * (1 - math.abs((h / 60) % 2 - 1))
  local m = v - c
  local r, g, b
  if h < 60 then r, g, b = c, x, 0
  elseif h < 120 then r, g, b = x, c, 0
  elseif h < 180 then r, g, b = 0, c, x
  elseif h < 240 then r, g, b = 0, x, c
  elseif h < 300 then r, g, b = x, 0, c
  else r, g, b = c, 0, x end
  return M.make_colour_rgb((r + m) * 255 + 0.5, (g + m) * 255 + 0.5, (b + m) * 255 + 0.5)
end
M.make_color_hsv = M.make_colour_hsv

-- Offset of a (possibly trimmed) texture-page frame from the draw position, in
-- scaled sprite space before rotation.
function M.frame_offset(meta, frame, xs, ys)
  local f = meta.frames[frame + 1]
  return (f.dst[1] - meta.origin[1]) * xs, (f.dst[2] - meta.origin[2]) * ys
end

-- GM floors the sub-image and wraps it into range (also for negatives).
function M.frame_index(count, sub)
  if count <= 0 then return 0 end
  return math.floor(sub) % count
end

return M
