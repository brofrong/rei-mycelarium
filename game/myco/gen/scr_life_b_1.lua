local gm = require("gm")

-- gml_Script_mushroom_type_sprite @ 0x1400b4340
function mushroom_type_sprite(self, other, t)
  if t == 0 then
    return sporeshroom
  elseif t == 1 then
    return predatorshroom
  elseif t == 2 then
    return explosiveshroom
  elseif t == 3 then
    return treeshroom
  elseif t == 4 then
    return diggershroom
  elseif t == 5 then
    return stinkyshroom
  elseif t == 6 then
    return experienceshroom
  elseif t == 7 then
    return consumptorshroom
  end
  return sporeshroom
end

-- gml_Script_shroom_sheet_cells @ 0x1400b48e0
function shroom_sheet_cells(self, other, spr)
  if spr == sporeshroom or spr == explosiveshroom or spr == explosiveshroom_exploded or spr == predatorshroom
    or spr == diggershroom or spr == experienceshroom or spr == consumptorshroom then
    return 2.0
  end
  if spr == stinkyshroom then
    return 3.0
  end
  return 1.0
end

-- gml_Script_mushroom_type_variants @ 0x1400b4ce0
function mushroom_type_variants(self, other, t)
  return shroom_sheet_cells(self, other, mushroom_type_sprite(self, other, t))
end

-- gml_Script_mushroom_cell_width @ 0x1400b4ef0
function mushroom_cell_width(self, other, spr)
  return gm.max(1.0, gm.div(gm.sprite_get_width(spr), shroom_sheet_cells(self, other, spr)))
end

-- gml_Script_mushroom_variant @ 0x1400b5210
function mushroom_variant(self, other, m)
  local v
  if gm.bool(gm.variable_struct_exists(m, "variant")) then
    v = m.variant
  else
    v = 0
  end
  return gm.clamp(gm.real(v), 0, mushroom_type_variants(self, other, m.type) - 1)
end

-- gml_Script_shroom_grow_scale @ 0x1400b5720
function shroom_grow_scale(self, other, m)
  if not gm.bool(gm.variable_struct_exists(m, "born")) then
    return 1.0
  end
  local t = gm.clamp((gm.current_time - m.born) / gm.max(1.0, vis(self, other, "shroom_grow_ms")), 0, 1.0)
  local c1 = 1.70158
  local u = t - 1
  local c3 = c1 + 1
  return gm.max(0, 1.0 + c3 * u * u * u + c1 * u * u)
end

-- gml_Script_mushroom_cell_left @ 0x1400b6150
function mushroom_cell_left(self, other, spr, v)
  local l = v * mushroom_cell_width(self, other, spr)
  if spr == stinkyshroom and v == 2.0 then
    l = l + 7
  end
  if spr == experienceshroom and v == 1.0 then
    l = l - 4.0
  end
  return l
end

-- gml_Script_mushroom_sprite_now @ 0x1400b6590
function mushroom_sprite_now(self, other, m)
  if m.type == 2.0 and not gm.bool(m.active) then
    return explosiveshroom_exploded
  end
  return mushroom_type_sprite(self, other, m.type)
end

-- gml_Script_mushroom_draw @ 0x1400b68e0
function mushroom_draw(self, other, m, x, y, col, alpha, ang, scale)
  if gm.is_undefined(ang) then
    ang = 0
  end
  if gm.is_undefined(scale) then
    scale = 1.0
  end
  local spr = mushroom_sprite_now(self, other, m)
  local cw = mushroom_cell_width(self, other, spr)
  local v = mushroom_variant(self, other, m)
  local left = mushroom_cell_left(self, other, spr, v)
  local s = shroom_grow_scale(self, other, m) * scale
  -- m[$ "face"] ?? 1 (struct_get_from_hash with the hash of "face")
  local face = m.face
  if gm.is_undefined(face) then
    face = 1.0
  end
  local xs = s * face
  local w = gm.min(cw, gm.sprite_get_width(spr) - left)
  local h = gm.sprite_get_height(spr)
  local ox = -gm.real(gm.sprite_get_xoffset(spr)) * xs
  local oy = -gm.real(gm.sprite_get_yoffset(spr)) * s
  if ang == 0 then
    gm.draw_sprite_part_ext(spr, 0, left, 0, w, h, x + ox, y + oy, xs, s, col, alpha)
    return
  end
  local c = gm.dcos(ang)
  local sn = gm.dsin(ang)
  gm.draw_sprite_general(spr, 0, left, 0, w, h, x + ox * c + oy * sn, y - ox * sn + oy * c, xs, s, ang, col, col, col, col, alpha)
end

-- gml_Script_predator_period @ 0x1400b88b0
function predator_period(self, other)
  local d = 1.0 + upg_bonus(self, other, "predator_digestion")
  return gm.max(60.0, bal(self, other, "predator_catch_interval") / d * perk_time_mult(self, other))
end

