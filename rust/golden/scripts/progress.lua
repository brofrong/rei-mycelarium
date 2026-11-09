-- Progress (scr_upgrades, tasks, goals, tutorial) as the LÖVE port computes it, for
-- rust/crates/myco_core/tests/progress.rs. Run with MYCO_SEED=31607 (the tutorial's
-- "nearest food / lake" reads the generated world; the chunks it reads are dumped too).
local gm = require("gm")

global.title_open, global.intro_active = false, false
global.pause_open, global.debug_open, global.logo_open, global.menu_open = false, false, false, false

local function b(k) return bal(global, global, k) end
local function copy(s)
  if not gm.is_struct(s) then return s end
  local t = {}
  local names = gm.variable_struct_get_names(s)
  for i = 0, gm.array_length(names) - 1 do
    t[names[i]] = copy(gm.variable_struct_get(s, names[i]))
  end
  return t
end
local function obj(t) -- force a JSON object even when empty
  t = t or {}
  t.__obj = true
  return t
end

-- upgrade definitions ---------------------------------------------------------------------
local defs = {}
for i = 0, gm.array_length(global.upgrade_defs) - 1 do
  local d = global.upgrade_defs[i]
  local icon = gm.variable_struct_get(d, "icon")
  local pos = upg_tree_pos(global, global, d)
  local via = upg_tree_via(global, global, d)
  local kids = {}
  local ch = upg_children(global, global, d.id)
  for k = 0, gm.array_length(ch) - 1 do kids[#kids + 1] = ch[k].id end
  defs[#defs + 1] = {
    id = d.id, trunk = d.trunk, kind = d.kind, parent = d.parent, req = d.req, tier = d.tier,
    px = d.px, py = d.py,
    bx = gm.variable_struct_get(d, "bx"), by = gm.variable_struct_get(d, "by"),
    available = d.available,
    icon = (type(icon) == "number" and icon >= 0) and gm.sprite_get_name(icon) or "",
    max_key = gm.variable_struct_get(d, "max_key") or "",
    step_key = gm.variable_struct_get(d, "step_key") or "",
    cost_key = gm.variable_struct_get(d, "cost_key") or "",
    pos = { pos.x, pos.y },
    via = gm.is_undefined(via) and "none" or { via.x, via.y },
    children = obj(kids),
    hypha_len = tree_hypha_length(global, global, d.id),
  }
end

-- per-level values in both modes --------------------------------------------------------------
local function set_levels(t)
  global.upgrade_levels = gm.struct({})
  global.upgrade_levels.origin = 1.0
  for k, v in pairs(t) do gm.variable_struct_set(global.upgrade_levels, k, v) end
end

local mode_before = game_mode(global, global)
local levels = {}
local states = {}
for _, mode in ipairs({ "exploration", "survival" }) do
  game_mode_apply(global, global, mode)
  local rows = {}
  local srows = {}
  for i = 0, gm.array_length(global.upgrade_defs) - 1 do
    local d = global.upgrade_defs[i]
    local mx = upg_max_level(global, global, d.id)
    local lv = {}
    for l = 0, mx + 1 do
      set_levels({ [d.id] = l })
      lv[#lv + 1] = {
        level = l,
        bonus = upg_bonus(global, global, d.id),
        cost = upg_cost(global, global, d.id),
        owned = upg_is_owned(global, global, d.id),
      }
    end
    rows[d.id] = { max = mx, levels = lv }
    -- can_buy / state: parent level 0 or 1, own level 0 or max, points around the cost
    local st = {}
    for _, plv in ipairs({ 0, 1 }) do
      for _, own in ipairs({ 0, mx }) do
        local lvls = { [d.id] = own }
        if d.parent ~= "" and d.parent ~= "origin" then lvls[d.parent] = plv end
        if d.parent == "origin" then lvls.origin = (plv == 0) and 0 or 1 end
        set_levels(lvls)
        local cost = upg_cost(global, global, d.id)
        for _, pts in ipairs({ 0, cost - 1, cost, 1000 }) do
          global.upgrade_points = pts
          for _, demo in ipairs({ 0, 1 }) do
            global.balance.demo_upgrades_only = demo
            st[#st + 1] = {
              parent_level = plv, own = own, points = pts, demo = demo,
              can_buy = upg_can_buy(global, global, d.id),
              state = upg_state(global, global, d.id),
              demo_locked = upg_demo_locked(global, global, d.id),
              req_met = upg_requirement_met(global, global, d.id),
            }
          end
          global.balance.demo_upgrades_only = 0
        end
      end
    end
    srows[d.id] = st
  end
  levels[mode] = rows
  states[mode] = srows
end
game_mode_apply(global, global, mode_before)

-- a purchase sequence -----------------------------------------------------------------------------
set_levels({})
global.upgrade_points = 40
local buys = {}
local seq = { "spore_count", "unlock_spore", "spore_count", "spore_count", "shot_range", "shot_range",
  "origin", "nope", "unlock_absorption", "absorption_yield", "unlock_protein", "spore_count",
  "spore_count", "spore_count", "spore_count", "spore_count", "unlock_digger", "digger_length",
  "unlock_repeller", "repeller_autoregen", "unlock_tree", "tree_water_yield", "tree_drop_rate" }
for _, id in ipairs(seq) do
  local ok = upg_buy(global, global, id)
  buys[#buys + 1] = { id = id, ok = ok, points = global.upgrade_points, levels = copy(global.upgrade_levels) }
end

-- grow animation ----------------------------------------------------------------------------------
upgrades_init(global, global)
global.upgrade_levels.unlock_spore = 1
global.upgrade_levels.spore_count = 2
local grow = {}
for f = 1, 90 do
  upgrades_grow_step(global, global)
  if f % 10 == 0 or f <= 3 then
    grow[#grow + 1] = { frame = f, spore = upg_grow(global, global, "unlock_spore"),
      count = upg_grow(global, global, "spore_count"), origin = upg_grow(global, global, "origin") }
  end
end
global.upgrade_levels.spore_count = 0
upgrades_grow_step(global, global)
grow[#grow + 1] = { frame = 91, spore = upg_grow(global, global, "unlock_spore"),
  count = upg_grow(global, global, "spore_count"), origin = upg_grow(global, global, "origin") }

-- tasks -------------------------------------------------------------------------------------------
upgrades_init(global, global)
tutorial_init(global, global)
tasks_init(global, global)
global.mycelium_length_px = 0
global.mushrooms = gm.array()
global.food = 0
global.water_free = 0
global.water_sources = gm.array()
global.cave_perks = gm.struct({})

local function tasks_snap(label)
  local t = global.tasks
  return {
    label = label,
    stats = copy(t.stats), done = obj(copy(t.done)), fill = obj(copy(t.fill)), fold_k = obj(copy(t.fold_k)),
    goals = copy(t.goals), stage = t.stage, popup = t.popup, shown = t.shown, alpha = t.alpha,
    points = global.upgrade_points,
    met = (function()
      local m = {}
      local d = tasks_defs(global, global)
      for i = 0, gm.array_length(d) - 1 do m[d[i]] = tasks_met(global, global, d[i]) end
      return m
    end)(),
    goal_migration = goal_met(global, global, "migration"),
    goal_magic = goal_met(global, global, "magic"),
    spore_count = goal_spore_count(global, global),
    visible = tasks_visible(global, global),
  }
end

local tsnaps = {}
local actions = {
  { "step" },
  { "final" },
  { "step" }, { "step" },
  { "stat", "water", 600 }, { "stat", "water", 400 }, { "step" },
  { "stat", "spores" }, { "stat", "food", 29.5 }, { "step" },
  { "stat", "food", 0.5 }, { "close", "tasks" }, { "step" }, { "step" },
  { "upg", "unlock_absorption" }, { "upg", "shot_range" }, { "step" },
  { "stat", "rain_points" }, { "stat", "built", 5 }, { "stat", "hedgehogs" }, { "step" },
  { "len", 32000 }, { "stat", "tree_built", 2 }, { "open", "tasks" }, { "close", "magic" }, { "step" },
  { "stat", "stinky_built", 1 }, { "step" },
  { "stat", "stinky_built", 1 }, { "step" }, { "step" },
  { "popup_clear" }, { "step" },
  { "spores", 60 }, { "food", 10000 }, { "water", 9999 }, { "step" },
  { "water", 1 }, { "step" },
  { "spores", 40 }, { "step" },
  { "popup_clear" }, { "step" },
  { "perks", "blue", "purple", "red", "green" }, { "step" },
  { "perks", "yellow" }, { "step" },
  { "hud_hidden" }, { "step" },
  { "survival" }, { "step" },
}
for n, a in ipairs(actions) do
  local k = a[1]
  if k == "step" then
    tasks_step(global, global)
  elseif k == "final" then
    global.tut.tips.final.state = 1
  elseif k == "stat" then
    tasks_stat_add(global, global, a[2], a[3])
  elseif k == "close" or k == "open" then
    if (k == "open") ~= gm.bool(tasks_block_open(global, global, a[2])) then
      tasks_block_toggle(global, global, a[2])
    end
  elseif k == "upg" then
    gm.variable_struct_set(global.upgrade_levels, a[2], 1)
  elseif k == "len" then
    global.mycelium_length_px = a[2]
  elseif k == "popup_clear" then
    global.tasks.popup = ""
  elseif k == "spores" then
    for _ = 1, a[2] do
      gm.array_push(global.mushrooms, gm.struct({ type = 0, stolen = false, carried_by = -1 }))
    end
    gm.array_push(global.mushrooms, gm.struct({ type = 0, stolen = true, carried_by = -1 }))
    gm.array_push(global.mushrooms, gm.struct({ type = 0, stolen = false, carried_by = 3 }))
    gm.array_push(global.mushrooms, gm.struct({ type = 1, stolen = false, carried_by = -1 }))
  elseif k == "food" then
    global.food = a[2]
  elseif k == "water" then
    global.water_free = global.water_free + a[2]
  elseif k == "perks" then
    for i = 2, #a do gm.variable_struct_set(global.cave_perks, a[i], true) end
  elseif k == "hud_hidden" then
    global.hud_hidden = true
  elseif k == "survival" then
    global.hud_hidden = false
    game_mode_apply(global, global, "survival")
  end
  tsnaps[#tsnaps + 1] = tasks_snap(n .. ":" .. table.concat((function()
    local s = {}
    for i = 1, #a do s[i] = tostring(a[i]) end
    return s
  end)(), " "))
