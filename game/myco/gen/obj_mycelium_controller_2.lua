local gm = require("gm")

-- gml_Script____struct___491__anon__28384__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066f910
function ___struct___491__anon__28384__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, vx, vy, weight, life, phase)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if vx == nil then vx = gm.undefined end
  if vy == nil then vy = gm.undefined end
  if weight == nil then weight = gm.undefined end
  if life == nil then life = gm.undefined end
  if phase == nil then phase = gm.undefined end
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.weight = weight
  self.age = 0
  self.life = life
  self.phase = phase
  self.show = true
end

-- gml_Script_anon__28384__gml_Object_obj_mycelium_controller_Create_0 @ 0x14066fdb0
function anon__28384__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, count, power)
  local weight = bal(self, other, "spore_base_weight") * gm.max(0.15, 1.0 - upg_bonus(self, other, "spore_lightness"))
  local i = 0
  while i < count do
    gm.array_push(self.myc_spores, gm.new(___struct___491__anon__28384__gml_Object_obj_mycelium_controller_Create_0,
      px + gm.random_range(-9.0, 9.0) * power,
      py + gm.random_range(-6.0, 6.0),
      gm.random_range(-0.42, 0.42) * power * 1.6 + global.wind_dir * global.wind_speed * 0.09,
      gm.random_range(-1.55, -0.35) * power,
      weight,
      gm.irandom_range(620, 920),
      gm.random(360.0)))
    i = i + 1
  end
end

-- gml_Script____struct___492__anon__29233__gml_Object_obj_mycelium_controller_Create_0 @ 0x140670b30
function ___struct___492__anon__29233__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, vx, vy, life, phase)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if vx == nil then vx = gm.undefined end
  if vy == nil then vy = gm.undefined end
  if life == nil then life = gm.undefined end
  if phase == nil then phase = gm.undefined end
  self.x = x
  self.y = y
  self.vx = vx
  self.vy = vy
  self.weight = 1.0
  self.age = 0
  self.life = life
  self.phase = phase
  self.show = true
  self.ray = true
end

-- gml_Script_anon__29233__gml_Object_obj_mycelium_controller_Create_0 @ 0x140671010
function anon__29233__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, count, rays, mult)
  local i = 0
  while i < count do
    local r = rays[gm.irandom(gm.array_length(rays) - 1)]
    local dir = r.a + gm.random_range(-2.5, 2.5)
    local spd = mult * r.power * gm.random_range(0.15, 1.0)
    gm.array_push(self.myc_spores, gm.new(___struct___492__anon__29233__gml_Object_obj_mycelium_controller_Create_0,
      px + gm.random_range(-4.0, 4.0),
      py + gm.random_range(-4.0, 4.0),
      gm.lengthdir_x(spd, dir),
      gm.lengthdir_y(spd, dir),
      gm.irandom_range(420, 700),
      gm.random(360.0)))
    i = i + 1
  end
end

-- gml_Script_anon__29888__gml_Object_obj_mycelium_controller_Create_0 @ 0x140671bc0
function anon__29888__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local keep = gm.array()
  local root_chance = self.myc_spore_root_chance(self, other)
  local i = 0
  while i < gm.array_length(self.myc_spores) do
    local s = self.myc_spores[i]
    local oy = s.y
    s.age = s.age + 1
    s.life = s.life - 1.0
    local ray = gm.variable_struct_get(s, "ray")
    if gm.is_undefined(ray) then ray = false end
    if gm.bool(ray) then
      s.vx = s.vx * 0.972
      s.vy = s.vy * 0.972 + 0.004
      s.x = s.x + (s.vx + gm.lengthdir_x(0.2, s.phase + s.age * 5))
      s.y = s.y + s.vy
      if s.life > 0 and s.y < self.myc_surface_y then
        gm.array_push(keep, s)
      end
    else
      s.vx = s.vx + global.wind_dir * global.wind_speed * 0.009
      s.vx = gm.clamp(s.vx, -2.2, 2.2)
      s.vy = s.vy + 0.014 * s.weight
      s.vy = gm.min(s.vy, 1.05 * s.weight)
      s.x = s.x + (s.vx + gm.lengthdir_x(0.36, s.phase + s.age * 5))
      s.y = s.y + s.vy
      if oy < self.myc_surface_y and s.y >= self.myc_surface_y then
        local show = gm.variable_struct_get(s, "show")
        if gm.is_undefined(show) then show = false end
        if not gm.bool(show) then
          if gm.random(1.0) < root_chance then
            self.myc_add_spore_knot(self, other, s.x)
          end
        end
      else
        if s.life > 0 and s.y < self.myc_surface_y + self.myc_tile then
          gm.array_push(keep, s)
        end
      end
    end
    i = i + 1
  end
  self.myc_spores = keep
end

