local gm = require("gm")

-- gml_Script_mushroom_type_cost @ 0x1400b1a30
function mushroom_type_cost(self, other, t)
  return gm.round(mushroom_type_cost_base(self, other, t) * perk_cost_mult(self, other))
end

-- gml_Script_mushroom_type_water_cost @ 0x1400b27e0
function mushroom_type_water_cost(self, other, t)
  return gm.round(mushroom_type_water_cost_base(self, other, t) * perk_cost_mult(self, other))
end

-- gml_Script_mushroom_type_affordable @ 0x1400b3a00
function mushroom_type_affordable(self, other, t)
  return global.food >= mushroom_type_cost(self, other, t)
    and water_total(self, other) >= mushroom_type_water_cost(self, other, t)
end

-- gml_Script_mushroom_type_name_key @ 0x1400b40e0
function mushroom_type_name_key(self, other, t)
  return "upg_" .. mushroom_type_unlock_id(self, other, t)
end

-- gml_Script_mushroom_type_unlocked @ 0x1400b86a0
function mushroom_type_unlocked(self, other, t)
  return upg_is_owned(self, other, mushroom_type_unlock_id(self, other, t))
end

-- gml_Script_anon__238__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064cb30
function anon__238__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.myc_camera = gm.view_camera[0]
  if self.myc_camera ~= -1 then
    self.myc_camera_x = gm.camera_get_view_x(self.myc_camera)
    self.myc_camera_y = gm.camera_get_view_y(self.myc_camera)
    self.myc_view_w = gm.camera_get_view_width(self.myc_camera)
    self.myc_view_h = gm.camera_get_view_height(self.myc_camera)
  end
end

-- gml_Script_anon__1762__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064d130
function anon__1762__gml_Object_obj_mycelium_controller_Create_0(self, other)
  return water_total(self, other) > 0
end

-- gml_Script_anon__1897__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064d240
function anon__1897__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if not gm.bool(self.myc_has_water_source(self, other)) then
    return "hint_needs_water"
  end
  if gm.bool(self.myc_has_mushroom_near(self, other, px, py, self.myc_tile * 2.2)) then
    return "hint_too_crowded"
  end
  return ""
end

-- gml_Script_anon__2138__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064d600
function anon__2138__gml_Object_obj_mycelium_controller_Create_0(self, other, tx, ty)
  return world_tile_at(self, other, tx, ty) == 2
end

-- gml_Script_anon__2406__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064d810
function anon__2406__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, cave)
  if gm.is_undefined(cave) then cave = 0 end
  if not gm.bool(cave) then
    if gm.bool(self.myc_is_blocked_point(self, other, px, py)) then
      return false
    end
  end
  if gm.bool(cave) then
    if gm.bool(world_cave_solid_any(self, other, px, py)) then
      return false
    end
  end
  return world_tile_at(self, other, px, py) ~= -1
end

-- gml_Script_anon__2806__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064ddc0
function anon__2806__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, r, cave)
  if gm.is_undefined(cave) then cave = 0 end
  if not gm.bool(self.myc_is_playable_point(self, other, px, py, cave)) then
    return false
  end
  if gm.bool(cave) then
    return true
  end
  return not gm.bool(self.myc_is_blocked_point(self, other, px - r, py))
    and not gm.bool(self.myc_is_blocked_point(self, other, px + r, py))
    and not gm.bool(self.myc_is_blocked_point(self, other, px, py - r))
    and not gm.bool(self.myc_is_blocked_point(self, other, px, py + r))
end

-- gml_Script_anon__3225__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064e820
function anon__3225__gml_Object_obj_mycelium_controller_Create_0(self, other, x1, y1, x2, y2, r, cave)
  if gm.is_undefined(cave) then cave = 0 end
  local dist = gm.point_distance(x1, y1, x2, y2)
  if dist <= 0 then
    return self.myc_is_playable_clear_point(self, other, x1, y1, r, cave)
  end
  local d = 0
  while d < dist do
    local t = d / dist
    if not gm.bool(self.myc_is_playable_clear_point(self, other,
        gm.lerp(x1, x2, t), gm.lerp(y1, y2, t), r, cave)) then
      return false
    end
    d = d + self.myc_sample_step
  end
  return self.myc_is_playable_clear_point(self, other, x2, y2, r, cave)
end

-- gml_Script_anon__4008__gml_Object_obj_mycelium_controller_Create_0 @ 0x14064f620
function anon__4008__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, dir, maxlen, cave)
  if gm.is_undefined(cave) then cave = 0 end
  local pen = myc_soil_penetration(self, other)
  local r = 2
  local budget = gm.max(0, maxlen)
  local best = 0
  local used = 0
  if not gm.bool(self.myc_is_playable_clear_point(self, other, px, py, r, cave)) then
    return 0
  end
  local d = self.myc_sample_step
  while used < budget do
    local sx = px + gm.lengthdir_x(d, dir)
    local sy = py + gm.lengthdir_y(d, dir)
    if not gm.bool(self.myc_is_playable_clear_point(self, other, sx, sy, r, cave)) then
      break
    end
    used = used + self.myc_sample_step * world_travel_cost(self, other, sx, sy, pen)
    best = d
    d = d + self.myc_sample_step
  end
  return best
end

-- gml_Script_anon__5108__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406502a0
function anon__5108__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, dir, len)
  if not (len > 0) then
    return 0
  end
  local samples = 0
  local sum = 0
  local d = 0
  while d <= len do
    samples = samples + 1
    local sx = px + gm.lengthdir_x(d, dir)
    local sy = py + gm.lengthdir_y(d, dir)
    sum = sum + world_soil_water_factor(self, other, sx, sy)
    d = d + self.myc_sample_step
  end
  local avg = sum / gm.max(1, samples)
  return myc_shot_water_rate(self, other) * len * avg
