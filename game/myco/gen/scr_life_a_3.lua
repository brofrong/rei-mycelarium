local gm = require("gm")

-- gml_Script_squirrel_trunk_edges @ 0x1400fdee0
function squirrel_trunk_edges(self, other, spr)
  if not gm.bool(gm.variable_global_exists("squirrel_trunks")) or not gm.is_struct(global.squirrel_trunks) then
    global.squirrel_trunks = gm.struct({})
  end
  local name = gm.sprite_get_name(spr)
  local cached = gm.variable_struct_get(global.squirrel_trunks, name)
  if not gm.is_undefined(cached) then
    return cached
  end
  local w = gm.sprite_get_width(spr)
  local h = gm.sprite_get_height(spr)
  local xo = gm.sprite_get_xoffset(spr)
  local surf = gm.surface_create(w, h)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_inv_src_alpha)
  gm.draw_sprite(spr, 0, xo, gm.sprite_get_yoffset(spr))
  gm.gpu_set_blendmode(gm.bm_normal)
  gm.surface_reset_target()
  local buf = gm.buffer_create(w * h * 4, 0, 1) -- buffer_fixed, alignment 1
  gm.buffer_get_surface(buf, surf, 0)
  gm.surface_free(surf)
  local lefts = gm.array_create(h, 0)
  local rights = gm.array_create(h, 0)
  local left = -34
  local right = 32
  local yy = 0
  while yy < h do
    local row = yy * w
    local seed = -1
    local d = 0
    while d <= 40 and seed < 0 do
      if xo - d >= 0 and gm.buffer_peek(buf, (row + xo - d) * 4 + 3, 1) >= 128 then
        seed = xo - d
      elseif xo + d < w and gm.buffer_peek(buf, (row + xo + d) * 4 + 3, 1) >= 128 then
        seed = xo + d
      end
      d = d + 1
    end
    if seed >= 0 then
      local l = seed
      local r = seed
      while l > 0 and gm.buffer_peek(buf, (row + l - 1) * 4 + 3, 1) >= 128 do
        l = l - 1
      end
      while r < w - 1 and gm.buffer_peek(buf, (row + r + 1) * 4 + 3, 1) >= 128 do
        r = r + 1
      end
      left = l - xo
      right = r + 1 - xo
    end
    lefts[yy] = left
    rights[yy] = right
    yy = yy + 1
  end
  gm.buffer_delete(buf)
  local smooth = vis(self, other, "squirrel_edge_smooth_rows")
  local sl = gm.array_create(h, 0)
  local sr = gm.array_create(h, 0)
  yy = 0
  while yy < h do
    local a = gm.max(0, yy - smooth)
    local n = gm.min(h - 1, yy + smooth) - a + 1
    local tl = gm.array_create(n, 0)
    local tr = gm.array_create(n, 0)
    gm.array_copy(tl, 0, lefts, a, n)
    gm.array_copy(tr, 0, rights, a, n)
    gm.array_sort(tl, true)
    gm.array_sort(tr, true)
    sl[yy] = tl[gm.div(n, 2)]
    sr[yy] = tr[gm.div(n, 2)]
    yy = yy + 1
  end
  local res = gm.new(___struct___76__squirrel_trunk_edges__scr_life, sl, sr, gm.sprite_get_yoffset(spr), h)
  gm.variable_struct_set(global.squirrel_trunks, name, res)
  return res
end

-- gml_Script_squirrel_art_init @ 0x140102440
function squirrel_art_init(self, other)
  if gm.bool(gm.variable_global_exists("squirrel_art_ready")) and gm.bool(global.squirrel_art_ready) then
    return
  end
  squirrel_trunk_edges(self, other, tree01)
  squirrel_trunk_edges(self, other, tree02)
  squirrel_trunk_edges(self, other, tree03)
  squirrel_frames_build(self, other)
  global.squirrel_art_ready = true
end

