local gm = require("gm")

-- gml_Script_tree_seed @ 0x1401f4fc0
function tree_seed(self, other, s)
  local h = 0
  local i = 1
  while i <= gm.string_length(s) do
    h = gm.mod(h * 31 + gm.ord(gm.string_char_at(s, i)), 99991)
    i = i + 1
  end
  return h * 0.013
end

-- gml_Script____struct___197__tree_strand_point__scr_skilltree @ 0x1401f5640
function ___struct___197__tree_strand_point__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_tree_strand_point @ 0x1401f5840
function tree_strand_point(self, other, x0, y0, x1, y1, seed, amp, t)
  local span = y1 - y0
  local cy1 = y0 + span * 0.58
  local cy2 = y1 - span * 0.48
  local u = 1 - t
  -- x: cubic bezier with control points x0, x0, x1, x1
  local bx0 = u * u * u * x0
  local bx1 = 3 * u * u * t * x0
  local bx2 = 3 * u * t * t * x1
  local bx3 = t * t * t * x1
  local px = bx0 + bx1
  px = px + bx2
  px = px + bx3
  -- y: cubic bezier with control points y0, cy1, cy2, y1
  local by0 = u * u * u * y0
  local by1 = 3 * u * u * t * cy1
  local by2 = 3 * u * t * t * cy2
  local by3 = t * t * t * y1
  local py = by0 + by1
  py = py + by2
  py = py + by3
  local a1 = t * math.pi * 2.3 + seed * 0.7
  local a2 = t * math.pi * 5.7 + seed * 1.9
  local wave = math.sin(a2) * 0.35 + math.sin(a1) * 0.65
  local env = math.sin(t * math.pi)
  px = px + wave * env * amp
  return gm.new(___struct___197__tree_strand_point__scr_skilltree, px, py)
end

-- gml_Script_tree_noise @ 0x1401f74b0
function tree_noise(self, other, a, b)
  local n = math.sin(a * 12.9898 + b * 78.233) * 43758.5453
  return (n - gm.floor(n)) * 2 - 1
end

-- gml_Script____struct___198__tree_strand_path__scr_skilltree @ 0x1401f79d0
function ___struct___198__tree_strand_path__scr_skilltree(self, other, x, y, t)
  self.x = x
  self.y = y
  self.t = t
end

-- gml_Script_tree_strand_path @ 0x1401f7c30
function tree_strand_path(self, other, x0, y0, x1, y1, seed, amp, wob, tmax)
  local steps = 30
  local pts = gm.array()
  local drift = 0
  local i = 0
  while i <= steps do
    local t = gm.min(i / steps, tmax)
    local p = tree_strand_point(self, other, x0, y0, x1, y1, seed, amp, t)
    local t2 = gm.min(1, t + 0.02)
    local q = tree_strand_point(self, other, x0, y0, x1, y1, seed, amp, t2)
    local dir = gm.point_direction(p.x, p.y, q.x, q.y) + 90
    local nz = tree_noise(self, other, seed, i)
    drift = drift * 0.6 + nz * 0.4
    local wmax = gm.max(2, wob * 1.4)
    local env = math.sin(t * math.pi)
    local off = drift * wmax * env
    local nx = p.x + gm.lengthdir_x(off, dir)
    local ny = p.y + gm.lengthdir_y(off, dir)
    gm.array_push(pts, gm.new(___struct___198__tree_strand_path__scr_skilltree, nx, ny, t))
    if t >= tmax then
      break
    end
    i = i + 1
  end
  return pts
end

-- gml_Script_tree_draw_hair @ 0x1401f96f0
function tree_draw_hair(self, other, x, y, dir, len, w, seed, branch)
  local segs = 4
  local cx = x
  local cy = y
  local i = 1
  while i <= segs do
    dir = dir + tree_noise(self, other, seed, i * 3.1) * 28
    local nx = cx + gm.lengthdir_x(len / segs, dir)
    local ny = cy + gm.lengthdir_y(len / segs, dir)
    gm.draw_line_width(cx, cy, nx, ny, gm.max(1, w * (1 - i / (segs + 1))))
    if gm.bool(branch) and i == 2 then
      local side
      if tree_noise(self, other, seed, 9) > 0 then
        side = 38
      else
        side = -38
      end
      tree_draw_hair(self, other, nx, ny, dir + side, len * 0.5, w * 0.6, seed + 5.3, false)
    end
    cx = nx
    cy = ny
    i = i + 1
  end
