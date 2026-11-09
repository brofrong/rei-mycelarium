local gm = require("gm")

-- gml_Script_world_big_bones @ 0x1402e6820
local world_big_bones_static = { _cache = gm.undefined, _key = "" }
function world_big_bones(self, other)
  local st = world_big_bones_static
  local w = global.world
  local key = gm.string(w.start_x) .. ":" .. gm.string(w.seed)
  if not gm.is_undefined(st._cache) and st._key == key then
    return st._cache
  end
  local out = gm.array()
  st._key = key
  st._cache = out
  local sw = vis(self, other, "screen_width_px")
  local sh = vis(self, other, "screen_height_px")
  local ylo = world_deep_y(self, other) + wld(self, other, "caves_margin_px")
  local yhi = world_ocean_y(self, other) - wld(self, other, "caves_margin_px")
  local rects = world_cave_rects(self, other)
  local dir = (world_hash01(self, other, 6, 1, 406) < 0.5) and -1 or 1
  local defs = gm.array(
    gm.new(___struct___317__world_big_bones__scr_world, dir),
    gm.new(___struct___318__world_big_bones__scr_world, -dir))
  local i = 0
  while i < gm.array_length(defs) do
    local d = defs[i]
    local bw = gm.sprite_get_width(d.sprite)
    local bh = gm.sprite_get_height(d.sprite)
    local cx = w.start_x + d.side * sw * cave_range(self, other, wld(self, other, "bones_big_x"), world_hash01(self, other, 7 + i, 2, 406))
    local cy = w.surface_y + sh * cave_range(self, other, wld(self, other, "bones_big_depth"), world_hash01(self, other, 7 + i, 3, 406))
    local by = gm.floor(gm.clamp(cy - bh * 0.5, ylo, gm.max(ylo, yhi - bh)))
    local bx = gm.floor(cx - bw * 0.5)
    local pad = wld(self, other, "caves_clearance_px") * 2
    for _k = 0, 11 do
      local ok = true
      local j = 0
      while j < gm.array_length(rects) and ok do
        local r = rects[j]
        if r.id == "yellow" then
          -- skip
        else
          ok = (bx + bw + pad <= r.x1) or (bx >= r.x2 + pad) or (by + bh + pad <= r.y1) or (by >= r.y2 + pad)
        end
        j = j + 1
      end
      if ok then
        break
      end
      bx = bx + d.side * (bw + pad)
    end
    gm.array_push(out, gm.new(___struct___319__world_big_bones__scr_world, d.id, d.sprite, bx, by, bx + bw, by + bh))
    i = i + 1
  end
  return out
end

-- gml_Script_world_draw_bones @ 0x1402ea6a0
function world_draw_bones(self, other, vx, vy, vw, vh)
  local w = global.world
  local kinds = bone_kinds(self, other)
  local dense = world_dense_y(self, other)
  local deep = world_deep_y(self, other)
  local tints = gm.array(vis(self, other, "bones_tint_plain"), vis(self, other, "bones_tint_clay"), vis(self, other, "bones_tint_deep"))
  local c0 = gm.floor((vx - 600) / w.chunk_w)
  local c1 = gm.floor((vx + vw) / w.chunk_w)
  local c = c0
  while c <= c1 do
    local list = world_get_chunk(self, other, c).bones
    if gm.is_undefined(list) then
      list = gm.array()
    end
    local k = 0
    while k < gm.array_length(list) do
      local b = list[k]
      if b.x + b.w < vx or b.x > vx + vw or b.y + b.h < vy or b.y > vy + vh then
        -- continue
      else
        local cy = b.y + b.h * 0.5
        local tint = tints[(cy >= deep) and 2 or ((cy >= dense) and 1 or 0)]
        local dx
        if b.flip >= 0 then
          dx = b.x
        else
          dx = b.x + b.w
        end
        gm.draw_sprite_ext(kinds[b.kind].sprite, 0, dx, b.y, b.flip, 1, 0, tint, 1)
      end
      k = k + 1
    end
    c = c + 1
  end
  local big = world_big_bones(self, other)
  local i = 0
  while i < gm.array_length(big) do
    local bb = big[i]
    if bb.x2 < vx or bb.x1 > vx + vw or bb.y2 < vy or bb.y1 > vy + vh then
      -- continue
    else
      gm.draw_sprite_ext(bb.sprite, 0, bb.x1, bb.y1, 1, 1, 0, vis(self, other, "bones_tint_deep"), 1)
    end
    i = i + 1
  end
  relics_wrap_draw_visible(self, other, vx, vy, vx + vw, vy + vh)
