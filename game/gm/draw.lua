-- GameMaker drawing API on love.graphics: sprites, text, shapes, primitives,
-- surfaces, blend modes, shaders and the world matrix. Colours are BGR ints.
local assets = require("gm.assets")
local dmath = require("gm.draw_math")
local text_layout = require("gm.text_layout")
local shader_conv = require("gm.shader_conv")
local value = require("gm.value")

local M = {}
for k, v in pairs(dmath) do M[k] = v end

local lg = love.graphics
local state = { colour = 0xFFFFFF, alpha = 1, font = -1, halign = 0, valign = 0 }

M.c_white, M.c_black, M.c_red, M.c_lime, M.c_blue = 0xFFFFFF, 0x000000, 0x0000FF, 0x00FF00, 0xFF0000
M.c_yellow, M.c_gray, M.c_grey, M.c_dkgray, M.c_ltgray = 0x00FFFF, 0x808080, 0x808080, 0x404040, 0xC0C0C0
M.c_orange, M.c_aqua, M.c_fuchsia = 0x40A0FF, 0xFFFF00, 0xFF00FF
M.fa_left, M.fa_center, M.fa_right, M.fa_top, M.fa_middle, M.fa_bottom = 0, 1, 2, 0, 1, 2

local function set_rgba(c, a)
  local r, g, b = dmath.bgr_to_rgb(c)
  lg.setColor(r, g, b, a)
end
local function apply() set_rgba(state.colour, state.alpha) end
M.apply_colour = apply

function M.draw_set_colour(c) state.colour = c end
M.draw_set_color = M.draw_set_colour
function M.draw_get_colour() return state.colour end
M.draw_get_color = M.draw_get_colour
function M.draw_set_alpha(a) state.alpha = a end
function M.draw_get_alpha() return state.alpha end
function M.draw_set_halign(h) state.halign = h end
function M.draw_set_valign(v) state.valign = v end
function M.draw_get_halign() return state.halign end
function M.draw_get_valign() return state.valign end

function M.draw_clear(c) local r, g, b = dmath.bgr_to_rgb(c); lg.clear(r, g, b, 1) end
function M.draw_clear_alpha(c, a) local r, g, b = dmath.bgr_to_rgb(c); lg.clear(r, g, b, a) end

-- Sprites --------------------------------------------------------------------
function M.sprite_exists(id) return assets.sprites[id] ~= nil end
function M.sprite_get_name(id) return assets.sprite(id).name end
function M.sprite_get_number(id) return #assets.sprite(id).frames end
function M.sprite_get_width(id) return assets.sprite(id).w end
function M.sprite_get_height(id) return assets.sprite(id).h end
function M.sprite_get_xoffset(id) return assets.sprite(id).origin[1] end
function M.sprite_get_yoffset(id) return assets.sprite(id).origin[2] end
function M.sprite_get_bbox_left(id) return assets.sprite(id).bbox[1] end
function M.sprite_get_bbox_top(id) return assets.sprite(id).bbox[2] end
function M.sprite_get_bbox_right(id) return assets.sprite(id).bbox[3] end
function M.sprite_get_bbox_bottom(id) return assets.sprite(id).bbox[4] end

