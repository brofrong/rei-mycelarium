local gm = require("gm")

-- gml_Script____struct___211__stones_init__scr_stones @ 0x14021aee0
function ___struct___211__stones_init__scr_stones(self, other, sprite, sprite_grey, left, w, ox, oy, radius, body)
  self.sprite = sprite
  self.sprite_grey = sprite_grey
  self.left = left
  self.w = w
  self.ox = ox
  self.oy = oy
  self.radius = radius
  self.body = body
end

-- gml_Script_stones_init @ 0x14021b330
function stones_init(self, other)
  if stones_ready(self, other) then
    return
  end
  local sw = gm.sprite_get_width(stones)
  local sh = gm.sprite_get_height(stones)
  local surf = gm.surface_create(sw, sh)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_inv_src_alpha)
  gm.draw_sprite(stones, 0, gm.sprite_get_xoffset(stones), gm.sprite_get_yoffset(stones))
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local buf = gm.buffer_create(sw * sh * 4, 0, 1)
  gm.buffer_get_surface(buf, surf, 0)
  local slices = stones_slice_sheet(self, other, buf, sw, sh)
  local kinds = gm.array()
  local reach = 0
  local i = 0
  while i < 7 do
    local left = slices[i].left
    local width = slices[i].width
    local x0 = width
    local y0 = sh
    local x1 = -1
    local y1 = -1
    local yy = 0
    while yy < sh do
      local xx = 0
      while xx < width do
        local a = gm.buffer_peek(buf, (yy * sw + left + xx) * 4 + 3, 1)
        if a < 128 then
          -- continue
        else
          x0 = gm.min(x0, xx)
          y0 = gm.min(y0, yy)
          x1 = gm.max(x1, xx)
          y1 = gm.max(y1, yy)
        end
        xx = xx + 1
      end
      yy = yy + 1
    end
    if x1 < x0 then
      x0 = 0
      y0 = 0
      x1 = width - 1
      y1 = sh - 1
    end
    local ox = gm.round((x0 + x1 + 1) * 0.5)
    local oy = gm.round((y0 + y1 + 1) * 0.5)
    local ax = gm.max(ox - x0, x1 + 1 - ox)
    local ay = gm.max(oy - y0, y1 + 1 - oy)
    local radius = gm.point_distance(0, 0, ax, ay)
    reach = gm.max(reach, radius)
    gm.array_push(kinds, gm.new(___struct___211__stones_init__scr_stones,
      gm.sprite_create_from_surface(surf, left, 0, width, sh, false, false, ox, oy),
      -1, left, width, ox, oy, radius, (ax + ay) * 0.5))
    i = i + 1
  end
  local grey = vis(self, other, "stone_grey_amount")
  local p = 0
  while p < sw * sh do
    local o = p * 4
    local r = gm.buffer_peek(buf, o, 1)
    local g = gm.buffer_peek(buf, o + 1, 1)
    local b = gm.buffer_peek(buf, o + 2, 1)
    local l = 0.3 * r + 0.59 * g + 0.11 * b
    gm.buffer_poke(buf, o, 1, gm.round(gm.lerp(r, l, grey)))
    gm.buffer_poke(buf, o + 1, 1, gm.round(gm.lerp(g, l, grey)))
    gm.buffer_poke(buf, o + 2, 1, gm.round(gm.lerp(b, l * 1.04, grey)))
    p = p + 1
  end
  gm.buffer_set_surface(buf, surf, 0)
  i = 0
  while i < 7 do
    local k = kinds[i]
    k.sprite_grey = gm.sprite_create_from_surface(surf, k.left, 0, k.w, sh, false, false, k.ox, k.oy)
    i = i + 1
  end
  gm.buffer_delete(buf)
  gm.surface_free(surf)
  global.stone_kinds = kinds
  global.stone_reach = reach
end

-- gml_Script_stones_ready @ 0x140220340
function stones_ready(self, other)
  local r = false
  if gm.variable_global_exists("stone_kinds") then
    r = gm.is_array(global.stone_kinds)
  end
  return r
end

