local gm = require("gm")

-- gml_Script_anon__62655__gml_Object_obj_mycelium_controller_Create_0 @ 0x140693930
function anon__62655__gml_Object_obj_mycelium_controller_Create_0(self, other, col, vx1, vy1, vx2, vy2)
  local i = 0
  while i < gm.array_length(self.myc_unbaked) do
    local e = self.myc_unbaked[i]
    if e.vbuf == -1 then
      if not (e.bx2 < vx1 or e.bx1 > vx2 or e.by2 < vy1 or e.by1 > vy2) then
        self.myc_growth_bake(self, other, e)
      end
    end
    i = i + 1
  end
  local t = 0
  while t < gm.array_length(self.myc_tile_list) do
    local tl = self.myc_tile_list[t]
    if not (tl.bx2 < vx1 or tl.bx1 > vx2 or tl.by2 < vy1 or tl.by1 > vy2) then
      local closed = tl.closed
      local k = 0
      while k < gm.array_length(closed) do
        gm.vertex_submit(closed[k], gm.pr_trianglelist, -1)
        k = k + 1
      end
      if tl.vbuf >= 0 then
        gm.vertex_submit(tl.vbuf, gm.pr_trianglelist, -1)
      end
      local pending = tl.pending
      local j = 0
      while j < gm.array_length(pending) do
        local p = pending[j]
        if p.vbuf >= 0 then
          if not (p.bx2 < vx1 or p.bx1 > vx2 or p.by2 < vy1 or p.by1 > vy2) then
            gm.vertex_submit(p.vbuf, gm.pr_trianglelist, -1)
          end
        end
        j = j + 1
      end
    end
    t = t + 1
  end
  local nv = 0
  local lim = 600
  gm.draw_primitive_begin(gm.pr_trianglelist)
  local a = 0
  while a < gm.array_length(self.myc_active) do
    local e = self.myc_active[a]
    if not gm.bool(e.finished) then
      local branches = e.branches
      local b = 0
      while b < gm.array_length(branches) do
        local br = branches[b]
        if not (br.bx2 < vx1 or br.bx1 > vx2 or br.by2 < vy1 or br.by1 > vy2) then
          local cells = br.cells
          local n = gm.array_length(cells)
          if n ~= 0 then
            local p = gm.clamp((e.age - br.start) / e.duration, 0, 1)
            if p > 0 then
              local shown = gm.min(gm.ceil(p * n), n)
              local nfull = gm.div(shown, 96)
              if nfull > 0 then
                if gm.is_undefined(gm.struct_get(br, "chunks")) then
                  br.chunks = gm.array()
                end
                local ch = br.chunks
                while gm.array_length(ch) < nfull do
                  local ci = gm.array_length(ch)
                  gm.array_push(ch, self.myc_cells_buffer(self, other, cells, ci * 96, (ci + 1) * 96))
                end
                gm.draw_primitive_end()
                local q = 0
                while q < nfull do
                  if ch[q] >= 0 then
                    gm.vertex_submit(ch[q], gm.pr_trianglelist, -1)
                  end
                  q = q + 1
                end
                gm.draw_primitive_begin(gm.pr_trianglelist)
                nv = 0
              end
              local c_i = nfull * 96
              while c_i < shown do
                local c = cells[c_i]
                local hw = gm.div(c.w, 2)
                local x0 = c.x - hw
                local y0 = c.y - hw
                local x1 = x0 + c.w
                local y1 = y0 + c.w
                gm.draw_vertex_color(x0, y0, col, 1.0)
                gm.draw_vertex_color(x1, y0, col, 1.0)
                gm.draw_vertex_color(x0, y1, col, 1.0)
                gm.draw_vertex_color(x1, y0, col, 1.0)
                gm.draw_vertex_color(x1, y1, col, 1.0)
                gm.draw_vertex_color(x0, y1, col, 1.0)
                nv = nv + 6
                if nv >= lim then
                  gm.draw_primitive_end()
                  gm.draw_primitive_begin(gm.pr_trianglelist)
                  nv = 0
                end
                c_i = c_i + 1
              end
            end
          end
        end
        b = b + 1
      end
    end
    a = a + 1
  end
  gm.draw_primitive_end()
  leaf_tendrils_draw(self, other, col)
end

-- gml_Script____struct___510__anon__67930__gml_Object_obj_mycelium_controller_Create_0 @ 0x140697d70
function ___struct___510__anon__67930__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, org_x, org_y, step, w, h)
  self.x = x
  self.y = y
  self.org_x = org_x
  self.org_y = org_y
  self.step = step
  self.w = w
  self.h = h
end