end

-- gml_Script_tree_draw_strand @ 0x1401fabc0
function tree_draw_strand(self, other, x0, y0, x1, y1, seed, amp, w, col, alpha, tmax, hairs)
  if tmax <= 0 then
    return
  end
  local pts = tree_strand_path(self, other, x0, y0, x1, y1, seed, amp, w, tmax)
  local n = gm.array_length(pts)
  gm.draw_set_color(col)
  gm.draw_set_alpha(alpha)
  local i = 1
  while i < n do
    local a = pts[i - 1]
    local b = pts[i]
    local ww = gm.max(1, w * gm.lerp(1, 0.4, b.t))
    gm.draw_line_width(a.x, a.y, b.x, b.y, ww)
    if ww > 3 then
      gm.draw_circle(b.x, b.y, ww * 0.5, false)
    end
    i = i + 1
  end
  if hairs > 0 then
    local count = 7
    local j = 0
    while j < count do
      local ht = 0.12 + j * (0.78 / (count - 1)) + tree_noise(self, other, seed, j + 40) * 0.04
      local idx = gm.clamp(gm.round(ht * 30), 0, 29)
      if idx >= n - 1 then
        break
      end
      local a = pts[idx]
      local b = pts[idx + 1]
      local side
      if tree_noise(self, other, seed, j + 60) > 0 then
        side = 1
      else
        side = -1
      end
      local hdir = gm.point_direction(a.x, a.y, b.x, b.y) + side * (55 + tree_noise(self, other, seed, j + 80) * 20)
      local hlen = hairs * ((math.abs(tree_noise(self, other, seed, j + 100)) + 1) * 0.5)
      tree_draw_hair(self, other, a.x, a.y, hdir, hlen, w * 0.35, seed + j * 1.7,
        tree_noise(self, other, seed, j + 120) > 0.2)
      j = j + 1
    end
  end
  -- TODO(port): draw_set_alpha argument lost in decompilation (passed in XMM0), assumed 1
  gm.draw_set_alpha(1)
end

-- gml_Script_tree_draw_glow @ 0x1401fdb40
function tree_draw_glow(self, other, x, y, r, col, alpha)
  gm.draw_set_color(col)
  local k = 3
  while k >= 1 do
    gm.draw_set_alpha(alpha * (0.12 * k))
    gm.draw_circle(x, y, r * (1 + k * 0.22), false)
    k = k - 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script_tree_draw_level_ring @ 0x1401fe150
function tree_draw_level_ring(self, other, x, y, r, lvl, maxlvl, col_on, col_off, w)
  if maxlvl <= 1 then
    return
  end
  local arc = 260
  local start = 90 + arc * 0.5
  local step = arc / maxlvl
  local i = 0
  while i < maxlvl do
    local a0 = start - i * step - step * 0.12
    local a1 = start - (i + 1) * step + step * 0.12
    if i >= lvl then
      gm.draw_set_color(col_off)
    else
      gm.draw_set_color(col_on)
    end
    if i >= lvl then
      gm.draw_set_alpha(0.35)
    else
      gm.draw_set_alpha(1)
    end
    gm.draw_line_width(x + gm.lengthdir_x(r, a0), y + gm.lengthdir_y(r, a0),
      x + gm.lengthdir_x(r, a1), y + gm.lengthdir_y(r, a1), gm.max(1, w))
    i = i + 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script____struct___199__tree_icon_box__scr_skilltree @ 0x1401ff390