-- gml_Script_squirrel_frames_build @ 0x140102840
function squirrel_frames_build(self, other)
  if gm.bool(gm.variable_global_exists("squirrel_frames")) and gm.bool(gm.sprite_exists(global.squirrel_frames)) then
    return
  end
  local fw = squirrel_frame_w(self, other)
  local fh = squirrel_frame_h(self, other)
  local surf = gm.surface_create(fw, fh)
  local spr = -1
  local i = 0
  while i < 9 do
    gm.surface_set_target(surf)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_inv_src_alpha)
    gm.draw_sprite_part(squirrel, 0, i * fw, 0, fw, fh, 0, 0)
    gm.gpu_set_blendmode(gm.bm_normal)
    gm.surface_reset_target()
    if i == 0 then
      spr = gm.sprite_create_from_surface(surf, 0, 0, fw, fh, false, false,
        vis(self, other, "squirrel_body_x"), vis(self, other, "squirrel_body_y"))
    else
      gm.sprite_add_from_surface(spr, surf, 0, 0, fw, fh, false, false)
    end
    i = i + 1
  end
  gm.surface_free(surf)
  global.squirrel_frames = spr
end

-- gml_Script_squirrel_edge_x @ 0x140103850
function squirrel_edge_x(self, other, s, side, py)
  local c = squirrel_trunk_edges(self, other, s.sprite)
  local row = gm.clamp(gm.round(py - s.y + c.origin_y), 0, c.h - 1)
  local l = c.left[row]
  local r = c.right[row]
  if s.flip < 0 then
    local t = l
    l = -r
    r = -t
  end
  return s.x + ((side <= 0) and l or r)
end

-- gml_Script_squirrel_climb_x @ 0x140104340
function squirrel_climb_x(self, other, flip, angle, s, side, py)
  local inset = side * vis(self, other, "squirrel_trunk_inset")
  local edge = squirrel_edge_x(self, other, s, side, py)
  local x = edge - inset
  local off = squirrel_offset(self, other, flip, angle,
    vis(self, other, "squirrel_grip_x"),
    vis(self, other, "squirrel_body_y"))
  return x - off[0]
end

-- gml_Script_squirrel_run_y @ 0x140104b30
function squirrel_run_y(self, other)
  return wld(self, other, "surface_height_px") + vis(self, other, "squirrel_run_body_dy")
end

-- gml_Script_squirrel_tail_drop @ 0x140104d80
function squirrel_tail_drop(self, other)
  return (vis(self, other, "squirrel_tail_tip_y") - vis(self, other, "squirrel_body_y")) * vis(self, other, "squirrel_scale")
end

-- gml_Script_squirrel_tree_near @ 0x1401050b0
function squirrel_tree_near(self, other, x, range, need_free, exclude)
  if not gm.bool(gm.instance_exists(obj_forest_controller)) then
    return gm.undefined
  end
  local fc = gm.instance_find(obj_forest_controller, 0)
  local c0 = gm.floor((x - range) / fc.chunk_width)
  local c1 = gm.floor((x + range) / fc.chunk_width)
  local best = gm.undefined
  local best_d = range
  local c = c0
  while c <= c1 do
    local trees = fc.surface_get_chunk(self, other, c).trees
    local i = 0
    while i < gm.array_length(trees) do
      local t = trees[i]
      if gm.bool(need_free) and t.sprite ~= tree03 then
        goto continue
      end
      if t == exclude then
        goto continue
      end
      if gm.abs(t.x - x) > best_d then
        goto continue
      end
      if gm.bool(need_free) and gm.bool(squirrel_tree_has_mushroom(self, other, t)) then
        goto continue
      end
      best_d = gm.abs(t.x - x)
      best = t
      ::continue::
      i = i + 1
    end
    c = c + 1
  end
  return best
end

-- gml_Script_squirrel_tree_has_mushroom @ 0x140106230
function squirrel_tree_has_mushroom(self, other, t)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 3 and not gm.bool(m.stolen) then
      if gm.abs(m.x - vis(self, other, "tree_mushroom_offset_x") - t.x) < 2 then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script_squirrel_takes @ 0x1401068b0
function squirrel_takes(self, other, m)
  if gm.bool(perk_has(self, other, "red")) then
    return false
  end
  if m.type == 3 then
    return true
  end
  local res = false
  if m.type == 7 and not gm.is_undefined(m.host) and not gm.bool(gm.variable_struct_exists(m.host, "part")) then
    local consumed = m.host.consumed
    if gm.is_undefined(consumed) then
      consumed = false
    end
    res = not gm.bool(consumed)
  end
  return res
end

-- gml_Script_squirrel_tree_for @ 0x140106ff0
function squirrel_tree_for(self, other, m)
  if m.type == 7 then
    return m.host
  end
  return squirrel_tree_near(self, other, m.x - vis(self, other, "tree_mushroom_offset_x"), 80, false, gm.undefined)