-- gml_Script_anon__67930__gml_Object_obj_mycelium_controller_Create_0 @ 0x140698150
function anon__67930__gml_Object_obj_mycelium_controller_Create_0(self, other, cx, cy, vx1, vy1, vx2, vy2)
  local ds = gm.max(1, vis(self, other, "myc_glow_downscale"))
  local w = gm.max(1, gm.ceil(self.myc_view_w / ds)) + 4
  local h = gm.max(1, gm.ceil(self.myc_view_h / ds)) + 4
  local ox = gm.floor(cx / ds) * ds - ds * 2
  local oy = gm.floor(cy / ds) * ds - ds * 2
  local old = gm.matrix_get(gm.matrix_world)
  self.myc_glow_seed = surface_fit(self, other, self.myc_glow_seed, w, h)
  self.myc_glow_blur = surface_fit(self, other, self.myc_glow_blur, w, h)
  if not (gm.surface_exists(self.myc_glow_seed) and gm.surface_exists(self.myc_glow_blur)) then
    self.myc_glow_at = gm.undefined
    return
  end
  gm.surface_set_target(self.myc_glow_seed)
  gm.draw_clear_alpha(0, 0)
  gm.matrix_set(gm.matrix_world, gm.matrix_build(-ox / ds, -oy / ds, 0, 0, 0, 0, 1 / ds, 1 / ds, 1))
  self.myc_draw_strands(self, other, 16777215, vx1, vy1, vx2, vy2)
  caves_wrap_draw_visible(self, other, vx1, vy1, vx2, vy2)
  relics_wrap_draw_visible(self, other, vx1, vy1, vx2, vy2)
  gm.matrix_set(gm.matrix_world, gm.matrix_build_identity())
  gm.surface_reset_target()
  gm.gpu_set_tex_filter(true)
  local thick = vis(self, other, "myc_glow_thicken")
  if thick > 0 then
    gm.surface_set_target(self.myc_glow_blur)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_one)
    local d = 0
    while d < 360 do
      gm.draw_surface(self.myc_glow_seed, gm.lengthdir_x(thick, d), gm.lengthdir_y(thick, d))
      d = d + 45
    end
    gm.draw_surface(self.myc_glow_seed, 0, 0)
    gm.surface_reset_target()
    gm.surface_set_target(self.myc_glow_seed)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_zero)
    gm.draw_surface(self.myc_glow_blur, 0, 0)
    gm.surface_reset_target()
  end
  local r = gm.max(0.5, vis(self, other, "myc_glow_radius")) * 0.25
  gm.shader_set(sh_glow_blur)
  gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_zero)
  local pass = 0
  while pass < gm.max(1, vis(self, other, "myc_glow_passes")) do
    gm.surface_set_target(self.myc_glow_blur)
    gm.shader_set_uniform_f(global.glow_blur_step_handle, r / w, 0)
    gm.draw_surface(self.myc_glow_seed, 0, 0)
    gm.surface_reset_target()
    gm.surface_set_target(self.myc_glow_seed)
    gm.shader_set_uniform_f(global.glow_blur_step_handle, 0, r / h)
    gm.draw_surface(self.myc_glow_blur, 0, 0)
    gm.surface_reset_target()
    pass = pass + 1
  end
  gm.shader_reset()
  gm.gpu_set_blendmode(gm.bm_normal)
  gm.gpu_set_tex_filter(false)
  gm.matrix_set(gm.matrix_world, old)
  self.myc_glow_at = gm.new(___struct___510__anon__67930__gml_Object_obj_mycelium_controller_Create_0,
    ox - gm.floor(cx), oy - gm.floor(cy), ox, oy, ds, w, h)
end

-- gml_Script_anon__71233__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069bc10
function anon__71233__gml_Object_obj_mycelium_controller_Create_0(self, other, on)
  self.myc_glow_on = on
  if gm.bool(on) then
    self.myc_pulse_start(self, other)
    self.myc_pulse_timer = vis(self, other, "myc_pulse_interval_seconds") * gm.game_get_speed(gm.gamespeed_fps)
  else
    self.myc_pulses = gm.array()
  end
end

-- gml_Script____struct___511__anon__71572__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069c020
function ___struct___511__anon__71572__gml_Object_obj_mycelium_controller_Create_0(self, other, born, far)
  self.born = born
  self.far = far
end

-- gml_Script_anon__71572__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069c1e0
function anon__71572__gml_Object_obj_mycelium_controller_Create_0(self, other)
  gm.array_push(self.myc_pulses, gm.new(___struct___511__anon__71572__gml_Object_obj_mycelium_controller_Create_0,
    gm.current_time, self.myc_far_max + vis(self, other, "myc_pulse_band_px") * 2))
end

-- gml_Script_anon__71848__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069c6f0
function anon__71848__gml_Object_obj_mycelium_controller_Create_0(self, other, p)
  local d = (gm.current_time - p.born) / 1000 * vis(self, other, "myc_pulse_speed_px")
  if d <= p.far then
    return d
  end
  return -1
end

-- gml_Script_anon__72092__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069cc00
function anon__72092__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, r, w, a)
  local seg = 96
  -- TODO(port): first max() operand is the runtime-initialised constant _DAT_141379be0; 0 assumed
  local rin = gm.max(0, r - w)
  local rout = r + w
  local pass = 0
  while pass < 2 do
    local r1, r2, a1, a2
    if pass == 0 then r1 = rin else r1 = r end
    if pass == 0 then r2 = r else r2 = rout end
    if pass == 0 then a1 = 0 else a1 = a end
    if pass == 0 then a2 = a else a2 = 0 end
    gm.draw_primitive_begin(gm.pr_trianglestrip)
    local i = 0
    while i <= seg do
      local ang = i / seg * 360
      gm.draw_vertex_colour(x + gm.lengthdir_x(r1, ang), y + gm.lengthdir_y(r1, ang), 16777215, a1)
      gm.draw_vertex_colour(x + gm.lengthdir_x(r2, ang), y + gm.lengthdir_y(r2, ang), 16777215, a2)
      i = i + 1
    end
    gm.draw_primitive_end()
    pass = pass + 1
  end
end