end

-- gml_Script____struct___321_____struct___320__relic_def__scr_world @ 0x1402ed0f0
function ___struct___321_____struct___320__relic_def__scr_world(self, other, w, feet)
  self.id = "t_rex"
  self.sprite = spr_skeleton_T_rex
  self.left = 0
  self.w = w
  self.feet = feet
end

-- gml_Script____struct___322_____struct___320__relic_def__scr_world @ 0x1402ed3b0
function ___struct___322_____struct___320__relic_def__scr_world(self, other, w, feet)
  self.id = "ufo"
  self.sprite = spr_UFO
  self.left = 0
  self.w = w
  self.feet = feet
end

-- gml_Script____struct___320__relic_def__scr_world @ 0x1402ed670
function ___struct___320__relic_def__scr_world(self, other, trex_w, trex_feet, ufo_w, ufo_feet)
  self.t_rex = gm.new(___struct___321_____struct___320__relic_def__scr_world, trex_w, trex_feet)
  self.ufo = gm.new(___struct___322_____struct___320__relic_def__scr_world, ufo_w, ufo_feet)
end

-- gml_Script_relic_def @ 0x1402edbb0
local relic_def_static = { _defs = gm.undefined }
function relic_def(self, other, id)
  local st = relic_def_static
  if gm.is_undefined(st._defs) then
    st._defs = gm.new(___struct___320__relic_def__scr_world,
      gm.sprite_get_width(spr_skeleton_T_rex), gm.array(),
      gm.sprite_get_width(spr_UFO), gm.array())
  end
  return gm.variable_struct_get(st._defs, id)
end

-- gml_Script____struct___323__world_relics__scr_world @ 0x1402ee430
function ___struct___323__world_relics__scr_world(self, other, id, relic, def, x1, y1, x2, y2)
  self.id = id
  self.relic = relic
  self.def = def
  self.k = 1
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_world_relics @ 0x1402ee820
local world_relics_static = { _for = gm.undefined, _out = gm.array() }
function world_relics(self, other)
  local st = world_relics_static
  local bones = world_big_bones(self, other)
  if st._for == bones then
    return st._out
  end
  st._for = bones
  st._out = gm.array()
  local i = 0
  while i < gm.array_length(bones) do
    local b = bones[i]
    local d = relic_def(self, other, b.id)
    if not gm.is_undefined(d) then
      gm.array_push(st._out, gm.new(___struct___323__world_relics__scr_world, "relic_" .. b.id, b.id, d, b.x1, b.y1, b.x2, b.y2))
    end
    i = i + 1
  end
  return st._out
end

-- gml_Script_relic_taken @ 0x1402ef690
function relic_taken(self, other, r)
  local res = false
  if gm.variable_global_exists("relic_taken") then
    local v = gm.variable_struct_get(global.relic_taken, r.relic)
    if gm.is_undefined(v) then
      v = 0
    end
    res = gm.bool(v)
  end
  return res
end

-- gml_Script_relic_wrapped @ 0x1402efaa0
function relic_wrapped(self, other, id)
  if gm.variable_global_exists("relic_taken") then
    local v = gm.variable_struct_get(global.relic_taken, id)
    if gm.is_undefined(v) then
      v = 0
    end
    if gm.bool(v) then
      local wr
      if gm.variable_global_exists("cave_wrap") then
        -- TODO(port): value comes from inlined helper FUN_1402f02e0 (not decompiled); assumed global.cave_wrap[$ id]
        wr = gm.variable_struct_get(global.cave_wrap, id)
      else
        wr = gm.undefined
      end
      if not gm.is_undefined(wr) then
        return (gm.current_time - wr.born) >= vis(self, other, "cave_wrap_ms")
      end
      return true
    end
  end
  return false
end

-- gml_Script____struct___324__relics_touch_at__scr_world @ 0x1402f0520
function ___struct___324__relics_touch_at__scr_world(self, other, born, from_x, from_y)
  self.born = born
  self.from_x = from_x
  self.from_y = from_y
end