end

-- gml_Script_squirrel_bite_y @ 0x140107500
function squirrel_bite_y(self, other, m)
  local by = m.y + vis(self, other, "squirrel_bite_dy")
  if m.type == 7 then
    return by + vis(self, other, "squirrel_consumptor_bite_dy")
  end
  return by
end

-- gml_Script_squirrel_shroom_bite @ 0x140107970
function squirrel_shroom_bite(self, other, type)
  if type == 7 then
    return gm.array(vis(self, other, "squirrel_consumptor_bite_x"), vis(self, other, "squirrel_consumptor_bite_y"))
  end
  return gm.array(vis(self, other, "squirrel_shroom_bite_x"), vis(self, other, "squirrel_shroom_bite_y"))
end

-- gml_Script____struct___77__squirrel_pick__scr_life @ 0x140107ec0
function ___struct___77__squirrel_pick__scr_life(self, other, mushroom, tree, home)
  self.mushroom = mushroom
  self.tree = tree
  self.home = home
end

-- gml_Script_squirrel_pick @ 0x1401080f0
function squirrel_pick(self, other, v, in_view)
  local picks = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if not gm.bool(squirrel_takes(self, other, m)) or gm.bool(m.stolen) or gm.bool(m.crow_target) then
      goto continue
    end
    if gm.bool(in_view) then
      if not (m.x >= v.x + 40 and m.x <= v.x + v.w - 40 and m.y >= v.y and m.y <= v.y + v.h) then
        goto continue
      end
    end
    do
      local taken = false
      local k = 0
      while k < gm.array_length(global.squirrels) do
        if global.squirrels[k].mushroom == m then
          taken = true
          break
        end
        k = k + 1
      end
      if gm.bool(taken) then
        goto continue
      end
      local tree = squirrel_tree_for(self, other, m)
      if gm.is_undefined(tree) then
        goto continue
      end
      local home
      if tree.sprite == tree03 then
        home = tree
      else
        home = squirrel_tree03_find(self, other, tree.x, 0, 0, true, tree)
      end
      if gm.is_undefined(home) then
        goto continue
      end
      gm.array_push(picks, gm.new(___struct___77__squirrel_pick__scr_life, m, tree, home))
    end
    ::continue::
    i = i + 1
  end
  if gm.array_length(picks) == 0 then
    return gm.undefined
  end
  return picks[gm.irandom(gm.array_length(picks) - 1)]
end

-- gml_Script_squirrel_sky_y @ 0x140109940
function squirrel_sky_y(self, other, s, v)
  local top = s.y - gm.sprite_get_yoffset(s.sprite)
  local margin = squirrel_frame_w(self, other) + 80
  if s.x > v.x - margin and s.x < v.x + v.w + margin then
    return gm.min(top, v.y)
  end
  return top
end

-- gml_Script_squirrel_any_target @ 0x14010a1b0
function squirrel_any_target(self, other)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if gm.bool(squirrel_takes(self, other, m)) and not gm.bool(m.stolen) and not gm.bool(m.crow_target) then
      local taken = false
      local k = 0
      while k < gm.array_length(global.squirrels) do
        if global.squirrels[k].mushroom == m then
          taken = true
          break
        end
        k = k + 1
      end
      if not gm.bool(taken) then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___78__squirrel_spawn__scr_life @ 0x14010a950
function ___struct___78__squirrel_spawn__scr_life(self, other, tree, home, target_tree, far_tree, direct, side, dir,
    mirror, y, stop_y, frame_timer, mushroom, empty_run)
  self.tree = tree
  self.home = home
  self.target_tree = target_tree
  self.far_tree = far_tree
  self.direct = direct
  self.side = side
  self.dir = dir
  self.mirror = mirror
  self.angle = 0
  self.x = 0
  self.y = y
  self.stop_y = stop_y
  self.run_line = 0
  self.state = 0
  self.turn_t = 0
  self.turn_from_angle = 0
  self.turn_to_angle = 0
  self.turn_from_x = 0
  self.turn_from_y = 0
  self.turn_to_x = 0
  self.turn_to_y = 0
  self.frame = 0
  self.frame_timer = frame_timer
  self.timer = 0
  self.mushroom = mushroom
  self.empty_run = empty_run
  self.grab_x = 0
  self.grab_y = 0
