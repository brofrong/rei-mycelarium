local gm = require("gm")

-- gml_Script____struct___67__boar_spawn__scr_life @ 0x1400d80a0
function ___struct___67__boar_spawn__scr_life(self, other, targets, x, y, dir, frame_timer)
  self.targets = targets
  self.x = x
  self.y = y
  self.dir = dir
  self.frame = 0
  self.frame_timer = frame_timer
  self.hits = 0
  self.fleeing = false
end

-- gml_Script_boar_spawn @ 0x1400d84a0
function boar_spawn(self, other, x, dir)
  gm.array_push(global.boars, gm.new(___struct___67__boar_spawn__scr_life,
    boar_pick_targets(self, other, x, dir), x, wld(self, other, "surface_height_px"), dir,
    vis(self, other, "boar_frame_interval")))
end

-- gml_Script____struct___68__boar_spawn_pack__scr_life @ 0x1400d8b00
function ___struct___68__boar_spawn_pack__scr_life(self, other, x, dir, wait)
  self.x = x
  self.dir = dir
  self.wait = wait
end

-- gml_Script_boar_spawn_pack @ 0x1400d8d50
function boar_spawn_pack(self, other, x, dir)
  boar_spawn(self, other, x, dir)
  if game_mode(self, other) ~= "survival" then
    return
  end
  local gap = gm.max(1, gm.round(vis(self, other, "boar_pack_gap_seconds") * global.fps))
  local i = 1
  while i < gm.max(1, vis(self, other, "boar_pack_size")) do
    gm.array_push(global.boar_pack, gm.new(___struct___68__boar_spawn_pack__scr_life, x, dir, gap * i))
    i = i + 1
  end
end

-- gml_Script_boar_summon @ 0x1400d9850
function boar_summon(self, other)
  local v = squirrel_view(self, other)
  local cx = v.x + v.w * 0.5
  local target = gm.undefined
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if gm.bool(boar_hits_mushroom(self, other, m)) then
      if gm.is_undefined(target) or gm.abs(m.x - cx) < gm.abs(target.x - cx) then
        target = m
      end
    end
    i = i + 1
  end
  local dir
  if gm.is_undefined(target) then
    dir = gm.choose(-1, 1)
  else
    dir = (target.x < cx) and -1 or 1
  end
  local margin = boar_frame_w(self, other) + 40
  local x
  if dir > 0 then
    x = v.x - margin
  else
    x = v.x + v.w + margin
  end
  boar_spawn_pack(self, other, x, dir)
  if gm.is_undefined(target) then
    return "boar (nothing to hit)"
  end
  return "boar"
end

-- gml_Script____struct___69__boar_fling__scr_life @ 0x1400daba0
function ___struct___69__boar_fling__scr_life(self, other, type, sprite, variant, x, y, vx, vy, spin, life)
  self.type = type
  self.sprite = sprite
  self.variant = variant
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.angle = 0
  self.spin = spin
  self.life = life
end

-- gml_Script_boar_fling @ 0x1400db0a0
function boar_fling(self, other, m, dir)
  local spr = mushroom_sprite_now(self, other, m)
  local cw = mushroom_cell_width(self, other, spr)
  local mtype = m.type
  local mx = m.x
  local my = m.y
  local variant = mushroom_variant(self, other, m)
  local x = mx - gm.sprite_get_xoffset(spr) + cw * 0.5
  local y = my - gm.sprite_get_yoffset(spr) + gm.sprite_get_height(spr) * 0.5
  local base_vx = bal(self, other, "boar_speed") * 0.5
  local vx = dir * (base_vx + gm.random_range(vis(self, other, "boar_fling_vx_min"), vis(self, other, "boar_fling_vx_max")))
  local vy = -gm.random_range(vis(self, other, "boar_fling_vy_min"), vis(self, other, "boar_fling_vy_max"))
  local spin = -dir * gm.random_range(vis(self, other, "boar_fling_spin_min"), vis(self, other, "boar_fling_spin_max"))
  local life = vis(self, other, "boar_fling_frames")
  local f = gm.new(___struct___69__boar_fling__scr_life, mtype, spr, variant, x, y, vx, vy, spin, life)
  gm.array_push(global.boar_flung, f)