local function frame_of(spr, sub)
  local s = assets.sprite(spr)
  local i = dmath.frame_index(#s.frames, sub)
  return s, s.frames[i + 1], i
end

function M.draw_sprite_ext(spr, sub, x, y, xs, ys, rot, col, alpha)
  local s, f, i = frame_of(spr, sub)
  local ox, oy = dmath.frame_offset(s, i, 1, 1)
  set_rgba(col, alpha)
  lg.draw(f.image, f.quad, x, y, -math.rad(rot), xs, ys, -ox, -oy)
end

function M.draw_sprite(spr, sub, x, y)
  M.draw_sprite_ext(spr, sub, x, y, 1, 1, 0, 0xFFFFFF, state.alpha)
end

-- Part of the (untrimmed) sprite image; origin is ignored, like in GameMaker.
local function part_quad(f, left, top, w, h)
  local fx, fy, fw, fh = f.dst[1], f.dst[2], f.src[3], f.src[4]
  local x1, y1 = math.max(left, fx), math.max(top, fy)
  local x2, y2 = math.min(left + w, fx + fw), math.min(top + h, fy + fh)
  if x2 <= x1 or y2 <= y1 then return nil end
  local q = lg.newQuad(f.src[1] + x1 - fx, f.src[2] + y1 - fy, x2 - x1, y2 - y1, f.image:getDimensions())
  return q, x1 - left, y1 - top
end

function M.draw_sprite_part_ext(spr, sub, left, top, w, h, x, y, xs, ys, col, alpha)
  local _, f = frame_of(spr, sub)
  local q, dx, dy = part_quad(f, left, top, w, h)
  if not q then return end
  set_rgba(col, alpha)
  lg.draw(f.image, q, x + dx * xs, y + dy * ys, 0, xs, ys)
end

function M.draw_sprite_part(spr, sub, left, top, w, h, x, y)
  M.draw_sprite_part_ext(spr, sub, left, top, w, h, x, y, 1, 1, 0xFFFFFF, state.alpha)
end

local general_mesh
function M.draw_sprite_general(spr, sub, left, top, w, h, x, y, xs, ys, rot, c1, c2, c3, c4, alpha)
  local _, f = frame_of(spr, sub)
  local q, dx, dy = part_quad(f, left, top, w, h)
  if not q then return end
  local qx, qy, qw, qh = q:getViewport()
  local iw, ih = f.image:getDimensions()
  local u1, v1, u2, v2 = qx / iw, qy / ih, (qx + qw) / iw, (qy + qh) / ih
  local function col(c) local r, g, b = dmath.bgr_to_rgb(c); return r, g, b, alpha end
  local r1, g1, b1 = col(c1)
  local r2, g2, b2 = col(c2)
  local r3, g3, b3 = col(c3)
  local r4, g4, b4 = col(c4)
  local verts = {
    { dx, dy, u1, v1, r1, g1, b1, alpha }, { dx + qw, dy, u2, v1, r2, g2, b2, alpha },
    { dx + qw, dy + qh, u2, v2, r3, g3, b3, alpha }, { dx, dy + qh, u1, v2, r4, g4, b4, alpha },
  }
  if not general_mesh then general_mesh = lg.newMesh(4, "fan", "stream") end
  general_mesh:setVertices(verts)
  general_mesh:setTexture(f.image)
  lg.setColor(1, 1, 1, 1)
  lg.draw(general_mesh, x, y, -math.rad(rot), xs, ys)
end

function M.draw_self(inst)
  if inst.sprite_index == nil or inst.sprite_index < 0 then return end
  M.draw_sprite_ext(inst.sprite_index, inst.image_index, inst.x, inst.y, inst.image_xscale, inst.image_yscale,
    inst.image_angle, inst.image_blend, inst.image_alpha)
end

-- Surfaces -------------------------------------------------------------------
local surfaces, next_surface = {}, 1
local target_stack = {}
M.surfaces = surfaces

function M.surface_create(w, h)
  local id = next_surface
  next_surface = next_surface + 1
  surfaces[id] = lg.newCanvas(math.max(1, math.floor(w)), math.max(1, math.floor(h)))
  return id
end
function M.surface_exists(id) return type(id) == "number" and surfaces[id] ~= nil end
function M.surface_free(id) if surfaces[id] then surfaces[id]:release(); surfaces[id] = nil end end
function M.surface_get_width(id) return surfaces[id]:getWidth() end
function M.surface_get_height(id) return surfaces[id]:getHeight() end
function M.surface_get_canvas(id) return surfaces[id] end
function M.surface_resize(id, w, h)
  local c = surfaces[id]
  w, h = math.max(1, math.floor(w)), math.max(1, math.floor(h))
  if c and c:getWidth() == w and c:getHeight() == h then return end
  if c then c:release() end
  surfaces[id] = lg.newCanvas(w, h)
end

function M.surface_set_target(id)
  local c = surfaces[id]
  if not c then return false end
  target_stack[#target_stack + 1] = { lg.getCanvas(), M.world_base }
  lg.push("all")
  lg.setCanvas({ c, stencil = true })
  lg.origin()
  M.world_base = love.math.newTransform()
  apply()
  return true
end

function M.surface_reset_target()
  local top = table.remove(target_stack)
  if not top then return false end
  lg.pop()
  M.world_base = top[2]
  return true
end

local function draw_canvas(c, x, y, sx, sy, rot, col, alpha)
  set_rgba(col or 0xFFFFFF, alpha or state.alpha)
  lg.draw(c, x, y, -math.rad(rot or 0), sx, sy)
end
function M.draw_surface(id, x, y) draw_canvas(surfaces[id], x, y, 1, 1) end
function M.draw_surface_ext(id, x, y, xs, ys, rot, col, alpha) draw_canvas(surfaces[id], x, y, xs, ys, rot, col, alpha) end
function M.draw_surface_stretched(id, x, y, w, h)
  local c = surfaces[id]
  draw_canvas(c, x, y, w / c:getWidth(), h / c:getHeight())
end
function M.draw_surface_stretched_ext(id, x, y, w, h, col, alpha)
  local c = surfaces[id]
  draw_canvas(c, x, y, w / c:getWidth(), h / c:getHeight(), 0, col, alpha)
end

local function canvas_region_image(c, x, y, w, h)
  local data = c:newImageData(nil, nil, x, y, w, h)
  local img = lg.newImage(data)
  return img, data
end

function M.sprite_create_from_surface(id, x, y, w, h, removeback, smooth, xorig, yorig)
  local img = canvas_region_image(surfaces[id], x, y, w, h)
  local fr = { image = img, quad = lg.newQuad(0, 0, w, h, w, h), src = { 0, 0, w, h }, dst = { 0, 0 }, bound = { w, h } }
  return assets.add_sprite({ name = "__dynamic", w = w, h = h, origin = { xorig or 0, yorig or 0 }, bbox = { 0, 0, w - 1, h - 1 },
    speed = 0, speed_type = 0, frames = { fr } })
end

function M.sprite_add_from_surface(spr, id, x, y, w, h)
  local s = assets.sprite(spr)
  local img = canvas_region_image(surfaces[id], x, y, w, h)
  s.frames[#s.frames + 1] = { image = img, quad = lg.newQuad(0, 0, w, h, w, h), src = { 0, 0, w, h }, dst = { 0, 0 }, bound = { w, h } }
end

-- Shapes and primitives ------------------------------------------------------------
function M.draw_rectangle(x1, y1, x2, y2, outline)
  apply()
  if outline then lg.rectangle("line", x1, y1, x2 - x1, y2 - y1)
  else lg.rectangle("fill", x1, y1, x2 - x1 + 1, y2 - y1 + 1) end
end

local quad_mesh
function M.draw_rectangle_colour(x1, y1, x2, y2, c1, c2, c3, c4, outline)
  if outline then
    set_rgba(c1, state.alpha)
    lg.rectangle("line", x1, y1, x2 - x1, y2 - y1)
    return
  end
  local function v(x, y, c) local r, g, b = dmath.bgr_to_rgb(c); return { x, y, 0, 0, r, g, b, state.alpha } end
  if not quad_mesh then quad_mesh = lg.newMesh(4, "fan", "stream") end
  quad_mesh:setVertices({ v(x1, y1, c1), v(x2 + 1, y1, c2), v(x2 + 1, y2 + 1, c3), v(x1, y2 + 1, c4) })
  lg.setColor(1, 1, 1, 1)
  lg.draw(quad_mesh)
end
M.draw_rectangle_color = M.draw_rectangle_colour

function M.draw_roundrect_ext(x1, y1, x2, y2, rx, ry, outline)
  apply()
  lg.rectangle(outline and "line" or "fill", x1, y1, x2 - x1, y2 - y1, rx / 2, ry / 2)
end

function M.draw_circle(x, y, r, outline) apply(); lg.circle(outline and "line" or "fill", x, y, r) end
-- Gradient circle: c1 at the centre, c2 at the edge.
function M.draw_circle_colour(x, y, r, c1, c2, outline)
  local r1, g1, b1 = dmath.bgr_to_rgb(c1)
  local r2, g2, b2 = dmath.bgr_to_rgb(c2)
  if outline then
    lg.setColor(r2, g2, b2, state.alpha)
    lg.circle("line", x, y, r)
    return
  end
  local seg = math.max(16, math.min(96, math.floor(r)))
  local verts = { { x, y, 0, 0, r1, g1, b1, state.alpha } }
  for i = 0, seg do
    local a = i / seg * math.pi * 2
    verts[#verts + 1] = { x + math.cos(a) * r, y + math.sin(a) * r, 0, 0, r2, g2, b2, state.alpha }
  end
  local mesh = lg.newMesh(verts, "fan", "stream")
  lg.setColor(1, 1, 1, 1)
  lg.draw(mesh)
  mesh:release()
end
M.draw_circle_color = M.draw_circle_colour

function M.draw_ellipse(x1, y1, x2, y2, outline)
  apply()
  lg.ellipse(outline and "line" or "fill", (x1 + x2) / 2, (y1 + y2) / 2, math.abs(x2 - x1) / 2, math.abs(y2 - y1) / 2)
end
function M.draw_triangle(x1, y1, x2, y2, x3, y3, outline) apply(); lg.polygon(outline and "line" or "fill", x1, y1, x2, y2, x3, y3) end

function M.draw_line(x1, y1, x2, y2) apply(); lg.setLineWidth(1); lg.line(x1, y1, x2, y2) end
function M.draw_line_width(x1, y1, x2, y2, w) apply(); lg.setLineWidth(w); lg.line(x1, y1, x2, y2); lg.setLineWidth(1) end

local line_mesh
local function gradient_line(x1, y1, x2, y2, w, c1, c2)
  local dx, dy = x2 - x1, y2 - y1
  local len = math.sqrt(dx * dx + dy * dy)
  if len == 0 then return end
  local nx, ny = -dy / len * w / 2, dx / len * w / 2
  local r1, g1, b1 = dmath.bgr_to_rgb(c1)
  local r2, g2, b2 = dmath.bgr_to_rgb(c2)
  if not line_mesh then line_mesh = lg.newMesh(4, "fan", "stream") end
  line_mesh:setVertices({
    { x1 + nx, y1 + ny, 0, 0, r1, g1, b1, state.alpha }, { x2 + nx, y2 + ny, 0, 0, r2, g2, b2, state.alpha },
    { x2 - nx, y2 - ny, 0, 0, r2, g2, b2, state.alpha }, { x1 - nx, y1 - ny, 0, 0, r1, g1, b1, state.alpha },
  })
  lg.setColor(1, 1, 1, 1)
  lg.draw(line_mesh)
end
function M.draw_line_colour(x1, y1, x2, y2, c1, c2) gradient_line(x1, y1, x2, y2, 1, c1, c2) end
M.draw_line_color = M.draw_line_colour
function M.draw_line_width_colour(x1, y1, x2, y2, w, c1, c2) gradient_line(x1, y1, x2, y2, w, c1, c2) end
M.draw_line_width_color = M.draw_line_width_colour

M.pr_pointlist, M.pr_linelist, M.pr_linestrip, M.pr_trianglelist, M.pr_trianglestrip, M.pr_trianglefan = 1, 2, 3, 4, 5, 6
local prim
function M.draw_primitive_begin(kind) prim = { kind = kind, verts = {} } end
function M.draw_primitive_begin_texture(kind, tex) prim = { kind = kind, verts = {}, texture = tex } end
function M.draw_vertex(x, y)
  local r, g, b = dmath.bgr_to_rgb(state.colour)
  prim.verts[#prim.verts + 1] = { x, y, 0, 0, r, g, b, state.alpha }
end
function M.draw_vertex_colour(x, y, c, a)
  local r, g, b = dmath.bgr_to_rgb(c)
  prim.verts[#prim.verts + 1] = { x, y, 0, 0, r, g, b, a }
end
M.draw_vertex_color = M.draw_vertex_colour
function M.draw_vertex_texture(x, y, u, v)
  local r, g, b = dmath.bgr_to_rgb(state.colour)
  prim.verts[#prim.verts + 1] = { x, y, u, v, r, g, b, state.alpha }
end
function M.draw_vertex_texture_colour(x, y, u, v, c, a)
  local r, g, b = dmath.bgr_to_rgb(c)
  prim.verts[#prim.verts + 1] = { x, y, u, v, r, g, b, a }
end
M.draw_vertex_texture_color = M.draw_vertex_texture_colour

function M.draw_primitive_end()
  local p = prim
  prim = nil
  local n = #p.verts
  if n == 0 then return end
  lg.setColor(1, 1, 1, 1)
  if p.kind == M.pr_pointlist then
    for _, v in ipairs(p.verts) do lg.setColor(v[5], v[6], v[7], v[8]); lg.points(v[1], v[2]) end
  elseif p.kind == M.pr_linelist or p.kind == M.pr_linestrip then
    local step = p.kind == M.pr_linelist and 2 or 1
    for i = 1, n - 1, step do
      local a, b = p.verts[i], p.verts[i + 1]
      lg.setColor(a[5], a[6], a[7], a[8])
      lg.line(a[1], a[2], b[1], b[2])
    end
  else
    if n < 3 then return end
    local mode = p.kind == M.pr_trianglestrip and "strip" or p.kind == M.pr_trianglefan and "fan" or "triangles"
    local mesh = lg.newMesh(p.verts, mode, "stream")
    if p.texture then mesh:setTexture(p.texture) end
    lg.draw(mesh)
    mesh:release()
  end
end

-- Blend modes --------------------------------------------------------------------
M.bm_normal, M.bm_add, M.bm_max, M.bm_subtract = 0, 1, 2, 3
M.bm_zero, M.bm_one, M.bm_src_colour, M.bm_inv_src_colour, M.bm_src_alpha, M.bm_inv_src_alpha = 1, 2, 3, 4, 5, 6
M.bm_dest_alpha, M.bm_inv_dest_alpha, M.bm_dest_colour, M.bm_inv_dest_colour, M.bm_src_alpha_sat = 7, 8, 9, 10, 11
M.bm_src_color, M.bm_inv_src_color, M.bm_dest_color, M.bm_inv_dest_color = 3, 4, 9, 10

function M.gpu_set_blendmode(mode)
  if mode == M.bm_add then lg.setBlendMode("add", "alphamultiply")
  elseif mode == M.bm_subtract then lg.setBlendMode("subtract", "alphamultiply")
  elseif mode == M.bm_max then lg.setBlendMode("lighten", "premultiplied")
  else lg.setBlendMode("alpha", "alphamultiply") end
end

local warned = {}
-- LÖVE 11 has no arbitrary blend factors; map the combinations GameMaker games use.
function M.gpu_set_blendmode_ext(src, dst)
  local key = src .. ":" .. dst
  if key == "5:6" then lg.setBlendMode("alpha", "alphamultiply")
  elseif key == "2:6" then lg.setBlendMode("alpha", "premultiplied")
  elseif key == "5:2" then lg.setBlendMode("add", "alphamultiply")
  elseif key == "2:2" then lg.setBlendMode("add", "premultiplied")
  elseif key == "9:1" or key == "1:3" then lg.setBlendMode("multiply", "premultiplied")
  elseif key == "10:2" or key == "2:4" then lg.setBlendMode("screen", "premultiplied")
  elseif key == "2:1" then lg.setBlendMode("replace", "premultiplied")
  elseif key == "1:6" or key == "1:4" then lg.setBlendMode("multiply", "premultiplied")
  else
    if not warned[key] then print("gm: unsupported blend factors " .. key .. ", using alpha"); warned[key] = true end
    lg.setBlendMode("alpha", "alphamultiply")
  end
end
function M.gpu_set_blendmode_ext_sepalpha(src, dst) M.gpu_set_blendmode_ext(src, dst) end

function M.gpu_set_tex_filter(on)
  local mode = on and "linear" or "nearest"
  for _, page in ipairs(assets.pages) do page:setFilter(mode, mode) end
  love.graphics.setDefaultFilter(mode, mode)
end
function M.gpu_get_tex_filter() return true end

-- Shaders ------------------------------------------------------------------------
local compiled = {}
local function shader_obj(id)
  local s = compiled[id]
  if s == nil then
    local def = assets.shaders[id]
    local ok, res = pcall(lg.newShader, shader_conv.fragment(def.fsh))
    if not ok then print("gm: shader " .. def.name .. " failed: " .. tostring(res)); res = false end
    compiled[id] = res
    s = res
  end
  return s or nil
end
local current_shader
function M.shader_set(id) current_shader = shader_obj(id); lg.setShader(current_shader) end
function M.shader_reset() current_shader = nil; lg.setShader() end
function M.shader_get_uniform(id, name) return { shader = id, name = name } end
function M.shader_get_sampler_index(id, name) return { shader = id, name = name } end
local function send(u, ...)
  local s = shader_obj(u.shader)
  if s and s:hasUniform(u.name) then s:send(u.name, ...) end
end
function M.shader_set_uniform_f(u, a, b, c, d)
  if d then send(u, { a, b, c, d }) elseif c then send(u, { a, b, c }) elseif b then send(u, { a, b }) else send(u, a) end
end
M.shader_set_uniform_i = M.shader_set_uniform_f
function M.shader_set_uniform_f_array(u, arr)
  local t = {}
  for i = 0, value.array_length(arr) - 1 do t[#t + 1] = arr[i] end
  send(u, unpack(t))
end
function M.texture_set_stage(u, tex) send(u, tex) end
function M.surface_get_texture(id) return surfaces[id] end
function M.sprite_get_texture(spr, sub) local _, f = frame_of(spr, sub); return f.image end

-- Matrices (matrix_world only affects drawing) -------------------------------------
M.matrix_view, M.matrix_projection, M.matrix_world = 0, 1, 2
M.world_base = love.math.newTransform()
local world = value.array(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1)

function M.matrix_build_identity() return value.array(1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1) end

function M.matrix_build(x, y, z, xr, yr, zr, xs, ys, zs)
  local cz, sz = math.cos(math.rad(-zr)), math.sin(math.rad(-zr))
  -- 2D games only use z rotation; x/y rotation is ignored.
  return value.array(cz * xs, sz * xs, 0, 0, -sz * ys, cz * ys, 0, 0, 0, 0, zs, 0, x, y, z, 1)
end

function M.matrix_get(kind)
  if kind == M.matrix_world then
    local m = value.array()
    for i = 0, 15 do m[i] = world[i] end
    return m
  end
  return M.matrix_build_identity()
end

function M.matrix_set(kind, m)
  if kind ~= M.matrix_world then return end
  for i = 0, 15 do world[i] = m[i] end
  local t = love.math.newTransform()
  t:setMatrix("column", m[0], m[1], m[2], m[3], m[4], m[5], m[6], m[7], m[8], m[9], m[10], m[11], m[12], m[13], m[14], m[15])
  lg.replaceTransform(M.world_base * t)
end

-- Text ---------------------------------------------------------------------------
local ttf, next_font = {}, 1000
local gm_batches = {}

local function font_def(id)
  if id == nil or id < 0 then id = assets.font_by_name.fnt_gm_20 or 0 end
  return ttf[id] or assets.fonts[id]
end

function M.draw_set_font(id) state.font = id end
function M.draw_get_font() return state.font end
function M.font_exists(id) return ttf[id] ~= nil or assets.fonts[id] ~= nil end
function M.font_get_size(id) return font_def(id).size end

function M.font_add(path, size, bold, italic, first, last)
  local p = path
  if not love.filesystem.getInfo(p) then p = "assets/" .. path end
  local ok, f = pcall(lg.newFont, p, size)
  if not ok then print("gm: font_add failed for " .. tostring(path)); return -1 end
  f:setFilter("linear", "linear")
  local id = next_font
  next_font = next_font + 1
  ttf[id] = { kind = "ttf", font = f, size = size, name = path }
  return id
end
function M.font_delete(id) if ttf[id] then ttf[id].font:release(); ttf[id] = nil end end

local utf8_cp

local function glyph_width(f, s)
  if f.kind == "ttf" then return f.font:getWidth(s) end
  local w = 0
  for _, ch in ipairs(value.utf8_chars(s)) do
    local g = f.glyphs[tostring(utf8_cp(ch))]
    if g then w = w + g.shift end
  end
  return w
end

utf8_cp = function(ch)
  local b1 = ch:byte(1)
  if b1 < 0x80 then return b1 end
  if b1 < 0xE0 then return (b1 - 0xC0) * 64 + ch:byte(2) - 0x80 end
  if b1 < 0xF0 then return ((b1 - 0xE0) * 64 + ch:byte(2) - 0x80) * 64 + ch:byte(3) - 0x80 end
  return (((b1 - 0xF0) * 64 + ch:byte(2) - 0x80) * 64 + ch:byte(3) - 0x80) * 64 + ch:byte(4) - 0x80
end

local function line_height(f)
  if f.kind == "ttf" then return f.font:getHeight() end
  if f.line_height and f.line_height > 0 then return f.line_height end
  local h = 0
  for _, g in pairs(f.glyphs) do if g.h > h then h = g.h end end
  f.line_height = h
  return h
end

local function batch_for(f)
  local b = gm_batches[f.index]
  if not b then
    b = lg.newSpriteBatch(f.frame.image, 256, "stream")
    gm_batches[f.index] = b
  end
  return b
end

local function draw_line_gm(f, s, x, y)
  local b = batch_for(f)
  local fx, fy = f.frame.src[1], f.frame.src[2]
  local iw, ih = f.frame.image:getDimensions()
  for _, ch in ipairs(value.utf8_chars(s)) do
    local g = f.glyphs[tostring(utf8_cp(ch))]
    if g then
      if g.w > 0 and g.h > 0 then
        b:add(lg.newQuad(fx + g.x, fy + g.y, g.w, g.h, iw, ih), x + g.offset, y)
      end
      x = x + g.shift
    end
  end
end

-- Draws text lines at (0,0) in the current transform with GameMaker alignment.
local function draw_lines(s, sep, max_w)
  local f = font_def(state.font)
  local lines = text_layout.wrap(s, max_w or -1, function(str) return glyph_width(f, str) end)
  local lh = (sep and sep >= 0) and sep or line_height(f)
  local total_h = lh * #lines
  local y0 = text_layout.align_y(state.valign, total_h)
  apply()
  if f.kind ~= "ttf" then batch_for(f):clear() end
  for i, line in ipairs(lines) do
    local x0 = text_layout.align_x(state.halign, glyph_width(f, line))
    local y = y0 + (i - 1) * lh
    if f.kind == "ttf" then
      lg.setFont(f.font)
      lg.print(line, x0, y)
    else
      draw_line_gm(f, line, x0, y)
    end
  end
  if f.kind ~= "ttf" then lg.draw(batch_for(f)) end
end

local function with_transform(x, y, xs, ys, rot, fn)
  lg.push()
  lg.translate(x, y)
  lg.rotate(-math.rad(rot or 0))
  lg.scale(xs or 1, ys or 1)
  fn()
  lg.pop()
end

function M.draw_text(x, y, s)
  s = value.string(s)
  with_transform(x, y, 1, 1, 0, function() draw_lines(s) end)
end
function M.draw_text_ext(x, y, s, sep, w)
  s = value.string(s)
  with_transform(x, y, 1, 1, 0, function() draw_lines(s, sep, w) end)
end
function M.draw_text_transformed(x, y, s, xs, ys, rot)
  s = value.string(s)
  with_transform(x, y, xs, ys, rot, function() draw_lines(s) end)
end
function M.draw_text_ext_transformed(x, y, s, sep, w, xs, ys, rot)
  s = value.string(s)
  with_transform(x, y, xs, ys, rot, function() draw_lines(s, sep, w) end)
end

function M.string_width(s)
  local f = font_def(state.font)
  local w = 0
  for _, line in ipairs(text_layout.wrap(value.string(s), -1, function() return 0 end)) do
    w = math.max(w, glyph_width(f, line))
  end
  return w
end
function M.string_height(s)
  local f = font_def(state.font)
  return line_height(f) * #text_layout.wrap(value.string(s), -1, function() return 0 end)
end
function M.string_width_ext(s, sep, w)
  local f = font_def(state.font)
  local mw = 0
  for _, line in ipairs(text_layout.wrap(value.string(s), w, function(str) return glyph_width(f, str) end)) do
    mw = math.max(mw, glyph_width(f, line))
  end
  return mw
end
function M.string_height_ext(s, sep, w)
  local f = font_def(state.font)
  local lh = (sep and sep >= 0) and sep or line_height(f)
  return lh * #text_layout.wrap(value.string(s), w, function(str) return glyph_width(f, str) end)
end

return M
