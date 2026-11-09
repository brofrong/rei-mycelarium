local gm = require("gm")

-- gml_Script_maze_cave_hollows @ 0x1402657c0
function maze_cave_hollows(self, other, m, x1, y1, x2, y2)
  caves_extend(self, other, x1)
  caves_extend(self, other, x2)
  local rects = world_cave_rects(self, other)
  local pad = wld(self, other, "caves_clearance_px") + 24
  local i = 0
  while i < gm.array_length(rects) do
    local r = rects[i]
    if r.x2 <= x1 or r.x1 >= x2 or r.y2 <= y1 or r.y1 >= y2 then
      -- continue
    else
      local cx = (r.x1 + r.x2) * 0.5 / 48
      local cy = (r.y1 + r.y2) * 0.5 / 48
      local rx = ((r.x2 - r.x1) * 0.5 + pad) / 48
      local ry = ((r.y2 - r.y1) * 0.5 + pad) / 48
      local phase = maze_rng(self, other, m) * 6.283
      local mask = gm.array_create(m.w * m.h, false)
      local yy = gm.floor(cy - ry * 1.2) - 1
      while yy <= gm.ceil(cy + ry * 1.2) + 1 do
        local xx = gm.floor(cx - rx * 1.2) - 1
        while xx <= gm.ceil(cx + rx * 1.2) + 1 do
          local dx = (xx + 0.5 - cx) / rx
          local dy = (yy + 0.5 - cy) / ry
          local a = gm.arctan2(dy, dx)
          local k = 1.05 + 0.08 * gm.sin(4 * a + phase)
          if gm.power(gm.abs(dx), 4) + gm.power(gm.abs(dy), 4) <= gm.power(k, 4) then
            local lx = xx - m.c1
            local ly = yy - m.top
            if lx >= 0 and ly >= 0 and lx < m.w and ly < m.h then
              m.open[ly * m.w + lx] = true
              mask[ly * m.w + lx] = true
            end
          end
          xx = xx + 1
        end
        yy = yy + 1
      end
      local sx = gm.clamp(gm.floor(cx) - m.c1, 0, m.w - 1)
      local sy = gm.clamp(gm.floor(cy) - m.top, 0, m.h - 1)
      maze_dig_out(self, other, m, maze_distance(self, other, m, mask), sx, sy)
    end
    i = i + 1
  end
end

-- gml_Script_maze_hollows @ 0x1402695b0
function maze_hollows(self, other, m, out)
  local max_cells = wld(self, other, "maze_solid_max_cells")
  local pass = 0
  while pass < 3 do
    local dist = maze_distance(self, other, m, gm.undefined)
    local made = 0
    local n = 0
    while n < 16 do
      local best = -1
      local best_d = max_cells
      local c = 0
      while c < m.w * m.h do
        local cy = m.top + gm.div(c, m.w)
        if dist[c] > best_d and cy >= m.ymin and cy <= m.ymax + 2 then
          best_d = dist[c]
          best = c
        end
        c = c + 1
      end
      if best < 0 then
        break
      end
      local hx = gm.mod(best, m.w)
      local hy = gm.div(best, m.w)
      local water = maze_rng(self, other, m) < wld(self, other, "maze_hollow_water_share")
      local room
      if water then
        room = maze_room(self, other, m, m.c1 + hx + 0.5, m.top + hy + 0.5, 3, 2.2)
      else
        room = maze_room(self, other, m, m.c1 + hx + 0.5, m.top + hy + 0.5,
          2.8 + maze_rng(self, other, m) * 2.4, 2 + maze_rng(self, other, m) * 1.6)
      end
      room.water = water
      room.food_min = wld(self, other, "maze_hollow_food_min")
      room.food_max = wld(self, other, "maze_hollow_food_max")
      gm.array_push(out, room)
      made = made + 1
      maze_dig_out(self, other, m, dist, hx, hy)
      local rr = max_cells + 4
      local dy = -rr
      while dy <= rr do
        local dx = -rr
        while dx <= rr do
          local xx = hx + dx
          local yy = hy + dy
          if xx >= 0 and yy >= 0 and xx < m.w and yy < m.h then
            dist[yy * m.w + xx] = gm.min(dist[yy * m.w + xx], gm.max(0, gm.abs(dy) + gm.abs(dx) - 3))
          end
          dx = dx + 1
        end
        dy = dy + 1
      end
      n = n + 1
    end
    if made == 0 then
      break
    end
    pass = pass + 1
  end