end

-- gml_Script_anon__5711__gml_Object_obj_mycelium_controller_Create_0 @ 0x140650a90
function anon__5711__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, dir, len)
  local d = 0
  while d <= len do
    local sx = px + gm.lengthdir_x(d, dir)
    local sy = py + gm.lengthdir_y(d, dir)
    if gm.bool(world_is_deep_soil(self, other, sx, sy)) then
      return bal(self, other, "deepsoil_shot_food_cost")
    end
    d = d + self.myc_sample_step
  end
  return 0
end

-- gml_Script____struct___466__anon__6019__gml_Object_obj_mycelium_controller_Create_0 @ 0x140650fb0
function ___struct___466__anon__6019__gml_Object_obj_mycelium_controller_Create_0(self, other, x1, y1, x2, y2, label)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.label = label
end

-- gml_Script_anon__6019__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406512c0
function anon__6019__gml_Object_obj_mycelium_controller_Create_0(self, other, wx, wy, label, offy)
  self.myc_sync_view(self, other)
  local gw = gm.max(1, gm.display_get_gui_width())
  local gh = gm.max(1, gm.display_get_gui_height())
  local sx = gw / gm.max(1, self.myc_view_w)
  local sy = gh / gm.max(1, self.myc_view_h)
  gm.draw_set_font(ui_font_small(self, other))
  local w = gm.max(110, gm.string_width(label) + 18)
  local h = 28
  local gx = (wx - self.myc_camera_x) * sx - w * 0.5
  local gy = (wy - self.myc_camera_y) * sy + offy
  gx = gm.clamp(gx, 8, gw - w - 8)
  gy = gm.clamp(gy, 8, gh - h - 8)
  global.___struct___466 = gm.method(gm.undefined, ___struct___466__anon__6019__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___466__anon__6019__gml_Object_obj_mycelium_controller_Create_0, gx, gy, gx + w, gy + h, label)
end

-- gml_Script_anon__6802__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406527a0
function anon__6802__gml_Object_obj_mycelium_controller_Create_0(self, other, opt)
  if opt.kind == "node" then
    return loc(self, other, "button_add_node")
      .. "  " .. gm.string(myc_node_food_cost(self, other))
      .. " " .. loc(self, other, "hud_food")
  else
    local t = opt.type
    local name = loc(self, other, mushroom_type_name_key(self, other, t))
    local cost = mushroom_type_cost(self, other, t)
    return gm.string_upper(name) .. "  " .. gm.string(cost) .. " " .. loc(self, other, "hud_food")
  end
end

-- gml_Script_anon__7134__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406534b0
function anon__7134__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local label = loc(self, other, "button_release_spores") .. " (" .. gm.string(bal(self, other, "spore_release_cost")) .. " " .. loc(self, other, "hud_food") .. ")"
  return self.myc_button_rect(self, other, self.myc_spore_prompt_x, self.myc_spore_prompt_y, label, -72.0)
end

-- gml_Script_anon__7351__gml_Object_obj_mycelium_controller_Create_0 @ 0x140653c70
function anon__7351__gml_Object_obj_mycelium_controller_Create_0(self, other, i, opt)
  local label = self.myc_build_option_label(self, other, opt)
  return self.myc_button_rect(self, other, self.myc_build_node_x, self.myc_build_node_y, label, -44.0 - i * 34)
end

-- gml_Script_anon__7521__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406540a0
function anon__7521__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local label = loc(self, other, "button_regenerate") .. " (" .. gm.string(repeller_regen_price(self, other)) .. " " .. loc(self, other, "hud_food") .. ")"
  return self.myc_button_rect(self, other, self.myc_spore_prompt_x, self.myc_spore_prompt_y, label, -72.0)
end

-- gml_Script_anon__7725__gml_Object_obj_mycelium_controller_Create_0 @ 0x140654820
function anon__7725__gml_Object_obj_mycelium_controller_Create_0(self, other, r, hover, pressed)
  local a = gm.bool(hover) and 0.78 or 0.44
  if gm.bool(pressed) and gm.bool(hover) then
    a = 1.0
  end
  gm.draw_set_colour(0)
  gm.draw_set_alpha(0.78)
  gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, false)
  gm.draw_set_colour(0xeaf8f5)
  gm.draw_set_alpha(a)
  gm.draw_rectangle(r.x1, r.y1, r.x2, r.y2, true)
  gm.draw_set_font(ui_font_small(self, other))
  gm.draw_set_halign(1)
  gm.draw_set_valign(1)
  gm.draw_text((r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5, r.label)
  gm.draw_set_halign(0)
  gm.draw_set_valign(0)
end

-- gml_Script_anon__8392__gml_Object_obj_mycelium_controller_Create_0 @ 0x140655440
function anon__8392__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.myc_spore_prompt = false
  self.myc_spore_prompt_index = -1
  self.myc_regen_prompt = false
  self.myc_mushroom_prompt = false
  self.myc_build_options = gm.array()
  self.myc_build_hover = -1
  self.myc_build_hint = ""
end

-- gml_Script_anon__8636__gml_Object_obj_mycelium_controller_Create_0 @ 0x140655740
function anon__8636__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local label = loc(self, other, self.myc_build_hint)
  local n = gm.is_array(self.myc_build_options) and gm.array_length(self.myc_build_options) or 0
  return self.myc_button_rect(self, other, self.myc_build_node_x, self.myc_build_node_y, label, n * -34 - 44)
end

-- gml_Script_anon__8801__gml_Object_obj_mycelium_controller_Create_0 @ 0x140655a90
function anon__8801__gml_Object_obj_mycelium_controller_Create_0(self, other, ox, oy)
  tutorial_event(self, other, "aim", ox, oy)
  self.myc_dragging = true
  self.myc_has_aim = true
  self.myc_aim_origin_x = ox
  self.myc_aim_origin_y = oy
  self.myc_aim_dir = self.myc_default_aim_dir
  self.myc_aim_len = 0
  self.myc_aim_reach = 0
  self.myc_aim_cost = 0
  self.myc_aim_food = 0
  self.myc_drag_start_x = gm.mouse_x
  self.myc_drag_start_y = gm.mouse_y
  self.myc_clear_prompts(self, other)
  gm.window_set_cursor(-1)
end

-- gml_Script_anon__9236__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406561b0
function anon__9236__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, r)
  local i = 0
  while i < gm.array_length(self.myc_knots) do
    local k = self.myc_knots[i]
    if gm.point_distance(px, py, k.x, k.y) <= r then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___467__anon__9512__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406566d0
