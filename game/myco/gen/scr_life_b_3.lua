local gm = require("gm")

-- gml_Script_worms_draw @ 0x1401655f0
function worms_draw(self, other, vx, vy, vw, vh)
  local col = gm.make_colour_rgb(vis(self, other, "worm_colour_r"), vis(self, other, "worm_colour_g"), vis(self, other, "worm_colour_b"))
  local band = gm.merge_colour(col, 8812740.0, 0.55)
  local hi = 14873582.0
  local th = vis(self, other, "worm_thickness_px")
  local struggle_frames = gm.max(1.0, vis(self, other, "worm_struggle_frames"))
  local ts = gm.max(1.0, th - 1)
  local half = gm.div(ts, 2)
  local trail_frames = gm.max(1.0, vis(self, other, "worm_trail_frames"))
  local trail_alpha = vis(self, other, "worm_trail_alpha")
  gm.draw_set_colour(gm.make_colour_rgb(vis(self, other, "worm_trail_colour_r"), vis(self, other, "worm_trail_colour_g"), vis(self, other, "worm_trail_colour_b")))
  local i = 0
  while i < gm.array_length(global.worm_trail) do
    local t = global.worm_trail[i]
    if not (t.x < vx - 8 or t.x > vx + vw + 8 or t.y < vy - 8 or t.y > vy + vh + 8) then
      gm.draw_set_alpha(trail_alpha * t.life / trail_frames)
      gm.draw_rectangle(t.x - half, t.y - half, t.x - half + ts - 1, t.y - half + ts - 1, false)
    end
    i = i + 1
  end
  gm.draw_set_alpha(1.0)
  local wi = 0
  while wi < gm.array_length(global.worms) do
    local w = global.worms[wi]
    local b = w.body
    local n = gm.array_length(b) - 1
    local h = b[0]
    if not (h.x < vx - 64 or h.x > vx + vw + 64 or h.y < vy - 64 or h.y > vy + vh + 64) then
      local a = 1.0
      local amp = 0
      local wig = 0
      if gm.bool(w.caught) then
        local p = 1.0 - w.timer / gm.max(1.0, w.total)
        amp = p * 0.6
        a = gm.clamp(w.timer / gm.max(1.0, w.total) / 0.3, 0, 1.0)
        if w.total - w.timer < w.touch_frames then
          wig = 1.5 * w.struggle / struggle_frames
        else
          wig = 0
        end
      end
      local body_col = gm.merge_colour(col, hi, amp)
      local band_col = gm.merge_colour(band, hi, amp)
      local total = 0
      local s = 0
      while s < n do
        total = total + gm.point_distance(b[s].x, b[s].y, b[s + 1].x, b[s + 1].y)
        s = s + 1
      end
      total = gm.max(1.0, total)
      gm.draw_set_alpha(a)
      local acc = 0
      s = 0
      while s < n do
        local p0 = b[s]
        local p1 = b[s + 1]
        local d = gm.point_distance(p0.x, p0.y, p1.x, p1.y)
        local dir = gm.point_direction(p0.x, p0.y, p1.x, p1.y)
        local k = 0
        while k < d do
          local u = (acc + k) / total
          local prof = gm.power(gm.abs(u * 2 - 1), 3.0)
          local sz = gm.max(2.0, gm.round(gm.lerp(th, th - 1.5, prof)))
          local off = wig * gm.sin(gm.current_time * 0.025 + u * 9)
          local px = gm.floor(gm.lerp(p0.x, p1.x, k / d) + gm.lengthdir_x(off, dir + 90))
          local py = gm.floor(gm.lerp(p0.y, p1.y, k / d) + gm.lengthdir_y(off, dir + 90))
          local hs = gm.div(sz, 2)
          if u > 0.22 and u < 0.34 then
            gm.draw_set_colour(band_col)
          else
            gm.draw_set_colour(body_col)
          end
          gm.draw_rectangle(px - hs, py - hs, px - hs + sz - 1, py - hs + sz - 1, false)
          k = k + 1
        end
        acc = acc + d
        s = s + 1
      end
    end
    wi = wi + 1
  end
  gm.draw_set_alpha(1.0)
  gm.draw_set_colour(16777215)
end

-- gml_Script____struct___105__intro_init__scr_life @ 0x14016bd80
function ___struct___105__intro_init__scr_life(self, other, x, y, frame_timer)
  self.x = x
  self.y = y
  self.frame = 0
  self.frame_timer = frame_timer
  self.dropped = false
  self.shroom_y = 0
  self.shroom_speed = 0
end

-- gml_Script_intro_init @ 0x14016c0f0
function intro_init(self, other)
  global.intro = gm.new(___struct___105__intro_init__scr_life,
    global.start_x - vis(self, other, "screen_width_px") * 0.5 - hedgehog_frame_w(self, other),
    wld(self, other, "surface_height_px"),
    vis(self, other, "hedgehog_frame_interval"))
end

-- gml_Script_intro_step @ 0x14016c720
function intro_step(self, other)
  if not gm.bool(global.intro_active) then
    return false
  end
  if gm.bool(global.pause_open) or gm.bool(global.title_open) then
    return true
  end
  local it = global.intro
  local floor_y = wld(self, other, "surface_height_px")
  local spd = vis(self, other, "intro_hedgehog_speed")
  it.frame_timer = it.frame_timer - 1.0
  if it.frame_timer <= 0 then
    it.frame_timer = vis(self, other, "hedgehog_frame_interval")
    it.frame = gm.mod(it.frame + 1, 8)
  end
  it.x = it.x + spd
  if not gm.bool(it.dropped) then
    if it.x >= global.start_x then
      it.dropped = true
      it.shroom_y = floor_y + hedgehog_back_y(self, other)
      it.shroom_speed = 0
    end
    return true
  else
    it.shroom_speed = gm.min(it.shroom_speed + vis(self, other, "intro_drop_gravity"), 16.0)
    it.shroom_y = it.shroom_y + it.shroom_speed
    if it.shroom_y < floor_y then
      return true
    end
    intro_finish(self, other)
    return false
  end
