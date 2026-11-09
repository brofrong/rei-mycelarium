local gm = require("gm")

-- gml_Script_stone_redress_after_cut @ 0x140241650
function stone_redress_after_cut(self, other, b, cut)
  local wld = global.world
  local keys = gm.variable_struct_get_names(cut)
  if gm.array_length(keys) == 0 then
    return
  end
  local is_stone = gm.method(gm.new(___struct___221__stone_redress_after_cut__scr_stones, b, cut, wld),
    anon__29665__stone_redress_after_cut__scr_stones)
  local x0 = gm.infinity
  local x1 = -gm.infinity
  local y0 = gm.infinity
  local y1 = -gm.infinity
  local i = 0
  while i < gm.array_length(keys) do
    local parts = gm.string_split(keys[i], ":")
    local cx = gm.real(parts[0])
    local cy = gm.real(parts[1])
    x0 = gm.min(x0, cx)
    x1 = gm.max(x1, cx)
    y0 = gm.min(y0, cy)
    y1 = gm.max(y1, cy)
    i = i + 1
  end
  x0 = x0 - 3
  x1 = x1 + 3
  y0 = y0 - 3
  y1 = y1 + 3
  local changed = 1
  local pass = 0
  while pass < 12 and gm.bool(changed) do
    changed = 0
    local y = y0
    while y <= y1 do
      local x = x0
      while x <= x1 do
        local key = stone_cell_key(self, other, x, y)
        if not gm.is_undefined(gm.variable_struct_get(b.stone_map, key)) then
          if gm.is_undefined(gm.variable_struct_get(cut, key)) then
            local remove = false
            if not gm.bool(is_stone(self, other, x - 1, y)) and not gm.bool(is_stone(self, other, x + 1, y)) then
              remove = true
            elseif not gm.bool(is_stone(self, other, x, y - 1)) and not gm.bool(is_stone(self, other, x, y + 1)) then
              remove = true
            end
            if remove then
              gm.variable_struct_set(cut, key, 1)
              gm.variable_struct_remove(b.stone_map, key)
              changed = 1
            end
          end
        end
        x = x + 1
      end
      y = y + 1
    end
    pass = pass + 1
  end
  b.stone_cells = gm.array_filter(b.stone_cells,
    gm.method(gm.new(___struct___222__stone_redress_after_cut__scr_stones, cut),
      anon__31443__stone_redress_after_cut__scr_stones))
  local y = y0
  while y <= y1 do
    local x = x0
    while x <= x1 do
      local cell = gm.variable_struct_get(b.stone_map, stone_cell_key(self, other, x, y))
      if not gm.is_undefined(cell) then
        -- cell[$ "band"] ?? floor(world_hash01(x, y, 43) * 2) (struct_get_from_hash with the hash of "band")
        local band = cell.band
        if gm.is_undefined(band) then
          band = gm.floor(world_hash01(self, other, x, y, 43) * 2)
        end
        local t = stone_tile_for(self, other, is_stone, x, y, band)
        cell.tc = t[0]
        cell.tr = t[1]
      end
      x = x + 1
    end
    y = y + 1
  end
end

-- gml_Script____struct___223__stone_dress_blob__scr_stones @ 0x140244820
function ___struct___223__stone_dress_blob__scr_stones(self, other, keys)
  self.keys = keys
end

-- gml_Script_anon__32538__stone_dress_blob__scr_stones @ 0x140244980
function anon__32538__stone_dress_blob__scr_stones(self, other, x, y)
  return not gm.is_undefined(gm.variable_struct_get(self.keys, stone_cell_key(self, other, x, y)))
end

-- gml_Script____struct___224__stone_dress_blob__scr_stones @ 0x140244cd0
function ___struct___224__stone_dress_blob__scr_stones(self, other, col, row, tc, tr, deep, band)
  self.col = col
  self.row = row
  self.tc = tc
  self.tr = tr
  self.deep = deep
  self.band = band
end

