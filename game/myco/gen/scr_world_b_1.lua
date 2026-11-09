local gm = require("gm")

-- gml_Script_world_food_spot_ok @ 0x1403257c0
function world_food_spot_ok(self, other, x, y, x1, y1, x2, y2, ctx)
  local w = global.world
  if x < x1 or x >= x2 or y < y1 or y >= y2 or y < w.surface_y + w.tile then
    return false
  end
  if gm.point_distance(ctx.x, ctx.y, x, y) < ctx.near then
    return false
  end
  if gm.bool(world_cave_hits(self, other, x, y, x, y, gm.undefined)) then
    return false
  end
  if world_tile_at(self, other, x, y) ~= 0 then
    return false
  end
  local chunk = world_get_chunk(self, other, gm.floor(x / w.chunk_w))
  return (not gm.bool(world_lake_covers(self, other, chunk, x, y))) and (not gm.bool(world_stone_covers(self, other, chunk, x, y)))
end

local function arr_len(a)
  if gm.is_array(a) then
    return gm.array_length(a)
  end
  return 0
end

-- gml_Script_world_start_guarantee @ 0x140326600
function world_start_guarantee(self, other)
  local w = global.world
  local ring = world_start_ring(self, other)
  local c0 = gm.floor((ring.x - ring.far - 400) / w.chunk_w)
  local c1 = gm.floor((ring.x + ring.far + 400) / w.chunk_w)
  local has_lake = false
  local has_food = false
  local c = c0
  while c <= c1 do
    local chunk = world_get_chunk(self, other, c)
    local i = 0
    while i < arr_len(chunk.lakes) do
      local L = chunk.lakes[i]
      local d = world_rect_distance(self, other, ring.x, ring.y, L.body_x, L.body_y, L.body_x + L.body_w, L.body_y + L.body_h)
      has_lake = has_lake or (d >= ring.near and d <= ring.far
        and gm.bool(world_start_on_screen(self, other, ring, L.body_x, L.body_y, L.body_x + L.body_w, L.body_y + L.body_h)))
      i = i + 1
    end
    i = 0
    while i < arr_len(chunk.foods) do
      local f = chunk.foods[i]
      local d = gm.point_distance(ring.x, ring.y, f.x, f.y)
      has_food = has_food or ((not gm.bool(f.collected)) and d >= ring.near and d <= ring.far
        and gm.bool(world_start_on_screen(self, other, ring, f.x, f.y, f.x, f.y)))
      i = i + 1
    end
    c = c + 1
  end
  if not has_lake then
    world_start_add_lake(self, other, ring)
  end
  if not has_food then
    world_start_add_food(self, other, ring)
  end
end

-- gml_Script____struct___343__world_start_add_lake__scr_world @ 0x140328540
function ___struct___343__world_start_add_lake__scr_world(self, other, cx, seed, kind, x, y, body_x, body_y, body_w, body_h, amount, full, flares)
  self.cx = cx
  self.seed = seed
  self.kind = kind
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

