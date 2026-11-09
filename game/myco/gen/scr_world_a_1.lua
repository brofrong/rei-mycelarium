local gm = require("gm")

-- gml_Script____struct___291__world_init__scr_world @ 0x1402c2780
function ___struct___291__world_init__scr_world(self, other, tile, surface_y, cols, chunk_w, bottom_y, hard_row, layer_rows, seed, turf, start_x, chunk_index)
  self.tile = tile
  self.surface_y = surface_y
  self.rows = 1000000.0
  self.cols = cols
  self.chunk_w = chunk_w
  self.bottom_y = bottom_y
  self.water_row = 1000000.0
  self.hard_row = hard_row
  self.layer_rows = layer_rows
  self.seed = seed
  self.turf = turf
  self.start_x = start_x
  self.chunks = gm.struct({})
  self.chunk_index = chunk_index
end

-- gml_Script_world_init @ 0x1402c2e30
function world_init(self, other, start_x)
  local tile = wld(self, other, "reference_tile_size")
  local surface_h = wld(self, other, "surface_height_px")
  local rows = gm.ceil(wld(self, other, "underground_screen_depth") * vis(self, other, "screen_height_px") / tile)
  local cols = wld(self, other, "chunk_cols")
  local hard_rows = gm.max(4.0, rows - wld(self, other, "bottom_water_layer_tiles") - gm.ceil(vis(self, other, "screen_height_px") / tile))
  global.world = gm.new(___struct___291__world_init__scr_world, tile, surface_h, cols, cols * tile,
    surface_h + 1000000.0 * tile, hard_rows, hard_rows, gm.irandom(99999), gm.asset_get_index("trintrava1"),
    start_x, gm.array())
  global.stones_moving = gm.array()
  world_start_guarantee(self, other)
  world_start_food_quota(self, other)
end

-- gml_Script____struct___293__world_start_ring__scr_world @ 0x1402c3f20
function ___struct___293__world_start_ring__scr_world(self, other, x, y, near, far, x1, x2, y1, y2)
  self.x = x
  self.y = y
  self.near = near
  self.far = far
  self.x1 = x1
  self.x2 = x2
  self.y1 = y1
  self.y2 = y2
end

-- gml_Script_world_start_ring @ 0x1402c4380
function world_start_ring(self, other)
  local range = bal(self, other, "base_shot_range")
  local w = global.world
  local pad = 56.0
  return gm.new(___struct___293__world_start_ring__scr_world, w.start_x, w.surface_y,
    range * wld(self, other, "start_resource_min_shots"),
    range * wld(self, other, "start_resource_max_shots"),
    w.start_x - vis(self, other, "screen_width_px") * 0.5 + pad,
    w.start_x + vis(self, other, "screen_width_px") * 0.5 - pad,
    w.surface_y + w.tile,
    vis(self, other, "screen_height_px") - pad)
end

-- gml_Script_world_start_on_screen @ 0x1402c5220
function world_start_on_screen(self, other, r, x1, y1, x2, y2)
  return x1 >= r.x1 and x2 <= r.x2 and y1 >= r.y1 and y2 <= r.y2
end

-- gml_Script_world_start_angle @ 0x1402c54f0
function world_start_angle(self, other, x, y)
  local spread = 50.0
  -- the second world_hash01 argument in both branches is DAT_141379b30 (never written -> 0)
  if world_hash01(self, other, x, 1.0, y) < 0.5 then
    return 352.0 - world_hash01(self, other, x, 0, y) * spread
  else
    return 188.0 + world_hash01(self, other, x, 0, y) * spread
  end
end

-- gml_Script_world_rect_distance @ 0x1402c5dd0
function world_rect_distance(self, other, px, py, x1, y1, x2, y2)
  local dx = gm.max(
    x1 - px,
    0,
    px - x2)
  local dy = gm.max(
    y1 - py,
    0,
    py - y2)
  return gm.sqrt(dx * dx + dy * dy)
end

-- gml_Script____struct___294__cave_defs__scr_world @ 0x1402c6570
function ___struct___294__cave_defs__scr_world(self, other, feet)
  self.id = "blue"
  self.sprite = cave_perk_blue
  self.left = 0
  self.w = 436.0
  self.hud = Blue
  self.feet = feet
end

-- gml_Script____struct___295__cave_defs__scr_world @ 0x1402c6880
function ___struct___295__cave_defs__scr_world(self, other, feet)
  self.id = "purple"
  self.sprite = cave_perk_purple
  self.left = 0
  self.w = 508.0
  self.hud = Purple
  self.feet = feet
end

