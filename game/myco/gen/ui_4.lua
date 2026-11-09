local gm = require("gm")

-- gml_Script____struct___432__gml_Object_obj_hud_Create_0 @ 0x1405fecf0
function ___struct___432__gml_Object_obj_hud_Create_0(self, other, a0, a1, a2)
  if a0 == nil then a0 = gm.undefined end
  if a1 == nil then a1 = gm.undefined end
  if a2 == nil then a2 = gm.undefined end
  self.wind_tab = gm.new(___struct___433_____struct___432__gml_Object_obj_hud_Create_0, a0)
  self.upgrade_tab = gm.new(___struct___434_____struct___432__gml_Object_obj_hud_Create_0, a1)
  self.bar = gm.new(___struct___435_____struct___432__gml_Object_obj_hud_Create_0, a2)
  self.upgrade_icon = gm.new(___struct___436_____struct___432__gml_Object_obj_hud_Create_0)
  self.food_icon = gm.new(___struct___437_____struct___432__gml_Object_obj_hud_Create_0)
  self.water_icon = gm.new(___struct___438_____struct___432__gml_Object_obj_hud_Create_0)
  self.upgrade_button = gm.new(___struct___439_____struct___432__gml_Object_obj_hud_Create_0)
  self.icon_gap = 6
  self.length_x = 960
  self.text_y = 24
  self.side_scale = 0.5
  self.length_scale = 0.8
  self.unit_scale = 0.42
  self.unit_gap = 8
  self.points_x = 1442
  self.points_y = 24
  self.points_scale = 0.6
  self.wind_x = 464.5
  self.wind_y = 24
  self.wind_scale = 0.45
  self.press_scale = 0.88
  self.press_frames = 10
end

-- gml_Script_anon__3164__gml_Object_obj_hud_Create_0 @ 0x1405ffd00
function anon__3164__gml_Object_obj_hud_Create_0(self, other)
  local dx = global.ui_ox - self.hud_layout_ox
  local dy = global.ui_by - self.hud_layout_by
  if dx == 0 and dy == 0 then
    return
  end
  self.hud_layout_ox = global.ui_ox
  self.hud_layout_by = global.ui_by
  local u = self.hud_ui
  u.wind_tab.x = u.wind_tab.x + dx
  u.upgrade_tab.x = u.upgrade_tab.x + dx
  u.bar.x = u.bar.x + dx
  u.upgrade_icon.x = u.upgrade_icon.x + dx
  u.food_icon.x = u.food_icon.x + dx
  u.water_icon.x = u.water_icon.x + dx
  u.upgrade_button.x = u.upgrade_button.x + dx
  u.length_x = u.length_x + dx
  u.points_x = u.points_x + dx
  u.wind_x = u.wind_x + dx
  local c = self.hud_camnav
  c.left_x = c.left_x + dx * 2
  c.origin_x = c.origin_x + dx * 2
  c.right_x = c.right_x + dx * 2
  c.y = c.y + dy
  local b = self.hud_build
  b.circle_y = b.circle_y + dy
  b.bar_y = b.bar_y + dy
  b.cy = b.cy + dy
  b.slot_y = b.slot_y + dy
end