-- gml_Script_stone_dress_blob @ 0x140245030
function stone_dress_blob(self, other, b, blob, band)
  local dy = world_deep_y(self, other)
  local deep = false
  local i = 0
  while i < gm.array_length(blob.list) do
    if (blob.list[i].row + 1) * 48 > dy then
      deep = true
      break
    end
    i = i + 1
  end
  local is_stone = gm.method(gm.new(___struct___223__stone_dress_blob__scr_stones, blob.keys),
    anon__32538__stone_dress_blob__scr_stones)
  i = 0
  while i < gm.array_length(blob.list) do
    local c = blob.list[i]
    local col = c.col
    local row = c.row
    local t = stone_tile_for(self, other, is_stone, col, row, band)
    local cell = gm.new(___struct___224__stone_dress_blob__scr_stones, col, row, t[0], t[1], deep, band)
    gm.array_push(b.stone_cells, cell)
    gm.variable_struct_set(b.stone_map, stone_cell_key(self, other, col, row), cell)
    i = i + 1
  end
end

-- gml_Script_world_paint_stone_deposits @ 0x1402462f0
function world_paint_stone_deposits(self, other, ch, layer)
  if wld(self, other, "stone_deposits_enabled") <= 0 then
    return
  end
  local w = global.world
  local key = world_layer_key(self, other, ch.cx, layer)
  local tries = gm.max(0, wld(self, other, "stone_deposit_tries"))
  local chance = wld(self, other, "stone_deposit_chance")
  local min_r = gm.max(1, wld(self, other, "stone_deposit_min_radius_cells"))
  local max_r = gm.max(min_r, wld(self, other, "stone_deposit_max_radius_cells"))
  local c0 = gm.ceil(ch.cx * w.chunk_w / 48)
  local c1 = gm.floor((ch.cx + 1) * w.chunk_w / 48) - 1
  local r0 = gm.ceil(world_layer_top(self, other, layer) / 48)
  local r1 = gm.floor(world_layer_bottom(self, other, layer) / 48) - 1
  local base_tries = tries
  local deep_row = gm.floor(world_deep_y(self, other) / 48)
  if r1 >= deep_row then
    tries = gm.round(tries * gm.max(1, wld(self, other, "stone_deposit_deep_mult")))
  end
  local dy = world_deep_y(self, other)
  local deep_layer = world_layer_top(self, other, layer) >= dy
  local i = 0
  while i < tries do
    local extra = i >= base_tries or deep_layer
    local rmax
    if extra then
      rmax = gm.max(max_r, wld(self, other, "stone_deposit_deep_max_radius_cells"))
    else
      rmax = max_r
    end
    local ch_now
    if extra then
      ch_now = wld(self, other, "stone_deposit_deep_chance")
    else
      ch_now = chance
    end
    if not (world_hash01(self, other, key, i, 51) >= ch_now) then
      local bias = gm.max(0.1, wld(self, other, "stone_deposit_size_bias"))
      local rx = min_r + gm.floor(gm.power(world_hash01(self, other, key, i, 52), bias) * (rmax - min_r + 1))
      local ry = min_r + gm.floor(gm.power(world_hash01(self, other, key, i, 53), bias) * (rmax - min_r + 1))
      local lo_c = c0 + rx
      local hi_c = c1 - rx
      local lo_r = r0 + ry
      local hi_r = r1 - ry
      if hi_c >= lo_c and hi_r >= lo_r then
        local cc = lo_c + gm.floor(world_hash01(self, other, key, i, 54) * (hi_c - lo_c + 1))
        local cr = lo_r + gm.floor(world_hash01(self, other, key, i, 55) * (hi_r - lo_r + 1))
        if not (i >= base_tries and cr < deep_row) then
          local min_l = gm.max(1, wld(self, other, "stone_deposit_min_lobes"))
          local max_l = gm.max(min_l, wld(self, other, "stone_deposit_max_lobes"))
          local lobes = min_l + gm.floor(world_hash01(self, other, key, i, 60) * (max_l - min_l + 1))
          local blob = stone_blob_cells(self, other, cc, cr, rx, ry, lobes)
          if gm.array_length(blob.list) ~= 0 then
            if gm.bool(stone_blob_fits(self, other, ch, blob)) then
              stone_dress_blob(self, other, ch, blob, gm.floor(world_hash01(self, other, key, i, 61) * 2))
            end
          end
        end
      end
    end
    i = i + 1
  end
