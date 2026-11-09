local gm = require("gm")

-- gml_Script_cine_step @ 0x140092b10
function cine_step(self, other)
  if not gm.bool(cine_active(self, other)) then
    return
  end
  local c = global.tasks.cine
  local dt = 1 / gm.max(1, gm.game_get_speed(gm.gamespeed_fps))
  c.t = c.t + dt
  if c.phase ~= "show" then
    if c.phase == "play" and c.t >= vis(self, other, "migration_cine_seconds") then
      c.phase = "hold"
      global.tasks.popup = "migration_end"
      tasks_stat_add(self, other, "ach_migrated", gm.undefined)
    end
    if c.phase == "out" then
      c.alpha = c.alpha - dt / gm.max(0.05, vis(self, other, "migration_cine_fade_out_seconds"))
      if c.alpha <= 0 then
        global.tasks.cine = gm.undefined
      end
    end
    return
  end
  if c.stop < 0 or c.t >= c.stops[c.stop].seconds then
    c.stop = c.stop + 1
    c.t = 0
    c.shot = false
    if c.stop >= gm.array_length(c.stops) then
      cine_planet_start(self, other)
      return
    end
  end
  local s = c.stops[c.stop]
  if gm.instance_exists(obj_forest_controller) then
    gm.with(self, obj_forest_controller, function(self, other)
      self.camera_x = s.x - self.camera_width * 0.5
      self.camera_y = s.y - self.camera_height * vis(self, other, "migration_camera_ground_share")
      self.camera_vx = 0
      self.camera_vy = 0
    end)
  end
  local delay = vis(self, other, "migration_shot_delay_seconds")
  local clock = c.clock
  if gm.is_undefined(clock) then clock = 0 end
  c.clock = clock + dt
  if gm.is_undefined(c.firing) then
    c.firing = cine_spore_spots(self, other)
    if not gm.is_array(c.firing) or gm.array_length(c.firing) == 0 then
      local i = 0
      while i < gm.array_length(c.stops) do
        if c.stops[i].count <= 0 then
          gm.array_push(c.firing, gm.new(___struct___50__cine_step__scr_game, c.stops[i].x, c.stops[i].y))
        end
        i = i + 1
      end
    end
    local i = 0
    while i < gm.array_length(c.firing) do
      c.firing[i].next = gm.random(vis(self, other, "migration_burst_seconds"))
      i = i + 1
    end
  end
  local view_w
  if gm.instance_exists(obj_forest_controller) then
    view_w = gm.instance_find(obj_forest_controller, 0).camera_width
  else
    view_w = 2000
  end
  local i = 0
  while i < gm.array_length(c.firing) do
    local f = c.firing[i]
    if c.clock >= f.next then
      f.next = f.next + vis(self, other, "migration_burst_seconds")
      if gm.abs(f.x - s.x) > view_w then
        -- (skipped: out of view)
      else
        if gm.instance_exists(obj_mycelium_controller) then
          gm.with(self, obj_mycelium_controller, function(self, other)
            self.myc_spawn_spores_show(self, other, f.x, f.y - 52,
              gm.round(bal(self, other, "spore_base_count") * (1 + upg_bonus(self, other, "spore_count"))), 1)
          end)
        end
      end
    end
    i = i + 1
  end
  if s.count <= 0 then
    return
  end
  if not gm.bool(c.shot) and c.t >= delay then
    c.shot = true
    s.rays = gm.array()
    s.left = s.count
    local j = 0
    while j < vis(self, other, "migration_mega_rays") do
      local a = -12 + (j + gm.random_range(0.15, 0.85)) * 204 / vis(self, other, "migration_mega_rays")
      gm.array_push(s.rays, gm.new(___struct___51__cine_step__scr_game, a, gm.random_range(0.65, 1)))
      j = j + 1
    end
  end
  if gm.bool(c.shot) and s.left > 0 then
    local emit = vis(self, other, "migration_mega_emit_seconds")
    local n = gm.min(s.left, gm.ceil(s.count * dt / gm.max(0.05, emit)))
    s.left = s.left - n
    if gm.instance_exists(obj_mycelium_controller) then
      gm.with(self, obj_mycelium_controller, function(self, other)
        self.myc_spawn_spores_rays(self, other, s.x, s.y - 22, n, s.rays, vis(self, other, "migration_mega_speed"))
      end)
    end
  end
  local since = c.t - delay
  c.shake_k = 1 + (vis(self, other, "migration_mega_shake_max") - 1) * cine_span(self, other, since, 0, s.seconds - delay)
  c.white = gm.power(cine_span(self, other, since, vis(self, other, "migration_white_from"), s.seconds - delay), 1.6)
end

-- gml_Script_cine_span @ 0x140096270
function cine_span(self, other, t, a, b)
  return gm.clamp((t - a) / gm.max(0.001, b - a), 0, 1)
end

-- gml_Script_cine_draw @ 0x1400965d0
function cine_draw(self, other)
  if not gm.bool(cine_active(self, other)) then
    return
  end
  if global.tasks.cine.phase == "show" then
    local w = global.tasks.cine.white
    if gm.is_undefined(w) then w = 0 end
    if w > 0 then
      gm.draw_set_alpha(w)
      gm.draw_set_colour(gm.c_white)
      gm.draw_rectangle(0, 0, gm.display_get_gui_width(), gm.display_get_gui_height(), false)
      gm.draw_set_alpha(1)
    end
  else
    local c = global.tasks.cine
    local gw = gm.display_get_gui_width()
    local gh = gm.display_get_gui_height()
    local tt
    if c.phase == "play" then
      tt = c.t
    else
      tt = vis(self, other, "migration_cine_seconds") + c.t
    end
    local al = c.alpha
    local ball = cine_ball(self, other)
    gm.draw_set_alpha(al)
    gm.draw_set_colour(gm.c_black)
    gm.draw_rectangle(0, 0, gw, gh, false)
    local i = 0
    while i < gm.array_length(c.stars) do
      local st = c.stars[i]
      local tw = 0.85 + 0.15 * gm.dsin(st.ph + tt * 60 * st.sp)
      local sx = st.x * gw
      local sy = st.y * gh
      gm.draw_set_alpha(al * st.a * tw)
      gm.draw_set_colour(gm.c_white)
      gm.draw_rectangle(sx, sy, sx + st.s - 1, sy + st.s - 1, false)
      if st.s >= 3 then
        local h = (st.s - 1) * 0.5
        gm.draw_set_alpha(al * st.a * tw * 0.45)
        gm.draw_rectangle(sx - st.s, sy + h - 0.5, sx + st.s * 2 - 1, sy + h + 0.5, false)
        gm.draw_rectangle(sx + h - 0.5, sy - st.s, sx + h + 0.5, sy + st.s * 2 - 1, false)
      end
      i = i + 1
    end
    local k = ball.r / 540
    local wk = cine_span(self, other, tt, vis(self, other, "migration_cine_white_from"), vis(self, other, "migration_cine_white_to"))
    repeller_cloud_art_init(self, other)
    local spr = global.repel_flash_sprite
    local half = gm.sprite_get_width(spr) * 0.5
    local col = gm.merge_colour(16755310, 16777215, wk)
    local pulse = 1 + 0.04 * math.sin(tt * 3)
    gm.gpu_set_blendmode(gm.bm_add)
    gm.draw_sprite_ext(spr, 0, ball.x, ball.y, ball.r * 2.4 * pulse / half, ball.r * 2.4 * pulse / half, 0, col, al * gm.lerp(0.35, 0.6, wk))
    gm.draw_sprite_ext(spr, 0, ball.x, ball.y, ball.r * 1.5 / half, ball.r * 1.5 / half, 0, col, al * gm.lerp(0.45, 0.9, wk))
    gm.gpu_set_blendmode(gm.bm_normal)
    gm.draw_sprite_ext(Earth, 0, ball.x - 668 * k, ball.y - 618 * k, k, k, 0, gm.c_white, al)
    local mk = cine_span(self, other, tt, vis(self, other, "migration_cine_myc_from"), vis(self, other, "migration_cine_myc_to"))
    if mk > 0 then
      local lw = 1 + 3 * wk + 1.5 * mk
      gm.draw_set_colour(15529206)
      gm.draw_set_alpha(al)
      i = 0
      while i < gm.array_length(c.segs) do
        local sg = c.segs[i]
        if sg.time > mk then
          -- (segment not reached yet)
        else
          gm.draw_line_width(ball.x + sg.x1 * ball.r, ball.y + sg.y1 * ball.r, ball.x + sg.x2 * ball.r, ball.y + sg.y2 * ball.r,
            gm.max(1, lw * (0.4 + 0.6 * sg.w)))
        end
        i = i + 1
      end
    end
    if wk > 0 then
      gm.draw_set_alpha(al * wk * wk)
      gm.draw_set_colour(gm.c_white)
      gm.draw_circle(ball.x, ball.y, ball.r, false)
    end
    local fade = 1 - cine_span(self, other, tt, 0, vis(self, other, "migration_white_clear_seconds"))
    if c.phase == "play" and fade > 0 then
      gm.draw_set_alpha(al * fade)
      gm.draw_set_colour(gm.c_white)
      gm.draw_rectangle(0, 0, gw, gh, false)
    end
    gm.draw_set_alpha(1)
    gm.draw_set_colour(gm.c_white)
  end
