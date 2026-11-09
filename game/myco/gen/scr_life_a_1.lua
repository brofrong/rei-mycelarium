local gm = require("gm")

-- gml_Script_hedgehog_frame_w @ 0x1400a9490
function hedgehog_frame_w(self, other)
  return gm.sprite_get_width(hedgehog) / 8.0 * vis(self, other, "hedgehog_scale")
end

-- gml_Script_hedgehog_frame_h @ 0x1400a9780
function hedgehog_frame_h(self, other)
  return gm.sprite_get_height(hedgehog) * vis(self, other, "hedgehog_scale")
end

-- gml_Script_hedgehog_back_y @ 0x1400a99e0
function hedgehog_back_y(self, other)
  return -hedgehog_frame_h(self, other) * 0.68
end

-- gml_Script____struct___54__hedgehog_back_init__scr_life @ 0x1400a9b60
function ___struct___54__hedgehog_back_init__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_hedgehog_back_init @ 0x1400a9d60
function hedgehog_back_init(self, other)
  if gm.bool(gm.variable_global_exists("hedgehog_back")) and gm.is_array(global.hedgehog_back) then
    return
  end
  local w = gm.sprite_get_width(hedgehog)
  local h = gm.sprite_get_height(hedgehog)
  local fw = gm.div(w, 8)
  local surf = gm.surface_create(w, h)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(2.0, 6.0)
  gm.draw_sprite(hedgehog, 0, gm.sprite_get_xoffset(hedgehog), gm.sprite_get_yoffset(hedgehog))
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local buf = gm.buffer_create(w * h * 4, 0, 1.0)
  gm.buffer_get_surface(buf, surf, 0)
  gm.surface_free(surf)
  local top = gm.array_create(w, h)
  local xx = 0
  while xx < w do
    local yy = 0
    while yy < h do
      if gm.buffer_peek(buf, (yy * w + xx) * 4 + 3, 1.0) >= 128.0 then
        top[xx] = yy
        break
      end
      yy = yy + 1
    end
    xx = xx + 1
  end
  gm.buffer_delete(buf)
  local smooth = vis(self, other, "hedgehog_back_smooth_cols")
  local sm = gm.array_create(w, h)
  xx = 0
  while xx < w do
    local f0 = gm.div(xx, fw) * fw
    local sum = 0
    local n = 0
    local k = -smooth
    while k <= smooth do
      local c = xx + k
      if not (c < f0) and not (c >= f0 + fw) then
        sum = sum + top[c]
        n = n + 1
      end
      k = k + 1
    end
    sm[xx] = sum / gm.max(1.0, n)
    xx = xx + 1
  end
  local best = 0
  xx = 1
  while xx < fw do
    if sm[xx] < sm[best] then
      best = xx
    end
    xx = xx + 1
  end
  local follow = vis(self, other, "hedgehog_back_follow_cols")
  local res = gm.array_create(8.0)
  local f = 0
  while f < 8.0 do
    local bx = gm.clamp(best, 0, fw - 1)
    local c = gm.max(0, best - follow)
    while c <= gm.min(fw - 1, best + follow) do
      if sm[f * fw + c] < sm[f * fw + bx] then
        bx = c
      end
      c = c + 1
    end
    res[f] = gm.new(___struct___54__hedgehog_back_init__scr_life, bx + 0.5, sm[f * fw + bx])
    f = f + 1
  end
  global.hedgehog_back = res
end

-- gml_Script_hedgehog_back_point @ 0x1400acde0
function hedgehog_back_point(self, other, x, y, dir, frame)
  if gm.bool(gm.variable_global_exists("hedgehog_back")) and gm.is_array(global.hedgehog_back) then
    local s = vis(self, other, "hedgehog_scale")
    local fw = gm.sprite_get_width(hedgehog) / 8.0
    local fh = gm.sprite_get_height(hedgehog)
    local sx
    if dir >= 0 then
      sx = -1.0
    else
      sx = 1.0
    end
    local p = global.hedgehog_back[gm.floor(gm.clamp(frame, 0, 7.0))]
    return gm.array(gm.floor(x) + sx * (p.x + vis(self, other, "hedgehog_carry_back_px") - fw * 0.5) * s,
      gm.floor(y) - fh * s + (p.y + vis(self, other, "hedgehog_carry_sink_px")) * s)
  end
  return gm.array(x, y + hedgehog_back_y(self, other))
end

-- gml_Script_hedgehog_draw @ 0x1400ae120
function hedgehog_draw(self, other, x, y, dir, frame)
  local s = vis(self, other, "hedgehog_scale")
  local fw = gm.sprite_get_width(hedgehog) / 8.0
  local fh = gm.sprite_get_height(hedgehog)
  local sx
  if dir >= 0 then
    sx = -1.0
  else
    sx = 1.0
  end
  local dx = gm.floor(x) - sx * fw * s * 0.5
  gm.draw_sprite_part_ext(hedgehog, 0, frame * fw, 0, fw, fh, dx, gm.floor(y) - fh * s, sx * s, s, 16777215.0, 1.0)