-- gml_Script_predator_yield @ 0x1400b8ca0
function predator_yield(self, other)
  local u = upg_bonus(self, other, "predator_yield")
  return gm.floor(bal(self, other, "predator_food_per_fly") + u)
end

-- gml_Script_repeller_max_charges @ 0x1400b8f60
function repeller_max_charges(self, other)
  return gm.max(1.0, bal(self, other, "repeller_base_charges"))
end

-- gml_Script_squirrels_step @ 0x140115c90
function squirrels_step(self, other)
  local view = squirrel_view(self, other)
  if gm.bool(squirrel_any_target(self, other)) then
    global.squirrel_timer = global.squirrel_timer - animal_timer_step(self, other)
    if global.squirrel_timer <= 0 then
      global.squirrel_timer = animal_interval(self, other, "squirrel_spawn_interval")
      local pick = squirrel_pick(self, other, view, false)
      if not gm.is_undefined(pick) then
        squirrel_spawn(self, other, pick, view)
      end
    end
  end
  local spd = bal(self, other, "squirrel_speed")
  local run_spd = bal(self, other, "squirrel_run_speed")
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(global.squirrels) do
    local s = global.squirrels[i]
    local m = s.mushroom
    if not gm.bool(s.empty_run) and (gm.is_undefined(m) or mushroom_index_of(self, other, m) < 0) then
      s.mushroom = gm.undefined
      m = gm.undefined
      s.empty_run = true
      if s.state == 0 then
        squirrel_flip_up(self, other, s)
      end
    end
    local state = s.state
    if state == 0 then
      squirrel_animate(self, other, s)
      s.y = s.y + spd
      s.x = squirrel_climb_x(self, other, s.mirror, s.angle, s.tree, s.side, s.y)
      local mouth = squirrel_point(self, other, s, vis(self, other, "squirrel_mouth_x"), vis(self, other, "squirrel_mouth_y"))
      if gm.bool(s.direct) then
        if not gm.is_undefined(m) then
          s.stop_y = squirrel_bite_y(self, other, m)
        end
        if mouth[1] >= s.stop_y then
          if not gm.is_undefined(m) then
            squirrel_grab(self, other, s)
          end
          squirrel_flip_up(self, other, s)
        end
      else
        local turn = 90 * s.dir
        local gap = vis(self, other, "squirrel_jump_gap")
        local gap_y = squirrel_run_y(self, other) - gap
        if s.y >= gap_y then
          s.y = gap_y
          local jx = s.x + s.dir * vis(self, other, "squirrel_jump_dx")
          squirrel_turn_start(self, other, s, 1.0, turn, jx, squirrel_run_y(self, other))
        end
      end
    elseif state == 1 then
      squirrel_animate(self, other, s)
      if gm.bool(squirrel_turn_step(self, other, s)) then
        s.state = 2
        s.run_line = s.y
      end
    elseif state == 2 then
      squirrel_animate(self, other, s)
      squirrel_check_repeller(self, other, s)
      m = s.mushroom
      s.x = s.x + s.dir * run_spd
      s.y = s.run_line
      local turn = 180 * s.dir
      local nside = -gm.real(s.dir)
      local top_y = s.y - vis(self, other, "squirrel_jump_gap")
      local tx = squirrel_climb_x(self, other, s.mirror, turn, s.target_tree, nside, top_y)
      if (s.x - (tx - s.dir * vis(self, other, "squirrel_jump_dx"))) * s.dir >= 0 then
        s.tree = s.target_tree
        s.side = nside
        squirrel_turn_start(self, other, s, 3.0, turn, tx, top_y)
      end
    elseif state == 3 then
      squirrel_animate(self, other, s)
      if gm.bool(squirrel_turn_step(self, other, s)) then
        s.state = 4
      end
    elseif state == 4 then
      squirrel_animate(self, other, s)
      s.y = s.y - spd
      s.x = squirrel_climb_x(self, other, s.mirror, s.angle, s.tree, s.side, s.y)
      if not gm.is_undefined(m) and not gm.bool(m.stolen) then
        local mouth = squirrel_point(self, other, s, vis(self, other, "squirrel_mouth_x"), vis(self, other, "squirrel_mouth_y"))
        local by = squirrel_bite_y(self, other, m)
        if mouth[1] <= by then
          squirrel_grab(self, other, s)
          squirrel_flip_down(self, other, s)
          s.dir = -gm.real(s.dir)
          s.target_tree = s.home
        end
      end
    end
    s.timer = gm.max(0, s.timer - 1)
    if not gm.is_undefined(m) and gm.bool(m.stolen) then
      local pose = squirrel_hold_pose(self, other, s)
      local t = 1 - s.timer / gm.max(1.0, vis(self, other, "squirrel_pull_frames"))
      t = 1 - (1 - t) * (1 - t)
      m.x = gm.lerp(s.grab_x, pose.x, t)
      m.y = gm.lerp(s.grab_y, pose.y, t)
      m.hold_xscale = gm.lerp(1.0, pose.xscale, t)
      m.hold_yscale = gm.lerp(1.0, pose.yscale, t)
      m.hold_angle = pose.angle * t
    end
    local removed = false
    if s.state == 4 then
      local drop = squirrel_tail_drop(self, other)
      if s.y + drop + 8 < squirrel_sky_y(self, other, s.tree, view) then
        removed = true
        local mi
        if not gm.is_undefined(m) then
          mi = mushroom_index_of(self, other, m)
        else
          mi = -1
        end
        if mi >= 0 and gm.bool(m.stolen) then
          gm.array_delete(global.mushrooms, mi, 1.0)
          animals_reindex_after_removal(self, other, mi)
          animals_rebind(self, other)
        end
      end
    end
    if not removed then
      gm.array_push(keep, s)
    end
    i = i + 1
  end
  global.squirrels = keep
