local gm = require("gm")

-- gml_Script_stinky_draw @ 0x14013e820
function stinky_draw(self, other, m)
  local s = m.stench
  if gm.is_struct(s) and s.n ~= 0 then
    -- m[$ "stench_live"] ?? true (struct_get_from_hash with the hash of "stench_live")
    local live = m.stench_live
    if gm.is_undefined(live) then
      live = true
    end
    if gm.bool(live) then
      stinky_art_init(self, other)
      local n = s.n
      local px = s.px
      local py = s.py
      local pix = s.pix
      local colour = s.colour
      local alpha = s.alpha
      local gr = (1.0 + vis(self, other, "stinky_glow_radius_px")) / 8.0
      local ga = vis(self, other, "stinky_glow_alpha") * 2
      local pad = 8.0 * gr * 1.2 + 4
      local glow = global.stinky_glow
      local pixel = global.stinky_pixel
      local cam = gm.view_camera[0]
      local l = ((cam == -1) and 0 or gm.camera_get_view_x(cam)) - pad
      local t = ((cam == -1) and 0 or gm.camera_get_view_y(cam)) - pad
      local r = l + ((cam == -1) and vis(self, other, "screen_width_px") or gm.camera_get_view_width(cam)) + pad * 2
      local b = t + ((cam == -1) and vis(self, other, "screen_height_px") or gm.camera_get_view_height(cam)) + pad * 2
      gm.gpu_set_blendmode(gm.bm_add)
      local i = 0
      while i < n do
        local x = px[i]
        local y = py[i]
        if not (x < l or x > r or y < t or y > b) then
          local p = pix[i]
          local sc = gr * (0.6 + 0.2 * p)
          gm.draw_sprite_ext(glow, 0, x + p * 0.5, y + p * 0.5, sc, sc, 0, colour[i], gm.min(1.0, alpha[i] * ga))
        end
        i = i + 1
      end
      gm.gpu_set_blendmode(gm.bm_normal)
      i = 0
      while i < n do
        local x = px[i]
        local y = py[i]
        if not (x < l or x > r or y < t or y > b) then
          local p = pix[i]
          gm.draw_sprite_ext(pixel, 0, x, y, p, p, 0, colour[i], alpha[i])
        end
        i = i + 1
      end
    end
  end
end

-- gml_Script_stinky_beam_hits @ 0x140141690
function stinky_beam_hits(self, other, px, py, r)
  local bh = vis(self, other, "stinky_beam_height_px")
  local sh = stinky_shape(self, other)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 5.0 and not gm.bool(m.stolen) then
      local d = m.y - stinky_cap_height(self, other, m) - py
      if not (d < -r or d > bh + r) then
        if gm.abs(px - (m.x + stinky_wind_shift(self, other, gm.max(0, d)))) <= stinky_funnel_radius(self, other, sh, d) + r then
          return true
        end
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script_stinky_squirrel_reach @ 0x140142370
function stinky_squirrel_reach(self, other)
  local top = stinky_shape(self, other).top
  local a0 = stinky_top_at(self, other, 0)
  local span = stinky_top_at(self, other, 4.0) - a0
  local k = (top - stinky_top_at(self, other, 0)) / gm.max(1.0, span)
  return gm.lerp(bal(self, other, "stinky_squirrel_reach_px"), bal(self, other, "stinky_squirrel_reach_max_px"), gm.clamp(k, 0, 1.0))
end

-- gml_Script_stinky_guards @ 0x140142ae0
function stinky_guards(self, other, sq)
  local reach = stinky_squirrel_reach(self, other)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m ~= sq and m.type == 5.0 and not gm.bool(m.stolen) then
      if gm.abs(m.x - sq.x) <= reach then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___90__life_near_zone__scr_life @ 0x140143100
function ___struct___90__life_near_zone__scr_life(self, other, x1, x2, w)
  self.x1 = x1
  self.x2 = x2
  self.w = w
end

-- gml_Script_life_near_zone @ 0x140143330
function life_near_zone(self, other)
  local v = squirrel_view(self, other)
  local m = v.w * bal(self, other, "life_near_screens")
  return gm.new(___struct___90__life_near_zone__scr_life, v.x - m, v.x + v.w + m, v.w)
end

-- gml_Script____struct___91__leaf_spawn__scr_life @ 0x1401439e0
function ___struct___91__leaf_spawn__scr_life(self, other, x, y, phase, mirror, turn, turn_delay, drift, life)
  self.x = x
  self.y = y
  self.phase = phase
  self.mirror = mirror
  self.turn = turn
  self.turn_timer = 0
  self.turned = false
  self.turn_delay = turn_delay
  self.angle = 0
  self.hold_angle = 0
  self.drift = drift
  self.landed = false
  self.absorbing = false
  self.timer = 0
  self.life = life