end

-- gml_Script_animal_interval @ 0x1400aee70
function animal_interval(self, other, key)
  local j = gm.clamp(bal(self, other, "animal_interval_jitter"), 0, 0.95)
  local lo = 1.0 - j
  local hi = 1.0 + j
  return bal(self, other, key) * global.fps * gm.random_range(lo, hi)
end

-- gml_Script_animal_mushroom_count @ 0x1400af2d0
function animal_mushroom_count(self, other)
  local n = 0
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.carried_by == -1.0 and not gm.bool(m.stolen) then
      n = n + 1
    end
    i = i + 1
  end
  return n
end

-- gml_Script_animal_timer_step @ 0x1400af6d0
function animal_timer_step(self, other)
  local max_cut = bal(self, other, "animal_interval_max_cut")
  local per = bal(self, other, "animal_interval_per_mushroom")
  local cut = gm.min(max_cut, animal_mushroom_count(self, other) * per)
  return 1.0 / gm.max(0.05, 1.0 - cut)
end

-- gml_Script_life_init @ 0x1400afbc0
function life_init(self, other)
  global.fps = gm.max(1.0, gm.game_get_speed(0))
  global.mushrooms = gm.array()
  global.animals = gm.array()
  global.leaves = gm.array()
  global.leaf_tendrils_out = gm.array()
  global.drops = gm.array()
  global.squirrels = gm.array()
  global.boars = gm.array()
  global.boar_flung = gm.array()
  global.boar_pack = gm.array()
  global.repel_clouds = gm.array()
  global.repel_kick = 0
  global.crows = gm.array()
  global.crow_pending = gm.array(gm.undefined, gm.undefined, gm.undefined)
  life_animal_timers_init(self, other)
  global.leaf_timer = bal(self, other, "leaf_spawn_interval")
  global.worms = gm.array()
  global.worm_trail = gm.array()
  global.worm_timer = 60.0
  global.stinky_live_count = 0
  global.stinky_live_last = 0
  life_far_init(self, other)
  intro_init(self, other)
end

-- gml_Script_life_animal_timers_init @ 0x1400b0990
function life_animal_timers_init(self, other)
  local dmax = bal(self, other, "hedgehog_first_delay_max")
  local dmin = bal(self, other, "hedgehog_first_delay_min")
  local r = gm.random_range(dmin, dmax)
  global.hedgehog_timer = (bal(self, other, "hedgehog_spawn_interval") + r) * global.fps
  global.squirrel_timer = animal_interval(self, other, "squirrel_spawn_interval")
  global.boar_timer = bal(self, other, "boar_first_delay") * global.fps
  global.crow_timer = animal_interval(self, other, "crow_spawn_interval")
  local t0 = bal(self, other, "experience_crow_interval") * global.fps
  local off = bal(self, other, "experience_crow_offset")
  local t1 = (bal(self, other, "experience_crow_interval") + off) * global.fps
  global.experience_crow_timers = gm.array(t0, t1)
end

-- gml_Script_repeller_autoregen_frames @ 0x1400b9180
function repeller_autoregen_frames(self, other)
  local lvl = upg_level(self, other, "repeller_autoregen")
  if not (lvl > 0) then
    return -1.0
  end
  local mmin = bal(self, other, "repeller_autoregen_minutes_min")
  local cut = (lvl - 1) * bal(self, other, "repeller_autoregen_minutes_step")
  local mins = gm.max(mmin, bal(self, other, "repeller_autoregen_minutes_first") - cut)
  return mins * 60 * global.fps * perk_time_mult(self, other)
end

-- gml_Script_repeller_regen_price @ 0x1400b9a20
function repeller_regen_price(self, other)
  local k = gm.max(0.2, 1.0 - upg_bonus(self, other, "repeller_regen_cost"))
  return gm.max(1.0, gm.round(bal(self, other, "repeller_regen_cost") * k))
end

-- gml_Script_tree_drop_period @ 0x1400b9ea0
function tree_drop_period(self, other)
  local rate = 1.0 + upg_bonus(self, other, "tree_drop_rate")
  return gm.max(30.0, bal(self, other, "tree_drop_interval") / rate * perk_time_mult(self, other))
end

-- gml_Script_tree_drop_water @ 0x1400ba290
function tree_drop_water(self, other)
  local yield = 1.0 + upg_bonus(self, other, "tree_water_yield")
  return bal(self, other, "tree_water_per_drop") * yield
end

-- gml_Script_leaf_face_for_wind @ 0x1400ba520
function leaf_face_for_wind(self, other, wind)
  local face
  if wind >= 0 then
    face = 1.0
  else
    face = 0
  end
  if vis(self, other, "leaf_face_flip") > 0 then
    return 1 - face
  end
  return face
end

-- gml_Script_leaf_face_sprite @ 0x1400ba8d0
function leaf_face_sprite(self, other, face)
  if face ~= 0 then
    return leaf_2
  end
  return leaf
end