end

-- gml_Script_squirrel_spawn @ 0x14010b320
function squirrel_spawn(self, other, p, v)
  local home = p.home
  local direct = (p.tree == home)
  local dirc = gm.sign(p.tree.x - home.x)
  local side
  if gm.bool(direct) then
    side = gm.choose(-1, 1)
  else
    side = dirc
  end
  local stop_y
  if gm.bool(gm.variable_struct_exists(p, "stop_y")) then
    stop_y = p.stop_y
  else
    stop_y = 0
  end
  if gm.bool(direct) then
    if not gm.is_undefined(p.mushroom) then
      side = (p.mushroom.x < home.x) and -1 or 1
      stop_y = squirrel_bite_y(self, other, p.mushroom)
    end
  end
  local y0 = squirrel_sky_y(self, other, home, v)
    - vis(self, other, "squirrel_scale") * (squirrel_frame_h(self, other) - vis(self, other, "squirrel_body_y")) - 8
  local s = gm.new(___struct___78__squirrel_spawn__scr_life, home, home, p.tree, p.tree, direct, side, dirc, -side,
    y0, stop_y, vis(self, other, "squirrel_frame_interval"), p.mushroom, gm.is_undefined(p.mushroom))
  s.x = squirrel_climb_x(self, other, s.mirror, 0, home, side, y0)
  gm.array_push(global.squirrels, s)
end

-- gml_Script____struct___79__squirrel_summon__scr_life @ 0x14010ccc0
function ___struct___79__squirrel_summon__scr_life(self, other, mushroom, tree, home, stop_y)
  self.mushroom = mushroom
  self.tree = tree
  self.home = home
  self.stop_y = stop_y
end

-- gml_Script_squirrel_summon @ 0x14010cf70
function squirrel_summon(self, other)
  local v = squirrel_view(self, other)
  local p = squirrel_pick(self, other, v, true)
  if gm.is_undefined(p) then
    p = squirrel_pick(self, other, v, false)
  end
  if not gm.is_undefined(p) then
    squirrel_spawn(self, other, p, v)
    if p.tree == p.home then
      return "squirrel: tree mushroom on its tree03"
    end
    return "squirrel: over the ground to another tree"
  end
  local t = squirrel_tree_near(self, other, v.x + v.w * 0.5, v.w * 0.5, true, gm.undefined)
  if gm.is_undefined(t) then
    return "squirrel: no free tree03 on screen"
  end
  squirrel_spawn(self, other, gm.new(___struct___79__squirrel_summon__scr_life, gm.undefined, t, t,
    wld(self, other, "surface_height_px") - vis(self, other, "tree_mushroom_height_px")), v)
  return "squirrel: empty run"
end

-- gml_Script____struct___80__squirrel_hold_pose__scr_life @ 0x14010ded0
function ___struct___80__squirrel_hold_pose__scr_life(self, other, x, y, xscale, yscale, angle)
  self.x = x
  self.y = y
  self.xscale = xscale
  self.yscale = yscale
  self.angle = angle
end

-- gml_Script_squirrel_hold_pose @ 0x14010e200
function squirrel_hold_pose(self, other, s)
  local cs = vis(self, other, "squirrel_carry_scale")
  local dir = -s.mirror
  local xs = dir * cs
  local ys = cs
  local ang = gm.angle_difference(vis(self, other, "squirrel_carry_angle") * dir + s.angle, 0)
  local spr = mushroom_type_sprite(self, other, s.mushroom.type)
  local fwd = mushroom_cell_width(self, other, spr) * cs * vis(self, other, "squirrel_carry_forward")
    / vis(self, other, "squirrel_scale")
  local mouth = squirrel_point(self, other, s,
    vis(self, other, "squirrel_mouth_x") + vis(self, other, "squirrel_hold_into_head_x"),
    vis(self, other, "squirrel_mouth_y") + vis(self, other, "squirrel_hold_into_head_y") + fwd)
  local bite = squirrel_shroom_bite(self, other, s.mushroom.type)
  local bx = xs * bite[0]
  local by = ys * bite[1]
  local c = gm.dcos(ang)
  local sn = gm.dsin(ang)
  return gm.new(___struct___80__squirrel_hold_pose__scr_life,
    mouth[0] - (bx * c + by * sn),
    mouth[1] - (-bx * sn + by * c),
    xs, ys, ang)
end

