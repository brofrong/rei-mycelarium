local gm = require("gm")

-- GML `a ?? b`
local function nz(v, d)
  if gm.is_undefined(v) then return d end
  return v
end

-- gml_Script_tasks_step @ 0x14006a640
function tasks_step(self, other)
  local t = global.tasks
  if game_mode(self, other) == "survival" then
    return
  end
  if gm.bool(global.title_open) or gm.bool(global.intro_active) or gm.bool(global.pause_open) or gm.bool(global.logo_open) then
    return
  end
  if not gm.bool(t.shown) and tutorial_state(self, other, "final") ~= 0 then
    t.shown = true
  end
  if not gm.bool(t.shown) then
    return
  end
  t.alpha = gm.min(1, t.alpha + 1 / gm.max(1, vis(self, other, "tasks_fade_frames")))
  local ids = gm.array("tasks", "migration", "magic")
  local i = 0
  while i < 3 do
    local id = ids[i]
    local dir = gm.bool(tasks_block_open(self, other, id)) and 1 or -1
    local k = nz(gm.variable_struct_get(t.fold_k, id), 1) + dir / gm.max(1, vis(self, other, "tasks_fold_frames"))
    gm.variable_struct_set(t.fold_k, id, gm.clamp(k, 0, 1))
    i = i + 1
  end
  if t.stage == 1 then
    local g = gm.array("migration", "magic")
    local j = 0
    while j < 2 do
      if not gm.bool(nz(gm.variable_struct_get(t.goals, g[j]), 0)) and gm.bool(goal_met(self, other, g[j])) then
        gm.variable_struct_set(t.goals, g[j], true)
        t.popup = (g[j] == "migration") and "migration_ready" or g[j]
        break
      end
      j = j + 1
    end
  else
    local defs = tasks_defs(self, other)
    local all = true
    local n = 0
    while n < gm.array_length(defs) do
      local d = defs[n]
      if not gm.bool(nz(gm.variable_struct_get(t.done, d), 0)) then
        if gm.bool(tasks_met(self, other, d)) then
          gm.variable_struct_set(t.done, d, true)
          gm.variable_struct_set(t.fill, d, 0)
          global.upgrade_points = global.upgrade_points + bal(self, other, "task_reward_points")
        end
        if not gm.bool(nz(gm.variable_struct_get(t.done, d), false)) then
          all = false
        end
      end
      n = n + 1
    end
    if all then
      t.stage = 1
      t.popup = "tasks"
    end
  end
end

-- gml_Script_tasks_block_open @ 0x14006cc60
function tasks_block_open(self, other, id)
  if id == "tasks" then
    return global.tasks.open
  end
  return nz(gm.variable_struct_get(global.tasks.goal_open, id), true)
end

-- gml_Script_tasks_block_toggle @ 0x14006d100
function tasks_block_toggle(self, other, id)
  local t = global.tasks
  if id == "tasks" then
    t.open = not gm.bool(t.open)
    return
  end
  gm.variable_struct_set(t.goal_open, id, not gm.bool(tasks_block_open(self, other, id)))
end

-- gml_Script____struct___30__tasks_block_items__scr_game @ 0x14006d4e0
function ___struct___30__tasks_block_items__scr_game(self, other, key, text, done)
  self.key = key
  self.text = text
  self.done = done
  self.reward = true
end

-- gml_Script____struct___31__tasks_block_items__scr_game @ 0x14006d760
function ___struct___31__tasks_block_items__scr_game(self, other, key, done, text)
  self.key = key
  self.reward = false
  self.done = done
  self.text = text
end

-- gml_Script____struct___32__tasks_block_items__scr_game @ 0x14006d9f0
function ___struct___32__tasks_block_items__scr_game(self, other, key, text, done)
  self.key = key
  self.text = text
  self.done = done
  self.reward = false
end

-- gml_Script_tasks_block_items @ 0x14006dc70
function tasks_block_items(self, other, block)
  local items = gm.array()
  if block == "tasks" then
    local defs = tasks_defs(self, other)
    local i = 0
    while i < gm.array_length(defs) do
      local key = "task_" .. defs[i]
      local text = loc(self, other, key)
      local done = nz(gm.variable_struct_get(global.tasks.done, defs[i]), false)
      local item = gm.new(___struct___30__tasks_block_items__scr_game, defs[i], text, done)
      gm.array_push(items, item)
      i = i + 1
    end
  elseif block == "migration" then
    local have = gm.array(goal_spore_count(self, other), water_total(self, other), global.food)
    local need = gm.array(bal(self, other, "migration_spore_mushrooms"), bal(self, other, "migration_water"),
      bal(self, other, "migration_food"))
    local ks = gm.array("spores", "water", "food")
    local i = 0
    while i < 3 do
      local key = "goal_item_" .. ks[i]
      local n = gm.min(have[i], need[i])
      local done = have[i] >= need[i]
      local text = loc(self, other, key) .. "  " .. goal_number_text(self, other, n) .. " / "
        .. goal_number_text(self, other, need[i])
      local item = gm.new(___struct___31__tasks_block_items__scr_game, "m:" .. ks[i], done, text)
      gm.array_push(items, item)
      i = i + 1
    end
  elseif block == "magic" then
    local c = gm.array("green", "yellow", "purple", "blue", "red")
    local i = 0
    while i < 5 do
      local key = "goal_item_" .. c[i]
      local text = loc(self, other, key)
      local done = perk_has(self, other, c[i])
      local item = gm.new(___struct___32__tasks_block_items__scr_game, "p:" .. c[i], text, done)
      gm.array_push(items, item)
      i = i + 1
    end
  end
  return items