end

-- gml_Script_boars_step @ 0x1400dc6b0
function boars_step(self, other)
  if gm.bool(boar_any_target(self, other)) then
    global.boar_timer = global.boar_timer - animal_timer_step(self, other)
    if global.boar_timer <= 0 then
      global.boar_timer = animal_interval(self, other, "boar_spawn_interval")
      local span = hedgehog_world_span(self, other)
      local m = boar_frame_w(self, other) + 60
      local from_left = gm.choose(true, false)
      local sx
      if gm.bool(from_left) then
        sx = span.left - m
      else
        sx = span.right + m
      end
      local sdir
      if gm.bool(from_left) then
        sdir = 1.0
      else
        sdir = -1.0
      end
      boar_spawn_pack(self, other, sx, sdir)
    end
  end
  local pack_next = gm.array()
  local i = 0
  while i < gm.array_length(global.boar_pack) do
    local p = global.boar_pack[i]
    p.wait = p.wait - 1
    if not (p.wait <= 0) then
      gm.array_push(pack_next, p)
    else
      boar_spawn(self, other, p.x, p.dir)
    end
    i = i + 1
  end
  global.boar_pack = pack_next
  local span = hedgehog_world_span(self, other)
  local m = boar_frame_w(self, other) + 60
  local keep = gm.array()
  i = 0
  while i < gm.array_length(global.boars) do
    local b = global.boars[i]
    local snout0 = b.x + b.dir * boar_frame_w(self, other) * vis(self, other, "boar_snout_dx")
    b.x = b.x + b.dir * bal(self, other, "boar_speed")
    local snout1 = b.x + b.dir * boar_frame_w(self, other) * vis(self, other, "boar_snout_dx")
    if gm.is_undefined(b.tremor) or gm.bool(b.tremor.done) then
      b.tremor = sound_loop_start(self, other, boar_tremor)
    end
    sound_loop_follow(self, other, b.tremor, b.x, b.y,
      vis(self, other, "boar_tremor_volume"), vis(self, other, "boar_tremor_reach_views"),
      vis(self, other, "boar_tremor_fade_in_frames"), vis(self, other, "boar_tremor_fade_out_frames"))
    b.frame_timer = b.frame_timer - 1
    if b.frame_timer <= 0 then
      b.frame_timer = vis(self, other, "boar_frame_interval")
      b.frame = gm.mod(b.frame + 1, 6)
    end
    if not gm.bool(b.fleeing) and gm.array_length(b.targets) > 0 then
      local lo = gm.min(snout0, snout1) - bal(self, other, "boar_reach_px")
      local hi = gm.max(snout0, snout1) + bal(self, other, "boar_reach_px")
      local left = gm.array()
      local j = 0
      while j < gm.array_length(b.targets) do
        local t = b.targets[j]
        local idx = mushroom_index_of(self, other, t)
        if idx < 0 or not gm.bool(boar_hits_mushroom(self, other, t)) then
          -- continue
        elseif t.x < lo or t.x > hi or (t.type == 2 and gm.bool(t.active)) then
          gm.array_push(left, t)
        else
          boar_fling(self, other, t, b.dir)
          gm.array_delete(global.mushrooms, idx, 1)
          animals_reindex_after_removal(self, other, idx)
          b.hits = b.hits + 1
        end
        j = j + 1
      end
      b.targets = left
      animals_rebind(self, other)
    end
    local gone = (b.dir < 0 and b.x < span.left - m) or (b.dir > 0 and b.x > span.right + m)
    if not gone then
      gm.array_push(keep, b)
    end
    i = i + 1
  end
  global.boars = keep
  local flung = gm.array()
  i = 0
  while i < gm.array_length(global.boar_flung) do
    local f = global.boar_flung[i]
    f.x = f.x + f.vx
    f.y = f.y + f.vy
    f.vy = f.vy + vis(self, other, "boar_fling_gravity")
    f.vx = f.vx * 0.985
    f.angle = f.angle + f.spin
    f.life = f.life - 1
    if f.life > 0 then
      gm.array_push(flung, f)
    end
    i = i + 1
  end
  global.boar_flung = flung