end

-- gml_Script_ach_stat @ 0x14009ce20
function ach_stat(self, other, key)
  if gm.bool(gm.variable_global_exists("tasks")) then
    local v = gm.variable_struct_get(global.tasks.stats, key)
    if gm.is_undefined(v) then v = 0 end
    return v
  else
    return 0
  end
end

-- gml_Script_ach_standing @ 0x14009d3a0
function ach_standing(self, other, type)
  local n = 0
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    if global.mushrooms[i].type == type and not gm.bool(global.mushrooms[i].stolen) then
      n = n + 1
    end
    i = i + 1
  end
  return n
end

-- gml_Script_ach_list @ 0x14009d7a0
function ach_list(self, other)
  local cm = mycelium_cm(self, other)
  local pred = gm.max(ach_stat(self, other, "ach_predators"), ach_standing(self, other, 1))
  local perks = gm.bool(perk_has(self, other, "blue")) and gm.bool(perk_has(self, other, "purple"))
    and gm.bool(perk_has(self, other, "red")) and gm.bool(perk_has(self, other, "green"))
    and gm.bool(perk_has(self, other, "yellow"))
  return gm.array(
    gm.array("NEW_ACHIEVEMENT_1_00", cm >= 1000),
    gm.array("NEW_ACHIEVEMENT_1_01", cm >= 50000),
    gm.array("NEW_ACHIEVEMENT_1_02", cm >= 150000),
    gm.array("NEW_ACHIEVEMENT_1_03", cm >= 450000),
    gm.array("NEW_ACHIEVEMENT_1_04", cm >= 1000000),
    gm.array("NEW_ACHIEVEMENT_1_05", pred >= 20),
    gm.array("NEW_ACHIEVEMENT_1_06", pred >= 100),
    gm.array("NEW_ACHIEVEMENT_1_07", ach_stat(self, other, "ach_repels") >= 1),
    gm.array("NEW_ACHIEVEMENT_1_08", ach_stat(self, other, "ach_repels") >= 20),
    gm.array("NEW_ACHIEVEMENT_1_09", ach_stat(self, other, "ach_tree_water") >= 1000),
    gm.array("NEW_ACHIEVEMENT_1_10", ach_stat(self, other, "ach_trees_eaten") >= 15),
    gm.array("NEW_ACHIEVEMENT_1_11", ach_stat(self, other, "spores") >= 20),
    gm.array("NEW_ACHIEVEMENT_1_12", ach_stat(self, other, "ach_digger_full") >= 1),
    gm.array("NEW_ACHIEVEMENT_1_13", ach_stat(self, other, "ach_hedgehog_spores") >= 1),
    gm.array("NEW_ACHIEVEMENT_1_14", ach_stat(self, other, "ach_stinky_scares") >= 10),
    gm.array("NEW_ACHIEVEMENT_1_15", perks),
    gm.array("NEW_ACHIEVEMENT_1_16", relic_wrapped(self, other, "ufo")),
    gm.array("NEW_ACHIEVEMENT_1_17", relic_wrapped(self, other, "t_rex")),
    gm.array("NEW_ACHIEVEMENT_1_18", ach_stat(self, other, "ach_migrated") >= 1)
  )
end

-- gml_Script_ach_step @ 0x14009f800
function ach_step(self, other)
  if gm.bool(global.title_open) or gm.bool(global.intro_active) then
    return
  end
  if not gm.bool(gm.variable_global_exists("ach_timer")) then
    global.ach_timer = 0
    global.ach_given = gm.struct({})
    global.ach_tried = gm.struct({})
    global.ach_logged_state = ""
  end
  global.ach_timer = global.ach_timer - 1
  if global.ach_timer > 0 then
    return
  end
  local fps = gm.max(1, gm.game_get_speed(gm.gamespeed_fps))
  global.ach_timer = gm.max(1, gm.round(fps * 0.5))
  local ok = gm.steam_api_is_initialized()
  local state
  if gm.bool(ok) then
    state = "initialised=1"
  else
    state = "initialised=0 (" .. gm.string(gm.steam_api_last_error()) .. ")"
  end
  if state ~= global.ach_logged_state then
    global.ach_logged_state = state
    gm.show_debug_message("[ach] steam " .. state)
  end
  if not gm.bool(ok) then
    return
  end
  local list = ach_list(self, other)
  local i = 0
  while i < gm.array_length(list) do
    local name = list[i][0]
    if gm.bool(list[i][1]) then
      local given = gm.variable_struct_get(global.ach_given, name)
      if gm.is_undefined(given) then given = false end
      if not gm.bool(given) then
        if gm.real(steam_userstats_get_achievement(self, other, name)) == 1 then
          gm.variable_struct_set(global.ach_given, name, true)
          gm.show_debug_message("[ach] held: " .. name)
        else
          local tried = gm.variable_struct_get(global.ach_tried, name)
          if gm.is_undefined(tried) then tried = -gm.infinity end
          if gm.current_time - tried >= 5000 then
            gm.variable_struct_set(global.ach_tried, name, gm.current_time)
            gm.steam_userstats_set_achievement(name)
            gm.steam_userstats_store_stats()
            gm.show_debug_message("[ach] set: " .. name)
          end
        end
      end
    end
    i = i + 1
  end
end

-- gml_Script_display_init @ 0x1400a1530
function display_init(self, other)
  if gm.bool(gm.variable_global_exists("display_sizes")) then
    return
  end
  local all = gm.array(gm.array(1280, 720), gm.array(1366, 768), gm.array(1600, 900), gm.array(1920, 1080),
    gm.array(2560, 1440), gm.array(3200, 1800), gm.array(3840, 2160))
  local dw = gm.display_get_width()
  local dh = gm.display_get_height() - 80
  global.display_sizes = gm.array()
  local i = 0
  while i < gm.array_length(all) do
    if all[i][0] <= dw and all[i][1] <= dh then
      gm.array_push(global.display_sizes, all[i])
    end
    i = i + 1
  end
  if not gm.is_array(global.display_sizes) or gm.array_length(global.display_sizes) == 0 then
    gm.array_push(global.display_sizes, all[0])
  end
  gm.ini_open("settings.ini")
  global.display_fullscreen = gm.ini_read_real("display", "fullscreen", 1) ~= 0
  local ww = gm.ini_read_real("display", "window_w", 0)
  local wh = gm.ini_read_real("display", "window_h", 0)
  gm.ini_close()
  global.display_size_index = gm.array_length(global.display_sizes) - 1
  i = 0
  while i < gm.array_length(global.display_sizes) do
    if global.display_sizes[i][0] == ww and global.display_sizes[i][1] == wh then
      global.display_size_index = i
    end
    i = i + 1
  end
  global.display_pending = 0
  global.view_w = 1920
  global.view_h = 1080
  global.surf_w = 1920
  global.surf_h = 1080
  global.ui_ox = 0
  global.ui_by = 0
  display_apply(self, other)
  display_fit(self, other)
end

-- gml_Script_display_fit @ 0x1400a2b00
function display_fit(self, other)
  local ww = gm.window_get_width()
  local wh = gm.window_get_height()
  if ww <= 0 or wh <= 0 then
    return false
  end
  if not gm.bool(gm.surface_exists(gm.application_surface)) then
    return false
  end
  local aspect = gm.clamp(ww / wh, 1.25, 3.5555555555555554)
  local vw, vh
  if aspect >= 1.7777777777777777 then
    vh = 1080
    vw = gm.round(1080 * aspect)
  else
    vw = 1920
    vh = gm.round(1920 / aspect)
  end
  local k = gm.min(ww / vw, wh / vh)
  k = gm.min(k, gm.sqrt(8294400 / (vw * vh)))
  local sw = gm.max(1, gm.round(vw * k))
  local sh = gm.max(1, gm.round(vh * k))
  if vw == global.view_w and vh == global.view_h and sw == global.surf_w and sh == global.surf_h
    and gm.display_get_gui_width() == vw and gm.display_get_gui_height() == vh
    and gm.surface_get_width(gm.application_surface) == sw and gm.surface_get_height(gm.application_surface) == sh then
    return false
  end
  global.view_w = vw
  global.view_h = vh
  global.surf_w = sw
  global.surf_h = sh
  global.ui_ox = (vw - 1920) * 0.5
  global.ui_by = vh - 1080
  gm.surface_resize(gm.application_surface, sw, sh)
  gm.display_set_gui_size(vw, vh)
  if gm.bool(gm.variable_global_exists("logo_surface")) and gm.bool(gm.surface_exists(global.logo_surface)) then
    gm.surface_free(global.logo_surface)
    global.logo_surface = -1
  end
  return true
end