end

-- gml_Script____struct___33__tasks_blocks__scr_game @ 0x1400706d0
function ___struct___33__tasks_blocks__scr_game(self, other, id, title, y, k)
  self.id = id
  self.title = title
  self.y = y
  self.k = k
end

-- gml_Script_tasks_blocks @ 0x140070980
function tasks_blocks(self, other)
  local t = global.tasks
  local ids
  if t.stage == 0 then
    ids = gm.array("tasks")
  elseif t.popup == "tasks" then
    ids = gm.array()
  else
    ids = gm.array("migration", "magic")
  end
  local y = vis(self, other, "tasks_y")
  local out = gm.array()
  local i = 0
  while i < gm.array_length(ids) do
    local id = ids[i]
    local k = nz(gm.variable_struct_get(t.fold_k, id), 1)
    local n
    if id == "tasks" then
      n = gm.array_length(tasks_defs(self, other))
    elseif id == "migration" then
      n = 3
    else
      n = 5
    end
    local title
    if id == "tasks" then
      title = "tasks_title"
    else
      title = "goal_" .. id .. "_label"
    end
    gm.array_push(out, gm.new(___struct___33__tasks_blocks__scr_game, id, loc(self, other, title), y, k))
    y = y + ((vis(self, other, "tasks_first_gap") + (n - 1) * vis(self, other, "tasks_line_px")) * k + vis(self, other, "tasks_block_gap"))
    i = i + 1
  end
  return out
end

-- gml_Script____struct___34__tasks_header_rect__scr_game @ 0x1400722e0
function ___struct___34__tasks_header_rect__scr_game(self, other, x1, y1, x2, y2, text_w)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.text_w = text_w
end

-- gml_Script_tasks_header_rect @ 0x1400725e0
function tasks_header_rect(self, other, b)
  gm.draw_set_font(ui_font(self, other))
  local x = vis(self, other, "tasks_x") + global.ui_ox
  local scale = vis(self, other, "tasks_title_scale")
  local tw = gm.string_width(b.title) * scale
  local th = gm.string_height("A") * scale
  gm.draw_set_font(ui_font_small(self, other))
  local by = b.y
  return gm.new(___struct___34__tasks_header_rect__scr_game,
    x - 8,
    by - th * 0.5 - 6,
    x + tw + vis(self, other, "tasks_arrow_gap") + vis(self, other, "tasks_arrow_px") + 8,
    by + th * 0.5 + 6,
    tw)
end

-- gml_Script_tasks_visible @ 0x140073690
function tasks_visible(self, other)
  return gm.bool(gm.variable_global_exists("tasks")) and gm.bool(global.tasks.shown) and not gm.bool(global.hud_hidden)
    and not gm.bool(global.title_open) and not gm.bool(global.intro_active) and game_mode(self, other) ~= "survival"
end

-- gml_Script_tasks_mouse_step @ 0x1400739d0
function tasks_mouse_step(self, other, mx, my)
  if not gm.bool(tasks_visible(self, other)) or gm.bool(ui_blocking(self, other)) then
    return false
  end
  local blocks = tasks_blocks(self, other)
  local i = 0
  while i < gm.array_length(blocks) do
    local r = tasks_header_rect(self, other, blocks[i])
    if gm.bool(gm.point_in_rectangle(mx, my, r.x1, r.y1, r.x2, r.y2)) then
      if gm.bool(gm.mouse_check_button_pressed(gm.mb_left)) then
        tasks_block_toggle(self, other, blocks[i].id)
      end
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script_tasks_draw @ 0x1400743a0
function tasks_draw(self, other, a0, a1, a2, a3)
  if not gm.bool(tasks_visible(self, other)) then
    return
  end
  local t = global.tasks
  local blocks = tasks_blocks(self, other)
  local step = 1 / gm.max(1, vis(self, other, "tasks_fill_frames"))
  local i = 0
  while i < gm.array_length(blocks) do
    local b = blocks[i]
    tasks_block_draw(self, other, b, tasks_block_items(self, other, b.id), a0, a1, a2, a3, step)
    i = i + 1
  end
  gm.draw_set_alpha(1)
  gm.draw_set_font(ui_font_small(self, other))
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_valign(gm.fa_top)
end