end

-- gml_Script_maze_branches @ 0x14026c9c0
function maze_branches(self, other, m, cells, out)
  local min_cells = wld(self, other, "maze_tunnel_min_cells")
  local max_cells = wld(self, other, "maze_tunnel_max_cells")
  local n = maze_rng_int(self, other, m, wld(self, other, "maze_branches_min"), wld(self, other, "maze_branches_max"))
  local i = 0
  while i < n do
    if gm.array_length(cells) < 1 then
      break
    end
    local hub = cells[gm.floor(maze_rng(self, other, m) * gm.array_length(cells))]
    local path = maze_tunnel(self, other, m, hub[0], hub[1], maze_rng(self, other, m) * 360,
      8 + maze_rng(self, other, m) * 22, min_cells * 0.8, max_cells * 0.8, gm.undefined, gm.undefined)
    if not gm.is_array(path) or gm.array_length(path) < 1 then
      -- continue
    elseif maze_rng(self, other, m) >= wld(self, other, "maze_chamber_chance") then
      -- continue
    else
      local e = path[gm.array_length(path) - 1]
      local ex = gm.clamp(gm.floor(e[0]) - m.c1, 0, m.w - 1)
      local ey = gm.clamp(gm.floor(e[1]) - m.top, 0, m.h - 1)
      if not gm.bool(maze_inside(self, other, m, ex, ey)) then
        -- continue
      else
        local water = maze_rng(self, other, m) < wld(self, other, "maze_water_share")
        local room
        if water then
          room = maze_room(self, other, m, e[0], e[1], 3, 2.2)
        else
          room = maze_room(self, other, m, e[0], e[1],
            2.4 + maze_rng(self, other, m) * 2, 1.8 + maze_rng(self, other, m) * 1.3)
        end
        room.water = water
        room.food_min = wld(self, other, "maze_food_min")
        room.food_max = wld(self, other, "maze_food_max")
        gm.array_push(out, room)
      end
    end
    i = i + 1
  end
end

-- gml_Script____struct___232__maze_finish__scr_stones @ 0x14026f7e0
function ___struct___232__maze_finish__scr_stones(self, other, stone_cells)
  self.stone_cells = stone_cells
  self.stone_map = gm.struct({})
end

-- gml_Script____struct___234__maze_finish__scr_stones @ 0x14026fa00
function ___struct___234__maze_finish__scr_stones(self, other, cx, seed, x, y, body_x, body_y, body_w, body_h, amount, full, flares)
  self.cx = cx
  self.seed = seed
  self.kind = 2
  self.x = x
  self.y = y
  self.body_x = body_x
  self.body_y = body_y
  self.body_w = body_w
  self.body_h = body_h
  self.amount = amount
  self.full = full
  self.connected = false
  self.flares = flares
  self.flare_seq = 0
end

-- gml_Script____struct___235__maze_finish__scr_stones @ 0x140270070
function ___struct___235__maze_finish__scr_stones(self, other, x, y, kind, cell)
  self.x = x
  self.y = y
  self.kind = kind
  self.cell = cell
end

-- gml_Script____struct___236__maze_finish__scr_stones @ 0x140270340
function ___struct___236__maze_finish__scr_stones(self, other, entries, foods, lakes)
  self.entries = entries
  self.foods = foods
  self.lakes = lakes
end