-- gml_Script_anon__73025__gml_Object_obj_mycelium_controller_Create_0 @ 0x14069dfe0
function anon__73025__gml_Object_obj_mycelium_controller_Create_0(self, other)
  if gm.is_undefined(self.myc_glow_at) or not gm.surface_exists(self.myc_glow_seed) then
    return
  end
  local g = self.myc_glow_at
  gm.gpu_set_tex_filter(true)
  if gm.bool(self.myc_glow_on) then
    local s = vis(self, other, "myc_glow_strength")
    local col = vis(self, other, "myc_glow_colour")
    gm.gpu_set_blendmode(gm.bm_add)
    while s > 0 do
      gm.draw_surface_ext(self.myc_glow_seed, g.x, g.y, g.step, g.step, 0, col, gm.min(1, s))
      s = s - 1
    end
    gm.gpu_set_blendmode(gm.bm_normal)
  end
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(self.myc_pulses) do
    if self.myc_pulse_radius(self, other, self.myc_pulses[i]) >= 0 then
      gm.array_push(keep, self.myc_pulses[i])
    end
    i = i + 1
  end
  self.myc_pulses = keep
  if gm.array_length(keep) > 0 then
    self.myc_pulse_surface = surface_fit(self, other, self.myc_pulse_surface, g.w, g.h)
    if not gm.surface_exists(self.myc_pulse_surface) then
      gm.gpu_set_tex_filter(false)
      return
    end
    local px = (self.x - g.org_x) / g.step
    local py = (self.y - g.org_y) / g.step
    local band = vis(self, other, "myc_pulse_band_px") / g.step
    gm.surface_set_target(self.myc_pulse_surface)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_one)
    local k = 0
    while k < gm.array_length(keep) do
      local p = keep[k]
      local rad = self.myc_pulse_radius(self, other, p)
      local fade = gm.clamp((p.far - rad) / gm.max(1, vis(self, other, "myc_pulse_band_px") * 2), 0, 1)
      self.myc_pulse_ring(self, other, px, py, rad / g.step, band, fade)
      k = k + 1
    end
    gm.gpu_set_blendmode_ext(gm.bm_dest_alpha, gm.bm_zero)
    gm.draw_surface(self.myc_glow_seed, 0, 0)
    gm.surface_reset_target()
    local s = vis(self, other, "myc_pulse_strength")
    local col = vis(self, other, "myc_pulse_colour")
    gm.gpu_set_blendmode(gm.bm_add)
    while s > 0 do
      gm.draw_surface_ext(self.myc_pulse_surface, g.x, g.y, g.step, g.step, 0, col, gm.min(1, s))
      s = s - 1
    end
    gm.gpu_set_blendmode(gm.bm_normal)
  end
  gm.gpu_set_tex_filter(false)
end

-- gml_Script_anon__77430__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a0cd0
function anon__77430__gml_Object_obj_mycelium_controller_Create_0(self, other, vb, x1, y1, x2, y2)
  gm.vertex_position(vb, x1, y1)
  gm.vertex_color(vb, self.myc_core_color, 1)
  gm.vertex_position(vb, x2, y1)
  gm.vertex_color(vb, self.myc_core_color, 1)
  gm.vertex_position(vb, x1, y2)
  gm.vertex_color(vb, self.myc_core_color, 1)
  gm.vertex_position(vb, x2, y1)
  gm.vertex_color(vb, self.myc_core_color, 1)
  gm.vertex_position(vb, x2, y2)
  gm.vertex_color(vb, self.myc_core_color, 1)
  gm.vertex_position(vb, x1, y2)
  gm.vertex_color(vb, self.myc_core_color, 1)
end

-- gml_Script_anon__78411__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a12b0
function anon__78411__gml_Object_obj_mycelium_controller_Create_0(self, other, vb, cells, horiz)
  local n = gm.array_length(cells)
  if n == 0 then
    return false
  end
  local lo = gm.infinity
  local hi = -gm.infinity
  local i = 0
  while i < n do
    local c = cells[i]
    local p
    if gm.bool(horiz) then p = c.x else p = c.y end
    local s = p - gm.div(c.w, 2)
    lo = gm.min(lo, s)
    hi = gm.max(hi, s + c.w)
    i = i + 1
  end
  local span = hi - lo
  -- TODO(port): array_create fill value is the runtime-initialised constant _DAT_141379be0; 0 assumed
  local amin = gm.array_create(span, 0)
  local amax = gm.array_create(span, -gm.infinity)
  i = 0
  while i < n do
    local c = cells[i]
    local w = c.w
    local hw = gm.div(w, 2)
    local q
    if gm.bool(horiz) then q = c.y else q = c.x end
    local a0 = q - hw
    local a1 = a0 + w
    local p
    if gm.bool(horiz) then p = c.x else p = c.y end
    local k0 = p - hw - lo
    local k = k0
    while k < k0 + w do
      if a0 <= amax[k] and a1 >= amin[k] then
        amin[k] = gm.min(amin[k], a0)
        amax[k] = gm.max(amax[k], a1)
      else
        if amax[k] > -gm.infinity then
          if gm.bool(horiz) then
            self.myc_span_quad(self, other, vb, lo + k, amin[k], lo + k + 1, amax[k])
          else
            self.myc_span_quad(self, other, vb, amin[k], lo + k, amax[k], lo + k + 1)
          end
        end
        amin[k] = a0
        amax[k] = a1
      end
      k = k + 1
    end
    i = i + 1
  end
  local k = 0
  while k < span do
    if amax[k] > -gm.infinity then
      if gm.bool(horiz) then
        self.myc_span_quad(self, other, vb, lo + k, amin[k], lo + k + 1, amax[k])
      else
        self.myc_span_quad(self, other, vb, amin[k], lo + k, amax[k], lo + k + 1)
      end
    end
    k = k + 1
  end
  return true