-- gml_Script_leaf_change_sprite @ 0x1400baa70
function leaf_change_sprite(self, other, face)
  if face ~= 0 then
    return leaf_change_2
  end
  return leaf_change
end

-- gml_Script_leaf_art_ready @ 0x1400bac10
function leaf_art_ready(self, other)
  return gm.bool(gm.variable_global_exists("leaf_art")) and gm.is_struct(global.leaf_art)
end

-- gml_Script____struct___55__leaf_art_init__scr_life @ 0x1400bae60
function ___struct___55__leaf_art_init__scr_life(self, other, faces, turns)
  self.faces = faces
  self.turns = turns
end

-- gml_Script_leaf_art_init @ 0x1400bb030
function leaf_art_init(self, other)
  if leaf_art_ready(self, other) then
    return
  end
  local f0 = leaf_art_read(self, other, leaf, 1.0)
  local f1 = leaf_art_read(self, other, leaf_2, 1.0)
  local t0 = leaf_art_read(self, other, leaf_change, 8.0)
  local t1 = leaf_art_read(self, other, leaf_change_2, 8.0)
  global.leaf_art = gm.new(___struct___55__leaf_art_init__scr_life, gm.array(f0, f1), gm.array(t0, t1))
end

-- gml_Script____struct___56__leaf_art_read__scr_life @ 0x1400bb8a0
function ___struct___56__leaf_art_read__scr_life(self, other, sprite, shape, x1, y1, x2, y2, mid_x, mid_y)
  self.sprite = sprite
  self.shape = shape
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.mid_x = mid_x
  self.mid_y = mid_y
end

-- gml_Script_leaf_art_read @ 0x1400bbcf0
function leaf_art_read(self, other, spr, frames)
  local w = gm.sprite_get_width(spr)
  local h = gm.sprite_get_height(spr)
  local fw = gm.div(w, frames)
  local surf = gm.surface_create(w, h)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(2.0, 6.0)
  gm.draw_sprite(spr, 0, gm.sprite_get_xoffset(spr), gm.sprite_get_yoffset(spr))
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local buf = gm.buffer_create(w * h * 4, 0, 1.0)
  gm.buffer_get_surface(buf, surf, 0)
  local sz = fw + 4
  local shp = gm.surface_create(sz, sz)
  local out = gm.array()
  local i = 0
  while i < frames do
    local x1 = fw
    local y1 = h
    local x2 = -1.0
    local y2 = -1.0
    local sx = 0
    local sy = 0
    local n = 0
    local yy = 0
    while yy < h do
      local xx = 0
      while xx < fw do
        local a = gm.buffer_peek(buf, ((yy * w + i * fw) + xx) * 4 + 3, 1.0)
        if a >= 128.0 then
          x1 = gm.min(x1, xx)
          y1 = gm.min(y1, yy)
          x2 = gm.max(x2, xx)
          y2 = gm.max(y2, yy)
          sx = sx + xx
          sy = sy + yy
          n = n + 1
        end
        xx = xx + 1
      end
      yy = yy + 1
    end
    if x2 < x1 then
      x1 = 0
      y1 = 0
      x2 = fw - 1
      y2 = h - 1
    end
    gm.surface_set_target(shp)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(2.0, 6.0)
    gm.draw_sprite_part(spr, 0, i * fw, 0, fw, h, 2.0, 2.0)
    gm.gpu_set_blendmode(0)
    gm.surface_reset_target()
    local s = gm.sprite_create_from_surface(shp, 0, 0, sz, sz, false, false, sz * 0.5, sz * 0.5)
    gm.surface_set_target(shp)
    gm.gpu_set_blendmode_ext_sepalpha(2.0, 1.0, 1.0, 2.0)
    gm.draw_rectangle_colour(0, 0, sz, sz, 16777215.0, 16777215.0, 16777215.0, 16777215.0, false)
    gm.gpu_set_blendmode(0)
    gm.surface_reset_target()
    local shape = gm.sprite_create_from_surface(shp, 0, 0, sz, sz, false, false, sz * 0.5, sz * 0.5)
    local mid_x, mid_y
    if n > 0 then
      mid_x = sx / n
    else
      mid_x = fw * 0.5
    end
    if n > 0 then
      mid_y = sy / n
    else
      mid_y = h * 0.5
    end
    gm.array_push(out, gm.new(___struct___56__leaf_art_read__scr_life, s, shape, x1, y1, x2, y2, mid_x, mid_y))
    i = i + 1
  end
  gm.surface_free(shp)
  gm.surface_free(surf)
  gm.buffer_delete(buf)
  i = gm.array_length(out) - 1
  while i >= 0 do
    out[i].off_x = out[i].mid_x - out[0].mid_x
    out[i].off_y = out[i].mid_y - out[0].mid_y
    i = i - 1
  end
  return out
end