-- gml_Script_relics_touch_at @ 0x1402f0750
function relics_touch_at(self, other, x, y)
  local rs = world_relics(self, other)
  local pad = vis(self, other, "cave_touch_px")
  local i = 0
  while i < gm.array_length(rs) do
    local r = rs[i]
    if x < r.x1 - pad or x > r.x2 + pad or y < r.y1 - pad or y > r.y2 + pad or relic_taken(self, other, r) then
      -- continue
    else
      local hit = cave_solid_at(self, other, r, x, y)
      local a = 0
      while a < 360 and not gm.bool(hit) do
        if gm.bool(cave_solid_at(self, other, r, x + gm.lengthdir_x(pad, a), y + gm.lengthdir_y(pad, a))) then
          hit = true
        else
          hit = gm.bool(cave_solid_at(self, other, r, x + gm.lengthdir_x(pad * 0.5, a), y + gm.lengthdir_y(pad * 0.5, a)))
        end
        a = a + 45
      end
      if gm.bool(hit) then
        gm.variable_struct_set(global.relic_taken, r.relic, true)
        gm.variable_struct_set(global.cave_flash, r.id, gm.current_time)
        gm.variable_struct_set(global.cave_wrap, r.id,
          gm.new(___struct___324__relics_touch_at__scr_world, gm.current_time, (x - r.x1) / r.k, (y - r.y1) / r.k))
        water_touch_sound(self, other, x, y)
      end
    end
    i = i + 1
  end
end

-- gml_Script_relics_wrap_draw_visible @ 0x1402f2220
function relics_wrap_draw_visible(self, other, x1, y1, x2, y2)
  local rs = world_relics(self, other)
  local t = gm.current_time
  local i = 0
  while i < gm.array_length(rs) do
    local r = rs[i]
    if not gm.bool(relic_taken(self, other, r)) or r.x2 < x1 or r.x1 > x2 or r.y2 < y1 or r.y1 > y2 then
      -- continue
    else
      cave_wrap_draw(self, other, r, t)
    end
    i = i + 1
  end
end

-- gml_Script_world_cave_hits @ 0x1402f28c0
function world_cave_hits(self, other, x1, y1, x2, y2, pad)
  if gm.is_undefined(pad) then
    pad = 0
  end
  local zones = world_clear_zones(self, other)
  local m = wld(self, other, "caves_clearance_px") + pad
  local i = 0
  while i < gm.array_length(zones) do
    local z = zones[i]
    if x2 > z.x1 - m and x1 < z.x2 + m and y2 > z.y1 - m and y1 < z.y2 + m then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___326__caves_mask_init__scr_world @ 0x1402f3260
function ___struct___326__caves_mask_init__scr_world(self, other, buffer, w, h, outline, stems)
  self.buffer = buffer
  self.w = w
  self.h = h
  self.outline = outline
  self.stems = stems
end

-- gml_Script____struct___327__caves_mask_init__scr_world @ 0x1402f3550
function ___struct___327__caves_mask_init__scr_world(self, other, buffer, w, h, outline, stems)
  self.buffer = buffer
  self.w = w
  self.h = h
  self.outline = outline
  self.stems = stems
end

-- gml_Script_caves_mask_init @ 0x1402f3840
function caves_mask_init(self, other)
  if gm.variable_global_exists("cave_masks") and gm.is_struct(global.cave_masks) then
    return
  end
  local masks = gm.struct({})
  local defs = cave_defs(self, other)
  local i = 0
  while i < gm.array_length(defs) do
    local c = defs[i]
    local h = gm.sprite_get_height(c.sprite)
    local surf = gm.surface_create(c.w, h)
    gm.surface_set_target(surf)
    gm.draw_clear_alpha(0, 0)
    gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_zero)
    gm.draw_sprite_part(c.sprite, 0, c.left, 0, c.w, h, 0, 0)
    gm.gpu_set_blendmode(gm.bm_normal)
    gm.surface_reset_target()
    local buf = gm.buffer_create(c.w * h * 4, 0, 1)
    gm.buffer_get_surface(buf, surf, 0)
    gm.surface_free(surf)
    local m = gm.new(___struct___326__caves_mask_init__scr_world, buf, c.w, h, gm.array(), gm.array())
    cave_outline_build(self, other, m, c)
    gm.variable_struct_set(masks, c.id, m)
    i = i + 1
  end
  global.cave_masks = masks
  local relics = gm.array("t_rex", "ufo")
  i = 0
  while i < gm.array_length(relics) do
    local d = relic_def(self, other, relics[i])
    local h = gm.sprite_get_height(d.sprite)
    local surf = gm.surface_create(d.w, h)
    if not gm.surface_exists(surf) then
      -- continue
    else
      gm.surface_set_target(surf)
      gm.draw_clear_alpha(0, 0)
      gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_zero)
      gm.draw_sprite_part(d.sprite, 0, 0, 0, d.w, h, 0, 0)
      gm.gpu_set_blendmode(gm.bm_normal)
      gm.surface_reset_target()
      local buf = gm.buffer_create(d.w * h * 4, 0, 1)
      gm.buffer_get_surface(buf, surf, 0)
      gm.surface_free(surf)
      local m = gm.new(___struct___327__caves_mask_init__scr_world, buf, d.w, h, gm.array(), gm.array())
      cave_outline_build(self, other, m, d)
      gm.variable_struct_set(masks, d.id, m)
    end
    i = i + 1
  end