-- gml_Script____struct___440__anon__4046__gml_Object_obj_hud_Create_0 @ 0x140600f50
function ___struct___440__anon__4046__gml_Object_obj_hud_Create_0(self, other, x1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = 0
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__4046__gml_Object_obj_hud_Create_0 @ 0x1406011d0
function anon__4046__gml_Object_obj_hud_Create_0(self, other)
  local t = self.hud_ui.upgrade_tab
  local x = t.x
  local s = t.scale
  local y = t.y
  return gm.new(___struct___440__anon__4046__gml_Object_obj_hud_Create_0, x,
    x + gm.sprite_get_width(UI_PIPKA) * s, y + gm.sprite_get_height(UI_PIPKA) * s)
end

-- gml_Script____struct___441__anon__4298__gml_Object_obj_hud_Create_0 @ 0x140601990
function ___struct___441__anon__4298__gml_Object_obj_hud_Create_0(self, other, x1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = 0
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__4298__gml_Object_obj_hud_Create_0 @ 0x140601c10
function anon__4298__gml_Object_obj_hud_Create_0(self, other)
  local t = self.hud_ui.wind_tab
  local x = t.x
  local s = t.scale
  local y = t.y
  return gm.new(___struct___441__anon__4298__gml_Object_obj_hud_Create_0, x,
    x + gm.sprite_get_width(UI_PIPKA) * s, y + gm.sprite_get_height(UI_PIPKA) * s)
end

-- gml_Script_anon__4545__gml_Object_obj_hud_Create_0 @ 0x1406023d0
function anon__4545__gml_Object_obj_hud_Create_0(self, other, spr, cx, cy, iscale, text, tscale)
  if spr == nil then spr = gm.undefined end
  if cx == nil then cx = gm.undefined end
  if cy == nil then cy = gm.undefined end
  if iscale == nil then iscale = gm.undefined end
  if text == nil then text = gm.undefined end
  if tscale == nil then tscale = gm.undefined end
  local w = gm.sprite_get_width(spr) * iscale
  local h = gm.sprite_get_height(spr) * iscale
  local tw = gm.string_width(text) * tscale
  local x = cx - (w + self.hud_ui.icon_gap + tw) * 0.5
  gm.draw_sprite_ext(spr, 0, x, cy - h * 0.5, iscale, iscale, 0, gm.c_white, 1)
  gm.draw_set_halign(gm.fa_left)
  self.hud_text(self, other, x + w + self.hud_ui.icon_gap, cy, text, tscale)
end

-- gml_Script_anon__5249__gml_Object_obj_hud_Create_0 @ 0x140603080
function anon__5249__gml_Object_obj_hud_Create_0(self, other, spr_a, spr_b, x, y, cond, press)
  if spr_a == nil then spr_a = gm.undefined end
  if spr_b == nil then spr_b = gm.undefined end
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if cond == nil then cond = gm.undefined end
  if press == nil then press = gm.undefined end
  local spr
  if gm.bool(cond) then
    spr = spr_b
  else
    spr = spr_a
  end
  local w = gm.sprite_get_width(spr)
  local h = gm.sprite_get_height(spr)
  local t = 1 - press / gm.max(1, self.hud_ui.press_frames)
  local s
  if press > 0 then
    s = gm.lerp(1, self.hud_ui.press_scale, math.sin(t * math.pi))
  else
    s = 1
  end
  gm.draw_sprite_ext(spr, 0, x + w * 0.5 * (1 - s), y + h * 0.5 * (1 - s), s, s, 0, gm.c_white, 1)
end

-- gml_Script____struct___442__gml_Object_obj_hud_Create_0 @ 0x140603f80
function ___struct___442__gml_Object_obj_hud_Create_0(self, other, price_gap)
  if price_gap == nil then price_gap = gm.undefined end
  self.circle_x = 64
  self.circle_y = 864
  self.bar_x = 32
  self.bar_y = 864
  self.cx = 126
  self.cy = 925
  self.r = 60
  self.bar_right = 1269
  self.slot_x0 = 240
  self.slot_dx = 125
  self.slot_y = 925
  self.slot_tail = 154
  self.price_gap = price_gap
  self.icon = 74
  self.price_scale = 0.42
  self.price_line = 26
  self.bite_size = 22
  self.slide_frames = 24
end

-- gml_Script____struct___443__gml_Object_obj_hud_Create_0 @ 0x140604500
function ___struct___443__gml_Object_obj_hud_Create_0(self, other)
  self.left_x = 1571
  self.origin_x = 1663
  self.right_x = 1755
  self.y = 864
  self.origin_r = 60
  self.arrow_pad = 8
  self.hover_scale = 1.08
end

-- gml_Script_anon__6977__gml_Object_obj_hud_Create_0 @ 0x1406047d0
function anon__6977__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  return gm.array(CAMERA_LEFT, ORIGIN_SHROOM, CAMERA_RIGHT)[i]
end

-- gml_Script_anon__7067__gml_Object_obj_hud_Create_0 @ 0x140604b00
function anon__7067__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local c = self.hud_camnav
  return gm.array(c.left_x, c.origin_x, c.right_x)[i]
end

-- gml_Script_anon__7246__gml_Object_obj_hud_Create_0 @ 0x140604f90
function anon__7246__gml_Object_obj_hud_Create_0(self, other, mx, my)
  if mx == nil then mx = gm.undefined end
  if my == nil then my = gm.undefined end
  local c = self.hud_camnav
  local i = 0
  while i < 3 do
    local spr = self.hud_cam_sprite(self, other, i)
    local x = self.hud_cam_x(self, other, i)
    if i == 1 then
      if gm.point_distance(mx, my, x + gm.sprite_get_width(spr) * 0.5, c.y + gm.sprite_get_height(spr) * 0.5) <= c.origin_r then
        return i
      end
    else
      local p = c.arrow_pad
      if gm.point_in_rectangle(mx, my,
          x + gm.sprite_get_bbox_left(spr) - p, c.y + gm.sprite_get_bbox_top(spr) - p,
          x + gm.sprite_get_bbox_right(spr) + p, c.y + gm.sprite_get_bbox_bottom(spr) + p) then
        return i
      end
    end
    i = i + 1
  end
  return -1
end

-- gml_Script_anon__8098__gml_Object_obj_hud_Create_0 @ 0x1406062e0
function anon__8098__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_camnav
  local i = 0
  while i < 3 do
    local spr = self.hud_cam_sprite(self, other, i)
    local w = gm.sprite_get_width(spr)
    local h = gm.sprite_get_height(spr)
    local press = self.hud_cam_press[i]
    local t = 1 - press / gm.max(1, self.hud_ui.press_frames)
    local s
    if press > 0 then
      s = gm.lerp(1, self.hud_ui.press_scale, math.sin(t * math.pi))
    else
      s = 1
    end
    if self.hud_cam_hover == i then
      s = s * c.hover_scale
    end
    local x = self.hud_cam_x(self, other, i)
    local y = c.y
    gm.draw_sprite_ext(spr, 0, x + w * 0.5 * (1 - s), y + h * 0.5 * (1 - s), s, s, 0, gm.c_white, 1)
    i = i + 1
  end
end

-- gml_Script_anon__9094__gml_Object_obj_hud_Create_0 @ 0x140607650
function anon__9094__gml_Object_obj_hud_Create_0(self, other)
  return gm.array(0, 1, 2, 3, 4, 5, 6, 7)
end

-- gml_Script_anon__9334__gml_Object_obj_hud_Create_0 @ 0x140607780
function anon__9334__gml_Object_obj_hud_Create_0(self, other, typ)
  if typ == nil then typ = gm.undefined end
  local all = self.hud_build_all_types(self, other)
  local i = 0
  while i < gm.array_length(all) do
    if all[i] == typ then
      return i + 1
    end
    i = i + 1
  end
  return 0
end

-- gml_Script_anon__9561__gml_Object_obj_hud_Create_0 @ 0x140607b20
function anon__9561__gml_Object_obj_hud_Create_0(self, other)
  local all = self.hud_build_all_types(self, other)
  local out = gm.array()
  local i = 0
  while i < gm.array_length(all) do
    if gm.bool(mushroom_type_unlocked(self, other, all[i])) then
      gm.array_push(out, all[i])
    end
    i = i + 1
  end
  return out
end

-- gml_Script_anon__9983__gml_Object_obj_hud_Create_0 @ 0x140608080
function anon__9983__gml_Object_obj_hud_Create_0(self, other)
  local b = self.hud_build
  local n = gm.array_length(self.hud_build_types(self, other))
  if n <= 1 then
    return b.slot_dx
  end
  return gm.min(b.slot_dx, (b.bar_right - b.slot_x0 - b.slot_tail) / (n - 1))
end

-- gml_Script____struct___444__anon__10241__gml_Object_obj_hud_Create_0 @ 0x140608710
function ___struct___444__anon__10241__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__10241__gml_Object_obj_hud_Create_0 @ 0x1406089a0
function anon__10241__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local b = self.hud_build
  local step = self.hud_build_slot_step(self, other)
  local x = b.slot_x0 + i * step
  local icon = b.icon
  local sy = b.slot_y
  return gm.new(___struct___444__anon__10241__gml_Object_obj_hud_Create_0,
    x - icon * 0.5, sy - icon * 0.5, x + step - 24, sy + icon * 0.5)
end

-- gml_Script_anon__10507__gml_Object_obj_hud_Create_0 @ 0x140609440
function anon__10507__gml_Object_obj_hud_Create_0(self, other, mx, my)
  if mx == nil then mx = gm.undefined end
  if my == nil then my = gm.undefined end
  local b = self.hud_build
  if gm.point_distance(mx, my, b.cx, b.cy) <= b.r then
    return true
  end
  local res = false
  if self.hud_build_slide > 0.5 then
    res = gm.bool(gm.point_in_rectangle(mx, my, b.cx, b.bar_y, b.bar_right, b.bar_y + 123))
  end
  return res
end

-- gml_Script_anon__10774__gml_Object_obj_hud_Create_0 @ 0x140609bf0
function anon__10774__gml_Object_obj_hud_Create_0(self, other)
  self.hud_build_open = false
  self.hud_place_type = -1
  self.hud_place_at = gm.undefined
end

-- gml_Script____struct___445__anon__10901__gml_Object_obj_hud_Create_0 @ 0x140609d70
function ___struct___445__anon__10901__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__10901__gml_Object_obj_hud_Create_0 @ 0x14060a020
function anon__10901__gml_Object_obj_hud_Create_0(self, other)
  local b = self.hud_ui.upgrade_button
  local x = b.x
  local y = b.y
  return gm.new(___struct___445__anon__10901__gml_Object_obj_hud_Create_0,
    x, y, x + gm.sprite_get_width(UPGRADE_CALM), y + gm.sprite_get_height(UPGRADE_CALM))
end

-- gml_Script_anon__11140__gml_Object_obj_hud_Create_0 @ 0x14060a700
function anon__11140__gml_Object_obj_hud_Create_0(self, other, mx, my)
  if mx == nil then mx = gm.undefined end
  if my == nil then my = gm.undefined end
  local r = self.hud_upgrades_button_rect(self, other)
  return gm.point_distance(mx, my, (r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5) <= (r.x2 - r.x1) * 0.5
end

-- gml_Script_anon__11820__gml_Object_obj_hud_Create_0 @ 0x14060ae60
function anon__11820__gml_Object_obj_hud_Create_0(self, other)
  return gm.display_get_gui_height() * 0.5 - 200
end

-- gml_Script_anon__11912__gml_Object_obj_hud_Create_0 @ 0x14060b060
function anon__11912__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local res = false
  if self.hud_pause_items[i] == "window_size" then
    res = gm.bool(global.display_fullscreen)
  end
  return res
end

-- gml_Script_anon__12865__gml_Object_obj_hud_Create_0 @ 0x14060b1e0
function anon__12865__gml_Object_obj_hud_Create_0(self, other)
  local cam_y
  if gm.instance_exists(obj_forest_controller) then
    cam_y = gm.instance_find(obj_forest_controller, 0).camera_view_y
  else
    cam_y = 0
  end
  return wld(self, other, "surface_height_px") - cam_y + 40
end

-- gml_Script____struct___446__anon__13059__gml_Object_obj_hud_Create_0 @ 0x14060b690
function ___struct___446__anon__13059__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__13059__gml_Object_obj_hud_Create_0 @ 0x14060b920
function anon__13059__gml_Object_obj_hud_Create_0(self, other)
  local gw = gm.display_get_gui_width()
  local w = 420
  local h = 64
  local y = self.hud_credits_top(self, other) + 380
  return gm.new(___struct___446__anon__13059__gml_Object_obj_hud_Create_0,
    (gw - w) * 0.5, y, (gw + w) * 0.5, y + h)
end

-- gml_Script_anon__13634__gml_Object_obj_hud_Create_0 @ 0x14060c120
function anon__13634__gml_Object_obj_hud_Create_0(self, other, mode)
  if mode == nil then mode = gm.undefined end
  self.hud_slot_mode = mode
  self.hud_slot_naming = -1
  self.hud_slot_metas = gm.array()
  local i = 0
  while i < 3 do
    gm.array_push(self.hud_slot_metas, save_slot_meta(self, other, i))
    i = i + 1
  end
end

-- gml_Script_anon__13863__gml_Object_obj_hud_Create_0 @ 0x14060c590
function anon__13863__gml_Object_obj_hud_Create_0(self, other)
  self.hud_slot_mode = ""
  self.hud_slot_naming = -1
end

-- gml_Script____struct___447__anon__13992__gml_Object_obj_hud_Create_0 @ 0x14060c6a0
function ___struct___447__anon__13992__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__13992__gml_Object_obj_hud_Create_0 @ 0x14060c950
function anon__13992__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local gw = gm.display_get_gui_width()
  local cam_y
  if gm.instance_exists(obj_forest_controller) then
    cam_y = gm.instance_find(obj_forest_controller, 0).camera_view_y
  else
    cam_y = 0
  end
  local top = wld(self, other, "surface_height_px") - cam_y + 60
  local w = 760
  local h = 76
  local x = (gw - w) * 0.5
  local y = top + i * (h + 12)
  return gm.new(___struct___447__anon__13992__gml_Object_obj_hud_Create_0, x, y, x + w, y + h)
end

-- gml_Script_anon__14519__gml_Object_obj_hud_Create_0 @ 0x14060d570
function anon__14519__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local m = self.hud_slot_metas[i]
  local head = loc(self, other, "slot_label") .. " " .. gm.string(i + 1) .. "  -  "
  if gm.is_undefined(m) then
    return head .. loc(self, other, "slot_empty")
  end
  local len = m.length_cm
  if gm.is_undefined(len) then
    len = 0
  end
  return head .. gm.string(m.name) .. "  -  " .. length_text(self, other, len)
end

-- gml_Script_anon__14951__gml_Object_obj_hud_Create_0 @ 0x14060e1a0
function anon__14951__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local m = self.hud_slot_metas[i]
  if gm.is_undefined(m) then
    return ""
  end
  local mode = m.mode
  if gm.is_undefined(mode) then
    mode = "exploration"
  end
  return loc(self, other, "mode_" .. gm.string(mode))
end

-- gml_Script_anon__15291__gml_Object_obj_hud_Create_0 @ 0x14060e810
function anon__15291__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local res = false
  if self.hud_title_items[i] == "load_game" then
    res = not gm.bool(self.hud_saves_exist)
  end
  return res
end

-- gml_Script____struct___448__anon__15409__gml_Object_obj_hud_Create_0 @ 0x14060e980
function ___struct___448__anon__15409__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__15409__gml_Object_obj_hud_Create_0 @ 0x14060ec30
function anon__15409__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local gw = gm.display_get_gui_width()
  local cam_y
  if gm.instance_exists(obj_forest_controller) then
    cam_y = gm.instance_find(obj_forest_controller, 0).camera_view_y
  else
    cam_y = 0
  end
  local top = wld(self, other, "surface_height_px") - cam_y + 90
  local w = 420
  local h = 60
  local x = (gw - w) * 0.5
  local y = top + i * (h + 8)
  return gm.new(___struct___448__anon__15409__gml_Object_obj_hud_Create_0, x, y, x + w, y + h)
end

-- gml_Script____struct___449__anon__16043__gml_Object_obj_hud_Create_0 @ 0x14060f850
function ___struct___449__anon__16043__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__16043__gml_Object_obj_hud_Create_0 @ 0x14060fb00
function anon__16043__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  local w = 420
  local x = (gw - w) * 0.5
  local y = self.hud_pause_cy(self, other) - 112 + i * 62
  return gm.new(___struct___449__anon__16043__gml_Object_obj_hud_Create_0, x, y, x + w, y + 26)
end

-- gml_Script____struct___450__anon__16388__gml_Object_obj_hud_Create_0 @ 0x140610420
function ___struct___450__anon__16388__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__16388__gml_Object_obj_hud_Create_0 @ 0x1406106d0
function anon__16388__gml_Object_obj_hud_Create_0(self, other, i)
  if i == nil then i = gm.undefined end
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  local w = 420
  local h = 64
  local x = (gw - w) * 0.5
  local y = self.hud_pause_cy(self, other) + 48 + i * (h + 12)
  return gm.new(___struct___450__anon__16388__gml_Object_obj_hud_Create_0, x, y, x + w, y + h)
end

-- gml_Script____struct___451__anon__16771__gml_Object_obj_hud_Create_0 @ 0x140611080
function ___struct___451__anon__16771__gml_Object_obj_hud_Create_0(self, other, x1, x2)
  if x1 == nil then x1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  self.x1 = x1
  self.y1 = 30
  self.x2 = x2
  self.y2 = 76
end

-- gml_Script_anon__16771__gml_Object_obj_hud_Create_0 @ 0x1406112d0
function anon__16771__gml_Object_obj_hud_Create_0(self, other)
  local gw = gm.display_get_gui_width()
  return gm.new(___struct___451__anon__16771__gml_Object_obj_hud_Create_0, gw - 152, gw - 32)
end

-- gml_Script____struct___452__anon__16960__gml_Object_obj_hud_Create_0 @ 0x1406116f0
function ___struct___452__anon__16960__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__16960__gml_Object_obj_hud_Create_0 @ 0x1406119a0
function anon__16960__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_close_button_rect(self, other)
  return gm.new(___struct___452__anon__16960__gml_Object_obj_hud_Create_0, c.x1 - 208, c.y1, c.x1 - 20, c.y2)
end

-- gml_Script____struct___453__anon__17165__gml_Object_obj_hud_Create_0 @ 0x140611f70
function ___struct___453__anon__17165__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__17165__gml_Object_obj_hud_Create_0 @ 0x140612220
function anon__17165__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_recentre_button_rect(self, other)
  return gm.new(___struct___453__anon__17165__gml_Object_obj_hud_Create_0, c.x1 - 580, c.y1, c.x1 - 20, c.y2)
end

-- gml_Script____struct___454__anon__17388__gml_Object_obj_hud_Create_0 @ 0x1406127f0
function ___struct___454__anon__17388__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__17388__gml_Object_obj_hud_Create_0 @ 0x140612aa0
function anon__17388__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_close_button_rect(self, other)
  return gm.new(___struct___454__anon__17388__gml_Object_obj_hud_Create_0, c.x1 - 208, c.y1, c.x1 - 20, c.y2)
end

-- gml_Script____struct___455__anon__17592__gml_Object_obj_hud_Create_0 @ 0x140613070
function ___struct___455__anon__17592__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__17592__gml_Object_obj_hud_Create_0 @ 0x140613320
function anon__17592__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_logo_reroll_rect(self, other)
  return gm.new(___struct___455__anon__17592__gml_Object_obj_hud_Create_0, c.x1 - 188, c.y1, c.x1 - 20, c.y2)
end

-- gml_Script____struct___456__anon__17796__gml_Object_obj_hud_Create_0 @ 0x1406138f0
function ___struct___456__anon__17796__gml_Object_obj_hud_Create_0(self, other, y1, x2, y2)
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = 0
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__17796__gml_Object_obj_hud_Create_0 @ 0x140613b60
function anon__17796__gml_Object_obj_hud_Create_0(self, other)
  return gm.new(___struct___456__anon__17796__gml_Object_obj_hud_Create_0, self.hud_tree_header_h,
    gm.display_get_gui_width(), gm.display_get_gui_height() - self.hud_tree_footer_h)
end

-- gml_Script_anon__17990__gml_Object_obj_hud_Create_0 @ 0x140613f50
function anon__17990__gml_Object_obj_hud_Create_0(self, other, id)
  if id == nil then id = gm.undefined end
  return upg_tree_pos(self, other, id)
end

-- gml_Script____struct___457__anon__18060__gml_Object_obj_hud_Create_0 @ 0x1406140f0
function ___struct___457__anon__18060__gml_Object_obj_hud_Create_0(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.x = x
  self.y = y
end

-- gml_Script_anon__18060__gml_Object_obj_hud_Create_0 @ 0x1406142f0
function anon__18060__gml_Object_obj_hud_Create_0(self, other, id)
  if id == nil then id = gm.undefined end
  local c = self.hud_tree_canvas(self, other)
  local w = self.hud_tree_world(self, other, id)
  return gm.new(___struct___457__anon__18060__gml_Object_obj_hud_Create_0,
    (c.x1 + c.x2) * 0.5 + (w.x - self.hud_tree_cx) * self.hud_tree_zoom,
    (c.y1 + c.y2) * 0.5 + (w.y - self.hud_tree_cy) * self.hud_tree_zoom)
end

-- gml_Script_anon__18562__gml_Object_obj_hud_Create_0 @ 0x140614de0
function anon__18562__gml_Object_obj_hud_Create_0(self, other, cx, y, w, h, scale, alpha)
  if cx == nil then cx = gm.undefined end
  if y == nil then y = gm.undefined end
  if w == nil then w = gm.undefined end
  if h == nil then h = gm.undefined end
  if scale == nil then scale = gm.undefined end
  if alpha == nil then alpha = gm.undefined end
  if alpha <= 0 then
    return
  end
  local padx = 8 * scale + 2
  local pady = 3 * scale + 1
  gm.draw_set_colour(self.hud_color_panel)
  gm.draw_set_alpha(vis(self, other, "tree_label_back_alpha") * gm.clamp(alpha, 0, 1))
  gm.draw_roundrect_ext(cx - w * 0.5 - padx, y - pady, cx + w * 0.5 + padx, y + h + pady,
    12 * scale, 12 * scale, false)
  gm.draw_set_alpha(1)
end

-- gml_Script_anon__19008__gml_Object_obj_hud_Create_0 @ 0x140615aa0
function anon__19008__gml_Object_obj_hud_Create_0(self, other, node)
  if node == nil then node = gm.undefined end
  local r = vis(self, other, "tree_node_radius")
  local kind = node.kind
  if kind == "root" then
    return r * vis(self, other, "tree_root_radius_mult")
  elseif kind == "unlock" then
    return r * vis(self, other, "tree_unlock_radius_mult")
  end
  return r
end

-- gml_Script____struct___458__anon__19273__gml_Object_obj_hud_Create_0 @ 0x140616090
function ___struct___458__anon__19273__gml_Object_obj_hud_Create_0(self, other, x1, y1, x2, y2)
  if x1 == nil then x1 = gm.undefined end
  if y1 == nil then y1 = gm.undefined end
  if x2 == nil then x2 = gm.undefined end
  if y2 == nil then y2 = gm.undefined end
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__19273__gml_Object_obj_hud_Create_0 @ 0x140616320
function anon__19273__gml_Object_obj_hud_Create_0(self, other)
  local c = self.hud_tree_canvas(self, other)
  local min_x = 0
  local max_x = 0
  local min_y = 0
  local max_y = 0
  local i = 0
  while i < gm.array_length(global.upgrade_defs) do
    local w = self.hud_tree_world(self, other, global.upgrade_defs[i])
    local r = self.hud_tree_radius(self, other, global.upgrade_defs[i])
    if i == 0 then
      min_x = w.x - r
      max_x = w.x + r
      min_y = w.y - r
      max_y = w.y + r
    else
      min_x = gm.min(min_x, w.x - r)
      max_x = gm.max(max_x, w.x + r)
      min_y = gm.min(min_y, w.y - r)
      max_y = gm.max(max_y, w.y + r)
    end
    i = i + 1
  end
  min_y = min_y - 80
  max_y = max_y + 90
  self.hud_tree_bounds = gm.new(___struct___458__anon__19273__gml_Object_obj_hud_Create_0, min_x, min_y, max_x, max_y)
  local margin = 90
  local zx = (c.x2 - c.x1 - margin * 2) / gm.max(1, max_x - min_x)
  local zy = (c.y2 - c.y1 - margin * 2) / gm.max(1, max_y - min_y)
  self.hud_tree_fit_zoom = gm.min(gm.min(zx, zy), vis(self, other, "tree_zoom_max"))
  self.hud_tree_zoom_lo = gm.min(vis(self, other, "tree_zoom_min"),
    self.hud_tree_fit_zoom * vis(self, other, "tree_zoom_out_past_fit"))
  self.hud_tree_zoom = self.hud_tree_fit_zoom
  self.hud_tree_cx = (min_x + max_x) * 0.5
  self.hud_tree_cy = (min_y + max_y) * 0.5
  self.hud_tree_fit_w = gm.display_get_gui_width()
  self.hud_tree_fit_h = gm.display_get_gui_height()
  self.hud_tree_fitted = true
end

-- gml_Script_anon__20896__gml_Object_obj_hud_Create_0 @ 0x1406187f0
function anon__20896__gml_Object_obj_hud_Create_0(self, other, size, min_size)
  if size == nil then size = gm.undefined end
  if min_size == nil then min_size = gm.undefined end
  return gm.max(min_size, size * self.hud_tree_zoom)
end

-- gml_Script_anon__20995__gml_Object_obj_hud_Create_0 @ 0x140618a30
function anon__20995__gml_Object_obj_hud_Create_0(self, other)
  local pad = 260
  self.hud_tree_cx = gm.clamp(self.hud_tree_cx, self.hud_tree_bounds.x1 - pad, self.hud_tree_bounds.x2 + pad)
  self.hud_tree_cy = gm.clamp(self.hud_tree_cy, self.hud_tree_bounds.y1 - pad, self.hud_tree_bounds.y2 + pad)
end

-- gml_Script_anon__21307__gml_Object_obj_hud_Create_0 @ 0x140618f10
function anon__21307__gml_Object_obj_hud_Create_0(self, other, mx, my)
  if mx == nil then mx = gm.undefined end
  if my == nil then my = gm.undefined end
  local c = self.hud_tree_canvas(self, other)
  if not gm.bool(gm.point_in_rectangle(mx, my, c.x1, c.y1, c.x2, c.y2)) then
    return ""
  end
  local i = gm.array_length(global.upgrade_defs) - 1
  while i >= 0 do
    local d = global.upgrade_defs[i]
    local p = self.hud_tree_screen(self, other, d)
    local r = self.hud_tree_radius(self, other, d) * self.hud_tree_zoom
    if gm.point_distance(mx, my, p.x, p.y) <= r then
      return d.id
    end
    i = i - 1
  end
  return ""
end

-- gml_Object_obj_hud_Create_0 @ 0x140619b70
function gml_Object_obj_hud_Create_0(self, other)
  self.depth = -1000
  self.hud_tree_zoom = 1
  self.hud_tree_cx = 0
  self.hud_tree_cy = 0
  self.hud_tree_fitted = false
  self.hud_tree_fit_zoom = 1
  self.hud_tree_zoom_lo = 0.25
  self.hud_tree_fit_w = 0
  self.hud_tree_fit_h = 0
  self.hud_tree_bounds = gm.new(___struct___431__gml_Object_obj_hud_Create_0)
  self.hud_tree_drag = false
  self.hud_tree_dragged = false
  self.hud_tree_drag_x = 0
  self.hud_tree_drag_y = 0
  self.hud_tree_press_id = ""
  self.hud_tree_header_h = 104
  self.hud_tree_footer_h = 72
  self.hud_hover_id = ""
  self.hud_pause_hover = -1
  self.hud_volume_drag = -1
  global.ui_capture_mouse = false
  global.right_click_used = false
  self.hud_color_ink = 14873582
  self.hud_color_dim = 8689804
  self.hud_color_deep = 4609098
  self.hud_color_good = 9098922
  self.hud_color_bad = 6846166
  self.hud_color_gold = 7914718
  self.hud_color_panel = 1185298
  self.hud_text = gm.method(self, anon__1101__gml_Object_obj_hud_Create_0)
  self.hud_fit = gm.method(self, anon__1399__gml_Object_obj_hud_Create_0)
  self.hud_text_box = gm.method(self, anon__1832__gml_Object_obj_hud_Create_0)
  self.hud_ui = gm.new(___struct___432__gml_Object_obj_hud_Create_0, -32, -32, -192)
  self.hud_layout_ox = 0
  self.hud_layout_by = 0
  self.hud_layout_apply = gm.method(self, anon__3164__gml_Object_obj_hud_Create_0)
  self.hud_wind_hover = false
  self.hud_points_hover = false
  self.hud_points_tab_rect = gm.method(self, anon__4046__gml_Object_obj_hud_Create_0)
  self.hud_wind_tab_rect = gm.method(self, anon__4298__gml_Object_obj_hud_Create_0)
  self.hud_icon_number = gm.method(self, anon__4545__gml_Object_obj_hud_Create_0)
  self.hud_build_btn_hover = false
  self.hud_build_btn_press = 0
  self.hud_upgrade_btn_hover = false
  self.hud_upgrade_btn_press = 0
  self.hud_draw_button = gm.method(self, anon__5249__gml_Object_obj_hud_Create_0)
  self.hud_build = gm.new(___struct___442__gml_Object_obj_hud_Create_0, -10)
  self.hud_camnav = gm.new(___struct___443__gml_Object_obj_hud_Create_0)
  self.hud_cam_hover = -1
  self.hud_cam_press = gm.array(0, 0, 0)
  self.hud_cam_sprite = gm.method(self, anon__6977__gml_Object_obj_hud_Create_0)
  self.hud_cam_x = gm.method(self, anon__7067__gml_Object_obj_hud_Create_0)
  self.hud_cam_pick = gm.method(self, anon__7246__gml_Object_obj_hud_Create_0)
  self.hud_cam_draw = gm.method(self, anon__8098__gml_Object_obj_hud_Create_0)
  self.hud_layout_apply(self, other)
  self.hud_build_open = false
  self.hud_build_slide = 0
  self.hud_build_hover = -1
  self.hud_place_type = -1
  self.hud_place_at = gm.undefined
  self.hud_build_all_types = gm.method(self, anon__9094__gml_Object_obj_hud_Create_0)
  self.hud_build_key_of = gm.method(self, anon__9334__gml_Object_obj_hud_Create_0)
  self.hud_build_types = gm.method(self, anon__9561__gml_Object_obj_hud_Create_0)
  self.hud_build_slot_step = gm.method(self, anon__9983__gml_Object_obj_hud_Create_0)
  self.hud_build_slot_rect = gm.method(self, anon__10241__gml_Object_obj_hud_Create_0)
  self.hud_build_mouse_over = gm.method(self, anon__10507__gml_Object_obj_hud_Create_0)
  self.hud_build_close = gm.method(self, anon__10774__gml_Object_obj_hud_Create_0)
  self.hud_upgrades_button_rect = gm.method(self, anon__10901__gml_Object_obj_hud_Create_0)
  self.hud_upgrades_button_over = gm.method(self, anon__11140__gml_Object_obj_hud_Create_0)
  self.hud_pause_items_game = gm.array("resume", "display_mode", "window_size", "units", "language", "main_menu", "quit")
  self.hud_pause_items_title = gm.array("resume", "display_mode", "window_size", "units", "language", "quit")
  self.hud_pause_items = self.hud_pause_items_game
  self.hud_pause_cy = gm.method(self, anon__11820__gml_Object_obj_hud_Create_0)
  self.hud_pause_item_disabled = gm.method(self, anon__11912__gml_Object_obj_hud_Create_0)
  self.hud_volume_channels = gm.array("sfx", "music")
  self.hud_title_items = gm.array("exploration", "survival", "load_game", "settings", "credits", "exit")
  self.hud_new_mode = "exploration"
  self.hud_title_hover = -1
  self.hud_credits_open = false
  self.hud_credits_back_hover = false
  self.hud_credits_columns = gm.array(
    gm.array(
      gm.array("credits_plan_kapkan", gm.array("Andrey Tyulenev", "Philipp Zybkovets")),
      gm.array("credits_smm_qa", gm.array("Daria Konakhova"))),
    gm.array(
      gm.array("credits_art_ui", gm.array("LaNa_FeeD", "Wothy (@wothella)", "Olga Zaporozhskaia", "Oniks")),
      gm.array("credits_forest_assets", gm.array("Eder Muniz"))))
  self.hud_credits_top = gm.method(self, anon__12865__gml_Object_obj_hud_Create_0)
  self.hud_credits_back_rect = gm.method(self, anon__13059__gml_Object_obj_hud_Create_0)
  self.hud_slot_mode = ""
  self.hud_slot_hover = -1
  self.hud_slot_naming = -1
  self.hud_slot_metas = gm.array()
  self.hud_slot_open = gm.method(self, anon__13634__gml_Object_obj_hud_Create_0)
  self.hud_slot_close = gm.method(self, anon__13863__gml_Object_obj_hud_Create_0)
  self.hud_slot_button_rect = gm.method(self, anon__13992__gml_Object_obj_hud_Create_0)
  self.hud_slot_label = gm.method(self, anon__14519__gml_Object_obj_hud_Create_0)
  self.hud_slot_mode_label = gm.method(self, anon__14951__gml_Object_obj_hud_Create_0)
  self.hud_saves_exist = save_any_exists(self, other)
  self.hud_title_item_disabled = gm.method(self, anon__15291__gml_Object_obj_hud_Create_0)
  self.hud_title_button_rect = gm.method(self, anon__15409__gml_Object_obj_hud_Create_0)
  self.hud_volume_labels = gm.array("volume_sfx", "volume_music")
  self.hud_volume_slider_rect = gm.method(self, anon__16043__gml_Object_obj_hud_Create_0)
  self.hud_pause_button_rect = gm.method(self, anon__16388__gml_Object_obj_hud_Create_0)
  self.hud_close_button_rect = gm.method(self, anon__16771__gml_Object_obj_hud_Create_0)
  self.hud_recentre_button_rect = gm.method(self, anon__16960__gml_Object_obj_hud_Create_0)
  self.hud_logo_button_rect = gm.method(self, anon__17165__gml_Object_obj_hud_Create_0)
  self.hud_logo_reroll_rect = gm.method(self, anon__17388__gml_Object_obj_hud_Create_0)
  self.hud_logo_clear_rect = gm.method(self, anon__17592__gml_Object_obj_hud_Create_0)
  self.hud_tree_canvas = gm.method(self, anon__17796__gml_Object_obj_hud_Create_0)
  self.hud_tree_world = gm.method(self, anon__17990__gml_Object_obj_hud_Create_0)
  self.hud_tree_screen = gm.method(self, anon__18060__gml_Object_obj_hud_Create_0)
  self.hud_tree_label_back = gm.method(self, anon__18562__gml_Object_obj_hud_Create_0)
  self.hud_tree_radius = gm.method(self, anon__19008__gml_Object_obj_hud_Create_0)
  self.hud_tree_fit = gm.method(self, anon__19273__gml_Object_obj_hud_Create_0)
  self.hud_tree_text_scale = gm.method(self, anon__20896__gml_Object_obj_hud_Create_0)
  self.hud_tree_clamp_view = gm.method(self, anon__20995__gml_Object_obj_hud_Create_0)
  self.hud_tree_pick = gm.method(self, anon__21307__gml_Object_obj_hud_Create_0)
end

-- gml_Object_obj_hud_Step_1 @ 0x14061d730
function gml_Object_obj_hud_Step_1(self, other)
  self.hud_layout_apply(self, other)
  if gm.bool(global.title_open) then
    self.hud_pause_items = self.hud_pause_items_title
  else
    self.hud_pause_items = self.hud_pause_items_game
  end
  water_counter_step(self, other)
  if false and gm.keyboard_check_pressed(gm.vk_f1) then
    global.debug_open = not gm.bool(global.debug_open)
  end
  if gm.keyboard_check_pressed(gm.vk_escape) and not gm.bool(tasks_popup_open(self, other))
      and not gm.bool(cine_blocking(self, other)) then
    if self.hud_slot_mode ~= "" then
      if self.hud_slot_naming >= 0 then
        self.hud_slot_naming = -1
      else
        self.hud_slot_close(self, other)
      end
    elseif gm.bool(self.hud_credits_open) then
      self.hud_credits_open = false
    elseif gm.bool(global.debug_open) then
      global.debug_open = false
    elseif gm.bool(global.logo_open) then
      global.logo_open = false
    elseif gm.bool(global.menu_open) then
      global.menu_open = false
    elseif self.hud_place_type >= 0 or gm.bool(self.hud_build_open) then
      self.hud_build_close(self, other)
    elseif gm.bool(global.pause_open) or not gm.bool(global.title_open) then
      global.pause_open = not gm.bool(global.pause_open)
    end
  end
  if not gm.bool(global.title_open) and global.title_alpha > 0 then
    global.title_alpha = gm.max(0, global.title_alpha - 0.03333333333333333)
  end
  local mx = gm.device_mouse_x_to_gui(0)
  local my = gm.device_mouse_y_to_gui(0)
  local click = gm.mouse_check_button_pressed(gm.mb_left)
  local rclick = gm.mouse_check_button_pressed(gm.mb_right)
  global.right_click_used = false
  local upg_rect = self.hud_upgrades_button_rect(self, other)
  self.hud_hover_id = ""
  self.hud_pause_hover = -1
  self.hud_title_hover = -1
  local hb = self.hud_build
  self.hud_build_btn_hover = gm.point_distance(mx, my, hb.cx, hb.cy) <= hb.r
  self.hud_upgrade_btn_hover = self.hud_upgrades_button_over(self, other, mx, my)
  self.hud_build_btn_press = gm.max(0, self.hud_build_btn_press - 1)
  local i = 0
  while i < 3 do
    self.hud_cam_press[i] = gm.max(0, self.hud_cam_press[i] - 1)
    i = i + 1
  end
  if gm.bool(global.hud_hidden) or gm.bool(global.title_open) or gm.bool(global.intro_active)
      or gm.bool(ui_blocking(self, other)) then
    self.hud_cam_hover = -1
  else
    self.hud_cam_hover = self.hud_cam_pick(self, other, mx, my)
  end
  local wr = self.hud_wind_tab_rect(self, other)
  local wh = false
  if not gm.bool(ui_blocking(self, other)) then
    wh = gm.bool(gm.point_in_rectangle(mx, my, wr.x1, wr.y1, wr.x2, wr.y2))
  end
  self.hud_wind_hover = wh
  local pr = self.hud_points_tab_rect(self, other)
  local ph = false
  if not gm.bool(ui_blocking(self, other)) then
    ph = gm.bool(gm.point_in_rectangle(mx, my, pr.x1, pr.y1, pr.x2, pr.y2))
  end
  self.hud_points_hover = ph
  self.hud_upgrade_btn_press = gm.max(0, self.hud_upgrade_btn_press - 1)
  local tasks_hit = tasks_mouse_step(self, other, mx, my)
  if not gm.bool(global.title_open) and not gm.bool(global.intro_active) and not gm.bool(ui_blocking(self, other)) then
    global.animal_marker_hover = animal_marker_at(self, other, mx, my)
  else
    global.animal_marker_hover = gm.undefined
  end
  local marker = global.animal_marker_hover
  if not gm.is_undefined(marker) and gm.bool(click) then
    camera_fly_to_marker(self, other, marker)
  end
  local capture
  if gm.bool(ui_blocking(self, other)) or gm.bool(tasks_hit) then
    capture = true
  elseif not gm.bool(global.hud_hidden) and gm.bool(self.hud_upgrade_btn_hover) then
    capture = true
  elseif self.hud_cam_hover >= 0 then
    capture = true
  elseif not gm.is_undefined(marker) then
    capture = true
  elseif gm.bool(global.hud_hidden) then
    capture = false
  elseif self.hud_place_type >= 0 then
    capture = true
  else
    capture = gm.bool(self.hud_build_mouse_over(self, other, mx, my))
  end
  global.ui_capture_mouse = capture
  cine_step(self, other)
  if gm.bool(cine_blocking(self, other)) and not gm.bool(tasks_popup_open(self, other)) then
    return
  end
  if gm.bool(tasks_popup_open(self, other)) then
    tasks_popup_step(self, other, mx, my, click)
    return
  end
  if gm.bool(global.pause_open) then
    if gm.bool(gm.mouse_check_button(gm.mb_left)) then
      i = 0
      while i < gm.array_length(self.hud_volume_channels) do
        local r = self.hud_volume_slider_rect(self, other, i)
        if gm.bool(click) then
          if gm.bool(gm.point_in_rectangle(mx, my, r.x1 - 12, r.y1 - 12, r.x2 + 12, r.y2 + 12)) then
            self.hud_volume_drag = i
          else
            self.hud_volume_drag = self.hud_volume_drag
          end
        end
        if self.hud_volume_drag == i then
          audio_set_volume(self, other, self.hud_volume_channels[i], (mx - r.x1) / gm.max(1, r.x2 - r.x1))
        end
        i = i + 1
      end
    else
      if self.hud_volume_drag ~= -1 then
        audio_save(self, other)
      end
      self.hud_volume_drag = -1
    end
    if self.hud_volume_drag ~= -1 then
    else
      local found = false
      i = 0
      while i < gm.array_length(self.hud_pause_items) do
        local r = self.hud_pause_button_rect(self, other, i)
        if gm.bool(gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)) then
          found = true
          break
        end
        i = i + 1
      end
      if found and not gm.bool(self.hud_pause_item_disabled(self, other, i)) then
        self.hud_pause_hover = i
        if gm.bool(rclick) and self.hud_pause_items[i] == "window_size" then
          display_cycle_size(self, other, -1)
        end
        if gm.bool(rclick) and self.hud_pause_items[i] == "language" then
          locale_cycle(self, other, -1)
        end
        if gm.bool(click) then
          local item = self.hud_pause_items[i]
          if item == "resume" then
            global.pause_open = false
          elseif item == "display_mode" then
            display_set_fullscreen(self, other, not gm.bool(global.display_fullscreen))
          elseif item == "window_size" then
            display_cycle_size(self, other, 1)
          elseif item == "black_metal" then
            global.bw_mode = not gm.bool(global.bw_mode)
          elseif item == "units" then
            global.units_imperial = not gm.bool(global.units_imperial)
          elseif item == "language" then
            locale_cycle(self, other, 1)
          elseif item == "main_menu" then
            audio_save(self, other)
            save_game(self, other)
            gm.game_restart()
          elseif item == "quit" then
            gm.game_end()
          end
        end
      end
    end
    return
  end
  if gm.bool(global.debug_open) then
    return
  end
  if gm.bool(global.title_open) and self.hud_slot_mode ~= "" then
    self.hud_slot_hover = -1
    if self.hud_slot_naming >= 0 then
      if gm.string_length(gm.keyboard_string) > 16 then
        gm.keyboard_string = gm.string_copy(gm.keyboard_string, 1, 16)
      end
      if gm.keyboard_check_pressed(gm.vk_enter) then
        local name = gm.string_trim(gm.keyboard_string)
        if name == "" then
          name = loc(self, other, "slot_default_name") .. " " .. gm.string(self.hud_slot_naming + 1)
        end
        save_start_new(self, other, self.hud_slot_naming, name)
        game_mode_start(self, other, self.hud_new_mode)
        self.hud_slot_close(self, other)
        global.title_open = false
      end
      return
    end
    i = 0
    local found = false
    while i <= 3 do
      local r = self.hud_slot_button_rect(self, other, i)
      if gm.bool(gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)) then
        found = true
        break
      end
      i = i + 1
    end
    if not found then
      return
    end
    if i < 3 and self.hud_slot_mode == "load" and gm.is_undefined(self.hud_slot_metas[i]) then
      return
    end
    self.hud_slot_hover = i
    if gm.bool(click) then
      if i == 3 then
        self.hud_slot_close(self, other)
      elseif self.hud_slot_mode == "new" then
        self.hud_slot_naming = i
        gm.keyboard_string = ""
        return
      elseif gm.bool(load_game(self, other, i)) then
        self.hud_slot_close(self, other)
      end
    end
    return
  end
  if gm.bool(global.title_open) and gm.bool(self.hud_credits_open) then
    local r = self.hud_credits_back_rect(self, other)
    self.hud_credits_back_hover = gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)
    if gm.bool(click) then
      self.hud_credits_open = false
    end
    return
  end
  if gm.bool(global.title_open) then
    i = 0
    local found = false
    while i < gm.array_length(self.hud_title_items) do
      local r = self.hud_title_button_rect(self, other, i)
      if gm.bool(gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)) then
        found = true
        break
      end
      i = i + 1
    end
    if found and not gm.bool(self.hud_title_item_disabled(self, other, i)) then
      self.hud_title_hover = i
      if gm.bool(click) then
        local item = self.hud_title_items[i]
        if item == "exploration" then
          self.hud_new_mode = "exploration"
          self.hud_slot_open(self, other, "new")
        elseif item == "survival" then
          self.hud_new_mode = "survival"
          self.hud_slot_open(self, other, "new")
        elseif item == "load_game" then
          self.hud_slot_open(self, other, "load")
        elseif item == "settings" then
          global.pause_open = true
        elseif item == "credits" then
          self.hud_credits_open = true
          self.hud_credits_back_hover = false
        elseif item == "exit" then
          gm.game_end()
        end
      end
    end
    return
  end
  if gm.bool(global.logo_open) then
    logo_step(self, other)
    local cr = self.hud_close_button_rect(self, other)
    if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, cr.x1, cr.y1, cr.x2, cr.y2)) then
      global.logo_open = false
      return
    end
    local typing = global.logo_phase == "type"
    local rr = self.hud_logo_reroll_rect(self, other)
    if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, rr.x1, rr.y1, rr.x2, rr.y2)) then
      if typing then
        if gm.bool(logo_has_letters(self, other)) then
          logo_grow_start(self, other)
          return
        end
      else
        logo_reroll(self, other)
      end
      return
    end
    local lr = self.hud_logo_clear_rect(self, other)
    if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, lr.x1, lr.y1, lr.x2, lr.y2)) then
      if typing then
        gm.keyboard_string = ""
        global.logo_text = ""
        return
      end
      logo_edit(self, other)
    end
    return
  end
  if not gm.bool(global.menu_open) then
    self.hud_tree_fitted = false
    self.hud_tree_drag = false
    if not gm.bool(global.hud_hidden) and gm.bool(click) and gm.bool(self.hud_upgrade_btn_hover) then
      self.hud_upgrade_btn_press = self.hud_ui.press_frames
      self.hud_build_close(self, other)
      global.menu_open = true
      return
    end
    local b = self.hud_build
    local dir
    if gm.bool(self.hud_build_open) then
      dir = 1
    else
      dir = -1
    end
    self.hud_build_slide = gm.clamp(self.hud_build_slide + dir / gm.max(1, b.slide_frames), 0, 1)
    self.hud_build_hover = -1
    if gm.bool(global.hud_hidden) or gm.bool(global.intro_active) then
      self.hud_build_close(self, other)
      return
    end
    if gm.mouse_check_button_pressed(gm.mb_right) and (self.hud_place_type >= 0 or gm.bool(self.hud_build_open)) then
      global.right_click_used = true
      self.hud_build_close(self, other)
      return
    end
    if gm.bool(click) and self.hud_cam_hover >= 0 then
      local ch = self.hud_cam_hover
      self.hud_cam_press[ch] = self.hud_ui.press_frames
      if ch == 0 then
        camera_jump_edge(self, other, -1)
      elseif ch == 1 then
        camera_jump_origin(self, other)
      elseif ch == 2 then
        camera_jump_edge(self, other, 1)
      end
      return
    end
    local on_circle = gm.point_distance(mx, my, b.cx, b.cy) <= b.r
    local types = self.hud_build_types(self, other)
    local all = self.hud_build_all_types(self, other)
    local k = 0
    local key_found = false
    while k < gm.min(8, gm.array_length(all)) do
      if gm.keyboard_check_pressed(gm.ord(gm.string(k + 1))) or gm.keyboard_check_pressed(97 + k) then
        key_found = true
        break
      end
      k = k + 1
    end
    if key_found then
      local t = all[k]
      if gm.bool(mushroom_type_unlocked(self, other, t)) then
        self.hud_build_open = true
        if gm.bool(mushroom_type_affordable(self, other, t)) then
          self.hud_place_type = t
          self.hud_place_at = gm.undefined
        end
        return
      end
    end
    if gm.bool(self.hud_build_open) and self.hud_place_type < 0 and self.hud_build_slide > 0.9 then
      i = 0
      while i < gm.array_length(types) do
        local r = self.hud_build_slot_rect(self, other, i)
        if gm.bool(gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)) then
          self.hud_build_hover = i
        end
        i = i + 1
      end
    end
    if self.hud_place_type >= 0 then
      self.hud_place_at = gm.undefined
      gm.with(self, obj_mycelium_controller, function(self, other)
        other.hud_place_at = self.myc_place_check(self, other, other.hud_place_type, gm.mouse_x, gm.mouse_y)
      end)
      if gm.bool(click) and not on_circle and not gm.bool(self.hud_build_mouse_over(self, other, mx, my)) then
        local ok = false
        local t = self.hud_place_type
        gm.with(self, obj_mycelium_controller, function(self, other)
          ok = self.myc_place_mushroom(self, other, t, gm.mouse_x, gm.mouse_y)
        end)
        if gm.bool(ok) then
          self.hud_build_close(self, other)
        end
      elseif gm.bool(click) and on_circle then
        self.hud_build_btn_press = self.hud_ui.press_frames
        self.hud_build_close(self, other)
      end
      return
    end
    if gm.bool(click) and on_circle then
      self.hud_build_btn_press = self.hud_ui.press_frames
      self.hud_build_open = not gm.bool(self.hud_build_open)
    elseif gm.bool(click) and self.hud_build_hover >= 0 then
      local t = types[self.hud_build_hover]
      if gm.bool(mushroom_type_affordable(self, other, t)) then
        self.hud_place_type = t
      end
    end
    return
  end
  if not gm.bool(self.hud_tree_fitted) or self.hud_tree_fit_w ~= gm.display_get_gui_width()
      or self.hud_tree_fit_h ~= gm.display_get_gui_height() then
    self.hud_tree_fit(self, other)
  end
  local cr = self.hud_close_button_rect(self, other)
  if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, cr.x1, cr.y1, cr.x2, cr.y2)) then
    global.menu_open = false
    return
  end
  local lb = self.hud_logo_button_rect(self, other)
  if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, lb.x1, lb.y1, lb.x2, lb.y2)) then
    logo_open(self, other)
    return
  end
  local rc = self.hud_recentre_button_rect(self, other)
  if gm.keyboard_check_pressed(82)
      or (gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, rc.x1, rc.y1, rc.x2, rc.y2))) then
    self.hud_tree_fit(self, other)
    return
  end
  self.hud_hover_id = self.hud_tree_pick(self, other, mx, my)
  local c = self.hud_tree_canvas(self, other)
  local wheel = gm.real(gm.mouse_wheel_up()) - gm.real(gm.mouse_wheel_down())
  if wheel ~= 0 and gm.bool(gm.point_in_rectangle(mx, my, c.x1, c.y1, c.x2, c.y2)) then
    local ccx = (c.x1 + c.x2) * 0.5
    local ccy = (c.y1 + c.y2) * 0.5
    local wx = self.hud_tree_cx + (mx - ccx) / self.hud_tree_zoom
    local wy = self.hud_tree_cy + (my - ccy) / self.hud_tree_zoom
    self.hud_tree_zoom = gm.clamp(self.hud_tree_zoom * (1 + wheel * vis(self, other, "tree_zoom_step")),
      self.hud_tree_zoom_lo, vis(self, other, "tree_zoom_max"))
    self.hud_tree_cx = wx - (mx - ccx) / self.hud_tree_zoom
    self.hud_tree_cy = wy - (my - ccy) / self.hud_tree_zoom
    self.hud_tree_clamp_view(self, other)
  end
  if gm.bool(click) and gm.bool(gm.point_in_rectangle(mx, my, c.x1, c.y1, c.x2, c.y2)) then
    self.hud_tree_drag = true
    self.hud_tree_dragged = false
    self.hud_tree_drag_x = mx
    self.hud_tree_drag_y = my
    self.hud_tree_press_id = self.hud_hover_id
  end
  if gm.bool(self.hud_tree_drag) and gm.bool(gm.mouse_check_button(gm.mb_left)) then
    local dx = mx - self.hud_tree_drag_x
    local dy = my - self.hud_tree_drag_y
    if math.abs(dy) + math.abs(dx) > 6 then
      self.hud_tree_dragged = true
    end
    if gm.bool(self.hud_tree_dragged) then
      self.hud_tree_cx = self.hud_tree_cx - dx / self.hud_tree_zoom
      self.hud_tree_cy = self.hud_tree_cy - dy / self.hud_tree_zoom
      self.hud_tree_clamp_view(self, other)
    end
    self.hud_tree_drag_x = mx
    self.hud_tree_drag_y = my
  end
  if gm.bool(gm.mouse_check_button_released(gm.mb_left)) then
    if gm.bool(self.hud_tree_drag) and not gm.bool(self.hud_tree_dragged) and self.hud_tree_press_id ~= ""
        and self.hud_tree_press_id == self.hud_hover_id then
      upg_buy(self, other, self.hud_tree_press_id)
    end
    self.hud_tree_drag = false
    self.hud_tree_press_id = ""
  end
