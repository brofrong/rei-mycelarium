local gm = require("gm")

-- array_length as compiled by YYC for `for (i = 0; i < array_length(a); ...)`: non-array -> 0
local function arr_len(a)
  if gm.is_array(a) then
    return gm.array_length(a)
  end
  return 0
end

-- gml_Script_world_survival_thin @ 0x14034ce90
function world_survival_thin(self, other, chunk)
  if game_mode(self, other) ~= "survival" then
    return gm.undefined
  end
  local share = gm.clamp(gm.real(wld(self, other, "survival_resource_share")), 0, 1.0)
  local foods = chunk.foods
  local kept = gm.array()
  local i = 0
  while i < arr_len(foods) do
    local f = foods[i]
    local skip = false
    -- f[$ "sv"] ?? false
    if not gm.bool(f.sv) then
      f.sv = true
      -- f[$ "maze"] ?? false
      if not gm.bool(f.maze) then
        if not gm.bool(world_survival_full_at(self, other, f.x, f.y)) then
          if world_hash01(self, other, gm.floor(f.x), gm.floor(f.y), 611.0) >= share then
            skip = true
          end
        end
      end
    end
    if not skip then
      gm.array_push(kept, f)
    end
    i = i + 1
  end
  chunk.foods = kept
  i = 0
  while i < arr_len(chunk.lakes) do
    local L = chunk.lakes[i]
    -- L[$ "sv"] ?? false
    if not gm.bool(L.sv) then
      L.sv = true
      if L.seed < 700.0 and not gm.bool(world_survival_full_at(self, other, L.body_x + L.body_w * 0.5, L.body_y + L.body_h * 0.5)) then
        L.full = gm.max(1.0, gm.round(L.full * share))
        L.amount = gm.min(L.amount, L.full)
      end
    end
    i = i + 1
  end
  return gm.undefined
end

-- gml_Script_world_paint_layer @ 0x14034ee80
function world_paint_layer(self, other, chunk, layer)
  local t0 = gm.get_timer()
  caves_extend(self, other, (chunk.cx + 0.5) * global.world.chunk_w)
  world_paint_layer_content(self, other, chunk, layer)
  world_paint_bones(self, other, chunk, layer)
  world_clear_caves(self, other, chunk)
  world_survival_thin(self, other, chunk)
  if gm.bool(gm.variable_global_exists("prof_gen_us")) then
    global.prof_gen_us = global.prof_gen_us + (gm.get_timer() - t0)
    global.prof_gen_layers = global.prof_gen_layers + 1
  end
  return gm.undefined
end

-- gml_Script_world_paint_layer_content @ 0x14034f6f0
function world_paint_layer_content(self, other, chunk, layer)
  local w = global.world
  local cx = chunk.cx
  if not (layer > 0) then
    local c = cx - 1
    while c <= cx + 1 do
      world_paint_stone_clusters(self, other, chunk, c)
      c = c + 1
    end
    c = cx - 1
    while c <= cx + 1 do
      world_paint_stone_pillar(self, other, chunk, c)
      c = c + 1
    end
    local obs = world_obstacles_at_chunk(self, other, chunk)
    local i = 0
    while i < arr_len(obs) do
      if obs[i].kind == "maze" then
        world_paint_maze_part(self, other, chunk, obs[i])
      end
      i = i + 1
    end
    -- DAT_141379b30 is never written -> 0
    world_paint_lakes(self, other, chunk, 0)
    i = 0
    while i < arr_len(obs) do
      if obs[i].kind == "wall" and obs[i].cx == cx then
        world_paint_surface_wall(self, other, chunk)
      end
      i = i + 1
    end
    world_paint_stone_deposits(self, other, chunk, 0)
    local sx = gm.floor(w.start_x / w.tile)
    local row = 0
    while row <= 4.0 do
      local dx = -3.0
      while dx <= 3.0 do
        if world_chunk_get(self, other, chunk, sx + dx, row) == 2.0 then
          world_chunk_set(self, other, chunk, sx + dx, row, 0, 0)
        end
        dx = dx + 1
      end
      row = row + 1
    end
    world_paint_food(self, other, chunk, 0)
    world_start_clear(self, other, chunk)
    world_paint_stones(self, other, chunk, 0)
    return gm.undefined
  end
  local oy = world_ocean_y(self, other)
  if world_layer_top(self, other, layer) >= oy then
    return gm.undefined
  end
  world_paint_clusters(self, other, chunk, layer)
  world_paint_lakes(self, other, chunk, layer)
  world_paint_stone_deposits(self, other, chunk, layer)
  world_paint_food(self, other, chunk, layer)
  world_paint_stones(self, other, chunk, layer)
  if world_layer_bottom(self, other, layer) > oy then
    world_clear_ocean(self, other, chunk, oy)
  end
  return gm.undefined
end