end

-- gml_Script____struct___106__intro_finish__scr_life @ 0x14016d6f0
function ___struct___106__intro_finish__scr_life(self, other, x, y, speed, frame, frame_timer, carrying)
  self.x = x
  self.y = y
  self.dir = 1.0
  self.speed = speed
  self.frame = frame
  self.frame_timer = frame_timer
  self.carrying = carrying
end

-- gml_Script_intro_finish @ 0x14016daf0
function intro_finish(self, other)
  global.intro_active = false
  local it = global.intro
  gm.array_push(global.animals, gm.new(___struct___106__intro_finish__scr_life,
    it.x, it.y, vis(self, other, "intro_hedgehog_speed"), it.frame, it.frame_timer, -1.0))
  gm.with(self, obj_mycelium_controller, function(self, other)
    self.myc_add_knot(self, other, self.x, self.y)
  end)
end

-- gml_Script_intro_draw @ 0x14016e4c0
function intro_draw(self, other)
  local it = global.intro
  hedgehog_draw(self, other, it.x, it.y, 1.0, it.frame)
  local sy
  if not gm.bool(it.dropped) then
    sy = it.y + hedgehog_back_y(self, other)
  else
    sy = it.shroom_y
  end
  local sx
  if not gm.bool(it.dropped) then
    sx = it.x
  else
    sx = global.start_x
  end
  gm.draw_sprite(originshroom, 0, gm.floor(sx), gm.floor(sy))
end

-- gml_Script_life_step @ 0x14016ed80
function life_step(self, other)
  mushrooms_step(self, other)
  drops_step(self, other)
  animals_step(self, other)
  boars_step(self, other)
  repeller_clouds_step(self, other)
  squirrels_step(self, other)
  crows_step(self, other)
  leaves_step(self, other)
  worms_step(self, other)
  life_far_step(self, other)
end

-- gml_Script_life_draw @ 0x14016f0a0
function life_draw(self, other)
  squirrel_art_init(self, other)
  if gm.bool(global.intro_active) then
    intro_draw(self, other)
  end
  hedgehog_back_init(self, other)
  local cam = gm.view_camera[0]
  local cx
  if cam ~= -1 then
    cx = gm.camera_get_view_x(cam)
  else
    cx = 0
  end
  local cy
  if cam ~= -1 then
    cy = gm.camera_get_view_y(cam)
  else
    cy = 0
  end
  local cw
  if cam ~= -1 then
    cw = gm.camera_get_view_width(cam)
  else
    cw = vis(self, other, "screen_width_px")
  end
  local ch
  if cam ~= -1 then
    ch = gm.camera_get_view_height(cam)
  else
    ch = vis(self, other, "screen_height_px")
  end
  local i = 0
  while i < gm.array_length(global.leaves) do
    local l = global.leaves[i]
    if not (l.x < cx - 64 or l.x > cx + cw + 64) then
      local a
      if not gm.bool(l.landed) or gm.bool(l.absorbing) then
        a = 1.0
      else
        a = gm.clamp(l.life / 60.0, 0.15, 1.0)
      end
      if gm.bool(l.absorbing) then
        a = a * gm.clamp(l.timer / gm.max(1.0, l.total) / 0.3, 0, 1)
      end
      leaf_draw(self, other, l, a)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if not (m.x < cx - 160 or m.x > cx + cw + 160) then
      if not (m.y < cy - 200 or m.y > cy + ch + 200) then
        if not gm.bool(m.stolen) then
          local col
          if gm.bool(m.active) or m.type == 2 then
            col = 16777215
          else
            col = 7766138
          end
          if m.flash > 0 then
            col = 11067135
          end
          if m.carried_by == -1 then
            mushroom_draw(self, other, m, gm.floor(m.x), gm.floor(m.y), col, 1.0, gm.undefined, gm.undefined)
            if m.type == 1 and gm.bool(m.active) then
              local j = 0
              while j < gm.array_length(m.flies) do
                local f = m.flies[j]
                local fx = m.x + gm.lengthdir_x(f.radius, f.angle)
                local fy = m.y - 62 + gm.lengthdir_y(f.radius * 0.45, f.angle) + gm.lengthdir_y(3.0, f.bob)
                gm.draw_sprite(fly, 0, gm.floor(fx), gm.floor(fy))
                j = j + 1
              end
            end
          end
        end
      end
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.mushrooms) do
    if global.mushrooms[i].type == 5 then
      stinky_draw(self, other, global.mushrooms[i])
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.animals) do
    local an = global.animals[i]
    local margin = hedgehog_border_margin(self, other)
    if not (an.x < cx - margin or an.x > cx + cw + margin) then
      hedgehog_draw(self, other, an.x, an.y, an.dir, an.frame)
      if an.carrying ~= -1 and an.carrying < gm.array_length(global.mushrooms) then
        local c = global.mushrooms[an.carrying]
        local col
        if gm.bool(c.active) or c.type == 2 then
          col = 16777215
        else
          col = 7766138
        end
        -- c[$ "carry_angle"] ?? 0 (struct_get_from_hash)
        local ca = c.carry_angle
        if gm.is_undefined(ca) then
          ca = 0
        end
        mushroom_draw(self, other, c, gm.floor(c.x), gm.floor(c.y), col, 1.0, ca, vis(self, other, "hedgehog_carry_scale"))
      end
    end
    i = i + 1
  end
  boars_draw(self, other, cx, cw)
  repeller_clouds_draw(self, other, cx, cw)
  local sq_margin = squirrel_frame_w(self, other) + 80
  i = 0
  while i < gm.array_length(global.squirrels) do
    local s = global.squirrels[i]
    if not (s.x < cx - sq_margin or s.x > cx + cw + sq_margin) then
      local m = s.mushroom
      if not gm.is_undefined(m) and gm.bool(m.stolen) then
        local spr = mushroom_type_sprite(self, other, m.type)
        local left = mushroom_cell_left(self, other, spr, mushroom_variant(self, other, m))
        local w = gm.min(mushroom_cell_width(self, other, spr), gm.sprite_get_width(spr) - left)
        local ox = -gm.real(gm.sprite_get_xoffset(spr)) * m.hold_xscale
        local oy = -gm.real(gm.sprite_get_yoffset(spr)) * m.hold_yscale
        local c = gm.dcos(m.hold_angle)
        local sn = gm.dsin(m.hold_angle)
        gm.draw_sprite_general(spr, 0, left, 0, w, gm.sprite_get_height(spr),
          gm.floor(m.x) + ox * c + oy * sn, gm.floor(m.y) - ox * sn + oy * c,
          m.hold_xscale, m.hold_yscale, m.hold_angle, 16777215.0, 16777215.0, 16777215.0, 16777215.0, 1.0)
      end
      squirrel_draw(self, other, s)
    end
    i = i + 1
  end
  crows_draw(self, other, cx, cy, cw, ch)
  hover_bar_step_draw(self, other)
  i = 0
  while i < gm.array_length(global.drops) do
    local d = global.drops[i]
    if not (d.x < cx - 64 or d.x > cx + cw + 64) then
      gm.draw_sprite(waterdrop, 0, gm.floor(d.x), gm.floor(d.y))
    end
    i = i + 1
  end