-- gml_Script____struct___296__cave_defs__scr_world @ 0x1402c6b90
function ___struct___296__cave_defs__scr_world(self, other, feet)
  self.id = "red"
  self.sprite = cave_perk_red
  self.left = 0
  self.w = 436.0
  self.hud = Red
  self.feet = feet
end

-- gml_Script____struct___297__cave_defs__scr_world @ 0x1402c6ea0
function ___struct___297__cave_defs__scr_world(self, other, feet)
  self.id = "green"
  self.sprite = cave_perk_green
  self.left = 0
  self.w = 436.0
  self.hud = Green
  self.feet = feet
end

-- gml_Script____struct___298__cave_defs__scr_world @ 0x1402c71b0
function ___struct___298__cave_defs__scr_world(self, other, feet)
  self.id = "yellow"
  self.sprite = cave_perk_yellow
  self.left = 0
  self.w = 508.0
  self.hud = Yellow
  self.feet = feet
end

local cave_defs_static = {}

-- gml_Script_cave_defs @ 0x1402c74c0
function cave_defs(self, other)
  if not cave_defs_static.init then
    cave_defs_static.init = true
    cave_defs_static._defs = gm.array(
      gm.new(___struct___294__cave_defs__scr_world, gm.array(
        gm.array(145.0, 172.0),
        gm.array(170.0, 167.0),
        gm.array(202.0, 160.0),
        gm.array(212.0, 157.0),
        gm.array(240.0, 160.0))),
      gm.new(___struct___295__cave_defs__scr_world, gm.array(
        gm.array(245.0, 182.0),
        gm.array(262.0, 190.0),
        gm.array(285.0, 182.0),
        gm.array(305.0, 177.0),
        gm.array(317.0, 180.0))),
      gm.new(___struct___296__cave_defs__scr_world, gm.array(
        gm.array(150.0, 172.0),
        gm.array(172.0, 175.0),
        gm.array(187.0, 165.0),
        gm.array(207.0, 162.0),
        gm.array(235.0, 165.0))),
      gm.new(___struct___297__cave_defs__scr_world, gm.array(
        gm.array(142.0, 165.0),
        gm.array(172.0, 160.0),
        gm.array(190.0, 157.0))),
      gm.new(___struct___298__cave_defs__scr_world, gm.array(
        gm.array(245.0, 180.0),
        gm.array(265.0, 190.0),
        gm.array(285.0, 185.0),
        gm.array(302.0, 177.0),
        gm.array(312.0, 185.0))))
  end
  return cave_defs_static._defs
end

-- gml_Script_cave_def @ 0x1402c8960
function cave_def(self, other, id)
  local defs = cave_defs(self, other)
  local i = 0
  while i < gm.array_length(defs) do
    if defs[i].id == id then
      return defs[i]
    end
    i = i + 1
  end
  return defs[0]
end

-- gml_Script_cave_range @ 0x1402c8d10
function cave_range(self, other, r, t)
  return gm.lerp(r[0], r[1], t)
end

-- gml_Script____struct___299__cave_place__scr_world @ 0x1402c8ff0
function ___struct___299__cave_place__scr_world(self, other, id, def, k, x1, y1, x2, y2)
  self.id = id
  self.def = def
  self.k = k
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end

-- gml_Script_cave_place @ 0x1402c93a0
function cave_place(self, other, list, kind, id, cx, cy, lo, hi, extra)
  if gm.is_undefined(extra) then
    extra = gm.array()
  end
  local def = cave_def(self, other, kind)
  local sc = vis(self, other, "cave_scale")
  local w = gm.round(def.w * sc)
  local h = gm.round(gm.sprite_get_height(def.sprite) * sc)
  local margin = wld(self, other, "caves_margin_px")
  local gap = wld(self, other, "caves_clearance_px") * 2
  local y0 = gm.floor(gm.clamp(cy - h * 0.5, lo + margin, gm.max(lo + margin, hi - margin - h)))
  local i = 0
  while i < 12 do
    local dir
    if gm.mod(i, 2) == 1.0 then
      dir = 1.0
    else
      dir = -1.0
    end
    local off = gm.div(i + 1, 2) * dir * (w + gap)
    local x0 = gm.floor(cx + off - w * 0.5)
    local ok = true
    local j = 0
    while j < gm.array_length(list) and ok do
      local r = list[j]
      ok = x0 + w + gap <= r.x1 or x0 >= r.x2 + gap or y0 + h + gap <= r.y1 or y0 >= r.y2 + gap
      j = j + 1
    end
    j = 0
    while j < gm.array_length(extra) and ok do
      local r = extra[j]
      ok = x0 + w + gap <= r.x1 or x0 >= r.x2 + gap or y0 + h + gap <= r.y1 or y0 >= r.y2 + gap
      j = j + 1
    end
    if ok then
      local p = gm.new(___struct___299__cave_place__scr_world, id, def, sc, x0, y0, x0 + w, y0 + h)
      gm.array_push(list, p)
      return p
    end
    i = i + 1
  end
  return gm.undefined