end

-- gml_Script_boar_shake @ 0x1400e0f00
function boar_shake(self, other, vx, vw)
  local c = cine_shake(self, other)
  if not gm.is_undefined(c) then
    return c
  end
  if not gm.variable_global_exists("boars") or gm.bool(ui_blocking(self, other)) then
    return gm.array(0, 0)
  end
  local k = global.repel_kick
  if gm.is_undefined(k) then
    k = 0
  end
  global.repel_kick = k * 0.82
  if global.repel_kick < 0.2 then
    global.repel_kick = 0
  end
  local base = gm.array(k * gm.random_range(-0.5, 0.5), k * gm.random_range(-1, 1))
  if gm.array_length(global.boars) == 0 then
    return base
  end
  local best = 0
  local fall = gm.max(1, vis(self, other, "boar_shake_falloff_px"))
  local i = 0
  while i < gm.array_length(global.boars) do
    local b = global.boars[i]
    local bx = b.x
    local d = gm.max(0, vx - bx, bx - (vx + vw))
    best = gm.max(best, 1 - d / fall)
    i = i + 1
  end
  if best <= 0 then
    return base
  end
  local amp = vis(self, other, "boar_shake_px") * best * best
  local t = gm.current_time
  return gm.array(
    base[0] + amp * 0.45 * (gm.sin(t * 0.031) * 0.6 + gm.sin(t * 0.057 + 1.3) * 0.4),
    base[1] + amp * (gm.sin(t * 0.043 + 0.7) * 0.6 + gm.sin(t * 0.089 + 2.1) * 0.4))
end

-- gml_Script_boars_draw @ 0x1400e2d30
function boars_draw(self, other, vx, vw)
  local scale = vis(self, other, "boar_scale")
  local fw = gm.sprite_get_width(boar) / 6
  local fh = gm.sprite_get_height(boar)
  local margin = boar_frame_w(self, other) + 40
  local i = 0
  while i < gm.array_length(global.boars) do
    local b = global.boars[i]
    if b.x < vx - margin or b.x > vx + vw + margin then
      -- continue
    else
      local face = (b.dir >= 0) and -1.0 or 1.0
      gm.draw_sprite_part_ext(boar, 0, b.frame * fw, 0, fw, fh,
        gm.floor(b.x) - face * fw * scale * 0.5, gm.floor(b.y) - fh * scale,
        face * scale, scale, 16777215, 1)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.boar_flung) do
    local f = global.boar_flung[i]
    local spr = f.sprite
    local left = mushroom_cell_left(self, other, spr, f.variant)
    local w = gm.min(mushroom_cell_width(self, other, spr), gm.sprite_get_width(spr) - left)
    local h = gm.sprite_get_height(spr)
    local c = gm.dcos(f.angle)
    local s = gm.dsin(f.angle)
    local hx = -w * 0.5
    local hy = -h * 0.5
    local alpha = gm.clamp(f.life / (vis(self, other, "boar_fling_frames") * 0.6), 0, 1)
    gm.draw_sprite_general(spr, 0, left, 0, w, h,
      f.x + hx * c + hy * s, f.y - hx * s + hy * c, 1, 1, f.angle,
      16777215, 16777215, 16777215, 16777215, alpha)
    i = i + 1
  end
end

-- gml_Script____struct___70__repeller_cloud_fire__scr_life @ 0x1400e5a90
function ___struct___70__repeller_cloud_fire__scr_life(self, other, x, y, ground, dir, flash, puffs, specks)
  self.x = x
  self.y = y
  self.ground = ground
  self.dir = dir
  self.flash = flash
  self.puffs = puffs
  self.specks = specks
end

-- gml_Script____struct___71__repeller_cloud_fire__scr_life @ 0x1400e5e90
function ___struct___71__repeller_cloud_fire__scr_life(self, other, x, y, vx, vy, size, grow, life, total, colour)
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.size = size
  self.grow = grow
  self.life = life
  self.total = total
  self.colour = colour
  self.dust = true