-- gml_Script_maze_finish @ 0x140270570
function maze_finish(self, other, m, cut, rooms, c0, c1, r0, r1, seal_top)
  local w = global.world
  local blob = stone_blob_finish(self, other, cut, c0, c1, r0, r1, 6)
  if gm.bool(seal_top) then
    local i = 0
    while i < gm.array_length(blob.list) do
      if blob.list[i].row == m.top then
        gm.variable_struct_set(blob.keys, stone_cell_key(self, other, blob.list[i].col, m.top - 1), true)
      end
      i = i + 1
    end
  end
  local st = gm.new(___struct___232__maze_finish__scr_stones, gm.array())
  stone_dress_blob(self, other, st, blob, maze_rng_int(self, other, m, 0, 1))
  local foods = gm.array()
  local lakes = gm.array()
  local kinds = world_food_kinds(self, other)
  local lake = lake_layout(self, other)[2]
  local i = 0
  while i < gm.array_length(rooms) do
    local r = rooms[i]
    local x0 = (r.cx - r.rx * 0.7) * 48
    local x1 = (r.cx + r.rx * 0.7) * 48
    local y0 = (r.cy - r.ry * 0.6) * 48
    local y1 = (r.cy + r.ry * 0.6) * 48
    local made_lake = false
    if gm.bool(r.water) then
      local lx = gm.floor(r.cx * 48 - lake.body_w * 0.5)
      local ly = gm.floor(r.cy * 48 - lake.body_h * 0.5)
      if gm.floor(lx / w.chunk_w) == gm.floor((lx + lake.body_w - 1) / w.chunk_w) then
        local amount = wld(self, other, "maze_water_amount")
        gm.array_push(lakes, gm.new(___struct___234__maze_finish__scr_stones,
          gm.floor(lx / w.chunk_w), 700 + i, lx - lake.body_x, ly - lake.body_y, lx, ly,
          lake.body_w, lake.body_h, amount, amount, gm.undefined))
        made_lake = true
      end
    end
    if not made_lake then
      local n = maze_rng_int(self, other, m, r.food_min, r.food_max)
      local j = 0
      while j < n do
        local kind = maze_rng_int(self, other, m, 0, kinds - 1)
        local tx = maze_rng(self, other, m)
        local ty = maze_rng(self, other, m)
        local cells = world_food_cells(self, other, kind) - 1
        gm.array_push(foods, gm.new(___struct___235__maze_finish__scr_stones,
          gm.lerp(x0, x1, tx), gm.lerp(y0, y1, ty), kind, maze_rng_int(self, other, m, 0, cells)))
        j = j + 1
      end
    end
    i = i + 1
  end
  return gm.new(___struct___236__maze_finish__scr_stones, st.stone_cells, foods, lakes)
end

-- gml_Script_maze_data @ 0x140274b00
function maze_data(self, other, part)
  if part.kind == "cluster" then
    return world_cluster_build(self, other, part)
  else
    return world_maze_build(self, other, part)
  end
end

-- gml_Script____struct___237__world_maze_build__scr_stones @ 0x140274f00
function ___struct___237__world_maze_build__scr_stones(self, other, key, c1, c2, top, w, h, xmin, xmax, ymin, ymax, open)
  self.key = key
  self.n = 0
  self.c1 = c1
  self.c2 = c2
  self.top = top
  self.w = w
  self.h = h
  self.keep_top = 3
  self.xmin = xmin
  self.xmax = xmax
  self.ymin = ymin
  self.ymax = ymax
  self.open = open
end