end

-- gml_Script_cave_host_pick @ 0x1402cc150
function cave_host_pick(self, other, hosts, side, dist)
  local sx = global.world.start_x
  local best = gm.undefined
  local best_d = math.huge
  local i = 0
  while i < gm.array_length(hosts) do
    local h = hosts[i]
    local d = (h.x1 + h.x2) * 0.5 - sx
    if not gm.bool(h.used) then
      if side == 0 or gm.sign(d) == side then
        local e = gm.abs(gm.abs(d) - dist)
        if e < best_d then
          best_d = e
          best = h
        end
      end
    end
    i = i + 1
  end
  return best
end

-- gml_Script____struct___300__world_cave_rects__scr_world @ 0x1402ccae0
function ___struct___300__world_cave_rects__scr_world(self, other, key, x1, y1, x2, y2)
  self.key = key
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.used = false
end

-- gml_Script____struct___301__world_cave_rects__scr_world @ 0x1402cce20
function ___struct___301__world_cave_rects__scr_world(self, other, key, x1, y1, x2, y2, cx, cy, r)
  self.key = key
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.used = false
  self.cx = cx
  self.cy = cy
  self.r = r
end

-- gml_Script_anon__10783__world_cave_rects__scr_world @ 0x1402cd290
function anon__10783__world_cave_rects__scr_world(self, other, list, kind, id, host, tx, ty, lo, hi)
  local def = cave_def(self, other, kind)
  local cx = gm.lerp(
    host.x1 + def.w * 0.5 + 144,
    host.x2 - def.w * 0.5 - 144,
    tx)
  local cy = gm.lerp(
    host.y1 + 336,
    host.y2 - 432,
    ty)
  host.used = true
  return cave_place(self, other, list, kind, id,
    cx,
    cy,
    gm.max(lo, host.y1),
    gm.min(hi, host.y2),
    gm.undefined)
end

-- gml_Script_anon__11290__world_cave_rects__scr_world @ 0x1402ce2f0
function anon__11290__world_cave_rects__scr_world(self, other, list, kind, id, host, tx, ty)
  host.used = true
  return cave_place(self, other, list, kind, id,
    host.cx + (tx - 0.5) * host.r * 48 * 0.4,
    host.cy + (ty - 0.5) * host.r * 48 * 0.3,
    host.y1,
    host.y2,
    gm.undefined)
end

-- gml_Script____struct___303_____struct___302__world_cave_rects__scr_world @ 0x1402ceea0
function ___struct___303_____struct___302__world_cave_rects__scr_world(self, other, dir, gap)
  self.dir = dir
  self.n = 0
  self.gap = gap
  self.cursor = 0
end

-- gml_Script____struct___304_____struct___302__world_cave_rects__scr_world @ 0x1402cf0c0
function ___struct___304_____struct___302__world_cave_rects__scr_world(self, other, gap)
  self.dir = 1.0
  self.n = 0
  self.gap = gap
  self.cursor = 0
end

-- gml_Script____struct___302__world_cave_rects__scr_world @ 0x1402cf2c0
function ___struct___302__world_cave_rects__scr_world(self, other, mazes, walls, left_dir, left_gap, right_gap)
  self.mazes = mazes
  self.walls = walls
  self.left = gm.new(___struct___303_____struct___302__world_cave_rects__scr_world, left_dir, left_gap)
  self.right = gm.new(___struct___304_____struct___302__world_cave_rects__scr_world, right_gap)
end

local world_cave_rects_static = {
  _cache = gm.undefined,
  _for_start = gm.undefined,
  _for_seed = gm.undefined,
  _for_yx = gm.undefined,
  _for_yy = gm.undefined,
}