end

-- gml_Script_crows_draw @ 0x140124540
function crows_draw(self, other, vx, vy, vw)
  local fw = crow_frame_w(self, other)
  local fh = gm.sprite_get_height(crow)
  local sc = vis(self, other, "crow_scale")
  local claw = crow_claw(self, other, 0)
  local margin = fw * sc + 200
  local i = 0
  while i < gm.array_length(global.crows) do
    local c = global.crows[i]
    if not (c.x < vx - margin) and not (c.x > vx + vw + margin) then
      local m = c.mushroom
      if not gm.is_undefined(m) and gm.bool(m.stolen) then
        mushroom_draw(self, other, m, gm.floor(m.x), gm.floor(m.y), 16777215.0, 1.0, gm.undefined, gm.undefined)
      end
      local xs = -gm.real(c.dir) * sc
      gm.draw_sprite_part_ext(crow, 0, c.frame * fw, 0, fw, fh,
        gm.floor(c.x - claw[0] * xs), gm.floor(c.y - claw[1] * sc), xs, sc, 16777215.0, 1.0)
    end
    i = i + 1
  end
end

-- gml_Script____struct___85__life_far_init__scr_life @ 0x140125ee0
function ___struct___85__life_far_init__scr_life(self, other)
  self.leaves = 0
  self.worms = 0
  self.food = 0
end

-- gml_Script_life_far_init @ 0x140126060
function life_far_init(self, other)
  global.far_leaf_cells = gm.struct({})
  global.far_worm_columns = gm.struct({})
  global.far_scan_from = 0
  global.far_timer = 0
  global.far_food_due = 0
  global.far_rate = gm.new(___struct___85__life_far_init__scr_life)
end

-- gml_Script_life_far_measure @ 0x1401264f0
function life_far_measure(self, other, net)
  local sy = wld(self, other, "surface_height_px") + wld(self, other, "reference_tile_size")
  local r = bal(self, other, "leaf_absorb_radius")
  local wt = worm_zone_top(self, other)
  local wb = worm_zone_bottom(self, other)
  local branches = net.branches
  local i = 0
  while i < gm.array_length(branches) do
    local pts = branches[i].points
    local n = gm.array_length(pts)
    local j = 0
    while j < n do
      local p = pts[j]
      local dy = p.y - sy
      if gm.abs(dy) <= r then
        local hw = gm.sqrt(r * r - dy * dy)
        local c1 = gm.floor((p.x + hw) / 16.0)
        local c = gm.floor((p.x - hw) / 16.0)
        while c <= c1 do
          local k = gm.string(c)
          if not gm.bool(gm.variable_struct_exists(global.far_leaf_cells, k)) then
            gm.variable_struct_set(global.far_leaf_cells, k, true)
          end
          c = c + 1.0
        end
      end
      if j ~= 0 then
        local q = pts[j - 1]
        local my = (p.y + q.y) * 0.5
        if not (my < wt) and not (my > wb) then
          local key = gm.string(gm.floor((p.x + q.x) * 0.5 / 1024.0))
          local cur = gm.variable_struct_get(global.far_worm_columns, key)
          if gm.is_undefined(cur) then
            cur = 0.0
          end
          gm.variable_struct_set(global.far_worm_columns, key, cur + gm.point_distance(p.x, p.y, q.x, q.y))
        end
      end
      j = j + 1.0
    end
    i = i + 1
  end
end

-- gml_Script_life_far_outside @ 0x1401289f0
function life_far_outside(self, other, a, b, v)
  local ov = gm.max(0, gm.min(b, v.x2) - gm.max(a, v.x1))
  return 1.0 - ov / gm.max(1.0, b - a)
end