-- gml_Script_display_apply @ 0x1400a4680
function display_apply(self, other)
  if gm.window_get_fullscreen() ~= global.display_fullscreen then
    gm.window_set_fullscreen(global.display_fullscreen)
  end
  if gm.bool(global.display_fullscreen) then
    global.display_pending = 0
  else
    global.display_pending = 4
  end
end

-- gml_Script_focus_step @ 0x1400a4990
function focus_step(self, other)
  if gm.bool(gm.window_has_focus()) then
    return
  end
  if not (vis(self, other, "pause_on_focus_loss") > 0) then
    return
  end
  if not gm.bool(ui_blocking(self, other)) then
    global.pause_open = true
  end
end

-- gml_Script_art_preload_step @ 0x1400a4c90
function art_preload_step(self, other)
  if gm.bool(gm.variable_global_exists("art_preloaded")) and gm.bool(global.art_preloaded) then
    return
  end
  if gm.bool(gm.window_has_focus()) and gm.window_get_width() > 0 and gm.window_get_height() > 0
    and gm.bool(gm.surface_exists(gm.application_surface)) and gm.bool(gm.variable_global_exists("world"))
    and gm.bool(gm.variable_global_exists("visual")) then
    global.art_preloaded = true
    stones_init(self, other)
    stone_tiles_init(self, other)
    caves_mask_init(self, other)
    lakes_init(self, other)
    hedgehog_back_init(self, other)
    leaf_art_init(self, other)
    leaf_fg_art_init(self, other)
    squirrel_art_init(self, other)
    stinky_art_init(self, other)
    repeller_cloud_art_init(self, other)
  end
end

-- gml_Script_display_step @ 0x1400a55e0
function display_step(self, other)
  if global.display_pending > 0 then
    global.display_pending = global.display_pending - 1
    if global.display_pending == 2 then
      local sz = display_window_size(self, other)
      gm.window_set_size(sz[0], sz[1])
    end
    if global.display_pending == 0 then
      gm.window_center()
    end
  end
  display_fit(self, other)
end

-- gml_Script_display_window_size @ 0x1400a5ac0
function display_window_size(self, other)
  return global.display_sizes[gm.real(global.display_size_index)]
end

-- gml_Script_display_save @ 0x1400a5c00
function display_save(self, other)
  local sz = display_window_size(self, other)
  gm.ini_open("settings.ini")
  gm.ini_write_real("display", "fullscreen", gm.bool(global.display_fullscreen) and 1 or 0)
  gm.ini_write_real("display", "window_w", sz[0])
  gm.ini_write_real("display", "window_h", sz[1])
  gm.ini_close()
end

-- gml_Script_display_set_fullscreen @ 0x1400a6020
function display_set_fullscreen(self, other, on)
  if on == nil then on = gm.undefined end
  global.display_fullscreen = on
  display_apply(self, other)
  display_save(self, other)
end

-- gml_Script_display_cycle_size @ 0x1400a61c0
function display_cycle_size(self, other, dir)
  local n = gm.array_length(global.display_sizes)
  global.display_size_index = gm.mod(global.display_size_index + dir + n, n)
  display_apply(self, other)
  display_save(self, other)
end

-- gml_GlobalScript_scr_game @ 0x1400a6500
function gml_GlobalScript_scr_game(self, other)
  self.game_init = gm.method(self, game_init)
  self.game_step = gm.method(self, game_step)
  self.game_mode_start = gm.method(self, game_mode_start)
  self.cheats_step = gm.method(self, cheats_step)
  self.post_layer_draw = gm.method(self, post_layer_draw)
  self.surface_fit = gm.method(self, surface_fit)
  self.bw_shader_set = gm.method(self, bw_shader_set)
  self.cheat_flash = gm.method(self, cheat_flash)
  self.ui_blocking = gm.method(self, ui_blocking)
  self.pixels_to_cm = gm.method(self, pixels_to_cm)
  self.length_text = gm.method(self, length_text)
  self.mycelium_cm = gm.method(self, mycelium_cm)
  self.mycelium_add_length = gm.method(self, mycelium_add_length)
  self.food_add = gm.method(self, food_add)
  self.food_spend = gm.method(self, food_spend)
  self.reach_expand = gm.method(self, reach_expand)
  self.camera_bounds = gm.method(self, camera_bounds)
  self.camera_jump_to = gm.method(self, camera_jump_to)
  self.camera_jump_origin = gm.method(self, camera_jump_origin)
  self.mycelium_extreme_tip = gm.method(self, mycelium_extreme_tip)
  self.camera_jump_edge = gm.method(self, camera_jump_edge)
  self.tutorial_init = gm.method(self, tutorial_init)
  self.tutorial_event = gm.method(self, tutorial_event)
  self.tutorial_show = gm.method(self, tutorial_show)
  self.tutorial_hide = gm.method(self, tutorial_hide)
  self.tutorial_show_nearest = gm.method(self, tutorial_show_nearest)
  self.tutorial_wrap = gm.method(self, tutorial_wrap)
  self.tutorial_state = gm.method(self, tutorial_state)
  self.tutorial_step = gm.method(self, tutorial_step)
  self.tutorial_draw = gm.method(self, tutorial_draw)
  self.tasks_defs = gm.method(self, tasks_defs)
  self.tasks_init = gm.method(self, tasks_init)
  self.tasks_stat_add = gm.method(self, tasks_stat_add)
  self.tasks_met = gm.method(self, tasks_met)
  self.tasks_step = gm.method(self, tasks_step)
  self.tasks_block_open = gm.method(self, tasks_block_open)
  self.tasks_block_toggle = gm.method(self, tasks_block_toggle)
  self.tasks_block_items = gm.method(self, tasks_block_items)
  self.tasks_blocks = gm.method(self, tasks_blocks)
  self.tasks_header_rect = gm.method(self, tasks_header_rect)
  self.tasks_visible = gm.method(self, tasks_visible)
  self.tasks_mouse_step = gm.method(self, tasks_mouse_step)
  self.tasks_draw = gm.method(self, tasks_draw)
  self.tasks_block_draw = gm.method(self, tasks_block_draw)
  self.goal_spore_count = gm.method(self, goal_spore_count)
  self.goal_met = gm.method(self, goal_met)
  self.tasks_popup_open = gm.method(self, tasks_popup_open)
  self.goal_number_text = gm.method(self, goal_number_text)
  self.tasks_popup_rect = gm.method(self, tasks_popup_rect)
  self.tasks_popup_buttons = gm.method(self, tasks_popup_buttons)
  self.tasks_popup_close = gm.method(self, tasks_popup_close)
  self.tasks_popup_step = gm.method(self, tasks_popup_step)
  self.goal_text_scale = gm.method(self, goal_text_scale)
  self.goal_sprite_fit = gm.method(self, goal_sprite_fit)
  self.goal_sprite_part_fit = gm.method(self, goal_sprite_part_fit)
  self.goal_fit = gm.method(self, goal_fit)
  self.tasks_popup_draw = gm.method(self, tasks_popup_draw)
  self.cine_ball = gm.method(self, cine_ball)
  self.cine_active = gm.method(self, cine_active)
  self.cine_blocking = gm.method(self, cine_blocking)
  self.cine_showing = gm.method(self, cine_showing)
  self.cine_spore_spots = gm.method(self, cine_spore_spots)
  self.cine_show_start = gm.method(self, cine_show_start)
  self.cine_planet_start = gm.method(self, cine_planet_start)
  self.cine_shake = gm.method(self, cine_shake)
  self.cine_step = gm.method(self, cine_step)
  self.cine_span = gm.method(self, cine_span)
  self.cine_draw = gm.method(self, cine_draw)
  self.steam_step = gm.method(self, steam_step)
  self.ach_stat = gm.method(self, ach_stat)
  self.ach_standing = gm.method(self, ach_standing)
  self.ach_list = gm.method(self, ach_list)
  self.ach_step = gm.method(self, ach_step)
  self.display_init = gm.method(self, display_init)
  self.display_fit = gm.method(self, display_fit)
  self.display_apply = gm.method(self, display_apply)
  self.focus_step = gm.method(self, focus_step)
  self.art_preload_step = gm.method(self, art_preload_step)
  self.display_step = gm.method(self, display_step)
  self.display_window_size = gm.method(self, display_window_size)
  self.display_save = gm.method(self, display_save)
  self.display_set_fullscreen = gm.method(self, display_set_fullscreen)
  self.display_cycle_size = gm.method(self, display_cycle_size)
end

-- gml_Script_experience_price_mult @ 0x1400b1480
function experience_price_mult(self, other)
  local n = 0
  local i = 0
  while i < gm.array_length(global.mushrooms) do
    if global.mushrooms[i].type == 6 and not gm.bool(global.mushrooms[i].stolen) then
      n = n + 1
    end
    i = i + 1
  end
  return gm.power(bal(self, other, "experience_cost_growth"), n)
end