function ___struct___467__anon__9512__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_anon__9512__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406568d0
function anon__9512__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if gm.bool(self.myc_has_knot_near(self, other, px, py, self.myc_knot_radius * 1.6)) then
    return false
  end
  if gm.bool(world_water_shown_at(self, other, px, py)) then
    return false
  end
  global.___struct___467 = gm.method(gm.undefined, ___struct___467__anon__9512__gml_Object_obj_mycelium_controller_Create_0)
  gm.array_push(self.myc_knots, gm.new(___struct___467__anon__9512__gml_Object_obj_mycelium_controller_Create_0, px, py))
  reach_expand(self, other, px, py)
  return true
end

-- gml_Script____struct___468__anon__10006__gml_Object_obj_mycelium_controller_Create_0 @ 0x140657090
function ___struct___468__anon__10006__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.sprite = smallshroom1
  self.cols = 9.0
  self.rows = 1.0
end

-- gml_Script____struct___469__anon__10006__gml_Object_obj_mycelium_controller_Create_0 @ 0x140657270
function ___struct___469__anon__10006__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.sprite = smallshroom3
  self.cols = 9.0
  self.rows = 1.0
end

-- gml_Script_anon__10006__gml_Object_obj_mycelium_controller_Create_0 @ 0x140657450
local anon__10006_static = {}
function anon__10006__gml_Object_obj_mycelium_controller_Create_0(self, other)
  if not anon__10006_static.initialised then
    anon__10006_static.initialised = true
    global.___struct___468 = gm.method(gm.undefined, ___struct___468__anon__10006__gml_Object_obj_mycelium_controller_Create_0)
    global.___struct___469 = gm.method(gm.undefined, ___struct___469__anon__10006__gml_Object_obj_mycelium_controller_Create_0)
    anon__10006_static._sheets = gm.array(
      gm.new(___struct___468__anon__10006__gml_Object_obj_mycelium_controller_Create_0),
      gm.new(___struct___469__anon__10006__gml_Object_obj_mycelium_controller_Create_0))
  end
  return anon__10006_static._sheets
end

-- gml_Script____struct___470__anon__10257__gml_Object_obj_mycelium_controller_Create_0 @ 0x140657960
function ___struct___470__anon__10257__gml_Object_obj_mycelium_controller_Create_0(self, other, kind, cell)
  self.kind = kind
  self.cell = cell
end

-- gml_Script_anon__10257__gml_Object_obj_mycelium_controller_Create_0 @ 0x140657b30
function anon__10257__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local sheets = self.myc_smallshroom_sheets(self, other)
  local kind = gm.irandom(gm.array_length(sheets) - 1)
  local s = sheets[kind]
  global.___struct___470 = gm.method(gm.undefined, ___struct___470__anon__10257__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___470__anon__10257__gml_Object_obj_mycelium_controller_Create_0, kind, gm.irandom(s.cols * s.rows - 1))
end

-- gml_Script____struct___471__anon__10512__gml_Object_obj_mycelium_controller_Create_0 @ 0x140658110
function ___struct___471__anon__10512__gml_Object_obj_mycelium_controller_Create_0(self, other, sprite, left, top, width, height)
  self.sprite = sprite
  self.left = left
  self.top = top
  self.width = width
  self.height = height
end

-- gml_Script_anon__10512__gml_Object_obj_mycelium_controller_Create_0 @ 0x140658420
function anon__10512__gml_Object_obj_mycelium_controller_Create_0(self, other, v)
  local sheet = self.myc_smallshroom_sheets(self, other)[v.kind]
  local fw = gm.div(gm.sprite_get_width(sheet.sprite), sheet.cols)
  local fh = gm.div(gm.sprite_get_height(sheet.sprite), sheet.rows)
  global.___struct___471 = gm.method(gm.undefined, ___struct___471__anon__10512__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___471__anon__10512__gml_Object_obj_mycelium_controller_Create_0,
    sheet.sprite, gm.mod(v.cell, sheet.cols) * fw, gm.div(v.cell, sheet.cols) * fh, fw, fh)
end

