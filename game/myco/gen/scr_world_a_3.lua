local gm = require("gm")

-- gml_Script_caves_wrap_hit @ 0x140309c00
function caves_wrap_hit(self, other, x, y, maxd)
  local best = gm.new(___struct___337__caves_wrap_hit__scr_world, x, y, maxd)
  if gm.variable_global_exists("cave_masks") then
    if gm.is_struct(global.cave_masks) then
      local rects = world_cave_rects(self, other)
      local pad = maxd + vis(self, other, "cave_branch_max_px") + 8
      local i = 0
      while i < gm.array_length(rects) do
        local r = rects[i]
        local id = r.def.id
        if gm.bool(cave_taken(self, other, r))
          and not (x < r.x1 - pad) and not (x > r.x2 + pad)
          and not (y < r.y1 - pad) and not (y > r.y2 + pad) then
          local wr = gm.variable_struct_get(global.cave_wrap, r.id)
          local skip = false
          if not gm.is_undefined(wr) then
            if gm.current_time - wr.born < vis(self, other, "cave_wrap_ms") then
              skip = true
            end
          end
          if not skip then
            local m = gm.variable_struct_get(global.cave_masks, id)
            local lines = gm.array(m.outline)
            local b = 0
            while b < gm.array_length(m.branches) do
              gm.array_push(lines, m.branches[b].points)
              b = b + 1
            end
            local k = 0
            while k < gm.array_length(lines) do
              local pts = lines[k]
              local n = gm.array_length(pts)
              local segs
              if k == 0 then
                segs = n
              else
                segs = n - 1
              end
              local j = 0
              while j < segs do
                local a = pts[j]
                local c = pts[gm.mod(j + 1, n)]
                local ax = r.x1 + a[0] * r.k
                local ay = r.y1 + a[1] * r.k
                local dx = (c[0] - a[0]) * r.k
                local dy = (c[1] - a[1]) * r.k
                local l2 = dx * dx + dy * dy
                local t
                if l2 > 0 then
                  t = gm.clamp(((x - ax) * dx + (y - ay) * dy) / l2, 0, 1)
                else
                  t = 0
                end
                local px = ax + dx * t
                local py = ay + dy * t
                local d = gm.point_distance(x, y, px, py)
                if d <= best.dist then
                  if not gm.bool(world_cave_solid_any(self, other, px, py)) then
                    best.hit = true
                    best.x = px
                    best.y = py
                    best.dist = d
                  end
                end
                j = j + 1
              end
              k = k + 1
            end
          end
        end
        i = i + 1
      end
      return best
    end
  end
  return best
end

-- gml_Script_caves_wrap_draw_visible @ 0x14030d480
function caves_wrap_draw_visible(self, other, x1, y1, x2, y2)
  local rects = world_cave_rects(self, other)
  local now = gm.current_time
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    if gm.bool(cave_taken(self, other, r))
      and not (r.x2 < x1) and not (r.x1 > x2)
      and not (r.y2 < y1) and not (r.y1 > y2) then
      cave_wrap_draw(self, other, r, now)
    end
    i = i + 1
  end
end