end

-- gml_Script_backdrop_leaf_band_setting @ 0x140173a30
function backdrop_leaf_band_setting(self, other, band, near_key, far_key)
  if band == 0 then
    return vis(self, other, far_key)
  else
    return vis(self, other, near_key)
  end
end

-- gml_Script_backdrop_leaf_count @ 0x140173dd0
function backdrop_leaf_count(self, other, band)
  local n = backdrop_leaf_band_setting(self, other, band, "backdrop_leaf_near_count", "backdrop_leaf_far_count")
  if global.wind_speed >= vis(self, other, "wind_max_strength") then
    n = gm.round(n * vis(self, other, "backdrop_leaf_strong_wind_mult"))
  end
  return n
end

-- gml_Script_backdrop_leaf_parallax @ 0x140174300
function backdrop_leaf_parallax(self, other, band)
  return backdrop_leaf_band_setting(self, other, band, "backdrop_leaf_near_parallax", "backdrop_leaf_far_parallax")
end

-- gml_Script____struct___107__backdrop_leaf_make__scr_life @ 0x140174510
function ___struct___107__backdrop_leaf_make__scr_life(self, other, lx, y, fall, drift, phase, face, change_from, scale, alpha)
  self.lx = lx
  self.y = y
  self.fall = fall
  self.drift = drift
  self.phase = phase
  self.angle = 0
  self.hold_angle = 0
  self.face = face
  self.change_from = change_from
  self.change_frame = 0
  self.change_timer = 0
  self.scale = scale
  self.alpha = alpha
end

-- gml_Script_backdrop_leaf_make @ 0x140174aa0
function backdrop_leaf_make(self, other, band, cam_x, w, anywhere)
  local floor_y = wld(self, other, "surface_height_px")
  local par = backdrop_leaf_parallax(self, other, band)
  local rs = gm.random_range(0.85, 1.15)
  local sc = backdrop_leaf_band_setting(self, other, band, "backdrop_leaf_near_scale", "backdrop_leaf_far_scale") * rs
  local lx = cam_x * par + gm.random_range(-80.0, w + 80)
  local y
  if not gm.bool(anywhere) then
    y = gm.random_range(-90.0, -10.0)
  else
    y = gm.random(floor_y)
  end
  local fall = gm.random_range(vis(self, other, "backdrop_leaf_fall_min"), vis(self, other, "backdrop_leaf_fall_max"))
  local drift = global.wind_dir * global.wind_speed * vis(self, other, "backdrop_leaf_wind_push")
  local phase = gm.random(360.0)
  local face = leaf_face_for_wind(self, other, global.wind_dir)
  local ra = gm.random_range(0.85, 1.1)
  local alpha = backdrop_leaf_band_setting(self, other, band, "backdrop_leaf_near_alpha", "backdrop_leaf_far_alpha") * ra
  return gm.new(___struct___107__backdrop_leaf_make__scr_life, lx, y, fall, drift, phase, face, -1.0, sc, alpha)
end

-- gml_Script_backdrop_leaves_init @ 0x140175ba0
function backdrop_leaves_init(self, other, cam_x, w)
  if gm.bool(gm.variable_global_exists("backdrop_leaves")) and gm.bool(gm.is_array(global.backdrop_leaves)) then
    return
  end
  global.backdrop_leaves = gm.array()
  local band = 0
  while band < 2.0 do
    local n = backdrop_leaf_count(self, other, band)
    local arr = gm.array()
    local i = 0
    while i < n do
      gm.array_push(arr, backdrop_leaf_make(self, other, band, cam_x, w, true))
      i = i + 1
    end
    gm.array_push(global.backdrop_leaves, arr)
    band = band + 1
  end
end