end

-- gml_Script_cave_hash @ 0x1402f5d00
function cave_hash(self, other, a, b, c)
  local v = math.sin(a * 12.9898 + b * 78.233 + c * 37.719) * 43758.5453
  return v - gm.floor(v)
end

-- gml_Script_cave_branch_points @ 0x1402f6230
function cave_branch_points(self, other, start, dir, len, seed)
  local pts = gm.array(gm.array(start[0], start[1]))
  local n = gm.max(2, gm.ceil(len / 3))
  local px = start[0]
  local py = start[1]
  local d = dir
  local i = 1
  while i <= n do
    d = d + (cave_hash(self, other, seed, i, 9) - 0.5) * 30
    px = px + gm.lengthdir_x(len / n, d)
    py = py + gm.lengthdir_y(len / n, d)
    gm.array_push(pts, gm.array(px, py))
    i = i + 1
  end
  return pts
end

-- gml_Script____struct___328__cave_outline_build__scr_world @ 0x1402f6fd0
function ___struct___328__cave_outline_build__scr_world(self, other, at, points)
  self.at = at
  self.points = points
end

-- gml_Script____struct___329__cave_outline_build__scr_world @ 0x1402f7190
function ___struct___329__cave_outline_build__scr_world(self, other, at, points)
  self.at = at
  self.delay = 0.5
  self.points = points
end