end

-- gml_Script____struct___226_____struct___225__world_obstacles_reach__scr_stones @ 0x14024a960
function ___struct___226_____struct___225__world_obstacles_reach__scr_stones(self, other, list, cursor)
  self.list = list
  self.count = 0
  self.cursor = cursor
end

-- gml_Script____struct___227_____struct___225__world_obstacles_reach__scr_stones @ 0x14024ab60
function ___struct___227_____struct___225__world_obstacles_reach__scr_stones(self, other, list, cursor)
  self.list = list
  self.count = 0
  self.cursor = cursor
end

-- gml_Script____struct___225__world_obstacles_reach__scr_stones @ 0x14024ad60
function ___struct___225__world_obstacles_reach__scr_stones(self, other, right_list, right_cursor, left_list, left_cursor)
  self.right = gm.new(___struct___226_____struct___225__world_obstacles_reach__scr_stones, right_list, right_cursor)
  self.left = gm.new(___struct___227_____struct___225__world_obstacles_reach__scr_stones, left_list, left_cursor)
end

-- gml_Script____struct___228__world_obstacles_reach__scr_stones @ 0x14024b2a0
function ___struct___228__world_obstacles_reach__scr_stones(self, other, key, c1, c2, x1, x2, data)
  self.kind = "maze"
  self.key = key
  self.c1 = c1
  self.c2 = c2
  self.x1 = x1
  self.x2 = x2
  self.data = data
end

-- gml_Script____struct___229__world_obstacles_reach__scr_stones @ 0x14024b670
function ___struct___229__world_obstacles_reach__scr_stones(self, other, key, cx, x1, x2)
  self.kind = "wall"
  self.key = key
  self.cx = cx
  self.x1 = x1
  self.x2 = x2
end

-- gml_Script_world_obstacles_reach @ 0x14024b970
function world_obstacles_reach(self, other, dir, target)
  local w = global.world
  if not gm.variable_struct_exists(w, "obstacles") then
    local start_min = wld(self, other, "surface_wall_min_start_px")
    w.obstacles = gm.new(___struct___225__world_obstacles_reach__scr_stones,
      gm.array(), w.start_x + start_min, gm.array(), w.start_x - start_min)
  end
  local s
  if dir > 0 then
    s = w.obstacles.right
  else
    s = w.obstacles.left
  end
  if wld(self, other, "surface_obstacles_enabled") <= 0 then
    return s.list
  end
  while true do
    local go
    if dir > 0 then
      go = s.cursor <= target
    else
      go = s.cursor >= target
    end
    if not go then
      break
    end
    local idx = s.count * 2 + (dir > 0 and 0 or 1)
    local gap = gm.lerp(wld(self, other, "surface_obstacle_gap_min_px"), wld(self, other, "surface_obstacle_gap_max_px"),
      world_hash01(self, other, idx, 0, 210))
    local every = gm.max(1, wld(self, other, "maze_every"))
    local new_cursor
    if gm.mod(s.count, every) == every - 1 then
      local wsc = gm.lerp(wld(self, other, "maze_min_width_screens"), wld(self, other, "maze_max_width_screens"),
        world_hash01(self, other, idx, 1, 211))
      local wcells = gm.max(12, gm.round(wsc * vis(self, other, "screen_width_px") / 48))
      local c = gm.round((s.cursor + dir * gap) / 48)
      local c1
      if dir > 0 then
        c1 = c
      else
        c1 = c - wcells
      end
      gm.array_push(s.list, gm.new(___struct___228__world_obstacles_reach__scr_stones,
        idx, c1, c1 + wcells - 1, c1 * 48, (c1 + wcells) * 48, gm.undefined))
      if dir > 0 then
        new_cursor = (c1 + wcells) * 48
      else
        new_cursor = c1 * 48
      end
    else
      local cx = gm.floor((s.cursor + dir * gap) / w.chunk_w)
      gm.array_push(s.list, gm.new(___struct___229__world_obstacles_reach__scr_stones,
        idx, cx, cx * w.chunk_w, (cx + 1) * w.chunk_w))
      if dir > 0 then
        new_cursor = (cx + 1) * w.chunk_w
      else
        new_cursor = cx * w.chunk_w
      end
    end
    s.cursor = new_cursor
    s.count = s.count + 1
  end
  return s.list