-- gml_Script_mushroom_type_cost_base @ 0x1400b1cf0
function mushroom_type_cost_base(self, other, t)
  if t == nil then t = gm.undefined end
  if t == 6 then
    return gm.round(bal(self, other, "experience_mushroom_cost") * experience_price_mult(self, other))
  elseif t == 7 then
    return bal(self, other, "consumptor_mushroom_cost")
  elseif t == 0 then
    return bal(self, other, "spore_mushroom_cost")
  elseif t == 1 then
    return bal(self, other, "predator_mushroom_cost")
  elseif t == 2 then
    return bal(self, other, "repeller_mushroom_cost")
  elseif t == 3 then
    return bal(self, other, "tree_mushroom_cost")
  elseif t == 4 then
    return bal(self, other, "digger_mushroom_cost")
  elseif t == 5 then
    return bal(self, other, "stinky_mushroom_cost")
  end
  return 0
end

-- gml_Script_mushroom_type_water_cost_base @ 0x1400b2aa0
function mushroom_type_water_cost_base(self, other, t)
  if t == nil then t = gm.undefined end
  if t == 0 then
    return bal(self, other, "spore_mushroom_water_cost")
  elseif t == 1 then
    return bal(self, other, "predator_mushroom_water_cost")
  elseif t == 2 then
    return bal(self, other, "repeller_mushroom_water_cost")
  elseif t == 3 then
    return bal(self, other, "tree_mushroom_water_cost")
  elseif t == 5 then
    return bal(self, other, "stinky_mushroom_water_cost")
  elseif t == 6 then
    return gm.round(bal(self, other, "experience_mushroom_water_cost") * experience_price_mult(self, other))
  elseif t == 7 then
    return bal(self, other, "consumptor_mushroom_water_cost")
  elseif t == 4 then
    local k = upg_level(self, other, "digger_length") / gm.max(1, upg_max_level(self, other, "digger_length"))
    return gm.round(gm.lerp(bal(self, other, "digger_water_cost_min"), bal(self, other, "digger_water_cost_max"), k))
  end
  return 0
end

-- gml_Script_mushroom_type_unlock_id @ 0x1400b3cf0
function mushroom_type_unlock_id(self, other, t)
  if t == nil then t = gm.undefined end
  if t == 0 then
    return "unlock_spore"
  elseif t == 1 then
    return "unlock_predator"
  elseif t == 2 then
    return "unlock_repeller"
  elseif t == 3 then
    return "unlock_tree"
  elseif t == 4 then
    return "unlock_digger"
  elseif t == 5 then
    return "unlock_stinky"
  elseif t == 6 then
    return "unlock_experience"
  elseif t == 7 then
    return "unlock_parasitism"
  end
  return ""
end

-- gml_Script_squirrel_grab @ 0x140115360
function squirrel_grab(self, other, sq)
  if sq == nil then sq = gm.undefined end
  local m = sq.mushroom
  if gm.bool(stinky_guards(self, other, m)) or gm.bool(perk_has(self, other, "red")) then
    if gm.bool(stinky_guards(self, other, m)) then
      tasks_stat_add(self, other, "ach_stinky_scares", gm.undefined)
    end
    sq.mushroom = gm.undefined
    sq.empty_run = true
    return
  end
  sq.timer = vis(self, other, "squirrel_pull_frames")
  sq.grab_x = m.x
  sq.grab_y = m.y
  m.stolen = true
end