end

-- gml_Script____struct___459__gml_Object_obj_hud_Draw_75 @ 0x140627350
function ___struct___459__gml_Object_obj_hud_Draw_75(self, other, text, colour)
  if text == nil then text = gm.undefined end
  if colour == nil then colour = gm.undefined end
  self.text = text
  self.colour = colour
  self.scale = 1
end

-- gml_Script____struct___460__gml_Object_obj_hud_Draw_75 @ 0x140627550
function ___struct___460__gml_Object_obj_hud_Draw_75(self, other, text, colour)
  if text == nil then text = gm.undefined end
  if colour == nil then colour = gm.undefined end
  self.text = text
  self.colour = colour
  self.scale = 0.6
end

-- gml_Script____struct___461__gml_Object_obj_hud_Draw_75 @ 0x140627750
function ___struct___461__gml_Object_obj_hud_Draw_75(self, other, text, colour)
  if text == nil then text = gm.undefined end
  if colour == nil then colour = gm.undefined end
  self.text = text
  self.colour = colour
  self.scale = 0.6
end

-- gml_Script____struct___462__gml_Object_obj_hud_Draw_75 @ 0x140627950
function ___struct___462__gml_Object_obj_hud_Draw_75(self, other, text, colour)
  if text == nil then text = gm.undefined end
  if colour == nil then colour = gm.undefined end
  self.text = text
  self.colour = colour
  self.scale = 0.8