end

-- gml_Script_leaf_spawn @ 0x140144050
function leaf_spawn(self, other, in_view, x, y)
  if gm.is_undefined(in_view) then
    in_view = false
  end
  if gm.is_undefined(x) then
    x = gm.undefined
  end
  if gm.is_undefined(y) then
    y = -40.0
  end
  local zone = life_near_zone(self, other)
  local v = squirrel_view(self, other)
  local lx = x
  if gm.is_undefined(lx) then
    if not gm.bool(in_view) then
      lx = gm.random_range(zone.x1, zone.x2)
    else
      lx = v.x + gm.random(v.w)
    end
  end
  local phase = gm.random(360.0)
  local mirror = gm.irandom(1)
  local dmax = vis(self, other, "leaf_turn_delay_max")
  local dmin = vis(self, other, "leaf_turn_delay_min")
  local turn_delay = gm.irandom_range(dmin, dmax)
  local drift = global.wind_dir * global.wind_speed * vis(self, other, "leaf_wind_push")
  local life = vis(self, other, "leaf_idle_life")
  gm.array_push(global.leaves, gm.new(___struct___91__leaf_spawn__scr_life, lx, y, phase, mirror, -1.0, turn_delay, drift, life))
end

-- gml_Script_leaf_gust_spawn @ 0x140145170
function leaf_gust_spawn(self, other)
  local zone = life_near_zone(self, other)
  local cx = gm.random_range(zone.x1, zone.x2)
  local gmax = vis(self, other, "leaf_gust_max")
  local gmin = vis(self, other, "leaf_gust_min")
  local n = gm.irandom_range(gmin, gmax)
  local spread = vis(self, other, "leaf_gust_spread_px")
  local i = 0
  while i < n do
    local x = cx + gm.random_range(-spread, spread)
    local y = -40.0 - gm.random(vis(self, other, "leaf_gust_height_px"))
    leaf_spawn(self, other, false, x, y)
    i = i + 1
  end
end

-- gml_Script____struct___92__leaves_step__scr_life @ 0x140145990
function ___struct___92__leaves_step__scr_life(self, other, cells, timer, total)
  self.cells = cells
  self.timer = timer
  self.total = total
end