-- gml_Script_world_cave_rects @ 0x1402cf8b0
function world_cave_rects(self, other)
  local st = world_cave_rects_static
  local w = global.world
  local spot = cave_yellow_spot(self, other)
  local yx
  if gm.is_struct(spot) then
    yx = spot.x
  else
    yx = gm.undefined
  end
  local yy
  if gm.is_struct(spot) then
    yy = spot.y
  else
    yy = gm.undefined
  end
  if not gm.is_undefined(st._cache) and st._for_start == w.start_x and st._for_seed == w.seed
    and st._for_yx == yx and st._for_yy == yy then
    return st._cache
  end
  local out = gm.array()
  st._for_start = w.start_x
  st._for_seed = w.seed
  st._for_yx = yx
  st._for_yy = yy
  st._cache = out
  if wld(self, other, "caves_enabled") <= 0 then
    return out
  end
  local sw = vis(self, other, "screen_width_px")
  local sh = vis(self, other, "screen_height_px")
  local sx = w.start_x
  local sy = w.surface_y
  local top = sy + w.tile * 2
  local dense = world_dense_y(self, other)
  local deep = world_deep_y(self, other)
  local ocean = world_ocean_y(self, other)
  local cell = 48
  local reach = sw * wld(self, other, "caves_host_reach_screens")
  local mazes = gm.array()
  local walls = gm.array()
  local wall_depth = gm.round(wld(self, other, "surface_wall_depth_screens") * sh / cell) * cell
  local side = -1
  while side <= 1 do
    local obs = world_obstacles_reach(self, other, side, sx + side * reach)
    local i = 0
    while i < gm.array_length(obs) do
      local o = obs[i]
      local r = gm.new(___struct___300__world_cave_rects__scr_world, o.key, o.x1, sy, o.x2, sy + wall_depth)
      if o.kind == "maze" then
        gm.array_push(mazes, r)
      else
        gm.array_push(walls, r)
      end
      i = i + 1
    end
    side = side + 2
  end
  local clay = gm.array()
  local hard = gm.array()
  local clusters = world_rock_clusters(self, other)
  local i = 0
  while i < gm.array_length(clusters) do
    local c = clusters[i]
    local r = gm.new(___struct___301__world_cave_rects__scr_world, c.id, c.x1, c.y1, c.x2, c.y2,
      (c.col + 0.5) * cell, (c.row + 0.5) * cell, c.r)
    if c.tier == "clay" then
      gm.array_push(clay, r)
    else
      gm.array_push(hard, r)
    end
    i = i + 1
  end
  local place_host = gm.method(self, anon__10783__world_cave_rects__scr_world)
  local place_cluster = gm.method(self, anon__11290__world_cave_rects__scr_world)
  -- the side argument is DAT_141379b30 (never written -> 0)
  local host = cave_host_pick(self, other, mazes, 0,
    sw * cave_range(self, other, wld(self, other, "caves_purple_x"), world_hash01(self, other, 2.0, 2.0, 303.0)))
  if not gm.is_undefined(host) then
    place_host(self, other, out, "purple", "purple", host,
      world_hash01(self, other, 2.0, 4.0, 303.0), world_hash01(self, other, 2.0, 3.0, 303.0), top, dense)
  else
    cave_place(self, other, out, "purple", "purple",
      sx + sw * cave_range(self, other, wld(self, other, "caves_purple_x"), world_hash01(self, other, 2.0, 2.0, 303.0)),
      sy + sh * cave_range(self, other, wld(self, other, "caves_purple_depth"), world_hash01(self, other, 2.0, 3.0, 303.0)),
      top, dense, walls)
  end
  local dir
  if world_hash01(self, other, 3.0, 1.0, 304.0) < 0.5 then
    dir = -1
  else
    dir = 1
  end
  local kinds = gm.array("red", "green")
  local k = 0
  while k < 2 do
    local s
    if k == 0 then
      s = dir
    else
      s = -dir
    end
    host = cave_host_pick(self, other, clay, s,
      sw * cave_range(self, other, wld(self, other, "caves_clay_x"), world_hash01(self, other, 3 + k, 2.0, 304.0)))
    if not gm.is_undefined(host) then
      place_cluster(self, other, out, kinds[k], kinds[k], host,
        world_hash01(self, other, 3 + k, 4.0, 304.0), world_hash01(self, other, 3 + k, 3.0, 304.0))
    end
    k = k + 1
  end
  if world_hash01(self, other, 1.0, 1.0, 301.0) < 0.5 then
    dir = -1
  else
    dir = 1
  end
  local first = wld(self, other, "caves_blue_first_x")
  local state = gm.new(___struct___302__world_cave_rects__scr_world, mazes, walls, -1.0,
    sw * wld(self, other, "caves_blue_every_x"), sw * wld(self, other, "caves_blue_every_x"))
  local b = cave_blue_place(self, other, out, state, "blue_0",
    sx + dir * sw * cave_range(self, other, first, world_hash01(self, other, 1.0, 2.0, 301.0)),
    sy + sh * cave_range(self, other, wld(self, other, "caves_blue_depth"), world_hash01(self, other, 1.0, 3.0, 301.0)))
  local bx
  if gm.is_undefined(b) then
    bx = sx + dir * sw * first[1]
  else
    bx = (b.x1 + b.x2) * 0.5
  end
  local rgap = state.right.gap
  local rbase
  if dir > 0 then
    rbase = bx
  else
    rbase = sx + sw * first[0]
  end
  state.right.cursor = rbase + rgap
  local lgap = state.left.gap
  local lbase
  if dir < 0 then
    lbase = bx
  else
    lbase = sx - sw * first[0]
  end
  state.left.cursor = lbase - lgap
  global.cave_blue_state = state
  caves_extend(self, other, sx - reach * 0.5)
  caves_extend(self, other, sx + reach * 0.5)
  if gm.is_struct(spot) then
    cave_place(self, other, out, "yellow", "yellow", spot.x, spot.y, spot.top, spot.bottom, world_big_bones(self, other))
  end
  return out
