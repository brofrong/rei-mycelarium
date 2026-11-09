local dm = require("gm.draw_math")

return {
  bgr = function()
    local r, g, b = dm.bgr_to_rgb(0x0000FF)
    assert(r == 1 and g == 0 and b == 0)
    r, g, b = dm.bgr_to_rgb(0xFF0000)
    assert(r == 0 and g == 0 and b == 1)
  end,
  make_colour = function()
    assert(dm.make_colour_rgb(255, 128, 0) == 0x0080FF)
    assert(dm.colour_get_red(0x0080FF) == 255 and dm.colour_get_green(0x0080FF) == 128 and dm.colour_get_blue(0x0080FF) == 0)
  end,
  origin_with_trim = function() -- origin (32,32), frame trimmed: dst=(4,6)
    local meta = { origin = { 32, 32 }, frames = { { dst = { 4, 6 } } } }
    local ox, oy = dm.frame_offset(meta, 0, 1, 1)
    assert(ox == -28 and oy == -26)
  end,
  origin_mirrored = function()
    local meta = { origin = { 32, 32 }, frames = { { dst = { 4, 6 } } } }
    local ox, oy = dm.frame_offset(meta, 0, -2, 1)
    assert(ox == 56 and oy == -26)
  end,
  frame_index_wraps_and_floors = function()
    assert(dm.frame_index(5, 2.7) == 2 and dm.frame_index(5, 7) == 2 and dm.frame_index(5, -1) == 4)
  end,
  merge_colour = function()
    assert(dm.merge_colour(0x000000, 0xFFFFFF, 0.5) == 0x808080 or dm.merge_colour(0x000000, 0xFFFFFF, 0.5) == 0x7F7F7F)
  end,
}