-- gml_Script_anon__31845__gml_Object_obj_mycelium_controller_Create_0 @ 0x140673c00
function anon__31845__gml_Object_obj_mycelium_controller_Create_0(self, other, e)
  local cx1 = gm.floor(e.bx1 / 512.0)
  local cy1 = gm.floor(e.by1 / 512.0)
  local cx2 = gm.floor(e.bx2 / 512.0)
  local cy2 = gm.floor(e.by2 / 512.0)
  local cx = cx1
  while cx <= cx2 do
    local cy = cy1
    while cy <= cy2 do
      local key = gm.string(cx) .. "," .. gm.string(cy)
      local cell = gm.variable_struct_get(self.myc_grid, key)
      if gm.is_undefined(cell) then
        gm.variable_struct_set(self.myc_grid, key, gm.array(e))
      else
        gm.array_push(cell, e)
      end
      cy = cy + 1
    end
    cx = cx + 1
  end
  self.myc_far_max = gm.max(self.myc_far_max,
    gm.point_distance(self.x, self.y, e.bx1, e.by1),
    gm.point_distance(self.x, self.y, e.bx2, e.by1),
    gm.point_distance(self.x, self.y, e.bx1, e.by2),
    gm.point_distance(self.x, self.y, e.bx2, e.by2))
end

-- gml_Script_anon__32773__gml_Object_obj_mycelium_controller_Create_0 @ 0x140675610
function anon__32773__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.myc_grid = gm.struct({})
  self.myc_far_max = 0
  local i = 0
  while i < gm.array_length(self.myc_growths) do
    self.myc_grid_add(self, other, self.myc_growths[i])
    i = i + 1
  end
end

-- gml_Script____struct___495__anon__32963__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406759d0
function ___struct___495__anon__32963__gml_Object_obj_mycelium_controller_Create_0(self, other, hit, x, y)
  if hit == nil then hit = gm.undefined end
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.hit = hit
  self.x = x
  self.y = y
end

-- gml_Script_anon__32963__gml_Object_obj_mycelium_controller_Create_0 @ 0x140675c30
function anon__32963__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, r)
  local hit = false
  local best = r
  local hx = px
  local hy = py
  self.myc_grid_stamp = self.myc_grid_stamp + 1
  local stamp = self.myc_grid_stamp
  local cands = gm.array()
  local cx1 = gm.floor((px - r) / 512.0)
  local cy1 = gm.floor((py - r) / 512.0)
  local cx2 = gm.floor((px + r) / 512.0)
  local cy2 = gm.floor((py + r) / 512.0)
  local cx = cx1
  while cx <= cx2 do
    local cy = cy1
    while cy <= cy2 do
      local cell = gm.variable_struct_get(self.myc_grid, gm.string(cx) .. "," .. gm.string(cy))
      if not gm.is_undefined(cell) then
        local k = 0
        while k < gm.array_length(cell) do
          local e = cell[k]
          local gs = gm.variable_struct_get(e, "grid_stamp")
          if gm.is_undefined(gs) then gs = -1.0 end
          if gs ~= stamp then
            e.grid_stamp = stamp
            gm.array_push(cands, e)
          end
          k = k + 1
        end
      end
      cy = cy + 1
    end
    cx = cx + 1
  end
  local i = 0
  while i < gm.array_length(cands) do
    local g = cands[i]
    local brs = g.branches
    local j = 0
    while j < gm.array_length(brs) do
      local b = brs[j]
      local pts = b.points
      local n = gm.array_length(pts)
      if n >= 2 then
        if not (px < b.bx1 - r or px > b.bx2 + r or py < b.by1 - r or py > b.by2 + r) then
          local prog = gm.clamp((g.age - b.start) / g.duration, 0, 1.0)
          if prog > 0 then
            local lim = prog * (n - 1)
            local k = 0
            while k < n - 1 do
              local t = gm.clamp(lim - k, 0, 1.0)
              if t <= 0 then
                break
              end
              local p0 = pts[k]
              local p1 = pts[k + 1]
              local dx = (p1.x - p0.x) * t
              local dy = (p1.y - p0.y) * t
              local dd = dx * dx + dy * dy
              if dd > 0 then
                local u = gm.clamp(((px - p0.x) * dx + (py - p0.y) * dy) / dd, 0, 1.0)
                local qx = p0.x + dx * u
                local qy = p0.y + dy * u
                local d = gm.point_distance(px, py, qx, qy)
                if d <= best then
                  hit = true
                  best = d
                  hx = qx
                  hy = qy
                end
              end
              k = k + 1
            end
          end
        end
      end
      j = j + 1
    end
    i = i + 1
  end
  local w = caves_wrap_hit(self, other, px, py, best)
  if gm.bool(w.hit) then
    hit = true
    hx = w.x
    hy = w.y
  end
  return gm.new(___struct___495__anon__32963__gml_Object_obj_mycelium_controller_Create_0, hit, hx, hy)
end

-- gml_Script____struct___496__anon__36616__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067a210
function ___struct___496__anon__36616__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.x = x
  self.y = y
end

-- gml_Script____struct___497__anon__36616__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067a410
function ___struct___497__anon__36616__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.x = x
  self.y = y