end

-- gml_Script_anon__80501__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a4300
function anon__80501__gml_Object_obj_mycelium_controller_Create_0(self, other, e)
  if e.vbuf == -1 then
    local vb = gm.vertex_create_buffer()
    local branches = e.branches
    local any = false
    gm.vertex_begin(vb, global.myc_vformat)
    local i = 0
    while i < gm.array_length(branches) do
      local b = branches[i]
      local cells = gm.struct_get(b, "cells")
      if gm.is_undefined(cells) then
        cells = self.myc_rasterize_branch(self, other, b.points, b.width)
      end
      local tall = (b.by2 - b.by1) > (b.bx2 - b.bx1)
      if gm.bool(self.myc_write_branch_spans(self, other, vb, cells, tall)) then
        any = true
      end
      b.cells = gm.undefined
      i = i + 1
    end
    gm.vertex_end(vb)
    if not any then
      gm.vertex_delete_buffer(vb)
      e.vbuf = -2
    else
      e.raw = gm.buffer_create_from_vertex_buffer(vb, gm.buffer_fixed, 1)
      gm.vertex_freeze(vb)
      e.vbuf = vb
      self.myc_tile_add(self, other, e)
    end
  end
end

-- gml_Script____struct___513__anon__81793__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a5640
function ___struct___513__anon__81793__gml_Object_obj_mycelium_controller_Create_0(self, other, raw, vbuf, closed, pending, since, bx1, by1, bx2, by2)
  self.raw = raw
  self.size = 0
  self.vbuf = vbuf
  self.closed = closed
  self.pending = pending
  self.since = since
  self.bx1 = bx1
  self.by1 = by1
  self.bx2 = bx2
  self.by2 = by2
end

-- gml_Script_anon__81793__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a5b40
function anon__81793__gml_Object_obj_mycelium_controller_Create_0(self, other, e)
  local tx = gm.floor((e.bx1 + e.bx2) * 0.5 / 1024)
  local ty = gm.floor((e.by1 + e.by2) * 0.5 / 1024)
  local key = gm.string(tx) .. "," .. gm.string(ty)
  local t = gm.variable_struct_get(self.myc_tiles, key)
  if gm.is_undefined(t) then
    t = gm.new(___struct___513__anon__81793__gml_Object_obj_mycelium_controller_Create_0,
      -1, -1, gm.array(), gm.array(), self.myc_frame, e.bx1, e.by1, e.bx2, e.by2)
    gm.variable_struct_set(self.myc_tiles, key, t)
    gm.array_push(self.myc_tile_list, t)
  else
    t.bx1 = gm.min(t.bx1, e.bx1)
    t.by1 = gm.min(t.by1, e.by1)
    t.bx2 = gm.max(t.bx2, e.bx2)
    t.by2 = gm.max(t.by2, e.by2)
  end
  if gm.array_length(t.pending) == 0 then
    t.since = self.myc_frame
  end
  gm.array_push(t.pending, e)
end

-- gml_Script_anon__82902__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a7670
function anon__82902__gml_Object_obj_mycelium_controller_Create_0(self, other, t)
  local pend = t.pending
  local total = 0
  local i = 0
  while i < gm.array_length(pend) do
    local r = gm.struct_get(pend[i], "raw")
    if gm.is_undefined(r) then r = -1 end
    if r >= 0 then
      total = total + gm.buffer_get_size(r)
    end
    i = i + 1
  end
  if total > 0 then
    if t.size > 0 and t.size + total > 262144 then
      gm.array_push(t.closed, t.vbuf)
      gm.buffer_delete(t.raw)
      t.raw = -1
      t.size = 0
      t.vbuf = -1
    end
    local off = t.size
    if t.raw >= 0 then
      gm.buffer_resize(t.raw, t.size + total)
    else
      t.raw = gm.buffer_create(total, gm.buffer_fixed, 1)
    end
    i = 0
    while i < gm.array_length(pend) do
      local p = pend[i]
      local r = gm.struct_get(p, "raw")
      if gm.is_undefined(r) then r = -1 end
      if r >= 0 then
        local sz = gm.buffer_get_size(r)
        gm.buffer_copy(r, 0, sz, t.raw, off)
        off = off + sz
        gm.buffer_delete(r)
        p.raw = -1
      end
      i = i + 1
    end
    t.size = off
    if t.vbuf >= 0 then
      gm.vertex_delete_buffer(t.vbuf)
    end
    t.vbuf = gm.vertex_create_buffer_from_buffer(t.raw, global.myc_vformat)
    gm.vertex_freeze(t.vbuf)
  end
  i = 0
  while i < gm.array_length(pend) do
    local p = pend[i]
    if p.vbuf >= 0 then
      gm.vertex_delete_buffer(p.vbuf)
    end
    p.vbuf = -3
    i = i + 1
  end
  t.pending = gm.array()
end

-- gml_Script_anon__84786__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a96d0
function anon__84786__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.myc_frame = self.myc_frame + 1
  if gm.array_length(self.myc_unbaked) > 0 then
    local deadline = gm.get_timer() + 1500
    while gm.array_length(self.myc_unbaked) > 0 do
      if gm.get_timer() >= deadline then break end
      self.myc_growth_bake(self, other, gm.array_pop(self.myc_unbaked))
    end
  end
  local i = 0
  while i < gm.array_length(self.myc_tile_list) do
    local t = self.myc_tile_list[i]
    if gm.array_length(t.pending) > 0 and self.myc_frame - t.since >= 30 then
      self.myc_tile_merge(self, other, t)
      break
    end
    i = i + 1
  end