-- gml_Script_world_start_add_lake @ 0x140328be0
function world_start_add_lake(self, other, ring)
  local w = global.world
  local kind = 1
  local lay = lake_layout(self, other)[kind]
  local gap = wld(self, other, "water_lake_gap")
  local clr = wld(self, other, "stone_water_clearance_px")
  local top = w.surface_y + w.tile
  local hard = w.surface_y + w.hard_row * w.tile
  local a = 0
  while a < 80 do
    local ang = world_start_angle(self, other, a, 91)
    -- the second world_hash01 argument is DAT_141379b30 (never written -> 0)
    local dist = gm.lerp(ring.near, ring.far, 0.2 + world_hash01(self, other, a, 0, 92) * 0.6)
      + gm.max(lay.body_w, lay.body_h) * 0.3
    local x1 = gm.floor(ring.x + gm.lengthdir_x(dist, ang) - lay.body_w * 0.5)
    local y1 = gm.floor(ring.y + gm.lengthdir_y(dist, ang) - lay.body_h * 0.5)
    local x2 = x1 + lay.body_w
    local y2 = y1 + lay.body_h
    local d = world_rect_distance(self, other, ring.x, ring.y, x1, y1, x2, y2)
    repeat
      if d < ring.near + 16 then break end
      if d > ring.far - 16 or y1 < top or y2 > hard then break end
      -- YYC inlined world_start_on_screen(ring, x1, y1, x2, y2) here
      if not (x1 >= ring.x1 and x2 <= ring.x2 and y1 >= ring.y1 and y2 <= ring.y2) then break end
      local cx = gm.floor(x1 / w.chunk_w)
      if x1 < cx * w.chunk_w + gap or x2 > (cx + 1) * w.chunk_w - gap then break end
      local chunk = world_get_chunk(self, other, cx)
      local ok = not gm.bool(world_cave_hits(self, other, x1, y1, x2, y2, gm.undefined))
      local i = 0
      while i < arr_len(chunk.lakes) and ok do
        local L = chunk.lakes[i]
        ok = not (x1 < L.body_x + L.body_w + gap and x2 + gap > L.body_x
          and y1 < L.body_y + L.body_h + gap and y2 + gap > L.body_y)
        i = i + 1
      end
      local yy = y1 - clr
      while yy <= y2 + clr and ok do
        local xx = x1 - clr
        while xx <= x2 + clr and ok do
          ok = not gm.bool(world_stone_covers(self, other, chunk, xx, yy))
          xx = xx + 16
        end
        yy = yy + 16
      end
      if not ok then break end
      local amount = lake_amount(self, other, kind)
      gm.array_push(chunk.lakes, gm.new(___struct___343__world_start_add_lake__scr_world,
        cx, 900 + a, kind, x1 - lay.body_x, y1 - lay.body_y, x1, y1, lay.body_w, lay.body_h,
        amount, amount, gm.undefined))
      local foods = gm.array()
      i = 0
      while i < arr_len(chunk.foods) do
        if not gm.bool(world_lake_covers(self, other, chunk, chunk.foods[i].x, chunk.foods[i].y)) then
          gm.array_push(foods, chunk.foods[i])
        end
        i = i + 1
      end
      chunk.foods = foods
      local stones = gm.array()
      i = 0
      while i < arr_len(chunk.stones) do
        local s = chunk.stones[i]
        if not gm.variable_struct_exists(s, "x") or not gm.bool(world_lake_covers(self, other, chunk, s.x, s.y)) then
          gm.array_push(stones, s)
        end
        i = i + 1
      end
      chunk.stones = stones
      do return true end
    until true
    a = a + 1
  end
  return false
end

-- gml_Script____struct___344__world_start_add_food__scr_world @ 0x14032d3d0
function ___struct___344__world_start_add_food__scr_world(self, other, x, y, kind, cell)
  self.x = x
  self.y = y
  self.kind = kind
  self.cell = cell
  self.collected = false
end

-- gml_Script_world_start_add_food @ 0x14032d700
function world_start_add_food(self, other, ring)
  local w = global.world
  local kinds = world_food_kinds(self, other)
  local a = 0
  while a < 80 do
    local ang = world_start_angle(self, other, a, 95)
    -- the second world_hash01 argument is DAT_141379b30 (never written -> 0)
    local dist = gm.lerp(ring.near, ring.far, 0.25 + world_hash01(self, other, a, 0, 96) * 0.5)
    local bx = ring.x + gm.lengthdir_x(dist, ang)
    local by = ring.y + gm.lengthdir_y(dist, ang)
    local placed = 0
    local j = 0
    while j < 4 do
      local x = bx + ((j == 0) and 0 or ((world_hash01(self, other, a, j, 97) - 0.5) * 80))
      local y = by + ((j == 0) and 0 or ((world_hash01(self, other, a, j, 98) - 0.5) * 80))
      local d = gm.point_distance(ring.x, ring.y, x, y)
      local cx = gm.floor(x / w.chunk_w)
      local chunk = world_get_chunk(self, other, cx)
      local row = gm.floor((y - w.surface_y) / w.tile)
      repeat
        if d < ring.near or d > ring.far or row < 1 or y >= w.surface_y + w.hard_row * w.tile
          or not gm.bool(world_start_on_screen(self, other, ring, x, y, x, y)) then
          break
        end
        if world_chunk_get(self, other, chunk, gm.floor(x / w.tile), row) ~= 0 then break end
        if gm.bool(world_lake_covers(self, other, chunk, x, y)) then break end
        if gm.bool(world_stone_covers(self, other, chunk, x, y)) then break end
        local tier = gm.min(kinds - 1, 1)
        local cells = world_food_cells(self, other, tier)
        gm.array_push(chunk.foods, gm.new(___struct___344__world_start_add_food__scr_world,
          x, y, tier, gm.min(cells - 1, gm.floor(world_hash01(self, other, a, j, 99) * cells))))
        placed = placed + 1
      until true
      j = j + 1
    end
    if placed > 0 then
      return true
    end
    a = a + 1
  end
  return false
