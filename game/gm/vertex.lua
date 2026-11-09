-- GameMaker vertex buffers on LÖVE meshes. A vertex starts at vertex_position;
-- colour/texcoord calls modify the current vertex.
local dmath = require("gm.draw_math")

local M = {}
local buffers, next_buf = {}, 0
local formats, next_fmt = {}, 0
local fmt_building

function M.vertex_format_begin() fmt_building = {} end
function M.vertex_format_add_position() end
function M.vertex_format_add_position_3d() end
function M.vertex_format_add_colour() end
M.vertex_format_add_color = M.vertex_format_add_colour
function M.vertex_format_add_texcoord() end
function M.vertex_format_add_normal() end
function M.vertex_format_end()
  local id = next_fmt
  next_fmt = next_fmt + 1
  formats[id] = fmt_building or {}
  fmt_building = nil
  return id
end
function M.vertex_format_delete(id) formats[id] = nil end

function M.vertex_create_buffer()
  local id = next_buf
  next_buf = next_buf + 1
  buffers[id] = { verts = {}, mesh = nil, frozen = false }
  return id
end
function M.vertex_delete_buffer(id)
  local b = buffers[id]
  if b and b.mesh then b.mesh:release() end
  buffers[id] = nil
end

function M.vertex_begin(id)
  local b = buffers[id]
  b.verts, b.frozen = {}, false
  if b.mesh then b.mesh:release(); b.mesh = nil end
end
function M.vertex_position(id, x, y)
  local v = buffers[id].verts
  v[#v + 1] = { x, y, 0, 0, 1, 1, 1, 1 }
end
function M.vertex_position_3d(id, x, y) M.vertex_position(id, x, y) end
function M.vertex_colour(id, col, alpha)
  local v = buffers[id].verts
  local cur = v[#v]
  cur[5], cur[6], cur[7] = dmath.bgr_to_rgb(col)
  cur[8] = alpha
end
M.vertex_color = M.vertex_colour
function M.vertex_texcoord(id, u, v)
  local vs = buffers[id].verts
  local cur = vs[#vs]
  cur[3], cur[4] = u, v
end
function M.vertex_normal() end
function M.vertex_end() end
function M.vertex_freeze(id) buffers[id].frozen = true end
function M.vertex_get_number(id) return #buffers[id].verts end

local MODES = { [4] = "triangles", [5] = "strip", [6] = "fan" }
function M.vertex_submit(id, prim, tex)
  local b = buffers[id]
  if not b or #b.verts < 3 then return end
  local mode = MODES[prim] or "triangles"
  if not b.mesh or b.mesh_mode ~= mode then
    if b.mesh then b.mesh:release() end
    b.mesh = love.graphics.newMesh(b.verts, mode, b.frozen and "static" or "stream")
    b.mesh_mode = mode
  end
  if type(tex) == "userdata" then b.mesh:setTexture(tex) else b.mesh:setTexture() end
  love.graphics.setColor(1, 1, 1, 1)
  love.graphics.draw(b.mesh)
end

return M