-- gml_Script_leaf_art_frame @ 0x1400bfa60
function leaf_art_frame(self, other, l)
  local cf
  if gm.bool(gm.variable_struct_exists(l, "change_from")) then
    cf = l.change_from
  else
    cf = -1.0
  end
  if cf >= 0 then
    return global.leaf_art.turns[gm.floor(cf)][gm.floor(l.change_frame)]
  end
  local face
  if gm.bool(gm.variable_struct_exists(l, "face")) then
    face = l.face
  else
    face = 0
  end
  return global.leaf_art.faces[gm.floor(face)][0]
end

-- gml_Script_leaf_fg_art_ready @ 0x1400c02b0
function leaf_fg_art_ready(self, other)
  return gm.bool(gm.variable_global_exists("leaf_fg_art")) and gm.is_struct(global.leaf_fg_art)
end

-- gml_Script____struct___57__leaf_fg_art_init__scr_life @ 0x1400c0500
function ___struct___57__leaf_fg_art_init__scr_life(self, other, rest, turn, turned, carry_x, carry_y)
  self.rest = rest
  self.turn = turn
  self.turned = turned
  self.carry_x = carry_x
  self.carry_y = carry_y
end

-- gml_Script_leaf_fg_art_init @ 0x1400c0810
function leaf_fg_art_init(self, other)
  if leaf_fg_art_ready(self, other) then
    return
  end
  local turn = leaf_fg_art_read(self, other, leaf_fg_anim, 14.0, true)
  local back = leaf_fg_art_read(self, other, leaf_fg_anim, 14.0, false)
  local last = turn[gm.array_length(turn) - 1]
  global.leaf_fg_art = gm.new(___struct___57__leaf_fg_art_init__scr_life, turn[0], turn,
    back[gm.array_length(back) - 1], last.mid_x, last.mid_y)
end

-- gml_Script____struct___58__leaf_fg_art_read__scr_life @ 0x1400c10b0
function ___struct___58__leaf_fg_art_read__scr_life(self, other, sprite, x1, y1, x2, y2, mid_x, mid_y)
  self.sprite = sprite
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.mid_x = mid_x
  self.mid_y = mid_y
end

-- gml_Script_leaf_fg_art_read @ 0x1400c1490
function leaf_fg_art_read(self, other, spr, frames, anchor_first)
  local w = gm.sprite_get_width(spr)
  local h = gm.sprite_get_height(spr)
  local fw = gm.div(w, frames)
  local surf = gm.surface_create(w, h)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(2.0, 6.0)
  gm.draw_sprite(spr, 0, gm.sprite_get_xoffset(spr), gm.sprite_get_yoffset(spr))
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local buf = gm.buffer_create(w * h * 4, 0, 1.0)
  gm.buffer_get_surface(buf, surf, 0)
  local sw = fw + 4
  local sh = h + 4
  local shp = gm.surface_create(sw, sh)
  local out = gm.array()
  local ref_x = -1.0
  local ref_y = -1.0
  local i = 0
  while i < frames do
    local x1 = fw
    local y1 = h
    local x2 = -1.0
    local y2 = -1.0
    local sx = 0
    local sy = 0
    local n = 0
    local yy = 0
    while yy < h do
      local xx = 0
      while xx < fw do
        local a = gm.buffer_peek(buf, ((yy * w + i * fw) + xx) * 4 + 3, 1.0)
        if a >= 128.0 then
          x1 = gm.min(x1, xx)
          y1 = gm.min(y1, yy)
          x2 = gm.max(x2, xx)
          y2 = gm.max(y2, yy)
          sx = sx + xx
          sy = sy + yy
          n = n + 1
        end
        xx = xx + 1
      end
      yy = yy + 1
    end
    if x2 < x1 then
      x1 = 0
      y1 = 0
      x2 = fw - 1
      y2 = h - 1
    end
    local mx, my
    if n > 0 then
      mx = sx / n
    else
      mx = fw * 0.5
    end
    if n > 0 then
      my = sy / n
    else
      my = h * 0.5
    end
    if ref_x < 0 or not gm.bool(anchor_first) then
      ref_x = mx
      ref_y = my
    end
    gm.surface_set_target(shp)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(2.0, 6.0)
    gm.draw_sprite_part(spr, 0, i * fw, 0, fw, h, 2.0, 2.0)
    gm.gpu_set_blendmode(0)
    gm.surface_reset_target()
    local s = gm.sprite_create_from_surface(shp, 0, 0, sw, sh, false, false, ref_x + 2, ref_y + 2)
    gm.array_push(out, gm.new(___struct___58__leaf_fg_art_read__scr_life, s, x1 - ref_x, y1 - ref_y,
      x2 + 1 - ref_x, y2 + 1 - ref_y, mx - ref_x, my - ref_y))
    i = i + 1
  end
  gm.surface_free(shp)
  gm.surface_free(surf)
  gm.buffer_delete(buf)
  return out
end

-- gml_Script_leaf_fg_frame @ 0x1400c4be0
function leaf_fg_frame(self, other, l)
  local t
  if gm.bool(gm.variable_struct_exists(l, "turn")) then
    t = l.turn
  else
    t = -1.0
  end
  if t >= 0 then
    return global.leaf_fg_art.turn[gm.floor(gm.clamp(t, 0, 13.0))]
  end
  if gm.bool(gm.variable_struct_exists(l, "turned")) and gm.bool(l.turned) then
    return global.leaf_fg_art.turned
  end
  return global.leaf_fg_art.rest