end

local function to_i64(x)
  if x >= 0 then
    return math.floor(x)
  end
  return math.ceil(x)
end

-- gml_Script_world_hash01 @ 0x1403306e0
function world_hash01(self, other, a, b, c)
  local seed = global.world.seed
  local ia = to_i64(a)
  local ib = to_i64(b)
  local h = bit.band(bit.arshift(ia, 16), 0xffff) * 0x5215 + bit.band(ia, 0xffff) * 0x9e37
    + bit.band(ib, 0xffff) * 0x9069 + c * 12983 + seed
  h = gm.mod(h, 65536)
  h = to_i64(h)
  h = bit.bxor(h, bit.arshift(h, 8))
  h = gm.mod(h * 0x9e37, 0x10000)
  h = bit.bxor(h, bit.arshift(h, 7))
  h = gm.mod(h * 0x9069, 0x10000)
  h = bit.bxor(h, bit.arshift(h, 9))
  return h / 65536
end

-- gml_Script_world_chunk_key @ 0x140330df0
function world_chunk_key(self, other, cx)
  return "c" .. gm.string(cx)
end

-- gml_Script_world_get_chunk @ 0x140331060
local world_get_chunk_static = { _last_world = gm.undefined, _last_cx = gm.undefined, _last_chunk = gm.undefined }
function world_get_chunk(self, other, cx)
  local st = world_get_chunk_static
  local w = global.world
  if st._last_cx == cx and st._last_world == w then
    return st._last_chunk
  end
  local key = world_chunk_key(self, other, cx)
  local ch = gm.variable_struct_get(w.chunks, key)
  if gm.is_undefined(ch) then
    ch = world_generate_chunk(self, other, cx)
    gm.variable_struct_set(w.chunks, key, ch)
    gm.array_push(w.chunk_index, cx)
  end
  st._last_world = w
  st._last_cx = cx
  st._last_chunk = ch
  return ch
end

-- gml_Script_world_chunk_set @ 0x140331ab0
function world_chunk_set(self, other, chunk, tx, ty, tile, amount)
  local w = global.world
  local lx = tx - chunk.cx * w.cols
  if lx < 0 or lx >= w.cols or ty < 0 or ty >= chunk.rows then
    return
  end
  local i = ty * w.cols + lx
  chunk.tiles[i] = tile
  chunk.amount[i] = amount
end

-- gml_Script_world_chunk_get @ 0x140332150
function world_chunk_get(self, other, chunk, tx, ty)
  local w = global.world
  local lx = tx - chunk.cx * w.cols
  if lx < 0 or lx >= w.cols or ty < 0 or ty >= chunk.rows then
    return -1
  end
  return chunk.tiles[ty * w.cols + lx]
end

-- gml_Script_world_paint_stone_clusters @ 0x140332700
function world_paint_stone_clusters(self, other, chunk, cx)
  if not (wld(self, other, "stones_enabled") > 0) then
    return
  end
  local w = global.world
  local count = gm.max(1, gm.round(wld(self, other, "stone_cluster_frequency") * w.cols))
  local rmin = wld(self, other, "stone_cluster_min_radius")
  local rmax = wld(self, other, "stone_cluster_max_radius")
  local k = 0
  while k < count do
    local tx = cx * w.cols + gm.floor(world_hash01(self, other, cx, k, 11) * w.cols)
    local ty0 = 2 + gm.floor(world_hash01(self, other, cx, k, 12) * gm.max(1, w.hard_row - 4))
    local rx = rmin + gm.floor(world_hash01(self, other, cx, k, 13) * (rmax - rmin + 1))
    local ry = gm.max(1, rx - 1)
    local dy = -ry
    while dy <= ry do
      local ny = dy / ry
      local dx = -rx
      while dx <= rx do
        local nx = dx / rx
        if nx * nx + ny * ny > 1 then
          -- continue
        else
          local ty = ty0 + dy
          if ty < 1 or ty >= w.hard_row then
            -- continue
          else
            -- tile amount argument is DAT_141379b30 (never written -> 0)
            world_chunk_set(self, other, chunk, tx + dx, ty, 2, 0)
          end
        end
        dx = dx + 1
      end
      dy = dy + 1
    end
    k = k + 1
  end