-- gml_Script_cave_outline_build @ 0x1402f73a0
function cave_outline_build(self, other, m, c)
  local w = m.w
  local h = m.h
  local buf = m.buffer
  local sx = 0
  local sy = 0
  local n = 0
  local y = 0
  while y < h do
    local x = 0
    while x < w do
      if gm.buffer_peek(buf, (y * w + x) * 4 + 3, gm.buffer_u8) >= 250 then
        sx = sx + x
        sy = sy + y
        n = n + 1
      end
      x = x + 3
    end
    y = y + 3
  end
  local cx = sx / gm.max(1, n)
  local cy = sy / gm.max(1, n)
  local maxr = gm.point_distance(0, 0, w, h)
  local radii = gm.array()
  local a = 0
  while a < 360 do
    local dx = gm.dcos(a)
    local dy = -gm.dsin(a)
    local r = 0
    local rr = maxr
    while rr > 0 do
      local px = gm.round(cx + dx * rr)
      local py = gm.round(cy + dy * rr)
      if px >= 0 and py >= 0 and px < w and py < h
        and gm.buffer_peek(buf, (py * w + px) * 4 + 3, gm.buffer_u8) >= 250 then
        r = rr
        while r + 1 < maxr do
          local qx = gm.round(cx + dx * (r + 1))
          local qy = gm.round(cy + dy * (r + 1))
          if qx < 0 or qy < 0 or qx >= w or qy >= h
            or gm.buffer_peek(buf, (qy * w + qx) * 4 + 3, gm.buffer_u8) < 250 then
            break
          end
          r = r + 1
        end
        break
      end
      rr = rr - 3
    end
    gm.array_push(radii, r)
    a = a + 1
  end
  local out = vis(self, other, "cave_wrap_out_px")
  local wob = vis(self, other, "cave_wrap_wobble_px")
  a = 0
  while a < 360 do
    local rs = (radii[gm.mod(a + 358, 360)] + radii[gm.mod(a + 359, 360)] + radii[a] * 2
      + radii[gm.mod(a + 1, 360)] + radii[gm.mod(a + 2, 360)]) / 6
    rs = rs + (out + wob * (math.sin(a * 0.13 + 1.7) * 0.4 + math.sin(a * 0.37) * 0.6))
    gm.array_push(m.outline, gm.array(cx + gm.dcos(a) * rs, cy - gm.dsin(a) * rs))
    a = a + 1
  end
  m.branches = gm.array()
  local seed = gm.string_length(c.id) * 131 + gm.ord(gm.string_char_at(c.id, 1))
  local every = gm.max(3, vis(self, other, "cave_branch_every"))
  local b = 0
  while b < 360 do
    local at = gm.mod(b + gm.floor(cave_hash(self, other, seed, b, 1) * every), 360)
    local len = gm.lerp(vis(self, other, "cave_branch_min_px"), vis(self, other, "cave_branch_max_px"), cave_hash(self, other, seed, b, 2))
    local dir = at + (cave_hash(self, other, seed, b, 3) - 0.5) * 2 * vis(self, other, "cave_branch_spread_deg")
    local pts = cave_branch_points(self, other, m.outline[at], dir, len, seed + b)
    gm.array_push(m.branches, gm.new(___struct___328__cave_outline_build__scr_world, at, pts))
    if cave_hash(self, other, seed, b, 4) < vis(self, other, "cave_branch_fork_chance") then
      local fp = pts[gm.floor(gm.array_length(pts) * 0.5)]
      local side = (cave_hash(self, other, seed, b, 5) < 0.5) and -1 or 1
      gm.array_push(m.branches, gm.new(___struct___329__cave_outline_build__scr_world, at,
        cave_branch_points(self, other, fp, dir + side * gm.lerp(25, 50, cave_hash(self, other, seed, b, 6)), len * 0.55, seed + b + 7)))
    end
    b = b + every
  end
  local f = 0
  while f < gm.array_length(c.feet) do
    local ft = c.feet[f]
    local best = gm.undefined
    local bd = gm.infinity
    local k = 0
    while k < 360 do
      local p = m.outline[k]
      if p[1] <= ft[1] then
        -- continue
      else
        local d = gm.abs(p[0] - ft[0]) * 3 + p[1]
        if d < bd then
          bd = d
          best = p
        end
      end
      k = k + 1
    end
    if gm.is_undefined(best) then
      -- continue
    else
      local stem = gm.array()
      local steps = gm.max(4, gm.ceil(gm.point_distance(best[0], best[1], ft[0], ft[1]) / 4))
      local ph = f * 1.9
      local j = 0
      while j <= steps do
        local t = j / steps
        local sway = vis(self, other, "cave_stem_sway_px") * math.sin(t * 3.141592653589793 * 2.5 + ph) * math.sin(t * 3.141592653589793)
        gm.array_push(stem, gm.array(gm.lerp(best[0], ft[0], t) + sway, gm.lerp(best[1], ft[1], t)))
        j = j + 1
      end
      gm.array_push(m.stems, stem)
    end
    f = f + 1
  end
end

-- gml_Script_cave_solid_at @ 0x1402fb990
function cave_solid_at(self, other, r, x, y)
  local lx = gm.floor((x - r.x1) / r.k)
  local ly = gm.floor((y - r.y1) / r.k)
  local w = r.def.w
  local h = gm.sprite_get_height(r.def.sprite)
  if lx < 0 or ly < 0 or lx >= w or ly >= h then
    return false
  end
  if not gm.variable_global_exists("cave_masks") or not gm.is_struct(global.cave_masks) then
    return lx > w * 0.25 and lx < w * 0.75 and ly > h * 0.25 and ly < h * 0.75
  end
  local mk = gm.variable_struct_get(global.cave_masks, r.def.id)
  if gm.is_undefined(mk) then
    return false
  end
  return gm.buffer_peek(mk.buffer, (ly * mk.w + lx) * 4 + 3, gm.buffer_u8) >= 250
end

-- gml_Script_world_cave_solid_any @ 0x1402fce00
function world_cave_solid_any(self, other, x, y)
  local rects = world_cave_rects(self, other)
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    if x < r.x1 or x >= r.x2 or y < r.y1 or y >= r.y2 then
      -- continue
    elseif gm.bool(cave_solid_at(self, other, r, x, y)) then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___330__world_clear_caves__scr_world @ 0x1402fd460