-- gml_Script_world_maze_build @ 0x140275510
function world_maze_build(self, other, part)
  if not gm.is_undefined(part.data) then
    return part.data
  end
  local w = global.world
  local top = gm.floor(w.surface_y / 48)
  local depth = gm.max(24, gm.round(wld(self, other, "surface_wall_depth_screens") * vis(self, other, "screen_height_px") / 48))
  local bottom = top + depth - 1
  local width = part.c2 - part.c1 + 1
  local m = gm.new(___struct___237__world_maze_build__scr_stones, part.key * 31 + 7, part.c1, part.c2, top, width,
    depth + 2, part.c1 + 2, part.c2 - 2, top + 4, bottom - 6, gm.array())
  m.open = gm.array_create(m.w * m.h, false)
  local tmin = wld(self, other, "maze_tunnel_min_cells")
  local tmax = wld(self, other, "maze_tunnel_max_cells")
  local rooms = gm.array()
  local x = m.c1 - 2
  local y = gm.lerp(m.ymin, m.ymax, maze_rng(self, other, m))
  local cells = gm.array()
  local segs = maze_rng_int(self, other, m, 2, 4)
  local s = 0
  while s <= segs do
    local target
    if s < segs then
      local t = (s + 1) / (segs + 1)
      local ry = maze_rng(self, other, m)
      target = gm.array(gm.lerp(m.xmin, m.xmax, t), gm.lerp(m.ymin, m.ymax, ry))
    else
      local ry = maze_rng(self, other, m)
      target = gm.array(m.c2 + 2, gm.lerp(m.ymin, m.ymax, ry))
    end
    local path = maze_tunnel(self, other, m, x, y, gm.point_direction(x, y, target[0], target[1]), 400,
      tmin, tmax * 1.1, target, true)
    if gm.is_array(path) and gm.array_length(path) > 0 then
      x = path[gm.array_length(path) - 1][0]
      y = path[gm.array_length(path) - 1][1]
      cells = gm.array_concat(cells, path)
    end
    s = s + 1
  end
  local extra = maze_rng_int(self, other, m, wld(self, other, "maze_extra_entrances_min"), wld(self, other, "maze_extra_entrances_max"))
  local e = 0
  while e < extra do
    local left = maze_rng(self, other, m) < 0.5
    local ty = maze_rng(self, other, m)
    local sx
    if left then
      sx = m.c1 - 2
    else
      sx = m.c2 + 2
    end
    local sy = gm.lerp(m.ymin, m.ymax, ty)
    local dir
    if left then
      dir = 0
    else
      dir = 180
    end
    maze_tunnel(self, other, m, sx, sy, dir, 10 + maze_rng(self, other, m) * 25, tmin * 0.9, tmax * 0.85,
      gm.undefined, true)
    e = e + 1
  end
  maze_branches(self, other, m, cells, rooms)
  maze_cave_hollows(self, other, m, part.c1 * 48, top * 48, (part.c2 + 1) * 48, (bottom + 1) * 48)
  maze_hollows(self, other, m, rooms)
  local cut = gm.struct({})
  local mid = (m.c1 + m.c2) * 0.5
  local half = width * 0.5 + 0.5
  local bite = wld(self, other, "surface_wall_side_bite_chance")
  local c = m.c1
  while c <= m.c2 do
    local t = gm.clamp((c - mid) / half, -1, 1)
    local rows = depth - gm.round((1 - gm.sqrt(gm.max(0, 1 - t * t))) * 4) + maze_rng_int(self, other, m, -1, 1)
    local r = 0
    while r < rows do
      if r > 2 and (c == m.c1 or c == m.c2) and maze_rng(self, other, m) < bite then
        -- continue
      elseif not gm.bool(maze_is_open(self, other, m, c, top + r)) then
        gm.variable_struct_set(cut, stone_cell_key(self, other, c, top + r), true)
      end
      r = r + 1
    end
    c = c + 1
  end
  part.data = maze_finish(self, other, m, cut, rooms, m.c1 - 1, m.c2 + 1, top - 1, bottom + 3, true)
  return part.data
end

-- gml_Script____struct___239__world_rock_clusters__scr_stones @ 0x14027bd90
function ___struct___239__world_rock_clusters__scr_stones(self, other, top, bottom, every)
  self.id = "clay"
  self.top = top
  self.bottom = bottom
  self.salt = 501
  self.every = every
end

-- gml_Script____struct___240__world_rock_clusters__scr_stones @ 0x14027c070
function ___struct___240__world_rock_clusters__scr_stones(self, other, top, bottom, every)
  self.id = "deep"
  self.top = top
  self.bottom = bottom
  self.salt = 502
  self.every = every
end

-- gml_Script____struct___241__world_rock_clusters__scr_stones @ 0x14027c350
function ___struct___241__world_rock_clusters__scr_stones(self, other, id, tier, key, col, row, r, c1, c2, x1, y1, x2, y2, data)
  self.kind = "cluster"
  self.id = id
  self.tier = tier
  self.key = key
  self.col = col
  self.row = row
  self.r = r
  self.c1 = c1
  self.c2 = c2
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.data = data
end