end

-- gml_Script_world_paint_stone_pillar @ 0x1403347b0
function world_paint_stone_pillar(self, other, chunk, cx)
  if wld(self, other, "stones_enabled") <= 0 then
    return
  end
  local w = global.world
  -- the second world_hash01 argument is DAT_141379b30 (never written -> 0)
  if world_hash01(self, other, cx, 0, 21) >= wld(self, other, "stone_pillar_chance") then
    return
  end
  local tx = cx * w.cols + gm.floor(world_hash01(self, other, cx, 1, 22) * w.cols)
  local width = 1 + gm.floor(world_hash01(self, other, cx, 2, 23) * 2)
  local ty = 1
  while ty < w.hard_row do
    local i = 0
    while i < width do
      -- tile amount argument is DAT_141379b30 (never written -> 0)
      world_chunk_set(self, other, chunk, tx + i, ty, 2, 0)
      i = i + 1
    end
    ty = ty + 1
  end
end

-- gml_Script____struct___345__lake_layout__scr_world @ 0x140335560
function ___struct___345__lake_layout__scr_world(self, other)
  self.left = 0
  self.width = 351
  self.body_x = 42
  self.body_y = 27
  self.body_w = 247
  self.body_h = 143
end

-- gml_Script____struct___346__lake_layout__scr_world @ 0x1403357a0
function ___struct___346__lake_layout__scr_world(self, other)
  self.left = 351
  self.width = 274
  self.body_x = 41
  self.body_y = 55
  self.body_w = 154
  self.body_h = 85
end

-- gml_Script____struct___347__lake_layout__scr_world @ 0x1403359e0
function ___struct___347__lake_layout__scr_world(self, other)
  self.left = 625
  self.width = 176
  self.body_x = 38
  self.body_y = 65
  self.body_w = 93
  self.body_h = 51
end

-- gml_Script_lake_layout @ 0x140335c20
local lake_layout_static = { _layout = gm.undefined, _init = false }
function lake_layout(self, other)
  local st = lake_layout_static
  if not st._init then
    st._init = true
    st._layout = gm.array(
      gm.new(___struct___345__lake_layout__scr_world),
      gm.new(___struct___346__lake_layout__scr_world),
      gm.new(___struct___347__lake_layout__scr_world))
  end
  return st._layout
end

-- gml_Script____struct___348__lakes_init__scr_world @ 0x1403362a0
function ___struct___348__lakes_init__scr_world(self, other, left, width, height, top, bottom, spans)
  self.left = left
  self.width = width
  self.height = height
  self.top = top
  self.bottom = bottom
  self.spans = spans
end

-- gml_Script_lakes_init @ 0x140336620
function lakes_init(self, other)
  if gm.bool(lakes_ready(self, other)) then
    return
  end
  local w = gm.sprite_get_width(water)
  local h = gm.sprite_get_height(water)
  local surf = gm.surface_create(w, h)
  gm.surface_set_target(surf)
  -- both draw_clear_alpha arguments are DAT_141379b30 (never written -> 0)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_inv_src_alpha)
  gm.draw_sprite(water, 0, gm.sprite_get_xoffset(water), gm.sprite_get_yoffset(water))
  -- gpu_set_blendmode argument is DAT_141379b30 (never written -> 0 = bm_normal)
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  -- buffer_create type argument is DAT_141379b30 (never written -> 0 = buffer_fixed)
  local buf = gm.buffer_create(w * h * 4, 0, 1)
  gm.buffer_get_surface(buf, surf, 0)
  gm.surface_free(surf)
  local layout = lake_layout(self, other)
  local kinds = gm.array()
  local i = 0
  while i < 3 do
    local left = layout[i].left
    local lw = gm.max(0, gm.min(layout[i].width, w - left))
    local rows = gm.array_create(h)
    local top = h
    local bottom = -1
    local y = 0
    while y < h do
      local spans = gm.array()
      local start = -1
      local x = 0
      while x <= lw do
        local solid = x < lw and gm.buffer_peek(buf, (y * w + left + x) * 4 + 3, gm.buffer_u8) >= 128
        if solid and start < 0 then
          start = x
        elseif (not solid) and start >= 0 then
          gm.array_push(spans, start, x - 1)
          start = -1
        end
        x = x + 1
      end
      rows[y] = spans
      if gm.is_array(spans) and gm.array_length(spans) > 0 then
        top = gm.min(top, y)
        bottom = gm.max(bottom, y)
      end
      y = y + 1
    end
    if bottom < top then
      local L = layout[i]
      top = L.body_y
      bottom = gm.min(h - 1, L.body_y + L.body_h - 1)
      y = top
      while y <= bottom do
        rows[y] = gm.array(L.body_x, L.body_x + L.body_w - 1)
        y = y + 1
      end
    end
    gm.array_push(kinds, gm.new(___struct___348__lakes_init__scr_world, left, lw, h, top, bottom, rows))
    i = i + 1
  end
  gm.buffer_delete(buf)
  global.lake_kinds = kinds