end

-- gml_Script_cave_yellow_spot @ 0x1402d4300
function cave_yellow_spot(self, other)
  if gm.variable_global_exists("cave_yellow") and gm.is_struct(global.cave_yellow) then
    return global.cave_yellow
  end
  return gm.undefined
end

-- gml_Script_world_area_untouched @ 0x1402d4630
function world_area_untouched(self, other, x1, y1, x2, y2, pad)
  local w = global.world
  local c1 = gm.floor((x1 - pad) / w.chunk_w)
  local c2 = gm.floor((x2 + pad) / w.chunk_w)
  local c = c1
  while c <= c2 do
    local ch = gm.variable_struct_get(w.chunks, world_chunk_key(self, other, c))
    if not gm.is_undefined(ch) then
      if w.surface_y + ch.rows * w.tile > y1 - pad then
        return false
      end
    end
    c = c + 1
  end
  return true
end

-- gml_Script____struct___305__cave_yellow_pick__scr_world @ 0x1402d5190
function ___struct___305__cave_yellow_pick__scr_world(self, other, x, y, top, bottom, cluster)
  self.x = x
  self.y = y
  self.top = top
  self.bottom = bottom
  self.cluster = cluster
end

-- gml_Script_cave_yellow_pick @ 0x1402d54c0
function cave_yellow_pick(self, other)
  local w = global.world
  local sw = vis(self, other, "screen_width_px")
  local beyond = sw * cave_range(self, other, wld(self, other, "caves_yellow_beyond_x"), world_hash01(self, other, 5.0, 2.0, 305.0))
  local pad = wld(self, other, "caves_clearance_px") + wld(self, other, "caves_margin_px")
  local dir
  if world_hash01(self, other, 5.0, 1.0, 305.0) < 0.5 then
    dir = -1.0
  else
    dir = 1.0
  end
  local clusters = world_rock_clusters(self, other)
  local best
  local t = 0
  while true do
    if t >= 2.0 then
      return gm.undefined
    end
    local s
    if t == 0 then
      s = dir
    else
      s = -dir
    end
    local tip = mycelium_extreme_tip(self, other, s)
    local tx
    if gm.is_undefined(tip) then
      tx = w.start_x
    else
      tx = tip.x
    end
    local need = gm.max(0, (tx - w.start_x) * s) + beyond
    best = gm.undefined
    local best_d = math.huge
    local i = 0
    while i < gm.array_length(clusters) do
      local c = clusters[i]
      if c.tier == "deep" then
        local d = ((c.x1 + c.x2) * 0.5 - w.start_x) * s
        if d >= need and not (d - need >= best_d) then
          if gm.bool(world_area_untouched(self, other, c.x1, c.y1, c.x2, c.y2, pad)) then
            best = c
            best_d = d - need
          end
        end
      end
      i = i + 1
    end
    if not gm.is_undefined(best) then
      break
    end
    t = t + 1
  end
  return gm.new(___struct___305__cave_yellow_pick__scr_world,
    (best.col + 0.5) * 48 + (world_hash01(self, other, 5.0, 4.0, 305.0) - 0.5) * best.r * 48 * 0.4,
    (best.row + 0.5) * 48 + (world_hash01(self, other, 5.0, 3.0, 305.0) - 0.5) * best.r * 48 * 0.3,
    best.y1, best.y2, best.id)
end