-- gml_Script_leaves_step @ 0x140145bd0
function leaves_step(self, other)
  global.leaf_timer = global.leaf_timer - 1.0
  if global.leaf_timer <= 0 then
    local zone = life_near_zone(self, other)
    global.leaf_timer = bal(self, other, "leaf_spawn_interval") * zone.w / gm.max(1.0, zone.x2 - zone.x1)
    leaf_spawn(self, other, gm.undefined, gm.undefined, gm.undefined)
  end
  local surf = wld(self, other, "surface_height_px")
  local ctrl
  if gm.instance_exists(obj_mycelium_controller) then
    ctrl = gm.instance_find(obj_mycelium_controller, 0)
  else
    ctrl = -4.0
  end
  local kept = gm.array()
  local i = 0
  while i < gm.array_length(global.leaves) do
    local leaf = global.leaves[i]
    if not gm.bool(leaf.landed) then
      if false and leaf.turn < 0 and not gm.bool(leaf.turned) then
        leaf.turn_delay = leaf.turn_delay - 1.0
        if leaf.turn_delay <= 0 then
          leaf_start_turn(self, other, leaf)
        end
      end
      if leaf.turn >= 0 then
        leaf.turn_timer = leaf.turn_timer - 1.0
        if leaf.turn_timer <= 0 then
          leaf.turn_timer = vis(self, other, "leaf_turn_frame_interval")
          leaf.turn = leaf.turn + 1
          if leaf.turn >= 14 then
            if gm.bool(leaf_fg_art_ready(self, other)) then
              leaf.x = leaf.x + global.leaf_fg_art.carry_x * ((leaf.mirror > 0) and -1.0 or 1.0)
              leaf.y = leaf.y + global.leaf_fg_art.carry_y
            end
            leaf.turn = -1
            leaf.turned = true
          end
        end
      end
      local push = global.wind_dir * global.wind_speed * vis(self, other, "leaf_wind_push")
      leaf.drift = leaf.drift + (push - leaf.drift) * gm.clamp(vis(self, other, "leaf_wind_ease"), 0, 1.0)
      leaf.phase = leaf.phase + vis(self, other, "leaf_sway_speed")
      local spin = gm.sin(gm.degtorad(leaf.phase)) * vis(self, other, "leaf_spin_degrees")
      local target
      if leaf.turn >= 0 then
        target = leaf.hold_angle
      else
        target = spin
      end
      leaf.angle = leaf.angle + (target - leaf.angle) * gm.clamp(vis(self, other, "leaf_spin_ease"), 0, 1.0)
      leaf.x = leaf.x + (gm.lengthdir_x(vis(self, other, "leaf_sway"), leaf.phase) + leaf.drift)
      leaf.y = leaf.y + vis(self, other, "leaf_fall_speed")
      local bottom
      if gm.bool(leaf_fg_art_ready(self, other)) then
        bottom = leaf.y + leaf_fg_drop(self, other, leaf)
      else
        bottom = leaf.y
      end
      if bottom >= surf then
        leaf.y = leaf.y - (bottom - surf)
        leaf.landed = true
        if gm.bool(upg_is_owned(self, other, "unlock_absorption")) and ctrl ~= -4 then
          local hit = ctrl.myc_get_mycelium_hit(self, other, leaf.x, surf + wld(self, other, "reference_tile_size"), bal(self, other, "leaf_absorb_radius"))
          if gm.bool(hit.hit) then
            leaf.absorbing = true
            local gf = gm.max(1.0, vis(self, other, "leaf_tendril_grow_frames"))
            leaf.timer = gf + gm.max(bal(self, other, "leaf_min_hang_frames"), leaf_decay_time(self, other) - gf)
            leaf.total = leaf.timer
            leaf.grow_frames = gf
            leaf.tendril = leaf_tendril_make(self, other, leaf, hit.x, hit.y, ctrl)
            leaf.sound = hyphae_sound_begin(self, other, gm.point_distance(hit.x, hit.y, leaf.x, surf), gf, leaf.x, surf)
          end
        end
      end
      gm.array_push(kept, leaf)
    elseif not gm.bool(leaf.absorbing) then
      leaf.life = leaf.life - 1
      if leaf.life > 0 then
        gm.array_push(kept, leaf)
      end
    else
      leaf.timer = leaf.timer - 1
      if gm.variable_struct_exists(leaf, "sound") then
        hyphae_sound_step(self, other, leaf.sound, leaf.total - leaf.timer)
      end
      if leaf.timer <= 0 then
        if gm.variable_struct_exists(leaf, "sound") then
          hyphae_sound_stop(self, other, leaf.sound)
        end
        food_add(self, other, leaf_food_value(self, other))
        if gm.variable_struct_exists(leaf, "tendril") and gm.is_array(leaf.tendril) then
          local rf = gm.max(1.0, vis(self, other, "leaf_tendril_retract_frames"))
          gm.array_push(global.leaf_tendrils_out, gm.new(___struct___92__leaves_step__scr_life, leaf.tendril, rf, rf))
        end
      else
        gm.array_push(kept, leaf)
      end
    end
    i = i + 1
  end
  global.leaves = kept
  local out = gm.array()
  i = 0
  while i < gm.array_length(global.leaf_tendrils_out) do
    local t = global.leaf_tendrils_out[i]
    t.timer = t.timer - 1
    if t.timer > 0 then
      gm.array_push(out, t)
    end
    i = i + 1
  end
  global.leaf_tendrils_out = out
end

-- gml_Script____struct___93__leaf_tendril_make__scr_life @ 0x140149ad0
function ___struct___93__leaf_tendril_make__scr_life(self, other, x1, y1)
  self.x1 = x1
  self.y1 = y1
  self.x2 = 8.0
  self.y2 = 6.0
  self.mid_x = 0
  self.mid_y = 0
end