-- gml_Script_world_rock_clusters @ 0x14027c9e0
local world_rock_clusters_static = { _init = false }
function world_rock_clusters(self, other)
  local st = world_rock_clusters_static
  if not st._init then
    st._init = true
    st._cache = gm.undefined
    st._key = ""
  end
  local w = global.world
  local key = gm.string(w.start_x) .. ":" .. gm.string(w.seed)
  if not gm.is_undefined(st._cache) and st._key == key then
    return st._cache
  end
  local out = gm.array()
  st._key = key
  st._cache = out
  if wld(self, other, "rock_clusters_enabled") <= 0 then
    return out
  end
  local sw = vis(self, other, "screen_width_px")
  local tiers = gm.array(
    gm.new(___struct___239__world_rock_clusters__scr_stones, world_dense_y(self, other), world_deep_y(self, other),
      wld(self, other, "rock_cluster_every_x")),
    gm.new(___struct___240__world_rock_clusters__scr_stones, world_deep_y(self, other),
      world_ocean_y(self, other) - wld(self, other, "ocean_clear_margin_px"), wld(self, other, "rock_cluster_deep_every_x")))
  local minc = wld(self, other, "rock_cluster_min_cells")
  local maxc = wld(self, other, "rock_cluster_max_cells")
  local t = 0
  while t < gm.array_length(tiers) do
    local tier = tiers[t]
    local s = -wld(self, other, "rock_cluster_per_side")
    while s <= wld(self, other, "rock_cluster_per_side") do
      local r = gm.floor(gm.lerp(minc, maxc + 1, world_hash01(self, other, s, 1, tier.salt)))
      local jitter = wld(self, other, "rock_cluster_jitter_x") * tier.every / gm.max(0.1, wld(self, other, "rock_cluster_every_x"))
      local cx = w.start_x + sw * (s * tier.every + (world_hash01(self, other, s, 2, tier.salt) - 0.5) * 2 * jitter)
      local pad = (r + 2) * 48
      local ymin = tier.top + pad + wld(self, other, "caves_margin_px")
      local ymax = tier.bottom - pad - wld(self, other, "caves_margin_px")
      if ymax < ymin then
        -- continue
      else
        local cy = gm.lerp(ymin, ymax, world_hash01(self, other, s, 3, tier.salt))
        local col = gm.floor(cx / 48)
        local row = gm.floor(cy / 48)
        gm.array_push(out, gm.new(___struct___241__world_rock_clusters__scr_stones,
          tier.id .. "_" .. gm.string(s), tier.id, 90000 + t * 1000 + s, col, row, r,
          col - r - 3, col + r + 3, (col - r - 3) * 48, (row - r - 3) * 48, (col + r + 4) * 48, (row + r + 4) * 48,
          gm.undefined))
      end
      s = s + 1
    end
    t = t + 1
  end
  return out
end

-- gml_Script____struct___242__world_cluster_build__scr_stones @ 0x1402812a0
function ___struct___242__world_cluster_build__scr_stones(self, other, key, c1, c2, top, w, h, xmin, xmax, ymin, ymax, open, inside)
  self.key = key
  self.n = 0
  self.c1 = c1
  self.c2 = c2
  self.top = top
  self.w = w
  self.h = h
  self.keep_top = 0
  self.xmin = xmin
  self.xmax = xmax
  self.ymin = ymin
  self.ymax = ymax
  self.open = open
  self.inside = inside
end