end

-- gml_Script____struct___72__repeller_cloud_fire__scr_life @ 0x1400e63c0
function ___struct___72__repeller_cloud_fire__scr_life(self, other, x, y, vx, vy, size, grow, life, total, colour)
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.size = size
  self.grow = grow
  self.life = life
  self.total = total
  self.colour = colour
  self.dust = false
end

-- gml_Script____struct___73__repeller_cloud_fire__scr_life @ 0x1400e68f0
function ___struct___73__repeller_cloud_fire__scr_life(self, other, x, y, vx, vy, life, total)
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.life = life
  self.total = total
end

-- gml_Script_repeller_cloud_fire @ 0x1400e6c70
function repeller_cloud_fire(self, other, r, dir)
  tasks_stat_add(self, other, "ach_repels", gm.undefined)
  r.active = false
  r.flash = vis(self, other, "repeller_flash_frames")
  r.charges = r.charges - 1
  local c = gm.new(___struct___70__repeller_cloud_fire__scr_life,
    r.x, r.y - vis(self, other, "repeller_cloud_dy"), wld(self, other, "surface_height_px"), dir,
    vis(self, other, "repeller_cloud_flash_frames"), gm.array(), gm.array())
  sfx_play_at(self, other, explosion, c.x, c.y, vis(self, other, "explosion_volume"), gm.undefined)
  local v = squirrel_view(self, other)
  local off = gm.max(0, v.x - c.x, c.x - (v.x + v.w))
  if off < v.w * 0.5 then
    local k = global.repel_kick
    if gm.is_undefined(k) then
      k = 0
    end
    global.repel_kick = gm.max(k, vis(self, other, "repeller_cloud_kick_px") * (1 - off / (v.w * 0.5)))
  end
  local i = 0
  while i < vis(self, other, "repeller_cloud_dust") do
    local d
    if gm.random(1) < 0.7 then
      d = dir
    else
      d = -dir
    end
    local life = gm.irandom_range(vis(self, other, "repeller_cloud_life_min"), vis(self, other, "repeller_cloud_life_max"))
    local col_t = gm.random(1)
    local px = c.x + gm.random_range(-10, 10)
    local py = c.ground - gm.random_range(2, 10)
    local pvx = d * vis(self, other, "repeller_cloud_speed") * gm.random_range(0.2, 0.75)
    local pvy = -gm.random_range(0, 1.2)
    local psize = gm.random_range(8, 16)
    local pgrow = gm.random_range(vis(self, other, "repeller_cloud_size_max") * 0.5, vis(self, other, "repeller_cloud_size_max") * 0.9)
    gm.array_push(c.puffs, gm.new(___struct___71__repeller_cloud_fire__scr_life,
      px, py, pvx, pvy, psize, pgrow, life, life, gm.merge_colour(14740206, 12767446, col_t)))
    i = i + 1
  end
  local spread = vis(self, other, "repeller_cloud_spread_deg")
  local base
  if dir <= 0 then
    base = 180
  else
    base = 0
  end
  i = 0
  while i < vis(self, other, "repeller_cloud_puffs") do
    local narrow
    if gm.random(1) < 0.8 then
      narrow = 0.5
    else
      narrow = 1.0
    end
    local a = base + gm.random_range(-spread, spread) * narrow + dir * 12
    local spd = vis(self, other, "repeller_cloud_speed") * (0.25 + 0.75 * gm.power(gm.random(1), 0.6))
    local life = gm.irandom_range(vis(self, other, "repeller_cloud_life_min"), vis(self, other, "repeller_cloud_life_max"))
    local col_t = gm.random(1)
    local px = c.x + gm.random_range(-6, 6)
    local py = c.y + gm.random_range(-6, 6)
    local pvx = gm.lengthdir_x(spd, a)
    local pvy = gm.lengthdir_y(spd, a)
    local psize = gm.random_range(vis(self, other, "repeller_cloud_size_min"), vis(self, other, "repeller_cloud_size_min") * 1.6)
    local pgrow = gm.random_range(vis(self, other, "repeller_cloud_size_max") * 0.6, vis(self, other, "repeller_cloud_size_max"))
    gm.array_push(c.puffs, gm.new(___struct___72__repeller_cloud_fire__scr_life,
      px, py, pvx, pvy, psize, pgrow, life, life, gm.merge_colour(16777215, 15133932, col_t)))
    i = i + 1
  end
  i = 0
  while i < vis(self, other, "repeller_cloud_specks") do
    local a = base + gm.random_range(-spread, spread) + dir * 12
    local spd = vis(self, other, "repeller_cloud_speed") * gm.random_range(0.5, 1.4)
    local life = gm.irandom_range(vis(self, other, "repeller_cloud_life_min"), vis(self, other, "repeller_cloud_life_max"))
    gm.array_push(c.specks, gm.new(___struct___73__repeller_cloud_fire__scr_life,
      c.x, c.y, gm.lengthdir_x(spd, a), gm.lengthdir_y(spd, a), life, life))
    i = i + 1
  end
  gm.array_push(global.repel_clouds, c)