-- gml_Script_squirrel_animate @ 0x14010fee0
function squirrel_animate(self, other, s)
  s.frame_timer = s.frame_timer - 1
  if s.frame_timer <= 0 then
    s.frame_timer = vis(self, other, "squirrel_frame_interval")
    s.frame = gm.mod(s.frame + 1, 9)
  end
end

-- gml_Script_squirrel_flip_up @ 0x140110440
function squirrel_flip_up(self, other, s)
  s.state = 4
  s.mirror = s.side
  s.angle = 180
end

-- gml_Script_squirrel_flip_down @ 0x1401106c0
function squirrel_flip_down(self, other, s)
  s.state = 0
  s.mirror = -s.side
  s.angle = 0
end

-- gml_Script_squirrel_tree03_find @ 0x140110910
function squirrel_tree03_find(self, other, x, dir, min_dist, need_free, exclude)
  if not gm.bool(gm.instance_exists(obj_forest_controller)) then
    return gm.undefined
  end
  local fc = gm.instance_find(obj_forest_controller, 0)
  local cw = fc.chunk_width
  local c0 = gm.floor(x / cw)
  local best = gm.undefined
  local best_d = gm.infinity
  local ring = 0
  while ring <= 256 do
    if (ring - 1) * cw > best_d then
      break
    end
    local cs
    if ring == 0 then
      cs = gm.array(c0)
    elseif dir == 0 then
      cs = gm.array(c0 - ring, c0 + ring)
    else
      cs = gm.array(c0 + dir * ring)
    end
    local k = 0
    while k < gm.array_length(cs) do
      local trees = fc.surface_get_chunk(self, other, cs[k]).trees
      local i = 0
      while i < gm.array_length(trees) do
        local t = trees[i]
        local d
        if dir == 0 then
          d = gm.abs(t.x - x)
        else
          d = (t.x - x) * dir
        end
        if t.sprite ~= tree03 then
          goto continue
        end
        if t == exclude or d >= best_d then
          goto continue
        end
        if dir ~= 0 and d < min_dist then
          goto continue
        end
        if gm.bool(need_free) and gm.bool(squirrel_tree_has_mushroom(self, other, t)) then
          goto continue
        end
        best_d = d
        best = t
        ::continue::
        i = i + 1
      end
      k = k + 1
    end
    ring = ring + 1
  end
  return best
end

-- gml_Script_squirrel_check_repeller @ 0x140112720
function squirrel_check_repeller(self, other, s)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 2 and gm.bool(m.active) and m.carried_by == -1 then
      local mouth = squirrel_point(self, other, s, vis(self, other, "squirrel_mouth_x"), vis(self, other, "squirrel_mouth_y"))
      local d = (m.x - mouth[0]) * s.dir
      if d >= 0 and not (d > bal(self, other, "squirrel_repeller_reach_px")) then
        repeller_cloud_fire(self, other, m, -s.dir)
        if not gm.is_undefined(s.mushroom) and not gm.bool(s.mushroom.stolen) then
          s.mushroom = gm.undefined
          s.empty_run = true
        end
        s.dir = -s.dir
        s.mirror = -s.mirror
        s.angle = -s.angle
        local t = squirrel_tree03_find(self, other, s.x, s.dir, vis(self, other, "squirrel_jump_dx") + 40, false, gm.undefined)
        local nt
        if not gm.is_undefined(t) then
          nt = t
        elseif s.target_tree == s.home then
          nt = s.far_tree
        else
          nt = s.home
        end
        s.target_tree = nt
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script_squirrel_turn_start @ 0x140113f50
function squirrel_turn_start(self, other, s, state, to_angle, to_x, to_y)
  s.state = state
  s.turn_t = 0
  s.turn_from_angle = s.angle
  s.turn_to_angle = to_angle
  s.turn_from_x = s.x
  s.turn_from_y = s.y
  s.turn_to_x = to_x
  s.turn_to_y = to_y
end

-- gml_Script_squirrel_turn_step @ 0x140114650
function squirrel_turn_step(self, other, s)
  local t = s.turn_t
  local frames = gm.max(1, vis(self, other, "squirrel_turn_frames"))
  s.turn_t = gm.min(1, t + 1 / frames)
  local u = 1 - s.turn_t
  local e = 1 - u * u
  s.angle = gm.lerp(s.turn_from_angle, s.turn_to_angle, e)
  s.x = gm.lerp(s.turn_from_x, s.turn_to_x, e)
  s.y = gm.lerp(s.turn_from_y, s.turn_to_y, e)
  return s.turn_t >= 1