end

-- gml_Script____struct___463__gml_Object_obj_hud_Draw_75 @ 0x140627b50
function ___struct___463__gml_Object_obj_hud_Draw_75(self, other, text, colour)
  if text == nil then text = gm.undefined end
  if colour == nil then colour = gm.undefined end
  self.text = text
  self.colour = colour
  self.scale = 0.6
end

-- gml_Script____struct___464__gml_Object_obj_hud_Draw_75 @ 0x140627d50
function ___struct___464__gml_Object_obj_hud_Draw_75(self, other, tag, key, value)
  if tag == nil then tag = gm.undefined end
  if key == nil then key = gm.undefined end
  if value == nil then value = gm.undefined end
  self.tag = tag
  self.key = key
  self.value = value
end

-- gml_Script____struct___465__gml_Object_obj_hud_Draw_75 @ 0x140627f90
function ___struct___465__gml_Object_obj_hud_Draw_75(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.x = x
  self.y = y
end

-- gml_Object_obj_hud_Draw_75 @ 0x140628190
function gml_Object_obj_hud_Draw_75(self, other)
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_valign(gm.fa_top)
  gm.draw_set_font(ui_font_small(self, other))
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  local cm = mycelium_cm(self, other)
  local step_cm = bal(self, other, "xp_length_step_cm")
  local xp_base = cm - global.mycelium_points_awarded * step_cm
  local i
  if not gm.bool(global.hud_hidden) then
    local u = self.hud_ui
    gm.draw_sprite_ext(UI_PIPKA, 0, u.wind_tab.x, u.wind_tab.y, u.wind_tab.scale, u.wind_tab.scale, 0, gm.c_white, 1)
    gm.draw_sprite_ext(UI_PIPKA, 0, u.upgrade_tab.x, u.upgrade_tab.y, u.upgrade_tab.scale, u.upgrade_tab.scale, 0, gm.c_white, 1)
    gm.draw_sprite_ext(UPPER_UI, 0, u.bar.x, u.bar.y, u.bar.scale, u.bar.scale, 0, gm.c_white, 1)
    gm.draw_sprite_ext(UPGRADE, 0, u.upgrade_icon.x, u.upgrade_icon.y, u.upgrade_icon.scale, u.upgrade_icon.scale, 0, gm.c_white, 1)
    self.hud_draw_button(self, other, UPGRADE_CALM, UPGRADE_HOVER, u.upgrade_button.x, u.upgrade_button.y,
      self.hud_upgrade_btn_hover, self.hud_upgrade_btn_press)
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_valign(gm.fa_middle)
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_colour(self.hud_color_ink)
    local len_val
    if gm.bool(global.units_imperial) then
      len_val = cm / 2.54
    else
      len_val = cm
    end
    local len_str = gm.string(gm.round(len_val))
    local unit_key
    if gm.bool(global.units_imperial) then
      unit_key = "hud_in"
    else
      unit_key = "hud_cm"
    end
    local unit_str = loc(self, other, unit_key)
    local len_w = gm.string_width(len_str) * u.length_scale
    local unit_w = gm.string_width(unit_str) * u.unit_scale
    local len_x = u.length_x - (len_w + u.unit_gap + unit_w) * 0.5
    gm.draw_set_halign(gm.fa_left)
    self.hud_text(self, other, len_x, u.text_y, len_str, u.length_scale)
    gm.draw_set_colour(self.hud_color_dim)
    self.hud_text(self, other, len_x + len_w + u.unit_gap, u.text_y + 4, unit_str, u.unit_scale)
    gm.draw_set_colour(self.hud_color_ink)
    local fi = u.food_icon
    local wi = u.water_icon
    self.hud_icon_number(self, other, apple_bite, fi.x + gm.sprite_get_width(apple_bite) * fi.scale * 0.5, u.text_y,
      fi.scale, gm.string(global.food), u.side_scale)
    self.hud_icon_number(self, other, water65, wi.x + gm.sprite_get_width(water65) * wi.scale * 0.5, u.text_y,
      wi.scale, gm.string(gm.round(water_shown(self, other))), u.side_scale)
    cave_perk_icons_draw(self, other, vis(self, other, "cave_icons_x") + global.ui_ox, vis(self, other, "cave_icons_y"))
    tasks_draw(self, other, self.hud_color_ink, self.hud_color_dim, self.hud_color_good, self.hud_color_panel)
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_valign(gm.fa_middle)
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_colour(self.hud_color_ink)
    if gm.instance_exists(obj_forest_controller) then
      local fc = gm.instance_find(obj_forest_controller, 0)
      cave_popups_draw(self, other, fc.camera_view_x, fc.camera_view_y)
      if not gm.bool(global.intro_active) then
        animal_markers_draw(self, other, fc.camera_view_x, fc.camera_view_y, fc.camera_width, fc.camera_height)
      end
      if false and gm.keyboard_check(67) and self.hud_slot_naming == -1 then
        caves_debug_draw(self, other, fc.camera_view_x, fc.camera_view_y, fc.camera_width, fc.camera_height)
      end
    end
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_valign(gm.fa_middle)
    gm.draw_set_colour(self.hud_color_ink)
    gm.draw_set_halign(gm.fa_left)
    if global.upgrade_points > 0 then
      gm.draw_set_colour(self.hud_color_good)
    else
      gm.draw_set_colour(self.hud_color_ink)
    end
    self.hud_text(self, other, u.points_x, u.points_y, gm.string(global.upgrade_points), u.points_scale)
    local arrows = ""
    i = 0
    while i < global.wind_speed do
      if global.wind_dir < 0 then
        arrows = arrows .. "<"
      else
        arrows = arrows .. ">"
      end
      i = i + 1
    end
    gm.draw_set_colour(self.hud_color_ink)
    gm.draw_set_halign(gm.fa_center)
    self.hud_text(self, other, u.wind_x, u.wind_y, arrows, u.wind_scale)
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
  end
  if not gm.bool(global.hud_hidden) and not gm.bool(global.title_open) and not gm.bool(global.intro_active) then
    local b = self.hud_build
    local vx, vy
    if gm.instance_exists(obj_forest_controller) then
      vx = gm.instance_find(obj_forest_controller, 0).camera_view_x
    else
      vx = 0
    end
    if gm.instance_exists(obj_forest_controller) then
      vy = gm.instance_find(obj_forest_controller, 0).camera_view_y
    else
      vy = 0
    end
    if self.hud_place_type >= 0 and not gm.is_undefined(self.hud_place_at) then
      local spr = mushroom_type_sprite(self, other, self.hud_place_type)
      local cw = mushroom_cell_width(self, other, spr)
      local col
      if gm.bool(self.hud_place_at.ok) then
        col = 0x78e678
      else
        col = 0x505aeb
      end
      local variant
      if self.hud_place_type == 7 then
        local face = self.hud_place_at.face
        if gm.is_undefined(face) then
          face = 1
        end
        variant = consumptor_variant(self, other, face)
      else
        variant = 0
      end
      local left = mushroom_cell_left(self, other, spr, variant)
      local pa = self.hud_place_at
      gm.draw_sprite_part_ext(spr, 0, left, 0, cw, gm.sprite_get_height(spr),
        pa.x - vx - gm.sprite_get_xoffset(spr), pa.y - vy - gm.sprite_get_yoffset(spr), 1, 1, col, 0.75)
    end
    if self.hud_build_slide > 0 then
      local e = 1 - gm.power(1 - self.hud_build_slide, 3)
      local off = -(b.bar_right - b.cx) * (1 - e)
      local types = self.hud_build_types(self, other)
      local bx = b.bar_x + off
      local cut = gm.max(0, b.cx - bx)
      if cut < gm.sprite_get_width(LOWER_UI_BAR) then
        gm.draw_sprite_part(LOWER_UI_BAR, 0, cut, 0, gm.sprite_get_width(LOWER_UI_BAR) - cut,
          gm.sprite_get_height(LOWER_UI_BAR), bx + cut, b.bar_y)
      end
      gm.draw_set_font(ui_font(self, other))
      gm.draw_set_valign(gm.fa_middle)
      if self.hud_build_slide < 1 then
        types = gm.array()
      end
      if gm.array_length(types) == 0 and self.hud_build_slide >= 1 then
        gm.draw_set_halign(gm.fa_left)
        gm.draw_set_colour(self.hud_color_dim)
        self.hud_text(self, other, b.slot_x0 - 40 + off, b.slot_y, loc(self, other, "build_none"), 0.38)
      end
      local step = self.hud_build_slot_step(self, other)
      i = 0
      while i < gm.array_length(types) do
        local t = types[i]
        local x = b.slot_x0 + i * step + off
        local cost = mushroom_type_cost(self, other, t)
        local wcost = mushroom_type_water_cost(self, other, t)
        local affordable = mushroom_type_affordable(self, other, t)
        local can_food = global.food >= cost
        local price_y
        if wcost > 0 then
          price_y = b.slot_y - b.price_line * 0.5
        else
          price_y = b.slot_y
        end
        if self.hud_build_hover == i then
          local gcol
          if gm.bool(affordable) then
            gcol = self.hud_color_good
          else
            gcol = self.hud_color_bad
          end
          tree_draw_glow(self, other, x, b.slot_y, b.icon * 0.5, gcol, 0.35)
        end
        local isc
        if t == 7 then
          isc = vis(self, other, "build_consumptor_icon_scale")
        else
          isc = 1
        end
        local isz = b.icon * isc
        local ialpha
        if gm.bool(affordable) then
          ialpha = 1
        else
          ialpha = 0.45
        end
        tree_draw_icon(self, other, mushroom_type_sprite(self, other, t), x, b.slot_y, isz, ialpha, true, false)
        local key = self.hud_build_key_of(self, other, t)
        if key > 0 then
          local kr = vis(self, other, "build_key_badge_radius")
          local ky = b.slot_y - b.icon * 0.5 - kr - vis(self, other, "build_key_badge_gap")
          gm.draw_set_colour(0x121612)
          gm.draw_set_alpha(0.92)
          gm.draw_circle(x, ky, kr, false)
          gm.draw_set_alpha(1)
          gm.draw_set_colour(self.hud_color_ink)
          gm.draw_circle(x, ky, kr, true)
          gm.draw_set_font(ui_font_small(self, other))
          gm.draw_set_halign(gm.fa_center)
          gm.draw_set_valign(gm.fa_middle)
          self.hud_text(self, other, x, ky, gm.string(key), vis(self, other, "build_key_badge_text"))
          gm.draw_set_font(ui_font(self, other))
          gm.draw_set_valign(gm.fa_middle)
        end
        local px = x + b.icon * 0.5 + b.price_gap
        local bs = b.bite_size / gm.max(1, gm.sprite_get_width(apple_bite))
        local falpha
        if can_food then
          falpha = 1
        else
          falpha = 0.5
        end
        gm.draw_sprite_ext(apple_bite, 0, px, price_y - gm.sprite_get_height(apple_bite) * bs * 0.5, bs, bs, 0,
          gm.c_white, falpha)
        gm.draw_set_halign(gm.fa_left)
        if can_food then
          gm.draw_set_colour(self.hud_color_ink)
        else
          gm.draw_set_colour(self.hud_color_bad)
        end
        self.hud_text(self, other, px + b.bite_size + 4, price_y, gm.string(cost), b.price_scale)
        if wcost > 0 then
          local can_w = water_total(self, other) >= wcost
          local wy = price_y + b.price_line
          local ws = b.bite_size / gm.max(1, gm.sprite_get_width(water65))
          local walpha
          if can_w then
            walpha = 1
          else
            walpha = 0.5
          end
          gm.draw_sprite_ext(water65, 0,
            px + b.bite_size * 0.5 + (gm.sprite_get_xoffset(water65) - gm.sprite_get_width(water65) * 0.5) * ws,
            wy + (gm.sprite_get_yoffset(water65) - gm.sprite_get_height(water65) * 0.5) * ws,
            ws, ws, 0, gm.c_white, walpha)
          if can_w then
            gm.draw_set_colour(self.hud_color_ink)
          else
            gm.draw_set_colour(self.hud_color_bad)
          end
          self.hud_text(self, other, px + b.bite_size + 4, wy, gm.string(wcost), b.price_scale)
        end
        i = i + 1
      end
      gm.draw_set_font(ui_font_small(self, other))
      gm.draw_set_halign(gm.fa_left)
      gm.draw_set_valign(gm.fa_top)
    end
    self.hud_draw_button(self, other, SHROOM_CALM, SHROOM_HOVER, b.circle_x, b.circle_y,
      self.hud_build_btn_hover, self.hud_build_btn_press)
    self.hud_cam_draw(self, other)
  end
  if global.cheat_timer > 0 then
    gm.draw_set_colour(self.hud_color_good)
    gm.draw_set_alpha(gm.clamp(global.cheat_timer / 45, 0, 1))
    self.hud_text(self, other, 40, 204, global.cheat_text, 1.4)
    gm.draw_set_alpha(1)
  end
  tutorial_draw(self, other, self.hud_upgrades_button_rect(self, other), self.hud_color_ink)
  if global.title_alpha > 0 then
    local vig = gm.clamp(vis(self, other, "menu_vignette_strength"), 0, 1) * global.title_alpha
    local shape = gm.max(1, vis(self, other, "menu_vignette_shape"))
    local vstart = vis(self, other, "menu_vignette_start")
    local vwidth = gm.max(0.01, vis(self, other, "menu_vignette_width"))
    local vpow = gm.max(0.01, vis(self, other, "menu_vignette_power"))
    local cols = 48
    local rows = 27
    local grid = gm.array_create((rows + 1) * (cols + 1), 0)
    local gy = 0
    while gy <= rows do
      local gx = 0
      while gx <= cols do
        local nx = math.abs(gx / cols * 2 - 1)
        local ny = math.abs(gy / rows * 2 - 1)
        local d = gm.power(gm.power(nx, shape) + gm.power(ny, shape), 1 / shape)
        local k = gm.clamp((d - vstart) / vwidth, 0, 1)
        k = k * k * (3 - 2 * k)
        grid[gy * (cols + 1) + gx] = vig * gm.power(k, vpow)
        gx = gx + 1
      end
      gy = gy + 1
    end
    local cw = gw / cols
    local ch = gh / rows
    gy = 0
    while gy < rows do
      gm.draw_primitive_begin(gm.pr_trianglelist)
      local gx = 0
      while gx < cols do
        local a00 = grid[gy * (cols + 1) + gx]
        local a10 = grid[gy * (cols + 1) + gx + 1]
        local a01 = grid[(gy + 1) * (cols + 1) + gx]
        local a11 = grid[(gy + 1) * (cols + 1) + gx + 1]
        if a00 + a10 + a01 + a11 <= 0 then
        else
          local x0 = gx * cw
          local y0 = gy * ch
          local x1 = x0 + cw
          local y1 = y0 + ch
          gm.draw_vertex_colour(x0, y0, 0, a00)
          gm.draw_vertex_colour(x1, y0, 0, a10)
          gm.draw_vertex_colour(x1, y1, 0, a11)
          gm.draw_vertex_colour(x0, y0, 0, a00)
          gm.draw_vertex_colour(x1, y1, 0, a11)
          gm.draw_vertex_colour(x0, y1, 0, a01)
        end
        gx = gx + 1
      end
      gm.draw_primitive_end()
      gy = gy + 1
    end
  end
  local ta = global.title_alpha
  if ta > 0 and not gm.bool(global.pause_open) and self.hud_slot_mode ~= "" then
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_valign(gm.fa_middle)
    local r0 = self.hud_slot_button_rect(self, other, 0)
    local scx = (r0.x1 + r0.x2) * 0.5
    local tkey
    if self.hud_slot_mode == "new" then
      tkey = "slot_title_new"
    else
      tkey = "slot_title_load"
    end
    local title = loc(self, other, tkey)
    if self.hud_slot_mode == "new" then
      title = title .. ("  -  " .. gm.string_upper(loc(self, other, "mode_" .. self.hud_new_mode)))
    end
    local lines = gm.array()
    local naming = self.hud_slot_naming
    if naming >= 0 then
      local cursor
      if gm.mod(gm.div(gm.current_time, 500), 2) == 0 then
        cursor = "_"
      else
        cursor = " "
      end
      title = loc(self, other, "slot_name_prompt")
      gm.array_push(lines, gm.new(___struct___459__gml_Object_obj_hud_Draw_75, gm.keyboard_string .. cursor,
        self.hud_color_ink))
      if not gm.is_undefined(self.hud_slot_metas[self.hud_slot_naming]) then
        gm.array_push(lines, gm.new(___struct___460__gml_Object_obj_hud_Draw_75,
          loc(self, other, "slot_overwrites") .. " " .. gm.string(self.hud_slot_metas[self.hud_slot_naming].name),
          self.hud_color_bad))
      end
      gm.array_push(lines, gm.new(___struct___461__gml_Object_obj_hud_Draw_75, loc(self, other, "slot_name_hint"),
        self.hud_color_dim))
    end
    local tsc = self.hud_fit(self, other, title, 0.8, gw - 80, 0)
    gm.draw_set_colour(0)
    gm.draw_set_alpha(ta * 0.6)
    gm.draw_text_transformed(scx + 3, r0.y1 - 40 + 3, title, tsc, tsc, 0)
    gm.draw_set_colour(self.hud_color_gold)
    gm.draw_set_alpha(ta)
    gm.draw_text_transformed(scx, r0.y1 - 40, title, tsc, tsc, 0)
    if naming >= 0 then
      i = 0
      while i < gm.array_length(lines) do
        local r = self.hud_slot_button_rect(self, other, i)
        local cy = (r.y1 + r.y2) * 0.5
        lines[i].scale = self.hud_fit(self, other, lines[i].text, lines[i].scale, r.x2 - r.x1 - 24, 0)
        gm.draw_set_colour(0)
        gm.draw_set_alpha(ta * 0.6)
        gm.draw_text_transformed(scx + 3, cy + 3, lines[i].text, lines[i].scale, lines[i].scale, 0)
        gm.draw_set_colour(lines[i].colour)
        gm.draw_set_alpha(ta)
        gm.draw_text_transformed(scx, cy, lines[i].text, lines[i].scale, lines[i].scale, 0)
        i = i + 1
      end
    else
      i = 0
      while i <= 3 do
        local r = self.hud_slot_button_rect(self, other, i)
        local label
        if i == 3 then
          label = loc(self, other, "slot_back")
        else
          label = self.hud_slot_label(self, other, i)
        end
        local empty = false
        if i < 3 then
          empty = gm.is_undefined(self.hud_slot_metas[i])
        end
        local cx = (r.x1 + r.x2) * 0.5
        local cy = (r.y1 + r.y2) * 0.5
        local col
        if self.hud_slot_hover == i then
          col = self.hud_color_ink
        else
          col = self.hud_color_dim
        end
        if empty and self.hud_slot_mode == "load" then
          col = self.hud_color_deep
        end
        local sc = self.hud_fit(self, other, label, 0.7, r.x2 - r.x1 - 24, r.y2 - r.y1)
        local sub
        if i < 3 then
          sub = self.hud_slot_mode_label(self, other, i)
        else
          sub = ""
        end
        local ly
        if sub == "" then
          ly = cy
        else
          ly = cy - 9
        end
        gm.draw_set_colour(0)
        gm.draw_set_alpha(ta * 0.6)
        gm.draw_text_transformed(cx + 3, ly + 3, label, sc, sc, 0)
        gm.draw_set_colour(col)
        gm.draw_set_alpha(ta)
        gm.draw_text_transformed(cx, ly, label, sc, sc, 0)
        if sub ~= "" then
          local ssc = self.hud_fit(self, other, sub, 0.36, r.x2 - r.x1 - 24, 0)
          gm.draw_set_colour(self.hud_color_deep)
          gm.draw_text_transformed(cx, r.y2 - 13, sub, ssc, ssc, 0)
        end
        i = i + 1
      end
    end
    gm.draw_set_alpha(1)
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
  elseif ta > 0 and not gm.bool(global.pause_open) and gm.bool(self.hud_credits_open) then
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_valign(gm.fa_middle)
    local top = self.hud_credits_top(self, other)
    local ccx = gm.array(gw * 0.5 - 300, gw * 0.5 + 300)
    local c = 0
    while c < 2 do
      local y = top
      local col = self.hud_credits_columns[c]
      local g = 0
      while g < gm.array_length(col) do
        local lines = gm.array(gm.new(___struct___462__gml_Object_obj_hud_Draw_75, loc(self, other, col[g][0]),
          self.hud_color_gold))
        local names = col[g][1]
        local n = 0
        while n < gm.array_length(names) do
          gm.array_push(lines, gm.new(___struct___463__gml_Object_obj_hud_Draw_75, names[n], self.hud_color_ink))
          n = n + 1
        end
        local k = 0
        while k < gm.array_length(lines) do
          lines[k].scale = self.hud_fit(self, other, lines[k].text, lines[k].scale, 560, 0)
          gm.draw_set_colour(0)
          gm.draw_set_alpha(ta * 0.6)
          gm.draw_text_transformed(ccx[c] + 3, y + 3, lines[k].text, lines[k].scale, lines[k].scale, 0)
          gm.draw_set_colour(lines[k].colour)
          gm.draw_set_alpha(ta)
          gm.draw_text_transformed(ccx[c], y, lines[k].text, lines[k].scale, lines[k].scale, 0)
          if k == 0 then
            y = y + 52
          else
            y = y + 40
          end
          k = k + 1
        end
        y = y + 28
        g = g + 1
      end
      c = c + 1
    end
    local br = self.hud_credits_back_rect(self, other)
    local bcx = (br.x1 + br.x2) * 0.5
    local bcy = (br.y1 + br.y2) * 0.5
    gm.draw_set_colour(0)
    gm.draw_set_alpha(ta * 0.6)
    gm.draw_text_transformed(bcx + 3, bcy + 3, loc(self, other, "slot_back"), 0.7, 0.7, 0)
    if gm.bool(self.hud_credits_back_hover) then
      gm.draw_set_colour(self.hud_color_ink)
    else
      gm.draw_set_colour(self.hud_color_dim)
    end
    gm.draw_set_alpha(ta)
    gm.draw_text_transformed(bcx, bcy, loc(self, other, "slot_back"), 0.7, 0.7, 0)
    gm.draw_set_alpha(1)
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
  elseif ta > 0 and not gm.bool(global.pause_open) then
    gm.draw_set_font(ui_font(self, other))
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_valign(gm.fa_middle)
    i = 0
    while i < gm.array_length(self.hud_title_items) do
      local r = self.hud_title_button_rect(self, other, i)
      local label = loc(self, other, "title_" .. self.hud_title_items[i])
      local tx = (r.x1 + r.x2) * 0.5
      local ty = (r.y1 + r.y2) * 0.5
      local sc = self.hud_fit(self, other, label, 1, r.x2 - r.x1 - 24, r.y2 - r.y1)
      gm.draw_set_colour(0)
      gm.draw_set_alpha(ta * 0.6)
      gm.draw_text_transformed(tx + 3, ty + 3, label, sc, sc, 0)
      if gm.bool(self.hud_title_item_disabled(self, other, i)) then
        gm.draw_set_colour(self.hud_color_deep)
      elseif self.hud_title_hover == i then
        gm.draw_set_colour(self.hud_color_ink)
      else
        gm.draw_set_colour(self.hud_color_dim)
      end
      gm.draw_set_alpha(ta)
      gm.draw_text_transformed(tx, ty, label, sc, sc, 0)
      i = i + 1
    end
    local lr = self.hud_title_button_rect(self, other, gm.array_length(self.hud_title_items) - 1)
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_valign(gm.fa_top)
    gm.draw_set_colour(self.hud_color_dim)
    gm.draw_set_alpha(ta * 0.7)
    gm.draw_text_transformed((lr.x1 + lr.x2) * 0.5, lr.y2 + 12, "v1.02", 1.1, 1.1, 0)
    gm.draw_set_alpha(1)
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
  end
  local tip = ""
  if gm.bool(self.hud_wind_hover) then
    tip = loc(self, other, "wind_tooltip")
  elseif gm.bool(self.hud_points_hover) then
    tip = loc(self, other, "points_tooltip") .. "\n" .. length_text(self, other, gm.max(0, gm.ceil(step_cm - xp_base)))
      .. " " .. loc(self, other, "points_till_next")
  elseif gm.instance_exists(obj_forest_controller) and not gm.bool(ui_blocking(self, other)) then
    local fc = gm.instance_find(obj_forest_controller, 0)
    tip = cave_hover_tip(self, other, gm.device_mouse_x_to_gui(0), gm.device_mouse_y_to_gui(0),
      vis(self, other, "cave_icons_x") + global.ui_ox, vis(self, other, "cave_icons_y"), fc.camera_view_x, fc.camera_view_y)
  end
  if tip ~= "" and not gm.bool(global.hud_hidden) then
    local tsc = self.hud_fit(self, other, tip, 1.25, gw * 0.6, 0)
    local pad = 12
    local tw = gm.string_width(tip) * tsc + pad * 2
    local th = gm.string_height(tip) * tsc + pad * 2
    local tx = gm.clamp(gm.device_mouse_x_to_gui(0) + 20, 8, gw - tw - 8)
    local ty = gm.clamp(gm.device_mouse_y_to_gui(0) + 24, 8, gh - th - 8)
    gm.draw_set_halign(gm.fa_left)
    gm.draw_set_valign(gm.fa_top)
    gm.draw_set_colour(self.hud_color_panel)
    gm.draw_set_alpha(0.95)
    gm.draw_rectangle(tx, ty, tx + tw, ty + th, false)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(self.hud_color_deep)
    gm.draw_rectangle(tx, ty, tx + tw, ty + th, true)
    gm.draw_set_colour(self.hud_color_ink)
    self.hud_text(self, other, tx + pad, ty + pad, tip, tsc)
  end
  if gm.bool(global.debug_open) then
    gm.draw_set_colour(self.hud_color_panel)
    gm.draw_set_alpha(0.94)
    gm.draw_rectangle(0, 0, gw, gh, false)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(self.hud_color_ink)
    self.hud_text(self, other, 40, 28, loc(self, other, "debug_title"), 2)
    local groups = gm.array(gm.array("bal", global.balance), gm.array("vis", global.visual), gm.array("wld", global.worldgen))
    local rows = gm.array()
    local g = 0
    while g < gm.array_length(groups) do
      local src = groups[g][1]
      local names = gm.variable_struct_get_names(src)
      gm.array_sort(names, true)
      local k = 0
      while k < gm.array_length(names) do
        gm.array_push(rows, gm.new(___struct___464__gml_Object_obj_hud_Draw_75, groups[g][0], names[k],
          gm.variable_struct_get(src, names[k])))
        k = k + 1
      end
      g = g + 1
    end
    local row_h = 44
    local col_w = 460
    i = 0
    while i < gm.array_length(rows) do
      local row = rows[i]
      local col = gm.div(i, row_h)
      local ri = gm.mod(i, row_h)
      local x = 40 + col * col_w
      local y = 116 + ri * 21
      if x > gw - 120 then
        break
      end
      gm.draw_set_colour(self.hud_color_good)
      self.hud_text(self, other, x, y, row.tag, 1.1)
      gm.draw_set_colour(self.hud_color_dim)
      self.hud_text(self, other, x + 42, y, row.key, 1.1)
      gm.draw_set_colour(self.hud_color_ink)
      self.hud_text(self, other, x + 340, y, gm.string(row.value), 1.1)
      i = i + 1
    end
    gm.draw_set_alpha(1)
    gm.draw_set_colour(gm.c_white)
    return
  end
  cine_draw(self, other)
  tasks_popup_draw(self, other, self.hud_color_ink, self.hud_color_dim, self.hud_color_panel)
  if gm.bool(global.pause_open) then
    gm.draw_set_font(ui_font_small(self, other))
    gm.draw_set_colour(0)
    gm.draw_set_alpha(0.68)
    gm.draw_rectangle(0, 0, gw, gh, false)
    gm.draw_set_alpha(1)
    i = 0
    while i < gm.array_length(self.hud_volume_channels) do
      local r = self.hud_volume_slider_rect(self, other, i)
      local v = audio_get_volume(self, other, self.hud_volume_channels[i])
      local kx = r.x1 + (r.x2 - r.x1) * v
      gm.draw_set_colour(self.hud_color_dim)
      self.hud_text(self, other, r.x1, r.y1 - 30, loc(self, other, self.hud_volume_labels[i]), 1.2)
      self.hud_text(self, other, r.x2 - 60, r.y1 - 30, gm.string(gm.round(v * 100)) .. "%", 1.2)
      gm.draw_set_colour(self.hud_color_panel)
      gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, false)
      gm.draw_set_colour(self.hud_color_good)
      gm.draw_rectangle(r.x1, r.y1, kx, r.y2, false)
      if self.hud_volume_drag == i then
        gm.draw_set_colour(self.hud_color_ink)
      else
        gm.draw_set_colour(self.hud_color_dim)
      end
      gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, true)
      gm.draw_rectangle(kx - 4, r.y1 - 6, kx + 4, r.y2 + 6, false)
      i = i + 1
    end
    i = 0
    while i < gm.array_length(self.hud_pause_items) do
      local r = self.hud_pause_button_rect(self, other, i)
      local label = loc(self, other, "button_" .. self.hud_pause_items[i])
      local hover = self.hud_pause_hover == i
      local disabled = self.hud_pause_item_disabled(self, other, i)
      if self.hud_pause_items[i] == "black_metal" then
        local key
        if gm.bool(global.bw_mode) then
          key = "toggle_on"
        else
          key = "toggle_off"
        end
        label = label .. (": " .. loc(self, other, key))
      end
      if self.hud_pause_items[i] == "language" then
        label = label .. (": " .. loc(self, other, "language_name"))
      end
      if self.hud_pause_items[i] == "units" then
        local key
        if gm.bool(global.units_imperial) then
          key = "units_imperial"
        else
          key = "units_metric"
        end
        label = label .. (": " .. loc(self, other, key))
      end
      if self.hud_pause_items[i] == "display_mode" then
        local key
        if gm.bool(global.display_fullscreen) then
          key = "display_fullscreen"
        else
          key = "display_windowed"
        end
        label = label .. (": " .. loc(self, other, key))
      end
      if self.hud_pause_items[i] == "window_size" then
        local ws = display_window_size(self, other)
        label = label .. (": " .. gm.string(ws[0]) .. "x" .. gm.string(ws[1]))
      end
      gm.draw_set_colour(self.hud_color_panel)
      gm.draw_set_alpha(0.95)
      gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, false)
      if hover then
        gm.draw_set_colour(self.hud_color_ink)
      else
        gm.draw_set_colour(self.hud_color_dim)
      end
      gm.draw_set_alpha(1)
      gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, true)
      local sc = gm.min(1.8, (r.x2 - r.x1 - 32) / gm.max(1, gm.string_width(label)))
      local ty = (r.y1 + r.y2) * 0.5 - gm.string_height(label) * sc * 0.5
      if hover then
        gm.draw_set_colour(self.hud_color_ink)
      else
        gm.draw_set_colour(self.hud_color_dim)
      end
      if gm.bool(disabled) then
        gm.draw_set_alpha(0.35)
      else
        gm.draw_set_alpha(1)
      end
      self.hud_text(self, other, (r.x1 + r.x2) * 0.5 - gm.string_width(label) * sc * 0.5, ty, label, sc)
      gm.draw_set_alpha(1)
      i = i + 1
    end
    local lr = self.hud_pause_button_rect(self, other, gm.array_length(self.hud_pause_items) - 1)
    gm.draw_set_colour(self.hud_color_dim)
    gm.draw_set_alpha(0.7)
    self.hud_text(self, other, (lr.x1 + lr.x2) * 0.5 - gm.string_width("v1.02") * 0.5, lr.y2 + 12, "v1.02", 1)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(gm.c_white)
    return
  end
  if gm.bool(global.logo_open) then
    gm.draw_set_colour(0)
    gm.draw_set_alpha(0.97)
    gm.draw_rectangle(0, 0, gw, gh, false)
    gm.draw_set_alpha(1)
    local typing = global.logo_phase == "type"
    gm.draw_set_halign(gm.fa_center)
    gm.draw_set_colour(self.hud_color_dim)
    self.hud_text(self, other, gw * 0.5, 38, loc(self, other, "logo_title"),
      self.hud_fit(self, other, loc(self, other, "logo_title"), 1.5, gw - 80, 0))
    if typing then
      local caret
      if gm.mod(gm.div(gm.current_time, 500), 2) == 0 then
        caret = loc(self, other, "logo_caret")
      else
        caret = " "
      end
      if global.logo_text == "" then
        gm.draw_set_colour(self.hud_color_deep)
      else
        gm.draw_set_colour(self.hud_color_ink)
      end
      local shown
      if global.logo_text == "" then
        shown = loc(self, other, "logo_empty")
      else
        shown = global.logo_text .. caret
      end
      self.hud_text(self, other, gw * 0.5, gh * 0.5 - 40, shown, 4)
      gm.draw_set_colour(self.hud_color_deep)
      self.hud_text(self, other, gw * 0.5, gh - 72, loc(self, other, "logo_typing_hint"),
        self.hud_fit(self, other, loc(self, other, "logo_typing_hint"), 1.3, gw - 80, 0))
    else
      logo_draw(self, other, gw * 0.5, gh * 0.5 - 10, gw - 200, gh - 400, self.hud_color_ink)
      gm.draw_set_colour(self.hud_color_deep)
      self.hud_text(self, other, gw * 0.5, gh - 112, global.logo_text, 1.5)
      self.hud_text(self, other, gw * 0.5, gh - 72, loc(self, other, "logo_grown_hint"),
        self.hud_fit(self, other, loc(self, other, "logo_grown_hint"), 1.3, gw - 80, 0))
    end
    gm.draw_set_halign(gm.fa_left)
    local cl = self.hud_logo_clear_rect(self, other)
    gm.draw_set_colour(self.hud_color_dim)
    gm.draw_rectangle(cl.x1, cl.y1, cl.x2, cl.y2, true)
    gm.draw_set_colour(self.hud_color_ink)
    local cl_label
    if typing then
      cl_label = loc(self, other, "button_clear")
    else
      cl_label = loc(self, other, "button_edit")
    end
    self.hud_text_box(self, other, cl, cl_label, 1.4, 20)
    local rr = self.hud_logo_reroll_rect(self, other)
    if typing and gm.bool(logo_has_letters(self, other)) then
      gm.draw_set_colour(self.hud_color_good)
    else
      gm.draw_set_colour(self.hud_color_dim)
    end
    gm.draw_rectangle(rr.x1, rr.y1, rr.x2, rr.y2, true)
    gm.draw_set_colour(self.hud_color_ink)
    local rr_label
    if typing then
      rr_label = loc(self, other, "button_grow")
    else
      rr_label = loc(self, other, "button_reroll")
    end
    self.hud_text_box(self, other, rr, rr_label, 1.4, 20)
    local cr = self.hud_close_button_rect(self, other)
    gm.draw_set_colour(self.hud_color_dim)
    gm.draw_rectangle(cr.x1, cr.y1, cr.x2, cr.y2, true)
    gm.draw_set_colour(self.hud_color_ink)
    self.hud_text_box(self, other, cr, loc(self, other, "button_close"), 1.4, 22)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(gm.c_white)
  else
    if not gm.bool(global.menu_open) then
      gm.draw_set_alpha(1)
      gm.draw_set_colour(gm.c_white)
    else
      local c = self.hud_tree_canvas(self, other)
      local zoom = self.hud_tree_zoom
      local pulse = (math.sin(gm.current_time / 340) + 1) * 0.5
      gm.draw_set_colour(0)
      gm.draw_set_alpha(0.86)
      gm.draw_rectangle(0, 0, gw, gh, false)
      gm.draw_set_colour(self.hud_color_panel)
      gm.draw_set_alpha(0.6)
      gm.draw_rectangle(c.x1, c.y1, c.x2, c.y2, false)
      gm.draw_set_alpha(1)
      i = 0
      while i < gm.array_length(global.upgrade_defs) do
        local d = global.upgrade_defs[i]
        if d.parent == "" then
        else
          local p = upg_def(self, other, d.parent)
          if gm.is_undefined(p) then
          else
            local met = upg_requirement_met(self, other, d.id)
            local grow = upg_grow(self, other, d.id)
            local a = self.hud_tree_screen(self, other, p)
            local bb = self.hud_tree_screen(self, other, d)
            local via = upg_tree_via(self, other, d)
            local vs = gm.undefined
            if not gm.is_undefined(via) then
              vs = gm.new(___struct___465__gml_Object_obj_hud_Draw_75,
                (c.x1 + c.x2) * 0.5 + (via.x - self.hud_tree_cx) * zoom,
                (c.y1 + c.y2) * 0.5 + (via.y - self.hud_tree_cy) * zoom)
            end
            local galpha
            if gm.bool(met) then
              galpha = 0.5
            else
              galpha = 0.22
            end
            tree_draw_guide(self, other, a.x, a.y, bb.x, bb.y, gm.max(1, 2.4 * zoom), self.hud_color_dim,
              galpha * (1 - grow), vs)
            if grow > 0 then
              local wa = self.hud_tree_world(self, other, p)
              local wb = self.hud_tree_world(self, other, d)
              local h = tree_hypha_get(self, other, d.id, wa.x, wa.y, wb.x, wb.y, via)
              tree_hypha_draw(self, other, h, (c.x1 + c.x2) * 0.5 - self.hud_tree_cx * zoom,
                (c.y1 + c.y2) * 0.5 - self.hud_tree_cy * zoom, zoom, self.hud_color_ink, grow)
            end
          end
        end
        i = i + 1
      end
      gm.draw_set_halign(gm.fa_center)
      i = 0
      while i < gm.array_length(global.upgrade_trunks) do
        local t = global.upgrade_trunks[i]
        local hd = upg_def(self, other, t.head)
        if not gm.is_undefined(hd) then
          local p = self.hud_tree_screen(self, other, hd)
          local sc = self.hud_tree_text_scale(self, other, 1.5, vis(self, other, "tree_trunk_scale_min"))
          if gm.variable_struct_exists(t, "pair") then
            local pd = upg_def(self, other, t.pair)
            if not gm.is_undefined(pd) then
              p.x = (p.x + self.hud_tree_screen(self, other, pd).x) * 0.5
            end
          end
          if gm.bool(upg_is_owned(self, other, t.head)) then
            gm.draw_set_colour(self.hud_color_good)
          else
            gm.draw_set_colour(self.hud_color_dim)
          end
          local name = loc(self, other, t.name_key)
          local en_name = gm.variable_struct_get(global.locale_en, t.name_key)
          if gm.is_undefined(en_name) then
            en_name = name
          end
          local tw = gm.max(gm.string_width(en_name), gm.string_width("FRUITING BODIES")) * 1.3 * sc
          self.hud_text(self, other, p.x, p.y - self.hud_tree_radius(self, other, hd) * zoom - 44 * zoom - 14, name,
            self.hud_fit(self, other, name, sc, tw, 0))
        end
        i = i + 1
      end
      i = 0
      while i < gm.array_length(global.upgrade_defs) do
        local d = global.upgrade_defs[i]
        local p = self.hud_tree_screen(self, other, d)
        local r = self.hud_tree_radius(self, other, d) * zoom
        local state = upg_state(self, other, d.id)
        local hover = self.hud_hover_id == d.id
        local lvl = upg_level(self, other, d.id)
        local maxl = upg_max_level(self, other, d.id)
        local ring = self.hud_color_deep
        local tcol = self.hud_color_dim
        if state == "root" then
          ring = self.hud_color_ink
          tcol = self.hud_color_ink
        elseif state == "ready" then
          ring = self.hud_color_good
          tcol = self.hud_color_ink
        elseif state == "owned" then
          ring = self.hud_color_ink
          tcol = self.hud_color_ink
        elseif state == "maxed" then
          ring = self.hud_color_gold
          tcol = self.hud_color_gold
        elseif state == "short" then
          ring = self.hud_color_dim
          tcol = self.hud_color_dim
        end
        if state == "ready" then
          tree_draw_glow(self, other, p.x, p.y, r, self.hud_color_good, 0.35 + 0.45 * pulse)
        end
        if hover then
          tree_draw_glow(self, other, p.x, p.y, r, self.hud_color_ink, 0.5)
        end
        gm.draw_set_colour(self.hud_color_panel)
        gm.draw_set_alpha(0.95)
        gm.draw_circle(p.x, p.y, r, false)
        gm.draw_set_alpha(1)
        if lvl > 0 and maxl > 0 then
          if state == "maxed" then
            gm.draw_set_colour(self.hud_color_gold)
          else
            gm.draw_set_colour(self.hud_color_good)
          end
          gm.draw_set_alpha(0.22)
          gm.draw_circle(p.x, p.y, r * 0.92 * gm.sqrt(lvl / maxl), false)
          gm.draw_set_alpha(1)
        end
        local demo = state == "demo"
        if not demo then
          local isc
          if d.kind == "unlock" or d.kind == "root" then
            isc = vis(self, other, "tree_shroom_icon_fit")
          else
            isc = 1.3
          end
          local ialpha
          if gm.bool(upg_is_owned(self, other, d.id)) or d.kind == "root" then
            ialpha = 1
          else
            ialpha = 0.4
          end
          tree_draw_icon(self, other, d.icon, p.x, p.y, r * isc, ialpha, true, gm.undefined)
        else
          tree_draw_padlock(self, other, p.x, p.y, r * 0.95, tcol, self.hud_color_panel)
        end
        if d.id == "unlock_absorption" and not demo then
          local la
          if gm.bool(upg_is_owned(self, other, d.id)) then
            la = 0.95
          else
            la = 0.4
          end
          tree_draw_leaf_wrap(self, other, p.x, p.y, r, self.hud_color_ink, la, zoom)
        end
        if d.icon < 0 and d.kind ~= "ghost" and not demo then
          gm.draw_set_colour(ring)
          if gm.bool(upg_is_owned(self, other, d.id)) then
            gm.draw_set_alpha(0.9)
          else
            gm.draw_set_alpha(0.45)
          end
          gm.draw_circle(p.x, p.y, gm.max(2, r * 0.16), false)
          gm.draw_set_alpha(1)
        end
        gm.draw_set_colour(ring)
        gm.draw_circle(p.x, p.y, r, true)
        gm.draw_circle(p.x, p.y, r - 1, true)
        if hover then
          gm.draw_circle(p.x, p.y, r + 4 * zoom, true)
        end
        if d.kind == "level" and not demo then
          local lcol
          if state == "maxed" then
            lcol = self.hud_color_gold
          else
            lcol = self.hud_color_good
          end
          tree_draw_level_ring(self, other, p.x, p.y, r + 8 * zoom, lvl, maxl, lcol, self.hud_color_deep, 4 * zoom)
        end
        if state == "ready" or state == "short" or state == "owned" then
          local cg = 0
          local kids = upg_children(self, other, d.id)
          local k = 0
          while k < gm.array_length(kids) do
            cg = gm.max(cg, upg_grow(self, other, kids[k].id))
            k = k + 1
          end
          local cost = gm.string(upg_cost(self, other, d.id))
          local csc = self.hud_tree_text_scale(self, other, vis(self, other, "tree_cost_scale"),
            vis(self, other, "tree_cost_scale_min"))
          self.hud_tree_label_back(self, other, p.x + r * 0.85, p.y - r * 1.35, gm.string_width(cost) * csc,
            gm.string_height(cost) * csc, zoom, cg)
          if state == "ready" then
            gm.draw_set_colour(self.hud_color_good)
          else
            gm.draw_set_colour(self.hud_color_bad)
          end
          self.hud_text(self, other, p.x + r * 0.85, p.y - r * 1.35, gm.string(upg_cost(self, other, d.id)), csc)
        end
        local nm
        if d.kind == "ghost" then
          nm = loc(self, other, "upg_symbiosis")
        else
          nm = loc(self, other, "upg_" .. d.id)
        end
        if d.kind == "root" then
          nm = ""
        end
        local nsc = self.hud_tree_text_scale(self, other, vis(self, other, "tree_name_scale"),
          vis(self, other, "tree_name_scale_min"))
        local nw = vis(self, other, "tree_name_width") * nsc / vis(self, other, "tree_name_scale")
        if nm ~= "" then
          local maxh = gm.string_height("M") * 3 * nsc
          local minsc = nsc * 0.45
          local k = 0
          while k < 12 and nsc > minsc do
            if gm.string_width_ext(nm, -1, nw / nsc) * nsc <= nw + 1
                and gm.string_height_ext(nm, -1, nw / nsc) * nsc <= maxh + 1 then
              break
            end
            nsc = nsc * 0.92
            k = k + 1
          end
        end
        if nm ~= "" then
          self.hud_tree_label_back(self, other, p.x, p.y + r + 10 * zoom, gm.string_width_ext(nm, -1, nw / nsc) * nsc,
            gm.string_height_ext(nm, -1, nw / nsc) * nsc, zoom, upg_grow(self, other, d.id))
        end
        gm.draw_set_colour(tcol)
        gm.draw_text_ext_transformed(p.x, p.y + r + 10 * zoom, nm, -1, nw / nsc, nsc, nsc, 0)
        i = i + 1
      end
      gm.draw_set_halign(gm.fa_left)
      -- TODO(port): colour at GML L909 taken as hud_color_panel (decompiler variable merge, panel read at L702)
      gm.draw_set_colour(self.hud_color_panel)
      gm.draw_set_alpha(0.96)
      local hh = self.hud_tree_header_h
      gm.draw_rectangle(0, 0, gw, hh, false)
      gm.draw_set_alpha(1)
      gm.draw_set_colour(self.hud_color_deep)
      gm.draw_line(0, hh, gw, hh)
      local lr = self.hud_logo_button_rect(self, other)
      local title = loc(self, other, "button_upgrades")
      local pts = loc(self, other, "hud_points") .. "  " .. gm.string(global.upgrade_points)
      local tsc = self.hud_fit(self, other, title, 2.4, (lr.x1 - 80) * 0.5, 0)
      local pw = gm.max(300, 40 + gm.string_width(title) * tsc + 30)
      gm.draw_set_colour(self.hud_color_ink)
      self.hud_text(self, other, 40, 30, title, tsc)
      if global.upgrade_points > 0 then
        gm.draw_set_colour(self.hud_color_good)
      else
        gm.draw_set_colour(self.hud_color_dim)
      end
      self.hud_text(self, other, pw, 36, pts, self.hud_fit(self, other, pts, 1.8, lr.x1 - 20 - pw, 0))
      gm.draw_set_colour(self.hud_color_dim)
      gm.draw_rectangle(lr.x1, lr.y1, lr.x2, lr.y2, true)
      gm.draw_set_colour(self.hud_color_ink)
      self.hud_text_box(self, other, lr, loc(self, other, "logo_title"), 1.3, 20)
      local rr = self.hud_recentre_button_rect(self, other)
      gm.draw_set_colour(self.hud_color_dim)
      gm.draw_rectangle(rr.x1, rr.y1, rr.x2, rr.y2, true)
      gm.draw_set_colour(self.hud_color_ink)
      self.hud_text_box(self, other, rr, loc(self, other, "tree_recentre"), 1.4, 20)
      local cr = self.hud_close_button_rect(self, other)
      gm.draw_set_colour(self.hud_color_dim)
      gm.draw_rectangle(cr.x1, cr.y1, cr.x2, cr.y2, true)
      gm.draw_set_colour(self.hud_color_ink)
      self.hud_text_box(self, other, cr, loc(self, other, "button_close"), 1.4, 22)
      local fh = self.hud_tree_footer_h
      local fy = gh - fh
      gm.draw_set_colour(self.hud_color_panel)
      gm.draw_set_alpha(0.96)
      gm.draw_rectangle(0, fy, gw, gh, false)
      gm.draw_set_alpha(1)
      gm.draw_set_colour(self.hud_color_deep)
      gm.draw_line(0, fy, gw, fy)
      gm.draw_set_colour(self.hud_color_dim)
      self.hud_text(self, other, 48, fy + 22, loc(self, other, "tree_hint"),
        self.hud_fit(self, other, loc(self, other, "tree_hint"), 1.3, gw - 96, fh - 30))
      if self.hud_hover_id ~= "" then
        local d = upg_def(self, other, self.hud_hover_id)
        local st = upg_state(self, other, self.hud_hover_id)
        local ghost = d.kind == "ghost"
        local nm
        if ghost then
          nm = loc(self, other, "upg_symbiosis")
        else
          nm = loc(self, other, "upg_" .. d.id)
        end
        local desc
        if ghost then
          desc = loc(self, other, "upg_symbiosis_desc")
        else
          desc = loc(self, other, "upg_" .. d.id .. "_desc")
        end
        local lvl_text = ""
        local status = ""
        local status_col = self.hud_color_dim
        if d.kind == "root" then
          desc = ""
        end
        if d.kind ~= "root" and not ghost then
          lvl_text = loc(self, other, "upgrade_level") .. " " .. gm.string(upg_level(self, other, self.hud_hover_id))
            .. " / " .. gm.string(upg_max_level(self, other, self.hud_hover_id))
        end
        if st == "root" then
        elseif st == "ghost" then
          status = loc(self, other, "upgrade_coming_later")
        elseif st == "locked" then
          local pd = upg_def(self, other, d.parent)
          status = loc(self, other, "upgrade_requires") .. "  " .. loc(self, other, "upg_" .. pd.id) .. "  "
            .. loc(self, other, "upgrade_level") .. " " .. gm.string(d.req)
        elseif st == "maxed" then
          status = loc(self, other, "upgrade_max")
          status_col = self.hud_color_gold
        elseif st == "demo" then
          status = loc(self, other, "upgrade_demo_locked")
        elseif st == "ready" then
          status = loc(self, other, "upgrade_cost") .. " " .. gm.string(upg_cost(self, other, self.hud_hover_id))
            .. "    " .. loc(self, other, "upgrade_grow")
          status_col = self.hud_color_good
        else
          status = loc(self, other, "upgrade_cost") .. " " .. gm.string(upg_cost(self, other, self.hud_hover_id))
            .. "    " .. loc(self, other, "upgrade_no_points")
          status_col = self.hud_color_bad
        end
        local pad = 18
        local tnsc = 1.7
        local ssc = 1.25
        local tpw = 460
        local gap = 10
        local maxw = gm.min(tpw + 240, gw - 24 - pad * 2)
        local lvl_part
        if lvl_text == "" then
          lvl_part = 0
        else
          lvl_part = 32 + gm.string_width(lvl_text) * ssc
        end
        tnsc = self.hud_fit(self, other, nm, tnsc, maxw - lvl_part, 0)
        local stsc = self.hud_fit(self, other, status, ssc, maxw, 0)
        local nmw = gm.string_width(nm) * tnsc
        local lvw = gm.string_width(lvl_text) * ssc
        local descw
        if desc == "" then
          descw = 0
        else
          descw = gm.min(tpw, gm.string_width(desc) * ssc)
        end
        local stw = gm.string_width(status) * stsc
        local lvl_w
        if lvl_text == "" then
          lvl_w = 0
        else
          lvl_w = 32 + lvw
        end
        local boxw = gm.max(nmw + lvl_w, descw, stw)
        local lh = gm.string_height("M") * tnsc
        local desch
        if desc == "" then
          desch = 0
        else
          desch = gm.string_height_ext(desc, -1, tpw / ssc) * ssc
        end
        local sth
        if status == "" then
          sth = 0
        else
          sth = gm.string_height("M") * ssc
        end
        local bw = boxw + pad * 2
        local st_part
        if sth > 0 then
          st_part = gap + sth
        else
          st_part = 0
        end
        local de_part
        if desch > 0 then
          de_part = gap + desch
        else
          de_part = 0
        end
        local bh = pad * 2 + lh + de_part + st_part
        local mxg = gm.device_mouse_x_to_gui(0)
        local myg = gm.device_mouse_y_to_gui(0)
        local bx = mxg + 26
        local by = myg + 26
        if bx + bw > gw - 12 then
          bx = mxg - 26 - bw
        end
        if by + bh > gh - 12 then
          by = myg - 26 - bh
        end
        bx = gm.clamp(bx, 12, gm.max(12, gw - 12 - bw))
        by = gm.clamp(by, 12, gm.max(12, gh - 12 - bh))
        gm.draw_set_colour(self.hud_color_panel)
        gm.draw_set_alpha(0.96)
        gm.draw_rectangle(bx, by, bx + bw, by + bh, false)
        gm.draw_set_alpha(1)
        if st == "ready" then
          gm.draw_set_colour(self.hud_color_good)
        else
          gm.draw_set_colour(self.hud_color_deep)
        end
        gm.draw_rectangle(bx, by, bx + bw, by + bh, true)
        local ty = by + pad
        gm.draw_set_colour(self.hud_color_ink)
        self.hud_text(self, other, bx + pad, ty, nm, tnsc)
        if lvl_text ~= "" then
          gm.draw_set_halign(gm.fa_right)
          gm.draw_set_colour(self.hud_color_dim)
          self.hud_text(self, other, bx + bw - pad, ty + (lh - gm.string_height("M") * ssc) * 0.5, lvl_text, ssc)
          gm.draw_set_halign(gm.fa_left)
        end
        ty = ty + lh
        if desch > 0 then
          ty = ty + gap
          gm.draw_set_colour(self.hud_color_dim)
          gm.draw_text_ext_transformed(bx + pad, ty, desc, -1, tpw / ssc, ssc, ssc, 0)
          ty = ty + desch
        end
        if sth > 0 then
          ty = ty + gap
          gm.draw_set_colour(status_col)
          self.hud_text(self, other, bx + pad, ty, status, stsc)
        end
      end
      gm.draw_set_alpha(1)
      gm.draw_set_colour(gm.c_white)
    end
  end
end