-- gml_Script____struct___355__world_clear_ocean__scr_world @ 0x140351b30
function ___struct___355__world_clear_ocean__scr_world(self, other, limit)
  self.limit = limit
end

-- gml_Script_anon__109412__world_clear_ocean__scr_world @ 0x140351c90
function anon__109412__world_clear_ocean__scr_world(self, other, L)
  return L.body_y + L.body_h < self.limit
end

-- gml_Script____struct___356__world_clear_ocean__scr_world @ 0x140351e50
function ___struct___356__world_clear_ocean__scr_world(self, other, limit)
  self.limit = limit
end

-- gml_Script_anon__109575__world_clear_ocean__scr_world @ 0x140351fb0
function anon__109575__world_clear_ocean__scr_world(self, other, v)
  return v.y < self.limit
end

-- gml_Script____struct___357__world_clear_ocean__scr_world @ 0x1403520e0
function ___struct___357__world_clear_ocean__scr_world(self, other, limit)
  self.limit = limit
end

-- gml_Script_anon__109735__world_clear_ocean__scr_world @ 0x140352240
function anon__109735__world_clear_ocean__scr_world(self, other, v)
  return v.y < self.limit
end

-- gml_Script____struct___358__world_clear_ocean__scr_world @ 0x140352370
function ___struct___358__world_clear_ocean__scr_world(self, other, limit)
  self.limit = limit
end

-- gml_Script_anon__109891__world_clear_ocean__scr_world @ 0x1403524d0
function anon__109891__world_clear_ocean__scr_world(self, other, v)
  return v.y + v.h < self.limit
end

-- gml_Script_world_clear_ocean @ 0x1403526a0
function world_clear_ocean(self, other, chunk, oy)
  local margin = wld(self, other, "ocean_clear_margin_px")
  local reach
  if gm.bool(gm.variable_global_exists("stone_reach")) then
    reach = global.stone_reach
  else
    reach = 48.0
  end
  chunk.lakes = gm.array_filter(chunk.lakes,
    gm.method(gm.new(___struct___355__world_clear_ocean__scr_world, oy - margin), anon__109412__world_clear_ocean__scr_world))
  chunk.foods = gm.array_filter(chunk.foods,
    gm.method(gm.new(___struct___356__world_clear_ocean__scr_world, oy - margin), anon__109575__world_clear_ocean__scr_world))
  chunk.stones = gm.array_filter(chunk.stones,
    gm.method(gm.new(___struct___357__world_clear_ocean__scr_world, oy - margin - reach), anon__109735__world_clear_ocean__scr_world))
  -- chunk[$ "bones"] ?? []
  local bones = chunk.bones
  if gm.is_undefined(bones) then
    bones = gm.array()
  end
  chunk.bones = gm.array_filter(bones,
    gm.method(gm.new(___struct___358__world_clear_ocean__scr_world, oy - margin), anon__109891__world_clear_ocean__scr_world))
  return gm.undefined
end

-- gml_Script____struct___359__world_locate__scr_world @ 0x140353fc0
function ___struct___359__world_locate__scr_world(self, other, cx, row, idx)
  self.cx = cx
  self.row = row
  self.idx = idx
end

-- gml_Script_world_locate @ 0x140354200
function world_locate(self, other, x, y)
  local w = global.world
  local row = gm.floor((y - w.surface_y) / w.tile)
  local col = gm.floor(x / w.tile)
  local cx = gm.floor(col / w.cols)
  return gm.new(___struct___359__world_locate__scr_world, cx, row, row * w.cols + (col - cx * w.cols))
end

-- gml_Script_world_tile_at @ 0x140354c70
function world_tile_at(self, other, x, y)
  local w = global.world
  if x ~= x or y ~= y then
    return 2.0
  end
  if y < w.surface_y then
    return -1.0
  end
  if not (y < w.bottom_y) then
    return 2.0
  end
  local row = gm.floor((y - w.surface_y) / w.tile)
  local col = gm.floor(x / w.tile)
  local cx = gm.floor(col / w.cols)
  if row < 0 then
    return -1.0
  end
  if row >= w.rows then
    return 2.0
  end
  if row >= world_ocean_row(self, other) then
    if gm.bool(world_bottom_is_rock(self, other)) then
      return 2.0
    end
    return 3.0
  end
  if gm.bool(world_cave_solid_any(self, other, x, y)) then
    return 2.0
  end
  local chunk = world_get_chunk(self, other, cx)
  world_chunk_reach(self, other, chunk, row)
  local t = chunk.tiles[row * w.cols + (col - cx * w.cols)]
  if t == 0 and gm.bool(stone_solid_at(self, other, chunk, x, y)) then
    return 2.0
  end
  if t == 0 and row >= world_dense_row(self, other) then
    return 1.0
  end
  return t
end