end
game_mode_apply(global, global, mode_before)
global.hud_hidden = false

-- goals: number text -----------------------------------------------------------------------------
local numbers = {}
for _, v in ipairs({ 0, 1, 12, 123, 999, 1000, 1234.5, 1235.5, 10000, 99999.4, 123456, 1234567, -1234, 0.49 }) do
  numbers[#numbers + 1] = { v, goal_number_text(global, global, v) }
end

-- goals: text fitting with a fixed fake font (the Rust test uses the same metrics) ------------------
local function mono_w(s) return gm.string_length(s) * 10 end
local function lines(s, w)
  if w <= 0 then return 1 end
  return math.max(1, math.ceil(mono_w(s) / w))
end
local saved = { gm.string_width, gm.string_height, gm.string_width_ext, gm.string_height_ext }
gm.string_width = function(s) return mono_w(s) end
gm.string_height = function(s) return 20 end
gm.string_width_ext = function(s, sep, w) return math.min(mono_w(s), w) end
gm.string_height_ext = function(s, sep, w) return lines(s, w) * (sep < 0 and 20 or sep) end
local fits = {}
for _, c in ipairs({
  { "Hello world", 1.0, 200, 0, -1 }, { "Hello world", 1.0, 50, 0, -1 }, { "Hello world", 2.0, 100, 0, -1 },
  { "Длинный текст задачи", 0.5, 60, 0, -1 }, { "A long goal text to wrap", 1.5, 120, 40, 24 },
  { "A long goal text to wrap", 1.0, 120, 30, -1 }, { "x", 0.06, 1, 0, -1 }, { "tiny", 1.0, 0.1, 0, -1 },
}) do
  fits[#fits + 1] = { args = c, scale = goal_fit(global, global, c[1], c[2], c[3], c[4], c[5]) }
end
local text_scale = { goal_text_scale(global, global, 36), goal_text_scale(global, global, 22) }
gm.string_width, gm.string_height, gm.string_width_ext, gm.string_height_ext = saved[1], saved[2], saved[3], saved[4]

-- tutorial ----------------------------------------------------------------------------------------
local wraps = {}
for _, s in ipairs({ "short", "Shoot the mycelium into the ground", "Найди воду рядом с грибницей и выпей её",
  "nospacesatallinthisverylongstringhereokay", "a b", "Two words exactly thirty-three!!" }) do
  wraps[#wraps + 1] = { s, tutorial_wrap(global, global, s) }
end

upgrades_init(global, global)
tutorial_init(global, global)
tasks_init(global, global)
global.mycelium_length_px = 0
global.weather.intensity = 0
global.start_x = 0

local function tut_snap(label)
  local t = global.tut
  local tips = {}
  for i = 0, gm.array_length(t.order) - 1 do
    local tp = gm.variable_struct_get(t.tips, t.order[i])
    tips[t.order[i]] = { state = tp.state, alpha = tp.alpha, timer = tp.timer, anchor = tp.anchor, x = tp.x, y = tp.y }
  end
  return {
    label = label, tips = tips, ate = t.ate, drank = t.drank, hand_done = t.hand_done,
    hand_alpha = t.hand_alpha, hand_frame = t.hand_frame, camera_alpha = t.camera_alpha,
    camera_frame = t.camera_frame, pending = obj(copy(t.events)),
  }
end

-- node near the start; dump the chunks tutorial_show_nearest reads
local node_x, node_y = 260, wld(global, global, "surface_height_px") + 300
local chunks = {}
do
  local c = gm.floor(node_x / global.world.chunk_w)
  for ci = c - 2, c + 2 do
    local ch = world_get_chunk(global, global, ci)
    local foods, lakes = {}, {}
    for i = 0, gm.array_length(ch.foods) - 1 do
      local f = ch.foods[i]
      foods[#foods + 1] = { x = f.x, y = f.y, collected = f.collected }
    end
    for i = 0, gm.array_length(ch.lakes) - 1 do
      local l = ch.lakes[i]
      lakes[#lakes + 1] = { body_x = l.body_x, body_y = l.body_y, body_w = l.body_w, body_h = l.body_h,
        connected = l.connected }
    end
    chunks[#chunks + 1] = { cx = ci, foods = obj(foods), lakes = obj(lakes) }
  end
end

local tsteps = {}
local tut_actions = {
  { 3, nil },
  { 1, "aim" }, { 8, nil },
  { 1, "shot", 100, 900 }, { 40, nil },
  { 1, "node", node_x, node_y }, { 40, nil },
  { 1, "clay", 300, 1200 }, { 2, nil },
  { 1, "food" }, { 35, nil },
  { 1, "water" }, { 35, nil },
  { 1, "len", 3200 }, { 5, nil },
  { 1, "len", 3201 }, { 330, nil },
  { 1, "rain_on" }, { 40, nil },
  { 1, "rain_off" }, { 40, nil },
  { 1, "menu_on" }, { 10, nil },
  { 1, "menu_off" }, { 1, "upg", "unlock_spore" }, { 40, nil },
  { 1, "mushroom" }, { 40, nil },
  { 1, "pause_on" }, { 5, nil }, { 1, "pause_off" },
  { 600, nil }, { 40, nil },
  { 1, "title_on", "aim" }, { 1, "title_off" }, { 3, nil },
}
for n, a in ipairs(tut_actions) do
  local frames, k = a[1], a[2]
  if k == "aim" or k == "shot" or k == "node" or k == "clay" or k == "food" or k == "water" or k == "mushroom" then
    tutorial_event(global, global, k, a[3], a[4])
  elseif k == "len" then
    global.mycelium_length_px = a[3]
  elseif k == "rain_on" then
    global.weather.intensity = 0.5
  elseif k == "rain_off" then
    global.weather.intensity = 0
  elseif k == "menu_on" then
    global.menu_open = true
  elseif k == "menu_off" then
    global.menu_open = false
  elseif k == "upg" then
    gm.variable_struct_set(global.upgrade_levels, a[3], 1)
  elseif k == "pause_on" then
    global.pause_open = true
  elseif k == "pause_off" then
    global.pause_open = false
  elseif k == "title_on" then
    tutorial_event(global, global, a[3])
    global.title_open = true
  elseif k == "title_off" then
    global.title_open = false
  end
  for _ = 1, frames do tutorial_step(global, global) end
  tsteps[#tsteps + 1] = tut_snap(n .. ":" .. tostring(k) .. " x" .. frames)
end
global.weather.intensity = 0

-- cheats -------------------------------------------------------------------------------------------
cheat_flash(global, global, "CHEAT")
local cheat = { global.cheat_text, global.cheat_timer }
for _ = 1, 10 do cheats_step(global, global) end
cheat[#cheat + 1] = global.cheat_timer

return {
  defs = defs,
  levels = levels,
  states = states,
  buys = buys,
  grow = grow,
  tasks = tsnaps,
  task_defs = (function()
    local d = tasks_defs(global, global)
    local t = {}
    for i = 0, gm.array_length(d) - 1 do t[#t + 1] = d[i] end
    return t
  end)(),
  numbers = numbers,
  fits = fits,
  text_scale = text_scale,
  wraps = wraps,
  chunk_w = global.world.chunk_w,
  chunks = chunks,
  surface_height_px = wld(global, global, "surface_height_px"),
  tutorial = tsteps,
  cheat = cheat,
}