end

-- gml_Script_world_obstacles_at_chunk @ 0x14024f120
function world_obstacles_at_chunk(self, other, ch)
  local w = global.world
  local x1 = ch.cx * w.chunk_w
  local x2 = x1 + w.chunk_w
  local dir
  if x1 + w.chunk_w * 0.5 < w.start_x then
    dir = -1
  else
    dir = 1
  end
  local list = world_obstacles_reach(self, other, dir, dir > 0 and x2 or x1)
  local out = gm.array()
  local i = 0
  while i < gm.array_length(list) do
    if list[i].x2 > x1 and list[i].x1 < x2 then
      gm.array_push(out, list[i])
    end
    i = i + 1
  end
  return out
end

local world_paint_surface_wall_static = { inited = false, _slopes = nil }

-- gml_Script_world_paint_surface_wall @ 0x14024fdf0
function world_paint_surface_wall(self, other, ch)
  local st = world_paint_surface_wall_static
  if not st.inited then
    st.inited = true
    st._slopes = gm.array(0, 0, -1, 1, -0.5, 0.5, -0.5, 0.5)
  end
  local w = global.world
  local cx = ch.cx
  local min_w = gm.max(3, wld(self, other, "surface_wall_min_width_cells"))
  local max_w = gm.max(min_w, wld(self, other, "surface_wall_max_width_cells"))
  local width = min_w + gm.floor(world_hash01(self, other, cx, 1, 192) * (max_w - min_w + 1))
  local depth = gm.max(3, gm.round(wld(self, other, "surface_wall_depth_screens") * vis(self, other, "screen_height_px") / 48))
  local x0 = cx * w.chunk_w
  local c0 = gm.ceil(x0 / 48) + 1
  local c1 = gm.floor((x0 + w.chunk_w) / 48) - 2
  local top = gm.floor(w.surface_y / 48)
  local min_start = wld(self, other, "surface_wall_min_start_px")
  local clear = wld(self, other, "stone_water_clearance_px")
  local span = c1 - c0 - width + 2
  if span <= 0 then
    return
  end
  local attempt = 0
  while attempt < 4 do
    local x0c = c0 + gm.floor(world_hash01(self, other, cx, 3 + attempt, 194) * span)
    local rows = depth + 2
    local L = gm.array_create(rows, x0c)
    local R = gm.array_create(rows, x0c + width - 1)
    local min_deep = gm.max(3, wld(self, other, "surface_wall_min_deep_width_cells"))
    local lx = x0c
    local rx = x0c + width - 1
    local sl = 0
    local sr = 0
    local runl = 0
    local runr = 0
    local minc = x0c
    local maxc = x0c + width - 1
    local r = 3
    while r < rows do
      if runl <= 0 then
        runl = 3 + gm.floor(world_hash01(self, other, r, cx + attempt * 7919, 199) * 8)
        sl = st._slopes[gm.floor(world_hash01(self, other, r, cx + attempt * 7919, 200) * gm.array_length(st._slopes))]
      end
      if runr <= 0 then
        runr = 3 + gm.floor(world_hash01(self, other, r, cx + attempt * 7919, 201) * 8)
        sr = st._slopes[gm.floor(world_hash01(self, other, r, cx + attempt * 7919, 202) * gm.array_length(st._slopes))]
      end
      runl = runl - 1
      runr = runr - 1
      local nl = gm.clamp(lx + sl, c0, c1)
      local nr = gm.clamp(rx + sr, c0, c1)
      if gm.floor(nr + 0.5) - gm.floor(nl + 0.5) + 1 >= min_deep then
        lx = nl
        rx = nr
      else
        sl = 0
        sr = 0
      end
      L[r] = gm.floor(lx + 0.5)
      R[r] = gm.floor(rx + 0.5)
      minc = gm.min(minc, L[r])
      maxc = gm.max(maxc, R[r])
      r = r + 1
    end
    local x0px = minc * 48
    local x1px = (maxc + 1) * 48
    if not (x1px > w.start_x - min_start and x0px < w.start_x + min_start) then
      local cut = gm.struct({})
      local bl = L[depth - 1]
      local br = R[depth - 1]
      local mid = (bl + br) * 0.5
      local half = (br - bl + 1) * 0.5 + 0.5
      local bite = wld(self, other, "surface_wall_side_bite_chance")
      local c = minc
      while c <= maxc do
        local t = gm.clamp((c - mid) / half, -1, 1)
        local dd = gm.round((1 - gm.sqrt(gm.max(0, 1 - t * t))) * half)
        local inner = gm.max(3, depth - dd + gm.round((world_hash01(self, other, c, cx, 195) * 2 - 1) * 1.2))
        r = 0
        while r < gm.min(inner, rows) do
          if not (c < L[r] or c > R[r]) then
            local skip = false
            if r > 2 and R[r] - L[r] + 1 - 2 >= min_deep then
              if c == L[r] and world_hash01(self, other, r, cx, 197) < bite then
                skip = true
              elseif c == R[r] and world_hash01(self, other, r, cx, 198) < bite then
                skip = true
              end
            end
            if not skip then
              gm.variable_struct_set(cut, stone_cell_key(self, other, c, top + r), true)
            end
          end
          r = r + 1
        end
        c = c + 1
      end
      local tunnel_rows = gm.max(0, wld(self, other, "surface_wall_lake_tunnel_rows") - 1) * 0.5
      local i = 0
      while i < gm.array_length(ch.lakes) do
        local lk = ch.lakes[i]
        local lx0 = lk.body_x - clear
        local lx1 = lk.body_x + lk.body_w + clear
        local ly0 = lk.body_y - clear
        local ly1 = lk.body_y + lk.body_h + clear
        if not (lx1 <= x0px or lx0 >= x1px or ly1 <= top * 48 or ly0 >= (top + depth + 2) * 48) then
          local seed = cx * 13 + i
          local lcx = (lk.body_x + lk.body_w * 0.5) / 48
          local lcy = (lk.body_y + lk.body_h * 0.5) / 48
          local lrx = (lx1 - lx0) * 0.5 / 48 + 0.4
          local lry = (ly1 - ly0) * 0.5 / 48 + 0.4
          stone_cut_round(self, other, cut, lcx, lcy, lrx, lry, seed)
          local mx = lk.body_x + lk.body_w * 0.5
          local lrow = gm.floor(lcy)
          local rr = gm.clamp(lrow - top, 0, rows - 1)
          local Lr = L[rr]
          local Rr = R[rr]
          local go_left = mx - Lr * 48 <= (Rr + 1) * 48 - mx
          local ex
          if go_left then
            ex = Lr - 1.5
          else
            ex = Rr + 2.5
          end
          stone_cut_tunnel(self, other, cut, lcx, lcy, ex,
            lcy + (world_hash01(self, other, seed, 1, 231) - 0.5) * 3,
            gm.max(1, tunnel_rows + 0.6), seed)
        end
        i = i + 1
      end
      local blob = stone_blob_finish(self, other, cut, minc - 1, maxc + 1, top - 1, top + rows + 1, 6)
      if not (gm.array_length(blob.list) < width * 3) then
        i = 0
        while i < gm.array_length(blob.list) do
          if blob.list[i].row == top then
            gm.variable_struct_set(blob.keys, stone_cell_key(self, other, blob.list[i].col, top - 1), true)
          end
          i = i + 1
        end
        stone_dress_blob(self, other, ch, blob, gm.floor(world_hash01(self, other, cx, 9, 196) * 2))
        return
      end
    end
    attempt = attempt + 1
  end