-- gml_Script_life_far_step @ 0x1401290a0
function life_far_step(self, other)
  local ctrl
  if gm.bool(gm.instance_exists(obj_mycelium_controller)) then
    ctrl = gm.instance_find(obj_mycelium_controller, 0)
  else
    ctrl = -4.0
  end
  global.far_timer = global.far_timer - 1.0
  if ctrl ~= -4.0 then
    local growths = ctrl.myc_growths
    local deadline = gm.get_timer() + 1000
    local i = global.far_scan_from
    while i < gm.array_length(growths) do
      if not (gm.get_timer() < deadline) then
        break
      end
      local g = growths[i]
      if gm.bool(g.finished) then
        -- g[$ "far_measured"] ?? false (struct_get_from_hash)
        local fm = g.far_measured
        if gm.is_undefined(fm) then
          fm = false
        end
        if not gm.bool(fm) then
          life_far_measure(self, other, g)
          g.far_measured = true
        end
      end
      i = i + 1
    end
    while global.far_scan_from < gm.array_length(growths) do
      -- growths[far_scan_from][$ "far_measured"] ?? false (struct_get_from_hash)
      local fm = growths[global.far_scan_from].far_measured
      if gm.is_undefined(fm) then
        fm = false
      end
      if not gm.bool(fm) then
        break
      end
      global.far_scan_from = global.far_scan_from + 1
    end
  end
  if ctrl ~= -4.0 and global.far_timer <= 0 then
    global.far_timer = bal(self, other, "life_far_interval_frames")
    local z = life_near_zone(self, other)
    local per_min = 60.0 * global.fps
    local leaves = 0
    local worms = 0
    if gm.bool(upg_is_owned(self, other, "unlock_absorption")) then
      local names = gm.variable_struct_get_names(global.far_leaf_cells)
      local cnt = 0
      local i = 0
      while i < gm.array_length(names) do
        local cx = gm.real(names[i]) * 16 + 8
        if cx < z.x1 or cx > z.x2 then
          cnt = cnt + 1
        end
        i = i + 1
      end
      local w = z.w
      local interval = bal(self, other, "leaf_spawn_interval")
      leaves = cnt * 16 / gm.max(1.0, w) * per_min / gm.max(1.0, interval)
    end
    if gm.bool(upg_is_owned(self, other, "unlock_protein")) then
      local wcap = bal(self, other, "worm_far_cap_per_screen_min") * 1024 / gm.max(1.0, z.w)
      local names = gm.variable_struct_get_names(global.far_worm_columns)
      local i = 0
      while i < gm.array_length(names) do
        local c0 = gm.real(names[i]) * 1024
        local out = life_far_outside(self, other, c0, c0 + 1024, z)
        if not (out <= 0) then
          local len = gm.variable_struct_get(global.far_worm_columns, names[i])
          worms = worms + gm.min(wcap, bal(self, other, "worm_far_catch_per_1000px_min") * len / 1000.0) * out
        end
        i = i + 1
      end
    end
    global.far_rate.leaves = leaves
    global.far_rate.worms = worms
    local wf = worms * bal(self, other, "worm_food_reward")
    global.far_rate.food = leaves * leaf_food_value(self, other) + wf
  end
  global.far_food_due = global.far_food_due + global.far_rate.food / (60.0 * global.fps)
  if global.far_food_due >= 1 then
    local f = gm.floor(global.far_food_due)
    global.far_food_due = global.far_food_due - f
    food_add(self, other, f)
  end
end

-- gml_Script____struct___86__consume_info__scr_life @ 0x14012c470
function ___struct___86__consume_info__scr_life(self, other, slots, minutes, food)
  self.slots = slots
  self.minutes = minutes
  self.food = food
end

-- gml_Script____struct___87__consume_info__scr_life @ 0x14012c6b0
function ___struct___87__consume_info__scr_life(self, other, slots, minutes, food)
  self.slots = slots
  self.minutes = minutes
  self.food = food
end

-- gml_Script_consume_info @ 0x14012c8f0
function consume_info(self, other, t)
  if gm.bool(gm.variable_struct_exists(t, "part")) then
    local kind = t.part.consume
    local slots = t.part.slots
    local km = "consumptor_" .. kind .. "_minutes"
    local kf = "consumptor_" .. kind .. "_food"
    return gm.new(___struct___86__consume_info__scr_life, gm.array_length(slots), bal(self, other, km), bal(self, other, kf))
  end
  local slots = bal(self, other, "consumptor_tree_slots")
  local minutes = bal(self, other, "consumptor_tree_minutes")
  local food = bal(self, other, "consumptor_tree_food")
  return gm.new(___struct___87__consume_info__scr_life, slots, minutes, food)
end

-- gml_Script_consume_frames @ 0x14012d630
function consume_frames(self, other, t)
  local minutes = consume_info(self, other, t).minutes
  local d = 1.0 + upg_bonus(self, other, "parasitism_speed")
  return minutes * 60 * global.fps / d * perk_time_mult(self, other)
end

-- gml_Script_consume_food @ 0x14012db00
function consume_food(self, other, t)
  local food = consume_info(self, other, t).food
  local k = 1.0 + upg_bonus(self, other, "parasitism_yield")
  return gm.round(food * k)
end

