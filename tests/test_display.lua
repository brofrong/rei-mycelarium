local dm = require("gm.display_math")

return {
  exact_16_9 = function()
    local f = dm.fit(3840, 2160, 0, 0, 3840, 2160, 1920, 1080)
    assert(f.scale == 2 and f.x == 0 and f.y == 0)
  end,
  notch_retina = function() -- 3024x1964 px, safe area below the notch (74 px at dpi 2)
    local f = dm.fit(3024, 1964, 0, 74, 3024, 1890, 1920, 1080)
    assert(f.y >= 74 and f.y + f.h <= 1964 and math.abs(f.w / f.h - 16 / 9) < 1e-3)
    local gx, gy = dm.screen_to_game(f, f.x + f.w - 1, f.y) -- top-right corner of the picture
    assert(gx > 1918 and gx < 1920 and gy >= 0 and gy < 1)
  end,
  outside_picture = function()
    local f = dm.fit(1000, 1000, 0, 0, 1000, 1000, 1920, 1080)
    local _, gy = dm.screen_to_game(f, 500, 5) -- in the top black bar
    assert(gy < 0)
  end,
  minimized = function()
    local f = dm.fit(0, 0, 0, 0, 0, 0, 1920, 1080)
    assert(f.scale == 0)
    local gx, gy = dm.screen_to_game(f, 10, 10)
    assert(gx == -1 and gy == -1)
  end,
  settings_roundtrip = function()
    local t = dm.ini_to_settings('[display]\r\nfullscreen="0.000000"\r\nwindow_w="1440.000000"\r\nwindow_h="810.000000"\r\n')
    assert(t.fullscreen == false and t.window_w == 1440 and t.window_h == 810)
    local back = dm.ini_to_settings(dm.settings_to_ini({fullscreen = true, window_w = 1280, window_h = 720}))
    assert(back.fullscreen == true and back.window_w == 1280 and back.window_h == 720)
  end,
  settings_defaults = function()
    local t = dm.ini_to_settings("")
    assert(t.fullscreen == true and t.window_w == 1280 and t.window_h == 720)
  end,
  view_macbook_safe_area = function() -- original display_fit on 3024x1890 (fullscreen below the notch)
    local v = dm.view_for_window(3024, 1890)
    assert(v.view_w == 1920 and v.view_h == 1200 and v.surf_w == 3024 and v.surf_h == 1890)
    assert(v.ui_ox == 0 and v.ui_by == 120)
  end,
  view_wide = function()
    local v = dm.view_for_window(3840, 1600)
    assert(v.view_w == 2592 and v.view_h == 1080 and v.surf_w == 3840 and v.surf_h == 1600 and v.ui_ox == 336 and v.ui_by == 0)
  end,
  view_caps_surface_at_4k = function()
    local v = dm.view_for_window(7680, 4320)
    assert(v.view_w == 1920 and v.view_h == 1080 and v.surf_w == 3840 and v.surf_h == 2160)
  end,
  view_clamps_ultrawide = function()
    local v = dm.view_for_window(5000, 1000)
    assert(v.view_w == 3840 and v.view_h == 1080 and v.surf_w == 3556 and v.surf_h == 1000)
  end,
  view_degenerate = function()
    assert(dm.view_for_window(0, 900) == nil)
  end,
  notch_fill_and_mouse = function() -- safe-area picture fills it fully; bottom-right maps to view size
    local v = dm.view_for_window(3024, 1890)
    local f = dm.fit(3024, 1964, 0, 74, 3024, 1890, v.surf_w, v.surf_h)
    assert(f.x == 0 and f.y == 74 and f.w == 3024 and f.h == 1890)
    local sx, sy = dm.screen_to_game(f, 3024, 1964)
    assert(sx * v.view_w / v.surf_w == 1920 and sy * v.view_h / v.surf_h == 1200)
  end,
  view_rounds_half_to_even = function()
    -- gml_Script_display_fit rounds with GML's round (banker's): n.5 goes to the even n
    assert(dm.view_for_window(4001, 2160).view_w == 2000)
    assert(dm.view_for_window(3840, 2401).view_h == 1200)
    assert(dm.view_for_window(4003, 2160).view_w == 2002)
  end,
}
