-- Localized strings as loc() returns them after locale_set(lang), for every available language,
-- plus the language list (locale_available order) and the font file per language, for
-- rust/crates/myco_core/tests/locale.rs.
local gm = require("gm")

local keys = {
  "hud_food", "hud_water", "hud_length", "hud_points", "hud_wind", "hud_cm", "hud_in",
  "wind_tooltip", "points_till_next", "hud_next_point", "button_add_node", "hint_no_nodes_in_water",
  "button_upgrades", "perk_purple_desc", "goal_tasks_text", "goal_migration_title", "goal_magic_done",
  "task_water", "task_food", "tut_node", "slot_default_name", "language_name", "button_language",
  "upg_parasitism_yield", "upg_parasitism_yield_desc", "upg_unlock_digger_desc", "tree_hint",
  "debug_title", "button_quit", "no_such_key_for_golden",
}

local before = global.locale_id
local langs = {}
for i = 0, gm.array_length(global.locale_langs) - 1 do langs[#langs + 1] = global.locale_langs[i] end

local strings, fonts = {}, {}
for _, lang in ipairs(langs) do
  locale_set(global, global, lang)
  local t = {}
  for _, k in ipairs(keys) do t[k] = loc(global, global, k) end
  strings[lang] = t
  fonts[lang] = locale_font_file(global, global, lang)
end
locale_set(global, global, before)

return { keys = keys, langs = langs, strings = strings, fonts = fonts }