function ___struct___330__world_clear_caves__scr_world(self, other, x1, y1, x2, y2)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__46074__world_clear_caves__scr_world @ 0x1402fd6f0
function anon__46074__world_clear_caves__scr_world(self, other, o)
  return o.body_x + o.body_w <= self.x1 or o.body_x >= self.x2 or o.body_y + o.body_h <= self.y1 or o.body_y >= self.y2
end

-- gml_Script_anon__46304__world_clear_caves__scr_world @ 0x1402fdae0
function anon__46304__world_clear_caves__scr_world(self, other, o)
  return o.x < self.x1 or o.x > self.x2 or o.y < self.y1 or o.y > self.y2
end

-- gml_Script____struct___331__world_clear_caves__scr_world @ 0x1402fdd80
function ___struct___331__world_clear_caves__scr_world(self, other, x1, y1, x2, y2)
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_anon__46678__world_clear_caves__scr_world @ 0x1402fe010
function anon__46678__world_clear_caves__scr_world(self, other, o)
  return o.x < self.x1 or o.x > self.x2 or o.y < self.y1 or o.y > self.y2
end

-- gml_Script_anon__46870__world_clear_caves__scr_world @ 0x1402fe2b0
function anon__46870__world_clear_caves__scr_world(self, other, o)
  return o.x + o.w <= self.x1 or o.x >= self.x2 or o.y + o.h <= self.y1 or o.y >= self.y2
end

-- gml_Script_world_clear_caves @ 0x1402fe6b0
function world_clear_caves(self, other, ch)
  local zones = world_clear_zones(self, other)
  local w = global.world
  local x0 = ch.cx * w.chunk_w
  local x1 = x0 + w.chunk_w
  local pad = wld(self, other, "caves_clearance_px")
  local i = 0
  while i < gm.array_length(zones) do
    local z = zones[i]
    local zx1 = z.x1 - pad
    local zy1 = z.y1 - pad
    local zx2 = z.x2 + pad
    local zy2 = z.y2 + pad
    if not (zx2 > x0 and zx1 < x1) then
      -- continue
    else
      local r = gm.new(___struct___330__world_clear_caves__scr_world, zx1, zy1, zx2, zy2)
      ch.lakes = gm.array_filter(ch.lakes, gm.method(r, anon__46074__world_clear_caves__scr_world))
      ch.foods = gm.array_filter(ch.foods, gm.method(r, anon__46304__world_clear_caves__scr_world))
      local sr
      if gm.variable_global_exists("stone_reach") then
        sr = global.stone_reach
      else
        sr = 48
      end
      local rs = gm.new(___struct___331__world_clear_caves__scr_world, zx1 - sr, zy1 - sr, zx2 + sr, zy2 + sr)
      ch.stones = gm.array_filter(ch.stones, gm.method(rs, anon__46678__world_clear_caves__scr_world))
      local bones = ch.bones
      if gm.is_undefined(bones) then
        bones = gm.array()
      end
      ch.bones = gm.array_filter(bones, gm.method(r, anon__46870__world_clear_caves__scr_world))
      local r0 = gm.max(0, gm.floor((zy1 - w.surface_y) / w.tile))
      local r1 = gm.min(ch.rows - 1, gm.floor((zy2 - w.surface_y) / w.tile))
      local c0 = gm.max(ch.cx * w.cols, gm.floor(zx1 / w.tile))
      local c1 = gm.min((ch.cx + 1) * w.cols - 1, gm.floor(zx2 / w.tile))
      local row = r0
      while row <= r1 do
        local col = c0
        while col <= c1 do
          if world_chunk_get(self, other, ch, col, row) == 2 then
            world_chunk_set(self, other, ch, col, row, 0, 0)
          end
          col = col + 1
        end
        row = row + 1
      end
      local cut = gm.struct({})
      local cells = ch.stone_cells
      local k = 0
      while k < gm.array_length(cells) do
        local s = cells[k]
        local sx = s.col * 48
        local sy = s.row * 48
        if sx + 48 <= zx1 or sx >= zx2 or sy + 48 <= zy1 or sy >= zy2 then
          -- continue
        else
          local key = stone_cell_key(self, other, s.col, s.row)
          if not gm.is_undefined(gm.variable_struct_get(cut, key)) then
            -- continue
          else
            local dep = gm.array(s)
            local seen = gm.struct({})
            local q = gm.array(s)
            gm.variable_struct_set(seen, key, true)
            while gm.array_length(q) > 0 and gm.array_length(dep) <= wld(self, other, "caves_clear_deposit_max_cells") do
              local cur = gm.array_pop(q)
              local dirs = gm.array(gm.array(1, 0), gm.array(-1, 0), gm.array(0, 1), gm.array(0, -1))
              local d = 0
              while d < 4 do
                local nk = stone_cell_key(self, other, cur.col + dirs[d][0], cur.row + dirs[d][1])
                local nb = gm.variable_struct_get(ch.stone_map, nk)
                if gm.is_undefined(nb) or not gm.is_undefined(gm.variable_struct_get(seen, nk)) then
                  -- continue
                else
                  gm.variable_struct_set(seen, nk, true)
                  gm.array_push(dep, nb)
                  gm.array_push(q, nb)
                end
                d = d + 1
              end
            end
            if gm.array_length(dep) > wld(self, other, "caves_clear_deposit_max_cells") then
              dep = gm.array(s)
            end
            local j = 0
            while j < gm.array_length(dep) do
              gm.variable_struct_set(cut, stone_cell_key(self, other, dep[j].col, dep[j].row), true)
              j = j + 1
            end
          end
        end
        k = k + 1
      end
      local names = gm.variable_struct_get_names(cut)
      if gm.array_length(names) > 0 then
        local keep = gm.array()
        k = 0
        while k < gm.array_length(cells) do
          local key = stone_cell_key(self, other, cells[k].col, cells[k].row)
          if gm.is_undefined(gm.variable_struct_get(cut, key)) then
            gm.array_push(keep, cells[k])
          end
          k = k + 1
        end
        k = 0
        while k < gm.array_length(names) do
          gm.variable_struct_remove(ch.stone_map, names[k])
          k = k + 1
        end
        ch.stone_cells = keep
        stone_redress_after_cut(self, other, ch, cut)
      end
    end
    i = i + 1
  end