end

-- gml_Script_stone_cut_round @ 0x140254d70
function stone_cut_round(self, other, keys, cx, cy, rx, ry, seed)
  local p1 = world_hash01(self, other, seed, 2, 232) * 6.283
  local p2 = world_hash01(self, other, seed, 3, 232) * 6.283
  local r = gm.floor(cy - ry * 1.3) - 1
  while r <= gm.ceil(cy + ry * 1.3) + 1 do
    local c = gm.floor(cx - rx * 1.3) - 1
    while c <= gm.ceil(cx + rx * 1.3) + 1 do
      local dx = (c + 0.5 - cx) / rx
      local dy = (r + 0.5 - cy) / ry
      local a = gm.arctan2(dy, dx)
      local k = 1 + gm.sin(3 * a + p1) * 0.16 + gm.sin(5 * a + p2) * 0.09
      if dx * dx + dy * dy <= k * k then
        gm.variable_struct_remove(keys, stone_cell_key(self, other, c, r))
      end
      c = c + 1
    end
    r = r + 1
  end
end

-- gml_Script_stone_cut_tunnel @ 0x140256880
function stone_cut_tunnel(self, other, keys, x0, y0, x1, y1, rad, seed)
  local len = gm.point_distance(x0, y0, x1, y1)
  local steps = gm.max(2, gm.ceil(len / 0.5))
  local dir = gm.point_direction(x0, y0, x1, y1)
  local ph = world_hash01(self, other, seed, 4, 232) * 6.283
  local i = 0
  while i <= steps do
    local t = i / steps
    local wob = gm.sin(t * 3.141592653589793) * gm.sin(t * 3.141592653589793 * 2 + ph) * 0.9
    local px = gm.lerp(x0, x1, t) + gm.lengthdir_x(wob, dir + 90)
    local py = gm.lerp(y0, y1, t) + gm.lengthdir_y(wob, dir + 90)
    local rr = rad * (gm.sin(t * 9 + ph) * 0.5 * 0.35 + 0.9750000000000001)
    local r = gm.floor(py - rr) - 1
    while r <= gm.ceil(py + rr) + 1 do
      local c = gm.floor(px - rr) - 1
      while c <= gm.ceil(px + rr) + 1 do
        local dx = c + 0.5 - px
        local dy = r + 0.5 - py
        if dx * dx + dy * dy <= rr * rr then
          gm.variable_struct_remove(keys, stone_cell_key(self, other, c, r))
        end
        c = c + 1
      end
      r = r + 1
    end
    i = i + 1
  end