-- gml_Script_consumptors_on @ 0x14012dea0
function consumptors_on(self, other, host)
  local res = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 7.0 and m.host == host and not gm.bool(m.stolen) then
      gm.array_push(res, m)
    end
    i = i + 1
  end
  return res
end

-- gml_Script_consumptor_variant @ 0x14012e450
function consumptor_variant(self, other, side)
  if side < 0 then
    return 0
  end
  return 1.0
end

-- gml_Script_consumptor_progress @ 0x14012e5b0
function consumptor_progress(self, other, m)
  if not gm.bool(m.stolen) and not gm.is_undefined(m.host) then
    -- m.host[$ "consume"] ?? 0 (struct_get_from_hash)
    local c = m.host.consume
    if gm.is_undefined(c) then
      c = 0
    end
    return gm.clamp(gm.real(c), 0, 1.0)
  end
  return gm.undefined
end

-- gml_Script_consume_finish @ 0x14012ea90
function consume_finish(self, other, t)
  food_add(self, other, consume_food(self, other, t))
  tasks_stat_add(self, other, "food", consume_food(self, other, t))
  local is_tree = not gm.bool(gm.variable_struct_exists(t, "part"))
  if is_tree then
    tasks_stat_add(self, other, "ach_trees_eaten", gm.undefined)
  end
  local i = gm.array_length(global.mushrooms) - 1
  while i >= 0 do
    local m = global.mushrooms[i]
    local kill
    if m.type == 7.0 and m.host == t and not gm.bool(m.stolen) then
      kill = true
    else
      kill = false
      if is_tree and m.type == 3.0 and not gm.bool(m.stolen) then
        local mx = m.x
        local tx = t.x
        kill = gm.abs(mx - vis(self, other, "tree_mushroom_offset_x") - tx) < 2.0
      end
    end
    if kill then
      gm.array_delete(global.mushrooms, i, 1.0)
      animals_reindex_after_removal(self, other, i)
    end
    i = i - 1.0
  end
  animals_rebind(self, other)
  if gm.bool(gm.instance_exists(obj_forest_controller)) then
    local fc = gm.instance_find(obj_forest_controller, 0)
    fc.forest_remove_host(fc, other, t)
  end
end

-- gml_Script_predator_progress @ 0x14012fb90
function predator_progress(self, other, m)
  if m.carried_by == -1.0 and not gm.bool(m.stolen) then
    local timer = m.timer
    return gm.clamp(1.0 - timer / predator_period(self, other), 0, 1.0)
  end
  return gm.undefined
end

-- gml_Script_experience_period @ 0x14012ff20
function experience_period(self, other)
  return gm.max(60.0, bal(self, other, "experience_point_interval") * global.fps * perk_time_mult(self, other))
end

-- gml_Script_experience_progress @ 0x140130260
function experience_progress(self, other, m)
  if gm.bool(m.stolen) then
    return gm.undefined
  end
  local timer = m.timer
  return gm.clamp(1.0 - timer / experience_period(self, other), 0, 1.0)
end

-- gml_Script_tree_shroom_progress @ 0x140130550
function tree_shroom_progress(self, other, m)
  if gm.bool(m.stolen) then
    return gm.undefined
  end
  local timer = m.timer
  return gm.clamp(1.0 - timer / gm.max(1.0, tree_drop_period(self, other)), 0, 1.0)
end

-- gml_Script_repeller_regen_progress @ 0x140130970
function repeller_regen_progress(self, other, m)
  local rf = repeller_autoregen_frames(self, other)
  if not gm.bool(m.active) and not (rf < 0) and not gm.bool(m.stolen) and m.carried_by == -1.0 then
    -- m[$ "regen_timer"] ?? 0 (struct_get_from_hash)
    local rt = m.regen_timer
    if gm.is_undefined(rt) then
      rt = 0
    end
    return gm.clamp(rt / rf, 0, 1.0)
  end
  return gm.undefined
end

-- gml_Script_hover_bar_draw @ 0x140130fb0
function hover_bar_draw(self, other, cx, by, p)
  local w = vis(self, other, "hover_bar_w")
  local h = vis(self, other, "hover_bar_h")
  local x = gm.floor(cx - w * 0.5)
  local y = gm.floor(by - h)
  local fw = gm.floor((w - 2) * gm.clamp(gm.real(p), 0, 1.0))
  gm.draw_set_colour(0)
  gm.draw_set_alpha(gm.real(vis(self, other, "hover_bar_back_alpha")))
  gm.draw_rectangle(x, y, x + w - 1, y + h - 1, false)
  gm.draw_set_alpha(1.0)
  gm.draw_set_colour(0xffffff)
  if fw > 0 then
    gm.draw_rectangle(x + 1, y + 1, x + fw, y + h - 2, false)
  end
end