end

-- gml_Script_anon__36616__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067a610
function anon__36616__gml_Object_obj_mycelium_controller_Create_0(self, other, x0, y0, dir0, len, segs, wobble, a6, width, cave, kink, a10)
  if gm.is_undefined(cave) then cave = false end
  if gm.is_undefined(kink) then kink = 0 end
  if gm.is_undefined(a10) then a10 = true end
  local pts = gm.array(gm.new(___struct___496__anon__36616__gml_Object_obj_mycelium_controller_Create_0, x0, y0))
  local perp = dir0 + 90
  local w = gm.max(2.0, width * 0.5)
  local off = 0
  local dir = dir0
  local step = len / gm.max(1.0, segs)
  local kink_chance = kink / gm.max(1.0, segs)
  local cx = x0
  local cy = y0
  local i = 1.0
  while i <= segs do
    local t = i / segs
    if kink > 0 then
      if gm.random(1.0) < kink_chance then
        local kr = gm.random_range(gm.real(vis(self, other, "strand_kink_min_deg")), gm.real(vis(self, other, "strand_kink_max_deg")))
        local kd = gm.choose(-1.0, 1.0) * kr
        local cone = vis(self, other, "strand_kink_cone_deg")
        dir = dir0 + gm.clamp(gm.angle_difference(dir + kd, dir0), -gm.real(cone), gm.real(cone))
        perp = dir + 90
      end
      cx = cx + gm.lengthdir_x(step, dir)
      cy = cy + gm.lengthdir_y(step, dir)
    else
      cx = x0 + gm.lengthdir_x(len * t, dir0)
      cy = y0 + gm.lengthdir_y(len * t, dir0)
    end
    off = off * 0.55 + gm.random_range(-wobble, wobble) * 0.45
    local amp = off * (1.0 - gm.abs(t - 0.5))
    local nx = cx + gm.lengthdir_x(amp, perp)
    local ny = cy + gm.lengthdir_y(amp, perp)
    local last = pts[gm.array_length(pts) - 1]
    if not gm.bool(self.myc_segment_is_playable(self, other, last.x, last.y, nx, ny, w, cave)) then
      nx = cx
      ny = cy
      if not gm.bool(self.myc_segment_is_playable(self, other, last.x, last.y, nx, ny, w, cave)) then
        if kink <= 0 then break end
        if dir == dir0 then break end
        dir = dir0
        perp = dir0 + 90
        cx = last.x + gm.lengthdir_x(step, dir0)
        cy = last.y + gm.lengthdir_y(step, dir0)
        nx = cx
        ny = cy
        if not gm.bool(self.myc_segment_is_playable(self, other, last.x, last.y, nx, ny, w, cave)) then break end
      end
    end
    gm.array_push(pts, gm.new(___struct___497__anon__36616__gml_Object_obj_mycelium_controller_Create_0, nx, ny))
    i = i + 1
  end
  return self.myc_build_branch(self, other, pts, a6, width, a10)
end

-- gml_Script____struct___498__anon__39420__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067dbe0
function ___struct___498__anon__39420__gml_Object_obj_mycelium_controller_Create_0(self, other, points, cells, start, width, bx1, by1, bx2, by2)
  if points == nil then points = gm.undefined end
  if cells == nil then cells = gm.undefined end
  if start == nil then start = gm.undefined end
  if width == nil then width = gm.undefined end
  if bx1 == nil then bx1 = gm.undefined end
  if by1 == nil then by1 = gm.undefined end
  if bx2 == nil then bx2 = gm.undefined end
  if by2 == nil then by2 = gm.undefined end
  self.points = points
  self.cells = cells
  self.shoved = false
  self.swept = 0
  self.start = start
  self.width = width
  self.bx1 = bx1
  self.by1 = by1
  self.bx2 = bx2
  self.by2 = by2
end

-- gml_Script_anon__39420__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067e0d0
function anon__39420__gml_Object_obj_mycelium_controller_Create_0(self, other, pts, start, width, raster)
  if gm.is_undefined(raster) then raster = true end
  local p0 = pts[0]
  local out = gm.array(p0)
  local minx = p0.x
  local miny = p0.y
  local maxx = p0.x
  local maxy = p0.y
  local i = 1.0
  while i < gm.array_length(pts) do
    local p = pts[i]
    gm.array_push(out, p)
    minx = gm.min(minx, p.x)
    miny = gm.min(miny, p.y)
    maxx = gm.max(maxx, p.x)
    maxy = gm.max(maxy, p.y)
    i = i + 1
  end
  local cells
  if gm.bool(raster) then
    cells = self.myc_rasterize_branch(self, other, out, width)
  else
    cells = gm.undefined
  end
  return gm.new(___struct___498__anon__39420__gml_Object_obj_mycelium_controller_Create_0,
    out, cells, start, width, minx, miny, maxx, maxy)
end