function ___struct___199__tree_icon_box__scr_skilltree(self, other, sporeshroom, predatorshroom, explosiveshroom,
                                                         diggershroom, stinkyshroom, experienceshroom, consumptorshroom)
  self.sporeshroom = sporeshroom
  self.predatorshroom = predatorshroom
  self.explosiveshroom = explosiveshroom
  self.diggershroom = diggershroom
  self.stinkyshroom = stinkyshroom
  self.experienceshroom = experienceshroom
  self.consumptorshroom = consumptorshroom
end

-- gml_Script_tree_icon_box @ 0x1401ff770
local tree_icon_box_static = {}
function tree_icon_box(self, other, spr)
  if tree_icon_box_static._boxes == nil then
    tree_icon_box_static._boxes = gm.new(___struct___199__tree_icon_box__scr_skilltree,
      gm.array(4.0, 15.0, 41.0, 63.0),
      gm.array(19.0, 8.0, 43.0, 59.0),
      gm.array(10.0, 18.0, 38.0, 47.0),
      gm.array(2.0, 11.0, 60.0, 63.0),
      gm.array(10.0, 26.0, 37.0, 56.0),
      gm.array(4.0, 25.0, 40.0, 63.0),
      gm.array(1.0, 3.0, 46.0, 43.0))
  end
  return gm.variable_struct_get(tree_icon_box_static._boxes, gm.sprite_get_name(spr))
end

-- gml_Script_tree_icon_scale @ 0x1402000d0
function tree_icon_scale(self, other, spr)
  if spr == diggershroom then
    return vis(self, other, "tree_digger_icon_scale")
  else
    return 1.0
  end
end

-- gml_Script_tree_draw_icon @ 0x140200350
function tree_draw_icon(self, other, spr, x, y, size, alpha, cropped, fit)
  if gm.is_undefined(cropped) then
    cropped = false
  end
  if gm.is_undefined(fit) then
    fit = 1.0
  end
  if spr < 0 or not gm.bool(gm.sprite_exists(spr)) then
    return
  end
  local box
  if gm.bool(cropped) then
    box = tree_icon_box(self, other, spr)
  else
    box = gm.undefined
  end
  if gm.is_undefined(box) then
    local cells = shroom_sheet_cells(self, other, spr)
    local w = gm.div(gm.sprite_get_width(spr), cells)
    local h = gm.sprite_get_height(spr)
    local sc = size / gm.max(1.0, gm.max(w, h))
    if cells <= 1 then
      local dx = x + (gm.sprite_get_xoffset(spr) - w * 0.5) * sc
      local dy = y + (gm.sprite_get_yoffset(spr) - h * 0.5) * sc
      gm.draw_sprite_ext(spr, 0, dx, dy, sc, sc, 0, 0xffffff, alpha)
      return
    end
    gm.draw_sprite_part_ext(spr, 0, 0, 0, w, h, x - w * 0.5 * sc, y - h * 0.5 * sc, sc, sc, 16777215.0, alpha)
    return
  end
  local bw = box[2] - box[0] + 1
  local bh = box[3] - box[1] + 1
  local sc
  if gm.bool(fit) then
    sc = size * tree_icon_scale(self, other, spr) / gm.max(bw, bh)
  else
    sc = size / gm.max(1.0, gm.max(mushroom_cell_width(self, other, spr), gm.sprite_get_height(spr)))
  end
  gm.draw_sprite_part_ext(spr, 0, box[0], box[1], bw, bh, x - bw * 0.5 * sc, y - bh * 0.5 * sc, sc, sc, 16777215.0, alpha)
end

-- gml_Script_tree_draw_guide @ 0x140203290
function tree_draw_guide(self, other, x0, y0, x1, y1, w, col, alpha, ctrl)
  if ctrl == nil then
    ctrl = gm.undefined
  end
  if alpha <= 0 then
    return
  end
  local segs = 24.0
  local prev = tree_strand_point(self, other, x0, y0, x1, y1, 0, 0, 0)
  gm.draw_set_color(col)
  gm.draw_set_alpha(alpha)
  local i = 1.0
  while i <= segs do
    local p
    if gm.is_undefined(ctrl) then
      p = tree_strand_point(self, other, x0, y0, x1, y1, 0, 0, i / segs)
    else
      p = tree_bend_point(self, other, x0, y0, ctrl.x, ctrl.y, x1, y1, i / segs)
    end
    gm.draw_line_width(prev.x, prev.y, p.x, p.y, w)
    prev = p
    i = i + 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script____struct___200__tree_bend_point__scr_skilltree @ 0x140204920