-- gml_Script_world_water_at @ 0x140355f70
function world_water_at(self, other, x, y)
  local w = global.world
  if x ~= x or y ~= y then
    return false
  end
  if y < w.surface_y or y >= w.bottom_y then
    return false
  end
  -- YYC inlined world_is_ocean_at(y) here
  if y >= world_ocean_y(self, other) then
    return not gm.bool(world_bottom_is_rock(self, other))
  end
  local loc = world_locate(self, other, x, y)
  if loc.row < 0 or loc.row >= w.rows then
    return false
  end
  local chunk = world_get_chunk(self, other, loc.cx)
  world_chunk_reach(self, other, chunk, loc.row)
  if chunk.tiles[loc.idx] == 3.0 then
    return true
  end
  local lakes = chunk.lakes
  local ready = lakes_ready(self, other)
  local i = 0
  while i < arr_len(lakes) do
    local L = lakes[i]
    if gm.bool(ready) then
      if gm.bool(lake_solid_at(self, other, global.lake_kinds[L.kind], gm.floor(x - L.x), gm.floor(y - L.y))) then
        return true
      end
    else
      if x >= L.body_x and x < L.body_x + L.body_w and y >= L.body_y and y < L.body_y + L.body_h then
        return true
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script_world_water_shown_at @ 0x140357620
function world_water_shown_at(self, other, x, y)
  local w = global.world
  if x ~= x or y ~= y then
    return false
  end
  if y < w.surface_y then
    return false
  end
  -- YYC inlined world_is_ocean_at(y) here
  if y >= world_ocean_y(self, other) then
    return not gm.bool(world_bottom_is_rock(self, other))
  end
  if not gm.bool(lakes_ready(self, other)) then
    return false
  end
  local lakes = world_get_chunk(self, other, gm.floor(x / w.chunk_w)).lakes
  local i = 0
  while i < arr_len(lakes) do
    local L = lakes[i]
    if not (L.amount <= 0) then
      local kd = global.lake_kinds[L.kind]
      local lx = gm.floor(x - L.x)
      local ly = gm.floor(y - L.y)
      if ly >= lake_cut_row(self, other, L, kd) then
        if gm.bool(lake_solid_at(self, other, kd, lx, ly)) then
          return true
        end
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script_world_is_passable @ 0x140358580
function world_is_passable(self, other, x, y)
  local t = world_tile_at(self, other, x, y)
  if t == -1.0 then
    return false
  end
  if t == 2.0 then
    return bal(self, other, "stone_passable") > 0
  end
  return true
end

-- gml_Script_world_is_deep_soil @ 0x140358960
function world_is_deep_soil(self, other, x, y)
  if world_tile_at(self, other, x, y) ~= 1.0 then
    return false
  end
  return world_locate(self, other, x, y).row >= world_deep_row(self, other)
end

-- gml_Script_world_soil_water_factor @ 0x140358d70
function world_soil_water_factor(self, other, x, y)
  if world_tile_at(self, other, x, y) ~= 1.0 then
    return 1.0
  end
  if world_locate(self, other, x, y).row >= world_deep_row(self, other) then
    return bal(self, other, "deepsoil_water_multiplier")
  end
  return bal(self, other, "hardsoil_water_multiplier")
end

-- gml_Script_world_travel_cost @ 0x1403593d0
function world_travel_cost(self, other, x, y, bonus)
  local w = global.world
  if world_tile_at(self, other, x, y) ~= 1.0 then
    return 1.0
  end
  local row = gm.floor((y - w.surface_y) / w.tile)
  local mult
  if row >= world_deep_row(self, other) then
    mult = bal(self, other, "deepsoil_speed_multiplier")
  else
    mult = bal(self, other, "hardsoil_speed_multiplier")
  end
  local speed = gm.clamp(gm.real(mult + bonus), 0.05, 1.0)
  return 1.0 / speed
end

-- gml_Script____struct___360__world_connect_lake_at__scr_world @ 0x140359d10
function ___struct___360__world_connect_lake_at__scr_world(self, other, lake)
  self.lake = lake
  self.cx = 0
  self.idx = 0
end

-- gml_Script_world_connect_lake_at @ 0x140359ed0
function world_connect_lake_at(self, other, x, y)
  if not gm.bool(lakes_ready(self, other)) then
    return false
  end
  local lakes = world_get_chunk(self, other, gm.floor(x / global.world.chunk_w)).lakes
  local i = 0
  while i < arr_len(lakes) do
    local L = lakes[i]
    if not gm.bool(L.connected) and not (L.amount <= 0) then
      local kd = global.lake_kinds[L.kind]
      local lx = gm.floor(x - L.x)
      local ly = gm.floor(y - L.y)
      if not (lx < -4.0) and not (lx > kd.width + 4) and not (ly < -4.0) and not (ly > kd.height + 4) then
        if gm.bool(lake_solid_at(self, other, kd, lx, ly))
          or gm.bool(lake_solid_at(self, other, kd, lx - 4, ly))
          or gm.bool(lake_solid_at(self, other, kd, lx + 4, ly))
          or gm.bool(lake_solid_at(self, other, kd, lx, ly - 4))
          or gm.bool(lake_solid_at(self, other, kd, lx, ly + 4)) then
          L.connected = true
          gm.array_push(global.water_sources, gm.new(___struct___360__world_connect_lake_at__scr_world, L))
          tasks_stat_add(self, other, "water", L.amount)
          tutorial_event(self, other, "water", gm.undefined, gm.undefined)
          return true
        end
      end
    end
    i = i + 1
  end
  return false