-- gml_Script____struct___499__anon__40328__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067f200
function ___struct___499__anon__40328__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, w)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if w == nil then w = gm.undefined end
  self.x = x
  self.y = y
  self.w = w
end

-- gml_Script____struct___500__anon__40328__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067f460
function ___struct___500__anon__40328__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  self.x = x
  self.y = y
  self.w = 1.0
end

-- gml_Script_anon__40328__gml_Object_obj_mycelium_controller_Create_0 @ 0x14067f6a0
function anon__40328__gml_Object_obj_mycelium_controller_Create_0(self, other, pts, width)
  local n = gm.array_length(pts)
  local out = gm.array()
  if n < 2.0 then
    return out
  end
  local total = 0
  local i = 1.0
  while i < n do
    total = total + gm.point_distance(pts[i - 1].x, pts[i - 1].y, pts[i].x, pts[i].y)
    i = i + 1
  end
  if total <= 0 then
    return out
  end
  local acc = 0
  local lx = -999999.0
  local ly = -999999.0
  i = 1.0
  while i < n do
    local a = pts[i - 1]
    local b = pts[i]
    local seg = gm.point_distance(a.x, a.y, b.x, b.y)
    if seg > 0 then
      local d = 0
      while d < seg do
        local t = d / seg
        local u = (acc + d) / total
        local w = gm.max(1.0, gm.round(gm.lerp(width, 1.0, u)))
        local cx = gm.floor(gm.lerp(a.x, b.x, t))
        local cy = gm.floor(gm.lerp(a.y, b.y, t))
        if cx ~= lx or cy ~= ly then
          gm.array_push(out, gm.new(___struct___499__anon__40328__gml_Object_obj_mycelium_controller_Create_0, cx, cy, w))
          lx = cx
          ly = cy
        end
        d = d + gm.max(1.0, w - 1)
      end
      acc = acc + seg
    end
    i = i + 1
  end
  local last = pts[n - 1]
  gm.array_push(out, gm.new(___struct___500__anon__40328__gml_Object_obj_mycelium_controller_Create_0, gm.floor(last.x), gm.floor(last.y)))
  return out
end

-- gml_Script____struct___502__anon__42245__gml_Object_obj_mycelium_controller_Create_0 @ 0x140681ce0
function ___struct___502__anon__42245__gml_Object_obj_mycelium_controller_Create_0(self, other, duration, finish_age, branches, budding, vbuf, bx1, by1, bx2, by2)
  if duration == nil then duration = gm.undefined end
  if finish_age == nil then finish_age = gm.undefined end
  if branches == nil then branches = gm.undefined end
  if budding == nil then budding = gm.undefined end
  if vbuf == nil then vbuf = gm.undefined end
  if bx1 == nil then bx1 = gm.undefined end
  if by1 == nil then by1 = gm.undefined end
  if bx2 == nil then bx2 = gm.undefined end
  if by2 == nil then by2 = gm.undefined end
  self.age = 0
  self.duration = duration
  self.finish_age = finish_age
  self.branches = branches
  self.scanned_length = 0
  self.finished = false
  self.budding = budding
  self.vbuf = vbuf
  self.bx1 = bx1
  self.by1 = by1
  self.bx2 = bx2
  self.by2 = by2
end

