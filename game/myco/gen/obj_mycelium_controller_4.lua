local gm = require("gm")

-- YYC inlines array_length as "0 unless the value is an array"
local function alen(a)
  if gm.is_array(a) then return gm.array_length(a) end
  return 0
end

-- gml_Object_obj_mycelium_controller_Draw_64 @ 0x1406b7820
function gml_Object_obj_mycelium_controller_Draw_64(self, other)
  self.myc_sync_view(self, other)
  local gw = gm.max(1, gm.display_get_gui_width())
  local gh = gm.max(1, gm.display_get_gui_height())
  local cx = self.myc_camera_x
  local cy = self.myc_camera_y
  local sx = gw / gm.max(1, self.myc_view_w)
  local sy = gh / gm.max(1, self.myc_view_h)
  local ui = (sx + sy) * 0.5
  local col = 0xe2f3ee
  local col_dark = 0x324036
  local ox = -gm.floor(cx)
  local oy = -gm.floor(cy)
  local x0 = cx - 64
  local y0 = cy - 64
  local x1 = cx + self.myc_view_w + 64
  local y1 = cy + self.myc_view_h + 64
  local old_world = gm.matrix_get(gm.matrix_world)
  gm.matrix_set(gm.matrix_world, gm.matrix_build(ox, oy, 0, 0, 0, 0, 1, 1, 1))
  local t0 = gm.get_timer()
  if gm.bool(self.myc_glow_on) or alen(self.myc_pulses) > 0 then
    self.myc_draw_glow(self, other, cx, cy, x0, y0, x1, y1)
  end
  self.myc_draw_strands(self, other, col, x0, y0, x1, y1)
  self.myc_prof_add(self, other, "draw", gm.get_timer() - t0)
  local gen_us = 0
  if gm.bool(gm.variable_global_exists("prof_gen_us")) then gen_us = global.prof_gen_us end
  self.myc_prof_add(self, other, "gen", gen_us)
  local life_us = 0
  if gm.bool(gm.variable_global_exists("prof_life_us")) then life_us = global.prof_life_us end
  self.myc_prof_add(self, other, "life", life_us)
  gm.matrix_set(gm.matrix_world, old_world)
  gm.draw_set_alpha(1)

  if gm.bool(self.myc_debug_draw) then
    local n_br = 0
    local n_cells = 0
    local n_vis = 0
    local n_cull = 0
    local i = 0
    while i < alen(self.myc_growths) do
      local g = self.myc_growths[i]
      local brs = g.branches
      n_br = n_br + alen(brs)
      local j = 0
      while j < alen(brs) do
        local b = brs[j]
        local pts = b.points
        local cells = b.cells
        local nc
        if gm.bool(gm.is_array(cells)) then nc = alen(cells) else nc = 0 end
        n_cells = n_cells + nc
        local t = gm.clamp((g.age - b.start) / g.duration, 0, 1)
        if t > 0 then
          n_vis = n_vis + gm.min(gm.ceil(t * nc), nc)
        end
        if b.bx2 < x0 or b.bx1 > x1 or b.by2 < y0 or b.by1 > y1 then
          n_cull = n_cull + 1
        end
        local k = 1
        while k < alen(pts) do
          gm.draw_line_color(pts[k - 1].x + ox, pts[k - 1].y + oy, pts[k].x + ox, pts[k].y + oy, 16711935, 16711935)
          k = k + 1
        end
        gm.draw_rectangle_color(b.bx1 + ox, b.by1 + oy, b.bx2 + ox, b.by2 + oy, 65280, 65280, 65280, 65280, true)
        j = j + 1
      end
      i = i + 1
    end
    gm.draw_set_color(0xffffff)
    gm.draw_set_halign(gm.fa_left)
    gm.draw_text_transformed(24, 232,
      "growths " .. gm.string(alen(self.myc_growths))
      .. "   branches " .. gm.string(n_br)
      .. "   culled " .. gm.string(n_cull)
      .. "   cells " .. gm.string(n_cells)
      .. "   visible " .. gm.string(n_vis)
      .. "   cam " .. gm.string(gm.floor(cx))
      .. "," .. gm.string(gm.floor(cy)),
      1.4, 1.4, 0)
    local names = gm.array("burst", "digger", "scan", "bake", "draw", "gen", "life")
    local s = "fps " .. gm.string(gm.int64(gm.fps_real)) .. "  ms:"
    local k = 0
    while k < alen(names) do
      local p = gm.variable_struct_get(self.myc_prof, names[k])
      if not gm.bool(gm.is_undefined(p)) then
        s = s .. "  " .. names[k] .. " " .. gm.string_format(p.now / 1000, 1, 1)
          .. "/" .. gm.string_format(p.peak / 1000, 1, 1)
      end
      k = k + 1
    end
    gm.draw_text_transformed(24, 292, s, 1.4, 1.4, 0)
    local layers = 0
    if gm.bool(gm.variable_global_exists("prof_gen_layers")) then layers = global.prof_gen_layers end
    gm.draw_text_transformed(24, 322,
      "wave: queue " .. gm.string(alen(self.myc_burst_queue) - self.myc_burst_next)
      .. "  shown " .. gm.string(alen(self.myc_burst_show) - self.myc_burst_show_next)
      .. "  growing " .. gm.string(alen(self.myc_burst_growing))
      .. "   unbaked " .. gm.string(alen(self.myc_unbaked))
      .. "   active " .. gm.string(alen(self.myc_active))
      .. "   tiles " .. gm.string(alen(self.myc_tile_list))
      .. "   ground layers this frame " .. gm.string(layers),
      1.4, 1.4, 0)
    if gm.bool(gm.variable_global_exists("far_rate")) then
      local fr = global.far_rate
      gm.draw_text_transformed(24, 262,
        "far / min:  leaves " .. gm.string_format(fr.leaves, 1, 1)
        .. "   worms " .. gm.string_format(fr.worms, 1, 1)
        .. "   food " .. gm.string_format(fr.food, 1, 1),
        1.4, 1.4, 0)
    end
    gm.draw_set_alpha(1)
  end

  local i = 0
  while i < alen(self.myc_spores) do
    local sp = self.myc_spores[i]
    local px = gm.floor(sp.x) + ox
    local py = gm.floor(sp.y) + oy
    if not (px < -8 or px > gw + 8 or py < -8 or py > gh + 8) then
      local r = math.fmod(i, 2) + 1
      gm.draw_set_color(col)
      gm.draw_set_alpha(gm.clamp(sp.life / 80, 0.35, 1))
      gm.draw_rectangle(px, py, px + r, py + r, false)
    end
    i = i + 1
  end
  gm.draw_set_alpha(1)

  local ks = vis(self, other, "knot_scale")
  local k1 = gm.floor(3.5 * ks - 0.5)
  local k2 = gm.floor(2.5 * ks - 0.5)
  local k3 = gm.floor(1.5 * ks - 0.5)
  i = 0
  while i < alen(self.myc_knots) do
    local kn = self.myc_knots[i]
    local kx = gm.floor(kn.x) + ox
    local ky = gm.floor(kn.y) + oy
    if not (kx < -32 or kx > gw + 32 or ky < -32 or ky > gh + 32) then
      gm.draw_set_color(col)
      gm.draw_rectangle(kx - k1, ky - k2, kx + k1, ky + k2, false)
      gm.draw_rectangle(kx - k2, ky - k1, kx + k2, ky + k1, false)
      gm.draw_set_color(col_dark)
      gm.draw_rectangle(kx - k3, ky - k3, kx + k3, ky + k3, false)
    end
    i = i + 1
  end

  if gm.bool(self.myc_dragging) and gm.bool(self.myc_has_aim) then
    local ax = (self.myc_aim_origin_x - cx) * sx
    local ay = (self.myc_aim_origin_y - cy) * sy
    local tx = (self.myc_aim_origin_x + gm.lengthdir_x(self.myc_aim_len, self.myc_aim_dir) - cx) * sx
    local ty = (self.myc_aim_origin_y + gm.lengthdir_y(self.myc_aim_len, self.myc_aim_dir) - cy) * sy
    local bx = (self.myc_aim_origin_x + gm.lengthdir_x(self.myc_tile * 0.45, self.myc_aim_dir) - cx) * sx
    local by = (self.myc_aim_origin_y + gm.lengthdir_y(self.myc_tile * 0.45, self.myc_aim_dir) - cy) * sy
    local ah = 17 * ui
    local aw = 10 * ui
    local hx = tx + gm.lengthdir_x(ah, self.myc_aim_dir + 180)
    local hy = ty + gm.lengthdir_y(ah, self.myc_aim_dir + 180)
    local p1x = hx + gm.lengthdir_x(aw, self.myc_aim_dir + 90)
    local p1y = hy + gm.lengthdir_y(aw, self.myc_aim_dir + 90)
    local p2x = hx + gm.lengthdir_x(aw, self.myc_aim_dir - 90)
    local p2y = hy + gm.lengthdir_y(aw, self.myc_aim_dir - 90)
    local ok = water_total(self, other) >= self.myc_aim_cost
      and global.food >= self.myc_aim_food
      and self.myc_aim_len >= self.myc_min_growth_len
    local acol
    if ok then acol = col else acol = 0x6876d6 end
    if self.myc_aim_reach > self.myc_aim_len + 1 then
      local rx = (self.myc_aim_origin_x + gm.lengthdir_x(self.myc_aim_reach, self.myc_aim_dir) - cx) * sx
      local ry = (self.myc_aim_origin_y + gm.lengthdir_y(self.myc_aim_reach, self.myc_aim_dir) - cy) * sy
      local tkx = gm.lengthdir_x(6 * ui, self.myc_aim_dir + 90)
      local tky = gm.lengthdir_y(6 * ui, self.myc_aim_dir + 90)
      gm.draw_set_alpha(0.2)
      gm.draw_line_width_color(tx, ty, rx, ry, 2 * ui, col, col)
      gm.draw_line_width_color(rx - tkx, ry - tky, rx + tkx, ry + tky, 3 * ui, col, col)
    end
    gm.draw_set_color(acol)
    gm.draw_set_alpha(0.18)
    gm.draw_circle(ax, ay, 13 * ui, false)
    gm.draw_line_width_color(bx, by, hx, hy, 13 * ui, acol, acol)
    gm.draw_triangle(tx, ty, p1x, p1y, p2x, p2y, false)
    gm.draw_set_alpha(0.62)
    gm.draw_line_width_color(bx, by, hx, hy, 6 * ui, acol, acol)
    gm.draw_circle(ax, ay, 5 * ui, false)
    gm.draw_set_alpha(1)
    gm.draw_line_width_color(bx, by, hx, hy, 3 * ui, acol, acol)
    gm.draw_triangle(tx, ty, p1x, p1y, p2x, p2y, false)
    gm.draw_circle(ax, ay, 2.5 * ui, false)
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_valign(gm.fa_middle)
    local label = length_text(self, other, pixels_to_cm(self, other, self.myc_aim_len))
      .. "  -" .. gm.string(gm.round(self.myc_aim_cost)) .. " " .. loc(self, other, "hud_water")
    if self.myc_aim_food > 0 then
      label = label .. "  -" .. gm.string(self.myc_aim_food) .. " " .. loc(self, other, "hud_food")
    end
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_text(tx, ty - 26 * ui, label)
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
  end

  if gm.bool(self.myc_spore_prompt) then
    self.myc_draw_prompt(self, other, self.myc_spore_button_rect(self, other),
      global.food >= bal(self, other, "spore_release_cost"), self.myc_spore_prompt_hover)
  end
  if gm.bool(self.myc_regen_prompt) then
    self.myc_draw_prompt(self, other, self.myc_regen_button_rect(self, other),
      global.food >= repeller_regen_price(self, other), self.myc_regen_prompt_hover)
  end
  if gm.bool(self.myc_mushroom_prompt) then
    i = 0
    while i < alen(self.myc_build_options) do
      local opt = self.myc_build_options[i]
      self.myc_draw_prompt(self, other, self.myc_build_button_rect(self, other, i, opt),
        self.myc_can_build_option(self, other, opt), self.myc_build_hover == i)
      i = i + 1
    end
    if self.myc_build_hint ~= "" then
      self.myc_draw_prompt(self, other, self.myc_build_hint_rect(self, other), false, false)
    end
  end
  gm.draw_set_alpha(1)
  gm.draw_set_color(0xffffff)
  global.prof_gen_us = 0
  global.prof_gen_layers = 0
end

-- gml_Object_obj_mycelium_controller_CleanUp_0 @ 0x1406bf3b0
function gml_Object_obj_mycelium_controller_CleanUp_0(self, other)
  gm.window_set_cursor(0)
  self.myc_tiles_clear(self, other)
  local i = 0
  while i < alen(self.myc_growths) do
    local g = self.myc_growths[i]
    if gm.bool(gm.variable_struct_exists(g, "sound")) then
      hyphae_sound_stop(self, other, g.sound)
    end
    self.myc_growth_free_chunks(self, other, g)
    if type(g.vbuf) == "number" and g.vbuf >= 0 then
      gm.vertex_delete_buffer(g.vbuf)
      g.vbuf = -1
    end
    i = i + 1
  end
  if gm.bool(gm.surface_exists(self.myc_glow_seed)) then
    gm.surface_free(self.myc_glow_seed)
  end
  if gm.bool(gm.surface_exists(self.myc_glow_blur)) then
    gm.surface_free(self.myc_glow_blur)
  end
  if gm.bool(gm.surface_exists(self.myc_pulse_surface)) then
    gm.surface_free(self.myc_pulse_surface)
  end
end