end

-- gml_Script_anon__85453__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406a9f70
function anon__85453__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local i = 0
  while i < gm.array_length(self.myc_tile_list) do
    local t = self.myc_tile_list[i]
    local k = 0
    while k < gm.array_length(t.pending) do
      local p = t.pending[k]
      local r = gm.struct_get(p, "raw")
      if gm.is_undefined(r) then r = -1 end
      if r >= 0 then
        gm.buffer_delete(r)
      end
      if p.vbuf >= 0 then
        gm.vertex_delete_buffer(p.vbuf)
      end
      p.raw = -1
      p.vbuf = -1
      k = k + 1
    end
    if t.vbuf >= 0 then
      gm.vertex_delete_buffer(t.vbuf)
    end
    k = 0
    while k < gm.array_length(t.closed) do
      gm.vertex_delete_buffer(t.closed[k])
      k = k + 1
    end
    if t.raw >= 0 then
      gm.buffer_delete(t.raw)
    end
    i = i + 1
  end
  self.myc_tiles = gm.struct({})
  self.myc_tile_list = gm.array()
end

-- gml_Script_anon__86877__anon__86458__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406ab0d0
function anon__86877__anon__86458__gml_Object_obj_mycelium_controller_Create_0(self, other, a, b)
  return b.bake_d - a.bake_d
end

-- gml_Script_anon__86458__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406ab290
function anon__86458__gml_Object_obj_mycelium_controller_Create_0(self, other, cx, cy)
  self.myc_unbaked = gm.array()
  self.myc_active = gm.array()
  local i = 0
  while i < gm.array_length(self.myc_growths) do
    local g = self.myc_growths[i]
    g.bake_d = gm.point_distance(cx, cy, (g.bx1 + g.bx2) * 0.5, (g.by1 + g.by2) * 0.5)
    gm.array_push(self.myc_unbaked, g)
    i = i + 1
  end
  gm.array_sort(self.myc_unbaked, gm.method(self, anon__86877__anon__86458__gml_Object_obj_mycelium_controller_Create_0))
end