end

-- gml_Script_leaf_fg_mirror @ 0x1400c5260
function leaf_fg_mirror(self, other, l)
  local m
  if gm.bool(gm.variable_struct_exists(l, "mirror")) then
    m = l.mirror
  else
    m = 0
  end
  if not (m > 0) then
    return 1.0
  end
  return -1.0
end

-- gml_Script_leaf_start_turn @ 0x1400c5690
function leaf_start_turn(self, other, l)
  return false
end

-- gml_Script_leaves_turn_all @ 0x1400c57c0
function leaves_turn_all(self, other)
  local i = 0
  while i < gm.array_length(global.leaves) do
    leaf_start_turn(self, other, global.leaves[i])
    i = i + 1
  end
end

-- gml_Script_leaf_fg_drop @ 0x1400c5ad0
function leaf_fg_drop(self, other, l)
  local f = leaf_fg_frame(self, other, l)
  local ang
  if gm.bool(gm.variable_struct_exists(l, "angle")) then
    ang = l.angle
  else
    ang = 0
  end
  local m = leaf_fg_mirror(self, other, l)
  local pts = gm.array(gm.array(f.x1 * m, f.y1), gm.array(f.x2 * m, f.y1),
    gm.array(f.x1 * m, f.y2), gm.array(f.x2 * m, f.y2))
  local best = -9999.0
  local i = 0
  while i < 4.0 do
    local d = gm.point_distance(0, 0, pts[i][0], pts[i][1])
    local a = gm.point_direction(0, 0, pts[i][0], pts[i][1]) + ang
    best = gm.max(best, gm.lengthdir_y(d, a))
    i = i + 1
  end
  return best
end

-- gml_Script_leaf_draw @ 0x1400c6fc0
function leaf_draw(self, other, l, alpha)
  leaf_fg_art_init(self, other)
  local f = leaf_fg_frame(self, other, l)
  local ang
  if gm.bool(gm.variable_struct_exists(l, "angle")) then
    ang = l.angle
  else
    ang = 0
  end
  local m = leaf_fg_mirror(self, other, l)
  gm.draw_sprite_ext(f.sprite, 0, gm.round(l.x), gm.round(l.y), m, 1.0, ang, 16777215, alpha)
end

-- gml_Script_leaf_decay_time @ 0x1400c7820
function leaf_decay_time(self, other)
  local k = 1.0 + upg_bonus(self, other, "absorption_speed")
  return gm.max(30.0, bal(self, other, "leaf_absorb_time") / k)
end

-- gml_Script_leaf_food_value @ 0x1400c7b60
function leaf_food_value(self, other)
  local b = upg_bonus(self, other, "absorption_yield")
  return gm.floor(bal(self, other, "leaf_food_reward") + b)
end

-- gml_Script____struct___59__mushroom_add__scr_life @ 0x1400c7e20
function ___struct___59__mushroom_add__scr_life(self, other, type, variant, x, y, charges, flies, carried_by, host, stench, born)
  self.type = type
  self.variant = variant
  self.x = x
  self.y = y
  self.active = true
  self.timer = 0
  self.charges = charges
  self.flies = flies
  self.carried_by = carried_by
  self.stolen = false
  self.crow_target = false
  self.host = host
  self.side = 1.0
  self.stench = stench
  self.stench_due = 0
  self.hold_angle = 0
  self.hold_xscale = 1.0
  self.hold_yscale = 1.0
  self.flash = 0
  self.born = born
end

-- gml_Script____struct___60__mushroom_add__scr_life @ 0x1400c8630
function ___struct___60__mushroom_add__scr_life(self, other, angle, radius, speed, bob)
  self.angle = angle
  self.radius = radius
  self.speed = speed
  self.bob = bob
end

-- gml_Script_mushroom_add @ 0x1400c88e0
function mushroom_add(self, other, type, x, y)
  local born = gm.current_time
  local variant = gm.irandom(mushroom_type_variants(self, other, type) - 1)
  local m = gm.new(___struct___59__mushroom_add__scr_life, type, variant, x, y,
    repeller_max_charges(self, other), gm.array(), -1.0, gm.undefined, gm.array(), born)
  if type == 1.0 then
    local n = bal(self, other, "predator_fly_orbit_count")
    local i = 0
    while i < n do
      local flies = m.flies
      local a = gm.random(360.0)
      local r = gm.random_range(26.0, 46.0)
      local sp = gm.random_range(1.4, 2.6)
      local bob = gm.random(360.0)
      gm.array_push(flies, gm.new(___struct___60__mushroom_add__scr_life, a, r, sp, bob))
      i = i + 1
    end
    m.timer = predator_period(self, other)
  end
  if type == 3.0 then
    m.timer = tree_drop_period(self, other)
  end
  if type == 6.0 then
    m.timer = experience_period(self, other)
  end
  gm.array_push(global.mushrooms, m)
  mushroom_grow_sound(self, other, gm.undefined, gm.undefined, gm.undefined)
  tasks_stat_add(self, other, "built", gm.undefined)
  if type == 3.0 or type == 7.0 then
    tasks_stat_add(self, other, "tree_built", gm.undefined)
  end
  if type == 5.0 then
    tasks_stat_add(self, other, "stinky_built", gm.undefined)
  end
  if type == 1.0 then
    tasks_stat_add(self, other, "ach_predators", gm.undefined)
  end
  return gm.array_length(global.mushrooms) - 1