-- gml_Script_world_cluster_build @ 0x140281920
function world_cluster_build(self, other, cl)
  if not gm.is_undefined(cl.data) then
    return cl.data
  end
  local n = cl.r * 2 + 7
  local m = gm.new(___struct___242__world_cluster_build__scr_stones, cl.key * 31 + 11,
    cl.col - cl.r - 3, cl.col + cl.r + 3, cl.row - cl.r - 3, n, n,
    cl.col - cl.r - 1, cl.col + cl.r + 1, cl.row - cl.r - 1, cl.row + cl.r + 1, gm.array(), gm.array())
  m.open = gm.array_create(n * n, false)
  m.inside = gm.array_create(n * n, false)
  local phase1 = maze_rng(self, other, m) * 6.283
  local phase2 = maze_rng(self, other, m) * 6.283
  local j = 0
  while j < n do
    local i = 0
    while i < n do
      local dx = m.c1 + i + 0.5 - (cl.col + 0.5)
      local dy = m.top + j + 0.5 - (cl.row + 0.5)
      local a = gm.arctan2(dy, dx)
      local rr = cl.r * (0.07 * gm.sin(5 * a + phase2) + 0.12 * gm.sin(3 * a + phase1) + 1)
      m.inside[j * n + i] = dx * dx + dy * dy <= rr * rr
      i = i + 1
    end
    j = j + 1
  end
  local tmin = wld(self, other, "maze_tunnel_min_cells")
  local tmax = wld(self, other, "maze_tunnel_max_cells")
  local rooms = gm.array()
  local cx = cl.col + 0.5
  local cy = cl.row + 0.5
  local a_in = maze_rng(self, other, m) * 360
  local a_out = a_in + 180 + (maze_rng(self, other, m) - 0.5) * 80
  local p_in = gm.array(cx + gm.lengthdir_x(cl.r + 2, a_in), cy + gm.lengthdir_y(cl.r + 2, a_in))
  local mx = cx + (maze_rng(self, other, m) - 0.5) * cl.r * 0.6
  local p_mid = gm.array(mx, cy + (maze_rng(self, other, m) - 0.5) * cl.r * 0.6)
  local p_out = gm.array(cx + gm.lengthdir_x(cl.r + 2, a_out), cy + gm.lengthdir_y(cl.r + 2, a_out))
  local path = maze_tunnel(self, other, m, p_in[0], p_in[1],
    gm.point_direction(p_in[0], p_in[1], p_mid[0], p_mid[1]), 200, tmin, tmax * 1.2, p_mid, true)
  local last
  if gm.is_array(path) and gm.array_length(path) > 0 then
    last = path[gm.array_length(path) - 1]
  else
    last = p_mid
  end
  path = gm.array_concat(path, maze_tunnel(self, other, m, last[0], last[1],
    gm.point_direction(last[0], last[1], p_out[0], p_out[1]), 200, tmin, tmax * 1.2, p_out, true))
  local extra = maze_rng_int(self, other, m, wld(self, other, "maze_extra_entrances_min"), wld(self, other, "maze_extra_entrances_max"))
  local e = 0
  while e < extra do
    local a = maze_rng(self, other, m) * 360
    local sx = cx + gm.lengthdir_x(cl.r + 2, a)
    local sy = cy + gm.lengthdir_y(cl.r + 2, a)
    local dir = a + 180 + (maze_rng(self, other, m) - 0.5) * 60
    maze_tunnel(self, other, m, sx, sy, dir, cl.r * (0.5 + maze_rng(self, other, m) * 0.6), tmin * 0.9, tmax * 0.85,
      gm.undefined, true)
    e = e + 1
  end
  maze_branches(self, other, m, path, rooms)
  maze_cave_hollows(self, other, m, cl.x1, cl.y1, cl.x2, cl.y2)
  maze_hollows(self, other, m, rooms)
  local cut = gm.struct({})
  j = 0
  while j < n do
    local i = 0
    while i < n do
      local k = j * n + i
      if gm.bool(m.inside[k]) and not gm.bool(m.open[k]) then
        gm.variable_struct_set(cut, stone_cell_key(self, other, m.c1 + i, m.top + j), true)
      end
      i = i + 1
    end
    j = j + 1
  end
  cl.data = maze_finish(self, other, m, cut, rooms, m.c1 - 1, m.c2 + 1, m.top - 1, m.top + n + 1, false)
  return cl.data
end