-- gml_Script_anon__10910__gml_Object_obj_mycelium_controller_Create_0 @ 0x140658e50
function anon__10910__gml_Object_obj_mycelium_controller_Create_0(self, other, px, gap)
  local i = 0
  while i < gm.array_length(self.myc_knots) do
    local k = self.myc_knots[i]
    if gm.variable_struct_exists(k, "shroom") then
      if gm.abs(k.x - px) <= gap then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___472__anon__11304__gml_Object_obj_mycelium_controller_Create_0 @ 0x140659320
function ___struct___472__anon__11304__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, shroom, born)
  self.x = x
  self.y = y
  self.shroom = shroom
  self.born = born
end

-- gml_Script_anon__11304__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406595f0
function anon__11304__gml_Object_obj_mycelium_controller_Create_0(self, other, px)
  if not gm.bool(world_is_passable(self, other, px, self.myc_surface_y + 4)) then
    return false
  end
  if gm.bool(self.myc_has_smallshroom_near(self, other, px, bal(self, other, "spore_shroom_gap_px"))) then
    return false
  end
  if gm.bool(self.myc_has_knot_near(self, other, px, self.myc_surface_y, self.myc_knot_radius * 1.6)) then
    return false
  end
  local now = gm.current_time
  global.___struct___472 = gm.method(gm.undefined, ___struct___472__anon__11304__gml_Object_obj_mycelium_controller_Create_0)
  gm.array_push(self.myc_knots, gm.new(___struct___472__anon__11304__gml_Object_obj_mycelium_controller_Create_0,
    px, self.myc_surface_y, self.myc_smallshroom_new(self, other), now))
  mushroom_grow_sound(self, other, px, self.myc_surface_y, vis(self, other, "smallshroom_grow_volume"))
  reach_expand(self, other, px, self.myc_surface_y)
  return true
end

-- gml_Script_anon__11972__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065a2d0
function anon__11972__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  self.myc_add_knot(self, other, px, py)
end

-- gml_Script_anon__12049__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065a4b0
function anon__12049__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, r)
  if gm.point_distance(px, py, self.x, self.y) <= r then
    return true
  end
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if not (m.type == 3 or m.type == 7) then
      if gm.point_distance(px, py, m.x, m.y) <= r then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___473__anon__12632__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065ade0
function ___struct___473__anon__12632__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.hit = true
  self.x = x
  self.y = y
end

-- gml_Script____struct___474__anon__12632__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065b050
function ___struct___474__anon__12632__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.hit = false
  self.x = 0
  self.y = 0
end

-- gml_Script_anon__12632__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065b270
function anon__12632__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, r)
  local i = 0
  while i < gm.array_length(self.myc_knots) do
    local k = self.myc_knots[i]
    if gm.point_distance(px, py, k.x, k.y) <= r then
      global.___struct___473 = gm.method(gm.undefined, ___struct___473__anon__12632__gml_Object_obj_mycelium_controller_Create_0)
      return gm.new(___struct___473__anon__12632__gml_Object_obj_mycelium_controller_Create_0, k.x, k.y)
    end
    i = i + 1
  end
  global.___struct___474 = gm.method(gm.undefined, ___struct___474__anon__12632__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___474__anon__12632__gml_Object_obj_mycelium_controller_Create_0)
end

-- gml_Script____struct___475__anon__13119__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065baf0
function ___struct___475__anon__13119__gml_Object_obj_mycelium_controller_Create_0(self, other, x1, y1, x2, y2)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__13119__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065bda0
function anon__13119__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local x1 = gm.floor(self.x) + gm.sprite_get_bbox_left(originshroom)
    - gm.sprite_get_xoffset(originshroom)
  local y1 = gm.floor(self.y) + gm.sprite_get_bbox_top(originshroom)
    - gm.sprite_get_yoffset(originshroom)
  global.___struct___475 = gm.method(gm.undefined, ___struct___475__anon__13119__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___475__anon__13119__gml_Object_obj_mycelium_controller_Create_0, x1, y1,
    x1 + gm.sprite_get_bbox_right(originshroom) - gm.sprite_get_bbox_left(originshroom),
    y1 + gm.sprite_get_bbox_bottom(originshroom) - gm.sprite_get_bbox_top(originshroom))
end

-- gml_Script____struct___476__anon__13668__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065cbd0
function ___struct___476__anon__13668__gml_Object_obj_mycelium_controller_Create_0(self, other, index, x, y)
  self.hit = true
  self.index = index
  self.x = x
  self.y = y
end

-- gml_Script____struct___477__anon__13668__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065cea0
function ___struct___477__anon__13668__gml_Object_obj_mycelium_controller_Create_0(self, other, index, x, y)
  self.hit = true
  self.index = index
  self.x = x
  self.y = y
end

-- gml_Script____struct___478__anon__13668__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065d170
function ___struct___478__anon__13668__gml_Object_obj_mycelium_controller_Create_0(self, other, index)
  self.hit = false
  self.index = index
  self.x = 0
  self.y = 0
end

-- gml_Script_anon__13668__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065d400
function anon__13668__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  local i = gm.array_length(global.mushrooms) - 1
  while i >= 0 do
    if not gm.bool(global.mushrooms[i].stolen) then
      local b = mushroom_hit_box(self, other, global.mushrooms[i])
      if gm.point_in_rectangle(px, py, b.x1, b.y1, b.x2, b.y2) then
        local m = global.mushrooms[i]
        global.___struct___476 = gm.method(gm.undefined, ___struct___476__anon__13668__gml_Object_obj_mycelium_controller_Create_0)
        return gm.new(___struct___476__anon__13668__gml_Object_obj_mycelium_controller_Create_0, i, m.x, m.y)
      end
    end
    i = i - 1
  end
  local o = self.myc_origin_box(self, other)
  if gm.point_in_rectangle(px, py, o.x1, o.y1, o.x2, o.y2) then
    global.___struct___477 = gm.method(gm.undefined, ___struct___477__anon__13668__gml_Object_obj_mycelium_controller_Create_0)
    return gm.new(___struct___477__anon__13668__gml_Object_obj_mycelium_controller_Create_0, -1.0, self.x, self.y)
  end
  global.___struct___478 = gm.method(gm.undefined, ___struct___478__anon__13668__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___478__anon__13668__gml_Object_obj_mycelium_controller_Create_0, -1.0)