end

-- gml_Script_repeller_animal_front @ 0x1400ebb10
function repeller_animal_front(self, other, x, dir, reach)
  return x + dir * reach
end

-- gml_Script_repeller_clouds_step @ 0x1400ebcb0
function repeller_clouds_step(self, other)
  local radius = bal(self, other, "repeller_trigger_radius")
  local hreach = hedgehog_frame_w(self, other) * 0.4
  local breach = boar_frame_w(self, other) * vis(self, other, "boar_snout_dx")
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if m.type == 2 and gm.bool(m.active) and m.carried_by == -1 and not gm.bool(m.stolen) then
      local fired = false
      local j = 0
      while j < gm.array_length(global.animals) and not fired do
        local a = global.animals[j]
        local gap = (m.x - repeller_animal_front(self, other, a.x, a.dir, hreach)) * a.dir
        if gap > 0 and gap <= radius then
          repeller_cloud_fire(self, other, m, -a.dir)
          fired = true
        end
        j = j + 1
      end
      j = 0
      while j < gm.array_length(global.boars) and not fired do
        local b = global.boars[j]
        local gap = (m.x - repeller_animal_front(self, other, b.x, b.dir, breach)) * b.dir
        if not gm.bool(b.fleeing) then
          if gap > 0 and gap <= radius then
            repeller_cloud_fire(self, other, m, -b.dir)
            fired = true
          end
        end
        j = j + 1
      end
    end
    i = i + 1
  end
  local drag = vis(self, other, "repeller_cloud_drag")
  local keep = gm.array()
  i = 0
  while i < gm.array_length(global.repel_clouds) do
    local c = global.repel_clouds[i]
    local front = 0
    local puffs = gm.array()
    local k = 0
    while k < gm.array_length(c.puffs) do
      local p = c.puffs[k]
      p.x = p.x + p.vx
      p.y = p.y + p.vy
      p.vx = p.vx * drag
      p.vy = p.vy * drag - 0.03
      p.size = gm.lerp(p.size, p.grow, 0.08)
      p.life = p.life - 1
      local floor_y = c.ground - p.size * 0.3
      if p.y > floor_y then
        p.y = floor_y
        local sx
        if p.vx == 0 then
          sx = c.dir
        else
          sx = p.vx
        end
        p.vx = p.vx + gm.sign(sx) * gm.abs(p.vy) * 0.5
        p.vy = gm.abs(p.vy) * -0.15
      end
      if not (p.life <= 0) then
        if not gm.bool(p.dust) and p.life > p.total * 0.35 then
          front = gm.max(front, (p.x - c.x) * c.dir + p.size * 0.5)
        end
        gm.array_push(puffs, p)
      end
      k = k + 1
    end
    c.puffs = puffs
    local specks = gm.array()
    k = 0
    while k < gm.array_length(c.specks) do
      local s = c.specks[k]
      s.x = s.x + s.vx
      s.y = s.y + s.vy
      s.vx = s.vx * 0.95
      s.vy = s.vy * 0.95 + 0.04
      s.life = s.life - 1
      if s.y > c.ground - 1 then
        s.y = c.ground - 1
        s.vx = s.vx * 0.5
        s.vy = 0
      end
      if s.life > 0 then
        gm.array_push(specks, s)
      end
      k = k + 1
    end
    c.specks = specks
    c.flash = gm.max(0, c.flash - 1)
    if front > 0 then
      local x0 = c.x
      local x1 = c.x + c.dir * front
      local lo = gm.min(x0, x1)
      local hi = gm.max(x0, x1)
      local j = 0
      while j < gm.array_length(global.animals) do
        local a = global.animals[j]
        local f = repeller_animal_front(self, other, a.x, a.dir, hreach)
        if a.dir == -c.dir and f >= lo and f <= hi then
          a.dir = -a.dir
          tasks_stat_add(self, other, "hedgehogs", gm.undefined)
        end
        j = j + 1
      end
      j = 0
      while j < gm.array_length(global.boars) do
        local b = global.boars[j]
        local f = repeller_animal_front(self, other, b.x, b.dir, breach)
        if not gm.bool(b.fleeing) and b.dir == -c.dir and f >= lo and f <= hi then
          b.dir = -b.dir
          b.fleeing = true
        end
        j = j + 1
      end
    end
    if gm.array_length(c.puffs) > 0 or gm.array_length(c.specks) > 0 then
      gm.array_push(keep, c)
    end
    i = i + 1
  end
  global.repel_clouds = keep