-- gml_Script____struct___212__stones_slice_sheet__scr_stones @ 0x140220590
function ___struct___212__stones_slice_sheet__scr_stones(self, other, left, width)
  self.left = left
  self.width = width
end

-- gml_Script____struct___213__stones_slice_sheet__scr_stones @ 0x140220750
function ___struct___213__stones_slice_sheet__scr_stones(self, other, left, width)
  self.left = left
  self.width = width
end

-- gml_Script____struct___214__stones_slice_sheet__scr_stones @ 0x140220910
function ___struct___214__stones_slice_sheet__scr_stones(self, other, left, width)
  self.left = left
  self.width = width
end

-- gml_Script_stones_slice_sheet @ 0x140220ad0
function stones_slice_sheet(self, other, buf, w, h)
  local slices = gm.array()
  local start = -1
  local x = 0
  while x < w do
    local filled = false
    local y = 0
    while y < h do
      if gm.buffer_peek(buf, (y * w + x) * 4 + 3, 1) > 0 then
        filled = true
        break
      end
      y = y + 1
    end
    if gm.bool(filled) then
      if start < 0 then
        start = x
      end
    else
      if start >= 0 then
        gm.array_push(slices, gm.new(___struct___212__stones_slice_sheet__scr_stones, start, x - start))
        start = -1
      end
    end
    x = x + 1
  end
  if start >= 0 then
    gm.array_push(slices, gm.new(___struct___213__stones_slice_sheet__scr_stones, start, w - start))
  end
  if gm.is_array(slices) and gm.array_length(slices) == 7 then
    -- keep
  else
    slices = gm.array()
    local i = 0
    while i < 7 do
      local l = gm.floor(i * w / 7)
      gm.array_push(slices, gm.new(___struct___214__stones_slice_sheet__scr_stones, l, gm.floor((i + 1) * w / 7) - l))
      i = i + 1
    end
  end
  return slices
end

-- gml_Script_stones_wake @ 0x1402223c0
function stones_wake(self, other, s)
  if gm.bool(s.moving) then
    return
  end
  s.moving = true
  gm.array_push(global.stones_moving, s)
end

-- gml_Script_stones_shove @ 0x1402226c0
function stones_shove(self, other, x, y, dir, force)
  if not stones_ready(self, other) then
    return
  end
  local w = global.world
  local reach = wld(self, other, "stone_shove_reach")
  local r = global.stone_reach + reach
  local dx = gm.lengthdir_x(1, dir)
  local dy = gm.lengthdir_y(1, dir)
  local px = -dy
  local py = dx
  local c0 = gm.floor((x - r) / w.chunk_w)
  local c1 = gm.floor((x + r) / w.chunk_w)
  local lead = wld(self, other, "stone_shove_lead")
  local spin = wld(self, other, "stone_shove_spin")
  local wobble = wld(self, other, "stone_shove_wobble")
  local c = c0
  while c <= c1 do
    local list = world_get_chunk(self, other, c).stones
    local i = 0
    while i < (gm.is_array(list) and gm.array_length(list) or 0) do
      local s = list[i]
      local ox = s.x - x
      local oy = s.y - y
      local d = gm.point_distance(0, 0, ox, oy)
      local rr = global.stone_kinds[s.kind].radius + reach
      if d > rr then
        -- continue
      else
        local lat = ox * px + oy * py
        local side
        if gm.abs(lat) <= 1 then
          side = s.side
        else
          side = gm.sign(lat)
        end
        local f = force * (1 - d / rr)
        local nx = px * side + dx * lead
        local ny = py * side + dy * lead
        local l = gm.point_distance(0, 0, nx, ny)
        if l > 0.001 then
          nx = nx / l
          ny = ny / l
        end
        s.vx = s.vx + nx * f
        s.vy = s.vy + ny * f
        s.spin = s.spin + (ox * ny - oy * nx) * f * spin
        s.spin = s.spin + gm.random_range(-1, 1) * f * wobble
        stones_wake(self, other, s)
      end
      i = i + 1
    end
    c = c + 1
  end
end

