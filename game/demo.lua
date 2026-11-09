-- Visual checks for the compatibility layer: MYCO_DEMO=draw love game
local gm = require("gm")
local display = require("gm.display")
local dm = require("gm.display_math")

local M = {}
local view, ttf, t = nil, nil, 0

function M.load()
  gm.assets.load(_G)
  display.init()
  ttf = gm.font_add("fonts/cyrillic-ger-spa-port.ttf", 48, false, false, 32, 1200)
  M.resize()
end

function M.resize()
  local ww, wh = display.window_size()
  view = dm.view_for_window(ww, wh)
  if not gm.surface_exists(M.surf) then M.surf = gm.surface_create(view.surf_w, view.surf_h) end
  gm.surface_resize(M.surf, view.surf_w, view.surf_h)
  display.update(view.surf_w, view.surf_h, view.view_w, view.view_h)
end

function M.update(dt) t = t + dt end

function M.draw()
  gm.surface_set_target(M.surf)
  love.graphics.scale(view.surf_w / view.view_w, view.surf_h / view.view_h)
  gm.draw_clear(gm.make_colour_rgb(30, 24, 20))
  gm.draw_sprite(Earth, 0, 40, 40)
  gm.draw_sprite_ext(apple, 0, 700, 200, 2, 2, t * 45, gm.c_white, 1)
  gm.draw_sprite_part_ext(boar, 0, 0, 0, 200, 200, 900, 100, 1, 1, gm.c_white, 1)
  gm.draw_set_font(fnt_gm_30)
  gm.draw_set_colour(gm.c_yellow)
  gm.draw_text(700, 450, "fnt_gm_30: ЕДА / ВОДА / FOOD 123")
  gm.draw_set_font(ttf)
  gm.draw_set_halign(gm.fa_center)
  gm.draw_set_colour(gm.c_white)
  gm.draw_text_ext(1200, 550, "TTF: Мицелий растёт под землёй, а ёжик ищет яблоки", -1, 600)
  gm.draw_set_halign(gm.fa_left)
  gm.draw_rectangle_colour(700, 800, 1100, 900, gm.c_red, gm.c_lime, gm.c_blue, gm.c_yellow, false)
  gm.draw_set_colour(gm.c_aqua)
  gm.draw_circle(1300, 850, 60, true)
  gm.draw_line_width_colour(1450, 800, 1800, 900, 8, gm.c_orange, gm.c_fuchsia)
  gm.surface_reset_target()
  love.graphics.clear(0, 0, 0, 1)
  display.present(gm.surface_get_canvas(M.surf))
end

return M