-- gml_Script_cave_wrap_draw @ 0x14030db20
function cave_wrap_draw(self, other, r, now)
  if not gm.variable_global_exists("cave_masks") then
    return
  end
  if not gm.is_struct(global.cave_masks) then
    return
  end
  local m = gm.variable_struct_get(global.cave_masks, r.def.id)
  if gm.is_undefined(m) then
    return
  end
  local pts = m.outline
  local n = gm.array_length(pts)
  if n == 0 then
    return
  end
  local wr = gm.variable_struct_get(global.cave_wrap, r.id)
  local t = 1
  local ts = 1
  local start = 0
  if not gm.is_undefined(wr) then
    local age = now - wr.born
    local dur = gm.max(1, vis(self, other, "cave_wrap_ms"))
    t = gm.clamp(age / dur, 0, 1)
    ts = gm.clamp((age - dur * vis(self, other, "cave_stem_start")) / gm.max(1, vis(self, other, "cave_stem_ms")), 0, 1)
    local best = gm.infinity
    local i = 0
    while i < n do
      local d = gm.point_distance(pts[i][0], pts[i][1], wr.from_x, wr.from_y)
      if d < best then
        best = d
        start = i
      end
      i = i + 1
    end
  end
  local e = 1 - gm.power(1 - t, 2)
  local len = gm.ceil(n * 0.5 * e)
  local x1 = r.x1
  local k = r.k
  local y1 = r.y1
  local halo = vis(self, other, "cave_wrap_halo_colour")
  local core = vis(self, other, "cave_wrap_core_colour")
  local halo_px = vis(self, other, "cave_wrap_halo_px")
  local core_px = vis(self, other, "cave_wrap_core_px")
  local pass = 0
  while pass < 2 do
    local col
    if pass == 0 then
      col = halo
    else
      col = core
    end
    local lw
    if pass == 0 then
      lw = halo_px
    else
      lw = core_px
    end
    gm.draw_set_alpha((pass == 0) and 0.45 or 1)
    local dir = -1
    while dir <= 1 do
      local s = 0
      while s < len do
        local a = pts[gm.mod(start + dir * s + n * 2, n)]
        local b = pts[gm.mod(start + dir * (s + 1) + n * 2, n)]
        gm.draw_line_width_colour(x1 + a[0] * k, y1 + a[1] * k, x1 + b[0] * k, y1 + b[1] * k, lw, col, col)
        s = s + 1
      end
      dir = dir + 2
    end
    local bi = 0
    while bi < gm.array_length(m.branches) do
      local br = m.branches[bi]
      local dd = gm.abs(br.at - start)
      dd = gm.min(dd, n - dd)
      local delay = br.delay
      if gm.is_undefined(delay) then
        delay = 0
      end
      local bt = gm.clamp((len - dd) / gm.max(1, vis(self, other, "cave_branch_grow")) - delay, 0, 1)
      if bt > 0 then
        local bp = br.points
        local nn = gm.array_length(bp) - 1
        local lim = nn * bt
        local s = 0
        while s < nn and s < lim do
          local a = bp[s]
          local c = bp[s + 1]
          local f = gm.min(1, lim - s)
          local w = gm.lerp(0.9, 0.45, s / gm.max(1, nn))
          gm.draw_line_width_colour(x1 + a[0] * k, y1 + a[1] * k,
            x1 + gm.lerp(a[0], c[0], f) * k, y1 + gm.lerp(a[1], c[1], f) * k, lw * w, col, col)
          s = s + 1
        end
      end
      bi = bi + 1
    end
    local si = 0
    while si < gm.array_length(m.stems) do
      local sp = m.stems[si]
      local nn = gm.array_length(sp) - 1
      local lim = nn * ts
      local s = 0
      while s < nn and s < lim do
        local a = sp[s]
        local c = sp[s + 1]
        local f = gm.min(1, lim - s)
        gm.draw_line_width_colour(x1 + a[0] * k, y1 + a[1] * k,
          x1 + gm.lerp(a[0], c[0], f) * k, y1 + gm.lerp(a[1], c[1], f) * k, lw * 0.8, col, col)
        s = s + 1
      end
      si = si + 1
    end
    pass = pass + 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script_cave_popups_draw @ 0x140314760