-- gml_Script____struct___94__leaf_tendril_make__scr_life @ 0x140149d70
function ___struct___94__leaf_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___95__leaf_tendril_make__scr_life @ 0x140149f70
function ___struct___95__leaf_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___96__leaf_tendril_make__scr_life @ 0x14014a170
function ___struct___96__leaf_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_leaf_tendril_make @ 0x14014a370
function leaf_tendril_make(self, other, leaf, hx, hy, ctrl)
  local surf = wld(self, other, "surface_height_px")
  local mir = leaf_fg_mirror(self, other, leaf)
  local box
  if not gm.bool(leaf_fg_art_ready(self, other)) then
    box = gm.new(___struct___93__leaf_tendril_make__scr_life, -8.0, -6.0)
  else
    box = leaf_fg_frame(self, other, leaf)
  end
  local lx1 = gm.min(box.x1 * mir, box.x2 * mir)
  local lx2 = gm.max(box.x1 * mir, box.x2 * mir)
  local cx = leaf.x + (lx1 + lx2) * 0.5
  local cy = leaf.y + (box.y1 + box.y2) * 0.5
  local hw = (lx2 - lx1) * 0.5 + 1
  local hh = (box.y2 - box.y1) * 0.5 + 1
  local drop
  if not gm.bool(leaf_fg_art_ready(self, other)) then
    drop = 0
  else
    drop = leaf_fg_drop(self, other, leaf)
  end
  local bottom = gm.floor(leaf.y + drop)
  local path = gm.array(gm.new(___struct___94__leaf_tendril_make__scr_life, hx, hy))
  local tx = cx
  local ty = gm.max(bottom, surf) + 2
  local d = gm.point_distance(hx, hy, tx, ty)
  local dir = gm.point_direction(hx, hy, tx, ty)
  local n = gm.max(3.0, gm.round(d / 14.0))
  local wob = 0
  local k = 1.0
  while k <= n do
    local t = k / n
    wob = wob * 0.55 + gm.random_range(-1.0, 1.0) * d * 0.06
    local off = wob * (1.0 - gm.abs(t - 0.5) * 2.0)
    gm.array_push(path, gm.new(___struct___95__leaf_tendril_make__scr_life,
      gm.lerp(hx, tx, t) + gm.lengthdir_x(off, dir + 90),
      gm.lerp(hy, ty, t) + gm.lengthdir_y(off, dir + 90)))
    k = k + 1
  end
  local turns = vis(self, other, "leaf_tendril_wrap_turns")
  local m = gm.max(8.0, gm.round(20.0 * turns))
  local side = gm.choose(-1.0, 1.0)
  local j = 0
  while j <= m do
    local t = j / m
    local a = 270.0 + side * 360 * turns * t
    local rr = gm.lerp(1.0, 0.45, t)
    gm.array_push(path, gm.new(___struct___96__leaf_tendril_make__scr_life,
      cx + gm.lengthdir_x(hw * rr, a),
      cy + gm.lengthdir_y(hh * rr, a)))
    j = j + 1
  end
  local th = vis(self, other, "strand_thickness")
  local cells = gm.array()
  gm.with(self, ctrl, function(self, other)
    cells = self.myc_rasterize_branch(self, other, path, th)
  end)
  return cells
end

-- gml_Script_leaf_tendril_cells_draw @ 0x14014e6b0
function leaf_tendril_cells_draw(self, other, cells, count, col)
  local verts = 0
  gm.draw_primitive_begin(gm.pr_trianglelist)
  local i = 0
  while i < count do
    local c = cells[i]
    local h = gm.div(c.w, 2)
    local x0 = c.x - h
    local y0 = c.y - h
    local x1 = x0 + c.w
    local y1 = y0 + c.w
    gm.draw_vertex_color(x0, y0, col, 1.0)
    gm.draw_vertex_color(x1, y0, col, 1.0)
    gm.draw_vertex_color(x0, y1, col, 1.0)
    gm.draw_vertex_color(x1, y0, col, 1.0)
    gm.draw_vertex_color(x1, y1, col, 1.0)
    gm.draw_vertex_color(x0, y1, col, 1.0)
    verts = verts + 6
    if verts >= 600.0 then
      gm.draw_primitive_end()
      gm.draw_primitive_begin(gm.pr_trianglelist)
      verts = 0
    end
    i = i + 1
  end
  gm.draw_primitive_end()
end

-- gml_Script_leaf_tendrils_draw @ 0x14014f6c0
function leaf_tendrils_draw(self, other, col)
  local i = 0
  while i < gm.array_length(global.leaves) do
    local leaf = global.leaves[i]
    if gm.bool(leaf.absorbing) and gm.variable_struct_exists(leaf, "tendril") and gm.is_array(leaf.tendril) then
      local n = gm.array_length(leaf.tendril)
      local gf
      if gm.variable_struct_exists(leaf, "grow_frames") then
        gf = leaf.grow_frames
      else
        gf = gm.max(1.0, vis(self, other, "leaf_tendril_grow_frames"))
      end
      local p = (leaf.total - leaf.timer) / gf
      leaf_tendril_cells_draw(self, other, leaf.tendril, gm.min(n, gm.ceil(gm.clamp(p, 0, 1.0) * n)), col)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.worms) do
    local w = global.worms[i]
    if gm.bool(w.caught) then
      local n = gm.array_length(w.tendril)
      local p = (w.total - w.timer) / gm.max(1.0, w.grow_frames)
      leaf_tendril_cells_draw(self, other, w.tendril, gm.min(n, gm.ceil(gm.clamp(p, 0, 1.0) * n)), col)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.leaf_tendrils_out) do
    local t = global.leaf_tendrils_out[i]
    local n = gm.array_length(t.cells)
    leaf_tendril_cells_draw(self, other, t.cells, gm.min(n, gm.ceil(t.timer / gm.max(1.0, t.total) * n)), col)
    i = i + 1
  end