end

-- gml_Script____struct___479__anon__14604__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065e770
function ___struct___479__anon__14604__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.hit = false
  self.x = x
  self.y = y
end

-- gml_Script____struct___480__anon__14604__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065e9d0
function ___struct___480__anon__14604__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.hit = true
  self.x = x
  self.y = y
end

-- gml_Script_anon__14604__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065ec40
function anon__14604__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if gm.abs(py - self.myc_surface_y) <= bal(self, other, "surface_build_band_px") then
    global.___struct___480 = gm.method(gm.undefined, ___struct___480__anon__14604__gml_Object_obj_mycelium_controller_Create_0)
    return gm.new(___struct___480__anon__14604__gml_Object_obj_mycelium_controller_Create_0, px, self.myc_surface_y)
  end
  global.___struct___479 = gm.method(gm.undefined, ___struct___479__anon__14604__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___479__anon__14604__gml_Object_obj_mycelium_controller_Create_0, px, py)
end

-- gml_Script_anon__14901__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065f280
function anon__14901__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if not gm.bool(self.myc_has_water_source(self, other)) then
    return false
  end
  if gm.bool(self.myc_has_mushroom_near(self, other, px, py, self.myc_tile * 2.2)) then
    return false
  end
  local b = self.myc_get_surface_build_at(self, other, px, py)
  if not gm.bool(b.hit) or not (gm.point_distance(b.x, b.y, px, py) <= self.myc_tile) then
    return false
  end
  return self.myc_get_mycelium_hit(self, other, px, py + self.myc_tile * 0.85, self.myc_tile * 1.2).hit
end

-- gml_Script_anon__15353__gml_Object_obj_mycelium_controller_Create_0 @ 0x14065fd60
function anon__15353__gml_Object_obj_mycelium_controller_Create_0(self, other, px)
  if not gm.bool(gm.instance_exists(obj_forest_controller)) then
    return gm.undefined
  end
  local f = gm.instance_find(obj_forest_controller, 0)
  return f.forest_find_tree_near(f, other, px, bal(self, other, "tree_build_reach_px"))
end

-- gml_Script_anon__15583__gml_Object_obj_mycelium_controller_Create_0 @ 0x140660300
function anon__15583__gml_Object_obj_mycelium_controller_Create_0(self, other, t, px)
  if not gm.bool(mushroom_type_unlocked(self, other, t)) or not gm.bool(mushroom_type_affordable(self, other, t)) then
    return false
  end
  if t == 3 then
    local tree = self.myc_find_tree_near(self, other, px)
    return not gm.is_undefined(tree) and not gm.bool(squirrel_tree_has_mushroom(self, other, tree))
  end
  return true
end

-- gml_Script_anon__15952__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406607d0
function anon__15952__gml_Object_obj_mycelium_controller_Create_0(self, other, t, px, py)
  if not gm.bool(self.myc_can_build_type(self, other, t, px, py)) then
    return false
  end
  if t == 3 then
    local tree = self.myc_find_tree_near(self, other, px)
    if gm.is_undefined(tree) then
      return false
    end
    food_spend(self, other, mushroom_type_cost(self, other, t))
    water_spend(self, other, mushroom_type_water_cost(self, other, t))
    local mx = tree.x + vis(self, other, "tree_mushroom_offset_x")
    local my = self.myc_surface_y - vis(self, other, "tree_mushroom_height_px")
    mushroom_add(self, other, t, mx, my)
    return true
  end
  food_spend(self, other, mushroom_type_cost(self, other, t))
  if mushroom_type_water_cost(self, other, t) > 0 then
    water_spend(self, other, mushroom_type_water_cost(self, other, t))
  end
  mushroom_add(self, other, t, px, py)
  self.myc_add_root_knot_for_mushroom(self, other, px, py)
  if t == 4 then
    self.myc_digger_burst(self, other, px, py)
    if gm.bool(upg_is_owned(self, other, "unlock_digger")) then
      if upg_level(self, other, "digger_length") >= upg_max_level(self, other, "digger_length") then
        tasks_stat_add(self, other, "ach_digger_full", gm.undefined)
      end
    end
  end
  return true
end

-- gml_Script____struct___481__anon__17079__gml_Object_obj_mycelium_controller_Create_0 @ 0x140661bb0
function ___struct___481__anon__17079__gml_Object_obj_mycelium_controller_Create_0(self, other, type)
  self.kind = "node"
  self.type = type
end

-- gml_Script____struct___482__anon__17079__gml_Object_obj_mycelium_controller_Create_0 @ 0x140661d60
function ___struct___482__anon__17079__gml_Object_obj_mycelium_controller_Create_0(self, other, type)
  self.kind = "shroom"
  self.type = type
end