-- gml_Script_tasks_block_draw @ 0x140074d30
function tasks_block_draw(self, other, b, items, c_text, c_reward, c_hover, c_box, step)
  local t = global.tasks
  local a = t.alpha
  local r = tasks_header_rect(self, other, b)
  local x = vis(self, other, "tasks_x") + global.ui_ox
  local y = b.y
  local ts = vis(self, other, "tasks_title_scale")
  local xs = vis(self, other, "tasks_text_scale")
  local lp = vis(self, other, "tasks_line_px")
  local box = vis(self, other, "tasks_box_px") * 0.5
  local ap = vis(self, other, "tasks_arrow_px")
  local hov = gm.point_in_rectangle(gm.device_mouse_x_to_gui(0), gm.device_mouse_y_to_gui(0), r.x1, r.y1, r.x2, r.y2)
  local k = b.k
  gm.draw_set_font(ui_font(self, other))
  gm.draw_set_valign(gm.fa_middle)
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_colour(0)
  gm.draw_set_alpha(a * 0.6)
  gm.draw_text_transformed(x + 3, y + 3, b.title, ts, ts, 0)
  gm.draw_set_colour(gm.bool(hov) and c_hover or c_text)
  gm.draw_set_alpha(a)
  gm.draw_text_transformed(x, y, b.title, ts, ts, 0)
  local ax = x + r.text_w + vis(self, other, "tasks_arrow_gap") + ap * 0.5
  local off = ap * 0.45 * gm.lerp(1, -1, 1 - k)
  gm.draw_triangle(ax - ap * 0.5, y + off, ax + ap * 0.5, y + off, ax, y - off, false)
  local i = 0
  while i < gm.array_length(items) do
    local it = items[i]
    local target = gm.bool(it.done) and 1 or 0
    local cur = nz(gm.variable_struct_get(t.fill, it.key), target)
    local v
    if cur >= target then
      v = gm.max(target, cur - step)
    else
      v = gm.min(target, cur + step)
    end
    gm.variable_struct_set(t.fill, it.key, v)
    i = i + 1
  end
  if not (k > 0) then
    return
  end
  local tx = x + box * 2 + vis(self, other, "tasks_box_gap")
  local icon = vis(self, other, "tasks_reward_icon_px") / gm.max(1, gm.sprite_get_height(UPGRADE))
  local ia = a * k * k
  local maxw = gm.display_get_gui_width() * 0.34 - (tx - x) - 60
  i = 0
  while i < gm.array_length(items) do
    xs = goal_fit(self, other, items[i].text, xs, maxw, 0, -1)
    i = i + 1
  end
  i = 0
  while i < gm.array_length(items) do
    local it = items[i]
    local f = nz(gm.variable_struct_get(t.fill, it.key), 0)
    local iy = y + (vis(self, other, "tasks_first_gap") + i * lp) * k
    local cx = x + box
    gm.draw_set_alpha(ia)
    gm.draw_set_colour(c_text)
    gm.draw_circle(cx, iy, box, false)
    gm.draw_set_colour(c_box)
    gm.draw_circle(cx, iy, box - 2, false)
    if f > 0 then
      gm.draw_set_colour(c_hover)
      gm.draw_circle(cx, iy, gm.max(1, (box - 2) * f), false)
    end
    gm.draw_set_colour(0)
    gm.draw_set_alpha(ia * 0.6)
    gm.draw_text_transformed(tx + 2, iy + 2, it.text, xs, xs, 0)
    gm.draw_set_colour((gm.bool(it.done) and gm.bool(it.reward)) and c_reward or c_text)
    gm.draw_set_alpha(ia)
    gm.draw_text_transformed(tx, iy, it.text, xs, xs, 0)
    if gm.bool(it.done) and gm.bool(it.reward) then
      local rx = tx + gm.string_width(it.text) * xs + vis(self, other, "tasks_reward_gap")
      local pulse = math.sin(f * math.pi) * 0.35 + 1.0
      local rs = xs * pulse
      local s = "+" .. gm.string(bal(self, other, "task_reward_points"))
      gm.draw_set_alpha(ia * f)
      gm.draw_set_colour(c_hover)
      gm.draw_text_transformed(rx, iy, s, rs, rs, 0)
      local isc = icon * pulse
      gm.draw_sprite_ext(UPGRADE, 0, rx + gm.string_width(s) * rs + 4, iy - gm.sprite_get_height(UPGRADE) * isc * 0.5,
        isc, isc, 0, 0xffffff, ia * f)
    end
    i = i + 1
  end