-- gml_Object_obj_mycelium_controller_Step_0 @ 0x1406b1620
function gml_Object_obj_mycelium_controller_Step_0(self, other)
  self.myc_sync_view(self, other)
  if gm.is_struct(global.cave_burst_pending) then
    self.myc_node_burst(self, other, global.cave_burst_pending.x, global.cave_burst_pending.y)
    global.cave_burst_pending = false
  end
  if not gm.bool(ui_blocking(self, other)) then
    local t0 = gm.get_timer()
    self.myc_node_burst_step(self, other)
    self.myc_prof_add(self, other, "burst", gm.get_timer() - t0)
    t0 = gm.get_timer()
    self.myc_digger_step(self, other)
    self.myc_prof_add(self, other, "digger", gm.get_timer() - t0)
  end
  if false and gm.bool(gm.keyboard_check_pressed(113)) then
    self.myc_debug_draw = not gm.bool(self.myc_debug_draw)
  end
  if not gm.bool(perk_has(self, other, "purple")) then
    if gm.bool(self.myc_glow_on) then
      self.myc_glow_set(self, other, false)
    end
  else
    if gm.bool(gm.keyboard_check_pressed(70)) then
      if not gm.instance_exists(obj_hud) or gm.instance_find(obj_hud, 0).hud_slot_naming == -1 then
        self.myc_glow_set(self, other, not gm.bool(self.myc_glow_on))
      end
    end
  end
  if gm.bool(self.myc_glow_on) and not gm.bool(ui_blocking(self, other)) then
    self.myc_pulse_timer = self.myc_pulse_timer - 1
    if self.myc_pulse_timer <= 0 then
      self.myc_pulse_timer = vis(self, other, "myc_pulse_interval_seconds") * gm.game_get_speed(gm.gamespeed_fps)
      self.myc_pulse_start(self, other)
    end
  end
  local blocking = ui_blocking(self, other)
  if gm.bool(blocking) then
    self.myc_clear_prompts(self, other)
    if gm.bool(self.myc_dragging) then
      self.myc_dragging = false
      self.myc_has_aim = false
      self.myc_aim_len = 0
      gm.window_set_cursor(gm.cr_default)
    end
  end
  local gx = gm.device_mouse_x_to_gui(0)
  local gy = gm.device_mouse_y_to_gui(0)
  self.myc_build_hover = -1
  if gm.bool(self.myc_mushroom_prompt) then
    local i = 0
    while i < gm.array_length(self.myc_build_options) do
      local r = self.myc_build_button_rect(self, other, i, self.myc_build_options[i])
      if gm.bool(gm.point_in_rectangle(gx, gy, r.x1, r.y1, r.x2, r.y2)) then
        self.myc_build_hover = i
        break
      end
      i = i + 1
    end
  end
  self.myc_spore_prompt_hover = false
  if gm.bool(self.myc_spore_prompt) then
    local r = self.myc_spore_button_rect(self, other)
    self.myc_spore_prompt_hover = gm.point_in_rectangle(gx, gy, r.x1, r.y1, r.x2, r.y2)
  end
  self.myc_regen_prompt_hover = false
  if gm.bool(self.myc_regen_prompt) then
    local r = self.myc_regen_button_rect(self, other)
    self.myc_regen_prompt_hover = gm.point_in_rectangle(gx, gy, r.x1, r.y1, r.x2, r.y2)
  end
  if not gm.bool(blocking) and not gm.bool(global.ui_capture_mouse) then
    if not gm.bool(global.right_click_used) and gm.bool(gm.mouse_check_button_pressed(gm.mb_right)) then
      if not gm.bool(self.myc_open_build_menu_at(self, other, gm.mouse_x, gm.mouse_y)) then
        self.myc_clear_prompts(self, other)
      end
    end
  end
  if not gm.bool(blocking) and not gm.bool(global.ui_capture_mouse) and gm.bool(gm.mouse_check_button_pressed(gm.mb_left)) then
    local handled = false
    if gm.bool(gm.keyboard_check(gm.vk_alt)) and gm.bool(self.myc_node_quick(self, other, gm.mouse_x, gm.mouse_y)) then
      handled = true
    end
    if gm.bool(self.myc_mushroom_prompt) and self.myc_build_hover >= 0 then
      if gm.bool(self.myc_apply_build_option(self, other, self.myc_build_options[self.myc_build_hover])) then
        self.myc_clear_prompts(self, other)
      end
      handled = true
    end
    if not handled and gm.bool(self.myc_spore_prompt) and gm.bool(self.myc_spore_prompt_hover) then
      if global.food >= bal(self, other, "spore_release_cost") then
        local sx = self.myc_spore_prompt_x
        local sy = self.myc_spore_prompt_y
        local carried = false
        if self.myc_spore_prompt_index >= 0 and self.myc_spore_prompt_index < gm.array_length(global.mushrooms) then
          sx = global.mushrooms[self.myc_spore_prompt_index].x
          sy = global.mushrooms[self.myc_spore_prompt_index].y
          carried = global.mushrooms[self.myc_spore_prompt_index].carried_by ~= -1
        end
        if gm.bool(self.myc_spawn_spores_from(self, other, sx, sy)) and carried then
          tasks_stat_add(self, other, "ach_hedgehog_spores", gm.undefined)
        end
        self.myc_spore_prompt = false
      end
      handled = true
    end
    if not handled and gm.bool(self.myc_regen_prompt) and gm.bool(self.myc_regen_prompt_hover) then
      if self.myc_spore_prompt_index >= 0 and self.myc_spore_prompt_index < gm.array_length(global.mushrooms) then
        local m = global.mushrooms[self.myc_spore_prompt_index]
        if not gm.bool(m.active) and gm.bool(food_spend(self, other, repeller_regen_price(self, other))) then
          m.active = true
          m.charges = repeller_max_charges(self, other)
        end
      end
      self.myc_clear_prompts(self, other)
      handled = true
    end
    if not handled then
      local k = self.myc_get_knot_at(self, other, gm.mouse_x, gm.mouse_y, self.myc_knot_radius + 6)
      if gm.bool(k.hit) then
        self.myc_start_aim_at(self, other, k.x, k.y)
        handled = true
      end
    end
    if not handled then
      local h = self.myc_get_mushroom_at(self, other, gm.mouse_x, gm.mouse_y)
      if gm.bool(h.hit) then
        self.myc_clear_prompts(self, other)
        self.myc_spore_prompt_index = h.index
        self.myc_spore_prompt_x = h.x
        self.myc_spore_prompt_y = h.y
        local mt = 0
        if h.index >= 0 then
          mt = global.mushrooms[h.index].type
        end
        if mt == 2 then
          self.myc_regen_prompt = not gm.bool(global.mushrooms[h.index].active)
        elseif mt == 0 then
          self.myc_spore_prompt = upg_is_owned(self, other, "unlock_spore")
        end
        handled = true
      end
    end
    if not handled then
      self.myc_clear_prompts(self, other)
    end
  end
  if gm.bool(self.myc_dragging) and gm.bool(gm.mouse_check_button(gm.mb_left)) then
    local d = gm.point_distance(self.myc_aim_origin_x, self.myc_aim_origin_y, gm.mouse_x, gm.mouse_y)
    if d > 6 then
      local dir = gm.point_direction(self.myc_aim_origin_x, self.myc_aim_origin_y, gm.mouse_x, gm.mouse_y)
      local pull = gm.min(d * bal(self, other, "shot_pull_scale"), myc_shot_range(self, other))
      self.myc_aim_dir = dir
      self.myc_aim_len = self.myc_measure_allowed_len_from(self, other, self.myc_aim_origin_x, self.myc_aim_origin_y, dir, pull)
      self.myc_aim_reach = self.myc_measure_allowed_len_from(self, other, self.myc_aim_origin_x, self.myc_aim_origin_y, dir, myc_shot_range(self, other))
      local len = self.myc_aim_len
      self.myc_aim_cost = self.myc_shot_cost_for(self, other, self.myc_aim_origin_x, self.myc_aim_origin_y, dir, len)
      self.myc_aim_food = self.myc_shot_food_for(self, other, self.myc_aim_origin_x, self.myc_aim_origin_y, dir, len)
      self.myc_has_aim = true
    end
  end
  if gm.bool(self.myc_dragging) and not gm.bool(gm.mouse_check_button(gm.mb_left)) then
    if gm.bool(self.myc_has_aim) and self.myc_aim_len >= self.myc_min_growth_len then
      self.myc_add_growth_from(self, other, self.myc_aim_origin_x, self.myc_aim_origin_y, self.myc_aim_dir, self.myc_aim_len)
    end
    self.myc_dragging = false
    self.myc_has_aim = false
    self.myc_aim_len = 0
    gm.window_set_cursor(gm.cr_default)
  end
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(self.myc_active) do
    local g = self.myc_active[i]
    local snd = false
    if gm.bool(gm.variable_struct_exists(g, "sound")) then
      snd = not gm.bool(g.sound.done)
    end
    if not gm.bool(g.finished) or snd then
      g.age = g.age + 1
      if snd then
        hyphae_sound_step(self, other, g.sound, g.age)
      end
      gm.array_push(keep, g)
    end
    i = i + 1
  end
  self.myc_active = keep
  local t1 = gm.get_timer()
  self.myc_scan_growths(self, other)
  self.myc_prof_add(self, other, "scan", gm.get_timer() - t1)
  t1 = gm.get_timer()
  self.myc_tiles_step(self, other)
  self.myc_prof_add(self, other, "bake", gm.get_timer() - t1)
  self.myc_update_spores(self, other)