-- gml_Script_stones_separate @ 0x140224fd0
function stones_separate(self, other, chunk, s)
  local k = global.stone_kinds[s.kind]
  local list = chunk.stones
  local pass = wld(self, other, "stone_pass_on")
  local spin = wld(self, other, "stone_shove_spin")
  local i = 0
  while i < (gm.is_array(list) and gm.array_length(list) or 0) do
    local o = list[i]
    if o == s then
      -- continue
    else
      local dx = o.x - s.x
      local dy = o.y - s.y
      local min_d = global.stone_kinds[o.kind].body + k.body
      local d = gm.point_distance(0, 0, dx, dy)
      if d >= min_d or d <= 0 then
        -- continue
      else
        local push = (min_d - d) / d
        o.vx = o.vx + dx * push * pass
        o.vy = o.vy + dy * push * pass
        o.spin = o.spin + push * (dx * s.vy - dy * s.vx) * spin
        s.vx = s.vx - dx * push * pass
        s.vy = s.vy - dy * push * pass
        stones_wake(self, other, o)
      end
    end
    i = i + 1
  end
end

-- gml_Script_stones_step @ 0x1402269c0
function stones_step(self, other)
  if not stones_ready(self, other) then
    return
  end
  local w = global.world
  local moving = global.stones_moving
  local drag = wld(self, other, "stone_drag")
  local ylo = w.surface_y + w.tile * 0.5
  local yhi = w.surface_y + w.water_row * w.tile
  local i = (gm.is_array(moving) and gm.array_length(moving) or 0) - 1
  while i >= 0 do
    local s = moving[i]
    s.vx = s.vx * drag
    s.vy = s.vy * drag
    s.spin = s.spin * drag
    if gm.point_distance(0, 0, s.vx, s.vy) < 0.04 and gm.abs(s.spin) < 0.12 then
      s.vx = 0
      s.vy = 0
      s.spin = 0
      s.moving = false
      gm.array_delete(moving, i, 1)
    else
      s.x = s.x + s.vx
      s.y = s.y + s.vy
      s.angle = s.angle + s.spin
      local lo = s.cx * w.chunk_w + 4
      local hi = lo + w.chunk_w - 8
      if s.x < lo or s.x > hi then
        s.x = gm.clamp(s.x, lo, hi)
        s.vx = 0
      end
      if s.y < ylo or s.y > yhi then
        s.y = gm.clamp(s.y, ylo, yhi)
        s.vy = 0
      end
      stones_separate(self, other, world_get_chunk(self, other, s.cx), s)
    end
    i = i - 1
  end
end

-- gml_Script____struct___215__world_paint_stones__scr_stones @ 0x140228670
function ___struct___215__world_paint_stones__scr_stones(self, other, cx, kind, x, y, side)
  self.cx = cx
  self.kind = kind
  self.x = x
  self.y = y
  self.angle = 0
  self.side = side
  self.vx = 0
  self.vy = 0
  self.spin = 0
  self.moving = false
end

-- gml_Script_world_paint_stones @ 0x140228ae0
function world_paint_stones(self, other, chunk, layer)
  local w = global.world
  local key = world_layer_key(self, other, chunk.cx, layer)
  local x0 = chunk.cx * w.chunk_w
  local spacing = gm.max(48, wld(self, other, "stone_spacing_px"))
  local band = w.layer_rows * w.tile
  local y0 = w.surface_y + layer * band
  local nx = gm.max(1, gm.round(w.chunk_w / spacing))
  local ny = gm.max(1, gm.round(band / spacing))
  local cw = w.chunk_w / nx
  local ch = band / ny
  local chance = wld(self, other, "stone_chance")
  local tx0 = gm.floor(w.start_x / w.tile)
  local j = 0
  while j < ny do
    local i = 0
    while i < nx do
      local idx = j * nx + i
      if world_hash01(self, other, key, idx, 61) < chance then
        local px = x0 + (i + 0.15 + world_hash01(self, other, key, idx, 62) * 0.7) * cw
        local py = y0 + (j + 0.15 + world_hash01(self, other, key, idx, 63) * 0.7) * ch
        local tx = gm.floor(px / w.tile)
        local ty = gm.floor((py - w.surface_y) / w.tile)
        if ty < 1 then
          -- continue
        elseif ty <= 5 and gm.abs(tx - tx0) <= 3 then
          -- continue
        elseif world_chunk_get(self, other, chunk, tx, ty) == 3 then
          -- continue
        elseif gm.bool(world_cave_hits(self, other, px, py, px, py, 24)) then
          -- continue
        elseif gm.bool(world_lake_covers(self, other, chunk, px, py)) then
          -- continue
        elseif gm.bool(world_stone_covers(self, other, chunk, px, py)) then
          -- continue
        else
          local list = chunk.stones
          local kind = gm.min(6, gm.floor(world_hash01(self, other, key, idx, 64) * 7))
          local side = (world_hash01(self, other, key, idx, 65) >= 0.5) and 1 or -1
          gm.array_push(list, gm.new(___struct___215__world_paint_stones__scr_stones, chunk.cx, kind, px, py, side))
        end
      end
      i = i + 1
    end
    j = j + 1
  end