function ___struct___200__tree_bend_point__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_tree_bend_point @ 0x140204b20
function tree_bend_point(self, other, x0, y0, cx, cy, x1, y1, t)
  local u = 1 - t
  local bx0 = u * u * x0
  local bx1 = 2 * u * t * cx
  local bx2 = t * t * x1
  local px = bx0 + bx1
  px = px + bx2
  local by0 = u * u * y0
  local by1 = 2 * u * t * cy
  local by2 = t * t * y1
  local py = by0 + by1
  py = py + by2
  return gm.new(___struct___200__tree_bend_point__scr_skilltree, px, py)
end

-- gml_Script____struct___201__tree_hypha_walk__scr_skilltree @ 0x1402056a0
function ___struct___201__tree_hypha_walk__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___202__tree_hypha_walk__scr_skilltree @ 0x1402058a0
function ___struct___202__tree_hypha_walk__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script____struct___203__tree_hypha_walk__scr_skilltree @ 0x140205aa0
function ___struct___203__tree_hypha_walk__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_tree_hypha_walk @ 0x140205ca0
function tree_hypha_walk(self, other, x0, y0, dir, len, steps, amp, seed, tx, ty, ctrl)
  if ctrl == nil then
    ctrl = gm.undefined
  end
  local has_target = not gm.is_undefined(tx)
  local pts = gm.array(gm.new(___struct___201__tree_hypha_walk__scr_skilltree, x0, y0))
  local drift = 0
  if has_target then
    dir = gm.point_direction(x0, y0, tx, ty)
    len = gm.point_distance(x0, y0, tx, ty)
  end
  local i = 1
  while i <= steps do
    local t = i / steps
    local nz = tree_noise(self, other, seed, i * 1.37)
    drift = drift * 0.55 + nz * amp * 0.45
    local env
    if has_target then
      env = math.sin(t * math.pi)
    else
      env = 1.0 - math.abs(t - 0.5)
    end
    local off = drift * env
    if has_target and not gm.is_undefined(ctrl) then
      local p = tree_bend_point(self, other, x0, y0, ctrl.x, ctrl.y, tx, ty, t)
      local t_next = gm.min(1, t + 0.01)
      local pn = tree_bend_point(self, other, x0, y0, ctrl.x, ctrl.y, tx, ty, t_next)
      local t_prev = gm.max(0, t - 0.01)
      local pp = tree_bend_point(self, other, x0, y0, ctrl.x, ctrl.y, tx, ty, t_prev)
      local tang = gm.point_direction(pp.x, pp.y, pn.x, pn.y)
      local nx = p.x + gm.lengthdir_x(off, tang + 90)
      local ny = p.y + gm.lengthdir_y(off, tang + 90)
      gm.array_push(pts, gm.new(___struct___202__tree_hypha_walk__scr_skilltree, nx, ny))
    else
      local dist_t = len * t
      local bx = x0 + gm.lengthdir_x(dist_t, dir)
      local nx = bx + gm.lengthdir_x(off, dir + 90)
      local by = y0 + gm.lengthdir_y(dist_t, dir)
      local ny = by + gm.lengthdir_y(off, dir + 90)
      gm.array_push(pts, gm.new(___struct___203__tree_hypha_walk__scr_skilltree, nx, ny))
    end
    i = i + 1
  end
  return pts
end

-- gml_Script____struct___204__tree_hypha_cells__scr_skilltree @ 0x140208ed0
function ___struct___204__tree_hypha_cells__scr_skilltree(self, other, x, y, w, d)
  self.x = x
  self.y = y
  self.w = w
  self.d = d
end