end

-- gml_Script_mushroom_hit_box @ 0x1400c9fe0
function mushroom_hit_box(self, other, m)
  local b = mushroom_hit_box_unmirrored(self, other, m)
  local face = gm.variable_struct_get(m, "face")
  if gm.is_undefined(face) then
    face = 1.0
  end
  if face < 0 then
    local x1 = b.x1
    b.x1 = 2.0 * m.x - b.x2
    b.x2 = 2.0 * m.x - x1
  end
  return b
end

-- gml_Script____struct___61__mushroom_hit_box_unmirrored__scr_life @ 0x1400ca7e0
function ___struct___61__mushroom_hit_box_unmirrored__scr_life(self, other, x1, y1, x2, y2)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script____struct___62__mushroom_hit_box_unmirrored__scr_life @ 0x1400caa90
function ___struct___62__mushroom_hit_box_unmirrored__scr_life(self, other, x1, y1, x2, y2)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_mushroom_hit_box_unmirrored @ 0x1400cad40
function mushroom_hit_box_unmirrored(self, other, m)
  local spr = mushroom_type_sprite(self, other, m.type)
  local top = m.y + gm.sprite_get_bbox_top(spr) - gm.sprite_get_yoffset(spr)
  local bot = top + (gm.sprite_get_bbox_bottom(spr) - gm.sprite_get_bbox_top(spr))
  if shroom_sheet_cells(self, other, spr) > 1.0 then
    local left = m.x - gm.sprite_get_xoffset(spr)
    return gm.new(___struct___61__mushroom_hit_box_unmirrored__scr_life, left, top,
      left + mushroom_cell_width(self, other, spr) - 1, bot)
  end
  local left = m.x + gm.sprite_get_bbox_left(spr) - gm.sprite_get_xoffset(spr)
  return gm.new(___struct___62__mushroom_hit_box_unmirrored__scr_life, left, top,
    left + (gm.sprite_get_bbox_right(spr) - gm.sprite_get_bbox_left(spr)), bot)
end

-- gml_Script____struct___63__mushrooms_step__scr_life @ 0x1400cbe50
function ___struct___63__mushrooms_step__scr_life(self, other, x, y, target_y)
  self.x = x
  self.y = y
  self.target_y = target_y
  self.speed = 0
end

-- gml_Script_mushrooms_step @ 0x1400cc120
function mushrooms_step(self, other)
  global.stinky_live_last = global.stinky_live_count
  global.stinky_live_count = 0
  local keep = gm.array()
  local sv = squirrel_view(self, other)
  local finished = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.flash > 0 then
      m.flash = m.flash - 1.0
    end
    if m.type == 2.0 then
      local regen = repeller_autoregen_frames(self, other)
      if gm.bool(m.active) or regen < 0 then
        m.regen_timer = 0
      else
        local rt = gm.variable_struct_get(m, "regen_timer")
        if gm.is_undefined(rt) then
          rt = 0
        end
        m.regen_timer = rt + 1
        if m.regen_timer >= regen then
          m.regen_timer = 0
          m.active = true
          m.charges = repeller_max_charges(self, other)
        end
      end
    end
    local t = m.type
    if t == 1.0 then
      local j = 0
      while j < gm.array_length(m.flies) do
        local f = m.flies[j]
        f.angle = f.angle + f.speed
        f.bob = f.bob + 3
        j = j + 1
      end
      if m.carried_by == -1.0 and not gm.bool(m.stolen) then
        m.timer = m.timer - 1.0
        if m.timer <= 0 then
          m.timer = predator_period(self, other)
          food_add(self, other, predator_yield(self, other))
          tasks_stat_add(self, other, "food", predator_yield(self, other))
        end
      else
        m.timer = predator_period(self, other)
      end
    elseif t == 5.0 then
      stinky_step(self, other, m, sv)
    elseif t == 7.0 then
      if not gm.bool(m.stolen) and not gm.is_undefined(m.host) then
        local consumed = gm.variable_struct_get(m.host, "consumed")
        if gm.is_undefined(consumed) then
          consumed = false
        end
        if not gm.bool(consumed) then
          local h = m.host
          local c = gm.variable_struct_get(h, "consume")
          if gm.is_undefined(c) then
            c = 0
          end
          h.consume = c + 1.0 / gm.max(1.0, consume_frames(self, other, h))
          if h.consume >= 1.0 then
            h.consumed = true
            gm.array_push(finished, h)
          end
        end
      end
    elseif t == 6.0 then
      if not gm.bool(m.stolen) and m.carried_by == -1.0 then
        m.timer = m.timer - 1.0
        if m.timer <= 0 then
          m.timer = experience_period(self, other)
          global.upgrade_points = global.upgrade_points + 1
          if gm.bool(weather_is_raining(self, other)) then
            tasks_stat_add(self, other, "rain_points", gm.undefined)
          end
          m.flash = vis(self, other, "experience_flash_frames")
        end
      end
    elseif t == 3.0 then
      if not gm.bool(m.stolen) then
        m.timer = m.timer - 1.0
        if not (m.timer > 0) then
          m.timer = tree_drop_period(self, other)
          gm.array_push(global.drops, gm.new(___struct___63__mushrooms_step__scr_life, m.x + 18, m.y - 150, m.y - 4))
        end
      end
    end
    gm.array_push(keep, m)
    i = i + 1
  end
  global.mushrooms = keep
  local j = 0
  while j < gm.array_length(finished) do
    consume_finish(self, other, finished[j])
    j = j + 1
  end