end

-- gml_Script_world_draw_stones @ 0x14022bc80
function world_draw_stones(self, other, chunk, vx, vy, vw, vh)
  local list = chunk.stones
  local pad = global.stone_reach + 2
  local i = 0
  while i < (gm.is_array(list) and gm.array_length(list) or 0) do
    local s = list[i]
    if s.x < vx - pad or s.x > vx + vw + pad then
      -- continue
    elseif s.y < vy - pad or s.y > vy + vh + pad then
      -- continue
    else
      local k = global.stone_kinds[s.kind]
      local bottom = s.y + k.radius
      local col
      if bottom <= world_deep_y(self, other) then
        col = 16777215
      else
        col = vis(self, other, "deep_stone_colour")
      end
      local spr
      if bottom <= world_dense_y(self, other) then
        spr = k.sprite
      else
        spr = k.sprite_grey
      end
      gm.draw_sprite_ext(spr, 0, gm.floor(s.x), gm.floor(s.y), 1, 1, s.angle, col, 1)
    end
    i = i + 1
  end
end

-- gml_Script_stone_role_tiles @ 0x14022d000
local stone_role_tiles_static = nil
function stone_role_tiles(self, other, up, down, left, right, depth)
  if stone_role_tiles_static == nil then
    local st = {}
    st.fill = gm.array(gm.array(3, 3))
    st.up = gm.array(gm.array(gm.array(2, 0), gm.array(3, 0), gm.array(4, 0)), gm.array(gm.array(2, 1), gm.array(3, 1), gm.array(4, 1)))
    st.down = gm.array(gm.array(gm.array(2, 6), gm.array(3, 6), gm.array(4, 6)), gm.array(gm.array(2, 5), gm.array(3, 5), gm.array(4, 5)))
    st.left = gm.array(gm.array(1, 2), gm.array(1, 3), gm.array(1, 4))
    st.right = gm.array(gm.array(5, 2), gm.array(5, 3), gm.array(5, 4))
    st.up_left = gm.array(gm.array(gm.array(0, 0), gm.array(1, 0)), gm.array(gm.array(0, 1), gm.array(1, 1)))
    st.up_right = gm.array(gm.array(gm.array(5, 0), gm.array(6, 0)), gm.array(gm.array(5, 1), gm.array(6, 1)))
    st.down_left = gm.array(gm.array(gm.array(0, 6), gm.array(1, 6)), gm.array(gm.array(0, 5), gm.array(1, 5)))
    st.down_right = gm.array(gm.array(gm.array(5, 6), gm.array(6, 6)), gm.array(gm.array(5, 5), gm.array(6, 5)))
    stone_role_tiles_static = st
  end
  local st = stone_role_tiles_static
  local d = gm.clamp(depth, 0, 1)
  local di = math.floor(d)
  if gm.bool(up) and gm.bool(left) then
    return st.up_left[di]
  end
  if gm.bool(up) and gm.bool(right) then
    return st.up_right[di]
  end
  if gm.bool(down) and gm.bool(left) then
    return st.down_left[di]
  end
  if gm.bool(down) and gm.bool(right) then
    return st.down_right[di]
  end
  if gm.bool(up) then
    return st.up[di]
  end
  if gm.bool(down) then
    return st.down[di]
  end
  if gm.bool(left) then
    return st.left
  end
  if gm.bool(right) then
    return st.right
  end
  return st.fill