-- gml_Script_anon__17079__gml_Object_obj_mycelium_controller_Create_0 @ 0x140661f10
function anon__17079__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local options = gm.array()
  if not gm.bool(world_water_shown_at(self, other, self.myc_build_node_x, self.myc_build_node_y)) then
    if not gm.bool(self.myc_has_knot_near(self, other, self.myc_build_node_x, self.myc_build_node_y, self.myc_knot_radius * 2)) then
      global.___struct___481 = gm.method(gm.undefined, ___struct___481__anon__17079__gml_Object_obj_mycelium_controller_Create_0)
      gm.array_push(options, gm.new(___struct___481__anon__17079__gml_Object_obj_mycelium_controller_Create_0, -1.0))
    end
  else
    self.myc_build_hint = "hint_no_nodes_in_water"
  end
  if gm.bool(self.myc_build_surface_ok) then
    -- TODO(port): the compiled binary has no code for GML lines 618-641 here (verified in disassembly at
    -- 0x140662600: only the line marker 622 is set); the "shroom" options (___struct___482) are never pushed.
  end
  return options
end

-- gml_Script_anon__18320__gml_Object_obj_mycelium_controller_Create_0 @ 0x140662760
function anon__18320__gml_Object_obj_mycelium_controller_Create_0(self, other, opt)
  if opt.kind == "node" then
    return global.food >= myc_node_food_cost(self, other)
  else
    return self.myc_can_build_type(self, other, opt.type, self.myc_build_surface_x, self.myc_build_surface_y)
  end
end

-- gml_Script_anon__18703__gml_Object_obj_mycelium_controller_Create_0 @ 0x140662b80
function anon__18703__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  local hit = self.myc_get_mycelium_hit(self, other, px, py, self.myc_mycelium_pick_radius)
  if not gm.bool(hit.hit) then
    return false
  end
  local sb = self.myc_get_surface_build_at(self, other, hit.x, hit.y)
  self.myc_clear_prompts(self, other)
  self.myc_build_node_x = hit.x
  self.myc_build_node_y = hit.y
  self.myc_build_surface_x = sb.x
  self.myc_build_surface_y = sb.y
  self.myc_build_surface_ok = gm.bool(sb.hit) and gm.bool(self.myc_surface_buildable_at(self, other, sb.x, sb.y))
  self.myc_build_options = self.myc_build_options_at(self, other)
  self.myc_mushroom_prompt = (gm.is_array(self.myc_build_options) and gm.array_length(self.myc_build_options) > 0)
    or self.myc_build_hint ~= ""
  return true
end

-- gml_Script____struct___483__anon__19580__gml_Object_obj_mycelium_controller_Create_0 @ 0x140663670
function ___struct___483__anon__19580__gml_Object_obj_mycelium_controller_Create_0(self, other, type)
  self.kind = "node"
  self.type = type
end

-- gml_Script_anon__19580__gml_Object_obj_mycelium_controller_Create_0 @ 0x140663820
function anon__19580__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  local hit = self.myc_get_mycelium_hit(self, other, px, py, self.myc_mycelium_pick_radius)
  if not gm.bool(hit.hit) or gm.bool(world_water_shown_at(self, other, hit.x, hit.y)) then
    return false
  end
  if gm.bool(self.myc_has_knot_near(self, other, hit.x, hit.y, self.myc_knot_radius * 2)) then
    return false
  end
  self.myc_clear_prompts(self, other)
  self.myc_build_node_x = hit.x
  self.myc_build_node_y = hit.y
  global.___struct___483 = gm.method(gm.undefined, ___struct___483__anon__19580__gml_Object_obj_mycelium_controller_Create_0)
  return self.myc_apply_build_option(self, other, gm.new(___struct___483__anon__19580__gml_Object_obj_mycelium_controller_Create_0, -1.0))
end

-- gml_Script_anon__20040__gml_Object_obj_mycelium_controller_Create_0 @ 0x140664260
function anon__20040__gml_Object_obj_mycelium_controller_Create_0(self, other, opt)
  if opt.kind == "node" then
    local cost = myc_node_food_cost(self, other)
    if global.food >= cost then
      if gm.bool(self.myc_add_knot(self, other, self.myc_build_node_x, self.myc_build_node_y)) then
        food_spend(self, other, cost)
        tutorial_event(self, other, "node", self.myc_build_node_x, self.myc_build_node_y)
        return true
      end
    end
    return false
  else
    local ok = self.myc_build_type_at(self, other, opt.type, self.myc_build_surface_x, self.myc_build_surface_y)
    if gm.bool(ok) then
      tutorial_event(self, other, "mushroom", gm.undefined, gm.undefined)
    end
    return ok
  end
end

-- gml_Script____struct___484__anon__21041__gml_Object_obj_mycelium_controller_Create_0 @ 0x140664b80
function ___struct___484__anon__21041__gml_Object_obj_mycelium_controller_Create_0(self, other, host, x, y, face, free)
  self.host = host
  self.x = x
  self.y = y
  self.face = face
  self.free = free
end

-- gml_Script____struct___485__anon__21041__gml_Object_obj_mycelium_controller_Create_0 @ 0x140664e90
function ___struct___485__anon__21041__gml_Object_obj_mycelium_controller_Create_0(self, other, host, x, y, face, free)
  self.host = host
  self.x = x
  self.y = y
  self.face = face
  self.free = free
end