-- gml_Script_world_paint_clusters_late @ 0x140288fe0
function world_paint_clusters_late(self, other, ch)
  local list = world_rock_clusters(self, other)
  local w = global.world
  local x0 = ch.cx * w.chunk_w
  local x1 = x0 + w.chunk_w
  local y1 = w.surface_y + ch.rows * w.tile
  if not gm.variable_struct_exists(ch, "clusters_done") then
    ch.clusters_done = gm.struct({})
  end
  local i = 0
  while i < gm.array_length(list) do
    local cl = list[i]
    if cl.x2 <= x0 or cl.x1 >= x1 or cl.y1 >= y1 then
      -- continue
    else
      local done = gm.variable_struct_get(ch.clusters_done, cl.id)
      if gm.is_undefined(done) then
        done = false
      end
      if gm.bool(done) then
        -- continue
      else
        gm.variable_struct_set(ch.clusters_done, cl.id, true)
        world_paint_maze_part(self, other, ch, cl)
      end
    end
    i = i + 1
  end
end

-- gml_Script_world_paint_clusters @ 0x14028a050
function world_paint_clusters(self, other, ch, layer)
  local w = global.world
  local list = world_rock_clusters(self, other)
  local x0 = ch.cx * w.chunk_w
  local x1 = x0 + w.chunk_w
  local y0 = world_layer_top(self, other, layer)
  local y1 = world_layer_bottom(self, other, layer)
  if not gm.variable_struct_exists(ch, "clusters_done") then
    ch.clusters_done = gm.struct({})
  end
  local i = 0
  while i < gm.array_length(list) do
    local cl = list[i]
    if cl.x2 <= x0 or cl.x1 >= x1 or cl.y2 <= y0 or cl.y1 >= y1 then
      -- continue
    else
      local done = gm.variable_struct_get(ch.clusters_done, cl.id)
      if gm.is_undefined(done) then
        done = false
      end
      if gm.bool(done) then
        -- continue
      else
        gm.variable_struct_set(ch.clusters_done, cl.id, true)
        world_paint_maze_part(self, other, ch, cl)
      end
    end
    i = i + 1
  end
end

-- gml_Script____struct___246__world_paint_maze_part__scr_stones @ 0x14028b090
function ___struct___246__world_paint_maze_part__scr_stones(self, other, x, y, kind, cell)
  self.x = x
  self.y = y
  self.kind = kind
  self.cell = cell
  self.collected = false
  self.maze = true
end

-- gml_Script_world_paint_maze_part @ 0x14028b400
function world_paint_maze_part(self, other, ch, part)
  local w = global.world
  local d = maze_data(self, other, part)
  local x0 = ch.cx * w.chunk_w
  local x1 = x0 + w.chunk_w
  local i = 0
  while i < gm.array_length(d.entries) do
    local e = d.entries[i]
    local ex = e.col * 48
    if ex < x1 and ex + 48 > x0 then
      gm.array_push(ch.stone_cells, e)
      gm.variable_struct_set(ch.stone_map, stone_cell_key(self, other, e.col, e.row), e)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(d.foods) do
    local f = d.foods[i]
    if f.x >= x0 and f.x < x1 then
      gm.array_push(ch.foods, gm.new(___struct___246__world_paint_maze_part__scr_stones, f.x, f.y, f.kind, f.cell))
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(d.lakes) do
    local lk = d.lakes[i]
    if lk.cx == ch.cx then
      gm.array_push(ch.lakes, gm.variable_clone(lk))
    end
    i = i + 1
  end
end

-- gml_Script_world_draw_stone_deposits @ 0x14028cb60
function world_draw_stone_deposits(self, other, ch, vx, vy, vw, vh)
  local w = global.world
  local cells = ch.stone_cells
  local x0 = ch.cx * w.chunk_w
  local x1 = x0 + w.chunk_w
  local i = 0
  while i < gm.array_length(cells) do
    local c = cells[i]
    local px = c.col * 48
    local py = c.row * 48
    if px + 48 < vx or px > vx + vw then
      -- continue
    elseif py + 48 < vy or py > vy + vh then
      -- continue
    else
      local l = gm.max(px, x0)
      local r = gm.min(px + 48, x1)
      if r <= l then
        -- continue
      else
        local deep = c.deep
        if gm.is_undefined(deep) then
          deep = py + 48 > world_deep_y(self, other)
        end
        local col
        if gm.bool(deep) then
          col = vis(self, other, "deep_stone_colour")
        else
          col = 16777215
        end
        gm.draw_sprite_part_ext(stone_tile, 0, c.tc * 64 + (l - px), c.tr * 64, r - l, 48, l, py, 1, 1, col, 1)
      end
    end
    i = i + 1
  end