-- gml_Script_hover_bar_step_draw @ 0x140131f70
function hover_bar_step_draw(self, other)
  if gm.bool(ui_blocking(self, other)) then
    return
  end
  local mx = gm.mouse_x
  local my = gm.mouse_y
  local gap = vis(self, other, "hover_bar_gap")
  local i = gm.array_length(global.mushrooms) - 1
  while i >= 0 do
    local m = global.mushrooms[i]
    local p = gm.undefined
    if m.type == 1.0 then
      p = predator_progress(self, other, m)
    elseif m.type == 3.0 then
      p = tree_shroom_progress(self, other, m)
    elseif m.type == 6.0 then
      p = experience_progress(self, other, m)
    elseif m.type == 7.0 then
      p = consumptor_progress(self, other, m)
    elseif m.type == 2.0 then
      p = repeller_regen_progress(self, other, m)
    end
    if not gm.is_undefined(p) then
      local hb = mushroom_hit_box(self, other, m)
      if gm.bool(gm.point_in_rectangle(mx, my, hb.x1, hb.y1, hb.x2, hb.y2)) then
        hover_bar_draw(self, other, (hb.x1 + hb.x2) * 0.5, hb.y1 - gap, p)
        return
      end
    end
    i = i - 1.0
  end
  local lr = vis(self, other, "hover_leaf_radius_px")
  i = 0
  while i < gm.array_length(global.leaves) do
    local l = global.leaves[i]
    if gm.bool(l.absorbing) then
      if gm.point_distance(mx, my, l.x, l.y) <= lr then
        local lx = l.x
        local ly = l.y
        local timer = l.timer
        local total = l.total
        hover_bar_draw(self, other, lx, ly - lr - gap, 1.0 - timer / gm.max(1.0, total))
        return
      end
    end
    i = i + 1
  end
  local wr = vis(self, other, "hover_worm_radius_px")
  i = 0
  while i < gm.array_length(global.worms) do
    local w = global.worms[i]
    if gm.bool(w.caught) then
      local body = w.body
      local top = gm.infinity
      local hit = false
      local j = 0
      while j < gm.array_length(body) do
        top = gm.min(top, body[j].y)
        if not gm.bool(hit) then
          hit = gm.point_distance(mx, my, body[j].x, body[j].y) <= wr
        else
          hit = true
        end
        j = j + 1
      end
      if gm.bool(hit) then
        local mid = body[gm.div(gm.array_length(body), 2)]
        local midx = mid.x
        local timer = w.timer
        local total = w.total
        local th = vis(self, other, "worm_thickness_px")
        hover_bar_draw(self, other, midx, top - th - gap, 1.0 - timer / gm.max(1.0, total))
        return
      end
    end
    i = i + 1
  end
end

local stinky_cap_height_static = { init = false, heights = nil }

-- gml_Script_stinky_cap_height @ 0x140134c20
function stinky_cap_height(self, other, m)
  if not stinky_cap_height_static.init then
    stinky_cap_height_static.init = true
    stinky_cap_height_static.heights = gm.array(31.0, 51.0, 30.0)
  end
  local heights = stinky_cap_height_static.heights
  local v = gm.real(mushroom_variant(self, other, m))
  local gs = shroom_grow_scale(self, other, m)
  local idx = gm.clamp(v, 0, 2.0)
  idx = idx >= 0 and math.floor(idx) or math.ceil(idx)
  return heights[idx] * gs - vis(self, other, "stinky_emit_into_cap_px")
end

-- gml_Script____struct___88__stinky_shape__scr_life @ 0x1401351c0
function ___struct___88__stinky_shape__scr_life(self, other, base, open, flare, top)
  self.base = base
  self.open = open
  self.flare = flare
  self.top = top
end

-- gml_Script_stinky_shape @ 0x140135470
function stinky_shape(self, other)
  local lvl = gm.clamp(gm.real(upg_level(self, other, "stinky_width")), 0, 4.0)
  local radii = vis(self, other, "stinky_base_radius_levels")
  local cones = vis(self, other, "stinky_cone_deg_levels")
  local flares = vis(self, other, "stinky_flare_power_levels")
  local li = math.floor(lvl)
  local base = radii[li]
  local t = gm.dtan(cones[li])
  local open = vis(self, other, "stinky_beam_height_px") * t
  return gm.new(___struct___88__stinky_shape__scr_life, base, open, flares[li], base + open)
end

-- gml_Script_stinky_top_at @ 0x140135ea0
function stinky_top_at(self, other, lvl)
  local radii = vis(self, other, "stinky_base_radius_levels")
  local cones = vis(self, other, "stinky_cone_deg_levels")
  local li = lvl >= 0 and math.floor(lvl) or math.ceil(lvl)
  local t = gm.dtan(cones[li])
  local open = vis(self, other, "stinky_beam_height_px") * t
  return radii[li] + open
end

-- gml_Script_stinky_funnel_radius @ 0x140136430
function stinky_funnel_radius(self, other, sh, dy)
  local t = gm.clamp(dy / gm.max(1.0, vis(self, other, "stinky_beam_height_px")), 0, 1.0)
  return sh.base + sh.open * gm.power(t, sh.flare)