-- gml_Script____struct___283__visual_defaults__scr_visual @ 0x1402a79e0
-- TODO(port): decompiler failed on this constructor; field list recovered from the disassembly
function ___struct___283__visual_defaults__scr_visual(self, other, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9, a10, a11, a12, a13, a14, a15, a16, a17, a18, a19, a20, a21, a22)
  if a0 == nil then a0 = gm.undefined end
  if a1 == nil then a1 = gm.undefined end
  if a2 == nil then a2 = gm.undefined end
  if a3 == nil then a3 = gm.undefined end
  if a4 == nil then a4 = gm.undefined end
  if a5 == nil then a5 = gm.undefined end
  if a6 == nil then a6 = gm.undefined end
  if a7 == nil then a7 = gm.undefined end
  if a8 == nil then a8 = gm.undefined end
  if a9 == nil then a9 = gm.undefined end
  if a10 == nil then a10 = gm.undefined end
  if a11 == nil then a11 = gm.undefined end
  if a12 == nil then a12 = gm.undefined end
  if a13 == nil then a13 = gm.undefined end
  if a14 == nil then a14 = gm.undefined end
  if a15 == nil then a15 = gm.undefined end
  if a16 == nil then a16 = gm.undefined end
  if a17 == nil then a17 = gm.undefined end
  if a18 == nil then a18 = gm.undefined end
  if a19 == nil then a19 = gm.undefined end
  if a20 == nil then a20 = gm.undefined end
  if a21 == nil then a21 = gm.undefined end
  if a22 == nil then a22 = gm.undefined end
  self.pause_on_focus_loss = 1
  self.screen_width_px = 1920
  self.screen_height_px = 1080
  self.tutorial_fade_frames = 30
  self.tutorial_tip_frames = 600
  self.tutorial_short_frames = 300
  self.tutorial_final_frames = 300
  self.tutorial_text_scale = 0.55
  self.tutorial_final_scale = 1.4
  self.tutorial_shoot_offset_y = 150
  self.tutorial_world_offset_y = 56
  self.tutorial_xp_cm = 1000
  self.tutorial_clay_seconds = 10
  self.tutorial_wrap_chars = 32
  self.tutorial_hand_fps = 20
  self.tutorial_hand_scale = 0.35
  self.tutorial_hand_offset_x = 48
  self.tutorial_hand_offset_y = 0
  self.tutorial_hand_fade_frames = 6
  self.tutorial_camera_scale = 1
  self.tutorial_camera_offset_y = 60
  self.tutorial_camera_fps = 10
  self.menu_vignette_strength = 0.73
  self.menu_vignette_shape = 4.1
  self.menu_vignette_start = 0.47
  self.menu_vignette_width = 0.66
  self.menu_vignette_power = 1.83
  self.strand_thickness = 2.56
  self.growth_duration_frames = 96
  self.growth_jitter = 0.65
  self.repeller_flash_frames = 40
  self.repeller_cloud_puffs = 480
  self.repeller_cloud_specks = 120
  self.repeller_cloud_dust = 60
  self.repeller_cloud_flash_frames = 12
  self.repeller_cloud_flash_px = 120
  self.repeller_cloud_flash_layers = 2
  self.repeller_cloud_pixel = 3
  self.repeller_cloud_kick_px = 3
  self.repeller_cloud_speed = 15
  self.repeller_cloud_drag = 0.9
  self.repeller_cloud_spread_deg = 70
  self.repeller_cloud_dy = 22
  self.repeller_cloud_size_min = 10
  self.repeller_cloud_size_max = 44
  self.repeller_cloud_life_min = 55
  self.repeller_cloud_life_max = 95
  self.repeller_cloud_alpha = 0.9
  self.digger_strand_width_mult = 1.35
  self.digger_fan_from = 185
  self.digger_fan_to = 355
  self.digger_fan_jitter = 4
  self.digger_root_depth = 6
  self.digger_kinks_per_base = 1.2
  self.digger_strands_budget_us = 3000
  self.strand_kink_min_deg = 12
  self.strand_kink_max_deg = 30
  self.strand_kink_cone_deg = 40
  self.tree_mushroom_offset_x = 26
  self.tree_mushroom_height_px = 210
  self.hedgehog_frame_interval = 4
  self.camera_shift_boost = 3
  self.bush_offset_y = 4
  self.plant_offset_y = 3
  self.decor_tree_offset_y = 3
  self.knot_scale = 1.9
  self.hedgehog_scale = a0
  self.animal_marker_radius = 46
  self.animal_marker_edge_px = 30
  self.animal_marker_fill = 0.85
  self.crow_warning_seconds = 5
  self.animal_marker_fade_seconds = 0.35
  self.animal_marker_follow_seconds = 3.5
  self.animal_marker_top_px = 130
  self.animal_marker_bottom_px = 200
  self.deep_stone_colour = a1
  self.stone_grey_amount = 0.75
  self.cave_scale = 1
  self.bones_tint_plain = a2
  self.bones_tint_clay = a3
  self.bones_tint_deep = a4
  self.debug_bones_nearest = 10
  self.cave_flash_ms = 1500
  self.cave_flash_alpha = 0.9
  self.cave_glow_alpha = 0.14
  self.cave_popup_ms = 4000
  self.cave_popup_rise_px = 70
  self.cave_popup_scale = 0.7
  self.cave_icon_px = 44
  self.cave_icon_smooth = true
  self.cave_icon_gap_px = 8
  self.cave_icons_x = 24
  self.cave_icons_y = 64
  self.tasks_x = 28
  self.tasks_y = 150
  self.tasks_title_scale = 0.7
  self.tasks_text_scale = 0.5
  self.tasks_first_gap = 28
  self.tasks_line_px = 22
  self.tasks_box_px = 14
  self.tasks_box_gap = 8
  self.tasks_reward_gap = 10
  self.tasks_reward_icon_px = 16
  self.tasks_arrow_px = 11
  self.tasks_arrow_gap = 10
  self.tasks_block_gap = 26
  self.migration_cine_seconds = 3.85
  self.migration_cine_ball_share = 0.1
  self.migration_cine_myc_from = 1
  self.migration_cine_myc_to = 2.45
  self.migration_cine_white_from = 2.25
  self.migration_cine_white_to = 3.25
  self.migration_spore_stops = 7
  self.migration_stop_seconds = 0.5
  self.migration_origin_seconds = 3.5
  self.migration_shot_delay_seconds = 0.05
  self.migration_burst_seconds = 0.3
  self.migration_camera_ground_share = 0.62
  self.migration_origin_spore_mult = 80
  self.migration_mega_rays = 30
  self.migration_mega_emit_seconds = 0.9
  self.migration_mega_speed = 30
  self.migration_mega_shake_max = 5
  self.migration_white_from = 1.2
  self.migration_white_clear_seconds = 0.5
  self.migration_shake_mult = 1
  self.migration_cine_fade_out_seconds = 0.5
  self.tasks_fade_frames = 40
  self.tasks_fold_frames = 12
  self.tasks_fill_frames = 24
  self.myc_glow_downscale = 2
  self.myc_glow_thicken = 1
  self.myc_glow_radius = 8
  self.myc_glow_passes = 2
  self.myc_glow_strength = 0.5
  self.myc_glow_colour = a5
  self.myc_pulse_interval_seconds = 20
  self.myc_pulse_speed_px = 900
  self.myc_pulse_band_px = 140
  self.myc_pulse_strength = 3
  self.myc_pulse_colour = a6
  self.cave_touch_px = 20
  self.cave_burst_per_frame = 2
  self.cave_burst_max_growing = 12
  self.cave_burst_instant_budget_us = 2500
  self.cave_wrap_out_px = 1
  self.cave_wrap_wobble_px = 2
  self.cave_wrap_ms = 2600
  self.cave_stem_start = 0.6
  self.cave_stem_ms = 1400
  self.cave_stem_sway_px = 4
  self.cave_wrap_core_colour = a7
  self.cave_wrap_halo_colour = a8
  self.cave_wrap_core_px = 2
  self.cave_wrap_halo_px = 5
  self.cave_branch_every = 9
  self.cave_branch_min_px = 18
  self.cave_branch_max_px = 55
  self.cave_branch_spread_deg = 55
  self.cave_branch_fork_chance = 0.35
  self.cave_branch_grow = 30
  self.ocean_deep_colour = a9
  self.ocean_fade_px = 420
  self.ocean_flare_cell_px = 150
  self.ocean_flare_chance = 0.45
  self.ocean_flare_top_gap_px = 50
  self.water_dip_hold_frames = 30
  self.water_dip_refill_keep = 0.93
  self.hedgehog_back_smooth_cols = 4
  self.hedgehog_back_follow_cols = 6
  self.hedgehog_carry_sink_px = 18
  self.hedgehog_carry_back_px = 30
  self.hedgehog_carry_angle = 50
  self.hedgehog_carry_scale = 0.85
  self.boar_scale = 1.3
  self.boar_frame_interval = 3
  self.boar_snout_dx = 0.42
  self.boar_shake_px = 5
  self.boar_shake_falloff_px = 900
  self.boar_tremor_volume = 1
  self.boar_tremor_reach_views = 1
  self.boar_tremor_fade_in_frames = 45
  self.boar_tremor_fade_out_frames = 60
  self.boar_fling_vx_min = 4
  self.boar_fling_vx_max = 8
  self.boar_fling_vy_min = 5
  self.boar_fling_vy_max = 9
  self.boar_fling_gravity = 0.35
  self.boar_fling_spin_min = 6
  self.boar_fling_spin_max = 16
  self.boar_fling_frames = 55
  self.consumptor_tree_foot_gap_px = 16
  self.experience_flash_frames = 24
  self.hover_bar_w = 40
  self.hover_bar_h = 5
  self.hover_bar_gap = 8
  self.hover_bar_back_alpha = 0.6
  self.hover_leaf_radius_px = 24
  self.hover_worm_radius_px = 14
  self.crow_scale = 1
  self.crow_frame_interval = 4
  self.crow_grab_frames = 10
  self.crow_dive_dx_px = 520
  self.crow_grab_into_cap_px = 6
  self.crow_climb_ease_frames = 20
  self.crow_climb_forward = 0.9
  self.crow_body_up_px = 30
  self.crow_body_pad_px = 26
  self.crow_flee_start_speed = 0.6
  self.crow_flee_forward = 0.35
  self.stinky_cone_deg_levels = a10
  self.stinky_beam_height_px = 540
  self.stinky_base_radius_levels = a11
  self.stinky_flare_power_levels = a12
  self.stinky_emit_per_frame = 1.1
  self.stinky_full_columns = 4
  self.stinky_emit_into_cap_px = 6
  self.stinky_rise_min = 0.35
  self.stinky_rise_max = 0.85
  self.stinky_lift = 0.0015
  self.stinky_sway_px = 0.3
  self.stinky_wind_push = 0.14
  self.stinky_wind_ease = 0.012
  self.stinky_alpha = 0.9
  self.stinky_glow_alpha = 0.07
  self.stinky_glow_radius_px = 3
  self.stinky_fade_in_frames = 18
  self.stinky_top_fade_px = 140
  self.stinky_hot_r = 255
  self.stinky_hot_g = 58
  self.stinky_hot_b = 42
  self.stinky_deep_r = 128
  self.stinky_deep_g = 12
  self.stinky_deep_b = 22
  self.squirrel_frame_interval = 3
  self.squirrel_scale = a13
  self.squirrel_mouth_x = 34
  self.squirrel_mouth_y = 138
  self.squirrel_grip_x = 72
  self.squirrel_body_x = 51
  self.squirrel_body_y = 95
  self.squirrel_tail_tip_y = 13
  self.squirrel_turn_frames = 10
  self.squirrel_run_body_dy = a14
  self.squirrel_jump_gap = 55
  self.squirrel_jump_dx = 40
  self.squirrel_edge_smooth_rows = 50
  self.squirrel_bite_dy = 0
  self.squirrel_trunk_inset = 10
  self.squirrel_shroom_bite_x = a15
  self.squirrel_shroom_bite_y = a16
  self.squirrel_consumptor_bite_x = 0
  self.squirrel_consumptor_bite_y = a17
  self.squirrel_consumptor_bite_dy = a18
  self.squirrel_hold_into_head_x = 4
  self.squirrel_hold_into_head_y = a19
  self.squirrel_carry_angle = 90
  self.squirrel_carry_scale = 0.7
  self.squirrel_carry_forward = 0.5
  self.squirrel_pull_frames = 8
  self.intro_hedgehog_speed = 4.5
  self.intro_drop_gravity = 0.55
  self.leaf_fall_speed = 1.25
  self.leaf_sway = 0.9
  self.leaf_sway_speed = 3.4
  self.leaf_wind_push = 0.28
  self.leaf_wind_ease = 0.05
  self.leaf_spin_degrees = 22
  self.leaf_spin_ease = 0.18
  self.leaf_change_orbit_scale = 3
  self.leaf_turn_frame_interval = 5
  self.leaf_turn_delay_min = 30
  self.leaf_turn_delay_max = 260
  self.leaf_change_frame_interval = 5
  self.leaf_face_flip = 0
  self.leaf_idle_life = 240
  self.leaf_tendril_grow_frames = 120
  self.leaf_tendril_wrap_turns = 1.5
  self.leaf_tendril_retract_frames = 45
  self.worm_length_px = 32
  self.worm_thickness_px = 4
  self.worm_segments = 12
  self.worm_colour_r = 236
  self.worm_colour_g = 188
  self.worm_colour_b = 194
  self.worm_stretch = 0.14
  self.worm_wave_speed = 0.09
  self.worm_struggle_frames = 150
  self.worm_tendril_wraps = 4
  self.worm_trail_spacing_px = 2
  self.worm_trail_frames = 900
  self.worm_trail_alpha = 0.5
  self.worm_trail_max = 8000
  self.worm_trail_colour_r = 34
  self.worm_trail_colour_g = 24
  self.worm_trail_colour_b = 20
  self.backdrop_leaf_colour = a20
  self.backdrop_leaf_far_count = 16
  self.backdrop_leaf_near_count = 11
  self.backdrop_leaf_strong_wind_mult = 2
  self.backdrop_leaf_far_parallax = 0.3
  self.backdrop_leaf_near_parallax = 0.42
  self.backdrop_leaf_far_scale = 0.22
  self.backdrop_leaf_near_scale = 0.34
  self.backdrop_leaf_far_alpha = 0.65
  self.backdrop_leaf_near_alpha = 0.85
  self.backdrop_leaf_fall_min = 0.47
  self.backdrop_leaf_fall_max = 1.07
  self.backdrop_leaf_sway = 0.35
  self.backdrop_leaf_wind_push = 0.35
  self.backdrop_leaf_wind_ease = 0.03
  self.wind_change_interval = 420
  self.wind_min_strength = 1
  self.wind_max_strength = 3
  self.rain_first_gap_frames = 7200
  self.rain_gap_min_frames = 7200
  self.rain_gap_max_frames = 14400
  self.rain_fade_in_frames = 300
  self.rain_hold_frames = 1800
  self.rain_fade_out_frames = 480
  self.rain_sound_volume = 1
  self.forest_volume = 1
  self.forest_rain_level = 0.3
  self.forest_duck_frames = 300
  self.ambience_pan = 0.4
  self.ambience_bird_min_sec = 60
  self.ambience_bird_max_sec = 120
  self.ambience_bird_volume = 1
  self.ambience_creak_min_sec = 60
  self.ambience_creak_max_sec = 120
  self.ambience_creak_volume = 1
  self.ambience_woodpecker_min_sec = 60
  self.ambience_woodpecker_max_sec = 120
  self.ambience_woodpecker_volume = 1
  self.ambience_leaves_min_sec = 90
  self.ambience_leaves_max_sec = 150
  self.ambience_leaves_volume = 1
  self.sound_camera_fade_views = 0.5
  self.sound_world_reach_views = 1
  self.sound_pan_edge = 0.4
  self.sound_pan_far = 0.85
  self.explosion_volume = 1
  self.mush_grow_volume = 1
  self.smallshroom_grow_volume = 1
  self.water_touch_volume = 1
  self.water_touch_cooldown_frames = 30
  self.hyphae_volume = 1
  self.hyphae_middle_volume = 0.6
  self.hyphae_taper_from = 0.6
  self.hyphae_taper_to = 0.5
  self.hyphae_end_at = 0.85
  self.hyphae_middle_fade_frames = 21
  self.hyphae_loop_start = 0.2
  self.hyphae_loop_tail = 0.43
  self.hyphae_len_tiny = 80
  self.hyphae_len_mid = 128
  self.hyphae_len_long = 224
  self.rain_max_drops = 560
  self.rain_max_splashes = 110
  self.rain_max_spray = 140
  self.rain_splash_chance = 0.45
  self.rain_wind_push = 0.9
  self.rain_gloom_lead_frames = 660
  self.rain_gloom_fade_frames = 270
  self.rain_gloom_strength = 0.8
  self.food_shade = 0.79
  self.food_glow_colour = a21
  self.food_glow_strength = 0.43
  self.food_glow_inner = 0.18
  self.food_glow_pulse_low = 0.25
  self.food_glow_pulse_high = 1.25
  self.food_glow_pulse_seconds = 10
  self.food_glow_radius = 3.5
  self.food_glow_downscale = 4
  self.food_glow_swell = 1.1
  self.water_flare_fps = 5
  self.water_flare_fade_in_seconds = 5
  self.water_flare_hold_seconds = 10
  self.water_flare_fade_out_seconds = 5
  self.water_flares_small = 2
  self.water_flares_medium = 3
  self.water_flares_big = 5
  self.water_edge_surface_row = 3
  self.water_reservoir_colour = a22
  self.spore_shroom_scale = 1.5
  self.shroom_grow_ms = 1000
  self.tree_layout_scale_x = 1.95
  self.tree_layout_scale_y = 1.85
  self.tree_name_width = 220
  self.tree_name_scale = 2.415
  self.tree_cost_scale = 1.7
  self.tree_label_back_alpha = 0.85
  self.tree_node_radius = 34
  self.tree_unlock_radius_mult = 1.6
  self.tree_root_radius_mult = 2.3
  self.tree_shroom_icon_fit = 1.5
  self.build_consumptor_icon_scale = 0.85
  self.tree_digger_icon_scale = 0.9
  self.tree_strand_wobble = 30
  self.tree_grow_speed = 5
  self.tree_hypha_width = 5
  self.tree_hypha_wander_max = 22
  self.tree_hypha_side_max = 110
  self.tree_zoom_min = 0.25
  self.tree_zoom_out_past_fit = 0.7
  self.tree_zoom_max = 1.4
  self.tree_name_scale_min = 0.966
  self.tree_cost_scale_min = 0.7
  self.tree_trunk_scale_min = 0.8
  self.tree_zoom_step = 0.12
  self.tree_pan_speed = 22
  self.logo_pen_angle = 40
  self.logo_pen_contrast = 0.07
  self.logo_nib = 0.105
  self.logo_glyph_width = 1
  self.logo_letter_advance = 0.94
  self.logo_collar = 1
  self.logo_collar_run = 0.3
  self.logo_min_pixels = 0.55
  self.logo_arch = 0.24
  self.logo_tilt = 8
  self.logo_size_spread = 0.4
  self.logo_bridges = 1
  self.logo_outline_columns = 22
  self.logo_edge_tolerance = 1.4
  self.logo_corner_turn = 75
  self.logo_corner_leg = 0.22
  self.logo_drip_chance = 0.45
  self.logo_blade_flare = 0.3
  self.logo_strand_jitter = 1.5
  self.logo_max_blades = 3000
  self.logo_thorn_width = 0.8
  self.logo_thorn_curl = 6
  self.logo_thorn_depth = 2
  self.logo_thorn_forks = 1
  self.logo_web_per_letter = 3
  self.logo_web_width = 0.2
  self.logo_web_length = 0.75
  self.logo_web_curl = 9
  self.logo_web_depth = 4
  self.logo_web_forks = 2
  self.logo_web_inner_length = 0.34
  self.logo_web_inner_density = 1.2
  self.logo_reach_base = 0.24
  self.logo_reach_wing = 2.2
  self.logo_reach_drip = 1.3
  self.logo_reach_top = 0.5
  self.logo_axis_reach = 1.5
  self.logo_grow_frames = 170
  self.bw_glow_strength = 0.45
  self.bw_glow_halo = 0.5
  self.bw_glow_threshold = 0.5
  self.glow_downscale = 6
  self.glow_wide = 3
  self.bw_gain = 1.15
  self.bw_pivot = 0.38
  self.bw_contrast = 1.45
  self.bw_gamma = 0.95
  self.roots_offset_y = 0
  self.roots_scale_big = 1
  self.roots_scale_small = 1
  self.stump_offset_y = 0
  self.volume_sfx = 0.8
  self.volume_music = 0.55
  self.camera_speed = 18
  self.camera_accel = 0.1
  self.camera_edge_margin_px = 40
  self.leaf_gust_min = 4
  self.leaf_gust_max = 7
  self.leaf_gust_spread_px = 160
  self.leaf_gust_height_px = 260
  self.boar_pack_size = 3
  self.boar_pack_gap_seconds = 0.5
  self.build_key_badge_radius = 15
  self.build_key_badge_gap = 6
  self.build_key_badge_text = 1
  self.camera_fly_ease = 0.14
  self.camera_fly_seconds = 2.5
  self.camera_reach_margin_px = 640