end

-- gml_Script____struct___361__world_connect_water_at__scr_world @ 0x14035b7d0
function ___struct___361__world_connect_water_at__scr_world(self, other, lake, cx, idx)
  self.lake = lake
  self.cx = cx
  self.idx = idx
end

-- gml_Script_world_connect_water_at @ 0x14035ba10
function world_connect_water_at(self, other, x, y)
  local w = global.world
  if y < w.surface_y then
    return false
  end
  if y >= w.bottom_y then
    return false
  end
  -- YYC inlined world_is_ocean_at(y) here
  if y >= world_ocean_y(self, other) then
    if gm.bool(world_bottom_is_rock(self, other)) then
      return false
    end
    if gm.bool(global.water_infinite) then
      return false
    end
    global.water_infinite = true
    return true
  end
  if gm.bool(world_connect_lake_at(self, other, x, y)) then
    return true
  end
  local loc = world_locate(self, other, x, y)
  local chunk = world_get_chunk(self, other, loc.cx)
  world_chunk_reach(self, other, chunk, loc.row)
  if chunk.tiles[loc.idx] == 3.0 and not gm.bool(chunk.connected[loc.idx]) then
    chunk.connected[loc.idx] = true
    tasks_stat_add(self, other, "water", chunk.amount[loc.idx])
    gm.array_push(global.water_sources, gm.new(___struct___361__world_connect_water_at__scr_world, gm.undefined, loc.cx, loc.idx))
    return true
  end
  return false
end

local world_food_sheets_static = { _sheets = nil }

-- gml_Script_world_food_sheets @ 0x14035ca80
function world_food_sheets(self, other)
  if world_food_sheets_static._sheets == nil then
    world_food_sheets_static._sheets = gm.array(food_small, food_medium, food_big)
  end
  return world_food_sheets_static._sheets
end

-- gml_Script_world_food_kinds @ 0x14035cdc0
function world_food_kinds(self, other)
  -- array_length(world_food_sheets())
  return arr_len(world_food_sheets(self, other))
end

-- gml_Script_world_food_sprite @ 0x14035ceb0
function world_food_sprite(self, other, kind)
  local sheets = world_food_sheets(self, other)
  local idx = gm.clamp(gm.real(kind), 0, arr_len(sheets) - 1)
  return sheets[gm.trunc(idx)]
end

-- gml_Script_world_food_cells @ 0x14035d0b0
function world_food_cells(self, other, kind)
  local spr = world_food_sprite(self, other, kind)
  return gm.max(1.0, gm.div(gm.sprite_get_height(spr), gm.sprite_get_width(spr)))
end

local world_food_value_static = { _values = nil }

-- gml_Script_world_food_value @ 0x14035d4a0
function world_food_value(self, other, kind)
  if world_food_value_static._values == nil then
    world_food_value_static._values = gm.array(1.0, 3.0, 5.0)
  end
  local values = world_food_value_static._values
  local idx = gm.clamp(gm.real(kind), 0, arr_len(values) - 1)
  return values[gm.trunc(idx)]
end

-- gml_Script____struct___364__world_food_columns__scr_world @ 0x14035d750
function ___struct___364__world_food_columns__scr_world(self, other, list, count, cols)
  self.list = list
  self.count = count
  self.cols = cols
end

local world_food_columns_static = { _for_world = nil, _indexes = nil }

-- gml_Script_world_food_columns @ 0x14035d980
function world_food_columns(self, other, chunk)
  local st = world_food_columns_static
  if st._indexes == nil then
    st._for_world = gm.undefined
    st._indexes = gm.struct({})
  end
  local w = global.world
  if st._for_world ~= w then
    st._for_world = w
    st._indexes = gm.struct({})
  end
  local key = world_chunk_key(self, other, chunk.cx)
  local foods = chunk.foods
  local e = gm.variable_struct_get(st._indexes, key)
  if gm.is_struct(e) and e.list == foods and e.count == arr_len(foods) then
    return e.cols
  end
  local x0 = chunk.cx * w.chunk_w
  local cols = gm.array_create(w.cols + 2)
  local i = 0
  while i < arr_len(cols) do
    cols[i] = gm.array()
    i = i + 1
  end
  i = 0
  while i < arr_len(foods) do
    local f = foods[i]
    local c = gm.clamp(gm.floor((f.x - x0) / w.tile) + 1, 0, arr_len(cols) - 1)
    gm.array_push(cols[gm.trunc(c)], f)
    i = i + 1
  end
  gm.variable_struct_set(st._indexes, key, gm.new(___struct___364__world_food_columns__scr_world, foods, arr_len(foods), cols))
  return cols