end

-- gml_Script_repeller_soft_sprite @ 0x1400f19c0
function repeller_soft_sprite(self, other, size, hardness)
  local r = size * 0.5
  local buf = gm.buffer_create(size * size * 4, 0, 1)
  local y = 0
  while y < size do
    local x = 0
    while x < size do
      local d = gm.point_distance(x + 0.5, y + 0.5, r, r) / r
      local a = gm.power(gm.max(0, 1 - d), hardness)
      gm.buffer_write(buf, 1, 255)
      gm.buffer_write(buf, 1, 255)
      gm.buffer_write(buf, 1, 255)
      gm.buffer_write(buf, 1, gm.round(a * 255))
      x = x + 1
    end
    y = y + 1
  end
  local surf = gm.surface_create(size, size)
  gm.buffer_set_surface(buf, surf, 0)
  gm.buffer_delete(buf)
  local spr = gm.sprite_create_from_surface(surf, 0, 0, size, size, false, false, r, r)
  gm.surface_free(surf)
  return spr
end

-- gml_Script_repeller_cloud_art_init @ 0x1400f2d90
function repeller_cloud_art_init(self, other)
  if gm.variable_global_exists("repel_flash_sprite") and gm.sprite_exists(global.repel_flash_sprite) then
    return
  end
  global.repel_flash_sprite = repeller_soft_sprite(self, other, 256, 2.2)
  global.repel_puff_sprite = repeller_soft_sprite(self, other, 64, 1.4)
end

-- gml_Script_repeller_cloud_draw_body @ 0x1400f31f0
function repeller_cloud_draw_body(self, other, c, vx, vy, s)
  local spr = global.repel_puff_sprite
  local half = gm.sprite_get_width(spr) * 0.5
  local alpha = vis(self, other, "repeller_cloud_alpha")
  local shade = 11054250
  local n = gm.array_length(c.puffs)
  local i = 0
  while i < n do
    local p = c.puffs[i]
    local a = alpha * gm.min(1, p.life / p.total / 0.5) * 0.45
    local sc = p.size * 0.62 / half * s
    gm.draw_sprite_ext(spr, 0, (p.x + 2 - vx) * s, (p.y + p.size * 0.18 - vy) * s, sc, sc, 0,
      gm.merge_colour(p.colour, shade, 0.35), a)
    i = i + 1
  end
  i = 0
  while i < n do
    local p = c.puffs[i]
    local a = alpha * gm.min(1, p.life / p.total / 0.5)
    local sc = p.size * 0.5 / half * s
    gm.draw_sprite_ext(spr, 0, (p.x - vx) * s, (p.y - p.size * 0.08 - vy) * s, sc, sc, 0, p.colour, a)
    i = i + 1
  end
  local px = gm.max(1, 2 * s)
  i = 0
  while i < gm.array_length(c.specks) do
    local k = c.specks[i]
    local sx = (k.x - vx) * s
    local sy = (k.y - vy) * s
    local a = gm.min(1, k.life / k.total * 2) * 0.8
    gm.draw_sprite_ext(global.stinky_pixel, 0, gm.floor(sx), gm.floor(sy), px, px, 0, 16777215, a)
    i = i + 1
  end