end

-- gml_Script_goal_spore_count @ 0x14007b530
function goal_spore_count(self, other)
  local n = 0
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 0 and not gm.bool(m.stolen) and m.carried_by == -1 then
      n = n + 1
    end
    i = i + 1
  end
  if gm.bool(nz(gm.variable_struct_get(global.tasks, "debug_spores"), false)) then
    n = gm.max(n, bal(self, other, "migration_spore_mushrooms"))
  end
  return n
end

-- gml_Script_goal_met @ 0x14007be00
function goal_met(self, other, id)
  if id == "migration" then
    local need = bal(self, other, "migration_spore_mushrooms")
    if not (goal_spore_count(self, other) >= need) then return false end
    if not (global.food >= bal(self, other, "migration_food")) then return false end
    local need_w = bal(self, other, "migration_water")
    return water_total(self, other) >= need_w
  end
  local defs = cave_defs(self, other)
  local i = 0
  while i < gm.array_length(defs) do
    if not gm.bool(perk_has(self, other, defs[i].id)) then
      return false
    end
    i = i + 1
  end
  return true
end

-- gml_Script_tasks_popup_open @ 0x14007c5c0
function tasks_popup_open(self, other)
  return gm.bool(gm.variable_global_exists("tasks")) and global.tasks.popup ~= "" and game_mode(self, other) ~= "survival"
end

-- gml_Script_goal_number_text @ 0x14007c900
function goal_number_text(self, other, v)
  local s = gm.string(gm.round(v))
  local out = ""
  while gm.string_length(s) > 3 do
    out = "." .. gm.string_copy(s, gm.string_length(s) - 2, 3) .. out
    s = gm.string_copy(s, 1, gm.string_length(s) - 3)
  end
  return s .. out
end

-- gml_Script____struct___35__tasks_popup_rect__scr_game @ 0x14007d2a0
function ___struct___35__tasks_popup_rect__scr_game(self, other, x1, y1, x2, y2, cx)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.cx = cx
end

-- gml_Script_tasks_popup_rect @ 0x14007d590
function tasks_popup_rect(self, other)
  local cx = gm.display_get_gui_width() * 0.5
  local cy = gm.display_get_gui_height() * 0.5
  local is_tasks = global.tasks.popup == "tasks"
  local w = is_tasks and 1300 or 1100
  local h = is_tasks and 640 or 300
  local y1 = cy - (h + 100) * 0.5
  local x1 = cx - w * 0.5
  local x2 = cx + w * 0.5
  local y2 = y1 + h
  return gm.new(___struct___35__tasks_popup_rect__scr_game, x1, y1, x2, y2, cx)
end

-- gml_Script____struct___36__tasks_popup_buttons__scr_game @ 0x14007e260
function ___struct___36__tasks_popup_buttons__scr_game(self, other, label, x1, y1, x2, y2)
  self.id = "close"
  self.label = label
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script____struct___37__tasks_popup_buttons__scr_game @ 0x14007e5a0
function ___struct___37__tasks_popup_buttons__scr_game(self, other, label, x1, y1, x2, y2)
  self.id = "spores"
  self.label = label
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script____struct___38__tasks_popup_buttons__scr_game @ 0x14007e8e0
function ___struct___38__tasks_popup_buttons__scr_game(self, other, label, x1, y1, x2, y2)
  self.id = "close"
  self.label = label
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_tasks_popup_buttons @ 0x14007ec20
function tasks_popup_buttons(self, other)
  local r = tasks_popup_rect(self, other)
  local w = 420
  local h = 64
  local y = r.y2 + 28
  local out = gm.array()
  if global.tasks.popup == "migration_end" then
    y = gm.display_get_gui_height() - h - 72
  end
  local ctor, label
  if global.tasks.popup == "tasks" then
    ctor, label = ___struct___36__tasks_popup_buttons__scr_game, "goal_button_grow"
  elseif global.tasks.popup == "migration_ready" then
    ctor, label = ___struct___37__tasks_popup_buttons__scr_game, "goal_button_spores"
  else
    ctor, label = ___struct___38__tasks_popup_buttons__scr_game, "goal_button_continue"
  end
  gm.array_push(out, gm.new(ctor, loc(self, other, label), r.cx - w * 0.5, y, r.cx + w * 0.5, y + h))
  return out
end

-- gml_Script_tasks_popup_close @ 0x140080410
function tasks_popup_close(self, other)
  local t = global.tasks
  t.popup = ""
  if gm.bool(cine_active(self, other)) and t.cine.phase == "hold" then
    t.cine.phase = "out"
    t.cine.t = 0
  end
end