end

-- gml_Object_obj_mycelium_controller_Draw_0 @ 0x1406b6280
function gml_Object_obj_mycelium_controller_Draw_0(self, other)
  if gm.bool(global.intro_active) then
    return
  end
  gm.draw_sprite(originshroom, 0, gm.floor(self.x), gm.floor(self.y))
  local i = 0
  while i < gm.array_length(self.myc_knots) do
    local k = self.myc_knots[i]
    if gm.bool(gm.variable_struct_exists(k, "shroom")) then
      if k.x >= self.myc_camera_x - 64 and k.x <= self.myc_camera_x + self.myc_view_w + 64 then
        local r = self.myc_smallshroom_rect(self, other, k.shroom)
        local s = vis(self, other, "spore_shroom_scale") * shroom_grow_scale(self, other, k)
        -- TODO(port): subimage argument is the runtime-initialised constant _DAT_141379c00; 0 assumed
        gm.draw_sprite_part_ext(r.sprite, 0, r.left, r.top, r.width, r.height,
          gm.floor(k.x) - r.width * s * 0.5, gm.floor(k.y) - r.height * s, s, s, 16777215, 1)
      end
    end
    i = i + 1
  end
end

-- gml_Object_obj_mycelium_controller_Create_0 @ 0x1406abd70
function gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.myc_surface_y = wld(self, other, "surface_height_px")
  self.myc_tile = wld(self, other, "reference_tile_size")
  self.myc_camera = gm.view_camera[0]
  self.myc_camera_x = 0
  self.myc_camera_y = 0
  self.myc_view_w = vis(self, other, "screen_width_px")
  self.myc_view_h = vis(self, other, "screen_height_px")
  self.myc_sync_view = gm.method(self, anon__238__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_sync_view(self, other)
  self.x = global.start_x
  self.y = self.myc_surface_y
  self.image_speed = 0
  self.depth = 0
  self.myc_sample_step = 5
  self.myc_min_growth_len = bal(self, other, "growth_min_length")
  self.myc_growth_duration = vis(self, other, "growth_duration_frames")
  self.myc_default_aim_dir = 270
  self.myc_food_pickup_radius = self.myc_tile * 0.35
  self.myc_dragging = false
  self.myc_has_aim = false
  self.myc_aim_origin_x = self.x
  self.myc_aim_origin_y = self.y + self.myc_tile
  self.myc_aim_dir = self.myc_default_aim_dir
  self.myc_aim_len = 0
  self.myc_aim_reach = 0
  self.myc_aim_cost = 0
  self.myc_aim_food = 0
  self.myc_drag_start_x = 0
  self.myc_drag_start_y = 0
  self.myc_growths = gm.array()
  self.myc_knots = gm.array()
  self.myc_knot_radius = 7
  self.myc_mycelium_pick_radius = 12
  self.myc_spore_prompt = false
  self.myc_spore_prompt_hover = false
  self.myc_spore_prompt_x = 0
  self.myc_spore_prompt_y = 0
  self.myc_mushroom_prompt = false
  self.myc_build_options = gm.array()
  self.myc_build_hover = -1
  self.myc_build_node_x = 0
  self.myc_build_node_y = 0
  self.myc_build_surface_x = 0
  self.myc_build_surface_y = 0
  self.myc_build_surface_ok = false
  self.myc_build_hint = ""
  self.myc_spore_prompt_index = -1
  self.myc_regen_prompt = false
  self.myc_regen_prompt_hover = false
  self.myc_spores = gm.array()
  self.myc_has_water_source = gm.method(self, anon__1762__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_surface_block_reason = gm.method(self, anon__1897__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_is_blocked_point = gm.method(self, anon__2138__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_is_playable_point = gm.method(self, anon__2406__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_is_playable_clear_point = gm.method(self, anon__2806__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_segment_is_playable = gm.method(self, anon__3225__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_measure_allowed_len_from = gm.method(self, anon__4008__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_shot_cost_for = gm.method(self, anon__5108__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_shot_food_for = gm.method(self, anon__5711__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_button_rect = gm.method(self, anon__6019__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_option_label = gm.method(self, anon__6802__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_spore_button_rect = gm.method(self, anon__7134__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_button_rect = gm.method(self, anon__7351__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_regen_button_rect = gm.method(self, anon__7521__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_draw_prompt = gm.method(self, anon__7725__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_clear_prompts = gm.method(self, anon__8392__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_hint_rect = gm.method(self, anon__8636__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_start_aim_at = gm.method(self, anon__8801__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_has_knot_near = gm.method(self, anon__9236__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_add_knot = gm.method(self, anon__9512__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_smallshroom_sheets = gm.method(self, anon__10006__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_smallshroom_new = gm.method(self, anon__10257__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_smallshroom_rect = gm.method(self, anon__10512__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_has_smallshroom_near = gm.method(self, anon__10910__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_add_spore_knot = gm.method(self, anon__11304__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_add_root_knot_for_mushroom = gm.method(self, anon__11972__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_has_mushroom_near = gm.method(self, anon__12049__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_get_knot_at = gm.method(self, anon__12632__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_origin_box = gm.method(self, anon__13119__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_get_mushroom_at = gm.method(self, anon__13668__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_get_surface_build_at = gm.method(self, anon__14604__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_surface_buildable_at = gm.method(self, anon__14901__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_find_tree_near = gm.method(self, anon__15353__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_can_build_type = gm.method(self, anon__15583__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_type_at = gm.method(self, anon__15952__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_options_at = gm.method(self, anon__17079__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_can_build_option = gm.method(self, anon__18320__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_open_build_menu_at = gm.method(self, anon__18703__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_node_quick = gm.method(self, anon__19580__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_apply_build_option = gm.method(self, anon__20040__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_consumptor_spot = gm.method(self, anon__21041__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_place_check = gm.method(self, anon__23310__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_add_link_growth = gm.method(self, anon__25115__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_place_mushroom = gm.method(self, anon__26361__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_spore_root_chance = gm.method(self, anon__27346__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_spawn_spores_from = gm.method(self, anon__27471__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_spawn_spores_show = gm.method(self, anon__28384__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_spawn_spores_rays = gm.method(self, anon__29233__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_update_spores = gm.method(self, anon__29888__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_grid = gm.struct({})
  self.myc_grid_stamp = 0
  self.myc_far_max = 0
  self.myc_grid_add = gm.method(self, anon__31845__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_grid_rebuild = gm.method(self, anon__32773__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_get_mycelium_hit = gm.method(self, anon__32963__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_make_branch = gm.method(self, anon__36616__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_build_branch = gm.method(self, anon__39420__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_rasterize_branch = gm.method(self, anon__40328__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_add_growth_from = gm.method(self, anon__42245__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_digger_burst = gm.method(self, anon__47204__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_digger_step = gm.method(self, anon__48225__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_node_burst = gm.method(self, anon__49515__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_burst_in_view = gm.method(self, anon__50255__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_burst_fire = gm.method(self, anon__50824__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_node_burst_step = gm.method(self, anon__52233__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_prof = gm.struct({})
  self.myc_prof_add = gm.method(self, anon__54631__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_finish_growth = gm.method(self, anon__54952__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_harvest_at = gm.method(self, anon__55404__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_cells_buffer = gm.method(self, anon__55774__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_growth_free_chunks = gm.method(self, anon__57044__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_scan_growths = gm.method(self, anon__57586__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_scan_growth = gm.method(self, anon__58028__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_debug_draw = false
  self.myc_glow_on = false
  self.myc_glow_seed = -1
  self.myc_glow_blur = -1
  self.myc_glow_at = gm.undefined
  self.myc_digger_queue = gm.array()
  self.myc_digger_next = 0
  self.myc_burst_queue = gm.array()
  self.myc_burst_next = 0
  self.myc_burst_show = gm.array()
  self.myc_burst_show_next = 0
  self.myc_burst_sounded = false
  self.myc_burst_growing = gm.array()
  self.myc_pulses = gm.array()
  self.myc_pulse_timer = vis(self, other, "myc_pulse_interval_seconds") * gm.game_get_speed(gm.gamespeed_fps)
  self.myc_pulse_surface = -1
  self.myc_draw_strands = gm.method(self, anon__62655__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_draw_glow = gm.method(self, anon__67930__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_glow_set = gm.method(self, anon__71233__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_pulse_start = gm.method(self, anon__71572__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_pulse_radius = gm.method(self, anon__71848__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_pulse_ring = gm.method(self, anon__72092__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_glow_composite = gm.method(self, anon__73025__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_core_color = 14873582
  if not gm.bool(gm.variable_global_exists("myc_vformat")) then
    gm.vertex_format_begin()
    gm.vertex_format_add_position()
    gm.vertex_format_add_colour()
    global.myc_vformat = gm.vertex_format_end()
  end
  self.myc_tiles = gm.struct({})
  self.myc_tile_list = gm.array()
  self.myc_unbaked = gm.array()
  self.myc_active = gm.array()
  self.myc_frame = 0
  self.myc_span_quad = gm.method(self, anon__77430__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_write_branch_spans = gm.method(self, anon__78411__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_growth_bake = gm.method(self, anon__80501__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_tile_add = gm.method(self, anon__81793__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_tile_merge = gm.method(self, anon__82902__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_tiles_step = gm.method(self, anon__84786__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_tiles_clear = gm.method(self, anon__85453__gml_Object_obj_mycelium_controller_Create_0)
  self.myc_bake_queue_all = gm.method(self, anon__86458__gml_Object_obj_mycelium_controller_Create_0)
  if not gm.bool(global.intro_active) then
    self.myc_add_knot(self, other, self.x, self.y)
  end
  reach_expand(self, other, self.x, self.y)
  gm.window_set_cursor(gm.cr_default)
end