-- gml_Script_tree_hypha_cells @ 0x140209180
function tree_hypha_cells(self, other, pts, w0)
  local n = gm.array_length(pts)
  local cells = gm.array()
  local total = 0
  local i = 1
  while i < n do
    total = total + gm.point_distance(pts[i - 1].x, pts[i - 1].y, pts[i].x, pts[i].y)
    i = i + 1
  end
  if total <= 0 then
    return cells
  end
  local acc = 0
  local lx = -999999.0
  local ly = -999999.0
  i = 1
  while i < n do
    local a = pts[i - 1]
    local b = pts[i]
    local seg = gm.point_distance(a.x, a.y, b.x, b.y)
    if seg > 0 then
      local d = 0
      while d < seg do
        local t = d / seg
        local w = gm.max(1, gm.round(gm.lerp(w0, 1, (acc + d) / total)))
        local cx = gm.floor(gm.lerp(a.x, b.x, t))
        local cy = gm.floor(gm.lerp(a.y, b.y, t))
        if not (cx == lx and cy == ly) then
          gm.array_push(cells, gm.new(___struct___204__tree_hypha_cells__scr_skilltree, cx, cy, w, (acc + d) / total))
          lx = cx
          ly = cy
        end
        d = d + gm.max(1, w - 1)
      end
      acc = acc + seg
    end
    i = i + 1
  end
  return cells
end

-- gml_Script____struct___205__tree_hypha_build__scr_skilltree @ 0x14020b2c0
function ___struct___205__tree_hypha_build__scr_skilltree(self, other, cells)
  self.cells = cells
  self.start = 0
  self.span = 1.0
end

-- gml_Script____struct___206__tree_hypha_build__scr_skilltree @ 0x14020b480
function ___struct___206__tree_hypha_build__scr_skilltree(self, other, cells, start, span)
  self.cells = cells
  self.start = start
  self.span = span
end

-- gml_Script____struct___207__tree_hypha_build__scr_skilltree @ 0x14020b6c0
function ___struct___207__tree_hypha_build__scr_skilltree(self, other, branches, vbuf, total, main_len)
  self.branches = branches
  self.vbuf = vbuf
  self.total = total
  self.main_len = main_len
end

-- gml_Script_tree_hypha_build @ 0x14020b950
function tree_hypha_build(self, other, x0, y0, x1, y1, seed, ctrl)
  if ctrl == nil then
    ctrl = gm.undefined
  end
  local dist = gm.point_distance(x0, y0, x1, y1)
  local dir = gm.point_direction(x0, y0, x1, y1)
  local hw = vis(self, other, "tree_hypha_width")
  local wander = gm.min(dist * vis(self, other, "growth_jitter") * 0.1, vis(self, other, "tree_hypha_wander_max"))
  local steps = 14.0
  local main = tree_hypha_walk(self, other, x0, y0, 0, 0, steps, wander, seed, x1, y1, ctrl)
  local main_cells = tree_hypha_cells(self, other, main, hw)
  local branches = gm.array(gm.new(___struct___205__tree_hypha_build__scr_skilltree, main_cells))
  local nside = bal(self, other, "growth_side_branches")
  local cum = gm.array(0)
  local main_len = 0.0
  local i = 1
  while i < gm.array_length(main) do
    local a = main[i - 1]
    local c = main[i]
    main_len = main_len + gm.point_distance(a.x, a.y, c.x, c.y)
    gm.array_push(cum, main_len)
    i = i + 1
  end
  local total = 1.0
  local b = 0
  while b < nside do
    local n_idx = tree_noise(self, other, seed, 20 + b)
    local idx = 2 + gm.floor(math.abs(n_idx) * (steps - 3))
    local p = main[idx]
    local q = main[idx + 1]
    local side
    if tree_noise(self, other, seed, 40 + b) > 0 then
      side = 1.0
    else
      side = -1.0
    end
    local n_ang = tree_noise(self, other, seed, 60 + b)
    local bdir = gm.point_direction(p.x, p.y, q.x, q.y) + side * (math.abs(n_ang) * 30.0 + 25.0)
    local n_len = tree_noise(self, other, seed, 80 + b)
    local side_max = vis(self, other, "tree_hypha_side_max")
    local blen = gm.min(dist * (math.abs(n_len) * 0.27 + 0.17), side_max)
    local bseed = seed + (b + 1) * 11.3
    local bpts = tree_hypha_walk(self, other, p.x, p.y, bdir, blen, 6.0, wander * 0.69, bseed,
      gm.undefined, gm.undefined, gm.undefined)
    local start = cum[idx] / gm.max(1.0, main_len)
    local span = blen / gm.max(1.0, main_len)
    local bcells = tree_hypha_cells(self, other, bpts, hw * 0.66)
    gm.array_push(branches, gm.new(___struct___206__tree_hypha_build__scr_skilltree, bcells, start, span))
    total = gm.max(total, start + span)
    b = b + 1
  end
  return gm.new(___struct___207__tree_hypha_build__scr_skilltree, branches, -1.0, total, main_len)