end

-- gml_Script_crow_frame_w @ 0x1401199f0
function crow_frame_w(self, other)
  return gm.div(gm.sprite_get_width(crow), 9)
end

local crow_claw_static = {}

-- gml_Script_crow_claw @ 0x140119bb0
function crow_claw(self, other, frame)
  if crow_claw_static.claws == nil then
    crow_claw_static.claws = gm.array(
      gm.array(74, 83), gm.array(70, 82), gm.array(70, 80), gm.array(70, 77), gm.array(76, 95),
      gm.array(76, 81), gm.array(74, 79), gm.array(74, 81), gm.array(74, 83))
  end
  return crow_claw_static.claws[gm.floor(gm.clamp(frame, 0, 8))]
end

-- gml_Script_crow_below_claws @ 0x14011a0f0
function crow_below_claws(self, other)
  local c = crow_claw(self, other, 0)
  return (gm.sprite_get_height(crow) - c[1]) * vis(self, other, "crow_scale")
end

-- gml_Script_crow_claw_offset @ 0x14011a500
function crow_claw_offset(self, other, frame, dir)
  local sc = vis(self, other, "crow_scale")
  local c0 = crow_claw(self, other, 0)
  local c = crow_claw(self, other, frame)
  local ndir = -dir
  local dx = (c[0] - c0[0]) * ndir * sc
  local dy = (c[1] - c0[1]) * sc
  return gm.array(dx, dy)
end

-- gml_Script_crow_cap_dy @ 0x14011ad10
function crow_cap_dy(self, other, m)
  local spr = mushroom_type_sprite(self, other, m.type)
  return gm.sprite_get_bbox_top(spr) - gm.sprite_get_yoffset(spr) + vis(self, other, "crow_grab_into_cap_px")
end

-- gml_Script_crow_can_take @ 0x14011b190
function crow_can_take(self, other, m)
  if gm.bool(m.stolen) or gm.bool(m.crow_target) or m.carried_by ~= -1 then
    return false
  end
  local i = 0
  while i < gm.array_length(global.squirrels) do
    if global.squirrels[i].mushroom == m then
      return false
    end
    i = i + 1
  end
  return true
end

-- gml_Script_crow_pick @ 0x14011b5c0
function crow_pick(self, other, v, type)
  if gm.is_undefined(type) then
    type = -1
  end
  local in_view = gm.array()
  local any = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if not gm.bool(crow_can_take(self, other, m)) then
      goto continue
    end
    if type >= 0 and m.type ~= type then
      goto continue
    end
    gm.array_push(any, m)
    if m.x > v.x + 40 and m.x < v.x + v.w - 40 and m.y > v.y and m.y < v.y + v.h then
      gm.array_push(in_view, m)
    end
    ::continue::
    i = i + 1
  end
  local pool
  if gm.array_length(in_view) > 0 then
    pool = in_view
  else
    pool = any
  end
  if gm.array_length(pool) > 0 then
    return pool[gm.irandom(gm.array_length(pool) - 1)]
  end
  return gm.undefined
end

-- gml_Script_crow_sky_y @ 0x14011c5b0
function crow_sky_y(self, other, v, h)
  return gm.min(0, v.y) - h - 10
end

-- gml_Script____struct___81__crow_spawn__scr_life @ 0x14011c8c0
function ___struct___81__crow_spawn__scr_life(self, other, dir, x, y, x0, y0, grab_x, grab_y, frame_timer, mushroom)
  self.state = 0
  self.dir = dir
  self.x = x
  self.y = y
  self.x0 = x0
  self.y0 = y0
  self.grab_x = grab_x
  self.grab_y = grab_y
  self.t = 0
  self.frame = 0
  self.frame_timer = frame_timer
  self.timer = 0
  self.speed = 0
  self.flee = false
  self.mushroom = mushroom
end