end

-- gml_Script_stone_inner_corner_tile @ 0x14022e8f0
local stone_inner_corner_tile_static = nil
function stone_inner_corner_tile(self, other, up, left)
  if stone_inner_corner_tile_static == nil then
    stone_inner_corner_tile_static = {
      up_left = gm.array(2, 2),
      up_right = gm.array(4, 2),
      down_left = gm.array(2, 4),
      down_right = gm.array(4, 4),
    }
  end
  local st = stone_inner_corner_tile_static
  if gm.bool(up) then
    return gm.bool(left) and st.up_left or st.up_right
  end
  return gm.bool(left) and st.down_left or st.down_right
end

-- gml_Script_stone_tiles_init @ 0x14022eee0
function stone_tiles_init(self, other)
  if stone_tiles_ready(self, other) then
    return
  end
  local tw = gm.sprite_get_width(stone_tile)
  local th = gm.sprite_get_height(stone_tile)
  local surf = gm.surface_create(tw, th)
  gm.surface_set_target(surf)
  gm.draw_clear_alpha(0, 0)
  gm.gpu_set_blendmode_ext(gm.bm_one, gm.bm_inv_src_alpha)
  gm.draw_sprite(stone_tile, 0, gm.sprite_get_xoffset(stone_tile), gm.sprite_get_yoffset(stone_tile))
  gm.gpu_set_blendmode(0)
  gm.surface_reset_target()
  local buf = gm.buffer_create(tw * th * 4, 0, 1)
  gm.buffer_get_surface(buf, surf, 0)
  gm.surface_free(surf)
  local tiles = gm.array()
  local ty = 0
  while ty < 7 do
    local tx = 0
    while tx < 7 do
      local x0 = tx * 64
      local y0 = ty * 64
      local rows = gm.array_create(48)
      local yy = 0
      while yy < 48 do
        local row = gm.array()
        local start = -1
        local py = y0 + yy
        local xx = 0
        while xx <= 48 do
          local solid = false
          if xx < 48 and x0 + xx < tw and py < th then
            solid = gm.buffer_peek(buf, (py * tw + x0 + xx) * 4 + 3, 1) >= 128
          end
          if gm.bool(solid) and start < 0 then
            start = xx
          elseif not gm.bool(solid) and start >= 0 then
            gm.array_push(row, start, xx - 1)
            start = -1
          end
          xx = xx + 1
        end
        rows[yy] = row
        yy = yy + 1
      end
      gm.array_push(tiles, rows)
      tx = tx + 1
    end
    ty = ty + 1
  end
  gm.buffer_delete(buf)
  global.stone_tiles = tiles
end

-- gml_Script_stone_tiles_ready @ 0x140231100
function stone_tiles_ready(self, other)
  local r = false
  if gm.variable_global_exists("stone_tiles") then
    r = gm.is_array(global.stone_tiles)
  end
  return r
end

-- gml_Script_stone_tile_solid_at @ 0x1402312a0
function stone_tile_solid_at(self, other, tx, ty, x, y)
  if y < 0 or y >= 48 then
    return false
  end
  local row = global.stone_tiles[ty * 7 + tx][gm.floor(y)]
  local i = 0
  while i < gm.array_length(row) do
    if x >= row[i] and x <= row[i + 1] then
      return true
    end
    i = i + 2
  end
  return false
end

-- gml_Script_stone_cell_key @ 0x1402318d0
function stone_cell_key(self, other, cx, cy)
  return gm.string(cx) .. ":" .. gm.string(cy)
end

-- gml_Script_stone_cell_at @ 0x140231c40
function stone_cell_at(self, other, chunk, x, y)
  local cells = chunk.stone_cells
  if not (gm.is_array(cells) and gm.array_length(cells) ~= 0) then
    return gm.undefined
  end
  return gm.variable_struct_get(chunk.stone_map, stone_cell_key(self, other, gm.floor(x / 48), gm.floor(y / 48)))
end

