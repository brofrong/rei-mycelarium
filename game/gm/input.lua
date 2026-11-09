-- Keyboard and mouse state with GameMaker semantics. "Pressed"/"released"
-- flags are latched until the end of the next game step.
local M = {
  mb_any = -1, mb_none = 0, mb_left = 1, mb_right = 2, mb_middle = 3,
  vk_nokey = 0, vk_anykey = 1, vk_backspace = 8, vk_tab = 9, vk_enter = 13, vk_shift = 16, vk_control = 17, vk_alt = 18,
  vk_escape = 27, vk_space = 32, vk_pageup = 33, vk_pagedown = 34, vk_end = 35, vk_home = 36,
  vk_left = 37, vk_up = 38, vk_right = 39, vk_down = 40, vk_delete = 46,
  vk_f1 = 112, vk_f2 = 113, vk_f3 = 114, vk_f4 = 115, vk_f5 = 116, vk_f6 = 117, vk_f7 = 118, vk_f8 = 119,
  vk_f9 = 120, vk_f10 = 121, vk_f11 = 122, vk_f12 = 123,
}

local KEYS = {
  backspace = 8, tab = 9, ["return"] = 13, kpenter = 13, lshift = 16, rshift = 16, lctrl = 17, rctrl = 17, lgui = 17, rgui = 17,
  lalt = 18, ralt = 18, escape = 27, space = 32, pageup = 33, pagedown = 34, ["end"] = 35, home = 36,
  left = 37, up = 38, right = 39, down = 40, delete = 46,
}
for i = 1, 12 do KEYS["f" .. i] = 111 + i end

function M.keycode(name)
  if KEYS[name] then return KEYS[name] end
  if #name == 1 then
    local b = name:upper():byte()
    if (b >= 48 and b <= 57) or (b >= 65 and b <= 90) then return b end
  end
  return nil
end

local down, pressed, released, mdown, mpressed, mreleased
M.wheel_up, M.wheel_down = false, false

function M.reset()
  down, pressed, released = {}, {}, {}
  mdown, mpressed, mreleased = {}, {}, {}
  M.wheel_up, M.wheel_down = false, false
end
M.reset()

function M.key_event(name, is_down)
  local code = M.keycode(name)
  if not code then return end
  if is_down then
    if not down[code] then pressed[code] = true end
    down[code] = true
  else
    down[code] = nil
    released[code] = true
  end
end

function M.mouse_event(button, is_down)
  if is_down then
    mpressed[button] = true
    mdown[button] = true
  else
    mdown[button] = nil
    mreleased[button] = true
  end
end

function M.wheel_event(dy)
  if dy > 0 then M.wheel_up = true elseif dy < 0 then M.wheel_down = true end
end

function M.end_step()
  pressed, released, mpressed, mreleased = {}, {}, {}, {}
  M.wheel_up, M.wheel_down = false, false
end

local function any(t) return next(t) ~= nil end
local function check(t, code)
  if code == M.vk_anykey then return any(t) end
  if code == M.vk_nokey then return not any(t) end
  return t[code] == true
end
function M.keyboard_check(k) return check(down, k) end
function M.keyboard_check_pressed(k) return check(pressed, k) end
function M.keyboard_check_released(k) return check(released, k) end

local function mcheck(t, b)
  if b == M.mb_any then return any(t) end
  if b == M.mb_none then return not any(t) end
  return t[b] == true
end
function M.mouse_check_button(b) return mcheck(mdown, b) end
function M.mouse_check_button_pressed(b) return mcheck(mpressed, b) end
function M.mouse_check_button_released(b) return mcheck(mreleased, b) end
function M.mouse_wheel_up() return M.wheel_up end
function M.mouse_wheel_down() return M.wheel_down end

return M