-- gml_Script_backdrop_leaves_step @ 0x140176470
function backdrop_leaves_step(self, other, cam_x, w)
  backdrop_leaves_init(self, other, cam_x, w)
  local floor_y = wld(self, other, "surface_height_px")
  local force = global.wind_dir * global.wind_speed
  local target_face = leaf_face_for_wind(self, other, global.wind_dir)
  local wind_ease = gm.clamp(vis(self, other, "backdrop_leaf_wind_ease"), 0, 1.0)
  local spin_ease = gm.clamp(vis(self, other, "leaf_spin_ease"), 0, 1.0)
  local push = vis(self, other, "backdrop_leaf_wind_push")
  local band = 0
  while band < 2.0 do
    local arr = global.backdrop_leaves[band]
    local par = backdrop_leaf_parallax(self, other, band)
    local count = backdrop_leaf_count(self, other, band)
    if gm.array_length(arr) < count then
      local nl = backdrop_leaf_make(self, other, band, cam_x, w, false)
      local edge
      if force >= 0 then
        edge = -100.0
      else
        edge = w + 100
      end
      nl.lx = cam_x * par + edge
      nl.y = gm.random(floor_y * 0.7)
      gm.array_push(arr, nl)
    end
    local i = gm.array_length(arr) - 1
    while i >= 0 do
      local l = arr[i]
      if l.change_from < 0 and l.face ~= target_face then
        l.change_from = l.face
        l.change_frame = 0
        l.change_timer = vis(self, other, "leaf_change_frame_interval")
        l.hold_angle = l.angle
      end
      if l.change_from >= 0 then
        l.change_timer = l.change_timer - 1.0
        if l.change_timer <= 0 then
          l.change_timer = vis(self, other, "leaf_change_frame_interval")
          l.change_frame = l.change_frame + 1
          if l.change_frame >= 8.0 then
            l.face = 1.0 - l.change_from
            l.change_from = -1.0
            l.change_frame = 0
          end
        end
      end
      l.drift = l.drift + (force * push - l.drift) * wind_ease
      l.phase = l.phase + 2.4
      local spin = gm.sin(gm.degtorad(l.phase)) * vis(self, other, "leaf_spin_degrees")
      local target
      if l.change_from >= 0 then
        target = l.hold_angle
      else
        target = spin
      end
      l.angle = l.angle + (target - l.angle) * spin_ease
      l.lx = l.lx + (l.drift + gm.lengthdir_x(vis(self, other, "backdrop_leaf_sway"), l.phase))
      l.y = l.y + l.fall
      local sx = l.lx - cam_x * par
      local off = sx < -140.0 or sx > w + 140
      if l.y > floor_y + 40 or off then
        if gm.array_length(arr) > count then
          gm.array_delete(arr, i, 1.0)
        else
          arr[i] = backdrop_leaf_make(self, other, band, cam_x, w, false)
          if off then
            local edge
            if force >= 0 then
              edge = -100.0
            else
              edge = w + 100
            end
            arr[i].lx = cam_x * par + edge
            arr[i].y = gm.random(floor_y * 0.7)
          end
        end
      end
      i = i - 1.0
    end
    band = band + 1
  end
end

-- gml_Script_backdrop_leaves_draw @ 0x140179cb0
function backdrop_leaves_draw(self, other, cam_x, view_y, view_w, view_h)
  if not (gm.bool(gm.variable_global_exists("backdrop_leaves")) and gm.bool(gm.is_array(global.backdrop_leaves))) then
    return
  end
  leaf_art_init(self, other)
  local floor_y = wld(self, other, "surface_height_px")
  if view_y > floor_y or view_y + view_h < 0 then
    return
  end
  local col = vis(self, other, "backdrop_leaf_colour")
  local band = 0
  while band < 2.0 do
    local arr = global.backdrop_leaves[band]
    local par = backdrop_leaf_parallax(self, other, band)
    local i = 0
    while i < gm.array_length(arr) do
      local l = arr[i]
      local sx = l.lx - cam_x * par
      if not (sx < -80.0 or sx > view_w + 80) then
        local art = leaf_art_frame(self, other, l)
        gm.draw_sprite_ext(art.shape, 0, gm.round(cam_x + sx), gm.round(l.y), l.scale, l.scale, l.angle, col, gm.clamp(l.alpha, 0, 1.0))
      end
      i = i + 1
    end
    band = band + 1
  end
end

-- gml_Script____struct___108__animal_marker_list__scr_life @ 0x14017af70
function ___struct___108__animal_marker_list__scr_life(self, other, src, frame, mirror, x, y, w, h)
  self.src = src
  self.sprite = hedgehog
  self.frames = 8.0
  self.frame = frame
  self.mirror = mirror
  self.x = x
  self.y = y
  self.w = w
  self.h = h
end

-- gml_Script____struct___109__animal_marker_list__scr_life @ 0x14017b400
function ___struct___109__animal_marker_list__scr_life(self, other, src, frame, mirror, x, y, w, h)
  self.src = src
  self.sprite = boar
  self.frames = 6.0
  self.frame = frame
  self.mirror = mirror
  self.x = x
  self.y = y
  self.w = w
  self.h = h
end

-- gml_Script____struct___110__animal_marker_list__scr_life @ 0x14017b890
function ___struct___110__animal_marker_list__scr_life(self, other, src, frame, mirror, x, y, w, h)
  self.src = src
  self.sprite = squirrel
  self.frames = 9.0
  self.frame = frame
  self.mirror = mirror
  self.x = x
  self.y = y
  self.w = w
  self.h = h
end

-- gml_Script____struct___111__animal_marker_list__scr_life @ 0x14017bd20
function ___struct___111__animal_marker_list__scr_life(self, other, src, frame, mirror, x, y, w, h)
  self.src = src
  self.sprite = crow
  self.frames = 9.0
  self.frame = frame
  self.mirror = mirror
  self.x = x
  self.y = y
  self.w = w
  self.h = h
end

-- gml_Script____struct___112__animal_marker_list__scr_life @ 0x14017c1b0
function ___struct___112__animal_marker_list__scr_life(self, other, src, frame, mirror, x, y, w, h)
  self.src = src
  self.sprite = crow
  self.frames = 9.0
  self.frame = frame
  self.mirror = mirror
  self.x = x
  self.y = y
  self.w = w
  self.h = h
  self.warn = true
end