end

-- gml_GlobalScript_scr_stones @ 0x14028e590
function gml_GlobalScript_scr_stones(self, other)
  self.stones_init = gm.method(self, stones_init)
  self.stones_ready = gm.method(self, stones_ready)
  self.stones_slice_sheet = gm.method(self, stones_slice_sheet)
  self.stones_wake = gm.method(self, stones_wake)
  self.stones_shove = gm.method(self, stones_shove)
  self.stones_separate = gm.method(self, stones_separate)
  self.stones_step = gm.method(self, stones_step)
  self.world_paint_stones = gm.method(self, world_paint_stones)
  self.world_draw_stones = gm.method(self, world_draw_stones)
  self.stone_role_tiles = gm.method(self, stone_role_tiles)
  self.stone_inner_corner_tile = gm.method(self, stone_inner_corner_tile)
  self.stone_tiles_init = gm.method(self, stone_tiles_init)
  self.stone_tiles_ready = gm.method(self, stone_tiles_ready)
  self.stone_tile_solid_at = gm.method(self, stone_tile_solid_at)
  self.stone_cell_key = gm.method(self, stone_cell_key)
  self.stone_cell_at = gm.method(self, stone_cell_at)
  self.stone_solid_at = gm.method(self, stone_solid_at)
  self.world_stone_covers = gm.method(self, world_stone_covers)
  self.stone_blob_cells = gm.method(self, stone_blob_cells)
  self.stone_blob_finish = gm.method(self, stone_blob_finish)
  self.stone_blob_largest_piece = gm.method(self, stone_blob_largest_piece)
  self.stone_blob_fits = gm.method(self, stone_blob_fits)
  self.stone_pick_tile = gm.method(self, stone_pick_tile)
  self.stone_tile_for = gm.method(self, stone_tile_for)
  self.stone_redress_after_cut = gm.method(self, stone_redress_after_cut)
  self.stone_dress_blob = gm.method(self, stone_dress_blob)
  self.world_paint_stone_deposits = gm.method(self, world_paint_stone_deposits)
  self.world_obstacles_reach = gm.method(self, world_obstacles_reach)
  self.world_obstacles_at_chunk = gm.method(self, world_obstacles_at_chunk)
  self.world_paint_surface_wall = gm.method(self, world_paint_surface_wall)
  self.stone_cut_round = gm.method(self, stone_cut_round)
  self.stone_cut_tunnel = gm.method(self, stone_cut_tunnel)
  self.maze_rng = gm.method(self, maze_rng)
  self.maze_rng_int = gm.method(self, maze_rng_int)
  self.maze_open_at = gm.method(self, maze_open_at)
  self.maze_is_open = gm.method(self, maze_is_open)
  self.maze_disc = gm.method(self, maze_disc)
  self.maze_room = gm.method(self, maze_room)
  self.maze_tunnel = gm.method(self, maze_tunnel)
  self.maze_distance = gm.method(self, maze_distance)
  self.maze_dig_out = gm.method(self, maze_dig_out)
  self.maze_inside = gm.method(self, maze_inside)
  self.maze_cave_hollows = gm.method(self, maze_cave_hollows)
  self.maze_hollows = gm.method(self, maze_hollows)
  self.maze_branches = gm.method(self, maze_branches)
  self.maze_finish = gm.method(self, maze_finish)
  self.maze_data = gm.method(self, maze_data)
  self.world_maze_build = gm.method(self, world_maze_build)
  self.world_rock_clusters = gm.method(self, world_rock_clusters)
  self.world_cluster_build = gm.method(self, world_cluster_build)
  self.world_paint_clusters_late = gm.method(self, world_paint_clusters_late)
  self.world_paint_clusters = gm.method(self, world_paint_clusters)
  self.world_paint_maze_part = gm.method(self, world_paint_maze_part)
  self.world_draw_stone_deposits = gm.method(self, world_draw_stone_deposits)
end