-- gml_Script_cave_yellow_step @ 0x1402d83c0
function cave_yellow_step(self, other)
  if gm.bool(global.title_open) or gm.bool(global.intro_active) or gm.bool(global.pause_open) then
    return
  end
  global.play_frames = global.play_frames + 1
  if not gm.is_undefined(cave_yellow_spot(self, other)) or gm.bool(perk_has(self, other, "yellow"))
    or not (wld(self, other, "caves_enabled") > 0) then
    return
  end
  -- game_get_speed argument is DAT_141379b30 (never written -> 0 = gamespeed_fps)
  local fps = gm.max(1.0, gm.game_get_speed(0))
  if global.play_frames < wld(self, other, "caves_yellow_after_minutes") * 60 * fps then
    return
  end
  if gm.mod(global.play_frames, fps * 5) == 0 then
    local spot = cave_yellow_pick(self, other)
    if not gm.is_undefined(spot) then
      global.cave_yellow = spot
      world_cave_rects(self, other)
    end
  end
end

-- gml_Script_cave_blue_place @ 0x1402d8ea0
function cave_blue_place(self, other, list, st, id, x, y)
  local w = global.world
  local top = w.surface_y + w.tile * 2
  local avoid = gm.array_concat(st.walls, gm.array())
  local i = 0
  while i < gm.array_length(st.mazes) do
    if gm.bool(st.mazes[i].used) then
      gm.array_push(avoid, st.mazes[i])
    end
    i = i + 1
  end
  local p = cave_place(self, other, list, "blue", id, x, y, top, world_dense_y(self, other), avoid)
  if not gm.is_undefined(p) then
    i = 0
    while i < gm.array_length(st.mazes) do
      local m = st.mazes[i]
      if p.x2 > m.x1 and p.x1 < m.x2 and p.y2 > m.y1 and p.y1 < m.y2 then
        m.used = true
      end
      i = i + 1
    end
  end
  return p
end

-- gml_Script____struct___306__caves_extend__scr_world @ 0x1402da040
function ___struct___306__caves_extend__scr_world(self, other, key, x1, y1, x2, y2)
  self.key = key
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
  self.used = false
end

-- gml_Script_caves_extend @ 0x1402da390
function caves_extend(self, other, x)
  if not (wld(self, other, "caves_blue_more") > 0) then
    return
  end
  local rects = world_cave_rects(self, other)
  if not (gm.variable_global_exists("cave_blue_state") and gm.is_struct(global.cave_blue_state)) then
    return
  end
  local w = global.world
  local st = global.cave_blue_state
  local sw = vis(self, other, "screen_width_px")
  local sh = vis(self, other, "screen_height_px")
  local dir
  if x >= w.start_x then
    dir = 1.0
  else
    dir = -1.0
  end
  local lane
  if dir <= 0 then
    lane = st.left
  else
    lane = st.right
  end
  local target = x + dir * sw * wld(self, other, "caves_extend_ahead_screens")
  while true do
    local go
    if dir <= 0 then
      go = lane.cursor >= target
    else
      go = lane.cursor <= target
    end
    if not go then
      break
    end
    lane.n = lane.n + 1
    local off = dir * lane.n
    local jit = (world_hash01(self, other, off, 4.0, 302.0) - 0.5) * 2 * wld(self, other, "caves_blue_jitter") * lane.gap
    local cx = lane.cursor + jit
    local obs = world_obstacles_reach(self, other, dir, cx + dir * sw * 4)
    local j = 0
    while j < gm.array_length(obs) do
      local o = obs[j]
      local found = false
      local arr
      if o.kind == "maze" then
        arr = st.mazes
      else
        arr = st.walls
      end
      local k = 0
      while k < gm.array_length(arr) and not found do
        found = arr[k].key == o.key
        k = k + 1
      end
      if not found then
        gm.array_push(arr, gm.new(___struct___306__caves_extend__scr_world, o.key, o.x1, w.surface_y, o.x2,
          w.surface_y + gm.round(wld(self, other, "surface_wall_depth_screens") * sh / 48.0) * 48))
      end
      j = j + 1
    end
    cave_blue_place(self, other, rects, st, "blue_" .. gm.string(off), cx,
      w.surface_y + sh * cave_range(self, other, wld(self, other, "caves_blue_depth"), world_hash01(self, other, off, 5.0, 302.0)))
    lane.gap = lane.gap * gm.max(1.0, wld(self, other, "caves_blue_gap_growth"))
    lane.cursor = lane.cursor + dir * lane.gap
  end
end

local world_clear_zones_static = { _cache = gm.undefined, _key = "" }

-- gml_Script_world_clear_zones @ 0x1402dd780
function world_clear_zones(self, other)
  local st = world_clear_zones_static
  local w = global.world
  local key = gm.string(w.start_x) .. ":" .. gm.string(w.seed)
  local rects = world_cave_rects(self, other)
  key = key .. (":" .. gm.string(gm.array_length(rects)))
  if not (not gm.is_undefined(st._cache) and st._key == key) then
    st._key = key
    st._cache = gm.array_concat(rects, world_big_bones(self, other))
  end
  return st._cache