-- gml_Script_tasks_popup_step @ 0x1400807c0
function tasks_popup_step(self, other, mx, my, click)
  local t = global.tasks
  if gm.bool(gm.keyboard_check_pressed(27)) or gm.bool(gm.keyboard_check_pressed(13)) then
    if t.popup == "migration_ready" then
      t.popup = ""
      cine_show_start(self, other)
    else
      tasks_popup_close(self, other)
    end
    return
  end
  if not gm.bool(click) then
    return
  end
  local btns = tasks_popup_buttons(self, other)
  local i = 0
  while i < gm.array_length(btns) do
    local b = btns[i]
    if gm.bool(gm.point_in_rectangle(mx, my, b.x1, b.y1, b.x2, b.y2)) then
      if b.id == "spores" then
        t.popup = ""
        cine_show_start(self, other)
        return
      end
      tasks_popup_close(self, other)
      if b.id == "exit" then
        save_game(self, other)
        gm.game_end()
      end
      return
    end
    i = i + 1
  end
end

-- gml_Script_goal_text_scale @ 0x140081460
function goal_text_scale(self, other, h)
  return h / gm.max(1, gm.string_height("A"))
end

-- gml_Script_goal_sprite_fit @ 0x140081710
function goal_sprite_fit(self, other, spr, cx, cy, w, h)
  local sw = gm.sprite_get_width(spr)
  local sh = gm.sprite_get_height(spr)
  local kx = w / gm.max(1, sw)
  local ky = h / gm.max(1, sh)
  local s = gm.min(kx, ky)
  local dy = (sh * 0.5 - gm.sprite_get_yoffset(spr)) * s
  local dx = (sw * 0.5 - gm.sprite_get_xoffset(spr)) * s
  local x = cx - dx
  local y = cy - dy
  gm.draw_sprite_ext(spr, 0, x, y, s, s, 0, 0xffffff, 1)
end

-- gml_Script_goal_sprite_part_fit @ 0x1400823b0
function goal_sprite_part_fit(self, other, spr, left, top, pw, ph, cx, cy, w, h)
  local kx = w / gm.max(1, pw)
  local ky = h / gm.max(1, ph)
  local s = gm.min(kx, ky)
  local x = cx - pw * s * 0.5
  local y = cy - ph * s * 0.5
  local col = 16777215
  local alpha = 1
  gm.draw_sprite_part_ext(spr, 0, left, top, pw, ph, x, y, s, s, col, alpha)
end

-- gml_Script_goal_fit @ 0x140082eb0
function goal_fit(self, other, text, scale, maxw, maxh, sep)
  local i = 0
  while i < 16 and scale > 0.05 do
    local w, h
    if sep >= 0 then
      w = gm.string_width_ext(text, sep, maxw / scale) * scale
    else
      w = gm.string_width(text) * scale
    end
    if sep >= 0 then
      h = gm.string_height_ext(text, sep, maxw / scale) * scale
    else
      h = gm.string_height(text) * scale
    end
    if w <= maxw + 1 then
      if maxh <= 0 then break end
      if h <= maxh + 1 then break end
    end
    scale = scale * 0.92
    i = i + 1
  end
  return scale
end