-- gml_Script_anon__42245__gml_Object_obj_mycelium_controller_Create_0 @ 0x140682270
function anon__42245__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py, dir, len, opts)
  if gm.is_undefined(opts) then opts = gm.struct({}) end
  local free = gm.variable_struct_get(opts, "free")
  if gm.is_undefined(free) then free = false end
  local pierce = gm.variable_struct_get(opts, "pierce")
  if gm.is_undefined(pierce) then pierce = false end
  if len < self.myc_min_growth_len then
    return false
  end
  if not gm.bool(free) then
    local cost = self.myc_shot_cost_for(self, other, px, py, dir, len)
    local food = self.myc_shot_food_for(self, other, px, py, dir, len)
    if water_total(self, other) < cost or global.food < food then
      return false
    end
    water_spend(self, other, cost)
    food_spend(self, other, food)
  end
  local wm = gm.variable_struct_get(opts, "width_mult")
  if gm.is_undefined(wm) then wm = 1.0 end
  local thick = vis(self, other, "strand_thickness") * wm
  local spread = myc_shot_spread(self, other)
  local range = gm.max(1.0, bal(self, other, "base_shot_range"))
  local mainlen = gm.min(len, range)
  local ratio = gm.max(1.0, len / range)
  local jitter = mainlen * vis(self, other, "growth_jitter") * 0.1 * spread
  local kpb = gm.variable_struct_get(opts, "kinks_per_base")
  if gm.is_undefined(kpb) then kpb = 0 end
  local kinks = kpb * ratio
  local instant = gm.variable_struct_get(opts, "instant")
  if gm.is_undefined(instant) then instant = false end
  local raster = not gm.bool(instant)
  local main = self.myc_make_branch(self, other, px, py, dir, len, gm.round(14 * ratio), jitter, 0, thick, pierce, kinks, raster)
  local branches = gm.array(main)
  local mpts = main.points
  local n = gm.array_length(mpts)
  local max_start = 0
  if n > 4 then
    local count = gm.round(bal(self, other, "growth_side_branches") * ratio)
    local step = (n - 4) / gm.max(1.0, count)
    local k = 0
    while k < count do
      local idx = gm.clamp(2 + gm.floor((k + gm.random(1.0)) * step), 0, n - 2)
      local bp = mpts[idx]
      local sang = gm.min(80.0, gm.random_range(25.0, 55.0) * gm.lerp(1.0, spread, 0.5))
      local sdir = dir + gm.choose(-1.0, 1.0) * sang
      local slen = self.myc_measure_allowed_len_from(self, other, bp.x, bp.y, sdir,
        gm.random_range(mainlen * 0.17, mainlen * 0.44) * spread, pierce)
      if not (slen < self.myc_tile * 0.5) then
        local sstart = 5 + (idx / gm.max(1.0, n - 1)) * self.myc_growth_duration
        local sb = self.myc_make_branch(self, other, bp.x, bp.y, sdir, slen, 6.0, jitter * 0.69, sstart, thick * 0.66, pierce, 0, raster)
        if gm.array_length(sb.points) > 1 then
          gm.array_push(branches, sb)
          max_start = gm.max(max_start, sstart)
        end
      end
      k = k + 1
    end
  end
  local bx1 = branches[0].bx1
  local by1 = branches[0].by1
  local bx2 = branches[0].bx2
  local by2 = branches[0].by2
  local i = 1
  while i < gm.array_length(branches) do
    bx1 = gm.min(bx1, branches[i].bx1)
    by1 = gm.min(by1, branches[i].by1)
    bx2 = gm.max(bx2, branches[i].bx2)
    by2 = gm.max(by2, branches[i].by2)
    i = i + 1
  end
  if not gm.bool(free) then
    tutorial_event(self, other, "shot", mpts[n - 1].x, mpts[n - 1].y)
    if tutorial_state(self, other, "clay") == 0 then
      local j = n - 1
      while j >= 0 do
        local p = mpts[j]
        if world_tile_at(self, other, p.x, p.y) == 1
          and world_locate(self, other, p.x, p.y).row >= world_dense_row(self, other) then
          tutorial_event(self, other, "clay", p.x, p.y)
          break
        end
        j = j - 1
      end
    end
  end
  local dur = self.myc_growth_duration
  local finish = (dur + max_start) + 2
  local bud_chance = myc_budding_chance(self, other)
  local g = gm.new(___struct___502__anon__42245__gml_Object_obj_mycelium_controller_Create_0,
    dur, finish, branches, gm.random(1.0) < bud_chance, -1.0, bx1, by1, bx2, by2)
  local silent = gm.variable_struct_get(opts, "silent")
  if gm.is_undefined(silent) then silent = false end
  if not gm.bool(silent) then
    g.sound = hyphae_sound_begin(self, other, len, finish, gm.undefined, gm.undefined)
  end
  gm.array_push(self.myc_growths, g)
  gm.array_push(self.myc_active, g)
  self.myc_grid_add(self, other, g)
  return true
end

-- gml_Script____struct___503__anon__47204__gml_Object_obj_mycelium_controller_Create_0 @ 0x140685cf0
function ___struct___503__anon__47204__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.done = false
end

-- gml_Script____struct___504__anon__47204__gml_Object_obj_mycelium_controller_Create_0 @ 0x140685e20
function ___struct___504__anon__47204__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, dir, sound)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if dir == nil then dir = gm.undefined end
  if sound == nil then sound = gm.undefined end
  self.x = x
  self.y = y
  self.dir = dir
  self.sound = sound
end

-- gml_Script_anon__47204__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406860f0
function anon__47204__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  local count = gm.max(1.0, bal(self, other, "digger_strand_count"))
  local fan_from = vis(self, other, "digger_fan_from")
  local fan_to = vis(self, other, "digger_fan_to")
  local ry = py + vis(self, other, "digger_root_depth")
  local shared = gm.new(___struct___503__anon__47204__gml_Object_obj_mycelium_controller_Create_0)
  local i = 0
  while i < count do
    local t
    if count == 1.0 then
      t = 0.5
    else
      t = i / (count - 1)
    end
    local jit = vis(self, other, "digger_fan_jitter")
    local jr = gm.random_range(-1.0, 1.0) * jit
    gm.array_push(self.myc_digger_queue, gm.new(___struct___504__anon__47204__gml_Object_obj_mycelium_controller_Create_0,
      px, ry, gm.lerp(fan_from, fan_to, t) + jr, shared))
    i = i + 1
  end
end