end

-- gml_Script_visual_defaults @ 0x1402af750
function visual_defaults(self, other)
  return gm.new(___struct___283__visual_defaults__scr_visual,
    0.6666666666666666,
    3552314,
    13490913,
    13946312,
    10200510,
    13497790,
    16735408,
    14873582,
    10534576,
    5128230,
    gm.array(13, 18.5, 24, 29, 33),
    gm.array(7, 7, 7, 9, 38),
    gm.array(1, 0.7, 0.45, 0.45, 0.45),
    0.5882352941176471,
    -6,
    -28,
    -2,
    -28,
    -28,
    -5,
    3222551,
    14086376,
    8615762)
end

-- gml_Script_visual_init @ 0x1402b00b0
function visual_init(self, other)
  global.visual = visual_defaults(self, other)
end

-- gml_Script_vis @ 0x1402b01e0
function vis(self, other, key)
  if key == nil then key = gm.undefined end
  return gm.variable_struct_get(global.visual, key)
end

-- gml_GlobalScript_scr_visual @ 0x1402b03e0
function gml_GlobalScript_scr_visual(self, other)
  self.visual_defaults = gm.method(self, visual_defaults)
  self.visual_init = gm.method(self, visual_init)
  self.vis = gm.method(self, vis)
end

-- gml_Script____struct___366__worldgen_defaults__scr_worldgen @ 0x1403871a0
function ___struct___366__worldgen_defaults__scr_worldgen(self, other, a0, a1, a2, a3, a4, a5, a6, a7, a8, a9)
  if a0 == nil then a0 = gm.undefined end
  if a1 == nil then a1 = gm.undefined end
  if a2 == nil then a2 = gm.undefined end
  if a3 == nil then a3 = gm.undefined end
  if a4 == nil then a4 = gm.undefined end
  if a5 == nil then a5 = gm.undefined end
  if a6 == nil then a6 = gm.undefined end
  if a7 == nil then a7 = gm.undefined end
  if a8 == nil then a8 = gm.undefined end
  if a9 == nil then a9 = gm.undefined end
  self.surface_height_px = 480
  self.reference_tile_size = 64
  self.underground_screen_depth = 3
  self.bottom_water_layer_tiles = 3
  self.water_per_tile = 200
  self.chunk_cols = 16
  self.soil_first_rows = 2
  self.soil_band_rows = 3
  self.dense_soil_depth_screens = 3
  self.dense_food_multiplier = 2
  self.deep_soil_depth_screens = 6
  self.deep_food_multiplier = 3
  self.deep_food_clump_chance = 0.8
  self.deep_food_clump_max_extra = 14
  self.deep_food_clump_spread_tiles = 4.5
  self.ocean_after_deep_screens = 4
  self.bottom_rock = 1
  self.survival_resource_share = 0.5
  self.survival_full_zone_x_screens = 0.5
  self.survival_full_zone_depth_screens = 1
  self.ocean_clear_margin_px = 24
  self.turf_offset_y = 15
  self.dirt_spacing_px = 96
  self.dirt_chance = 0.7
  self.stone_spacing_px = 140
  self.stone_chance = 0.65
  self.stone_shove_power = 1.4
  self.stone_shove_reach = 6
  self.stone_shove_lead = 0.35
  self.stone_shove_spin = 0.06
  self.stone_shove_wobble = 0.5
  self.stone_drag = 0.86
  self.stone_pass_on = 0.35
  self.stone_deposits_enabled = 1
  self.stone_deposit_tries = 8
  self.stone_deposit_chance = 0.7
  self.stone_deposit_min_radius_cells = 1
  self.stone_deposit_max_radius_cells = 7
  self.stone_deposit_size_bias = 2
  self.stone_deposit_min_lobes = 2
  self.stone_deposit_max_lobes = 5
  self.stone_deposit_nibble_chance = 0.3
  self.stone_deposit_grow_chance = 0.22
  self.stone_deposit_gap_cells = 1
  self.stone_water_clearance_px = 48
  self.stone_deposit_deep_mult = 7
  self.stone_deposit_deep_chance = 0.9
  self.stone_deposit_deep_max_radius_cells = 9
  self.surface_obstacles_enabled = 1
  self.surface_obstacle_gap_min_px = 1500
  self.surface_obstacle_gap_max_px = 5500
  self.maze_every = 3
  self.maze_min_width_screens = 1
  self.maze_max_width_screens = 3
  self.maze_extra_entrances_min = 2
  self.maze_extra_entrances_max = 4
  self.maze_branches_min = 3
  self.maze_branches_max = 6
  self.maze_join_chance = 0.35
  self.maze_chamber_chance = 0.6
  self.maze_max_chambers = 4
  self.maze_water_share = 0.4
  self.maze_food_min = 14
  self.maze_food_max = 20
  self.maze_water_amount = 100
  self.maze_tunnel_min_cells = 1
  self.maze_tunnel_max_cells = 2
  self.maze_bulge_chance = 0.025
  self.maze_solid_max_cells = 6
  self.maze_hollow_food_min = 3
  self.maze_hollow_food_max = 6
  self.maze_hollow_water_share = 0.15
  self.surface_wall_min_start_px = 1600
  self.surface_wall_min_width_cells = 5
  self.surface_wall_max_width_cells = 9
  self.surface_wall_depth_screens = 3
  self.surface_wall_side_bite_chance = 0.35
  self.surface_wall_min_deep_width_cells = 4
  self.surface_wall_lake_tunnel_rows = 3
  self.stones_enabled = 0
  self.stone_cluster_frequency = 0.22
  self.stone_cluster_min_radius = 2
  self.stone_cluster_max_radius = 4
  self.stone_pillar_chance = 0.12
  self.water_lake_tries = 5
  self.water_lake_chance = 0.6
  self.water_lake_gap = 24
  self.water_lake_big_chance_top = 0.05
  self.water_lake_big_chance_bottom = 0.6
  self.water_lake_small_chance_top = 0.6
  self.water_lake_small_chance_bottom = 0.1
  self.water_lake_small_amount = 25
  self.water_lake_medium_amount = 67
  self.water_lake_big_amount = 150
  self.food_spawn_near_min_px = 200
  self.food_spawn_near_max_px = 440
  self.food_distance_rarity_px = 16000
  self.food_clump_chance = 0.45
  self.start_screen_food_min = 65
  self.start_screen_food_max = 70
  self.start_side_screen_food_min = 35
  self.food_clump_max_extra = 9
  self.caves_enabled = 1
  self.caves_clearance_px = 72
  self.caves_clear_deposit_max_cells = 60
  self.caves_host_reach_screens = 40
  self.caves_margin_px = 80
  self.caves_blue_first_x = a0
  self.caves_blue_depth = a1
  self.caves_blue_more = 0
  self.caves_blue_every_x = 10
  self.caves_blue_gap_growth = 1.25
  self.caves_blue_jitter = 0.2
  self.caves_extend_ahead_screens = 6
  self.caves_purple_x = a2
  self.caves_purple_depth = a3
  self.caves_clay_x = a4
  self.caves_clay_depth = a5
  self.caves_yellow_after_minutes = 120
  self.caves_yellow_beyond_x = a6
  self.caves_yellow_depth = a7
  self.rock_clusters_enabled = 1
  self.rock_cluster_every_x = 4.5
  self.rock_cluster_deep_every_x = 4.5
  self.rock_cluster_jitter_x = 1.2
  self.rock_cluster_per_side = 10
  self.rock_cluster_min_cells = 20
  self.rock_cluster_max_cells = 28
  self.bones_tries_per_layer = 4
  self.bones_chance = 0.22
  self.bones_big_x = a8
  self.bones_big_depth = a9
  self.bush_chance = 0.7
  self.bush_spacing_min_px = 180
  self.bush_spacing_max_px = 520
  self.plant_chance = 0.75
  self.plant_spacing_min_px = 40
  self.plant_spacing_max_px = 170
  self.decor_tree_chance = 0.45
  self.decor_tree_spacing_min_px = 350
  self.decor_tree_spacing_max_px = 900
  self.start_tree_clear_px = 100
  self.stump_chunk_chance = 0.3
  self.stump_tree_gap_px = 30
  self.start_resource_min_shots = 2
  self.start_resource_max_shots = 3