-- gml_Script_crow_spawn @ 0x14011cf40
function crow_spawn(self, other, m, v, dir)
  if gm.is_undefined(dir) then
    dir = gm.choose(-1, 1)
  end
  local below = crow_below_claws(self, other)
  local y0 = crow_sky_y(self, other, v, below)
  local x0 = m.x - dir * vis(self, other, "crow_dive_dx_px")
  m.crow_target = true
  local grab_y = m.y + crow_cap_dy(self, other, m)
  local c = gm.new(___struct___81__crow_spawn__scr_life, dir, x0, y0, x0, y0,
    m.x, grab_y, vis(self, other, "crow_frame_interval"), m)
  gm.array_push(global.crows, c)
end

-- gml_Script____struct___82__crow_warn_check__scr_life @ 0x14011dd10
function ___struct___82__crow_warn_check__scr_life(self, other, mushroom, dir, x, y)
  self.mushroom = mushroom
  self.dir = dir
  self.x = x
  self.y = y
end

-- gml_Script_crow_warn_check @ 0x14011dfe0
function crow_warn_check(self, other, slot, timer, v, type)
  if not gm.bool(gm.variable_global_exists("crow_pending")) then
    global.crow_pending = gm.array(gm.undefined, gm.undefined, gm.undefined)
  end
  if not gm.is_undefined(global.crow_pending[slot]) then
    return
  end
  if timer / animal_timer_step(self, other) > vis(self, other, "crow_warning_seconds") * global.fps then
    return
  end
  local m = crow_pick(self, other, v, type)
  if gm.is_undefined(m) then
    return
  end
  local dir = gm.choose(-1, 1)
  global.crow_pending[slot] = gm.new(___struct___82__crow_warn_check__scr_life, m, dir,
    m.x - dir * vis(self, other, "crow_dive_dx_px"), crow_sky_y(self, other, v, crow_below_claws(self, other)))
end

-- gml_Script_crow_due @ 0x14011edf0
function crow_due(self, other, slot, v, type)
  local p
  if gm.bool(gm.variable_global_exists("crow_pending")) then
    p = global.crow_pending[slot]
  else
    p = gm.undefined
  end
  if gm.bool(gm.variable_global_exists("crow_pending")) then
    global.crow_pending[slot] = gm.undefined
  end
  if not gm.is_undefined(p) and gm.bool(gm.array_contains(global.mushrooms, p.mushroom))
      and gm.bool(crow_can_take(self, other, p.mushroom)) then
    crow_spawn(self, other, p.mushroom, v, p.dir)
    if gm.bool(gm.variable_global_exists("animal_markers")) then
      local i = 0
      while i < gm.array_length(global.animal_markers) do
        if global.animal_markers[i].src == p then
          global.animal_markers[i].src = global.crows[gm.array_length(global.crows) - 1]
          global.animal_markers[i].warn = false
        end
        i = i + 1
      end
    end
    return
  end
  local m = crow_pick(self, other, v, type)
  if not gm.is_undefined(m) then
    crow_spawn(self, other, m, v, gm.undefined)
  end
end

-- gml_Script_crow_summon @ 0x14011ff90
function crow_summon(self, other)
  local v = squirrel_view(self, other)
  local m = crow_pick(self, other, v, gm.undefined)
  if gm.is_undefined(m) then
    return "crow: no free mushroom"
  end
  crow_spawn(self, other, m, v, gm.undefined)
  return "crow"
end