-- gml_Script_anon__21041__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406651a0
function anon__21041__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if not gm.bool(gm.instance_exists(obj_forest_controller)) then
    return gm.undefined
  end
  local fc = gm.instance_find(obj_forest_controller, 0)
  local host = fc.forest_find_host_near(fc, other, px, bal(self, other, "tree_build_reach_px"))
  if gm.is_undefined(host) then
    return gm.undefined
  end
  local list = consumptors_on(self, other, host)
  local has_room = gm.array_length(list) < consume_info(self, other, host).slots
  if gm.bool(gm.variable_struct_exists(host, "part")) then
    local best = gm.undefined
    local best_d = gm.infinity
    local best_free = false
    local i = 0
    while i < gm.array_length(host.part.slots) do
      local s = fc.stump_slot_at(fc, other, host, i)
      local taken = false
      local j = 0
      while j < gm.array_length(list) do
        if not taken then
          taken = gm.point_distance(list[j].x, list[j].y, s.x, s.y) < 2
        else
          taken = true
        end
        j = j + 1
      end
      local d = gm.point_distance(px, py, s.x, s.y)
      if (not taken and not best_free) or ((not taken) == best_free and d < best_d) then
        best = s
        best_d = d
        best_free = not taken
      end
      i = i + 1
    end
    global.___struct___484 = gm.method(gm.undefined, ___struct___484__anon__21041__gml_Object_obj_mycelium_controller_Create_0)
    return gm.new(___struct___484__anon__21041__gml_Object_obj_mycelium_controller_Create_0,
      host, best.x, best.y, best.face, best_free and has_room)
  end
  local side = (px >= host.x) and 1.0 or -1.0
  local top = host.y - gm.sprite_get_yoffset(host.sprite) + bal(self, other, "consumptor_tree_top_row")
  local foot = host.y - vis(self, other, "consumptor_tree_foot_gap_px")
  local cy = gm.clamp(gm.round(py), top, foot)
  local ok = has_room
  local i = 0
  while i < gm.array_length(list) do
    local cs = gm.variable_struct_get(list[i], "side")
    if gm.is_undefined(cs) then cs = 1.0 end
    if cs == side then
      if gm.abs(list[i].y - cy) < bal(self, other, "consumptor_min_gap_px") then
        ok = false
      end
    end
    i = i + 1
  end
  local ex = squirrel_edge_x(self, other, host, side, cy)
  global.___struct___485 = gm.method(gm.undefined, ___struct___485__anon__21041__gml_Object_obj_mycelium_controller_Create_0)
  return gm.new(___struct___485__anon__21041__gml_Object_obj_mycelium_controller_Create_0, host, gm.round(ex), cy, side, ok)
end

-- gml_Script____struct___486__anon__23310__gml_Object_obj_mycelium_controller_Create_0 @ 0x140667e70
function ___struct___486__anon__23310__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, host, root_x, from_x, from_y)
  self.ok = false
  self.x = x
  self.y = y
  self.face = 1.0
  self.host = host
  self.root_x = root_x
  self.from_x = from_x
  self.from_y = from_y
end

-- gml_Script_anon__23310__gml_Object_obj_mycelium_controller_Create_0 @ 0x140668280
function anon__23310__gml_Object_obj_mycelium_controller_Create_0(self, other, t, px, py)
  if gm.is_undefined(py) then py = self.myc_surface_y end
  global.___struct___486 = gm.method(gm.undefined, ___struct___486__anon__23310__gml_Object_obj_mycelium_controller_Create_0)
  local plan = gm.new(___struct___486__anon__23310__gml_Object_obj_mycelium_controller_Create_0,
    px, self.myc_surface_y, gm.undefined, px, px, self.myc_surface_y)
  local root_x = px
  local continue_checks = true
  if t == 7 then
    local spot = self.myc_consumptor_spot(self, other, px, py)
    if gm.is_undefined(spot) then
      return plan
    end
    plan.x = spot.x
    plan.y = spot.y
    plan.face = spot.face
    plan.host = spot.host
    root_x = spot.host.x
    if not gm.bool(spot.free) then
      plan.root_x = root_x
      continue_checks = false
    end
  end
  if continue_checks then
    if t == 3 then
      local tree = self.myc_find_tree_near(self, other, px)
      if gm.is_undefined(tree) then
        return plan
      end
      plan.x = tree.x + vis(self, other, "tree_mushroom_offset_x")
      plan.y = self.myc_surface_y - vis(self, other, "tree_mushroom_height_px")
      root_x = tree.x
    end
    plan.root_x = root_x
    if gm.bool(self.myc_can_build_type(self, other, t, px, self.myc_surface_y))
      and gm.bool(self.myc_has_water_source(self, other)) then
      if t ~= 3 and t ~= 7 then
        if gm.bool(self.myc_has_mushroom_near(self, other, plan.x, self.myc_surface_y, bal(self, other, "mushroom_min_gap_px"))) then
          return plan
        end
      end
      local hit = self.myc_get_mycelium_hit(self, other, root_x, self.myc_surface_y, bal(self, other, "mushroom_build_reach_px"))
      if gm.bool(hit.hit) and gm.bool(self.myc_segment_is_playable(self, other, hit.x, hit.y, root_x, self.myc_surface_y + 2, true)) then
        plan.ok = true
        plan.from_x = hit.x
        plan.from_y = hit.y
        return plan
      end
      return plan
    end
  end
  return plan
end

-- gml_Script____struct___487__anon__25115__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066a600
function ___struct___487__anon__25115__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___488__anon__25115__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066a800
function ___struct___488__anon__25115__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___489__anon__25115__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066aa00
function ___struct___489__anon__25115__gml_Object_obj_mycelium_controller_Create_0(self, other, duration, finish_age, sound, branches, vbuf, bx1, by1, bx2, by2)
  self.age = 0
  self.duration = duration
  self.finish_age = finish_age
  self.sound = sound
  self.branches = branches
  self.scanned_length = 0
  self.finished = false
  self.budding = false
  self.no_length = true
  self.vbuf = vbuf
  self.bx1 = bx1
  self.by1 = by1
  self.bx2 = bx2
  self.by2 = by2