end

-- gml_Script_lakes_ready @ 0x1403393c0
function lakes_ready(self, other)
  return gm.variable_global_exists("lake_kinds") and gm.is_array(global.lake_kinds)
end

-- gml_Script_lake_solid_at @ 0x140339610
function lake_solid_at(self, other, kind, x, y)
  if y < 0 or y >= kind.height then
    return false
  end
  local spans = kind.spans[to_i64(y)]
  local i = 0
  while i < arr_len(spans) do
    if x >= spans[i] and x <= spans[i + 1] then
      return true
    end
    i = i + 2
  end
  return false
end

-- gml_Script_lake_pick_kind @ 0x140339af0
function lake_pick_kind(self, other, t, r)
  local big = gm.lerp(wld(self, other, "water_lake_big_chance_top"), wld(self, other, "water_lake_big_chance_bottom"), t)
  local small = gm.lerp(wld(self, other, "water_lake_small_chance_top"), wld(self, other, "water_lake_small_chance_bottom"), t)
  if r < big then
    return 0
  end
  if r < big + small then
    return 2
  end
  return 1
end

-- gml_Script_lake_amount @ 0x14033a230
function lake_amount(self, other, kind)
  if kind == 0 then
    return wld(self, other, "water_lake_big_amount")
  elseif kind == 1 then
    return wld(self, other, "water_lake_medium_amount")
  end
  return wld(self, other, "water_lake_small_amount")
end

-- gml_Script_lake_flare_count @ 0x14033a5b0
function lake_flare_count(self, other, kind)
  if kind == 0 then
    return vis(self, other, "water_flares_big")
  elseif kind == 1 then
    return vis(self, other, "water_flares_medium")
  end
  return vis(self, other, "water_flares_small")
end

-- gml_Script____struct___349__world_paint_lakes__scr_world @ 0x14033a930
function ___struct___349__world_paint_lakes__scr_world(self, other, cx, seed, kind, x, y, body_x, body_y, body_w, body_h, amount, full, flares)
  self.cx = cx
  self.seed = seed
  self.kind = kind
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

-- gml_Script_world_paint_lakes @ 0x14033afd0
function world_paint_lakes(self, other, chunk, layer)
  local w = global.world
  local key = world_layer_key(self, other, chunk.cx, layer)
  local lay = lake_layout(self, other)
  local x0 = chunk.cx * w.chunk_w
  local top = world_layer_top(self, other, layer)
  local bottom = world_layer_bottom(self, other, layer)
  local gap = wld(self, other, "water_lake_gap")
  local clr = wld(self, other, "stone_water_clearance_px")
  -- first max() operand is DAT_141379b30 (never written -> 0)
  local tries = gm.max(0, wld(self, other, "water_lake_tries"))
  local chance = wld(self, other, "water_lake_chance")
  local t = 0
  while t < tries do
    if world_hash01(self, other, key, t, 81) < chance then
      local depth = world_hash01(self, other, key, t, 82)
      local kind = lake_pick_kind(self, other, depth, world_hash01(self, other, key, t, 83))
      local L = lay[kind]
      -- first max() operand is DAT_141379b30 (never written -> 0)
      local bx = gm.floor(x0 + gap + world_hash01(self, other, key, t, 84) * gm.max(0, w.chunk_w - L.body_w - gap * 2))
      local by = gm.floor(gm.lerp(top, gm.max(top, bottom - L.body_h), depth))
      local ok = true
      local i = 0
      while i < arr_len(chunk.lakes) do
        local o = chunk.lakes[i]
        if bx < o.body_x + o.body_w + gap and bx + L.body_w + gap > o.body_x
          and by < o.body_y + o.body_h + gap and by + L.body_h + gap > o.body_y then
          ok = false
          break
        end
        i = i + 1
      end
      if ok then
        if gm.bool(world_cave_hits(self, other, bx, by, bx + L.body_w, by + L.body_h, gm.undefined)) then
          ok = false
        end
      end
      local yy = by - clr
      while yy <= by + L.body_h + clr and ok do
        local xx = bx - clr
        while xx <= bx + L.body_w + clr and ok do
          ok = not gm.bool(world_stone_covers(self, other, chunk, xx, yy))
          xx = xx + 16
        end
        yy = yy + 16
      end
      if ok then
        local amount = lake_amount(self, other, kind)
        gm.array_push(chunk.lakes, gm.new(___struct___349__world_paint_lakes__scr_world,
          chunk.cx, t + layer * 8, kind, bx - L.body_x, by - L.body_y, bx, by, L.body_w, L.body_h,
          amount, amount, gm.undefined))
      end
    end
    t = t + 1
  end