end

-- gml_Script____struct___89__stinky_pool__scr_life @ 0x140136a00
function ___struct___89__stinky_pool__scr_life(self, other, side, rise, offset, speed, age, phase, size, heat, drift, px, py, pix, colour, alpha)
  self.n = 0
  self.side = side
  self.rise = rise
  self.offset = offset
  self.speed = speed
  self.age = age
  self.phase = phase
  self.size = size
  self.heat = heat
  self.drift = drift
  self.px = px
  self.py = py
  self.pix = pix
  self.colour = colour
  self.alpha = alpha
end

-- gml_Script_stinky_pool @ 0x1401370e0
function stinky_pool(self, other, m)
  if not gm.bool(gm.is_struct(m.stench)) then
    m.stench = gm.new(___struct___89__stinky_pool__scr_life,
      gm.array(), gm.array(), gm.array(), gm.array(), gm.array(), gm.array(), gm.array(),
      gm.array(), gm.array(), gm.array(), gm.array(), gm.array(), gm.array(), gm.array())
  end
  return m.stench
end

-- gml_Script_stinky_particle_add @ 0x140137c30
function stinky_particle_add(self, other, pool, rise)
  local n = pool.n
  pool.side[n] = gm.random_range(-1.0, 1.0)
  pool.rise[n] = rise
  local off
  if rise > 0 then
    off = stinky_wind_shift(self, other, rise)
  else
    off = 0
  end
  pool.offset[n] = off
  local rmax = gm.real(vis(self, other, "stinky_rise_max"))
  local rmin = gm.real(vis(self, other, "stinky_rise_min"))
  pool.speed[n] = gm.random_range(rmin, rmax)
  local age
  if rise > 0 then
    age = 999.0
  else
    age = 0
  end
  pool.age[n] = age
  pool.phase[n] = gm.random(360.0)
  pool.size[n] = gm.choose(1.0, 1.0, 2.0, 2.0, 2.0, 3.0)
  pool.heat[n] = gm.random(1.0)
  local drift
  if rise > 0 then
    drift = stinky_wind_target(self, other)
  else
    drift = 0
  end
  pool.drift[n] = drift
  pool.n = n + 1
end

-- gml_Script_stinky_wind_target @ 0x140138ae0
function stinky_wind_target(self, other)
  return global.wind_dir * global.wind_speed * vis(self, other, "stinky_wind_push")
end

-- gml_Script_stinky_wind_shift @ 0x140138d60
function stinky_wind_shift(self, other, rise)
  local rmax = vis(self, other, "stinky_rise_max")
  local avg = gm.max(0.05, (vis(self, other, "stinky_rise_min") + rmax) * 0.5)
  return stinky_wind_target(self, other) * rise / avg
end