-- gml_Script_stone_solid_at @ 0x1402322c0
function stone_solid_at(self, other, chunk, x, y)
  local c = stone_cell_at(self, other, chunk, x, y)
  if gm.is_undefined(c) then
    return false
  end
  if not stone_tiles_ready(self, other) then
    return true
  end
  local lx = gm.floor(x) - c.col * 48
  local ly = gm.floor(y) - c.row * 48
  return stone_tile_solid_at(self, other, c.tc, c.tr, lx, ly)
end

-- gml_Script_world_stone_covers @ 0x140232c40
function world_stone_covers(self, other, chunk, x, y)
  return not gm.is_undefined(stone_cell_at(self, other, chunk, x, y))
end

-- gml_Script_stone_blob_cells @ 0x140232f00
function stone_blob_cells(self, other, cx, cy, rx, ry, count)
  local grid = gm.struct({})
  local x0 = cx - rx - 1
  local x1 = cx + rx + 1
  local y0 = cy - ry - 1
  local y1 = cy + ry + 1
  local b = 0
  while b < count do
    local seed = cy * 8 + b
    local brx = gm.max(1, gm.round(rx * (0.3 + 0.55 * world_hash01(self, other, cx, seed, 56))))
    local bry = gm.max(1, gm.round(ry * (0.3 + 0.55 * world_hash01(self, other, cx, seed, 57))))
    local ox = gm.round((world_hash01(self, other, cx, seed, 58) * 2 - 1) * (rx - brx))
    local oy = gm.round((world_hash01(self, other, cx, seed, 59) * 2 - 1) * (ry - bry))
    local dy = -bry
    while dy <= bry do
      local dx = -brx
      while dx <= brx do
        local nx = dx / (brx + 0.5)
        local ny = dy / (bry + 0.5)
        if nx * nx + ny * ny > 1 then
          -- continue
        else
          gm.variable_struct_set(grid, stone_cell_key(self, other, cx + ox + dx, cy + oy + dy), true)
        end
        dx = dx + 1
      end
      dy = dy + 1
    end
    b = b + 1
  end
  local nibble = wld(self, other, "stone_deposit_nibble_chance")
  local grow = wld(self, other, "stone_deposit_grow_chance")
  local drop = gm.array()
  local add = gm.array()
  local y = y0
  while y <= y1 do
    local x = x0
    while x <= x1 do
      local key = stone_cell_key(self, other, x, y)
      local here = not gm.is_undefined(gm.variable_struct_get(grid, key))
      local up = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x, y - 1)))
      local down = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x, y + 1)))
      local left = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x - 1, y)))
      local right = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x + 1, y)))
      if here then
        if not (up and down and left and right) and world_hash01(self, other, x, y, 62) < nibble then
          gm.array_push(drop, key)
        end
      else
        if gm.abs(x - cx) <= rx and gm.abs(y - cy) <= ry then
          if (up or down or left or right) and world_hash01(self, other, x, y, 63) < grow then
            gm.array_push(add, key)
          end
        end
      end
      x = x + 1
    end
    y = y + 1
  end
  local i = 0
  while i < gm.array_length(drop) do
    gm.variable_struct_remove(grid, drop[i])
    i = i + 1
  end
  i = 0
  while i < gm.array_length(add) do
    gm.variable_struct_set(grid, add[i], true)
    i = i + 1
  end
  return stone_blob_finish(self, other, grid, x0, x1, y0, y1, gm.undefined)
end

-- gml_Script____struct___217__stone_blob_finish__scr_stones @ 0x140237280
function ___struct___217__stone_blob_finish__scr_stones(self, other, col, row)
  self.col = col
  self.row = row
end

-- gml_Script____struct___219__stone_blob_finish__scr_stones @ 0x140237440
function ___struct___219__stone_blob_finish__scr_stones(self, other, keys, list)
  self.keys = keys
  self.list = list
end