end

-- gml_Script_maze_rng @ 0x140258d70
function maze_rng(self, other, m)
  m.n = m.n + 1
  return world_hash01(self, other, m.key, m.n, 220)
end

-- gml_Script_maze_rng_int @ 0x1402590c0
function maze_rng_int(self, other, m, lo, hi)
  return lo + gm.min(hi - lo, gm.floor(maze_rng(self, other, m) * (hi - lo + 1)))
end

-- gml_Script_maze_open_at @ 0x140259620
function maze_open_at(self, other, m, col, row)
  local lc = col - m.c1
  local lr = row - m.top
  if not (lc >= 0) or not (lc < m.w) or not (lr >= m.keep_top) or not (lr < m.h) then
    return
  end
  m.open[gm.floor(lc + lr * m.w)] = 1
end

-- gml_Script_maze_is_open @ 0x140259c50
function maze_is_open(self, other, m, col, row)
  local lc = col - m.c1
  local lr = row - m.top
  if not (lc >= 0) or not (lc < m.w) or not (lr >= 0) or not (lr < m.h) then
    return false
  end
  return m.open[gm.floor(lc + lr * m.w)]
end

-- gml_Script_maze_disc @ 0x14025a270
function maze_disc(self, other, m, cx, cy, r)
  local row = gm.floor(cy - r) - 1
  while row <= gm.ceil(cy + r) + 1 do
    local col = gm.floor(cx - r) - 1
    while col <= gm.ceil(cx + r) + 1 do
      local dx = col + 0.5 - cx
      local dy = row + 0.5 - cy
      if dx * dx + dy * dy <= r * r then
        maze_open_at(self, other, m, col, row)
      end
      col = col + 1
    end
    row = row + 1
  end