-- gml_Script_animal_marker_list @ 0x14017c690
function animal_marker_list(self, other)
  local list = gm.array()
  local i = 0
  while i < gm.array_length(global.animals) do
    local a = global.animals[i]
    local h = hedgehog_frame_h(self, other)
    local mirror
    if a.dir >= 0 then
      mirror = -1.0
    else
      mirror = 1.0
    end
    gm.array_push(list, gm.new(___struct___108__animal_marker_list__scr_life, a, a.frame, mirror, a.x, a.y - h * 0.5, hedgehog_frame_w(self, other), h))
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.boars) do
    local b = global.boars[i]
    local h = gm.sprite_get_height(boar) * vis(self, other, "boar_scale")
    local mirror
    if b.dir >= 0 then
      mirror = -1.0
    else
      mirror = 1.0
    end
    gm.array_push(list, gm.new(___struct___109__animal_marker_list__scr_life, b, b.frame, mirror, b.x, b.y - h * 0.5, boar_frame_w(self, other), h))
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.squirrels) do
    local s = global.squirrels[i]
    local sc = vis(self, other, "squirrel_scale")
    gm.array_push(list, gm.new(___struct___110__animal_marker_list__scr_life, s, s.frame, s.mirror, s.x, s.y,
      squirrel_frame_w(self, other) * sc, gm.sprite_get_height(squirrel) * sc))
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.crows) do
    local c = global.crows[i]
    local sc = vis(self, other, "crow_scale")
    local h = gm.sprite_get_height(crow) * sc
    gm.array_push(list, gm.new(___struct___111__animal_marker_list__scr_life, c, c.frame, -c.dir, c.x, c.y - h * 0.5, crow_frame_w(self, other) * sc, h))
    i = i + 1
  end
  if gm.bool(gm.variable_global_exists("crow_pending")) then
    i = 0
    while i < gm.array_length(global.crow_pending) do
      local p = global.crow_pending[i]
      if not gm.bool(gm.is_undefined(p)) then
        local sc = vis(self, other, "crow_scale")
        local h = gm.sprite_get_height(crow) * sc
        gm.array_push(list, gm.new(___struct___112__animal_marker_list__scr_life, p, gm.mod(gm.current_time / 120.0, 9), -p.dir, p.x, p.y - h * 0.5, crow_frame_w(self, other) * sc, h))
      end
      i = i + 1
    end
  end
  return list
end

-- gml_Script_animal_markers_update @ 0x14017fe40
function animal_markers_update(self, other, vx, vy, vw, vh)
  if not gm.bool(gm.variable_global_exists("animal_markers")) then
    global.animal_markers = gm.array()
  end
  local cur = animal_marker_list(self, other)
  local old = global.animal_markers
  local blocked = ui_blocking(self, other)
  local dt = gm.delta_time / 1000000.0
  local fade = gm.max(0.01, vis(self, other, "animal_marker_fade_seconds"))
  local i = 0
  while i < gm.array_length(old) do
    old[i].seen = false
    i = i + 1
  end
  local k = 0
  while k < gm.array_length(cur) do
    local c = cur[k]
    local m = gm.undefined
    local j = 0
    while j < gm.array_length(old) do
      if old[j].src == c.src then
        m = old[j]
        break
      end
      j = j + 1
    end
    if gm.bool(gm.is_undefined(m)) then
      m = c
      m.alpha = 0
      m.vx = 0
      m.vy = 0
      m.gone = false
      gm.array_push(old, m)
    elseif not gm.bool(blocked) then
      m.vx = c.x - m.x
      m.vy = c.y - m.y
      m.x = c.x
      m.y = c.y
      m.frame = c.frame
      m.mirror = c.mirror
    end
    m.seen = true
    k = k + 1
  end
  local keep = gm.array()
  i = 0
  while i < gm.array_length(old) do
    local m = old[i]
    if not gm.bool(m.seen) and not gm.bool(m.gone) then
      m.gone = true
      -- m[$ "warn"] ?? false (struct_get_from_hash)
      local warn = m.warn
      if gm.is_undefined(warn) then
        warn = false
      end
      if not gm.bool(warn) then
        m.follow = vis(self, other, "animal_marker_follow_seconds")
      else
        m.follow = 0
      end
      warn = m.warn
      if gm.is_undefined(warn) then
        warn = false
      end
      if gm.bool(warn) then
        m.vx = 0
        m.vy = 0
      end
    end
    if gm.bool(m.gone) and not gm.bool(blocked) then
      m.x = m.x + m.vx
      m.y = m.y + m.vy
      m.frame = gm.mod(m.frame + 0.25, m.frames)
      m.follow = m.follow - dt
    end
    local sx = m.x - vx
    local sy = m.y - vy
    local on = false
    if sx + m.w * 0.5 > 0 and sx - m.w * 0.5 < vw and sy + m.h * 0.5 > 0 and sy - m.h * 0.5 < vh then
      on = true
    end
    local target
    if not on or (gm.bool(m.gone) and m.follow <= 0) then
      target = 0
    else
      target = 1.0
    end
    if target > m.alpha then
      m.alpha = gm.min(target, m.alpha + dt / fade)
    else
      m.alpha = gm.max(target, m.alpha - dt / fade)
    end
    if not gm.bool(m.gone) or m.alpha > 0 or m.follow > 0 then
      gm.array_push(keep, m)
    end
    i = i + 1
  end
  global.animal_markers = keep
end