-- gml_Script_stone_blob_finish @ 0x140237600
function stone_blob_finish(self, other, grid, x0, x1, y0, y1, protect)
  if gm.is_undefined(protect) then
    protect = 0
  end
  local list = gm.array()
  local changed = true
  while changed do
    changed = false
    local y = y0
    while y <= y1 do
      local x = x0
      while x <= x1 do
        local key = stone_cell_key(self, other, x, y)
        local here = not gm.is_undefined(gm.variable_struct_get(grid, key))
        local up = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x, y - 1)))
        local down = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x, y + 1)))
        local left = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x - 1, y)))
        local right = not gm.is_undefined(gm.variable_struct_get(grid, stone_cell_key(self, other, x + 1, y)))
        if not here then
          if up and down and left and right then
            gm.variable_struct_set(grid, key, true)
            changed = true
          end
        else
          if (not left and not right) or (not up and not down) then
            gm.variable_struct_remove(grid, key)
            changed = true
          end
        end
        x = x + 1
      end
      y = y + 1
    end
  end
  local pieces = stone_blob_largest_piece(self, other, grid, x0, x1, y0, y1, protect)
  local i = 0
  while i < gm.array_length(pieces) do
    local p = pieces[i]
    gm.array_push(list, gm.new(___struct___217__stone_blob_finish__scr_stones, p[0], p[1]))
    i = i + 1
  end
  local keys = gm.struct({})
  i = 0
  while i < gm.array_length(list) do
    gm.variable_struct_set(keys, stone_cell_key(self, other, list[i].col, list[i].row), true)
    i = i + 1
  end
  return gm.new(___struct___219__stone_blob_finish__scr_stones, keys, list)
end

-- gml_Script_stone_blob_largest_piece @ 0x140239c30
function stone_blob_largest_piece(self, other, grid, x0, x1, y0, y1, protect)
  if gm.is_undefined(protect) then
    protect = 0
  end
  local seen = gm.struct({})
  local best = gm.array()
  local y = y0
  while y <= y1 do
    local x = x0
    while x <= x1 do
      local key = stone_cell_key(self, other, x, y)
      if gm.is_undefined(gm.variable_struct_get(grid, key)) or not gm.is_undefined(gm.variable_struct_get(seen, key)) then
        -- continue
      else
        local piece = gm.array()
        local stack = gm.array(gm.array(x, y))
        gm.variable_struct_set(seen, key, true)
        while gm.is_array(stack) and gm.array_length(stack) > 0 do
          local c = gm.array_pop(stack)
          gm.array_push(piece, c)
          local dirs = gm.array(gm.array(1, 0), gm.array(-1, 0), gm.array(0, 1), gm.array(0, -1))
          local d = 0
          while d < 4 do
            local nx = c[0] + dirs[d][0]
            local ny = c[1] + dirs[d][1]
            local nk = stone_cell_key(self, other, nx, ny)
            if gm.is_undefined(gm.variable_struct_get(grid, nk)) or not gm.is_undefined(gm.variable_struct_get(seen, nk)) then
              -- continue
            else
              gm.variable_struct_set(seen, nk, true)
              gm.array_push(stack, gm.array(nx, ny))
            end
            d = d + 1
          end
        end
        if protect <= 0 then
          if gm.array_length(piece) > gm.array_length(best) then
            best = piece
          end
        else
          if gm.array_length(piece) >= protect then
            local i = 0
            while i < gm.array_length(piece) do
              gm.array_push(best, piece[i])
              i = i + 1
            end
          end
        end
      end
      x = x + 1
    end
    y = y + 1
  end
  return best
end

-- gml_Script_stone_blob_fits @ 0x14023bae0
function stone_blob_fits(self, other, chunk, blob)
  local w = global.world
  local clear = wld(self, other, "stone_water_clearance_px")
  local gap = wld(self, other, "stone_deposit_gap_cells") * 48
  local tx0 = w.start_x - wld(self, other, "start_tree_clear_px")
  local tx1 = w.start_x + wld(self, other, "start_tree_clear_px")
  local ty1 = w.surface_y + 6 * w.tile
  local i = 0
  while i < (gm.is_array(blob.list) and gm.array_length(blob.list) or 0) do
    local c = blob.list[i]
    local x0 = c.col * 48
    local y0 = c.row * 48
    local x1 = x0 + 48
    local y1 = y0 + 48
    if x1 > tx0 and x0 < tx1 and y0 < ty1 then
      return false
    end
    if gm.bool(world_cave_hits(self, other, x0, y0, x1, y1, gm.undefined)) then
      return false
    end
    local j = 0
    while j < (gm.is_array(chunk.lakes) and gm.array_length(chunk.lakes) or 0) do
      local l = chunk.lakes[j]
      if x1 + clear > l.body_x and x0 - clear < l.body_x + l.body_w and y1 + clear > l.body_y and y0 - clear < l.body_y + l.body_h then
        return false
      end
      j = j + 1
    end
    local g = gm.floor(gap / 48)
    local dy = -g
    while dy <= g do
      local dx = -g
      while dx <= g do
        if not gm.is_undefined(gm.variable_struct_get(chunk.stone_map, stone_cell_key(self, other, c.col + dx, c.row + dy))) then
          return false
        end
        dx = dx + 1
      end
      dy = dy + 1
    end
    i = i + 1
  end
  return true