function cave_popups_draw(self, other, vx, vy)
  local ms = vis(self, other, "cave_popup_ms")
  local keep = gm.array()
  gm.draw_set_font(ui_font(self, other))
  gm.draw_set_halign(1)
  gm.draw_set_valign(2)
  local i = 0
  while i < gm.array_length(global.cave_popups) do
    local p = global.cave_popups[i]
    local k = (gm.current_time - p.born) / gm.max(1, ms)
    if k >= 1 then
      -- continue
    else
      gm.array_push(keep, p)
      local a = gm.min(1, k * 6) * gm.min(1, (1 - k) * 3)
      local px = p.x - vx
      local py = p.y - vy - vis(self, other, "cave_popup_rise_px") * k
      local sc = goal_fit(self, other, p.text, vis(self, other, "cave_popup_scale"), gm.display_get_gui_width() - 48, 0, -1)
      gm.draw_set_alpha(a * 0.85)
      gm.draw_set_colour(0)
      gm.draw_text_transformed(px + 2, py + 2, p.text, sc, sc, 0)
      gm.draw_set_alpha(a)
      gm.draw_set_colour(0xffffff)
      gm.draw_text_transformed(px, py, p.text, sc, sc, 0)
    end
    i = i + 1
  end
  global.cave_popups = keep
  gm.draw_set_alpha(1)
  gm.draw_set_halign(0)
  gm.draw_set_valign(0)
  gm.draw_set_font(ui_font_small(self, other))
end

-- gml_Script_cave_perk_icons_draw @ 0x140316230
function cave_perk_icons_draw(self, other, x, y)
  local defs = cave_defs(self, other)
  local px = vis(self, other, "cave_icon_px")
  local gap = vis(self, other, "cave_icon_gap_px")
  local old_filter = gm.gpu_get_tex_filter()
  gm.gpu_set_tex_filter(vis(self, other, "cave_icon_smooth"))
  local i = 0
  while i < gm.array_length(defs) do
    local d = defs[i]
    if not gm.bool(perk_has(self, other, d.id)) then
      -- continue
    else
      local spr = d.hud
      local w = gm.sprite_get_width(spr)
      local h = gm.sprite_get_height(spr)
      local s = px / gm.max(w, h)
      gm.draw_sprite_ext(spr, 0,
        x + (px - w * s) * 0.5 + gm.sprite_get_xoffset(spr) * s,
        y + (px - h * s) * 0.5 + gm.sprite_get_yoffset(spr) * s,
        s, s, 0, 0xffffff, 1)
      x = x + (px + gap)
    end
    i = i + 1
  end
  gm.gpu_set_tex_filter(old_filter)
end

-- gml_Script_cave_hover_tip @ 0x140317800
function cave_hover_tip(self, other, mx, my, ix, iy, vx, vy)
  local defs = cave_defs(self, other)
  local px = vis(self, other, "cave_icon_px")
  local gap = vis(self, other, "cave_icon_gap_px")
  local x = ix
  local i = 0
  while i < gm.array_length(defs) do
    if gm.bool(perk_has(self, other, defs[i].id)) then
      if mx >= x and mx < x + px and my >= iy and my < iy + px then
        return loc(self, other, "perk_" .. defs[i].id .. "_desc")
      end
      x = x + (px + gap)
    end
    i = i + 1
  end
  local wx = mx + vx
  local wy = my + vy
  local rects = world_cave_rects(self, other)
  i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    if gm.bool(cave_taken(self, other, r))
      and wx >= r.x1 and wx < r.x2 and wy >= r.y1 and wy < r.y2 then
      if gm.bool(cave_solid_at(self, other, r, wx, wy)) then
        return loc(self, other, "perk_" .. r.def.id .. "_desc")
      end
    end
    i = i + 1
  end
  return ""
end

-- gml_Script_perk_debug_next @ 0x140318d60
function perk_debug_next(self, other)
  local defs = cave_defs(self, other)
  local rects = world_cave_rects(self, other)
  local i = 0
  while i <= gm.array_length(defs) do
    local id
    if i >= gm.array_length(defs) then
      id = "blue"
    else
      id = defs[i].id
    end
    if i < gm.array_length(defs) and gm.bool(perk_has(self, other, id)) then
      -- continue
    else
      local j = 0
      while j < gm.array_length(rects) do
        if rects[j].def.id == id and not gm.bool(cave_taken(self, other, rects[j])) then
          cave_take(self, other, rects[j], gm.undefined, gm.undefined)
          return "cave " .. rects[j].id
        end
        j = j + 1
      end
    end
    i = i + 1
  end
  return "all caves taken"