end

-- gml_Script_worm_zone_top @ 0x1401516a0
function worm_zone_top(self, other)
  local d = bal(self, other, "worm_depth_min_px")
  return wld(self, other, "surface_height_px") + d
end

-- gml_Script_worm_zone_bottom @ 0x1401518f0
function worm_zone_bottom(self, other)
  local sh = vis(self, other, "screen_height_px")
  local d = bal(self, other, "worm_depth_screens") * sh
  return wld(self, other, "surface_height_px") + d
end

-- gml_Script_worm_cell_ok @ 0x140151c20
function worm_cell_ok(self, other, x, y, top)
  if gm.is_undefined(top) then
    top = worm_zone_top(self, other)
  end
  if y < top or y > worm_zone_bottom(self, other) then
    return false
  end
  local t = world_tile_at(self, other, x, y)
  if t == 0 then
    return true
  end
  return t == 1.0
end

-- gml_Script_worm_segment_px @ 0x140152120
function worm_segment_px(self, other)
  local m = gm.max(1.0, vis(self, other, "worm_segments") - 1)
  return vis(self, other, "worm_length_px") / m
end

-- gml_Script____struct___97__worm_add__scr_life @ 0x1401524b0
function ___struct___97__worm_add__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___98__worm_add__scr_life @ 0x1401526b0
function ___struct___98__worm_add__scr_life(self, other, body, dir, side, phase, check)
  self.body = body
  self.dir = dir
  self.turn = 0
  self.side = side
  self.phase = phase
  self.check = check
  self.trail_walk = 0
  self.caught = false
  self.timer = 0
  self.total = 1.0
  self.struggle = 0
end

-- gml_Script_worm_add @ 0x140152b20
function worm_add(self, other, x, y, dir)
  local body = gm.array()
  local n = gm.max(2.0, vis(self, other, "worm_segments"))
  local seg = worm_segment_px(self, other)
  local i = 0
  while i < n do
    gm.array_push(body, gm.new(___struct___97__worm_add__scr_life,
      x - gm.lengthdir_x(seg * i, dir),
      y - gm.lengthdir_y(seg * i, dir)))
    i = i + 1
  end
  local side = gm.choose(-1.0, 1.0)
  local phase = gm.random(360.0)
  local check = gm.irandom(bal(self, other, "worm_catch_check_frames"))
  local w = gm.new(___struct___98__worm_add__scr_life, body, dir, side, phase, check)
  gm.array_push(global.worms, w)
  return w
end

-- gml_Script_worm_keep_bottom @ 0x140153aa0
function worm_keep_bottom(self, other, vy, vh)
  local zb = worm_zone_bottom(self, other)
  return gm.min(zb, vy + vh * (1.0 + bal(self, other, "worm_spawn_below_screens")))
end

-- gml_Script_worm_spawn @ 0x140153e50
function worm_spawn(self, other, in_view)
  local cam = gm.view_camera[0]
  local vx = (cam == -1) and 0 or gm.camera_get_view_x(cam)
  local vy = (cam == -1) and 0 or gm.camera_get_view_y(cam)
  local vw = (cam == -1) and vis(self, other, "screen_width_px") or gm.camera_get_view_width(cam)
  local vh = (cam == -1) and vis(self, other, "screen_height_px") or gm.camera_get_view_height(cam)
  local top = worm_zone_top(self, other) + 8
  local bot = worm_keep_bottom(self, other, vy, vh) - 8
  local margin = vis(self, other, "worm_length_px") + 16
  if not (bot > top or bot ~= bot or top ~= top) then
    return gm.undefined
  end
  local k = 0
  while k < 12.0 do
    local zone = life_near_zone(self, other)
    local x
    if not gm.bool(in_view) then
      x = gm.random_range(zone.x1, zone.x2)
    else
      x = vx + gm.random(vw)
    end
    local y
    if not gm.bool(in_view) then
      y = gm.random_range(top, bot)
    else
      y = gm.clamp(vy + gm.random(vh), top, bot)
    end
    local dir = gm.random(360.0)
    local skip = false
    if not gm.bool(in_view) then
      if x > vx - margin and x < vx + vw + margin and y > vy - margin and y < vy + vh + margin then
        skip = true
      end
    end
    if not skip then
      local tx = x - gm.lengthdir_x(vis(self, other, "worm_length_px"), dir)
      local ty = y - gm.lengthdir_y(vis(self, other, "worm_length_px"), dir)
      if gm.bool(worm_cell_ok(self, other, x, y, gm.undefined)) and gm.bool(worm_cell_ok(self, other, tx, ty, gm.undefined)) then
        return worm_add(self, other, x, y, dir)
      end
    end
    k = k + 1
  end
  return gm.undefined