end

-- gml_Script_stone_pick_tile @ 0x14023de90
function stone_pick_tile(self, other, a, b, list, salt)
  local n = gm.is_array(list) and gm.array_length(list) or 0
  local i = gm.min(n - 1, gm.floor(world_hash01(self, other, a, b, salt) * n))
  return list[i]
end

-- gml_Script_stone_tile_for @ 0x14023e370
function stone_tile_for(self, other, has, col, row, depth)
  local up = not gm.bool(has(self, other, col, row - 1))
  local down = not gm.bool(has(self, other, col, row + 1))
  local left = not gm.bool(has(self, other, col - 1, row))
  local right = not gm.bool(has(self, other, col + 1, row))
  local tiles = stone_role_tiles(self, other, up, down, left, right, depth)
  local tile = stone_pick_tile(self, other, col, row, tiles, 41)
  if not up and not down and not left and not right then
    local corners = gm.array()
    if not gm.bool(has(self, other, col - 1, row - 1)) then
      gm.array_push(corners, gm.array(true, true))
    end
    if not gm.bool(has(self, other, col + 1, row - 1)) then
      gm.array_push(corners, gm.array(true, false))
    end
    if not gm.bool(has(self, other, col - 1, row + 1)) then
      gm.array_push(corners, gm.array(false, true))
    end
    if not gm.bool(has(self, other, col + 1, row + 1)) then
      gm.array_push(corners, gm.array(false, false))
    end
    local n = gm.array_length(corners)
    if n > 0 then
      local idx = gm.min(n - 1, gm.floor(world_hash01(self, other, col, row, 42) * n))
      local c = corners[idx]
      tile = stone_inner_corner_tile(self, other, c[0], c[1])
    end
  end
  return tile
end

-- gml_Script____struct___221__stone_redress_after_cut__scr_stones @ 0x140240360
function ___struct___221__stone_redress_after_cut__scr_stones(self, other, chunk, dropped, world)
  self.chunk = chunk
  self.dropped = dropped
  self.world = world
end

-- gml_Script_anon__29665__stone_redress_after_cut__scr_stones @ 0x140240590
function anon__29665__stone_redress_after_cut__scr_stones(self, other, col, row)
  local key = stone_cell_key(self, other, col, row)
  if not gm.is_undefined(gm.variable_struct_get(self.dropped, key)) then
    return false
  end
  if not gm.is_undefined(gm.variable_struct_get(self.chunk.stone_map, key)) then
    return true
  end
  local c = gm.variable_struct_get(self.world.chunks, world_chunk_key(self, other, gm.floor(col * 48 / self.world.chunk_w)))
  local r = false
  if not gm.is_undefined(c) then
    if c ~= self.chunk then
      r = not gm.is_undefined(gm.variable_struct_get(c.stone_map, key))
    end
  end
  return r
end

-- gml_Script____struct___222__stone_redress_after_cut__scr_stones @ 0x140241110
function ___struct___222__stone_redress_after_cut__scr_stones(self, other, dropped)
  self.dropped = dropped
end

-- gml_Script_anon__31443__stone_redress_after_cut__scr_stones @ 0x140241270
function anon__31443__stone_redress_after_cut__scr_stones(self, other, c)
  return gm.is_undefined(gm.variable_struct_get(self.dropped, stone_cell_key(self, other, c.col, c.row)))
end