end

-- gml_Script_world_collect_food_at @ 0x14035f040
function world_collect_food_at(self, other, x, y, r)
  local w = global.world
  local total = 0
  local c0 = gm.floor((x - r) / w.chunk_w)
  local c1 = gm.floor((x + r) / w.chunk_w)
  local cx = c0
  while cx <= c1 do
    local chunk = world_get_chunk(self, other, cx)
    local cols = world_food_columns(self, other, chunk)
    local x0 = cx * w.chunk_w
    local a = gm.clamp(gm.floor((x - r - x0) / w.tile) + 1, 0, arr_len(cols) - 1)
    local b = gm.clamp(gm.floor((x + r - x0) / w.tile) + 1, 0, arr_len(cols) - 1)
    if a == 1.0 then
      a = 0
    end
    if b == arr_len(cols) - 2 then
      b = arr_len(cols) - 1
    end
    local c = a
    while c <= b do
      local col = cols[gm.trunc(c)]
      local j = 0
      while j < arr_len(col) do
        local f = col[j]
        if not gm.bool(f.collected) then
          if gm.point_distance(x, y, f.x, f.y) <= r then
            f.collected = true
            total = total + world_food_value(self, other, f.kind)
          end
        end
        j = j + 1
      end
      c = c + 1
    end
    cx = cx + 1
  end
  if total > 0 then
    tutorial_event(self, other, "food", gm.undefined, gm.undefined)
  end
  return total
end

-- gml_Script_water_source_amount @ 0x140360d20
function water_source_amount(self, other, src)
  if gm.is_struct(src.lake) then
    return src.lake.amount
  end
  return world_get_chunk(self, other, src.cx).amount[src.idx]
end

-- gml_Script_water_source_take @ 0x140361120
function water_source_take(self, other, src, want)
  local take = gm.min(water_source_amount(self, other, src), want)
  if gm.is_struct(src.lake) then
    local L = src.lake
    L.amount = L.amount - take
    return take
  end
  local chunk = world_get_chunk(self, other, src.cx)
  chunk.amount[src.idx] = chunk.amount[src.idx] - take
  if chunk.amount[src.idx] <= 0 then
    chunk.tiles[src.idx] = 0
  end
  return take
end

-- gml_Script_water_total @ 0x140361920
function water_total(self, other)
  if gm.bool(global.water_infinite) then
    return 1000000.0
  end
  local total = global.water_free
  local i = 0
  while i < arr_len(global.water_sources) do
    total = total + water_source_amount(self, other, global.water_sources[i])
    i = i + 1
  end
  return total
end

-- gml_Script_water_spend @ 0x140361d50
function water_spend(self, other, n)
  if gm.bool(global.water_infinite) then
    global.water_dip = global.water_dip + n
    global.water_dip_hold = vis(self, other, "water_dip_hold_frames")
    return gm.undefined
  end
  local need = n
  local take = gm.min(global.water_free, need)
  global.water_free = global.water_free - take
  need = need - take
  while need > 0 and gm.is_array(global.water_sources) and gm.array_length(global.water_sources) >= 1 do
    local src = global.water_sources[0]
    need = need - water_source_take(self, other, src, need)
    if water_source_amount(self, other, src) <= 0 then
      -- DAT_141379b30 is never written -> 0
      gm.array_delete(global.water_sources, 0, 1.0)
    end
  end
  return gm.undefined
end

-- gml_Script_water_shown @ 0x140362550
function water_shown(self, other)
  if gm.bool(global.water_infinite) then
    -- DAT_141379b30 is never written -> 0
    return gm.max(0, 1000000.0 - global.water_dip)
  end
  return water_total(self, other)
end

-- gml_Script_water_counter_step @ 0x1403627e0
function water_counter_step(self, other)
  if global.water_dip <= 0 then
    return gm.undefined
  end
  if global.water_dip_hold > 0 then
    global.water_dip_hold = global.water_dip_hold - 1.0
  else
    -- DAT_141379b30 is never written -> 0
    global.water_dip = gm.max(0, global.water_dip * vis(self, other, "water_dip_refill_keep") - 1)
  end
  return gm.undefined
end