-- gml_Script_crows_step @ 0x140120340
function crows_step(self, other)
  local v = squirrel_view(self, other)
  local any = false
  local i = 0
  while i < gm.array_length(global.mushrooms) and not gm.bool(any) do
    any = crow_can_take(self, other, global.mushrooms[i])
    i = i + 1
  end
  if gm.bool(any) then
    global.crow_timer = global.crow_timer - animal_timer_step(self, other)
    crow_warn_check(self, other, 0, global.crow_timer, v, -1)
    if global.crow_timer <= 0 then
      global.crow_timer = animal_interval(self, other, "crow_spawn_interval")
      crow_due(self, other, 0, v, -1)
    end
  end
  local any_exp = false
  i = 0
  while i < gm.array_length(global.mushrooms) and not gm.bool(any_exp) do
    local ok = false
    if global.mushrooms[i].type == 6 then
      ok = gm.bool(crow_can_take(self, other, global.mushrooms[i]))
    end
    any_exp = ok
    i = i + 1
  end
  if gm.bool(any_exp) then
    local k = 0
    while k < gm.array_length(global.experience_crow_timers) do
      global.experience_crow_timers[k] = global.experience_crow_timers[k] - animal_timer_step(self, other)
      crow_warn_check(self, other, 1 + k, global.experience_crow_timers[k], v, 6)
      if global.experience_crow_timers[k] <= 0 then
        global.experience_crow_timers[k] = animal_interval(self, other, "experience_crow_interval")
        crow_due(self, other, 1 + k, v, 6)
      end
      k = k + 1
    end
  end
  local keep = gm.array()
  local ci = 0
  while ci < gm.array_length(global.crows) do
    local c = global.crows[ci]
    local m = c.mushroom
    if not gm.is_undefined(m) and mushroom_index_of(self, other, m) < 0 then
      c.mushroom = gm.undefined
      m = gm.undefined
    end
    local state = c.state
    if state == 0 then
      if not gm.is_undefined(m) then
        c.grab_x = m.x
        c.grab_y = m.y + crow_cap_dy(self, other, m)
      end
      local sc_up = vis(self, other, "crow_body_up_px") * vis(self, other, "crow_scale")
      local sc_pad = vis(self, other, "crow_body_pad_px") * vis(self, other, "crow_scale")
      if not gm.bool(stinky_beam_hits(self, other, c.x, c.y - sc_up, sc_pad)) then
        local co = crow_claw_offset(self, other, 4, c.dir)
        local tx = c.grab_x - co[0]
        local ty = c.grab_y - co[1]
        local dist = gm.max(1, gm.point_distance(c.x0, c.y0, tx, ty))
        c.t = gm.min(1, c.t + bal(self, other, "crow_dive_speed") / dist)
        local e = 1 - (1 - c.t) * (1 - c.t)
        c.x = gm.lerp(c.x0, tx, c.t)
        c.y = gm.lerp(c.y0, ty, e)
        c.frame = gm.min(3, gm.floor(c.t * 4))
        if c.t >= 1 then
          c.state = 1
          c.frame = 4
          c.timer = vis(self, other, "crow_grab_frames")
          if not gm.is_undefined(m) then
            m.stolen = true
          end
        end
      else
        tasks_stat_add(self, other, "ach_stinky_scares", gm.undefined)
        if not gm.is_undefined(m) then
          m.crow_target = false
        end
        c.mushroom = gm.undefined
        m = gm.undefined
        c.state = 2
        c.frame = 5
        c.frame_timer = vis(self, other, "crow_frame_interval")
        c.speed = bal(self, other, "crow_climb_speed") * vis(self, other, "crow_flee_start_speed")
        c.flee = true
      end
    elseif state == 1 then
      c.timer = c.timer - 1
      if c.timer <= 0 then
        c.state = 2
        c.frame = 5
        c.frame_timer = vis(self, other, "crow_frame_interval")
      end
    elseif state == 2 then
      c.frame_timer = c.frame_timer - 1
      if c.frame_timer <= 0 then
        c.frame_timer = vis(self, other, "crow_frame_interval")
        if c.frame >= 8 then
          c.frame = 5
        else
          c.frame = c.frame + 1
        end
      end
      local climb = bal(self, other, "crow_climb_speed")
      c.speed = gm.min(climb, c.speed + climb / gm.max(1, vis(self, other, "crow_climb_ease_frames")))
      local fwd
      if not gm.bool(c.flee) then
        fwd = vis(self, other, "crow_climb_forward")
      else
        fwd = vis(self, other, "crow_flee_forward")
      end
      c.x = c.x + c.dir * c.speed * fwd
      c.y = c.y - c.speed
    end
    if not gm.is_undefined(m) and gm.bool(m.stolen) then
      local co = crow_claw_offset(self, other, c.frame, c.dir)
      m.x = c.x + co[0]
      m.y = c.y + co[1] - crow_cap_dy(self, other, m)
    end
    local below = crow_below_claws(self, other)
    if not gm.is_undefined(m) then
      below = gm.max(below, gm.sprite_get_height(mushroom_type_sprite(self, other, m.type)))
    end
    if c.state == 2 and c.y < crow_sky_y(self, other, v, below) then
      local idx
      if not gm.is_undefined(m) then
        idx = mushroom_index_of(self, other, m)
      else
        idx = -1
      end
      if idx >= 0 then
        gm.array_delete(global.mushrooms, idx, 1)
        animals_reindex_after_removal(self, other, idx)
        animals_rebind(self, other)
      end
    else
      gm.array_push(keep, c)
    end
    ci = ci + 1
  end
  global.crows = keep
end