end

-- gml_Script_anon__25115__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066b050
function anon__25115__gml_Object_obj_mycelium_controller_Create_0(self, other, x1, y1, x2, y2)
  local dist = gm.point_distance(x1, y1, x2, y2)
  if dist < 2.0 then
    return
  end
  local thick = vis(self, other, "strand_thickness") * 0.8
  local br = self.myc_make_branch(self, other, x1, y1, gm.point_direction(x1, y1, x2, y2), dist,
    gm.max(4.0, gm.round(dist / 24.0)), dist * vis(self, other, "growth_jitter") * 0.1, 0, thick)
  local pts = br.points
  if gm.is_array(pts) and gm.array_length(pts) > 1 then
    global.___struct___487 = gm.method(gm.undefined, ___struct___487__anon__25115__gml_Object_obj_mycelium_controller_Create_0)
    pts[gm.array_length(pts) - 1] = gm.new(___struct___487__anon__25115__gml_Object_obj_mycelium_controller_Create_0, x2, y2)
  else
    global.___struct___488 = gm.method(gm.undefined, ___struct___488__anon__25115__gml_Object_obj_mycelium_controller_Create_0)
    gm.array_push(pts, gm.new(___struct___488__anon__25115__gml_Object_obj_mycelium_controller_Create_0, x2, y2))
  end
  local vb = self.myc_build_branch(self, other, pts, 0, thick)
  local bx1, by1, bx2, by2 = vb.bx1, vb.by1, vb.bx2, vb.by2
  global.___struct___489 = gm.method(gm.undefined, ___struct___489__anon__25115__gml_Object_obj_mycelium_controller_Create_0)
  local snd = hyphae_sound_begin(self, other, dist, self.myc_growth_duration + 2, gm.undefined, gm.undefined)
  gm.array_push(self.myc_growths, gm.new(___struct___489__anon__25115__gml_Object_obj_mycelium_controller_Create_0,
    self.myc_growth_duration, self.myc_growth_duration + 2, snd, gm.array(vb), -1.0, bx1, by1, bx2, by2))
  gm.array_push(self.myc_active, self.myc_growths[gm.array_length(self.myc_growths) - 1])
  self.myc_grid_add(self, other, self.myc_growths[gm.array_length(self.myc_growths) - 1])
end

-- gml_Script_anon__26361__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066cd80
function anon__26361__gml_Object_obj_mycelium_controller_Create_0(self, other, t, px, py)
  if gm.is_undefined(py) then py = self.myc_surface_y end
  local plan = self.myc_place_check(self, other, t, px, py)
  if not gm.bool(plan.ok) then
    return false
  end
  if t == 7 then
    food_spend(self, other, mushroom_type_cost(self, other, t))
    water_spend(self, other, mushroom_type_water_cost(self, other, t))
    local idx = mushroom_add(self, other, t, plan.x, plan.y)
    global.mushrooms[idx].host = plan.host
    global.mushrooms[idx].side = plan.face
    local variant = consumptor_variant(self, other, plan.face)
    global.mushrooms[idx].variant = variant
  else
    local bx
    if t == 3 then
      bx = px
    else
      bx = plan.x
    end
    if not gm.bool(self.myc_build_type_at(self, other, t, bx, self.myc_surface_y)) then
      return false
    end
  end
  self.myc_add_link_growth(self, other, plan.from_x, plan.from_y, plan.root_x, self.myc_surface_y)
  tutorial_event(self, other, "mushroom", gm.undefined, gm.undefined)
  return true
end

-- gml_Script_anon__27346__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066e200
function anon__27346__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local bonus = upg_bonus(self, other, "spore_quality")
  return gm.clamp(bal(self, other, "spore_base_node_chance") + bonus, 0, 1.0)
end

-- gml_Script____struct___490__anon__27471__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066e470
function ___struct___490__anon__27471__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, vx, vy, weight, life, phase)
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.weight = weight
  self.age = 0
  self.life = life
  self.phase = phase
end

-- gml_Script_anon__27471__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066e8a0
function anon__27471__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  if not gm.bool(food_spend(self, other, bal(self, other, "spore_release_cost"))) then
    return false
  end
  tasks_stat_add(self, other, "spores", gm.undefined)
  local count_bonus = upg_bonus(self, other, "spore_count")
  local count = gm.round(bal(self, other, "spore_base_count") * (1.0 + count_bonus))
  local light = gm.max(0.15, 1.0 - upg_bonus(self, other, "spore_lightness"))
  local weight = bal(self, other, "spore_base_weight") * light
  local i = 0
  while i < count do
    global.___struct___490 = gm.method(gm.undefined, ___struct___490__anon__27471__gml_Object_obj_mycelium_controller_Create_0)
    local sx = px + gm.random_range(-9.0, 9.0)
    local sy = py - gm.random_range(42.0, 64.0)
    local svx = gm.random_range(-0.42, 0.42) + global.wind_dir * global.wind_speed * 0.09
    local svy = gm.random_range(-1.55, -0.35)
    local life = gm.irandom_range(620, 920)
    local phase = gm.random(360.0)
    gm.array_push(self.myc_spores, gm.new(___struct___490__anon__27471__gml_Object_obj_mycelium_controller_Create_0,
      sx, sy, svx, svy, weight, life, phase))
    i = i + 1
  end
  return true
end