-- gml_Script____struct___505__anon__48225__gml_Object_obj_mycelium_controller_Create_0 @ 0x140686fa0
function ___struct___505__anon__48225__gml_Object_obj_mycelium_controller_Create_0(self, other, width_mult, kinks_per_base, silent)
  if width_mult == nil then width_mult = gm.undefined end
  if kinks_per_base == nil then kinks_per_base = gm.undefined end
  if silent == nil then silent = gm.undefined end
  self.free = true
  self.pierce = true
  self.width_mult = width_mult
  self.kinks_per_base = kinks_per_base
  self.silent = silent
end

-- gml_Script_anon__48225__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406872a0
function anon__48225__gml_Object_obj_mycelium_controller_Create_0(self, other)
  if gm.array_length(self.myc_digger_queue) == 0 then
    return
  end
  local deadline = gm.get_timer() + vis(self, other, "digger_strands_budget_us")
  local range = myc_digger_range(self, other)
  local done_n = 0
  while self.myc_digger_next < gm.array_length(self.myc_digger_queue)
    and (done_n == 0 or gm.get_timer() < deadline) do
    local q = self.myc_digger_queue[self.myc_digger_next]
    self.myc_digger_next = self.myc_digger_next + 1
    done_n = done_n + 1
    local len = self.myc_measure_allowed_len_from(self, other, q.x, q.y, q.dir, range, true)
    local ok = self.myc_add_growth_from(self, other, q.x, q.y, q.dir, len,
      gm.new(___struct___505__anon__48225__gml_Object_obj_mycelium_controller_Create_0,
        vis(self, other, "digger_strand_width_mult"), vis(self, other, "digger_kinks_per_base"), q.sound.done))
    if gm.bool(ok) then
      q.sound.done = true
    end
  end
  if self.myc_digger_next >= gm.array_length(self.myc_digger_queue) then
    self.myc_digger_queue = gm.array()
    self.myc_digger_next = 0
  end
end

-- gml_Script____struct___506__anon__49515__gml_Object_obj_mycelium_controller_Create_0 @ 0x140688400
function ___struct___506__anon__49515__gml_Object_obj_mycelium_controller_Create_0(self, other, x, y, d)
  if x == nil then x = gm.undefined end
  if y == nil then y = gm.undefined end
  if d == nil then d = gm.undefined end
  self.x = x
  self.y = y
  self.d = d
end

-- gml_Script_anon__49789__anon__49515__gml_Object_obj_mycelium_controller_Create_0 @ 0x140688650
function anon__49789__anon__49515__gml_Object_obj_mycelium_controller_Create_0(self, other, a, b)
  return a.d - b.d
end

-- gml_Script_anon__49515__gml_Object_obj_mycelium_controller_Create_0 @ 0x140688810
function anon__49515__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  local list = gm.array()
  local i = 0
  while i < gm.array_length(self.myc_knots) do
    local k = self.myc_knots[i]
    gm.array_push(list, gm.new(___struct___506__anon__49515__gml_Object_obj_mycelium_controller_Create_0,
      k.x, k.y, gm.point_distance(px, py, k.x, k.y)))
    i = i + 1
  end
  gm.array_sort(list, gm.method(self, anon__49789__anon__49515__gml_Object_obj_mycelium_controller_Create_0))
  if self.myc_burst_next > 0 then
    gm.array_delete(self.myc_burst_queue, 0, self.myc_burst_next)
    self.myc_burst_next = 0
  end
  self.myc_burst_queue = gm.array_concat(self.myc_burst_queue, list)
end

-- gml_Script_anon__50255__gml_Object_obj_mycelium_controller_Create_0 @ 0x140689430
function anon__50255__gml_Object_obj_mycelium_controller_Create_0(self, other, p)
  local m = myc_shot_range(self, other)
  return p.x > self.myc_camera_x - m
    and p.x < (self.myc_camera_x + self.myc_view_w) + m
    and p.y > self.myc_camera_y - m
    and p.y < (self.myc_camera_y + self.myc_view_h) + m
end

-- gml_Script____struct___507__anon__50824__gml_Object_obj_mycelium_controller_Create_0 @ 0x1406899e0
function ___struct___507__anon__50824__gml_Object_obj_mycelium_controller_Create_0(self, other, silent, instant)
  if silent == nil then silent = gm.undefined end
  if instant == nil then instant = gm.undefined end
  self.free = true
  self.silent = silent
  self.instant = instant
end

-- gml_Script_anon__50824__gml_Object_obj_mycelium_controller_Create_0 @ 0x140689c10
function anon__50824__gml_Object_obj_mycelium_controller_Create_0(self, other, k, instant)
  local dir = gm.random(360.0)
  local len = self.myc_measure_allowed_len_from(self, other, k.x, k.y, dir, myc_shot_range(self, other))
  if len < self.myc_min_growth_len then
    return gm.undefined
  end
  local silent
  if gm.bool(instant) then
    silent = true
  else
    silent = gm.bool(self.myc_burst_sounded)
  end
  local ok = self.myc_add_growth_from(self, other, k.x, k.y, dir, len,
    gm.new(___struct___507__anon__50824__gml_Object_obj_mycelium_controller_Create_0, silent, instant))
  if not gm.bool(ok) then
    return gm.undefined
  end
  local g = self.myc_growths[gm.array_length(self.myc_growths) - 1]
  if gm.bool(instant) then
    g.instant = true
    g.age = g.finish_age
    self.myc_scan_growth(self, other, g)
    self.myc_growth_bake(self, other, g)
  else
    self.myc_burst_sounded = true
  end
  return g