end

-- gml_Script____struct___307__bone_kinds__scr_world @ 0x1402de1f0
function ___struct___307__bone_kinds__scr_world(self, other)
  self.id = "squirrel"
  self.sprite = spr_skeleton_squirrel
  self.weight = 30.0
  self.near = 0.8
  self.far = 2.4
  self.deep_share = 0
end

-- gml_Script____struct___308__bone_kinds__scr_world @ 0x1402de4a0
function ___struct___308__bone_kinds__scr_world(self, other)
  self.id = "hedgehog"
  self.sprite = spr_skeleton_hedgehog
  self.weight = 30.0
  self.near = 0.8
  self.far = 2.4
  self.deep_share = 0
end

-- gml_Script____struct___309__bone_kinds__scr_world @ 0x1402de750
function ___struct___309__bone_kinds__scr_world(self, other)
  self.id = "crow"
  self.sprite = spr_skeleton_crow
  self.weight = 24.0
  self.near = 0.6
  self.far = 2.0
  self.deep_share = 0
end

-- gml_Script____struct___310__bone_kinds__scr_world @ 0x1402dea00
function ___struct___310__bone_kinds__scr_world(self, other)
  self.id = "pig"
  self.sprite = spr_skeleton_pig
  self.weight = 10.0
  self.near = 1.5
  self.far = 3.5
  self.deep_share = 0.2
end

-- gml_Script____struct___311__bone_kinds__scr_world @ 0x1402decc0
function ___struct___311__bone_kinds__scr_world(self, other)
  self.id = "wolf"
  self.sprite = spr_skeleton_wolf
  self.weight = 8.0
  self.near = 1.5
  self.far = 3.5
  self.deep_share = 0.2
end

-- gml_Script____struct___312__bone_kinds__scr_world @ 0x1402def80
function ___struct___312__bone_kinds__scr_world(self, other)
  self.id = "elk"
  self.sprite = spr_skeleton_elk
  self.weight = 5.0
  self.near = 1.5
  self.far = 3.5
  self.deep_share = 0.2
end

local bone_kinds_static = {}

-- gml_Script_bone_kinds @ 0x1402df240
function bone_kinds(self, other)
  if not bone_kinds_static.init then
    bone_kinds_static.init = true
    bone_kinds_static._kinds = gm.array(
      gm.new(___struct___307__bone_kinds__scr_world),
      gm.new(___struct___308__bone_kinds__scr_world),
      gm.new(___struct___309__bone_kinds__scr_world),
      gm.new(___struct___310__bone_kinds__scr_world),
      gm.new(___struct___311__bone_kinds__scr_world),
      gm.new(___struct___312__bone_kinds__scr_world))
  end
  return bone_kinds_static._kinds
end

-- gml_Script_bone_kind_weight @ 0x1402dfd20
function bone_kind_weight(self, other, k, depth)
  local t = gm.clamp((depth - k.near) / gm.max(0.01, k.far - k.near), 0, 1.0)
  return k.weight * gm.lerp(1.0, k.deep_share, t)
end

-- gml_Script____struct___313__bone_kind_pick__scr_world @ 0x1402e0350
function ___struct___313__bone_kind_pick__scr_world(self, other, kind)
  self.kind = kind
  self.share = 0
end

-- gml_Script____struct___314__bone_kind_pick__scr_world @ 0x1402e04e0
function ___struct___314__bone_kind_pick__scr_world(self, other, kind, share)
  self.kind = kind
  self.share = share
end

-- gml_Script____struct___315__bone_kind_pick__scr_world @ 0x1402e06a0
function ___struct___315__bone_kind_pick__scr_world(self, other, kind, share)
  self.kind = kind
  self.share = share
end

-- gml_Script_bone_kind_pick @ 0x1402e0860
function bone_kind_pick(self, other, r, depth)
  local kinds = bone_kinds(self, other)
  local total = 0
  local base = 0
  local i = 0
  while i < gm.array_length(kinds) do
    total = total + bone_kind_weight(self, other, kinds[i], depth)
    base = base + kinds[i].weight
    i = i + 1
  end
  if not (total > 0) then
    return gm.new(___struct___313__bone_kind_pick__scr_world, -1.0)
  end
  local roll = r * total
  i = 0
  while i < gm.array_length(kinds) do
    roll = roll - bone_kind_weight(self, other, kinds[i], depth)
    if roll < 0 then
      return gm.new(___struct___314__bone_kind_pick__scr_world, i, total / base)
    end
    i = i + 1
  end
  return gm.new(___struct___315__bone_kind_pick__scr_world, gm.array_length(kinds) - 1, total / base)