end

-- gml_Script_repeller_cloud_draw_flash @ 0x1400f5f70
function repeller_cloud_draw_flash(self, other, c, vx, vy, s)
  if c.flash <= 0 then
    return
  end
  local spr = global.repel_flash_sprite
  local half = gm.sprite_get_width(spr) * 0.5
  local t = 1 - c.flash / gm.max(1, vis(self, other, "repeller_cloud_flash_frames"))
  local rad = gm.lerp(30, vis(self, other, "repeller_cloud_flash_px"), gm.sqrt(t)) * s
  local a = 1 - t * t
  local sx = (c.x - vx) * s
  local sy = (c.y - vy) * s
  local i = 0
  while i < gm.max(1, vis(self, other, "repeller_cloud_flash_layers")) do
    local halo = rad * 1.8 / half
    gm.draw_sprite_ext(spr, 0, sx, sy, halo, halo, 0, 16777215, a * 0.6)
    local core_x = rad / half
    local core_y = rad * 0.85 / half
    gm.draw_sprite_ext(spr, 0, sx + c.dir * rad * 0.3, sy, core_x, core_y, 0, 16777215, a)
    local hot = rad * 0.45 / half
    gm.draw_sprite_ext(spr, 0, sx, sy, hot, hot, 0, 16777215, a)
    i = i + 1
  end
end

-- gml_Script_repeller_cloud_surface @ 0x1400f7c70
function repeller_cloud_surface(self, other, name, w, h)
  local surf
  if gm.variable_global_exists(name) then
    surf = gm.variable_global_get(name)
  else
    surf = -1
  end
  if gm.surface_exists(surf) and gm.surface_get_width(surf) == w and gm.surface_get_height(surf) == h then
    return surf
  end
  if gm.surface_exists(surf) then
    gm.surface_free(surf)
  end
  surf = gm.surface_create(w, h)
  gm.variable_global_set(name, surf)
  return surf
end

-- gml_Script_repeller_clouds_draw @ 0x1400f8480
function repeller_clouds_draw(self, other)
  if gm.array_length(global.repel_clouds) == 0 then
    return
  end
  local v = squirrel_view(self, other)
  local shown = gm.array()
  local i = 0
  while i < gm.array_length(global.repel_clouds) do
    local c = global.repel_clouds[i]
    if c.x >= v.x - 500 and c.x <= v.x + v.w + 500 then
      gm.array_push(shown, c)
    end
    i = i + 1
  end
  if gm.array_length(shown) == 0 then
    return
  end
  stinky_art_init(self, other)
  repeller_cloud_art_init(self, other)
  local px = gm.max(1, gm.round(vis(self, other, "repeller_cloud_pixel")))
  local inv = 1 / px
  local ox = gm.floor(v.x / px) * px
  local oy = gm.floor(v.y / px) * px
  local sw = gm.ceil(v.w / px) + 2
  local sh = gm.ceil(v.h / px) + 2
  local filt = gm.gpu_get_tex_filter()
  local body = repeller_cloud_surface(self, other, "repel_body_surface", sw, sh)
  local flash = repeller_cloud_surface(self, other, "repel_flash_surface", sw, sh)
  if not (gm.surface_exists(body) and gm.surface_exists(flash)) then
    return
  end
  gm.surface_set_target(body)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_tex_filter(true)
  gm.gpu_set_blendmode_ext_sepalpha(5, 6, 2, 6)
  i = 0
  while i < gm.array_length(shown) do
    repeller_cloud_draw_body(self, other, shown[i], ox, oy, inv)
    i = i + 1
  end
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local any_flash = false
  i = 0
  while i < gm.array_length(shown) do
    if not any_flash then
      any_flash = shown[i].flash > 0
    else
      any_flash = true
    end
    i = i + 1
  end
  if any_flash then
    gm.surface_set_target(flash)
    gm.draw_clear_alpha(0, 1)
    gm.gpu_set_blendmode(1)
    i = 0
    while i < gm.array_length(shown) do
      repeller_cloud_draw_flash(self, other, shown[i], ox, oy, inv)
      i = i + 1
    end
    gm.gpu_set_blendmode(0)
    gm.surface_reset_target()
  end
  gm.gpu_set_tex_filter(false)
  gm.gpu_set_blendmode_ext(2, 6)
  gm.draw_surface_ext(body, ox, oy, px, px, 0, 16777215, 1)
  if any_flash then
    gm.gpu_set_blendmode(1)
    gm.draw_surface_ext(flash, ox, oy, px, px, 0, 16777215, 1)
  end
  gm.gpu_set_blendmode(0)
  gm.gpu_set_tex_filter(filt)