-- gml_Script_world_draw_turf @ 0x140362cb0
function world_draw_turf(self, other, x0, y0, vw, vh)
  local w = global.world
  local spr = w.turf
  if spr < 0 then
    return gm.undefined
  end
  local sw = gm.sprite_get_width(spr)
  if sw <= 0 then
    return gm.undefined
  end
  local ty = w.surface_y + wld(self, other, "turf_offset_y")
  local top = ty - gm.sprite_get_yoffset(spr)
  if top > y0 + vh or top + gm.sprite_get_height(spr) < y0 then
    return gm.undefined
  end
  local start = gm.floor(x0 / sw) * sw
  local stop = x0 + vw
  local xx = start
  while xx < stop do
    -- YYC: direct draw_sprite call with float coordinates
    gm.draw_sprite(spr, 0, xx, ty)
    xx = xx + sw
  end
  return gm.undefined
end

-- gml_Script_world_soil_band @ 0x140363860
function world_soil_band(self, other, row)
  local first = gm.max(1.0, wld(self, other, "soil_first_rows"))
  if row < first then
    return 0
  end
  return 1.0 + gm.div(row - first, gm.max(1.0, wld(self, other, "soil_band_rows")))
end

-- gml_Script_world_seam_row_at_screens @ 0x140363d80
function world_seam_row_at_screens(self, other, screens)
  local first = gm.max(1.0, wld(self, other, "soil_first_rows"))
  local band = gm.max(1.0, wld(self, other, "soil_band_rows"))
  local row = gm.ceil(screens * vis(self, other, "screen_height_px") / wld(self, other, "reference_tile_size"))
  -- DAT_141379b30 is never written -> 0
  return first + gm.max(0, gm.ceil((row - first) / band)) * band
end

local world_dense_row_static = { _row = gm.undefined }

-- gml_Script_world_dense_row @ 0x140364790
function world_dense_row(self, other)
  local st = world_dense_row_static
  if gm.is_undefined(st._row) then
    st._row = world_seam_row_at_screens(self, other, wld(self, other, "dense_soil_depth_screens"))
  end
  return st._row
end

local world_deep_row_static = { _row = gm.undefined }

-- gml_Script_world_deep_row @ 0x140364c10
function world_deep_row(self, other)
  local st = world_deep_row_static
  if gm.is_undefined(st._row) then
    local screens = wld(self, other, "deep_soil_depth_screens")
    local dense = world_dense_row(self, other)
    st._row = gm.max(dense, world_seam_row_at_screens(self, other, screens))
  end
  return st._row
end

-- gml_Script_world_soil_tier @ 0x1403651f0
function world_soil_tier(self, other, row)
  if row >= world_deep_row(self, other) then
    return 2.0
  end
  if row >= world_dense_row(self, other) then
    return 1.0
  end
  return 0
end

-- gml_Script_world_deep_y @ 0x140365460
function world_deep_y(self, other)
  local w = global.world
  return w.surface_y + world_deep_row(self, other) * w.tile
end

local world_ocean_row_static = { _row = gm.undefined }

-- gml_Script_world_ocean_row @ 0x1403656c0
function world_ocean_row(self, other)
  local st = world_ocean_row_static
  if gm.is_undefined(st._row) then
    local after = wld(self, other, "ocean_after_deep_screens")
    local screens = wld(self, other, "deep_soil_depth_screens") + after
    local min_row = world_deep_row(self, other) + 1
    st._row = gm.max(min_row, world_seam_row_at_screens(self, other, screens))
  end
  return st._row
end

-- gml_Script_world_ocean_y @ 0x140365e20
function world_ocean_y(self, other)
  local w = global.world
  return w.surface_y + world_ocean_row(self, other) * w.tile
end

-- gml_Script_world_is_ocean_at @ 0x140366080
function world_is_ocean_at(self, other, y)
  return y >= world_ocean_y(self, other)
end

-- gml_Script_world_dense_y @ 0x140366170
function world_dense_y(self, other)
  local w = global.world
  return w.surface_y + world_dense_row(self, other) * w.tile
end

-- gml_Script_world_soil_sprite @ 0x1403663d0
function world_soil_sprite(self, other, row)
  local tier = world_soil_tier(self, other, row)
  local m = gm.mod(gm.real(world_soil_band(self, other, row)), 6.0)
  if tier == 2.0 then
    if m == 0 then return deepsoil1
    elseif m == 1 then return deepsoil2
    elseif m == 2 then return deepsoil3
    elseif m == 3 then return deepsoil4
    elseif m == 4 then return deepsoil3
    end
    return deepsoil2
  end
  if tier == 1.0 then
    if m == 0 then return densesoil1
    elseif m == 1 then return densesoil2
    elseif m == 2 then return densesoil3
    elseif m == 3 then return densesoil4
    elseif m == 4 then return densesoil3
    end
    return densesoil2
  end
  if m == 0 then return soil1
  elseif m == 1 then return soil2
  elseif m == 2 then return soil3
  elseif m == 3 then return soil4
  elseif m == 4 then return soil3
  end
  return soil2