end

-- gml_Script_drops_step @ 0x1400cf810
function drops_step(self, other)
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(global.drops) do
    local d = global.drops[i]
    d.speed = gm.min(d.speed + 0.22, 7.0)
    d.y = d.y + d.speed
    if d.y >= d.target_y then
      global.water_free = global.water_free + tree_drop_water(self, other)
      tasks_stat_add(self, other, "ach_tree_water", tree_drop_water(self, other))
    else
      gm.array_push(keep, d)
    end
    i = i + 1
  end
  global.drops = keep
end

-- gml_Script____struct___64__hedgehog_world_span__scr_life @ 0x1400d0190
function ___struct___64__hedgehog_world_span__scr_life(self, other, left, right)
  self.left = left
  self.right = right
end

-- gml_Script_hedgehog_world_span @ 0x1400d0360
function hedgehog_world_span(self, other)
  local vw = global.view_w
  local vh = global.view_h
  local b = camera_bounds(self, other, vw, vh)
  return gm.new(___struct___64__hedgehog_world_span__scr_life, b.x1, b.x2 + vw)
end

-- gml_Script_hedgehog_border_margin @ 0x1400d08d0
function hedgehog_border_margin(self, other)
  return hedgehog_frame_w(self, other) + 60
end

-- gml_Script____struct___65__animal_spawn_hedgehog__scr_life @ 0x1400d0a30
function ___struct___65__animal_spawn_hedgehog__scr_life(self, other, x, y, dir, speed, frame_timer, carrying)
  self.x = x
  self.y = y
  self.dir = dir
  self.speed = speed
  self.frame = 0
  self.frame_timer = frame_timer
  self.carrying = carrying
end

-- gml_Script_animal_spawn_hedgehog @ 0x1400d0e20
function animal_spawn_hedgehog(self, other)
  local span = hedgehog_world_span(self, other)
  local margin = hedgehog_border_margin(self, other)
  local from_left = gm.choose(true, false)
  local x
  if gm.bool(from_left) then
    x = span.left - margin
  else
    x = span.right + margin
  end
  local y = wld(self, other, "surface_height_px")
  local dir
  if gm.bool(from_left) then
    dir = 1.0
  else
    dir = -1.0
  end
  local speed = bal(self, other, "hedgehog_speed")
  local fi = vis(self, other, "hedgehog_frame_interval")
  gm.array_push(global.animals, gm.new(___struct___65__animal_spawn_hedgehog__scr_life, x, y, dir, speed, fi, -1.0))
end

-- gml_Script____struct___66__hedgehog_summon__scr_life @ 0x1400d18f0
function ___struct___66__hedgehog_summon__scr_life(self, other, x, y, dir, speed, frame_timer, carrying)
  self.x = x
  self.y = y
  self.dir = dir
  self.speed = speed
  self.frame = 0
  self.frame_timer = frame_timer
  self.carrying = carrying
end

-- gml_Script_hedgehog_summon @ 0x1400d1ce0
function hedgehog_summon(self, other)
  local sv = squirrel_view(self, other)
  local cx = sv.x + sv.w * 0.5
  local best = gm.undefined
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if gm.bool(hedgehog_takes(self, other, m)) and m.carried_by == -1.0 and not gm.bool(m.stolen) then
      if gm.is_undefined(best) or gm.abs(m.x - cx) < gm.abs(best.x - cx) then
        best = m
      end
    end
    i = i + 1
  end
  local dir
  if not gm.is_undefined(best) then
    if best.x >= cx then
      dir = 1.0
    else
      dir = -1.0
    end
  else
    dir = gm.choose(-1.0, 1.0)
  end
  local margin = hedgehog_frame_w(self, other) + 40
  local x
  if not (dir > 0) then
    x = sv.x + sv.w + margin
  else
    x = sv.x - margin
  end
  local y = wld(self, other, "surface_height_px")
  local speed = bal(self, other, "hedgehog_speed")
  local fi = vis(self, other, "hedgehog_frame_interval")
  gm.array_push(global.animals, gm.new(___struct___66__hedgehog_summon__scr_life, x, y, dir, speed, fi, -1.0))
  if not gm.is_undefined(best) then
    return "hedgehog"
  end
  return "hedgehog (nothing to carry)"