end

-- gml_Script_squirrel_frame_w @ 0x1400faf40
function squirrel_frame_w(self, other)
  return gm.sprite_get_width(squirrel) / 9
end

-- gml_Script_squirrel_frame_h @ 0x1400fb180
function squirrel_frame_h(self, other)
  return gm.sprite_get_height(squirrel)
end

-- gml_Script_squirrel_offset @ 0x1400fb310
function squirrel_offset(self, other, face, angle, px, py)
  local s = vis(self, other, "squirrel_scale")
  local dx = (px - vis(self, other, "squirrel_body_x")) * face * s
  local dy = (py - vis(self, other, "squirrel_body_y")) * s
  local c = gm.dcos(angle)
  local sn = gm.dsin(angle)
  local ox = dx * c + dy * sn
  local oy = -dx * sn + dy * c
  return gm.array(ox, oy)
end

-- gml_Script_squirrel_point @ 0x1400fbe80
function squirrel_point(self, other, sq, px, py)
  local mirror = sq.mirror
  local angle = sq.angle
  local o = squirrel_offset(self, other, mirror, angle, px, py)
  local x = sq.x + o[0]
  local y = sq.y + o[1]
  return gm.array(x, y)
end

-- gml_Script_squirrel_draw @ 0x1400fc4e0
function squirrel_draw(self, other, sq)
  squirrel_frames_build(self, other)
  local s = vis(self, other, "squirrel_scale")
  gm.draw_sprite_ext(global.squirrel_frames, sq.frame, sq.x, sq.y, sq.mirror * s, s, sq.angle, 16777215, 1)
end

-- gml_Script____struct___74__squirrel_view__scr_life @ 0x1400fcae0
function ___struct___74__squirrel_view__scr_life(self, other, x, y, w, h)
  self.x = x
  self.y = y
  self.w = w
  self.h = h
end

-- gml_Script_squirrel_view @ 0x1400fcdb0
function squirrel_view(self, other)
  local cam = gm.view_camera[0]
  local x, y, w, h
  if cam == -1 then
    x = 0
  else
    x = gm.camera_get_view_x(cam)
  end
  if cam == -1 then
    y = 0
  else
    y = gm.camera_get_view_y(cam)
  end
  if cam == -1 then
    w = vis(self, other, "screen_width_px")
  else
    w = gm.camera_get_view_width(cam)
  end
  if cam == -1 then
    h = vis(self, other, "screen_height_px")
  else
    h = gm.camera_get_view_height(cam)
  end
  return gm.new(___struct___74__squirrel_view__scr_life, x, y, w, h)
end

-- gml_Script_mushroom_index_of @ 0x1400fd990
function mushroom_index_of(self, other, m)
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    if global.mushrooms[i] == m then
      return i
    end
    i = i + 1
  end
  return -1
end

-- gml_Script____struct___76__squirrel_trunk_edges__scr_life @ 0x1400fdc30
function ___struct___76__squirrel_trunk_edges__scr_life(self, other, left, right, origin_y, h)
  self.left = left
  self.right = right
  self.origin_y = origin_y
  self.h = h
end