end

-- gml_Script____struct___209__tree_hypha_get__scr_skilltree @ 0x14020f820
function ___struct___209__tree_hypha_get__scr_skilltree(self, other, key, hypha)
  self.key = key
  self.hypha = hypha
end

-- gml_Script_tree_hypha_get @ 0x14020f9e0
function tree_hypha_get(self, other, id, x0, y0, x1, y1, ctrl)
  if ctrl == nil then
    ctrl = gm.undefined
  end
  if not gm.bool(gm.variable_global_exists("tree_hyphae")) then
    global.tree_hyphae = gm.struct({})
  end
  local ctrl_key
  if gm.is_undefined(ctrl) then
    ctrl_key = ""
  else
    ctrl_key = ":" .. gm.string(ctrl.x) .. ":" .. gm.string(ctrl.y)
  end
  local key = gm.string(x0) .. ":" .. gm.string(y0) .. ":" .. gm.string(x1) .. ":" .. gm.string(y1) .. ":"
    .. gm.string(vis(self, other, "tree_hypha_width")) .. ctrl_key .. ":v5"
  local e = gm.variable_struct_get(global.tree_hyphae, id)
  if not gm.is_undefined(e) and e.key == key then
    return e.hypha
  end
  if not gm.is_undefined(e) then
    if gm.bool(gm.variable_struct_exists(e.hypha, "vbuf")) then
      if e.hypha.vbuf >= 0 then
        gm.vertex_delete_buffer(e.hypha.vbuf)
      end
    end
  end
  local h = tree_hypha_build(self, other, x0, y0, x1, y1, tree_seed(self, other, id), ctrl)
  gm.variable_struct_set(global.tree_hyphae, id, gm.new(___struct___209__tree_hypha_get__scr_skilltree, key, h))
  return h
end

-- gml_Script_tree_hypha_length @ 0x140211cd0
function tree_hypha_length(self, other, id)
  if gm.bool(gm.variable_global_exists("tree_hyphae")) then
    local e = gm.variable_struct_get(global.tree_hyphae, id)
    if not gm.is_undefined(e) then
      if gm.bool(gm.variable_struct_exists(e.hypha, "main_len")) then
        return e.hypha.main_len * e.hypha.total
      end
    end
  end
  local d = upg_def(self, other, id)
  local par
  if not gm.is_undefined(d) then
    par = upg_def(self, other, d.parent)
  else
    par = gm.undefined
  end
  if gm.is_undefined(par) then
    return 1.0
  end
  local a = upg_tree_pos(self, other, par)
  local b = upg_tree_pos(self, other, d)
  return gm.point_distance(a.x, a.y, b.x, b.y) * 1.3
end

-- gml_Script_tree_hypha_cell_quad @ 0x140212bd0
function tree_hypha_cell_quad(self, other, c, col)
  local h = gm.div(c.w, 2)
  local x0 = c.x - h
  local y0 = c.y - h
  local x1 = x0 + c.w
  local y1 = y0 + c.w
  gm.draw_vertex_color(x0, y0, col, 1.0)
  gm.draw_vertex_color(x1, y0, col, 1.0)
  gm.draw_vertex_color(x0, y1, col, 1.0)
  gm.draw_vertex_color(x1, y0, col, 1.0)
  gm.draw_vertex_color(x1, y1, col, 1.0)
  gm.draw_vertex_color(x0, y1, col, 1.0)
