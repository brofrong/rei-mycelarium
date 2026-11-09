-- GameMaker vertex buffers on LÖVE meshes. A vertex starts at vertex_position;
-- colour/texcoord calls modify the current vertex. Buffers created from vertex
-- buffers hold real bytes in the format's layout, so games can merge them.
local dmath = require("gm.draw_math")
local ffi = require("ffi")

local M = {}
local buffers, next_buf = {}, 0
local formats, next_fmt = {}, 0
local fmt_building

function M.vertex_format_begin() fmt_building = {} end
local function add_attr(kind) fmt_building = fmt_building or {}; fmt_building[#fmt_building + 1] = kind end
function M.vertex_format_add_position() add_attr("pos") end
function M.vertex_format_add_position_3d() add_attr("pos3") end
function M.vertex_format_add_colour() add_attr("col") end
M.vertex_format_add_color = M.vertex_format_add_colour
function M.vertex_format_add_texcoord() add_attr("uv") end
function M.vertex_format_add_normal() add_attr("normal") end
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

function M.vertex_begin(id, fmt)
  local b = buffers[id]
  b.format = fmt
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

local SIZES = { pos = 8, pos3 = 12, col = 4, uv = 8, normal = 12 }
local function layout(fmt)
  local attrs = formats[fmt] or { "pos", "col" }
  local size = 0
  for _, a in ipairs(attrs) do size = size + SIZES[a] end
  return attrs, size
end

function M.serialize(id)
  local b = buffers[id]
  local attrs, size = layout(b.format)
  local n = #b.verts
  local out = ffi.new("uint8_t[?]", math.max(1, n * size))
  for i, v in ipairs(b.verts) do
    local p = out + (i - 1) * size
    for _, a in ipairs(attrs) do
      if a == "pos" or a == "pos3" then
        local f = ffi.cast("float*", p)
        f[0], f[1] = v[1], v[2]
        if a == "pos3" then f[2] = 0 end
      elseif a == "col" then
        for k = 0, 3 do p[k] = math.floor(math.max(0, math.min(1, v[5 + k])) * 255 + 0.5) end
      elseif a == "uv" then
        local f = ffi.cast("float*", p)
        f[0], f[1] = v[3], v[4]
      end
      p = p + SIZES[a]
    end
  end
  return ffi.string(out, n * size)
end

function M.restore_bytes(id, bytes, fmt)
  local b = buffers[id]
  b.format = fmt
  local attrs, size = layout(fmt)
  local data = ffi.cast("const uint8_t*", bytes)
  b.verts = {}
  for i = 0, math.floor(#bytes / size) - 1 do
    local p = data + i * size
    local v = { 0, 0, 0, 0, 1, 1, 1, 1 }
    for _, a in ipairs(attrs) do
      if a == "pos" or a == "pos3" then
        local f = ffi.cast("const float*", p)
        v[1], v[2] = f[0], f[1]
      elseif a == "col" then
        for k = 0, 3 do v[5 + k] = p[k] / 255 end
      elseif a == "uv" then
        local f = ffi.cast("const float*", p)
        v[3], v[4] = f[0], f[1]
      end
      p = p + SIZES[a]
    end
    b.verts[i + 1] = v
  end
  b.frozen = true
end

function M.vertex_at(id, i) return buffers[id].verts[i + 1] end

return M