end

-- gml_Script_worldgen_defaults @ 0x1403898b0
function worldgen_defaults(self, other)
  return gm.new(___struct___366__worldgen_defaults__scr_worldgen,
    gm.array(2.38, 3.74), gm.array(0.6, 2.3), gm.array(4.76, 6.8), gm.array(1.2, 2.5), gm.array(1.36, 5.95),
    gm.array(3.6, 5.4), gm.array(3, 5), gm.array(6.8, 9.3), gm.array(6, 10), gm.array(7.2, 9.2))
end

-- gml_Script_worldgen_init @ 0x14038a070
function worldgen_init(self, other)
  global.worldgen = worldgen_defaults(self, other)
end

-- gml_Script_wld @ 0x14038a1a0
function wld(self, other, key)
  if key == nil then key = gm.undefined end
  return gm.variable_struct_get(global.worldgen, key)
end

-- gml_GlobalScript_scr_worldgen @ 0x14038a3a0
function gml_GlobalScript_scr_worldgen(self, other)
  self.worldgen_defaults = gm.method(self, worldgen_defaults)
  self.worldgen_init = gm.method(self, worldgen_init)
  self.wld = gm.method(self, wld)
end

local SteamId_static = {}
-- gml_Script_SteamId @ 0x14038a5c0
function SteamId(self, other)
  if SteamId_static.__uid == nil then
    SteamId_static.__uid = 3945585790
  end
  self.id64 = gm.undefined
  self.account_id = gm.undefined
  self.account_instance = gm.undefined
  self.universe = gm.undefined
  self.account_type = gm.undefined
  self.is_valid = gm.undefined
  self.is_lobby = gm.undefined
  self.is_individual = gm.undefined
  self.is_game_server = gm.undefined
  self.is_anon_game_server = gm.undefined
  self.is_anon_user = gm.undefined
  self.is_content_server = gm.undefined
  self.is_clan = gm.undefined
  self.is_chat = gm.undefined
end

local SteamFriendsGetFollowerCountResult_static = {}
-- gml_Script_SteamFriendsGetFollowerCountResult @ 0x14038ac80
function SteamFriendsGetFollowerCountResult(self, other)
  if SteamFriendsGetFollowerCountResult_static.__uid == nil then
    SteamFriendsGetFollowerCountResult_static.__uid = 760337428
  end
  self.result = gm.undefined
  self.steam_id = gm.undefined
  self.count = gm.undefined
end

local SteamFriendsIsFollowingResult_static = {}
-- gml_Script_SteamFriendsIsFollowingResult @ 0x14038af10
function SteamFriendsIsFollowingResult(self, other)
  if SteamFriendsIsFollowingResult_static.__uid == nil then
    SteamFriendsIsFollowingResult_static.__uid = 499178470
  end
  self.result = gm.undefined
  self.steam_id = gm.undefined
  self.is_following = gm.undefined
end

local SteamFriendsEnumerateFollowingListResult_static = {}
-- gml_Script_SteamFriendsEnumerateFollowingListResult @ 0x14038b1a0
function SteamFriendsEnumerateFollowingListResult(self, other)
  if SteamFriendsEnumerateFollowingListResult_static.__uid == nil then
    SteamFriendsEnumerateFollowingListResult_static.__uid = 2082408288
  end
  self.result = gm.undefined
  self.steam_ids = gm.undefined
  self.total_result_count = gm.undefined
