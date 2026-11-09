-- Procedural black-metal logo (scr_logo) reference for rust/crates/myco_app/tests/logo.rs:
-- the model logo_build makes for a few texts/seeds (ribbons, blades, bounds), small helpers
-- (logo_noise, logo_text_seed, logo_reach, logo_read_typing) and the vertices logo_paint emits
-- at several growth progresses (draw_vertex is recorded instead of drawn).
local gm = require("gm")
local S = global

local function arr(a)
  local out = {}
  for i = 0, gm.array_length(a) - 1 do out[#out + 1] = a[i] end
  return out
end

local function pts(a)
  local out = {}
  for _, p in ipairs(arr(a)) do out[#out + 1] = { p[0], p[1] } end
  return out
end

local function model(m)
  local ribbons, blades = {}, {}
  for _, r in ipairs(arr(m.ribbons)) do
    ribbons[#ribbons + 1] = {
      pts = pts(r.pts), len = arr(r.len), total = r.total, nib = r.nib, letter = r.letter,
      tip_start = gm.bool(r.tip_start), tip_end = gm.bool(r.tip_end),
    }
  end
  for _, b in ipairs(arr(m.blades)) do
    blades[#blades + 1] = { pts = pts(b.pts), width = b.width, root = b.root, birth = b.birth, span = b.span }
  end
  return {
    ribbons = ribbons, blades = blades, letters = m.letters, duration = m.duration,
    x1 = m.x1, x2 = m.x2, y1 = m.y1, y2 = m.y2,
    letter_x1 = m.letter_x1, letter_x2 = m.letter_x2, letter_y1 = m.letter_y1, letter_y2 = m.letter_y2,
  }
end

if global.logo_glyphs == nil then logo_init(S, S) end

local builds = {}
for _, c in ipairs({ { "MYCELARIUM", 1 }, { "MYCELARIUM", 1 + logo_text_seed(S, S, "MYCELARIUM") },
                     { "AB  Q", 7 }, { "W", 3 }, { "XYZ JKL", 42 }, { "", 1 }, { "   ", 1 } }) do
  builds[#builds + 1] = { text = c[1], seed = c[2], model = model(logo_build(S, S, c[1], c[2])) }
end

local noise = {}
for _, ab in ipairs({ { 0, 0 }, { 1, 4.1 }, { 137.5, 2.2 }, { -3, 6.1 }, { 1e4 + 0.25, 8.2 } }) do
  noise[#noise + 1] = { ab[1], ab[2], logo_noise(S, S, ab[1], ab[2]) }
end
local text_seed = {}
for _, s in ipairs({ "", "A", "MYCELARIUM", "BLACK METAL", "ZZZZZZZZZZZZZZZZZZ" }) do
  text_seed[#text_seed + 1] = { s, logo_text_seed(S, S, s) }
end
local reach = {}
for dir = -30, 400, 37 do
  for _, side in ipairs({ -1, -0.4, 0, 0.6, 1 }) do
    reach[#reach + 1] = { dir, side, logo_reach(S, S, dir, side) }
  end
end
local typing = {}
for _, s in ipairs({ "hello world", "Ab1!c d", "abcdefghijklmnopqrstuvwxyz", "Привет x" }) do
  gm.keyboard_string = s
  logo_read_typing(S, S)
  typing[#typing + 1] = { s, global.logo_text, gm.keyboard_string }
end

-- vertices logo_paint emits (MYCELARIUM, the seed logo_grow_start uses) at a few progresses
local verts = {}
local saved = { gm.draw_primitive_begin, gm.draw_vertex, gm.draw_primitive_end, gm.draw_set_colour, gm.draw_set_alpha }
local cur
gm.draw_primitive_begin = function() cur = {} end
gm.draw_vertex = function(x, y) cur[#cur + 1] = x; cur[#cur + 1] = y end
gm.draw_primitive_end = function() verts[#verts].strips[#verts[#verts].strips + 1] = cur end
gm.draw_set_colour = function() end
gm.draw_set_alpha = function() end
local m = logo_build(S, S, "MYCELARIUM", 1 + logo_text_seed(S, S, "MYCELARIUM"))
local old_model, old_progress = global.logo_model, global.logo_progress
global.logo_model = m
for _, p in ipairs({ 0.3, 1.7, 3.05, m.duration }) do
  global.logo_progress = p
  verts[#verts + 1] = { progress = p, strips = {} }
  logo_paint(S, S, 960, 590, 1720, 800, 0xFFFFFF)
end
global.logo_model, global.logo_progress = old_model, old_progress
gm.draw_primitive_begin, gm.draw_vertex, gm.draw_primitive_end, gm.draw_set_colour, gm.draw_set_alpha =
  saved[1], saved[2], saved[3], saved[4], saved[5]

return { builds = builds, noise = noise, text_seed = text_seed, reach = reach, typing = typing, paint = verts }
