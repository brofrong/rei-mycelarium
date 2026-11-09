-- Upgrade-tree screen (scr_skilltree tree_* + obj_hud hud_tree_*) as the LÖVE port computes it,
-- for rust/crates/myco_app/tests/tree.rs: hypha geometry, the fitted layout (node positions in
-- GUI coordinates) at several GUI sizes, and the tree branch of obj_hud Step_1 driven by mocked
-- mouse/keyboard input (hover, wheel zoom, drag, click-to-buy, recentre, close).
local gm = require("gm")

global.title_open, global.intro_active = false, false
global.pause_open, global.debug_open, global.logo_open, global.menu_open = false, false, false, false
global.hud_hidden = false

local hud = gm.instance_find(obj_hud, 0)
assert(hud ~= nil and hud ~= gm.noone, "no obj_hud instance")

local function obj(t)
  t = t or {}
  t.__obj = true
  return t
end

local defs = {}
for i = 0, gm.array_length(global.upgrade_defs) - 1 do defs[#defs + 1] = global.upgrade_defs[i] end

local out = {}

-- pure helpers ----------------------------------------------------------------------------
out.seed = obj()
for _, d in ipairs(defs) do out.seed[d.id] = tree_seed(global, global, d.id) end
out.noise = {}
for _, ab in ipairs({ { 0, 0 }, { 1.3, 2 }, { 12.5, 7.1 }, { -3, 40 }, { 99.9, 120 } }) do
  out.noise[#out.noise + 1] = { ab[1], ab[2], tree_noise(global, global, ab[1], ab[2]) }
end
out.strand = {}
for _, t in ipairs({ 0, 0.25, 0.5, 0.8, 1 }) do
  local p = tree_strand_point(global, global, 10, 20, 300, 400, 1.7, 12, t)
  out.strand[#out.strand + 1] = { t, p.x, p.y }
end
out.strand_path = {}
do
  local pts = tree_strand_path(global, global, 10, 20, 300, 400, 1.7, 12, 6, 0.7)
  for i = 0, gm.array_length(pts) - 1 do
    out.strand_path[#out.strand_path + 1] = { pts[i].x, pts[i].y, pts[i].t }
  end
end
out.bend = {}
for _, t in ipairs({ 0, 0.3, 0.5, 1 }) do
  local p = tree_bend_point(global, global, 0, 0, 50, -80, 120, 30, t)
  out.bend[#out.bend + 1] = { t, p.x, p.y }
end

-- hyphae: build every link exactly as Draw_75 does (world positions, via) -------------------
local function cells_of(cells)
  local r, sx, sy, sw, sd = {}, 0, 0, 0, 0
  for k = 0, gm.array_length(cells) - 1 do
    local c = cells[k]
    sx, sy, sw, sd = sx + c.x, sy + c.y, sw + c.w, sd + c.d
  end
  return { n = gm.array_length(cells), sx = sx, sy = sy, sw = sw, sd = sd }
end
out.hyphae = {}
local full = { shot_range = true, unlock_absorption = true, unlock_stinky = true }
for _, d in ipairs(defs) do
  local p = upg_def(global, global, d.parent)
  if d.parent ~= "" and not gm.is_undefined(p) then
    local wa = hud.hud_tree_world(hud, hud, p)
    local wb = hud.hud_tree_world(hud, hud, d)
    local via = upg_tree_via(global, global, d)
    local h = tree_hypha_get(global, global, d.id, wa.x, wa.y, wb.x, wb.y, via)
    local e = { id = d.id, main_len = h.main_len, total = h.total, branches = {},
      length = tree_hypha_length(global, global, d.id) }
    for b = 0, gm.array_length(h.branches) - 1 do
      local br = h.branches[b]
      local s = cells_of(br.cells)
      s.start, s.span = br.start, br.span
      if full[d.id] then
        s.cells = {}
        for k = 0, gm.array_length(br.cells) - 1 do
          local c = br.cells[k]
          s.cells[#s.cells + 1] = { c.x, c.y, c.w, c.d }
        end
      end
      e.branches[#e.branches + 1] = s
    end
    out.hyphae[#out.hyphae + 1] = e
  end
end

-- layout at several GUI sizes ---------------------------------------------------------------
local gw0, gh0 = gm.display_get_gui_width(), gm.display_get_gui_height()
local function layout_snapshot()
  local c = hud.hud_tree_canvas(hud, hud)
  local nodes = {}
  for _, d in ipairs(defs) do
    local p = hud.hud_tree_screen(hud, hud, d)
    nodes[#nodes + 1] = { d.id, p.x, p.y, hud.hud_tree_radius(hud, hud, d) * hud.hud_tree_zoom }
  end
  local b = hud.hud_tree_bounds
  return {
    canvas = { c.x1, c.y1, c.x2, c.y2 },
    zoom = hud.hud_tree_zoom, fit_zoom = hud.hud_tree_fit_zoom, zoom_lo = hud.hud_tree_zoom_lo,
    cx = hud.hud_tree_cx, cy = hud.hud_tree_cy,
    bounds = { b.x1, b.y1, b.x2, b.y2 },
    nodes = nodes,
  }
end
out.layouts = {}
for _, sz in ipairs({ { 1920, 1080 }, { 2238, 1405 }, { 1920, 1920 }, { 3840, 1080 }, { 1280, 720 } }) do
  gm.display_set_gui_size(sz[1], sz[2])
  hud.hud_tree_fit(hud, hud)
  local s = layout_snapshot()
  s.gui = { sz[1], sz[2] }
  out.layouts[#out.layouts + 1] = s
end

-- Step_1 tree branch with mocked input --------------------------------------------------------
gm.display_set_gui_size(1920, 1080)
local real = {}
for _, k in ipairs({ "device_mouse_x_to_gui", "device_mouse_y_to_gui", "mouse_check_button",
  "mouse_check_button_pressed", "mouse_check_button_released", "mouse_wheel_up", "mouse_wheel_down",
  "keyboard_check_pressed" }) do real[k] = gm[k] end
local inp = { x = 0, y = 0, down = false, pressed = false, released = false, wup = false, wdown = false, keys = {} }
gm.device_mouse_x_to_gui = function() return inp.x end
gm.device_mouse_y_to_gui = function() return inp.y end
gm.mouse_check_button = function(b) return b == gm.mb_left and inp.down end
gm.mouse_check_button_pressed = function(b) return b == gm.mb_left and inp.pressed end
gm.mouse_check_button_released = function(b) return b == gm.mb_left and inp.released end
gm.mouse_wheel_up = function() return inp.wup end
gm.mouse_wheel_down = function() return inp.wdown end
gm.keyboard_check_pressed = function(k) return inp.keys[k] == true end

local function levels()
  local t = obj()
  for _, d in ipairs(defs) do t[d.id] = upg_level(global, global, d.id) end
  return t
end
local function pos_of(id)
  local p = hud.hud_tree_screen(hud, hud, upg_def(global, global, id))
  return p.x, p.y
end

global.upgrade_points = 3
global.upgrade_levels = gm.struct({})
global.upgrade_levels.origin = 1.0
global.menu_open = false
hud.hud_tree_fitted = false
-- one frame closed (resets fitted/drag as the original does), then open
gml_Object_obj_hud_Step_1(hud, hud)
global.menu_open = true

out.steps = {}
local function frame(label, a)
  inp.x, inp.y = a.x or inp.x, a.y or inp.y
  inp.pressed, inp.released = a.pressed or false, a.released or false
  if a.down ~= nil then inp.down = a.down end
  inp.wup, inp.wdown = a.wup or false, a.wdown or false
  inp.keys = a.keys or {}
  gml_Object_obj_hud_Step_1(hud, hud)
  out.steps[#out.steps + 1] = {
    label = label, input = { inp.x, inp.y, inp.down, inp.pressed, inp.released, inp.wup, inp.wdown, a.key or "" },
    menu_open = gm.bool(global.menu_open),
    hover = hud.hud_hover_id, zoom = hud.hud_tree_zoom, cx = hud.hud_tree_cx, cy = hud.hud_tree_cy,
    drag = gm.bool(hud.hud_tree_drag), dragged = gm.bool(hud.hud_tree_dragged), press_id = hud.hud_tree_press_id,
    points = global.upgrade_points, levels = levels(),
  }
end

frame("open (fit)", {})
local sx, sy = pos_of("shot_range")
frame("hover shot_range", { x = sx + 3, y = sy - 2 })
frame("press shot_range", { pressed = true, down = true })
frame("release shot_range (buy)", { released = true, down = false })
frame("press again", { pressed = true, down = true })
frame("release again (buy 2)", { released = true, down = false })
local ux, uy = pos_of("unlock_absorption")
frame("hover locked/short node", { x = ux, y = uy })
frame("press short", { pressed = true, down = true })
frame("release short (rejected)", { released = true, down = false })
frame("wheel up", { x = 700, y = 500, wup = true })
frame("wheel up 2", { x = 1500, y = 300, wup = true })
frame("wheel down", { x = 900, y = 800, wdown = true })
frame("press empty", { x = 300, y = 900, pressed = true, down = true })
frame("move 3px (no drag yet)", { x = 302, y = 901 })
frame("move far (drag)", { x = 360, y = 940 })
frame("move far 2", { x = 900, y = 200 })
frame("release (no buy)", { released = true, down = false })
for i = 1, 12 do frame("wheel down far " .. i, { x = 100, y = 150, wdown = true }) end
for i = 1, 4 do frame("drag far press " .. i, { x = 960, y = 540, pressed = true, down = true }) frame("drag far move " .. i, { x = 1900, y = 1000 }) frame("drag far rel " .. i, { released = true, down = false }) end
local sx2, sy2 = pos_of("shot_range")
frame("press node then drag", { x = sx2, y = sy2, pressed = true, down = true })
frame("drag off node", { x = sx2 + 40, y = sy2 + 40 })
frame("release after drag (no buy)", { released = true, down = false })
frame("recentre key R", { x = 10, y = 500, keys = { [82] = true }, key = "R" })
local cr = hud.hud_close_button_rect(hud, hud)
frame("click close", { x = (cr.x1 + cr.x2) * 0.5, y = (cr.y1 + cr.y2) * 0.5, pressed = true, down = true })

for k, f in pairs(real) do gm[k] = f end
gm.display_set_gui_size(gw0, gh0)
global.menu_open = false
return out