end

local SteamFriendsRequestClanOfficerListResult_static = {}
-- gml_Script_SteamFriendsRequestClanOfficerListResult @ 0x14038b430
function SteamFriendsRequestClanOfficerListResult(self, other)
  if SteamFriendsRequestClanOfficerListResult_static.__uid == nil then
    SteamFriendsRequestClanOfficerListResult_static.__uid = 3581036
  end
  self.success = gm.undefined
  self.clan_id = gm.undefined
  self.officers = gm.undefined
end

local SteamFriendsDownloadClanActivityCountsResult_static = {}
-- gml_Script_SteamFriendsDownloadClanActivityCountsResult @ 0x14038b6c0
function SteamFriendsDownloadClanActivityCountsResult(self, other)
  if SteamFriendsDownloadClanActivityCountsResult_static.__uid == nil then
    SteamFriendsDownloadClanActivityCountsResult_static.__uid = 2553683852
  end
  self.result = gm.undefined
end

local SteamFriendsAvatarImageLoaded_static = {}
-- gml_Script_SteamFriendsAvatarImageLoaded @ 0x14038b880
function SteamFriendsAvatarImageLoaded(self, other)
  if SteamFriendsAvatarImageLoaded_static.__uid == nil then
    SteamFriendsAvatarImageLoaded_static.__uid = 3288597523
  end
  self.steam_id_64 = gm.undefined
  self.image_handle = gm.undefined
  self.width = gm.undefined
  self.height = gm.undefined
end

local SteamFriendsClanActivityCounts_static = {}
-- gml_Script_SteamFriendsClanActivityCounts @ 0x14038bb80
function SteamFriendsClanActivityCounts(self, other)
  if SteamFriendsClanActivityCounts_static.__uid == nil then
    SteamFriendsClanActivityCounts_static.__uid = 3710411397
  end
  self.online = gm.undefined
  self.in_game = gm.undefined
  self.chatting = gm.undefined
end

-- gml_Object_obj_game_Create_0 @ 0x1405f8800
function gml_Object_obj_game_Create_0(self, other)
  gm.application_surface_draw_enable(false)
  game_init(self, other)
  audio_init(self, other)
end

-- gml_Object_obj_game_Step_0 @ 0x1405f89c0
function gml_Object_obj_game_Step_0(self, other)
  game_step(self, other)
  save_step(self, other)
  display_step(self, other)
  focus_step(self, other)
  audio_step(self, other)
  cheats_step(self, other)
  steam_step(self, other)
end

-- gml_Object_obj_game_Other_3 @ 0x1405f8c00
function gml_Object_obj_game_Other_3(self, other)
  save_game(self, other)
end

-- gml_Object_obj_game_Draw_74 @ 0x1405f8c80
function gml_Object_obj_game_Draw_74(self, other)
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  global.gui_surface = surface_fit(self, other, global.gui_surface, gw, gh)
  global.gui_surface_on = gm.surface_exists(global.gui_surface)
  if gm.bool(global.gui_surface_on) then
    gm.surface_set_target(global.gui_surface)
    gm.draw_clear_alpha(0, 0)
  end
end

-- gml_Object_obj_game_Draw_75 @ 0x1405f9170
function gml_Object_obj_game_Draw_75(self, other)
  if not gm.bool(gm.variable_global_exists("gui_surface_on")) or not gm.bool(global.gui_surface_on) then
    return
  end
  gm.surface_reset_target()
  global.gui_surface_on = false
  if not gm.bool(gm.surface_exists(global.gui_surface)) then
    return
  end
  if gm.instance_exists(obj_mycelium_controller) then
    gm.with(self, obj_mycelium_controller, function(self, other)
      self.myc_glow_composite(self, other)
    end)
  end
  post_layer_draw(self, other, global.gui_surface, 0, 0, gm.display_get_gui_width(), gm.display_get_gui_height())
end

-- gml_Object_obj_game_Draw_77 @ 0x1405f9830
function gml_Object_obj_game_Draw_77(self, other)
  art_preload_step(self, other)
  if not gm.bool(gm.surface_exists(gm.application_surface)) then
    return
  end
  gm.draw_clear(gm.c_black)
  local pos = gm.application_get_position()
  local x1 = pos[0]
  local y1 = pos[1]
  local w = pos[2] - pos[0]
  local h = pos[3] - pos[1]
  post_layer_draw(self, other, gm.application_surface, x1, y1, w, h)
end

-- gml_Object_obj_gm_button_Create_0 @ 0x1405f9f90
function gml_Object_obj_gm_button_Create_0(self, other)
  gm.event_inherited(self, other)
  self.font = fnt_gm_20
  self.locked = false
  self.clicked = false
  self.halign = 1
  self.valign = 1
end

-- gml_Object_obj_gm_button_Step_0 @ 0x1405fa1c0
function gml_Object_obj_gm_button_Step_0(self, other)
  if gm.bool(self.locked) then
    self.image_index = 2
    return
  end
  if gm.bool(gm.point_in_rectangle(gm.mouse_x, gm.mouse_y, self.bbox_left, self.bbox_top, self.bbox_right, self.bbox_bottom)) then
    self.image_index = 1
    if gm.bool(gm.mouse_check_button_pressed(gm.mb_left)) then
      self.image_index = 0
    end
  else
    self.image_index = 0
  end
end

-- gml_Object_obj_gm_button1_Create_0 @ 0x1405fa870
function gml_Object_obj_gm_button1_Create_0(self, other)
  gm.event_inherited(self, other)
  self.font = fnt_gm_20
  self.locked = false
  self.clicked = false
  self.halign = 1
  self.valign = 1
end

-- gml_Object_obj_gm_button1_Step_0 @ 0x1405faaa0
function gml_Object_obj_gm_button1_Step_0(self, other)
  if gm.bool(self.locked) then
    self.image_index = 2
    return
  end
  if gm.bool(gm.point_in_rectangle(gm.mouse_x, gm.mouse_y, self.bbox_left, self.bbox_top, self.bbox_right, self.bbox_bottom)) then
    self.image_index = 1
    if gm.bool(gm.mouse_check_button_pressed(gm.mb_left)) then
      self.image_index = 0
    end
  else
    self.image_index = 0
  end
end

-- gml_Object_obj_gm_text_Create_0 @ 0x1405fb150
function gml_Object_obj_gm_text_Create_0(self, other)
  self.text = ""
  self.font = fnt_gm_15
  self.color = 16777215
  self.halign = 0
  self.valign = 0
end

-- gml_Object_obj_gm_text_Draw_0 @ 0x1405fb330
function gml_Object_obj_gm_text_Draw_0(self, other)
  gm.draw_set_font(self.font)
  gm.draw_set_colour(self.color)
  gm.draw_set_halign(self.halign)
  gm.draw_set_valign(self.valign)
  if gm.bool(gm.sprite_exists(self.sprite_index)) then
    gm.draw_self(self)
  end
  gm.draw_text(self.x, self.y, self.text)
end

-- gml_Object_obj_gm_text1_Create_0 @ 0x1405fb7c0
function gml_Object_obj_gm_text1_Create_0(self, other)
  self.text = ""
  self.font = fnt_gm_15
  self.color = 16777215
  self.halign = 0
  self.valign = 0
end

-- gml_Object_obj_gm_text1_Draw_0 @ 0x1405fb9a0
function gml_Object_obj_gm_text1_Draw_0(self, other)
  gm.draw_set_font(self.font)
  gm.draw_set_colour(self.color)
  gm.draw_set_halign(self.halign)
  gm.draw_set_valign(self.valign)
  if gm.bool(gm.sprite_exists(self.sprite_index)) then
    gm.draw_self(self)
  end
  gm.draw_text(self.x, self.y, self.text)
end

-- gml_Object_obj_gm_textbox_Draw_0 @ 0x1405fbe30
function gml_Object_obj_gm_textbox_Draw_0(self, other)
  gm.draw_set_valign(gm.fa_top)
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_font(self.font)
  gm.draw_set_colour(self.color)
  gm.draw_self(self)
  gm.draw_text_ext(self.bbox_left + 10, self.bbox_top + 10, self.text, gm.font_get_size(self.font), self.sprite_width - 20)
end

-- gml_Object_obj_life_Create_0 @ 0x14064c670
function gml_Object_obj_life_Create_0(self, other)
  self.depth = -10
  life_init(self, other)
end

-- gml_Object_obj_life_Step_0 @ 0x14064c790
function gml_Object_obj_life_Step_0(self, other)
  if gm.bool(intro_step(self, other)) then
    return
  end
  if gm.bool(ui_blocking(self, other)) then
    return
  end
  local t0 = gm.get_timer()
  life_step(self, other)
  global.prof_life_us = gm.get_timer() - t0
end

-- gml_Object_obj_life_Draw_0 @ 0x14064cab0
function gml_Object_obj_life_Draw_0(self, other)
  life_draw(self, other)
end