end

-- gml_Script_world_dirt_sprite @ 0x140367210
function world_dirt_sprite(self, other, row)
  local tier = world_soil_tier(self, other, row)
  local m = gm.mod(gm.real(world_soil_band(self, other, row)), 6.0)
  if tier == 2.0 then
    if m == 0 then return deepdirt1
    elseif m == 1 then return deepdirt2
    elseif m == 2 then return deepdirt3
    elseif m == 3 then return deepdirt4
    elseif m == 4 then return deepdirt3
    end
    return deepdirt2
  end
  if tier == 1.0 then
    if m == 0 then return densedirt1
    elseif m == 1 then return densedirt2
    elseif m == 2 then return densedirt3
    elseif m == 3 then return densedirt4
    elseif m == 4 then return densedirt3
    end
    return densedirt2
  end
  if m == 0 then return dirt1
  elseif m == 1 then return dirt2
  elseif m == 2 then return dirt3
  elseif m == 3 then return dirt4
  elseif m == 4 then return dirt3
  end
  return dirt2
end

-- gml_Script_world_soil_edge_sprite @ 0x140368050
function world_soil_edge_sprite(self, other, band, row)
  local tier = world_soil_tier(self, other, row - 1)
  local m = gm.mod(gm.real(band), 6.0)
  if tier == 2.0 then
    if m == 0 then return deepsoil1_edge
    elseif m == 1 then return deepsoil2_edge
    elseif m == 2 then return deepsoil3_edge
    elseif m == 3 then return deepsoil4_edge
    elseif m == 4 then return deepsoil3_edge
    end
    return deepsoil2_edge
  end
  if tier == 1.0 then
    if m == 0 then return densesoil1_edge
    elseif m == 1 then return densesoil2_edge
    elseif m == 2 then return densesoil3_edge
    elseif m == 3 then return densesoil4_edge
    elseif m == 4 then return densesoil3_edge
    end
    return densesoil2_edge
  end
  if m == 0 then return soil1_edge
  elseif m == 1 then return soil2_edge
  elseif m == 2 then return soil3_edge
  elseif m == 3 then return soil4_edge
  elseif m == 4 then return soil3_edge
  end
  return soil2_edge
end

-- gml_Script_world_draw_soil_edges @ 0x140368e70
function world_draw_soil_edges(self, other, x0, y0, vw, vh)
  local w = global.world
  local first = gm.max(1.0, wld(self, other, "soil_first_rows"))
  local band_rows = gm.max(1.0, wld(self, other, "soil_band_rows"))
  -- DAT_141379b30 is never written -> 0
  local band = gm.max(0, gm.div(gm.floor((y0 - w.surface_y) / w.tile - first), band_rows) - 4)
  local row_end = gm.min(w.rows, gm.ceil((y0 + vh - w.surface_y) / w.tile) + 1)
  local row = first + band * band_rows
  while row < row_end do
    local spr = world_soil_edge_sprite(self, other, band, row)
    local sw = gm.sprite_get_width(spr)
    band = band + 1
    if not (sw <= 0) then
      local yy = w.surface_y + row * w.tile
      if not (yy > y0 + vh) and not (yy + gm.sprite_get_height(spr) < y0) then
        local start = gm.floor(x0 / sw) * sw
        local stop = x0 + vw
        local xx = start
        while xx < stop do
          -- YYC: direct draw_sprite call with float coordinates
          gm.draw_sprite(spr, 0, xx, yy)
          xx = xx + sw
        end
      end
    end
    row = row + band_rows
  end
  return gm.undefined
end

-- gml_Script_world_bottom_is_rock @ 0x14036a7f0
function world_bottom_is_rock(self, other)
  return wld(self, other, "bottom_rock") > 0 and game_mode(self, other) == "survival"
end

local world_draw_bottom_rock_static = { _edge_set = nil, _fill_set = nil }

-- gml_Script_world_draw_bottom_rock @ 0x14036aa90
function world_draw_bottom_rock(self, other, x0, y0, vw, vh, oy)
  local st = world_draw_bottom_rock_static
  if st._edge_set == nil then
    -- DAT_141379b30 is never written -> 0
    st._edge_set = stone_role_tiles(self, other, true, false, false, false, 0)
    st._fill_set = stone_role_tiles(self, other, false, false, false, false, 0)
  end
  local ts = 48.0
  local col = vis(self, other, "deep_stone_colour")
  local c0 = gm.floor(x0 / ts)
  local c1 = gm.floor((x0 + vw) / ts)
  local r0 = gm.max(0, gm.floor((y0 - oy) / ts))
  local r1 = gm.floor((y0 + vh - oy) / ts)
  local r = r0
  while r <= r1 do
    local set
    if r == 0 then
      set = st._edge_set
    else
      set = st._fill_set
    end
    local yy = oy + r * ts
    local c = c0
    while c <= c1 do
      local t = stone_pick_tile(self, other, c, r, set, 41.0)
      gm.draw_sprite_part_ext(stone_tile, 0, t[0] * 64, t[1] * 64, ts, ts, c * ts, yy, 1.0, 1.0, col, 1.0)
      c = c + 1
    end
    r = r + 1
  end
  return gm.undefined