end

-- gml_Script_worm_absorb_frames @ 0x140155d40
function worm_absorb_frames(self, other)
  local gf = gm.max(1.0, vis(self, other, "leaf_tendril_grow_frames"))
  local sp = 1.0 + upg_bonus(self, other, "absorption_speed")
  return gm.max(gf + 60, bal(self, other, "worm_absorb_time") / sp)
end

-- gml_Script____struct___99__worm_move__scr_life @ 0x1401562d0
function ___struct___99__worm_move__scr_life(self, other, x, y, life)
  self.x = x
  self.y = y
  self.life = life
end

local worm_move_offsets = nil

-- gml_Script_worm_move @ 0x140156520
function worm_move(self, other, w)
  if worm_move_offsets == nil then
    worm_move_offsets = gm.array(30.0, -30.0, 60.0, -60.0, 90.0, -90.0, 135.0, -135.0, 180.0)
  end
  local offs = worm_move_offsets
  local body = w.body
  local head = body[0]
  local rain = weather_is_raining(self, other)
  local top = worm_zone_top(self, other)
  local floor_y
  if not gm.bool(rain) then
    floor_y = gm.min(top, head.y - 2)
  else
    floor_y = wld(self, other, "surface_height_px") + 4
  end
  w.phase = w.phase + vis(self, other, "worm_wave_speed")
  w.turn = w.turn * 0.95 + gm.random_range(-0.35, 0.35)
  w.dir = w.dir + w.turn
  if not gm.bool(rain) then
    if head.y < top then
      w.dir = w.dir + gm.angle_difference(270.0, w.dir) * 0.03
    end
  else
    w.dir = w.dir + gm.angle_difference(90.0, w.dir) * 0.04
  end
  local look = 6.0
  if not gm.bool(worm_cell_ok(self, other, head.x + gm.lengthdir_x(look, w.dir), head.y + gm.lengthdir_y(look, w.dir), floor_y)) then
    local found = false
    local i = 0
    while i < gm.array_length(offs) do
      local nd = w.dir + offs[i] * w.side
      if gm.bool(worm_cell_ok(self, other, head.x + gm.lengthdir_x(look, nd), head.y + gm.lengthdir_y(look, nd), floor_y)) then
        w.dir = nd
        w.turn = 0
        found = true
        break
      end
      i = i + 1
    end
    if not found then
      w.side = -w.side
      return
    end
  end
  local spd = bal(self, other, "worm_speed") * (0.3 + 0.7 * gm.max(0, gm.sin(w.phase)))
  if world_tile_at(self, other, head.x, head.y) == 1.0 then
    spd = spd * bal(self, other, "worm_hardsoil_speed")
  end
  if gm.bool(rain) then
    spd = spd * bal(self, other, "worm_rain_speed")
  end
  head.x = head.x + gm.lengthdir_x(spd, w.dir)
  head.y = head.y + gm.lengthdir_y(spd, w.dir)
  w.trail_walk = w.trail_walk + spd
  if w.trail_walk >= vis(self, other, "worm_trail_spacing_px") and gm.array_length(global.worm_trail) < vis(self, other, "worm_trail_max") then
    w.trail_walk = 0
    local tx = gm.floor(head.x)
    local ty = gm.floor(head.y)
    local r = gm.div(gm.max(1.0, vis(self, other, "worm_thickness_px") - 1), 2) + 1
    if not gm.bool(world_water_at(self, other, tx - r, ty - r))
      and not gm.bool(world_water_at(self, other, tx + r, ty - r))
      and not gm.bool(world_water_at(self, other, tx - r, ty + r))
      and not gm.bool(world_water_at(self, other, tx + r, ty + r)) then
      gm.array_push(global.worm_trail, gm.new(___struct___99__worm_move__scr_life, tx, ty, vis(self, other, "worm_trail_frames")))
    end
  end
  local seg = worm_segment_px(self, other)
  local st = vis(self, other, "worm_stretch")
  local i = 1.0
  while i < gm.array_length(body) do
    local a = body[i - 1]
    local b = body[i]
    local d = seg * (1.0 + st * gm.sin(w.phase - i * 0.5))
    local ang = gm.point_direction(a.x, a.y, b.x, b.y)
    b.x = a.x + gm.lengthdir_x(d, ang)
    b.y = a.y + gm.lengthdir_y(d, ang)
    i = i + 1
  end