end

-- gml_Script_tree_hypha_draw @ 0x1402137a0
function tree_hypha_draw(self, other, h, x, y, scale, col, prog)
  local old = gm.matrix_get(gm.matrix_world)
  gm.matrix_set(gm.matrix_world, gm.matrix_build(x, y, 0, 0, 0, 0, scale, scale, 1.0))
  if prog >= 1 then
    if gm.bool(gm.variable_global_exists("myc_vformat")) then
      if h.vbuf < 0 then
        local vb = gm.vertex_create_buffer()
        gm.vertex_begin(vb, global.myc_vformat)
        local b = 0
        while b < gm.array_length(h.branches) do
          local cells = h.branches[b].cells
          local k = 0
          while k < gm.array_length(cells) do
            local c = cells[k]
            local hw = gm.div(c.w, 2)
            local x0 = c.x - hw
            local y0 = c.y - hw
            local x1 = x0 + c.w
            local y1 = y0 + c.w
            gm.vertex_position(vb, x0, y0)
            gm.vertex_color(vb, col, 1.0)
            gm.vertex_position(vb, x1, y0)
            gm.vertex_color(vb, col, 1.0)
            gm.vertex_position(vb, x0, y1)
            gm.vertex_color(vb, col, 1.0)
            gm.vertex_position(vb, x1, y0)
            gm.vertex_color(vb, col, 1.0)
            gm.vertex_position(vb, x1, y1)
            gm.vertex_color(vb, col, 1.0)
            gm.vertex_position(vb, x0, y1)
            gm.vertex_color(vb, col, 1.0)
            k = k + 1
          end
          b = b + 1
        end
        gm.vertex_end(vb)
        gm.vertex_freeze(vb)
        h.vbuf = vb
      end
      gm.vertex_submit(h.vbuf, gm.pr_trianglelist, -1)
      gm.matrix_set(gm.matrix_world, old)
      return
    end
  end
  local nv = 0
  local lim = prog * h.total
  gm.draw_primitive_begin(gm.pr_trianglelist)
  local b = 0
  while b < gm.array_length(h.branches) do
    local br = h.branches[b]
    local cells = br.cells
    local frac = gm.clamp((lim - br.start) / gm.max(0.001, br.span), 0, 1.0)
    if frac > 0 then
      local k = 0
      while k < gm.array_length(cells) do
        if cells[k].d > frac then
          break
        end
        tree_hypha_cell_quad(self, other, cells[k], col)
        nv = nv + 6
        if nv >= 600 then
          gm.draw_primitive_end()
          gm.draw_primitive_begin(gm.pr_trianglelist)
          nv = 0
        end
        k = k + 1
      end
    end
    b = b + 1
  end
  gm.draw_primitive_end()
  gm.matrix_set(gm.matrix_world, old)
end