end

-- gml_Script____struct___231__maze_room__scr_stones @ 0x14025b1e0
function ___struct___231__maze_room__scr_stones(self, other, cx, cy, rx, ry)
  self.cx = cx
  self.cy = cy
  self.rx = rx
  self.ry = ry
end

-- gml_Script_maze_room @ 0x14025b470
function maze_room(self, other, m, cx, cy, rx, ry)
  cy = gm.clamp(cy, m.ymin + ry * 0.6, m.ymax + 2 - ry)
  cx = gm.clamp(cx, m.xmin + rx * 0.5, m.xmax - rx * 0.5)
  local p1 = maze_rng(self, other, m) * 6.283
  local p2 = maze_rng(self, other, m) * 6.283
  local row = gm.floor(cy - ry) - 2
  while row <= gm.ceil(cy + ry) + 2 do
    local col = gm.floor(cx - rx) - 2
    while col <= gm.ceil(cx + rx) + 2 do
      local dx = (col + 0.5 - cx) / rx
      local dy = (row + 0.5 - cy) / ry
      local a = gm.arctan2(dy, dx)
      local k = 1 + gm.sin(3 * a + p1) * 0.18 + gm.sin(5 * a + p2) * 0.1
      if dx * dx + dy * dy <= k * k then
        maze_open_at(self, other, m, col, row)
      end
      col = col + 1
    end
    row = row + 1
  end
  return gm.new(___struct___231__maze_room__scr_stones, cx, cy, rx, ry)
end

-- gml_Script_maze_tunnel @ 0x14025d780
function maze_tunnel(self, other, m, x, y, dir, len, r0, r1, target, free)
  if gm.is_undefined(target) then
    target = gm.undefined
  end
  if gm.is_undefined(free) then
    free = false
  end
  local ph = maze_rng(self, other, m) * 6.283
  local fr = 0.15 + maze_rng(self, other, m) * 0.2
  local drift = 0
  local bulge = 0
  local bulge_amp = 0
  local pts = gm.array()
  local steps = gm.floor(len / 0.6)
  local bulge_ch = wld(self, other, "maze_bulge_chance")
  local i = 0
  while i < steps do
    local rr = gm.lerp(r0, r1, (gm.sin(i * 0.6 * fr + ph) + 1) * 0.5)
    if bulge <= 0 then
      if maze_rng(self, other, m) < bulge_ch then
        bulge = 6 + maze_rng(self, other, m) * 6
        bulge_amp = 1 + maze_rng(self, other, m) * 1.6
      end
    end
    if bulge > 0 then
      bulge = bulge - 0.6
      rr = rr + bulge_amp * gm.sin(3.141592653589793 * gm.min(1, bulge / 6))
    end
    maze_disc(self, other, m, x, y, rr)
    gm.array_push(pts, gm.array(x, y))
    drift = drift * 0.85 + (maze_rng(self, other, m) - 0.5) * 14
    dir = dir + drift
    if not gm.is_undefined(target) then
      local td = gm.point_direction(x, y, target[0], target[1])
      dir = dir + gm.angle_difference(td, dir) * 0.12
      if gm.point_distance(x, y, target[0], target[1]) < 1 then
        return pts
      end
    end
    local nx = x + gm.lengthdir_x(0.6, dir)
    local ny = y + gm.lengthdir_y(0.6, dir)
    if not gm.bool(free) then
      if nx < m.xmin or nx > m.xmax then
        dir = 180 - dir
        nx = x
      end
    end
    if ny < m.ymin or ny > m.ymax then
      dir = -dir
      ny = y
    end
    x = nx
    y = ny
    i = i + 1
  end
  return pts