end

-- gml_Script_perk_has @ 0x140302420
function perk_has(self, other, id)
  local res = false
  if gm.variable_global_exists("cave_perks") then
    local v = gm.variable_struct_get(global.cave_perks, id)
    if gm.is_undefined(v) then
      v = 0
    end
    res = gm.bool(v)
  end
  return res
end

-- gml_Script_cave_taken @ 0x1403027e0
function cave_taken(self, other, c)
  local res = false
  if gm.variable_global_exists("cave_taken") then
    local v = gm.variable_struct_get(global.cave_taken, c.id)
    if gm.is_undefined(v) then
      v = 0
    end
    res = gm.bool(v)
  end
  return res
end

-- gml_Script_caves_touch_at @ 0x140302be0
function caves_touch_at(self, other, x, y)
  local rects = world_cave_rects(self, other)
  local pad = vis(self, other, "cave_touch_px")
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    if gm.bool(cave_taken(self, other, r)) then
      -- continue
    elseif x < r.x1 - pad or x > r.x2 + pad or y < r.y1 - pad or y > r.y2 + pad then
      -- continue
    else
      local hit = cave_solid_at(self, other, r, x, y)
      local a = 0
      while a < 360 and not gm.bool(hit) do
        if gm.bool(cave_solid_at(self, other, r, x + gm.lengthdir_x(pad, a), y + gm.lengthdir_y(pad, a))) then
          hit = true
        else
          hit = gm.bool(cave_solid_at(self, other, r, x + gm.lengthdir_x(pad * 0.5, a), y + gm.lengthdir_y(pad * 0.5, a)))
        end
        a = a + 45
      end
      if gm.bool(hit) then
        cave_take(self, other, r, x, y)
      end
    end
    i = i + 1
  end
end

-- gml_Script____struct___334__cave_take__scr_world @ 0x140303f60
function ___struct___334__cave_take__scr_world(self, other, born, from_x, from_y)
  self.born = born
  self.from_x = from_x
  self.from_y = from_y
end

-- gml_Script____struct___335__cave_take__scr_world @ 0x1403041a0
function ___struct___335__cave_take__scr_world(self, other, text, x, y, born)
  self.text = text
  self.x = x
  self.y = y
  self.born = born
end

