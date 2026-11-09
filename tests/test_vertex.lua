local vertex = require("gm.vertex")
local buffer = require("gm.buffer")

local function fmt_pos_col()
  vertex.vertex_format_begin()
  vertex.vertex_format_add_position()
  vertex.vertex_format_add_colour()
  return vertex.vertex_format_end()
end

local function vb_with(fmt, pts)
  local vb = vertex.vertex_create_buffer()
  vertex.vertex_begin(vb, fmt)
  for _, p in ipairs(pts) do
    vertex.vertex_position(vb, p[1], p[2])
    vertex.vertex_colour(vb, 0x0000FF, 0.5)
  end
  vertex.vertex_end(vb)
  return vb
end

return {
  bytes_roundtrip_through_merged_buffers = function()
    local fmt = fmt_pos_col()
    local a = vertex.serialize(vb_with(fmt, { { 1, 2 }, { 3, 4 }, { 5, 6 } }))
    local b = vertex.serialize(vb_with(fmt, { { 7, 8 }, { 9, 10 }, { 11, 12 } }))
    assert(#a == 36 and #b == 36) -- 12 bytes per vertex: 2 floats + RGBA
    local merged = buffer.create(72, buffer.fixed, 1)
    buffer.copy(buffer.from_string(a), 0, 36, merged, 0)
    buffer.copy(buffer.from_string(b), 0, 36, merged, 36)
    local vb = vertex.vertex_create_buffer()
    vertex.restore_bytes(vb, buffer.to_string(merged), fmt)
    assert(vertex.vertex_get_number(vb) == 6)
    local v = vertex.vertex_at(vb, 5)
    assert(v[1] == 11 and v[2] == 12 and v[5] == 1 and v[6] == 0 and math.abs(v[8] - 128 / 255) < 1e-6)
  end,
}