end

-- gml_Script____struct___338__caves_debug_draw__scr_world @ 0x140319a70
function ___struct___338__caves_debug_draw__scr_world(self, other, blue, purple, red, green, yellow)
  self.blue = blue
  self.purple = purple
  self.red = red
  self.green = green
  self.yellow = yellow
end

-- gml_Script____struct___339__caves_debug_draw__scr_world @ 0x140319d80
function ___struct___339__caves_debug_draw__scr_world(self, other, x, y, name, colour, dim)
  self.x = x
  self.y = y
  self.name = name
  self.colour = colour
  self.dim = dim
  self.r = 40
end

-- gml_Script____struct___340__caves_debug_draw__scr_world @ 0x14031a0e0
function ___struct___340__caves_debug_draw__scr_world(self, other, x, y, name)
  self.x = x
  self.y = y
  self.name = name
  self.colour = 16777215
  self.dim = false
  self.r = 60
end

-- gml_Script____struct___341__caves_debug_draw__scr_world @ 0x14031a410
function ___struct___341__caves_debug_draw__scr_world(self, other, x, y, name, colour, d)
  self.x = x
  self.y = y
  self.name = name
  self.colour = colour
  self.dim = false
  self.r = 24
  self.d = d
end

-- gml_Script_anon__68900__caves_debug_draw__scr_world @ 0x14031a7c0
function anon__68900__caves_debug_draw__scr_world(self, other, a, b)
  return a.d - b.d
end

-- gml_Script_caves_debug_draw @ 0x14031a980
local caves_debug_draw_static = { _colours = gm.undefined }
function caves_debug_draw(self, other, vx, vy, vw, vh)
  local st = caves_debug_draw_static
  if gm.is_undefined(st._colours) then
    st._colours = gm.new(___struct___338__caves_debug_draw__scr_world, 16765550, 16742590, 7237375, 7270550, 5956095)
  end
  local hw = vw * 0.5
  local hh = vh * 0.5
  local cx = vx + hw
  local cy = vy + hh
  local margin = 48
  local bone_col = 13492715
  local items = gm.array()
  local rects = world_cave_rects(self, other)
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    local col = gm.variable_struct_get(st._colours, r.def.id)
    if gm.is_undefined(col) then
      col = 16777215
    end
    gm.array_push(items, gm.new(___struct___339__caves_debug_draw__scr_world,
      (r.x1 + r.x2) * 0.5, (r.y1 + r.y2) * 0.5, r.id, col, cave_taken(self, other, r)))
    i = i + 1
  end
  local big = world_big_bones(self, other)
  i = 0
  while i < gm.array_length(big) do
    gm.array_push(items, gm.new(___struct___340__caves_debug_draw__scr_world,
      (big[i].x1 + big[i].x2) * 0.5, (big[i].y1 + big[i].y2) * 0.5, big[i].id))
    i = i + 1
  end
  local w = global.world
  local kinds = bone_kinds(self, other)
  local bones = gm.array()
  local c0 = gm.floor(cx / w.chunk_w)
  local c = c0 - 24
  while c <= c0 + 24 do
    local ch = gm.variable_struct_get(w.chunks, world_chunk_key(self, other, c))
    if gm.is_undefined(ch) then
      -- continue
    else
      local list = gm.variable_struct_get(ch, "bones")
      if gm.is_undefined(list) then
        list = gm.array()
      end
      i = 0
      while i < gm.array_length(list) do
        local b = list[i]
        local bx = b.x + b.w * 0.5
        local by = b.y + b.h * 0.5
        gm.array_push(bones, gm.new(___struct___341__caves_debug_draw__scr_world,
          bx, by, kinds[b.kind].id, bone_col, gm.point_distance(cx, cy, bx, by)))
        i = i + 1
      end
    end
    c = c + 1
  end
  gm.array_sort(bones, gm.method(self, anon__68900__caves_debug_draw__scr_world))
  i = 0
  while i < gm.min(gm.array_length(bones), vis(self, other, "debug_bones_nearest")) do
    gm.array_push(items, bones[i])
    i = i + 1
  end
  gm.draw_set_font(ui_font(self, other))
  gm.draw_set_halign(1)
  gm.draw_set_valign(1)
  i = 0
  while i < gm.array_length(items) do
    local it = items[i]
    local sx = it.x - vx
    local sy = it.y - vy
    local label = it.name .. "  " .. length_text(self, other, pixels_to_cm(self, other, gm.point_distance(cx, cy, it.x, it.y)))
    gm.draw_set_alpha(gm.bool(it.dim) and 0.45 or 1)
    gm.draw_set_colour(it.colour)
    if sx >= 0 and sx <= vw and sy >= 0 and sy <= vh then
      gm.draw_circle(sx, sy, it.r, true)
      gm.draw_circle(sx, sy, it.r + 1, true)
      gm.draw_text_transformed(sx, sy - it.r - 20, label, 0.5, 0.5, 0)
    else
      local dx = sx - hw
      local dy = sy - hh
      local k = gm.min((hw - margin) / gm.max(0.001, gm.abs(dx)), (hh - margin) / gm.max(0.001, gm.abs(dy)))
      local ax = hw + dx * k
      local ay = hh + dy * k
      local dir = gm.point_direction(hw, hh, sx, sy)
      gm.draw_triangle(
        ax + gm.lengthdir_x(22, dir), ay + gm.lengthdir_y(22, dir),
        ax + gm.lengthdir_x(14, dir + 140), ay + gm.lengthdir_y(14, dir + 140),
        ax + gm.lengthdir_x(14, dir - 140), ay + gm.lengthdir_y(14, dir - 140),
        false)
      gm.draw_text_transformed(ax - gm.lengthdir_x(34, dir), ay - gm.lengthdir_y(34, dir), label, 0.45, 0.45, 0)
    end
    i = i + 1
  end
  gm.draw_set_alpha(1)
  gm.draw_set_colour(0xffffff)
  gm.draw_set_halign(0)
  gm.draw_set_valign(0)
  gm.draw_set_font(ui_font_small(self, other))