-- gml_Script_animal_markers_draw @ 0x140183d70
function animal_markers_draw(self, other, vx, vy, vw, vh)
  animal_markers_update(self, other, vx, vy, vw, vh)
  local list = global.animal_markers
  local r = vis(self, other, "animal_marker_radius")
  local edge = r + vis(self, other, "animal_marker_edge_px")
  local hw = vw * 0.5
  local hh = vh * 0.5
  local col = 14873582.0
  local bg = 1185298.0
  local i = 0
  while i < gm.array_length(list) do
    local m = list[i]
    local a = m.alpha
    if not (a <= 0) then
      local sx = m.x - vx
      local sy = m.y - vy
      local dx = sx - hw
      local dy = sy - hh
      local tx
      if dx > 0 then
        tx = (vw - edge - hw) / dx
      elseif dx < 0 then
        tx = (edge - hw) / dx
      else
        tx = gm.infinity
      end
      local ty
      if dy > 0 then
        ty = (vh - vis(self, other, "animal_marker_bottom_px") - hh) / dy
      elseif dy < 0 then
        ty = (vis(self, other, "animal_marker_top_px") - hh) / dy
      else
        ty = gm.infinity
      end
      local t = gm.min(1.0, tx, ty)
      local ux = hw + dx * t
      local uy = hh + dy * t
      local dir = gm.point_direction(hw, hh, sx, sy)
      local hover = gm.bool(gm.variable_global_exists("animal_marker_hover")) and global.animal_marker_hover == m
      m.ui_x = ux
      m.ui_y = uy
      gm.draw_set_alpha(a)
      gm.draw_set_colour(col)
      gm.draw_triangle(
        ux + gm.lengthdir_x(r + 18, dir), uy + gm.lengthdir_y(r + 18, dir),
        ux + gm.lengthdir_x(r + 4, dir + 16), uy + gm.lengthdir_y(r + 4, dir + 16),
        ux + gm.lengthdir_x(r + 4, dir - 16), uy + gm.lengthdir_y(r + 4, dir - 16),
        false)
      gm.draw_set_colour(bg)
      gm.draw_set_alpha(0.92 * a)
      gm.draw_circle(ux, uy, r, false)
      local fw = gm.sprite_get_width(m.sprite) / m.frames
      local fh = gm.sprite_get_height(m.sprite)
      local sc = r * 2 * vis(self, other, "animal_marker_fill") / gm.max(fw, fh)
      local fr = gm.clamp(gm.floor(m.frame), 0, m.frames - 1)
      gm.draw_sprite_part_ext(m.sprite, 0, fr * fw, 0, fw, fh,
        ux - m.mirror * fw * sc * 0.5, uy - fh * sc * 0.5, m.mirror * sc, sc, 16777215.0, a)
      gm.draw_set_alpha(a)
      gm.draw_set_colour(col)
      gm.draw_circle(ux, uy, r, true)
      gm.draw_circle(ux, uy, r - 1, true)
      if hover then
        gm.draw_circle(ux, uy, r + 3, true)
        gm.draw_circle(ux, uy, r + 4, true)
      end
    end
    i = i + 1
  end
  gm.draw_set_colour(16777215)
  gm.draw_set_alpha(1.0)
end