-- gml_Script____struct___336__cave_take__scr_world @ 0x140304470
function ___struct___336__cave_take__scr_world(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_cave_take @ 0x140304670
function cave_take(self, other, r, x, y)
  if gm.is_undefined(x) then
    x = gm.undefined
  end
  if gm.is_undefined(y) then
    y = gm.undefined
  end
  if gm.bool(cave_taken(self, other, r)) then
    return
  end
  local id = r.def.id
  local first = not gm.bool(perk_has(self, other, id))
  gm.variable_struct_set(global.cave_taken, r.id, true)
  gm.variable_struct_set(global.cave_perks, id, true)
  if id == "yellow" and first then
    local k = 1 - bal(self, other, "perk_yellow_time_cut")
    local i = 0
    while i < gm.array_length(global.mushrooms) do
      local m = global.mushrooms[i]
      if gm.variable_struct_exists(m, "timer") then
        m.timer = m.timer * k
      end
      i = i + 1
    end
  end
  gm.variable_struct_set(global.cave_flash, r.id, gm.current_time)
  local fx = x
  if gm.is_undefined(fx) then
    fx = (r.x1 + r.x2) * 0.5
  end
  local fy = y
  if gm.is_undefined(fy) then
    fy = r.y1
  end
  gm.variable_struct_set(global.cave_wrap, r.id,
    gm.new(___struct___334__cave_take__scr_world, gm.current_time, (fx - r.x1) / r.k, (fy - r.y1) / r.k))
  gm.array_push(global.cave_popups,
    gm.new(___struct___335__cave_take__scr_world, loc(self, other, "perk_" .. id), (r.x1 + r.x2) * 0.5, r.y1, gm.current_time))
  if id == "blue" then
    global.cave_burst_pending = gm.new(___struct___336__cave_take__scr_world, (r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5)
  end
  if id == "purple" and first and gm.instance_exists(obj_mycelium_controller) then
    gm.with(self, obj_mycelium_controller, function(self, other)
      self.myc_glow_set(self, other, true)
    end)
  end
  local sx = x
  if gm.is_undefined(sx) then
    sx = (r.x1 + r.x2) * 0.5
  end
  local sy = y
  if gm.is_undefined(sy) then
    sy = (r.y1 + r.y2) * 0.5
  end
  water_touch_sound(self, other, sx, sy)
end

-- gml_Script_perk_time_mult @ 0x1403073c0
function perk_time_mult(self, other)
  if gm.bool(perk_has(self, other, "yellow")) then
    return 1 - bal(self, other, "perk_yellow_time_cut")
  end
  return 1
end

-- gml_Script_perk_cost_mult @ 0x140307660
function perk_cost_mult(self, other)
  if gm.bool(perk_has(self, other, "green")) then
    return 1 - bal(self, other, "perk_green_discount")
  end
  return 1
end

-- gml_Script_world_draw_cave @ 0x140307900
function world_draw_cave(self, other, vx, vy, vw, vh)
  caves_mask_init(self, other)
  local rects = world_cave_rects(self, other)
  local t = gm.current_time
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    local d = r.def
    if r.x2 < vx or r.x1 > vx + vw or r.y2 < vy or r.y1 > vy + vh then
      -- continue
    else
      local h = gm.sprite_get_height(d.sprite)
      local k = r.k
      gm.draw_sprite_part_ext(d.sprite, 0, d.left, 0, d.w, h, r.x1, r.y1, k, k, 16777215, 1)
      if not gm.bool(cave_taken(self, other, r)) then
        -- continue
      else
        local fl = gm.variable_struct_get(global.cave_flash, r.id)
        if gm.is_undefined(fl) then
          fl = -100000
        end
        local age = t - fl
        local f = gm.max(0, 1 - age / gm.max(1, vis(self, other, "cave_flash_ms")))
        local glow = vis(self, other, "cave_glow_alpha") * (math.sin(t / 700 + i) * 0.25 + 0.75)
        local a = gm.max(glow, f * vis(self, other, "cave_flash_alpha"))
        gm.gpu_set_blendmode(gm.bm_add)
        gm.draw_sprite_part_ext(d.sprite, 0, d.left, 0, d.w, h, r.x1, r.y1, k, k, 16777215, a)
        gm.gpu_set_blendmode(gm.bm_normal)
        cave_wrap_draw(self, other, r, t)
      end
    end
    i = i + 1
  end
end

-- gml_Script____struct___337__caves_wrap_hit__scr_world @ 0x140309950
function ___struct___337__caves_wrap_hit__scr_world(self, other, x, y, dist)
  self.hit = false
  self.x = x
  self.y = y
  self.dist = dist
end