end

-- gml_Script_maze_distance @ 0x1402604d0
function maze_distance(self, other, m, seeds)
  if gm.is_undefined(seeds) then
    seeds = gm.undefined
  end
  local n = m.w * m.h
  local dist = gm.array_create(n, -1)
  local q = gm.array_create(n + 4, 0)
  local head = 0
  local tail = 0
  -- m[$ "inside"] (struct_get_from_hash with the hash of "inside")
  local has_inside = gm.is_array(m.inside)
  local i = 0
  while i < n do
    local cx = gm.mod(i, m.w)
    local cy = gm.div(i, m.w)
    local is_seed
    if gm.bool(m.open[i]) and (gm.is_undefined(seeds) or not gm.bool(seeds[i])) then
      is_seed = true
    elseif cx == 0 or cx == m.w - 1 or cy == 0 or cy == m.h - 1 then
      is_seed = true
    else
      is_seed = false
      if gm.bool(has_inside) then
        is_seed = not gm.bool(m.inside[i])
      end
    end
    if is_seed then
      dist[i] = 0
      q[tail] = i
      tail = tail + 1
    end
    i = i + 1
  end
  while head < tail do
    i = q[head]
    local cx = gm.mod(i, m.w)
    local cy = gm.div(i, m.w)
    head = head + 1
    if cx > 0 and dist[i - 1] < 0 then
      dist[i - 1] = dist[i] + 1
      q[tail] = i - 1
      tail = tail + 1
    end
    if cx < m.w - 1 and dist[i + 1] < 0 then
      dist[i + 1] = dist[i] + 1
      q[tail] = i + 1
      tail = tail + 1
    end
    if cy > 0 and dist[i - m.w] < 0 then
      dist[i - m.w] = dist[i] + 1
      q[tail] = i - m.w
      tail = tail + 1
    end
    if cy < m.h - 1 and dist[i + m.w] < 0 then
      dist[i + m.w] = dist[i] + 1
      q[tail] = i + m.w
      tail = tail + 1
    end
  end
  return dist
end

-- gml_Script_maze_inside @ 0x1402653b0
function maze_inside(self, other, m, lc, lr)
  -- m[$ "inside"] (struct_get_from_hash with the hash of "inside")
  if not gm.is_array(m.inside) then
    return true
  end
  return gm.bool(m.inside[gm.floor(lc + lr * m.w)])
end

-- gml_Script_maze_dig_out @ 0x1402630e0
function maze_dig_out(self, other, m, dist, lc, lr)
  local tmin = wld(self, other, "maze_tunnel_min_cells")
  local ph = maze_rng(self, other, m) * 6
  local k = 0
  while k < 120 do
    if not (dist[gm.floor(lc + lr * m.w)] > 0) then
      return
    end
    local bc = lc
    local br = lr
    local bd = dist[gm.floor(lc + lr * m.w)]
    local dy = -1
    while dy <= 1 do
      local dx = -1
      while dx <= 1 do
        local nc = lc + dx
        local nr = lr + dy
        if nc >= 0 and nr >= 0 and nc < m.w and nr < m.h
          and dist[gm.floor(nc + nr * m.w)] >= 0 and dist[gm.floor(nc + nr * m.w)] < bd then
          bd = dist[gm.floor(nc + nr * m.w)]
          bc = nc
          br = nr
        end
        dx = dx + 1
      end
      dy = dy + 1
    end
    if bc == lc and br == lr then
      return
    end
    lc = bc
    lr = br
    maze_disc(self, other, m,
      m.c1 + lc + 0.5 + gm.sin(k * 0.5 + ph) * 0.6,
      m.top + lr + 0.5,
      gm.lerp(tmin * 0.9, tmin * 1.5, (gm.sin(k * 0.3 + ph) + 1) * 0.5))
    k = k + 1
  end
end
