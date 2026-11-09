-- Pure display math (no love dependency): fitting the 1920x1080 application
-- surface into the window's safe area and mapping the mouse back.
local M = {}

-- Rectangle (in window pixels) where the application surface is drawn:
-- the largest aspect-preserving box inside the safe area, centred.
function M.fit(win_w, win_h, safe_x, safe_y, safe_w, safe_h, base_w, base_h)
  if win_w <= 0 or win_h <= 0 or safe_w <= 0 or safe_h <= 0 then
    return { x = 0, y = 0, w = 0, h = 0, scale = 0 }
  end
  local scale = math.min(safe_w / base_w, safe_h / base_h)
  local w, h = base_w * scale, base_h * scale
  return {
    x = safe_x + (safe_w - w) / 2,
    y = safe_y + (safe_h - h) / 2,
    w = w,
    h = h,
    scale = scale,
  }
end

-- Inverse of fit: window pixel -> game pixel. Points outside the picture map
-- outside 0..base; a degenerate fit maps everything to -1, -1.
function M.screen_to_game(fit, px, py)
  if fit.scale == 0 then return -1, -1 end
  return (px - fit.x) / fit.scale, (py - fit.y) / fit.scale
end

local function round(x) return math.floor(x + 0.5) end

-- Mirror of the original gml_Script_display_fit: the view keeps 1920x1080 as a
-- minimum and grows along the window's aspect (clamped to 1.25..32/9); the
-- application surface follows the window size, capped at 4K pixels.
function M.view_for_window(ww, wh)
  if ww <= 0 or wh <= 0 then return nil end
  local aspect = math.max(1.25, math.min(32 / 9, ww / wh))
  local vw, vh
  if aspect >= 16 / 9 then
    vh = 1080
    vw = round(1080 * aspect)
  else
    vw = 1920
    vh = round(1920 / aspect)
  end
  local k = math.min(ww / vw, wh / vh, math.sqrt(3840 * 2160 / (vw * vh)))
  return {
    view_w = vw,
    view_h = vh,
    surf_w = math.max(round(vw * k), 1),
    surf_h = math.max(round(vh * k), 1),
    ui_ox = (vw - 1920) * 0.5,
    ui_by = vh - 1080,
  }
end

local DEFAULTS ={ fullscreen = true, window_w = 1280, window_h = 720 }

-- settings.ini in the original's format: [display] fullscreen/window_w/window_h as quoted reals.
function M.ini_to_settings(text)
  local t = { fullscreen = DEFAULTS.fullscreen, window_w = DEFAULTS.window_w, window_h = DEFAULTS.window_h }
  local section
  for line in (text or ""):gmatch("[^\r\n]+") do
    local s = line:match("^%s*%[(.-)%]%s*$")
    if s then
      section = s
    elseif section == "display" then
      local k, v = line:match('^%s*([%w_]+)%s*=%s*"?([^"]*)"?%s*$')
      local n = tonumber(v)
      if k == "fullscreen" and n then
        t.fullscreen = n ~= 0
      elseif (k == "window_w" or k == "window_h") and n and n > 0 then
        t[k] = math.floor(n)
      end
    end
  end
  return t
end

function M.settings_to_ini(t)
  return string.format('[display]\r\nfullscreen="%f"\r\nwindow_w="%f"\r\nwindow_h="%f"\r\n',
    t.fullscreen and 1 or 0, t.window_w, t.window_h)
end

return M