end

-- gml_Script_world_start_clear @ 0x14031e080
function world_start_clear(self, other, ch)
  local ring = world_start_ring(self, other)
  local lakes = gm.array()
  local foods = gm.array()
  local i = 0
  while i < gm.array_length(ch.lakes) do
    local l = ch.lakes[i]
    if world_rect_distance(self, other, ring.x, ring.y, l.body_x, l.body_y, l.body_x + l.body_w, l.body_y + l.body_h) >= ring.near then
      gm.array_push(lakes, l)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(ch.foods) do
    local f = ch.foods[i]
    if gm.point_distance(ring.x, ring.y, f.x, f.y) >= ring.near then
      gm.array_push(foods, f)
    end
    i = i + 1
  end
  ch.lakes = lakes
  ch.foods = foods
end

-- gml_Script_world_start_food_quota @ 0x14031f140
function world_start_food_quota(self, other)
  local w = global.world
  local sw = vis(self, other, "screen_width_px")
  local sh = vis(self, other, "screen_height_px")
  local x0 = w.start_x - sw * 0.5
  local mn = wld(self, other, "start_screen_food_min")
  local mx = wld(self, other, "start_screen_food_max")
  local n = wld(self, other, "start_screen_food_min")
    + gm.floor(world_hash01(self, other, 7, 7, 120) * (mx - mn + 1))
  world_food_fill(self, other, x0, 0, x0 + sw, sh, n, n, 121)
  local salt = 130
  local row = 0
  while row <= 1 do
    local col = -1
    while col <= 1 do
      if row == 0 and col == 0 then
        -- continue
      else
        local cx = x0 + col * sw
        local cy = row * sh
        world_food_fill(self, other, cx, cy, cx + sw, cy + sh, wld(self, other, "start_side_screen_food_min"), -1, salt)
        salt = salt + 10
      end
      col = col + 1
    end
    row = row + 1
  end