end

-- gml_Script_world_lake_covers @ 0x14033e8d0
function world_lake_covers(self, other, chunk, x, y)
  local lakes = chunk.lakes
  local i = 0
  while i < arr_len(lakes) do
    local L = lakes[i]
    if x >= L.body_x and x < L.body_x + L.body_w and y >= L.body_y and y < L.body_y + L.body_h then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script_lake_flare_life_ms @ 0x14033ef20
function lake_flare_life_ms(self, other)
  -- the 0 operands of the inner max() calls are DAT_141379b30 (never written -> 0)
  return gm.max(1, (gm.max(0, vis(self, other, "water_flare_fade_in_seconds"))
    + gm.max(0, vis(self, other, "water_flare_hold_seconds"))
    + gm.max(0, vis(self, other, "water_flare_fade_out_seconds"))) * 1000)
end

-- gml_Script_lake_flare_alpha @ 0x14033f5f0
function lake_flare_alpha(self, other, age)
  -- the 0 operands of max() are DAT_141379b30 (never written -> 0)
  local fin = gm.max(0, vis(self, other, "water_flare_fade_in_seconds")) * 1000
  local hold = gm.max(0, vis(self, other, "water_flare_hold_seconds")) * 1000
  local fout = gm.max(0, vis(self, other, "water_flare_fade_out_seconds")) * 1000
  if age < 0 then
    return 0
  end
  if age < fin then
    return age / fin
  end
  if age < fin + hold then
    return 1
  end
  if age < fin + hold + fout then
    return 1 - (age - fin - hold) / fout
  end
  return 0
end