end

-- gml_Script_hedgehog_takes @ 0x1400d36c0
function hedgehog_takes(self, other, m)
  return m.type ~= 3.0 and m.type ~= 7.0 and m.type ~= 4.0
end

-- gml_Script_animals_step @ 0x1400d3900
function animals_step(self, other)
  global.hedgehog_timer = global.hedgehog_timer - animal_timer_step(self, other)
  if global.hedgehog_timer <= 0 then
    global.hedgehog_timer = animal_interval(self, other, "hedgehog_spawn_interval")
    animal_spawn_hedgehog(self, other)
  end
  local span = hedgehog_world_span(self, other)
  local margin = hedgehog_border_margin(self, other)
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(global.animals) do
    local a = global.animals[i]
    a.x = a.x + a.dir * a.speed
    a.frame_timer = a.frame_timer - 1.0
    if a.frame_timer <= 0 then
      a.frame_timer = vis(self, other, "hedgehog_frame_interval")
      a.frame = gm.mod(a.frame + 1, 6)
    end
    local j = 0
    while j < gm.array_length(global.mushrooms) do
      local m = global.mushrooms[j]
      if m.type ~= 3.0 and m.carried_by == -1.0 then
        if gm.abs(m.x - a.x) <= bal(self, other, "hedgehog_reach_px") then
          if not (m.type == 2.0 and gm.bool(m.active)) then
            if gm.bool(hedgehog_takes(self, other, m)) and a.carrying == -1.0
              and not gm.bool(m.crow_target) and not gm.bool(m.stolen) then
              m.carried_by = i
              a.carrying = j
            end
          end
        end
      end
      j = j + 1
    end
    if a.carrying ~= -1.0 and a.carrying < gm.array_length(global.mushrooms) then
      local c = global.mushrooms[a.carrying]
      local p = hedgehog_back_point(self, other, a.x, a.y, a.dir, a.frame)
      c.x = p[0]
      c.y = p[1]
      c.carry_angle = -a.dir * vis(self, other, "hedgehog_carry_angle")
    end
    local gone
    if a.dir < 0 and a.x < span.left - margin then
      gone = true
    elseif a.dir > 0 then
      gone = a.x > span.right + margin
    else
      gone = false
    end
    if gone then
      if a.carrying ~= -1.0 and a.carrying < gm.array_length(global.mushrooms) then
        gm.array_delete(global.mushrooms, a.carrying, 1.0)
        animals_reindex_after_removal(self, other, a.carrying)
      end
    else
      gm.array_push(keep, a)
    end
    i = i + 1
  end
  global.animals = keep
  animals_rebind(self, other)
end

-- gml_Script_animals_reindex_after_removal @ 0x1400d6120
function animals_reindex_after_removal(self, other, removed)
  local i = 0
  while i < gm.array_length(global.animals) do
    local a = global.animals[i]
    if a.carrying == removed then
      a.carrying = -1.0
    elseif a.carrying > removed then
      a.carrying = a.carrying - 1.0
    end
    i = i + 1
  end
end

-- gml_Script_animals_rebind @ 0x1400d6570
function animals_rebind(self, other)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    global.mushrooms[i].carried_by = -1.0
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.animals) do
    local a = global.animals[i]
    if a.carrying ~= -1.0 and a.carrying < gm.array_length(global.mushrooms) then
      global.mushrooms[a.carrying].carried_by = i
    else
      a.carrying = -1.0
    end
    i = i + 1
  end
end

-- gml_Script_boar_frame_w @ 0x1400d6c80
function boar_frame_w(self, other)
  return gm.sprite_get_width(boar) / 6.0 * vis(self, other, "boar_scale")
end

-- gml_Script_boar_hits_mushroom @ 0x1400d6f70
function boar_hits_mushroom(self, other, m)
  return m.type ~= 3.0 and m.type ~= 7.0 and m.carried_by == -1.0
    and not gm.bool(m.stolen) and not gm.bool(m.crow_target)
end

-- gml_Script_boar_any_target @ 0x1400d7290
function boar_any_target(self, other)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    if gm.bool(boar_hits_mushroom(self, other, global.mushrooms[i])) then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script_boar_pick_targets @ 0x1400d75c0
function boar_pick_targets(self, other, x, dir)
  local list = gm.array()
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if gm.bool(boar_hits_mushroom(self, other, m)) and (m.x - x) * dir > 0 then
      if not (m.type == 2.0 and gm.bool(m.active)) then
        gm.array_push(list, global.mushrooms[i])
      end
    end
    i = i + 1
  end
  list = gm.array_shuffle(list)
  gm.array_resize(list, gm.min(gm.array_length(list), bal(self, other, "boar_hits")))
  return list
end
