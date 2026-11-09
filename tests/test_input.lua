local input = require("gm.input")

return {
  key_pressed_lasts_one_step = function()
    input.reset()
    input.key_event("escape", true)
    assert(input.keyboard_check_pressed(input.vk_escape) and input.keyboard_check(input.vk_escape))
    input.end_step()
    assert(not input.keyboard_check_pressed(input.vk_escape) and input.keyboard_check(input.vk_escape))
    input.key_event("escape", false)
    assert(input.keyboard_check_released(input.vk_escape) and not input.keyboard_check(input.vk_escape))
  end,
  letters_map_to_uppercase_codes = function()
    input.reset()
    input.key_event("f", true)
    assert(input.keyboard_check_pressed(string.byte("F")))
  end,
  mouse_buttons = function()
    input.reset()
    input.mouse_event(1, true)
    assert(input.mouse_check_button_pressed(input.mb_left) and input.mouse_check_button(input.mb_left))
    assert(input.mouse_check_button_pressed(input.mb_any) and not input.mouse_check_button(input.mb_right))
    input.end_step()
    assert(not input.mouse_check_button_pressed(input.mb_left) and input.mouse_check_button(input.mb_left))
  end,
  press_and_release_within_one_step_still_seen = function()
    input.reset()
    input.mouse_event(1, true)
    input.mouse_event(1, false)
    assert(input.mouse_check_button_pressed(input.mb_left) and input.mouse_check_button_released(input.mb_left))
  end,
}