-- gml_Script____struct___350__lake_flare_spot__scr_world @ 0x140340280
function ___struct___350__lake_flare_spot__scr_world(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_lake_flare_spot @ 0x140340480
function lake_flare_spot(self, other, lake, kind, skip, min_y)
  local fw = gm.div(gm.sprite_get_width(water_flare), 3)
  local fh = gm.sprite_get_height(water_flare)
  local ox0 = gm.floor(fw * 0.1)
  local ox1 = gm.ceil(fw * 0.9) - 1
  local oy0 = gm.floor(fh * 0.2)
  local oy1 = gm.ceil(fh * 0.8) - 1
  local xmin = -ox0
  local xmax = kind.width - 1 - ox1
  local ymin = gm.max(min_y, kind.top) - oy0
  local ymax = kind.bottom - oy1
  if xmax < xmin or ymax < ymin then
    return gm.undefined
  end
  local seed = lake.cx * 64 + lake.seed
  local k = 0
  while k < 12 do
    lake.flare_seq = lake.flare_seq + 1
    local x = xmin + gm.floor(world_hash01(self, other, seed, lake.flare_seq, 91) * (xmax - xmin + 1))
    local y = ymin + gm.floor(world_hash01(self, other, seed, lake.flare_seq, 92) * (ymax - ymin + 1))
    if gm.bool(lake_solid_at(self, other, kind, x + ox0, y + oy0))
      and gm.bool(lake_solid_at(self, other, kind, x + ox1, y + oy0))
      and gm.bool(lake_solid_at(self, other, kind, x + ox0, y + oy1))
      and gm.bool(lake_solid_at(self, other, kind, x + ox1, y + oy1)) then
      local hit = false
      local i = 0
      while i < arr_len(lake.flares) do
        local f = lake.flares[i]
        if i == skip then
          -- continue
        elseif not gm.bool(f.placed) then
          -- continue
        elseif gm.abs(f.x - x) < fw * 0.75 and gm.abs(f.y - y) < fh * 0.5 then
          hit = true
          break
        end
        i = i + 1
      end
      if not hit then
        return gm.new(___struct___350__lake_flare_spot__scr_world, x, y)
      end
    end
    k = k + 1
  end
  return gm.undefined
end

-- gml_Script____struct___351__lake_init_flares__scr_world @ 0x140343480
function ___struct___351__lake_init_flares__scr_world(self, other, born)
  self.x = 0
  self.y = 0
  self.born = born
  self.placed = false
end

-- gml_Script_lake_init_flares @ 0x1403436f0
function lake_init_flares(self, other, lake, kind, min_y)
  local count = lake_flare_count(self, other, lake.kind)
  local life = lake_flare_life_ms(self, other)
  lake.flares = gm.array()
  lake.flare_seq = 0
  local i = 0
  while i < count do
    gm.array_push(lake.flares, gm.new(___struct___351__lake_init_flares__scr_world, gm.current_time))
    i = i + 1
  end
  i = 0
  while i < count do
    local f = lake.flares[i]
    local spot = lake_flare_spot(self, other, lake, kind, i, min_y)
    if not gm.is_undefined(spot) then
      f.x = spot.x
      f.y = spot.y
      f.placed = true
      f.born = gm.current_time + gm.floor(world_hash01(self, other, lake.cx * 64 + lake.seed, i, 93) * life * 0.5)
    end
    i = i + 1
  end
end

-- gml_Script_lake_cut_row @ 0x140344b10
function lake_cut_row(self, other, lake, kind)
  local t = 1 - gm.clamp(lake.amount / gm.max(1, lake.full), 0, 1)
  return gm.floor(gm.lerp(kind.top, kind.bottom + 1, t))
end

-- gml_Script____struct___352__world_paint_food__scr_world @ 0x140345120
function ___struct___352__world_paint_food__scr_world(self, other, x, y, kind, cell)
  self.x = x
  self.y = y
  self.kind = kind
  self.cell = cell
  self.collected = false
end

-- gml_Script_world_paint_food @ 0x140345450
function world_paint_food(self, other, chunk, layer)
  local w = global.world
  local key = world_layer_key(self, other, chunk.cx, layer)
  local x0 = chunk.cx * w.chunk_w
  local dist = gm.abs(x0 + w.chunk_w * 0.5 - w.start_x)
  local rarity = 1 + dist / gm.max(1, wld(self, other, "food_distance_rarity_px"))
  -- the second world_hash01 argument is DAT_141379b30 (never written -> 0)
  local spacing = gm.lerp(wld(self, other, "food_spawn_near_min_px"), wld(self, other, "food_spawn_near_max_px"),
    world_hash01(self, other, key, 0, 31)) * rarity
  local count = gm.round(w.chunk_w / gm.max(1, spacing))
  local top = world_layer_top(self, other, layer)
  local bottom = world_layer_bottom(self, other, layer)
  local base = count
  local dense_y = w.surface_y + world_dense_row(self, other) * w.tile
  local deep_y = world_deep_y(self, other)
  local dense_count = count
  if bottom > dense_y then
    dense_count = gm.round(base * gm.max(1, wld(self, other, "dense_food_multiplier")))
    count = dense_count
  end
  if bottom > deep_y then
    count = gm.max(count, gm.round(base * gm.max(1, wld(self, other, "deep_food_multiplier"))))
  end
  local kinds = world_food_kinds(self, other)
  local i = 0
  while i < count do
    local fx = x0 + world_hash01(self, other, key, i, 32) * w.chunk_w
    local fy = gm.lerp(top, bottom, world_hash01(self, other, key, i, 33))
    if i >= base and fy < dense_y then
      -- continue
    elseif i >= dense_count and fy < deep_y then
      -- continue
    else
      local deep = fy >= deep_y
      local chance = (not deep) and wld(self, other, "food_clump_chance") or wld(self, other, "deep_food_clump_chance")
      local max_extra = (not deep) and wld(self, other, "food_clump_max_extra") or wld(self, other, "deep_food_clump_max_extra")
      local spread = w.tile * ((not deep) and 3 or wld(self, other, "deep_food_clump_spread_tiles"))
      local extra = 0
      if world_hash01(self, other, key, i, 34) < chance then
        extra = gm.min(15, 1 + gm.floor(world_hash01(self, other, key, i, 35) * max_extra))
      end
      local j = 0
      while j <= extra do
        local px = fx
        local py = fy
        if j > 0 then
          px = px + (world_hash01(self, other, key, i * 16 + j, 36) - 0.5) * spread
          py = py + (world_hash01(self, other, key, i * 16 + j, 37) - 0.5) * spread
        end
        px = gm.clamp(px, x0 + 8, x0 + w.chunk_w - 8)
        py = gm.clamp(py, top, bottom)
        local tx = gm.floor(px / w.tile)
        local ty = gm.floor((py - w.surface_y) / w.tile)
        if world_chunk_get(self, other, chunk, tx, ty) ~= 0 then
          -- continue
        elseif gm.bool(world_cave_hits(self, other, px, py, px, py, gm.undefined)) then
          -- continue
        elseif gm.bool(world_lake_covers(self, other, chunk, px, py)) then
          -- continue
        elseif gm.bool(world_stone_covers(self, other, chunk, px, py)) then
          -- continue
        else
          local tier = gm.min(kinds - 1, gm.floor(world_hash01(self, other, key, i * 16 + j, 38) * kinds))
          local cells = world_food_cells(self, other, tier)
          gm.array_push(chunk.foods, gm.new(___struct___352__world_paint_food__scr_world, px, py, tier,
            gm.min(cells - 1, gm.floor(world_hash01(self, other, key, i * 16 + j, 39) * cells))))
        end
        j = j + 1
      end
    end
    i = i + 1
  end
end

-- gml_Script_world_layer_top @ 0x14034a5d0
function world_layer_top(self, other, layer)
  local w = global.world
  return w.surface_y + ((layer == 0) and 1 or (layer * w.layer_rows)) * w.tile
end

-- gml_Script_world_layer_bottom @ 0x14034a970
function world_layer_bottom(self, other, layer)
  local w = global.world
  return w.surface_y + (layer + 1) * w.layer_rows * w.tile
end

-- gml_Script_world_layer_key @ 0x14034ad10
function world_layer_key(self, other, cx, layer)
  return (layer == 0) and cx or (cx + layer * 0x250000)
end

-- gml_Script____struct___353__world_generate_chunk__scr_world @ 0x14034af10
function ___struct___353__world_generate_chunk__scr_world(self, other, cx, tiles, amount, connected, foods, bones, stones, lakes, stone_cells)
  self.cx = cx
  self.rows = 0
  self.tiles = tiles
  self.amount = amount
  self.connected = connected
  self.foods = foods
  self.bones = bones
  self.stones = stones
  self.lakes = lakes
  self.stone_cells = stone_cells
  self.stone_map = gm.struct({})
end

-- gml_Script_world_generate_chunk @ 0x14034b4a0
function world_generate_chunk(self, other, cx)
  local ch = gm.new(___struct___353__world_generate_chunk__scr_world, cx,
    gm.array(), gm.array(), gm.array(), gm.array(),
    gm.array(), gm.array(), gm.array(), gm.array())
  -- reach argument is DAT_141379b30 (never written -> 0)
  world_chunk_reach(self, other, ch, 0)
  return ch
end

-- gml_Script_world_chunk_reach @ 0x14034bc10
function world_chunk_reach(self, other, chunk, row)
  local w = global.world
  if row < chunk.rows then
    return
  end
  if not (row < w.rows) then
    return
  end
  while chunk.rows <= row do
    local layer = gm.div(chunk.rows, w.layer_rows)
    local i0 = chunk.rows * w.cols
    local i1 = (chunk.rows + w.layer_rows) * w.cols
    local i = i0
    while i < i1 do
      chunk.tiles[i] = 0
      chunk.amount[i] = 0
      chunk.connected[i] = false
      i = i + 1
    end
    chunk.rows = chunk.rows + w.layer_rows
    world_paint_layer(self, other, chunk, layer)
  end
end

-- gml_Script_world_survival_full_at @ 0x14034c7b0
function world_survival_full_at(self, other, x, y)
  local w = global.world
  local xs = wld(self, other, "survival_full_zone_x_screens")
  local half = vis(self, other, "screen_width_px") * xs
  if not (gm.abs(x - w.start_x) <= half) then
    return false
  end
  local ds = wld(self, other, "survival_full_zone_depth_screens")
  local depth = w.surface_y + vis(self, other, "screen_height_px") * ds
  return y <= depth
end