-- gml_Script_stinky_step @ 0x1401392c0
function stinky_step(self, other, m, v)
  local beam = vis(self, other, "stinky_beam_height_px")
  local sh = stinky_shape(self, other)
  local reach = sh.top + gm.abs(gm.real(stinky_wind_shift(self, other, beam))) + 120
  local visible = false
  if not (m.x < v.x - reach) and m.x <= v.x + v.w + reach then
    local vb = v.y + v.h
    if m.y - stinky_cap_height(self, other, m) - beam <= vb and m.y >= v.y - 200 then
      visible = true
    end
  end
  if not visible then
    m.stench_live = false
    return
  end
  m.stench_live = true
  global.stinky_live_count = global.stinky_live_count + 1
  local pool = stinky_pool(self, other, m)
  local top2 = stinky_top_at(self, other, 2.0)
  local gs = shroom_grow_scale(self, other, m)
  local emit = vis(self, other, "stinky_emit_per_frame") * gs * sh.top / gm.max(1.0, top2)
  local lastc = gm.max(1.0, global.stinky_live_last)
  emit = emit * gm.min(1.0, vis(self, other, "stinky_full_columns") / lastc)
  if pool.n == 0 and shroom_grow_scale(self, other, m) >= 1 then
    local rmax = vis(self, other, "stinky_rise_max")
    local avg = (vis(self, other, "stinky_rise_min") + rmax) * 0.5
    local cnt = gm.round(emit * beam / avg)
    local k = 0
    while k < cnt do
      stinky_particle_add(self, other, pool, gm.random(gm.real(beam)))
      k = k + 1
    end
  end
  m.stench_due = m.stench_due + emit
  while m.stench_due >= 1 do
    m.stench_due = m.stench_due - 1
    stinky_particle_add(self, other, pool, 0)
  end
  local lift = vis(self, other, "stinky_lift")
  local sway = vis(self, other, "stinky_sway_px")
  local wt = stinky_wind_target(self, other)
  local ease = vis(self, other, "stinky_wind_ease")
  local base = sh.base
  local open = sh.open
  local flare = sh.flare
  local inv_beam = 1.0 / gm.max(1.0, beam)
  local top_fade = 1.0 / gm.max(1.0, vis(self, other, "stinky_top_fade_px"))
  local fade_in = 1.0 / gm.max(1.0, vis(self, other, "stinky_fade_in_frames"))
  local alpha_k = vis(self, other, "stinky_alpha")
  local hb = gm.real(vis(self, other, "stinky_hot_b"))
  local hg = gm.real(vis(self, other, "stinky_hot_g"))
  local hr = gm.real(vis(self, other, "stinky_hot_r"))
  local hot = gm.make_colour_rgb(hr, hg, hb)
  local db = gm.real(vis(self, other, "stinky_deep_b"))
  local dg = gm.real(vis(self, other, "stinky_deep_g"))
  local dr = gm.real(vis(self, other, "stinky_deep_r"))
  local deep = gm.make_colour_rgb(dr, dg, db)
  local shade = stinky_shade_table(self, other, hot, deep)
  local mx = m.x
  local my = m.y - stinky_cap_height(self, other, m)
  local side = pool.side
  local rise = pool.rise
  local offset = pool.offset
  local spd = pool.speed
  local age = pool.age
  local phase = pool.phase
  local size = pool.size
  local heat = pool.heat
  local drift = pool.drift
  local px = pool.px
  local py = pool.py
  local pix = pool.pix
  local colour = pool.colour
  local alpha = pool.alpha
  local n = pool.n
  local i = 0
  while i < n do
    local sp = spd[i] + lift
    local r = rise[i] + sp
    if r >= beam then
      n = n - 1
      if i < n then
        side[i] = side[n]
        rise[i] = rise[n]
        offset[i] = offset[n]
        spd[i] = spd[n]
        age[i] = age[n]
        phase[i] = phase[n]
        size[i] = size[n]
        heat[i] = heat[n]
        drift[i] = drift[n]
      end
    else
      local a = age[i] + 1
      local ph = phase[i] + 3
      local d = drift[i]
      d = d + (wt - d) * ease
      local off = offset[i] + gm.dsin(ph) * sway + d
      local t = r * inv_beam
      local xoff = side[i] * (base + open * gm.power(t, flare)) + off
      spd[i] = sp
      rise[i] = r
      age[i] = a
      phase[i] = ph
      drift[i] = d
      offset[i] = off
      px[i] = gm.floor(mx + xoff)
      py[i] = gm.floor(my - r)
      pix[i] = gm.max(1.0, gm.round(size[i] * (1 - t * 0.45)))
      colour[i] = shade[gm.round(gm.clamp(t * 0.8 + heat[i] * 0.35, 0, 1.0) * 256.0)]
      alpha[i] = alpha_k * gm.min(1.0, a * fade_in) * gm.clamp((beam - r) * top_fade, 0, 1.0) * (0.75 + 0.25 * gm.dsin(ph * 2.3))
      i = i + 1
    end
  end
  pool.n = n
end

-- gml_Script_stinky_art_init @ 0x14013cdb0
function stinky_art_init(self, other)
  if gm.bool(gm.variable_global_exists("stinky_glow")) and gm.bool(gm.sprite_exists(global.stinky_glow)) then
    return
  end
  local sz = 16.0
  local r = sz * 0.5
  local surf = gm.surface_create(sz, sz)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  local rr = r
  while rr >= 1.0 do
    local k = 1.0 - rr / r
    gm.draw_set_alpha(0.1 * k * k + 0.02)
    gm.draw_circle_colour(r, r, rr, 16777215.0, 16777215.0, false)
    rr = rr - 0.5
  end
  gm.draw_set_alpha(1.0)
  gm.surface_reset_target()
  global.stinky_glow = gm.sprite_create_from_surface(surf, 0, 0, sz, sz, false, false, r, r)
  gm.surface_free(surf)
  surf = gm.surface_create(1.0, 1.0)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(16777215.0, 1.0)
  gm.surface_reset_target()
  global.stinky_pixel = gm.sprite_create_from_surface(surf, 0, 0, 1.0, 1.0, false, false, 0, 0)
  gm.surface_free(surf)
end

local stinky_shade_table_static = { init = false, table = nil, for_hot = nil, for_deep = nil }

-- gml_Script_stinky_shade_table @ 0x14013dfb0
function stinky_shade_table(self, other, hot, deep)
  local st = stinky_shade_table_static
  if not st.init then
    st.init = true
    st.table = gm.array()
    st.for_hot = -1.0
    st.for_deep = -1.0
  end
  if hot == st.for_hot and deep == st.for_deep then
    return st.table
  end
  st.table = gm.array_create(257.0)
  local i = 0
  while i <= 256.0 do
    st.table[i] = gm.merge_colour(hot, deep, i / 256.0)
    i = i + 1
  end
  st.for_hot = hot
  st.for_deep = deep
  return st.table
end