end

-- gml_Script_anon__52233__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068a870
function anon__52233__gml_Object_obj_mycelium_controller_Create_0(self, other)
  if self.myc_burst_next >= gm.array_length(self.myc_burst_queue)
    and self.myc_burst_show_next >= gm.array_length(self.myc_burst_show) then
    if gm.array_length(self.myc_burst_queue) > 0 or gm.array_length(self.myc_burst_show) > 0 then
      self.myc_burst_queue = gm.array()
      self.myc_burst_next = 0
      self.myc_burst_show = gm.array()
      self.myc_burst_show_next = 0
    end
    self.myc_burst_growing = gm.array()
    self.myc_burst_sounded = false
    return
  end
  local deadline = gm.get_timer() + vis(self, other, "cave_burst_instant_budget_us")
  while self.myc_burst_next < gm.array_length(self.myc_burst_queue) and gm.get_timer() < deadline do
    local k = self.myc_burst_queue[self.myc_burst_next]
    self.myc_burst_next = self.myc_burst_next + 1
    if gm.bool(self.myc_burst_in_view(self, other, k)) then
      gm.array_push(self.myc_burst_show, k)
    else
      self.myc_burst_fire(self, other, k, true)
    end
  end
  local still = gm.array()
  local i = 0
  while i < gm.array_length(self.myc_burst_growing) do
    if not gm.bool(self.myc_burst_growing[i].finished) then
      gm.array_push(still, self.myc_burst_growing[i])
    end
    i = i + 1
  end
  self.myc_burst_growing = still
  local started = 0
  local per_frame = gm.max(1.0, vis(self, other, "cave_burst_per_frame"))
  local limit = gm.min(per_frame, vis(self, other, "cave_burst_max_growing") - gm.array_length(self.myc_burst_growing))
  while self.myc_burst_show_next < gm.array_length(self.myc_burst_show) do
    local k = self.myc_burst_show[self.myc_burst_show_next]
    if not gm.bool(self.myc_burst_in_view(self, other, k)) then
      if gm.get_timer() >= deadline then
        return
      end
      self.myc_burst_show_next = self.myc_burst_show_next + 1
      self.myc_burst_fire(self, other, k, true)
    else
      if started >= limit then
        return
      end
      self.myc_burst_show_next = self.myc_burst_show_next + 1
      started = started + 1
      local g = self.myc_burst_fire(self, other, k, false)
      if not gm.is_undefined(g) then
        gm.array_push(self.myc_burst_growing, g)
      end
    end
  end
end

-- gml_Script____struct___509__anon__54631__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068c0c0
function ___struct___509__anon__54631__gml_Object_obj_mycelium_controller_Create_0(self, other)
  self.now = 0
  self.peak = 0
end

-- gml_Script_anon__54631__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068c210
function anon__54631__gml_Object_obj_mycelium_controller_Create_0(self, other, key, us)
  local e = gm.variable_struct_get(self.myc_prof, key)
  if gm.is_undefined(e) then
    e = gm.new(___struct___509__anon__54631__gml_Object_obj_mycelium_controller_Create_0)
    gm.variable_struct_set(self.myc_prof, key, e)
  end
  e.now = us
  e.peak = gm.max(e.peak * 0.97, us)
end

-- gml_Script_anon__54952__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068c910
function anon__54952__gml_Object_obj_mycelium_controller_Create_0(self, other, g)
  g.finished = true
  self.myc_growth_free_chunks(self, other, g)
  gm.array_push(self.myc_unbaked, g)
  if not gm.bool(g.budding) then
    return
  end
  local b = g.branches[gm.irandom(gm.array_length(g.branches) - 1)]
  local p = b.points[gm.array_length(b.points) - 1]
  self.myc_add_knot(self, other, p.x, p.y)
end

-- gml_Script_anon__55404__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068d050
function anon__55404__gml_Object_obj_mycelium_controller_Create_0(self, other, px, py)
  global.food = global.food + world_collect_food_at(self, other, px, py, self.myc_food_pickup_radius)
  if gm.bool(world_connect_water_at(self, other, px, py)) then
    water_touch_sound(self, other, px, py)
  end
  caves_touch_at(self, other, px, py)
  relics_touch_at(self, other, px, py)
  reach_expand(self, other, px, py)
end