end

-- gml_Script_world_draw_ocean @ 0x14036c560
function world_draw_ocean(self, other, x0, y0, vw, vh)
  local oy = world_ocean_y(self, other)
  local bottom = y0 + vh
  if bottom <= oy then
    return gm.undefined
  end
  if gm.bool(world_bottom_is_rock(self, other)) then
    world_draw_bottom_rock(self, other, x0, y0, vw, vh, oy)
  end
  local x1 = x0 - 1
  local x2 = x0 + vw + 1
  local rock = world_bottom_is_rock(self, other)
  local col_water = vis(self, other, "water_reservoir_colour")
  local col_deep = vis(self, other, "ocean_deep_colour")
  local fade = vis(self, other, "ocean_fade_px")
  local top = gm.max(oy, y0)
  if not gm.bool(rock) and top < oy + fade then
    local fy = gm.min(bottom, oy + fade)
    local ca = gm.merge_colour(col_water, col_deep, (top - oy) / fade)
    local cb = gm.merge_colour(col_water, col_deep, (fy - oy) / fade)
    gm.draw_rectangle_colour(x1, top, x2, fy, ca, ca, cb, cb, false)
  end
  if not gm.bool(rock) and bottom > oy + fade then
    -- YYC: direct draw_set_colour / draw_rectangle calls (float coordinates)
    gm.draw_set_colour(col_deep)
    gm.draw_rectangle(x1, gm.max(top, oy + fade), x2, bottom, false)
  end
  gm.draw_set_colour(0xffffff)
  if not gm.bool(rock) then
    world_draw_ocean_flares(self, other, x0, y0, vw, vh, oy)
  end
  local orow = world_ocean_row(self, other)
  local spr = world_soil_edge_sprite(self, other, world_soil_band(self, other, orow - 1), orow)
  local sw = gm.sprite_get_width(spr)
  if sw > 0 and oy + gm.sprite_get_height(spr) >= y0 and oy <= bottom then
    local xx = gm.floor(x0 / sw) * sw
    while xx < x0 + vw do
      -- YYC: direct draw_sprite call with float coordinates
      gm.draw_sprite(spr, 0, xx, oy)
      xx = xx + sw
    end
  end
  return gm.undefined
end

-- gml_Script_world_draw_ocean_flares @ 0x14036e5f0
function world_draw_ocean_flares(self, other, x0, y0, vw, vh, oy)
  local cell = gm.max(32.0, vis(self, other, "ocean_flare_cell_px"))
  local chance = vis(self, other, "ocean_flare_chance")
  local fw = gm.div(gm.sprite_get_width(water_flare), 3)
  local fh = gm.sprite_get_height(water_flare)
  local gap = vis(self, other, "ocean_flare_top_gap_px")
  local life = lake_flare_life_ms(self, other)
  local fps = gm.max(1.0, vis(self, other, "water_flare_fps"))
  local now = gm.current_time
  local ytop = gm.max(y0 - fh, oy + gap)
  if ytop >= y0 + vh then
    return gm.undefined
  end
  local c0 = gm.floor((x0 - fw) / cell)
  local c1 = gm.floor((x0 + vw) / cell)
  local r0 = gm.floor((ytop - oy) / cell)
  local r1 = gm.floor((y0 + vh - oy) / cell)
  local r = r0
  while r <= r1 do
    local c = c0
    while c <= c1 do
      local t = now + world_hash01(self, other, c, r, 97.0) * life
      local cyc = gm.floor(t / life)
      local age = t - cyc * life
      if world_hash01(self, other, c * 7 + cyc, r, 98.0) < chance then
        local a = lake_flare_alpha(self, other, age)
        if not (a <= 0) then
          local fx = c * cell + world_hash01(self, other, c + cyc * 131, r, 99.0) * (cell - fw)
          local fy = oy + r * cell + world_hash01(self, other, c, r + cyc * 131, 100.0) * (cell - fh)
          if not (fy < oy + gap) then
            -- second fmod operand loaded from 0x140e07f10 = 3.0
            local frame = gm.mod(gm.real(gm.floor(age * fps / 1000.0)), 3.0)
            -- DAT_141379b30 is never written -> 0
            gm.draw_sprite_part_ext(water_flare, 0, frame * fw, 0, fw, fh, gm.floor(fx), gm.floor(fy), 1.0, 1.0, 16777215.0, a)
          end
        end
      end
      c = c + 1
    end
    r = r + 1
  end
  return gm.undefined
end