end

-- gml_Script_world_food_value_in @ 0x140320750
function world_food_value_in(self, other, x1, y1, x2, y2)
  local w = global.world
  local total = 0
  local reach = gm.floor((y2 - w.surface_y) / w.tile)
  local c = gm.floor(x1 / w.chunk_w)
  while c <= gm.floor(x2 / w.chunk_w) do
    local ch = world_get_chunk(self, other, c)
    world_chunk_reach(self, other, ch, reach)
    local i = 0
    while i < gm.array_length(ch.foods) do
      local f = ch.foods[i]
      if not gm.bool(f.collected) and f.x >= x1 and f.x < x2 and f.y >= y1 and f.y < y2 then
        total = total + world_food_value(self, other, f.kind)
      end
      i = i + 1
    end
    c = c + 1
  end
  return total
end

-- gml_Script____struct___342__world_food_fill__scr_world @ 0x140321580
function ___struct___342__world_food_fill__scr_world(self, other, x, y, kind, cell)
  self.x = x
  self.y = y
  self.kind = kind
  self.cell = cell
  self.collected = false
end

-- gml_Script_world_food_fill @ 0x1403218b0
function world_food_fill(self, other, x1, y1, x2, y2, fmin, fmax, salt)
  local w = global.world
  local v = world_food_value_in(self, other, x1, y1, x2, y2)
  if fmax >= 0 and v > fmax then
    local c = gm.floor(x1 / w.chunk_w)
    while c <= gm.floor(x2 / w.chunk_w) and v > fmax do
      local foods = world_get_chunk(self, other, c).foods
      local k = world_food_kinds(self, other) - 1
      while k >= 0 and v > fmax do
        local j = gm.array_length(foods) - 1
        while j >= 0 and v > fmax do
          local f = foods[j]
          local val = world_food_value(self, other, f.kind)
          if f.kind == k then
            if gm.bool(f.collected) or f.x < x1 or f.x >= x2 or f.y < y1 or f.y >= y2 or v - val < fmin then
              -- continue
            else
              gm.array_delete(foods, j, 1)
              v = v - val
            end
          end
          j = j - 1
        end
        k = k - 1
      end
      c = c + 1
    end
  end
  local target
  if fmax >= 0 then
    target = fmax
  else
    target = fmin
  end
  local ring = world_start_ring(self, other)
  local pad = 32
  local t = 0
  while t < 400 and v < fmin do
    local x = gm.lerp(x1 + pad, x2 - pad, world_hash01(self, other, t, 1, salt))
    local y = gm.lerp(gm.max(y1, w.surface_y + w.tile) + pad, y2 - pad, world_hash01(self, other, t, 2, salt))
    local a = 0
    while a < 4 and v < fmin do
      local ox
      if a == 0 then
        ox = 0
      else
        ox = (world_hash01(self, other, t, a + 3, salt) - 0.5) * 90
      end
      local px = x + ox
      local oy
      if a == 0 then
        oy = 0
      else
        oy = (world_hash01(self, other, t, a + 7, salt) - 0.5) * 90
      end
      local py = y + oy
      if gm.bool(world_food_spot_ok(self, other, px, py, x1, y1, x2, y2, ring)) then
        local need = target - v
        local kind
        if need >= 5 then
          kind = 2
        elseif need >= 3 then
          kind = 1
        else
          kind = 0
        end
        local ch = world_get_chunk(self, other, gm.floor(px / w.chunk_w))
        local cells = world_food_cells(self, other, kind)
        gm.array_push(ch.foods, gm.new(___struct___342__world_food_fill__scr_world, px, py, kind,
          gm.min(cells - 1, gm.floor(world_hash01(self, other, t, a + 11, salt) * cells))))
        v = v + world_food_value(self, other, kind)
      end
      a = a + 1
    end
    t = t + 1
  end
end