-- gml_Script_anon__55774__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068d760
function anon__55774__gml_Object_obj_mycelium_controller_Create_0(self, other, cells, from, to)
  if to <= from then
    return -2.0
  end
  local vb = gm.vertex_create_buffer()
  gm.vertex_begin(vb, global.myc_vformat)
  local i = from
  while i < to do
    local c = cells[i]
    local h = gm.div(c.w, 2)
    local x1 = c.x - h
    local y1 = c.y - h
    local x2 = x1 + c.w
    local y2 = y1 + c.w
    gm.vertex_position(vb, x1, y1)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    gm.vertex_position(vb, x2, y1)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    gm.vertex_position(vb, x1, y2)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    gm.vertex_position(vb, x2, y1)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    gm.vertex_position(vb, x2, y2)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    gm.vertex_position(vb, x1, y2)
    gm.vertex_colour(vb, self.myc_core_color, 1.0)
    i = i + 1
  end
  gm.vertex_end(vb)
  gm.vertex_freeze(vb)
  return vb
end

-- gml_Script_anon__57044__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068e610
function anon__57044__gml_Object_obj_mycelium_controller_Create_0(self, other, g)
  local brs = g.branches
  local i = 0
  while i < gm.array_length(brs) do
    local ch = gm.variable_struct_get(brs[i], "chunks")
    if not gm.is_undefined(ch) then
      local j = 0
      while j < gm.array_length(ch) do
        if ch[j] >= 0 then
          gm.vertex_delete_buffer(ch[j])
        end
        j = j + 1
      end
      brs[i].chunks = gm.array()
    end
    i = i + 1
  end
end

-- gml_Script_anon__57586__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068eef0
function anon__57586__gml_Object_obj_mycelium_controller_Create_0(self, other)
  local i = 0
  while i < gm.array_length(self.myc_active) do
    local g = self.myc_active[i]
    if not gm.bool(g.finished) then
      self.myc_scan_growth(self, other, g)
    end
    i = i + 1
  end
end

-- gml_Script_anon__58028__gml_Object_obj_mycelium_controller_Create_0 @ 0x14068f260
function anon__58028__gml_Object_obj_mycelium_controller_Create_0(self, other, g)
  local total = 0.0
  local brs = g.branches
  local i = 0
  while i < gm.array_length(brs) do
    local b = brs[i]
    local pts = b.points
    local n = gm.array_length(pts)
    if n >= 2.0 then
      local prog = gm.clamp((g.age - b.start) / g.duration, 0, 1.0)
      if prog > 0 then
        local lim = prog * (n - 1)
        local thick = vis(self, other, "strand_thickness")
        local power = wld(self, other, "stone_shove_power") * b.width / gm.max(0.01, thick)
        local upto = gm.min(gm.floor(lim), n - 2)
        local inst = gm.variable_struct_get(g, "instant")
        if gm.is_undefined(inst) then inst = false end
        if gm.bool(inst) then
          b.swept = gm.max(b.swept, upto)
          b.shoved = true
        end
        if upto > b.swept then
          local j = b.swept
          while j < upto do
            local p0 = pts[j]
            local p1 = pts[j + 1]
            stones_shove(self, other, (p0.x + p1.x) * 0.5, (p0.y + p1.y) * 0.5,
              gm.point_direction(p0.x, p0.y, p1.x, p1.y), power)
            j = j + 1
          end
          b.swept = upto
        end
        if prog >= 1.0 and not gm.bool(b.shoved) then
          b.shoved = true
          local a = pts[n - 1]
          local c = pts[n - 2]
          stones_shove(self, other, a.x, a.y, gm.point_direction(c.x, c.y, a.x, a.y), power)
        end
        local count_len = false
        if i == 0 then
          count_len = not gm.variable_struct_exists(g, "no_length")
        end
        local sd = gm.variable_struct_get(b, "scan_done")
        if gm.is_undefined(sd) then sd = 0 end
        local sdl = gm.variable_struct_get(b, "scan_done_len")
        if gm.is_undefined(sdl) then sdl = 0 end
        local hi = gm.min(gm.floor(lim), n - 1)
        local j = sd
        while j < hi do
          local p0 = pts[j]
          local p1 = pts[j + 1]
          sdl = sdl + gm.point_distance(p0.x, p0.y, p1.x, p1.y)
          self.myc_harvest_at(self, other, p1.x, p1.y)
          j = j + 1
        end
        b.scan_done = gm.max(sd, hi)
        b.scan_done_len = sdl
        local partial = 0.0
        if hi < n - 1 then
          local f = gm.clamp(lim - hi, 0, 1.0)
          if f > 0 then
            local p0 = pts[hi]
            local p1 = pts[hi + 1]
            local qx = gm.lerp(p0.x, p1.x, f)
            local qy = gm.lerp(p0.y, p1.y, f)
            partial = gm.point_distance(p0.x, p0.y, qx, qy)
            self.myc_harvest_at(self, other, qx, qy)
          end
        end
        if count_len then
          total = total + (sdl + partial)
        end
      end
    end
    i = i + 1
  end
  mycelium_add_length(self, other, total - g.scanned_length)
  g.scanned_length = total
  if g.age >= g.finish_age then
    self.myc_finish_growth(self, other, g)
  end
end