end

-- gml_Script____struct___316__world_paint_bones__scr_world @ 0x1402e1650
function ___struct___316__world_paint_bones__scr_world(self, other, kind, x, y, w, h, flip)
  self.kind = kind
  self.x = x
  self.y = y
  self.w = w
  self.h = h
  self.flip = flip
end

-- gml_Script_world_paint_bones @ 0x1402e19f0
function world_paint_bones(self, other, ch, layer)
  local w = global.world
  local key = world_layer_key(self, other, ch.cx, layer)
  local x0 = ch.cx * w.chunk_w
  local top = world_layer_top(self, other, layer) + w.tile
  local bot = gm.min(world_layer_bottom(self, other, layer), world_ocean_y(self, other) - wld(self, other, "ocean_clear_margin_px"))
  local kinds = bone_kinds(self, other)
  if not gm.variable_struct_exists(ch, "bones") then
    ch.bones = gm.array()
  end
  local t = 0
  while t < wld(self, other, "bones_tries_per_layer") do
    local y = gm.lerp(top, bot, world_hash01(self, other, key, t, 404.0))
    local pick = bone_kind_pick(self, other, world_hash01(self, other, key, t, 402.0), (y - w.surface_y) / vis(self, other, "screen_height_px"))
    if pick.kind >= 0 and not (world_hash01(self, other, key, t, 401.0) >= wld(self, other, "bones_chance") * pick.share) then
      local k = pick.kind
      local spr = kinds[k].sprite
      local bw = gm.sprite_get_width(spr)
      local bh = gm.sprite_get_height(spr)
      if bot - top >= bh + 16 then
        local bx = gm.floor(x0 + world_hash01(self, other, key, t, 403.0) * gm.max(1.0, w.chunk_w - bw))
        local by = gm.floor(gm.clamp(y - bh * 0.5, top, bot - bh))
        local skip = false
        if gm.abs(bx + bw * 0.5 - w.start_x) < vis(self, other, "screen_width_px") * 0.5 then
          if by < w.surface_y + vis(self, other, "screen_height_px") then
            skip = true
          end
        end
        if not skip and not gm.bool(world_cave_hits(self, other, bx, by, bx + bw, by + bh, gm.undefined)) then
          local ok = true
          local i = 0
          while i < 5.0 and ok do
            local fx
            if i == 4.0 then
              fx = 0.5
            else
              fx = math.fmod(i, 2.0)
            end
            local px = bx + fx * bw
            local fy
            if i == 4.0 then
              fy = 0.5
            else
              fy = gm.div(i, 2)
            end
            local py = by + fy * bh
            local good = false
            if not gm.bool(world_lake_covers(self, other, ch, px, py)) then
              if not gm.bool(world_stone_covers(self, other, ch, px, py)) then
                good = world_chunk_get(self, other, ch, gm.floor(px / w.tile), gm.floor((py - w.surface_y) / w.tile)) ~= 2.0
              end
            end
            ok = good
            i = i + 1
          end
          local j = 0
          while j < gm.array_length(ch.bones) and ok do
            local b = ch.bones[j]
            ok = bx + bw + 32 <= b.x or bx >= b.x + b.w + 32 or by + bh + 32 <= b.y or by >= b.y + b.h + 32
            j = j + 1
          end
          if ok then
            local flip
            if world_hash01(self, other, key, t, 405.0) < 0.5 then
              flip = -1.0
            else
              flip = 1.0
            end
            gm.array_push(ch.bones, gm.new(___struct___316__world_paint_bones__scr_world, k, bx, by, bw, bh, flip))
          end
        end
      end
    end
    t = t + 1
  end
end

-- gml_Script____struct___317__world_big_bones__scr_world @ 0x1402e6070
function ___struct___317__world_big_bones__scr_world(self, other, side)
  self.id = "t_rex"
  self.sprite = spr_skeleton_T_rex
  self.side = side
end

-- gml_Script____struct___318__world_big_bones__scr_world @ 0x1402e62a0
function ___struct___318__world_big_bones__scr_world(self, other, side)
  self.id = "ufo"
  self.sprite = spr_UFO
  self.side = side
end

-- gml_Script____struct___319__world_big_bones__scr_world @ 0x1402e64d0
function ___struct___319__world_big_bones__scr_world(self, other, id, sprite, x1, y1, x2, y2)
  self.id = id
  self.sprite = sprite
  self.x1 = x1
  self.y1 = y1
  self.x2 = x2
  self.y2 = y2
end