-- gml_Script_tasks_popup_draw @ 0x140083ed0
function tasks_popup_draw(self, other, c_text, c_btn_text, c_btn)
  if not gm.bool(tasks_popup_open(self, other)) then
    return
  end
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  local r = tasks_popup_rect(self, other)
  local cx = r.cx
  local mx = gm.device_mouse_x_to_gui(0)
  local my = gm.device_mouse_y_to_gui(0)
  gm.draw_set_font(ui_font(self, other))
  gm.draw_set_halign(gm.fa_center)
  gm.draw_set_valign(gm.fa_middle)
  local is_end = global.tasks.popup == "migration_end"
  if not is_end then
    gm.draw_set_colour(0)
    gm.draw_set_alpha(0.55)
    gm.draw_rectangle(0, 0, gw, gh, false)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(0x2a3132)
    gm.draw_roundrect_ext(r.x1, r.y1, r.x2, r.y2, 90, 90, false)
  end
  gm.draw_set_colour(c_text)
  if not is_end then
    if global.tasks.popup == "tasks" then
      local s = goal_text_scale(self, other, 36)
      s = goal_fit(self, other, loc(self, other, "goal_tasks_title"), s, 1240, 0, -1)
      gm.draw_text_transformed(cx, r.y1 + 58, loc(self, other, "goal_tasks_title"), s, s, 0)
      s = goal_text_scale(self, other, 22)
      local sep = 40 / s
      s = goal_fit(self, other, loc(self, other, "goal_tasks_text"), s, 1240, 160, sep)
      gm.draw_text_ext_transformed(cx, r.y1 + 162, loc(self, other, "goal_tasks_text"), sep, 1240 / s, s, s, 0)
      local ly = r.y1 + 266
      gm.draw_rectangle(cx - 3, ly, cx + 3, r.y2 - 44, false)
      s = goal_text_scale(self, other, 32)
      local lx = cx - 323
      local rx = cx + 320
      sep = 38 / s
      local ts = gm.min(goal_fit(self, other, loc(self, other, "goal_migration_title"), s, 600, 120, sep),
        goal_fit(self, other, loc(self, other, "goal_magic_title"), s, 600, 120, sep))
      gm.draw_text_ext_transformed(lx, ly + 55, loc(self, other, "goal_migration_title"), sep, 600 / ts, ts, ts, 0)
      gm.draw_text_ext_transformed(rx, ly + 55, loc(self, other, "goal_magic_title"), sep, 600 / ts, ts, ts, 0)
      local icons = gm.array(-1, apple_bite, water65)
      local needs = gm.array(bal(self, other, "migration_spore_mushrooms"), bal(self, other, "migration_food"), bal(self, other, "migration_water"))
      local xs = gm.array(cx - 542, cx - 373, cx - 152)
      local ns = goal_text_scale(self, other, 30)
      local i = 0
      while i < 3 do
        if i == 0 then
          goal_sprite_part_fit(self, other, sporeshroom, 4, 15, 38, 49, xs[i], ly + 207, 76, 88)
        else
          goal_sprite_fit(self, other, icons[i], xs[i], ly + 207, 80, 84)
        end
        gm.draw_text_transformed(xs[i], ly + 287, goal_number_text(self, other, needs[i]), ns, ns, 0)
        i = i + 1
      end
      goal_sprite_fit(self, other, mush5, rx, ly + 235, 490, 184)
    else
      local s = goal_text_scale(self, other, 26)
      local txt = loc(self, other, (global.tasks.popup == "migration_ready") and "goal_migration_done" or "goal_magic_done")
      local sep = 52 / s
      s = goal_fit(self, other, txt, s, (r.x2 - r.x1) - 40, (r.y2 - r.y1) * 0.55, sep)
      gm.draw_text_ext_transformed(cx, (r.y1 + r.y2) * 0.5, txt, sep, ((r.x2 - r.x1) - 40) / s, s, s, 0)
    end
  end
  local btns = tasks_popup_buttons(self, other)
  local i = 0
  while i < gm.array_length(btns) do
    local b = btns[i]
    local hov = gm.point_in_rectangle(mx, my, b.x1, b.y1, b.x2, b.y2)
    local bs = gm.min(goal_text_scale(self, other, 20), (b.x2 - b.x1 - 32) / gm.max(1, gm.string_width(b.label)))
    gm.draw_set_colour(c_btn)
    gm.draw_set_alpha(0.95)
    gm.draw_rectangle(b.x1, b.y1, b.x2, b.y2, false)
    gm.draw_set_alpha(1)
    gm.draw_set_colour(gm.bool(hov) and c_text or c_btn_text)
    gm.draw_rectangle(b.x1, b.y1, b.x2, b.y2, true)
    gm.draw_text_transformed((b.x1 + b.x2) * 0.5, (b.y1 + b.y2) * 0.5, b.label, bs, bs, 0)
    i = i + 1
  end
  gm.draw_set_alpha(1)
  gm.draw_set_font(ui_font_small(self, other))
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_valign(gm.fa_top)
end

-- gml_Script____struct___39__cine_ball__scr_game @ 0x140087670
function ___struct___39__cine_ball__scr_game(self, other, x, y, r)
  self.x = x
  self.y = y
  self.r = r
end

-- gml_Script_cine_ball @ 0x1400878c0
function cine_ball(self, other)
  local gh = gm.display_get_gui_height()
  return gm.new(___struct___39__cine_ball__scr_game, gm.display_get_gui_width() * 0.5, gh * 0.45,
    gh * vis(self, other, "migration_cine_ball_share"))
end

-- gml_Script_cine_active @ 0x140087ee0
function cine_active(self, other)
  return gm.bool(gm.variable_global_exists("tasks")) and gm.is_struct(gm.variable_struct_get(global.tasks, "cine"))
end

-- gml_Script_cine_blocking @ 0x140088220
function cine_blocking(self, other)
  return gm.bool(cine_active(self, other)) and global.tasks.cine.phase ~= "out"
end

-- gml_Script_cine_showing @ 0x140088420
function cine_showing(self, other)
  return gm.bool(cine_active(self, other)) and global.tasks.cine.phase == "show"
end