-- gml_GlobalScript_scr_life @ 0x140189630
function gml_GlobalScript_scr_life(self, other)
  self.hedgehog_frame_w = gm.method(self, hedgehog_frame_w)
  self.hedgehog_frame_h = gm.method(self, hedgehog_frame_h)
  self.hedgehog_back_y = gm.method(self, hedgehog_back_y)
  self.hedgehog_back_init = gm.method(self, hedgehog_back_init)
  self.hedgehog_back_point = gm.method(self, hedgehog_back_point)
  self.hedgehog_draw = gm.method(self, hedgehog_draw)
  self.animal_interval = gm.method(self, animal_interval)
  self.animal_mushroom_count = gm.method(self, animal_mushroom_count)
  self.animal_timer_step = gm.method(self, animal_timer_step)
  self.life_init = gm.method(self, life_init)
  self.life_animal_timers_init = gm.method(self, life_animal_timers_init)
  self.experience_price_mult = gm.method(self, experience_price_mult)
  self.mushroom_type_cost = gm.method(self, mushroom_type_cost)
  self.mushroom_type_cost_base = gm.method(self, mushroom_type_cost_base)
  self.mushroom_type_water_cost = gm.method(self, mushroom_type_water_cost)
  self.mushroom_type_water_cost_base = gm.method(self, mushroom_type_water_cost_base)
  self.mushroom_type_affordable = gm.method(self, mushroom_type_affordable)
  self.mushroom_type_unlock_id = gm.method(self, mushroom_type_unlock_id)
  self.mushroom_type_name_key = gm.method(self, mushroom_type_name_key)
  self.mushroom_type_sprite = gm.method(self, mushroom_type_sprite)
  self.shroom_sheet_cells = gm.method(self, shroom_sheet_cells)
  self.mushroom_type_variants = gm.method(self, mushroom_type_variants)
  self.mushroom_cell_width = gm.method(self, mushroom_cell_width)
  self.mushroom_variant = gm.method(self, mushroom_variant)
  self.shroom_grow_scale = gm.method(self, shroom_grow_scale)
  self.mushroom_cell_left = gm.method(self, mushroom_cell_left)
  self.mushroom_sprite_now = gm.method(self, mushroom_sprite_now)
  self.mushroom_draw = gm.method(self, mushroom_draw)
  self.mushroom_type_unlocked = gm.method(self, mushroom_type_unlocked)
  self.predator_period = gm.method(self, predator_period)
  self.predator_yield = gm.method(self, predator_yield)
  self.repeller_max_charges = gm.method(self, repeller_max_charges)
  self.repeller_autoregen_frames = gm.method(self, repeller_autoregen_frames)
  self.repeller_regen_price = gm.method(self, repeller_regen_price)
  self.tree_drop_period = gm.method(self, tree_drop_period)
  self.tree_drop_water = gm.method(self, tree_drop_water)
  self.leaf_face_for_wind = gm.method(self, leaf_face_for_wind)
  self.leaf_face_sprite = gm.method(self, leaf_face_sprite)
  self.leaf_change_sprite = gm.method(self, leaf_change_sprite)
  self.leaf_art_ready = gm.method(self, leaf_art_ready)
  self.leaf_art_init = gm.method(self, leaf_art_init)
  self.leaf_art_read = gm.method(self, leaf_art_read)
  self.leaf_art_frame = gm.method(self, leaf_art_frame)
  self.leaf_fg_art_ready = gm.method(self, leaf_fg_art_ready)
  self.leaf_fg_art_init = gm.method(self, leaf_fg_art_init)
  self.leaf_fg_art_read = gm.method(self, leaf_fg_art_read)
  self.leaf_fg_frame = gm.method(self, leaf_fg_frame)
  self.leaf_fg_mirror = gm.method(self, leaf_fg_mirror)
  self.leaf_start_turn = gm.method(self, leaf_start_turn)
  self.leaves_turn_all = gm.method(self, leaves_turn_all)
  self.leaf_fg_drop = gm.method(self, leaf_fg_drop)
  self.leaf_draw = gm.method(self, leaf_draw)
  self.leaf_decay_time = gm.method(self, leaf_decay_time)
  self.leaf_food_value = gm.method(self, leaf_food_value)
  self.mushroom_add = gm.method(self, mushroom_add)
  self.mushroom_hit_box = gm.method(self, mushroom_hit_box)
  self.mushroom_hit_box_unmirrored = gm.method(self, mushroom_hit_box_unmirrored)
  self.mushrooms_step = gm.method(self, mushrooms_step)
  self.drops_step = gm.method(self, drops_step)
  self.hedgehog_world_span = gm.method(self, hedgehog_world_span)
  self.hedgehog_border_margin = gm.method(self, hedgehog_border_margin)
  self.animal_spawn_hedgehog = gm.method(self, animal_spawn_hedgehog)
  self.hedgehog_summon = gm.method(self, hedgehog_summon)
  self.hedgehog_takes = gm.method(self, hedgehog_takes)
  self.animals_step = gm.method(self, animals_step)
  self.animals_reindex_after_removal = gm.method(self, animals_reindex_after_removal)
  self.animals_rebind = gm.method(self, animals_rebind)
  self.boar_frame_w = gm.method(self, boar_frame_w)
  self.boar_hits_mushroom = gm.method(self, boar_hits_mushroom)
  self.boar_any_target = gm.method(self, boar_any_target)
  self.boar_pick_targets = gm.method(self, boar_pick_targets)
  self.boar_spawn = gm.method(self, boar_spawn)
  self.boar_spawn_pack = gm.method(self, boar_spawn_pack)
  self.boar_summon = gm.method(self, boar_summon)
  self.boar_fling = gm.method(self, boar_fling)
  self.boars_step = gm.method(self, boars_step)
  self.boar_shake = gm.method(self, boar_shake)
  self.boars_draw = gm.method(self, boars_draw)
  self.repeller_cloud_fire = gm.method(self, repeller_cloud_fire)
  self.repeller_animal_front = gm.method(self, repeller_animal_front)
  self.repeller_clouds_step = gm.method(self, repeller_clouds_step)
  self.repeller_soft_sprite = gm.method(self, repeller_soft_sprite)
  self.repeller_cloud_art_init = gm.method(self, repeller_cloud_art_init)
  self.repeller_cloud_draw_body = gm.method(self, repeller_cloud_draw_body)
  self.repeller_cloud_draw_flash = gm.method(self, repeller_cloud_draw_flash)
  self.repeller_cloud_surface = gm.method(self, repeller_cloud_surface)
  self.repeller_clouds_draw = gm.method(self, repeller_clouds_draw)
  self.squirrel_frame_w = gm.method(self, squirrel_frame_w)
  self.squirrel_frame_h = gm.method(self, squirrel_frame_h)
  self.squirrel_offset = gm.method(self, squirrel_offset)
  self.squirrel_point = gm.method(self, squirrel_point)
  self.squirrel_draw = gm.method(self, squirrel_draw)
  self.squirrel_view = gm.method(self, squirrel_view)
  self.mushroom_index_of = gm.method(self, mushroom_index_of)
  self.squirrel_trunk_edges = gm.method(self, squirrel_trunk_edges)
  self.squirrel_art_init = gm.method(self, squirrel_art_init)
  self.squirrel_frames_build = gm.method(self, squirrel_frames_build)
  self.squirrel_edge_x = gm.method(self, squirrel_edge_x)
  self.squirrel_climb_x = gm.method(self, squirrel_climb_x)
  self.squirrel_run_y = gm.method(self, squirrel_run_y)
  self.squirrel_tail_drop = gm.method(self, squirrel_tail_drop)
  self.squirrel_tree_near = gm.method(self, squirrel_tree_near)
  self.squirrel_tree_has_mushroom = gm.method(self, squirrel_tree_has_mushroom)
  self.squirrel_takes = gm.method(self, squirrel_takes)
  self.squirrel_tree_for = gm.method(self, squirrel_tree_for)
  self.squirrel_bite_y = gm.method(self, squirrel_bite_y)
  self.squirrel_shroom_bite = gm.method(self, squirrel_shroom_bite)
  self.squirrel_pick = gm.method(self, squirrel_pick)
  self.squirrel_sky_y = gm.method(self, squirrel_sky_y)
  self.squirrel_any_target = gm.method(self, squirrel_any_target)
  self.squirrel_spawn = gm.method(self, squirrel_spawn)
  self.squirrel_summon = gm.method(self, squirrel_summon)
  self.squirrel_hold_pose = gm.method(self, squirrel_hold_pose)
  self.squirrel_animate = gm.method(self, squirrel_animate)
  self.squirrel_flip_up = gm.method(self, squirrel_flip_up)
  self.squirrel_flip_down = gm.method(self, squirrel_flip_down)
  self.squirrel_tree03_find = gm.method(self, squirrel_tree03_find)
  self.squirrel_check_repeller = gm.method(self, squirrel_check_repeller)
  self.squirrel_turn_start = gm.method(self, squirrel_turn_start)
  self.squirrel_turn_step = gm.method(self, squirrel_turn_step)
  self.squirrel_grab = gm.method(self, squirrel_grab)
  self.squirrels_step = gm.method(self, squirrels_step)
  self.crow_frame_w = gm.method(self, crow_frame_w)
  self.crow_claw = gm.method(self, crow_claw)
  self.crow_below_claws = gm.method(self, crow_below_claws)
  self.crow_claw_offset = gm.method(self, crow_claw_offset)
  self.crow_cap_dy = gm.method(self, crow_cap_dy)
  self.crow_can_take = gm.method(self, crow_can_take)
  self.crow_pick = gm.method(self, crow_pick)
  self.crow_sky_y = gm.method(self, crow_sky_y)
  self.crow_spawn = gm.method(self, crow_spawn)
  self.crow_warn_check = gm.method(self, crow_warn_check)
  self.crow_due = gm.method(self, crow_due)
  self.crow_summon = gm.method(self, crow_summon)
  self.crows_step = gm.method(self, crows_step)
  self.crows_draw = gm.method(self, crows_draw)
  self.life_far_init = gm.method(self, life_far_init)
  self.life_far_measure = gm.method(self, life_far_measure)
  self.life_far_outside = gm.method(self, life_far_outside)
  self.life_far_step = gm.method(self, life_far_step)
  self.consume_info = gm.method(self, consume_info)
  self.consume_frames = gm.method(self, consume_frames)
  self.consume_food = gm.method(self, consume_food)
  self.consumptors_on = gm.method(self, consumptors_on)
  self.consumptor_variant = gm.method(self, consumptor_variant)
  self.consumptor_progress = gm.method(self, consumptor_progress)
  self.consume_finish = gm.method(self, consume_finish)
  self.predator_progress = gm.method(self, predator_progress)
  self.experience_period = gm.method(self, experience_period)
  self.experience_progress = gm.method(self, experience_progress)
  self.tree_shroom_progress = gm.method(self, tree_shroom_progress)
  self.repeller_regen_progress = gm.method(self, repeller_regen_progress)
  self.hover_bar_draw = gm.method(self, hover_bar_draw)
  self.hover_bar_step_draw = gm.method(self, hover_bar_step_draw)
  self.stinky_cap_height = gm.method(self, stinky_cap_height)
  self.stinky_shape = gm.method(self, stinky_shape)
  self.stinky_top_at = gm.method(self, stinky_top_at)
  self.stinky_funnel_radius = gm.method(self, stinky_funnel_radius)
  self.stinky_pool = gm.method(self, stinky_pool)
  self.stinky_particle_add = gm.method(self, stinky_particle_add)
  self.stinky_wind_target = gm.method(self, stinky_wind_target)
  self.stinky_wind_shift = gm.method(self, stinky_wind_shift)
  self.stinky_step = gm.method(self, stinky_step)
  self.stinky_art_init = gm.method(self, stinky_art_init)
  self.stinky_shade_table = gm.method(self, stinky_shade_table)
  self.stinky_draw = gm.method(self, stinky_draw)
  self.stinky_beam_hits = gm.method(self, stinky_beam_hits)
  self.stinky_squirrel_reach = gm.method(self, stinky_squirrel_reach)
  self.stinky_guards = gm.method(self, stinky_guards)
  self.life_near_zone = gm.method(self, life_near_zone)
  self.leaf_spawn = gm.method(self, leaf_spawn)
  self.leaf_gust_spawn = gm.method(self, leaf_gust_spawn)
  self.leaves_step = gm.method(self, leaves_step)
  self.leaf_tendril_make = gm.method(self, leaf_tendril_make)
  self.leaf_tendril_cells_draw = gm.method(self, leaf_tendril_cells_draw)
  self.leaf_tendrils_draw = gm.method(self, leaf_tendrils_draw)
  self.worm_zone_top = gm.method(self, worm_zone_top)
  self.worm_zone_bottom = gm.method(self, worm_zone_bottom)
  self.worm_cell_ok = gm.method(self, worm_cell_ok)
  self.worm_segment_px = gm.method(self, worm_segment_px)
  self.worm_add = gm.method(self, worm_add)
  self.worm_keep_bottom = gm.method(self, worm_keep_bottom)
  self.worm_spawn = gm.method(self, worm_spawn)
  self.worm_absorb_frames = gm.method(self, worm_absorb_frames)
  self.worm_move = gm.method(self, worm_move)
  self.worm_body_at = gm.method(self, worm_body_at)
  self.worm_tendril_make = gm.method(self, worm_tendril_make)
  self.worm_catch = gm.method(self, worm_catch)
  self.worms_step = gm.method(self, worms_step)
  self.worms_draw = gm.method(self, worms_draw)
  self.intro_init = gm.method(self, intro_init)
  self.intro_step = gm.method(self, intro_step)
  self.intro_finish = gm.method(self, intro_finish)
  self.intro_draw = gm.method(self, intro_draw)
  self.life_step = gm.method(self, life_step)
  self.life_draw = gm.method(self, life_draw)
  self.backdrop_leaf_band_setting = gm.method(self, backdrop_leaf_band_setting)
  self.backdrop_leaf_count = gm.method(self, backdrop_leaf_count)
  self.backdrop_leaf_parallax = gm.method(self, backdrop_leaf_parallax)
  self.backdrop_leaf_make = gm.method(self, backdrop_leaf_make)
  self.backdrop_leaves_init = gm.method(self, backdrop_leaves_init)
  self.backdrop_leaves_step = gm.method(self, backdrop_leaves_step)
  self.backdrop_leaves_draw = gm.method(self, backdrop_leaves_draw)
  self.animal_marker_list = gm.method(self, animal_marker_list)
  self.animal_markers_update = gm.method(self, animal_markers_update)
  self.animal_markers_draw = gm.method(self, animal_markers_draw)
  self.animal_marker_at = gm.method(self, animal_marker_at)
  self.camera_fly_to_marker = gm.method(self, camera_fly_to_marker)
end

-- gml_Script_animal_marker_at @ 0x140188860
function animal_marker_at(self, other, px, py)
  if not gm.bool(gm.variable_global_exists("animal_markers")) then
    return gm.undefined
  end
  local hit = vis(self, other, "animal_marker_radius") + 6
  local list = global.animal_markers
  local i = gm.array_length(list) - 1
  while i >= 0 do
    local m = list[i]
    if m.alpha >= 0.5 and gm.bool(gm.variable_struct_exists(m, "ui_x")) then
      if gm.point_distance(px, py, m.ui_x, m.ui_y) <= hit then
        return m
      end
    end
    i = i - 1.0
  end
  return gm.undefined
end

-- gml_Script____struct___113__camera_fly_to_marker__scr_life @ 0x140189200
function ___struct___113__camera_fly_to_marker__scr_life(self, other, marker)
  self.marker = marker
  self.t = 0
end

-- gml_Script_camera_fly_to_marker @ 0x140189390
function camera_fly_to_marker(self, other, marker)
  global.camera_fly = gm.new(___struct___113__camera_fly_to_marker__scr_life, marker)
end