-- gml_Script____struct___210__tree_draw_leaf_wrap__scr_skilltree @ 0x140216040
function ___struct___210__tree_draw_leaf_wrap__scr_skilltree(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_tree_draw_leaf_wrap @ 0x140216240
function tree_draw_leaf_wrap(self, other, x, y, r, col, alpha, w)
  gm.draw_set_color(col)
  gm.draw_set_alpha(alpha)
  local k = 0
  while k < 2 do
    local segs = 28.0
    local turns = 1.35
    local dirn
    if k == 0 then
      dirn = 1.0
    else
      dirn = -1.0
    end
    local a0
    if k == 0 then
      a0 = 250.0
    else
      a0 = 70.0
    end
    local prev = gm.undefined
    local i = 0
    while i <= segs do
      local t = i / segs
      local ang = a0 + dirn * 360 * turns * t
      local rr = gm.lerp(0.95, 0.4, t)
      local px = x + gm.lengthdir_x(r * 0.62 * rr, ang)
      local py = y + gm.lengthdir_y(r * 0.42 * rr, ang)
      local p = gm.new(___struct___210__tree_draw_leaf_wrap__scr_skilltree, px, py)
      if not gm.is_undefined(prev) then
        local lw = gm.max(1.0, 2.4 * w * (1.0 - t * 0.6))
        gm.draw_line_width(prev.x, prev.y, p.x, p.y, lw)
      end
      prev = p
      i = i + 1
    end
    k = k + 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script_tree_draw_padlock @ 0x140217c10
function tree_draw_padlock(self, other, x, y, s, col, col_hole)
  local bw = s * 0.9
  local bh = s * 0.68
  local by = y - bh * 0.2
  local sr = bw * 0.3
  local lw = gm.max(2.0, s * 0.13)
  local n = 12.0
  gm.draw_set_color(col)
  local i = 0
  while i < n do
    local a0 = 180.0 * i / n
    local a1 = 180.0 * (i + 1) / n
    local sx0 = x + gm.lengthdir_x(sr, a0)
    local sy0 = by - bh * 0.28 + gm.lengthdir_y(sr, a0)
    local sx1 = x + gm.lengthdir_x(sr, a1)
    local sy1 = by - bh * 0.28 + gm.lengthdir_y(sr, a1)
    gm.draw_line_width(sx0, sy0, sx1, sy1, lw)
    i = i + 1
  end
  gm.draw_line_width(x - sr, by - bh * 0.28, x - sr, by, lw)
  gm.draw_line_width(x + sr, by - bh * 0.28, x + sr, by, lw)
  gm.draw_rectangle(x - bw * 0.5, by, x + bw * 0.5, by + bh, false)
  gm.draw_set_color(col_hole)
  gm.draw_circle(x, by + bh * 0.4, gm.max(1.0, s * 0.09), false)
  local kx0 = x - gm.max(1.0, s * 0.035)
  local ky0 = by + bh * 0.4
  local kx1 = x + gm.max(1.0, s * 0.035)
  local ky1 = by + bh * 0.72
  gm.draw_rectangle(kx0, ky0, kx1, ky1, false)
end

-- gml_GlobalScript_scr_skilltree @ 0x14021a290
function gml_GlobalScript_scr_skilltree(self, other)
  self.tree_seed = gm.method(self, tree_seed)
  self.tree_strand_point = gm.method(self, tree_strand_point)
  self.tree_noise = gm.method(self, tree_noise)
  self.tree_strand_path = gm.method(self, tree_strand_path)
  self.tree_draw_hair = gm.method(self, tree_draw_hair)
  self.tree_draw_strand = gm.method(self, tree_draw_strand)
  self.tree_draw_glow = gm.method(self, tree_draw_glow)
  self.tree_draw_level_ring = gm.method(self, tree_draw_level_ring)
  self.tree_icon_box = gm.method(self, tree_icon_box)
  self.tree_icon_scale = gm.method(self, tree_icon_scale)
  self.tree_draw_icon = gm.method(self, tree_draw_icon)
  self.tree_draw_guide = gm.method(self, tree_draw_guide)
  self.tree_bend_point = gm.method(self, tree_bend_point)
  self.tree_hypha_walk = gm.method(self, tree_hypha_walk)
  self.tree_hypha_cells = gm.method(self, tree_hypha_cells)
  self.tree_hypha_build = gm.method(self, tree_hypha_build)
  self.tree_hypha_get = gm.method(self, tree_hypha_get)
  self.tree_hypha_length = gm.method(self, tree_hypha_length)
  self.tree_hypha_cell_quad = gm.method(self, tree_hypha_cell_quad)
  self.tree_hypha_draw = gm.method(self, tree_hypha_draw)
  self.tree_draw_leaf_wrap = gm.method(self, tree_draw_leaf_wrap)
  self.tree_draw_padlock = gm.method(self, tree_draw_padlock)
end

-- gml_Script____struct___248__upgrades_init__scr_upgrades @ 0x140290300
function ___struct___248__upgrades_init__scr_upgrades(self, other, tier)
  self.id = "origin"
  self.trunk = ""
  self.kind = "root"
  self.parent = ""
  self.req = 0
  self.tier = tier
  self.px = 0
  self.py = 0
  self.available = true
  self.icon = originshroom
end