end

-- gml_Script____struct___100__worm_body_at__scr_life @ 0x14015b040
function ___struct___100__worm_body_at__scr_life(self, other, x, y, dir)
  self.x = x
  self.y = y
  self.dir = dir
end

-- gml_Script_worm_body_at @ 0x14015b2a0
function worm_body_at(self, other, w, s)
  local body = w.body
  local last = gm.array_length(body) - 1
  local hi = gm.max(0, last - 1)
  local i = gm.clamp(gm.floor(s), 0, hi)
  local f = gm.clamp(s - i, 0, 1.0)
  local a = body[i]
  local b = body[gm.min(i + 1, last)]
  local px = gm.lerp(a.x, b.x, f)
  local py = gm.lerp(a.y, b.y, f)
  local pd = gm.point_direction(a.x, a.y, b.x, b.y)
  return gm.new(___struct___100__worm_body_at__scr_life, px, py, pd)
end

-- gml_Script____struct___101__worm_tendril_make__scr_life @ 0x14015c0d0
function ___struct___101__worm_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___102__worm_tendril_make__scr_life @ 0x14015c2d0
function ___struct___102__worm_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___103__worm_tendril_make__scr_life @ 0x14015c4d0
function ___struct___103__worm_tendril_make__scr_life(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_worm_tendril_make @ 0x14015c6d0
function worm_tendril_make(self, other, w, hx, hy, ctrl)
  local body = w.body
  local last = gm.array_length(body) - 1
  local best = 0
  local bd = gm.infinity
  local i = 0
  while i <= last do
    local d = gm.point_distance(hx, hy, body[i].x, body[i].y)
    if d < bd then
      bd = d
      best = i
    end
    i = i + 1
  end
  local path = gm.array(gm.new(___struct___101__worm_tendril_make__scr_life, hx, hy))
  local bx = body[best].x
  local by = body[best].y
  local dir = gm.point_direction(hx, hy, bx, by)
  local n = gm.max(3.0, gm.round(bd / 10.0))
  local wob = 0
  local k = 1.0
  while k <= n do
    local t = k / n
    wob = wob * 0.55 + gm.random_range(-1.0, 1.0) * bd * 0.06
    local off = wob * (1.0 - gm.abs(t - 0.5) * 2.0)
    gm.array_push(path, gm.new(___struct___102__worm_tendril_make__scr_life,
      gm.lerp(hx, bx, t) + gm.lengthdir_x(off, dir + 90),
      gm.lerp(hy, by, t) + gm.lengthdir_y(off, dir + 90)))
    k = k + 1
  end
  local npath = gm.array_length(path)
  local fwd = not (best > last * 0.5 or best ~= best)
  local rem
  if not fwd then
    rem = last - best
  else
    rem = best
  end
  local span = rem + last
  local steps = gm.max(16.0, gm.round(span * worm_segment_px(self, other)))
  local th = vis(self, other, "worm_thickness_px") * 0.5 + 1
  local wraps = vis(self, other, "worm_tendril_wraps")
  local j = 1.0
  while j <= steps do
    local s = j / steps * span
    local idx
    if fwd then
      if s > rem then
        idx = s - rem
      else
        idx = best - s
      end
    else
      if s > rem then
        idx = last - (s - rem)
      else
        idx = best + s
      end
    end
    local p = worm_body_at(self, other, w, idx)
    local r = gm.sin(s / span * 3.141592653589793 * 2 * wraps) * th
    gm.array_push(path, gm.new(___struct___103__worm_tendril_make__scr_life,
      p.x + gm.lengthdir_x(r, p.dir + 90),
      p.y + gm.lengthdir_y(r, p.dir + 90)))
    j = j + 1
  end
  local total = 0
  local acc = 0
  local q = 1.0
  while q < gm.array_length(path) do
    local seg = gm.point_distance(path[q - 1].x, path[q - 1].y, path[q].x, path[q].y)
    acc = acc + seg
    if q < npath then
      total = total + seg
    end
    q = q + 1
  end
  w.touch_frames = gm.round(gm.max(1.0, vis(self, other, "leaf_tendril_grow_frames")) * total / gm.max(1.0, acc))
  local cells = gm.array()
  gm.with(self, ctrl, function(self, other)
    cells = self.myc_rasterize_branch(self, other, path, vis(self, other, "strand_thickness"))
  end)
  return cells
end

-- gml_Script_worm_catch @ 0x140160e40
function worm_catch(self, other, w, src, ctrl)
  local gf = gm.max(1.0, vis(self, other, "leaf_tendril_grow_frames"))
  local body = w.body
  local n = gm.array_length(body)
  local mid = body[gm.div(n, 2)]
  w.caught = true
  w.timer = worm_absorb_frames(self, other)
  w.total = w.timer
  w.grow_frames = gf
  w.struggle = vis(self, other, "worm_struggle_frames")
  w.tendril = worm_tendril_make(self, other, w, src.x, src.y, ctrl)
  local d = gm.point_distance(src.x, src.y, mid.x, mid.y)
  w.sound = hyphae_sound_begin(self, other, d, gf, mid.x, mid.y)
end

-- gml_Script____struct___104__worms_step__scr_life @ 0x140161cf0
function ___struct___104__worms_step__scr_life(self, other, cells, timer, total)
  self.cells = cells
  self.timer = timer
  self.total = total
end

-- gml_Script_worms_step @ 0x140161f30
function worms_step(self, other)
  global.worm_timer = global.worm_timer - 1.0
  if not (global.worm_timer > 0) then
    global.worm_timer = bal(self, other, "worm_spawn_interval")
    if gm.array_length(global.worms) < bal(self, other, "worm_max_count") then
      worm_spawn(self, other, false)
    end
  end
  local cam = gm.view_camera[0]
  local vw = (cam == -1) and vis(self, other, "screen_width_px") or gm.camera_get_view_width(cam)
  local vh = (cam == -1) and vis(self, other, "screen_height_px") or gm.camera_get_view_height(cam)
  local vy = (cam == -1) and 0 or gm.camera_get_view_y(cam)
  local keep = worm_keep_bottom(self, other, vy, vh) + vh * 0.5
  local zone = life_near_zone(self, other)
  local ctrl
  if not gm.instance_exists(obj_mycelium_controller) then
    ctrl = -4.0
  else
    ctrl = gm.instance_find(obj_mycelium_controller, 0)
  end
  local can_catch = false
  if gm.bool(upg_is_owned(self, other, "unlock_protein")) then
    can_catch = ctrl ~= -4.0
  end
  local kept = gm.array()
  local i = 0
  while i < gm.array_length(global.worms) do
    local w = global.worms[i]
    if gm.bool(w.caught) then
      w.timer = w.timer - 1.0
      if w.total - w.timer >= w.touch_frames then
        w.struggle = gm.max(0, w.struggle - 1)
      end
      hyphae_sound_step(self, other, w.sound, w.total - w.timer)
      if w.timer > 0 or w.timer ~= w.timer then
        gm.array_push(kept, w)
      else
        hyphae_sound_stop(self, other, w.sound)
        food_add(self, other, bal(self, other, "worm_food_reward"))
        local rf = gm.max(1.0, vis(self, other, "leaf_tendril_retract_frames"))
        gm.array_push(global.leaf_tendrils_out, gm.new(___struct___104__worms_step__scr_life, w.tendril, rf, rf))
      end
    else
      local hx = w.body[0].x
      if not (hx < zone.x1 or hx > zone.x2 or w.body[0].y > keep) then
        worm_move(self, other, w)
        if can_catch then
          w.check = w.check - 1.0
          if w.check <= 0 then
            w.check = bal(self, other, "worm_catch_check_frames")
            local mid = w.body[gm.div(gm.array_length(w.body), 2)]
            local half = vis(self, other, "worm_length_px") * 0.5
            local r = bal(self, other, "worm_catch_radius") + half
            local hit = ctrl.myc_get_mycelium_hit(self, other, mid.x, mid.y, r)
            if gm.bool(hit.hit) then
              worm_catch(self, other, w, hit, ctrl)
            end
          end
        end
        gm.array_push(kept, w)
      end
    end
    i = i + 1
  end
  global.worms = kept
  local tkept = gm.array()
  i = 0
  while i < gm.array_length(global.worm_trail) do
    local t = global.worm_trail[i]
    t.life = t.life - 1.0
    if t.life > 0 then
      gm.array_push(tkept, t)
    end
    i = i + 1
  end
  global.worm_trail = tkept
end