-- gml_Script____struct___40__cine_spore_spots__scr_game @ 0x140088620
function ___struct___40__cine_spore_spots__scr_game(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_cine_spore_spots @ 0x140088820
function cine_spore_spots(self, other)
  local out = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 0 and not gm.bool(m.stolen) and m.carried_by == -1 then
      gm.array_push(out, gm.new(___struct___40__cine_spore_spots__scr_game, m.x, m.y))
    end
    i = i + 1
  end
  return out
end

-- gml_Script____struct___41__cine_show_start__scr_game @ 0x140089090
function ___struct___41__cine_show_start__scr_game(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___42__cine_show_start__scr_game @ 0x140089290
function ___struct___42__cine_show_start__scr_game(self, other, x, y, seconds)
  self.x = x
  self.y = y
  self.seconds = seconds
  self.count = 0
end

-- gml_Script____struct___43__cine_show_start__scr_game @ 0x140089510
function ___struct___43__cine_show_start__scr_game(self, other, x, y, seconds, count)
  self.x = x
  self.y = y
  self.seconds = seconds
  self.count = count
end

-- gml_Script____struct___44__cine_show_start__scr_game @ 0x1400897c0
function ___struct___44__cine_show_start__scr_game(self, other, stop, stops)
  self.phase = "show"
  self.t = 0
  self.alpha = 1
  self.stop = stop
  self.shot = false
  self.stops = stops
end

-- gml_Script_cine_show_start @ 0x140089aa0
function cine_show_start(self, other)
  local cam_cx = 0
  local cam_cy = 0
  if gm.bool(gm.instance_exists(obj_forest_controller)) then
    local fc = gm.instance_find(obj_forest_controller, 0)
    local cw = fc.camera_width
    cam_cx = fc.camera_x + cw * 0.5
    cam_cy = fc.camera_y + fc.camera_height * 0.5
  end
  local ox = global.start_x
  local oy = wld(self, other, "surface_height_px")
  if gm.bool(gm.instance_exists(obj_mycelium_controller)) then
    local mc = gm.instance_find(obj_mycelium_controller, 0)
    ox = mc.x
    oy = mc.y
  end
  local spots = cine_spore_spots(self, other)
  local nearest = gm.undefined
  local best = math.huge
  local i = 0
  while i < gm.array_length(spots) do
    local d = gm.point_distance(cam_cx, cam_cy, spots[i].x, spots[i].y)
    if d < best then
      best = d
      nearest = spots[i]
    end
    i = i + 1
  end
  local stops = gm.array()
  local rest = gm.array()
  i = 0
  while i < gm.array_length(spots) do
    if spots[i] ~= nearest then
      gm.array_push(rest, spots[i])
    end
    i = i + 1
  end
  rest = gm.array_shuffle(rest)
  if not gm.is_undefined(nearest) then
    gm.array_push(stops, nearest)
  end
  i = 0
  while i < gm.array_length(rest) and gm.array_length(stops) < vis(self, other, "migration_spore_stops") do
    gm.array_push(stops, rest[i])
    i = i + 1
  end
  while not (gm.array_length(stops) > 1) do
    local side = (gm.array_length(stops) ~= 0) and 1 or -1
    local n = gm.array_length(stops)
    gm.array_push(stops, gm.new(___struct___41__cine_show_start__scr_game, ox + side * (n * 300 + 1400), oy))
  end
  local count = gm.round(bal(self, other, "spore_base_count")
    * (1 + bal(self, other, "upg_spore_count_per_level") * bal(self, other, "upg_spore_count_max_level"))
    * vis(self, other, "migration_origin_spore_mult"))
  local list = gm.array()
  i = 0
  while i < gm.array_length(stops) do
    gm.array_push(list, gm.new(___struct___42__cine_show_start__scr_game, stops[i].x, stops[i].y, vis(self, other, "migration_stop_seconds")))
    i = i + 1
  end
  gm.array_push(list, gm.new(___struct___43__cine_show_start__scr_game, ox, oy, vis(self, other, "migration_origin_seconds"), count))
  global.tasks.cine = gm.new(___struct___44__cine_show_start__scr_game, -1, list)
end

-- gml_Script____struct___45__cine_planet_start__scr_game @ 0x14008c980
function ___struct___45__cine_planet_start__scr_game(self, other, x, y, s, a, ph, sp)
  self.x = x
  self.y = y
  self.s = s
  self.a = a
  self.ph = ph
  self.sp = sp
end

-- gml_Script____struct___46__cine_planet_start__scr_game @ 0x14008ccf0
function ___struct___46__cine_planet_start__scr_game(self, other, x, y, delay, pace)
  self.x = x
  self.y = y
  self.delay = delay
  self.pace = pace
end

-- gml_Script____struct___47__cine_planet_start__scr_game @ 0x14008cfa0
function ___struct___47__cine_planet_start__scr_game(self, other, x, y, a, seed)
  self.x = x
  self.y = y
  self.a = a
  self.w = 1
  self.seed = seed
end

-- gml_Script____struct___48__cine_planet_start__scr_game @ 0x14008d290
function ___struct___48__cine_planet_start__scr_game(self, other, x1, y1, x2, y2, time, w)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.time = time
  self.w = w
end

-- gml_Script____struct___49__cine_planet_start__scr_game @ 0x14008d5f0
function ___struct___49__cine_planet_start__scr_game(self, other, x, y, a, w, seed)
  self.x = x
  self.y = y
  self.a = a
  self.w = w
  self.seed = seed
end

-- gml_Script_cine_planet_start @ 0x14008d900
function cine_planet_start(self, other)
  local c = global.tasks.cine
  local stars = gm.array()
  local i = 0
  while i < 260 do
    gm.array_push(stars, gm.new(___struct___45__cine_planet_start__scr_game,
      gm.random(1), gm.random(1), gm.choose(1, 2, 2, 2, 3, 3, 4), gm.random_range(0.7, 1), gm.random(360), gm.random_range(1, 4)))
    i = i + 1
  end
  local segs = gm.array()
  local front = gm.array()
  local n = 7
  local tmax = 0.01
  i = 0
  while i < n do
    local ang = gm.random(360)
    local r
    if i == 0 then
      r = gm.random(0.2)
    else
      r = gm.random_range(0.25, 0.8)
    end
    local delay
    local hx = gm.lengthdir_x(r, ang)
    local hy = gm.lengthdir_y(r, ang)
    if i == 0 then
      delay = 0
    else
      delay = gm.random_range(0.05, 0.45)
    end
    local hub = gm.new(___struct___46__cine_planet_start__scr_game, hx, hy, delay, gm.random_range(0.6, 1.4))
    local k = 10
    local j = 0
    while j < k do
      gm.array_push(front, gm.new(___struct___47__cine_planet_start__scr_game, hub.x, hub.y,
        j * 360 / k + gm.random_range(-12, 12), hub))
      j = j + 1
    end
    i = i + 1
  end
  local spd = 0.03
  while gm.array_length(front) >= 1 and not (gm.array_length(segs) >= 9000) do
    local nxt = gm.array()
    i = 0
    while i < gm.array_length(front) do
      local f = front[i]
      local s = f.seed
      local target
      if f.x == s.x and f.y == s.y then
        target = f.a
      else
        target = gm.point_direction(s.x, s.y, f.x, f.y)
      end
      local jitter = gm.random_range(-32, 32)
      f.a = f.a + (jitter + gm.angle_difference(target, f.a) * 0.2)
      local nx = f.x + gm.lengthdir_x(spd, f.a)
      local ny = f.y + gm.lengthdir_y(spd, f.a)
      if gm.point_distance(0, 0, nx, ny) <= 0.97 then
        local t = s.delay + gm.point_distance(s.x, s.y, f.x, f.y) / s.pace
        gm.array_push(segs, gm.new(___struct___48__cine_planet_start__scr_game, f.x, f.y, nx, ny, t, f.w))
        tmax = gm.max(tmax, t)
        f.x = nx
        f.y = ny
        gm.array_push(nxt, f)
        if gm.array_length(nxt) + gm.array_length(front) < 240 and gm.random(1) < 0.12 and f.w > 0.2 then
          local rr = gm.random_range(25, 55)
          local sign = gm.choose(-1, 1)
          gm.array_push(nxt, gm.new(___struct___49__cine_planet_start__scr_game, f.x, f.y, f.a + sign * rr, f.w * 0.82, s))
        end
      end
      i = i + 1
    end
    front = nxt
  end
  i = 0
  while i < gm.array_length(segs) do
    segs[i].time = segs[i].time / tmax
    i = i + 1
  end
  c.phase = "play"
  c.t = 0
  c.white = 1
  c.shake_k = 1
  c.stars = stars
  c.segs = segs
end

-- gml_Script_cine_shake @ 0x140091950
function cine_shake(self, other)
  if not gm.bool(cine_showing(self, other)) then
    return gm.undefined
  end
  local k = nz(gm.variable_struct_get(global.tasks.cine, "shake_k"), 1)
  local mult = vis(self, other, "migration_shake_mult")
  local amp = vis(self, other, "boar_shake_px") * mult * k
  local t = gm.current_time
  local ax = amp * 0.45
  local dx = ax * (math.sin(t * 0.057 + 1.3) * 0.4 + math.sin(t * 0.031) * 0.6)
  local dy = amp * (math.sin(t * 0.089 + 2.1) * 0.4 + math.sin(t * 0.043 + 0.7) * 0.6)
  return gm.array(dx, dy)
end

-- gml_Script____struct___50__cine_step__scr_game @ 0x140092750
function ___struct___50__cine_step__scr_game(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___51__cine_step__scr_game @ 0x140092950
function ___struct___51__cine_step__scr_game(self, other, a, power)
  self.a = a
  self.power = power
end
