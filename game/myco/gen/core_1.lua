local gm = require("gm")

-- gml_Script____struct___8__balance_defaults__scr_balance @ 0x1400459b0
function ___struct___8__balance_defaults__scr_balance(self, other)
  self.demo_upgrades_only = 0
  self.save_autosave_seconds = 180.0
  self.start_food = 50.0
  self.start_water = 80.0
  self.pixels_per_cm = 3.2
  self.xp_length_step_cm = 1000.0
  self.base_shot_range = 256.0
  self.base_shot_spread = 1.0
  self.shot_pull_scale = 1.0
  self.base_shot_water_cost = 5.0
  self.base_node_food_cost = 5.0
  self.base_random_node_chance = 0
  self.growth_min_length = 48.0
  self.growth_side_branches = 4.0
  self.base_hardsoil_penetration = 0
  self.hardsoil_speed_multiplier = 0.35
  self.hardsoil_water_multiplier = 2.0
  self.deepsoil_speed_multiplier = 0.2
  self.deepsoil_water_multiplier = 5.0
  self.deepsoil_shot_food_cost = 10.0
  self.perk_green_discount = 0.25
  self.perk_yellow_time_cut = 0.5
  self.task_reward_points = 2.0
  self.migration_spore_mushrooms = 100.0
  self.migration_food = 10000.0
  self.migration_water = 10000.0
  self.stone_passable = 0
  self.surface_build_band_px = 40.0
  self.mushroom_build_reach_px = 110.0
  self.mushroom_min_gap_px = 84.0
  self.spore_mushroom_cost = 10.0
  self.spore_mushroom_water_cost = 10.0
  self.spore_release_cost = 10.0
  self.spore_base_count = 28.0
  self.spore_base_weight = 1.0
  self.spore_base_node_chance = 0.16
  self.spore_shroom_gap_px = 14.0
  self.predator_mushroom_cost = 10.0
  self.predator_mushroom_water_cost = 25.0
  self.predator_food_per_fly = 3.0
  self.predator_fly_orbit_count = 3.0
  self.predator_catch_interval = 8400.0
  self.repeller_mushroom_cost = 15.0
  self.repeller_mushroom_water_cost = 15.0
  self.repeller_regen_cost = 10.0
  self.repeller_trigger_radius = 70.0
  self.repeller_base_charges = 1.0
  self.tree_mushroom_cost = 20.0
  self.tree_mushroom_water_cost = 5.0
  self.tree_water_per_drop = 5.0
  self.tree_drop_interval = 3600.0
  self.tree_build_reach_px = 190.0
  self.digger_mushroom_cost = 60.0
  self.experience_mushroom_cost = 40.0
  self.experience_mushroom_water_cost = 60.0
  self.experience_cost_growth = 1.5
  self.experience_point_interval = 180.0
  self.experience_crow_interval = 150.0
  self.experience_crow_offset = 40.0
  self.stinky_mushroom_cost = 20.0
  self.stinky_mushroom_water_cost = 20.0
  self.stinky_squirrel_reach_px = 160.0
  self.stinky_squirrel_reach_max_px = 320.0
  self.digger_water_cost_min = 70.0
  self.digger_water_cost_max = 240.0
  self.digger_strand_count = 10.0
  self.animal_interval_jitter = 0.33
  self.animal_interval_per_mushroom = 0.01
  self.animal_interval_max_cut = 0.6
  self.hedgehog_spawn_interval = 240.0
  self.hedgehog_first_delay_min = 60.0
  self.hedgehog_first_delay_max = 90.0
  self.hedgehog_speed = 2.4
  self.hedgehog_reach_px = 48.0
  self.boar_spawn_interval = 300.0
  self.boar_first_delay = 720.0
  self.boar_speed = 8.3
  self.boar_hits = 3.0
  self.boar_reach_px = 36.0
  self.squirrel_spawn_interval = 180.0
  self.squirrel_speed = 4.0
  self.squirrel_run_speed = 6.0
  self.squirrel_repeller_reach_px = 40.0
  self.crow_spawn_interval = 240.0
  self.crow_dive_speed = 12.0
  self.crow_climb_speed = 7.0
  self.life_near_screens = 1.5
  self.life_far_interval_frames = 120.0
  self.worm_far_catch_per_1000px_min = 0.3
  self.worm_far_cap_per_screen_min = 1.0
  self.leaf_spawn_interval = 2700.0
  self.leaf_absorb_radius = 110.0
  self.leaf_absorb_time = 1200.0
  self.leaf_min_hang_frames = 30.0
  self.leaf_food_reward = 2.0
  self.worm_max_count = 16.0
  self.worm_spawn_interval = 900.0
  self.worm_depth_min_px = 48.0
  self.worm_depth_screens = 3.5
  self.worm_spawn_below_screens = 2.0
  self.worm_speed = 0.3
  self.worm_hardsoil_speed = 0.5
  self.worm_catch_radius = 36.0
  self.worm_catch_check_frames = 10.0
  self.worm_absorb_time = 1500.0
  self.worm_food_reward = 2.0
  self.worm_rain_speed = 4.0
  self.consumptor_mushroom_cost = 20.0
  self.consumptor_mushroom_water_cost = 15.0
  self.consumptor_tree_slots = 6.0
  self.consumptor_tree_minutes = 24.0
  self.consumptor_tree_food = 300.0
  self.consumptor_log_minutes = 18.0
  self.consumptor_log_food = 180.0
  self.consumptor_fallen_minutes = 12.0
  self.consumptor_fallen_food = 120.0
  self.consumptor_stump_minutes = 6.0
  self.consumptor_stump_food = 60.0
  self.consumptor_tree_top_row = 323.0
  self.consumptor_min_gap_px = 34.0
  self.upgrade_cost_base = 1.0
  self.upgrade_cost_growth = 1.4
  self.upgrade_cost_tier_mult = 1.25
  self.upg_shot_range_per_level = 0.2125
  self.upg_shot_range_max_level = 10.0
  self.upg_shot_width_per_level = 0.15
  self.upg_shot_width_max_level = 8.0
  self.upg_water_efficiency_per_level = 0.07
  self.upg_water_efficiency_max_level = 8.0
  self.upg_node_efficiency_per_level = 0.08
  self.upg_node_efficiency_max_level = 8.0
  self.upg_budding_chance_per_level = 0.1
  self.upg_budding_chance_max_level = 10.0
  self.upg_soil_penetration_per_level = 0.15
  self.upg_soil_penetration_max_level = 5.0
  self.upg_unlock_spore_cost = 2.0
  self.upg_spore_count_per_level = 0.2
  self.upg_spore_count_max_level = 6.0
  self.upg_spore_lightness_per_level = 0.12
  self.upg_spore_lightness_max_level = 6.0
  self.upg_spore_quality_per_level = 0.05
  self.upg_spore_quality_max_level = 6.0
  self.upg_unlock_predator_cost = 3.0
  self.upg_predator_digestion_per_level = 0.06
  self.upg_predator_digestion_max_level = 6.0
  self.upg_predator_yield_per_level = 0.5
  self.upg_predator_yield_max_level = 6.0
  self.upg_unlock_repeller_cost = 3.0
  self.upg_repeller_autoregen_per_level = 1.0
  self.upg_repeller_autoregen_max_level = 5.0
  self.repeller_autoregen_minutes_first = 5.0
  self.repeller_autoregen_minutes_min = 1.0
  self.upg_repeller_regen_cost_per_level = 0.15
  self.upg_repeller_regen_cost_max_level = 4.0
  self.upg_unlock_tree_cost = 3.0
  self.upg_unlock_digger_cost = 4.0
  self.upg_unlock_stinky_cost = 3.0
  self.upg_unlock_experience_cost = 5.0
  self.upg_stinky_width_per_level = 1.0
  self.upg_stinky_width_max_level = 4.0
  self.upg_digger_length_per_level = 0.4
  self.upg_digger_length_max_level = 5.0
  self.upg_tree_water_yield_per_level = 0.1
  self.upg_tree_water_yield_max_level = 6.0
  self.upg_tree_drop_rate_per_level = 0.06
  self.upg_tree_drop_rate_max_level = 6.0
  self.upg_unlock_absorption_cost = 3.0
  self.upg_absorption_speed_per_level = 0.15
  self.upg_absorption_speed_max_level = 6.0
  self.upg_absorption_yield_per_level = 0.5
  self.upg_absorption_yield_max_level = 6.0
  self.upg_unlock_protein_cost = 4.0
  self.upg_unlock_parasitism_cost = 5.0
  self.upg_parasitism_speed_per_level = 0.15
  self.upg_parasitism_speed_max_level = 6.0
  self.upg_parasitism_yield_per_level = 0.1
  self.upg_parasitism_yield_max_level = 6.0
end

-- gml_Script_balance_defaults @ 0x140048660
function balance_defaults(self, other)
  global.___struct___8 = gm.method(gm.undefined, ___struct___8__balance_defaults__scr_balance)
  return gm.new(___struct___8__balance_defaults__scr_balance)
end

-- gml_Script_balance_init @ 0x140048880
function balance_init(self, other)
  global.balance = balance_defaults(self, other)
end

-- gml_Script_game_mode_apply @ 0x1400489b0
function game_mode_apply(self, other, mode)
  if mode == "survival" then
    global.game_mode = "survival"
  else
    global.game_mode = "exploration"
  end
  global.balance = balance_defaults(self, other)
  local b = global.balance
  if global.game_mode == "survival" then
    local k = 1.7
    local names = gm.variable_struct_get_names(b)
    local i = 0
    while i < gm.array_length(names) do
      local n = names[i]
      local len = gm.string_length(n)
      if gm.string_pos("digger", n) == 0 then
        local hit = false
        if len > 14 and gm.string_copy(n, len - 13, 14) == "_mushroom_cost" then
          hit = true
        elseif len > 20 and gm.string_copy(n, len - 19, 20) == "_mushroom_water_cost" then
          hit = true
        end
        if hit then
          gm.variable_struct_set(b, n, gm.ceil(gm.variable_struct_get(b, n) * k))
        end
      end
      i = i + 1
    end
    b.digger_mushroom_cost = gm.ceil(b.digger_mushroom_cost * 3)
    b.digger_water_cost_min = gm.ceil(b.digger_water_cost_min * 3)
    b.digger_water_cost_max = gm.ceil(b.digger_water_cost_max * 3)
    b.base_shot_water_cost = b.base_shot_water_cost * k
    b.deepsoil_shot_food_cost = gm.ceil(b.deepsoil_shot_food_cost * k)
    b.base_node_food_cost = 8
    b.start_food = b.start_food + 20
    b.start_water = b.start_water + 20
    b.upgrade_cost_growth = 1.7
    return
  end
  b.digger_mushroom_cost = gm.ceil(b.digger_mushroom_cost * 2)
  b.digger_water_cost_min = gm.ceil(b.digger_water_cost_min * 2)
  b.digger_water_cost_max = gm.ceil(b.digger_water_cost_max * 2)
  local k2 = 1.7
  local keys = gm.array("hedgehog_spawn_interval", "hedgehog_first_delay_min", "hedgehog_first_delay_max",
    "boar_spawn_interval", "boar_first_delay", "squirrel_spawn_interval", "crow_spawn_interval",
    "experience_crow_interval", "experience_crow_offset")
  local i = 0
  while i < gm.array_length(keys) do
    gm.variable_struct_set(b, keys[i], gm.variable_struct_get(b, keys[i]) * k2)
    i = i + 1
  end
end

-- gml_Script_game_mode @ 0x14004ad80
function game_mode(self, other)
  if not gm.variable_global_exists("game_mode") then
    return "exploration"
  end
  return global.game_mode
end

-- gml_Script_bal @ 0x14004b000
function bal(self, other, key)
  return gm.variable_struct_get(global.balance, key)
end

-- gml_GlobalScript_scr_balance @ 0x14004b200
function gml_GlobalScript_scr_balance(self, other)
  self.balance_defaults = gm.method(self, balance_defaults)
  self.balance_init = gm.method(self, balance_init)
  self.game_mode_apply = gm.method(self, game_mode_apply)
  self.game_mode = gm.method(self, game_mode)
  self.bal = gm.method(self, bal)
end

-- gml_Script_game_init @ 0x14004bb30
function game_init(self, other)
  gm.randomise()
  balance_init(self, other)
  game_mode_apply(self, other, "exploration")
  visual_init(self, other)
  worldgen_init(self, other)
  locale_init(self, other)
  display_init(self, other)
  upgrades_init(self, other)
  logo_init(self, other)
  tutorial_init(self, other)
  tasks_init(self, other)
  save_init(self, other)
  global.start_x = 0
  global.food = bal(self, other, "start_food")
  global.water_free = bal(self, other, "start_water")
  global.water_sources = gm.array()
  global.water_infinite = false
  global.water_dip = 0
  global.water_dip_hold = 0
  global.cave_perks = gm.struct({})
  global.cave_flash = gm.struct({})
  global.cave_taken = gm.struct({})
  global.animal_markers = gm.array()
  global.cave_wrap = gm.struct({})
  global.cave_popups = gm.array()
  global.cave_burst_pending = false
  global.play_frames = 0
  global.cave_yellow = gm.undefined
  global.relic_taken = gm.struct({})
  global.prof_gen_us = 0
  global.prof_gen_layers = 0
  global.prof_life_us = 0
  global.right_click_used = false
  global.mycelium_length_px = 0
  global.mycelium_points_awarded = 0
  global.menu_open = false
  global.debug_open = false
  global.pause_open = false
  global.intro_active = true
  global.title_open = true
  global.title_alpha = 1
  global.bw_mode = false
  if not gm.variable_global_exists("units_imperial") then
    global.units_imperial = false
  end
  global.hud_hidden = false
  global.bw_gain_handle = gm.shader_get_uniform(sh_blackmetal, "u_gain")
  global.bw_pivot_handle = gm.shader_get_uniform(sh_blackmetal, "u_pivot")
  global.bw_contrast_handle = gm.shader_get_uniform(sh_blackmetal, "u_contrast")
  global.bw_gamma_handle = gm.shader_get_uniform(sh_blackmetal, "u_gamma")
  global.bw_threshold_handle = gm.shader_get_uniform(sh_blackmetal, "u_threshold")
  global.bw_bright_handle = gm.shader_get_uniform(sh_blackmetal, "u_bright")
  global.bw_glow_near = -1
  global.bw_glow_far = -1
  global.glow_blur_step_handle = gm.shader_get_uniform(sh_glow_blur, "u_step")
  global.food_glow_seed = -1
  global.food_glow_blur = -1
  global.gui_surface = -1
  global.cheat_text = ""
  global.cheat_timer = 0
  world_init(self, other, global.start_x)
  global.wind_dir = gm.choose(-1, 1)
  global.wind_speed = gm.random_range(vis(self, other, "wind_min_strength"), vis(self, other, "wind_max_strength"))
  global.wind_timer = vis(self, other, "wind_change_interval")
  weather_init(self, other)
  local margin = vis(self, other, "camera_reach_margin_px")
  global.cam_box_x1 = global.start_x - vis(self, other, "screen_width_px") * 0.5 - margin
  global.cam_box_x2 = global.start_x + vis(self, other, "screen_width_px") * 0.5 + margin
  global.cam_box_y1 = 0
  global.cam_box_y2 = vis(self, other, "screen_height_px")
end

-- gml_Script_game_step @ 0x14004e5f0
function game_step(self, other)
  cave_yellow_step(self, other)
  global.wind_timer = global.wind_timer - 1
  if global.wind_timer <= 0 then
    global.wind_dir = gm.choose(-1, 1)
    global.wind_speed = gm.random_range(vis(self, other, "wind_min_strength"), vis(self, other, "wind_max_strength"))
    global.wind_timer = vis(self, other, "wind_change_interval")
    if game_mode(self, other) == "exploration" then
      if global.wind_speed >= vis(self, other, "wind_max_strength") and not gm.bool(global.title_open)
        and not gm.bool(global.intro_active) and gm.variable_global_exists("leaves") then
        leaf_gust_spawn(self, other)
      end
    end
  end
  upgrades_grow_step(self, other)
  tutorial_step(self, other)
  tasks_step(self, other)
  local pts = gm.floor(mycelium_cm(self, other) / bal(self, other, "xp_length_step_cm"))
  if pts > global.mycelium_points_awarded then
    global.upgrade_points = global.upgrade_points + (pts - global.mycelium_points_awarded)
    if gm.bool(weather_is_raining(self, other)) then
      tasks_stat_add(self, other, "rain_points", pts - global.mycelium_points_awarded)
    end
    global.mycelium_points_awarded = pts
  end
end

-- gml_Script_game_mode_start @ 0x14004f3c0
function game_mode_start(self, other, mode)
  game_mode_apply(self, other, mode)
  life_animal_timers_init(self, other)
  global.food = bal(self, other, "start_food")
  global.water_free = bal(self, other, "start_water")
  if game_mode(self, other) == "survival" and gm.variable_global_exists("world") then
    local keys = gm.variable_struct_get_names(global.world.chunks)
    local i = 0
    while i < gm.array_length(keys) do
      world_survival_thin(self, other, gm.variable_struct_get(global.world.chunks, keys[i]))
      i = i + 1
    end
  end
end

-- gml_Script_cheats_step @ 0x14004fd10
function cheats_step(self, other)
  -- TODO(port): helper FUN_14002f5a0(x, 0) assumed to be "x > 0"
  if global.cheat_timer > 0 then
    global.cheat_timer = global.cheat_timer - 1
  end
  -- the remaining cheat code is compiled out (function returns right after L157)
end

-- gml_Script_post_layer_draw @ 0x140050060
function post_layer_draw(self, other, surf, x, y, w, h)
  if gm.bool(global.bw_mode) then
    local glow = vis(self, other, "bw_glow_strength")
    local halo = vis(self, other, "bw_glow_halo")
    local ds, wide, gw, gh, fw, fh
    if glow > 0 then
      ds = gm.max(2, vis(self, other, "glow_downscale"))
      wide = gm.max(2, vis(self, other, "glow_wide"))
      gw = gm.max(1, w / ds)
      gh = gm.max(1, h / ds)
      fw = gm.max(1, gw / wide)
      fh = gm.max(1, gh / wide)
      global.bw_glow_near = surface_fit(self, other, global.bw_glow_near, gw, gh)
      global.bw_glow_far = surface_fit(self, other, global.bw_glow_far, fw, fh)
      if not (gm.surface_exists(global.bw_glow_near) and gm.surface_exists(global.bw_glow_far)) then
        glow = 0
      end
    end
    if glow > 0 then
      gm.gpu_set_tex_filter(true)
      bw_shader_set(self, other, 1)
      gm.surface_set_target(global.bw_glow_near)
      gm.draw_clear_alpha(0, 0)
      gm.draw_surface_stretched(surf, 0, 0, gw, gh)
      gm.surface_reset_target()
      gm.shader_reset()
      gm.surface_set_target(global.bw_glow_far)
      gm.draw_clear_alpha(0, 0)
      gm.draw_surface_stretched(global.bw_glow_near, 0, 0, fw, fh)
      gm.surface_reset_target()
      gm.gpu_set_tex_filter(false)
    end
    bw_shader_set(self, other, 0)
    gm.draw_surface_stretched(surf, x, y, w, h)
    gm.shader_reset()
    if glow <= 0 then
      return
    end
    gm.gpu_set_tex_filter(true)
    gm.gpu_set_blendmode(gm.bm_add)
    gm.draw_surface_stretched_ext(global.bw_glow_near, x, y, w, h, 16777215, gm.clamp(glow, 0, 1))
    gm.draw_surface_stretched_ext(global.bw_glow_far, x, y, w, h, 16777215, gm.clamp(glow * halo, 0, 1))
    gm.gpu_set_blendmode(gm.bm_normal)
    gm.gpu_set_tex_filter(false)
    return
  end
  gm.draw_surface_stretched(surf, x, y, w, h)
end

-- gml_Script_surface_fit @ 0x140052390
function surface_fit(self, other, surf, w, h)
  w = gm.max(1, gm.floor(w))
  h = gm.max(1, gm.floor(h))
  if gm.surface_exists(surf) then
    local sw = gm.surface_get_width(surf)
    local sh = gm.surface_get_height(surf)
    if sw ~= w or sh ~= h then
      gm.surface_free(surf)
    end
  end
  if gm.surface_exists(surf) then
    return surf
  end
  -- TODO(port): ternary branch is an outlined helper (FUN_140052e20); assumed surface_create(w, h)
  return gm.surface_create(w, h)
end

-- gml_Script_bw_shader_set @ 0x140052fc0
function bw_shader_set(self, other, bright)
  gm.shader_set(sh_blackmetal)
  gm.shader_set_uniform_f(global.bw_gain_handle, vis(self, other, "bw_gain"))
  gm.shader_set_uniform_f(global.bw_pivot_handle, vis(self, other, "bw_pivot"))
  gm.shader_set_uniform_f(global.bw_contrast_handle, vis(self, other, "bw_contrast"))
  gm.shader_set_uniform_f(global.bw_gamma_handle, vis(self, other, "bw_gamma"))
  gm.shader_set_uniform_f(global.bw_threshold_handle, vis(self, other, "bw_glow_threshold"))
  gm.shader_set_uniform_f(global.bw_bright_handle, bright)
end

-- gml_Script_cheat_flash @ 0x140053760
function cheat_flash(self, other, text)
  global.cheat_text = text
  global.cheat_timer = 150
end

-- gml_Script_ui_blocking @ 0x1400538b0
function ui_blocking(self, other)
  return gm.bool(global.menu_open) or gm.bool(global.debug_open) or gm.bool(global.pause_open)
    or gm.bool(global.logo_open) or gm.bool(global.intro_active) or gm.bool(global.title_open)
    or gm.bool(tasks_popup_open(self, other)) or gm.bool(cine_blocking(self, other))
end

-- gml_Script_pixels_to_cm @ 0x140053b30
function pixels_to_cm(self, other, px)
  return px / gm.max(0.01, bal(self, other, "pixels_per_cm"))
end

-- gml_Script_length_text @ 0x140053de0
function length_text(self, other, cm)
  if gm.bool(global.units_imperial) then
    return gm.string(gm.round(cm / 2.54)) .. " " .. loc(self, other, "hud_in")
  end
  return gm.string(gm.round(cm)) .. " " .. loc(self, other, "hud_cm")
end

-- gml_Script_mycelium_cm @ 0x1400545e0
function mycelium_cm(self, other)
  return pixels_to_cm(self, other, global.mycelium_length_px)
end

-- gml_Script_mycelium_add_length @ 0x140054770
function mycelium_add_length(self, other, px)
  global.mycelium_length_px = global.mycelium_length_px + gm.max(0, px)
end

-- gml_Script_food_add @ 0x140054930
function food_add(self, other, n)
  global.food = global.food + n
end

-- gml_Script_food_spend @ 0x1400549f0
function food_spend(self, other, n)
  if global.food < n then
    return false
  end
  global.food = global.food - n
  return true
end

-- gml_Script_reach_expand @ 0x140054b20
function reach_expand(self, other, x, y)
  local m = vis(self, other, "camera_reach_margin_px")
  local x1 = x - m
  local x2 = x + m
  local y1 = gm.max(0, y - m)
  local y2 = gm.min(global.world.bottom_y, y + m)
  global.cam_box_x1 = gm.min(global.cam_box_x1, x1)
  global.cam_box_x2 = gm.max(global.cam_box_x2, x2)
  global.cam_box_y1 = gm.min(global.cam_box_y1, y1)
  global.cam_box_y2 = gm.max(global.cam_box_y2, y2)
end

-- gml_Script____struct___14__camera_bounds__scr_game @ 0x140055500
function ___struct___14__camera_bounds__scr_game(self, other, x1, x2, y1, y2)
  self.x1 = x1
  self.x2 = x2
  self.y1 = y1
  self.y2 = y2
end

-- gml_Script_camera_bounds @ 0x1400557b0
function camera_bounds(self, other, vw, vh)
  local x1 = global.cam_box_x1
  local x2 = global.cam_box_x2 - vw
  local y1 = global.cam_box_y1
  local y2 = gm.min(global.cam_box_y2, global.world.bottom_y) - vh
  if x2 < x1 then
    x1 = (global.cam_box_x1 + global.cam_box_x2) * 0.5 - vw * 0.5
    x2 = x1
  end
  if y2 < y1 then
    y2 = y1
  end
  global.___struct___14 = gm.method(gm.undefined, ___struct___14__camera_bounds__scr_game)
  return gm.new(___struct___14__camera_bounds__scr_game, x1, x2, y1, y2)
end

-- gml_Script_camera_jump_to @ 0x140056130
function camera_jump_to(self, other, x, y)
  gm.with(self, obj_forest_controller, function(self, other)
    local b = camera_bounds(self, other, self.camera_width, self.camera_height)
    self.camera_x = gm.clamp(x - self.camera_width * 0.5, b.x1, b.x2)
    self.camera_y = gm.clamp(y - self.camera_height * 0.5, b.y1, b.y2)
    self.camera_vx = 0
    self.camera_vy = 0
  end)
end

-- gml_Script_camera_jump_origin @ 0x140056990
function camera_jump_origin(self, other)
  if not (gm.instance_exists(obj_mycelium_controller) and gm.instance_exists(obj_forest_controller)) then
    return
  end
  -- obj_mycelium_controller.x (first instance)
  local ox = gm.instance_find(obj_mycelium_controller, 0).x
  gm.with(self, obj_forest_controller, function(self, other)
    local b = camera_bounds(self, other, self.camera_width, self.camera_height)
    self.camera_x = gm.clamp(ox - self.camera_width * 0.5, b.x1, b.x2)
    self.camera_y = gm.clamp(0, b.y1, b.y2)
    self.camera_vx = 0
    self.camera_vy = 0
  end)
end

-- gml_Script____struct___15__mycelium_extreme_tip__scr_game @ 0x1400572c0
function ___struct___15__mycelium_extreme_tip__scr_game(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_mycelium_extreme_tip @ 0x1400574c0
function mycelium_extreme_tip(self, other, dir)
  if not gm.instance_exists(obj_mycelium_controller) then
    return gm.undefined
  end
  local best = gm.undefined
  gm.with(self, obj_mycelium_controller, function(self, other)
    local i = 0
    while i < gm.array_length(self.myc_growths) do
      local branches = self.myc_growths[i].branches
      local j = 0
      while j < gm.array_length(branches) do
        local b = branches[j]
        local edge
        if dir >= 0 then
          edge = b.bx2
        else
          edge = b.bx1
        end
        if gm.is_undefined(best) or (edge - best.x) * dir > 0 then
          local pts = b.points
          local k = 0
          while k < gm.array_length(pts) do
            local p = pts[k]
            if gm.is_undefined(best) or (p.x - best.x) * dir > 0 then
              global.___struct___15 = gm.method(gm.undefined, ___struct___15__mycelium_extreme_tip__scr_game)
              best = gm.new(___struct___15__mycelium_extreme_tip__scr_game, p.x, p.y)
            end
            k = k + 1
          end
        end
        j = j + 1
      end
      i = i + 1
    end
  end)
  return best
end

-- gml_Script_camera_jump_edge @ 0x140058950
function camera_jump_edge(self, other, dir)
  local tip = mycelium_extreme_tip(self, other, dir)
  if gm.is_undefined(tip) then
    camera_jump_origin(self, other)
    return
  end
  camera_jump_to(self, other, tip.x, tip.y)
end

-- gml_Script____struct___16__tutorial_init__scr_game @ 0x140058d20
function ___struct___16__tutorial_init__scr_game(self, other, order, hand_frames, camera_sprite)
  self.order = order
  self.tips = gm.struct({})
  self.events = gm.struct({})
  self.ate = false
  self.drank = false
  self.hand_frames = hand_frames
  self.hand_frame = 0
  self.hand_alpha = 0
  self.hand_done = false
  self.camera_sprite = camera_sprite
  self.camera_alpha = 0
  self.camera_frame = 0
end

-- gml_Script____struct___19__tutorial_init__scr_game @ 0x1400592b0
function ___struct___19__tutorial_init__scr_game(self, other, timer)
  self.state = 0
  self.alpha = 0
  self.timer = timer
  self.anchor = "world"
  self.x = 0
  self.y = 0
end

-- gml_Script_tutorial_init @ 0x1400595b0
function tutorial_init(self, other)
  global.___struct___16 = gm.method(gm.undefined, ___struct___16__tutorial_init__scr_game)
  global.tut = gm.new(___struct___16__tutorial_init__scr_game,
    gm.array("shoot", "node", "salt", "water", "xp", "upgrade", "mushrooms", "rain", "clay", "final"),
    gm.array(),
    gm.asset_get_index("CAMERA_TUTOR"))
  local i = 0
  while i <= 28 do
    local spr = gm.asset_get_index("tutor_gribi_1_" .. gm.string_replace_all(gm.string_format(i, 5, 0), " ", "0"))
    if spr >= 0 then
      gm.array_push(global.tut.hand_frames, spr)
    end
    i = i + 1
  end
  i = 0
  while i < gm.array_length(global.tut.order) do
    global.___struct___19 = gm.method(gm.undefined, ___struct___19__tutorial_init__scr_game)
    gm.variable_struct_set(global.tut.tips, global.tut.order[i], gm.new(___struct___19__tutorial_init__scr_game, -1))
    i = i + 1
  end
end

-- gml_Script____struct___20__tutorial_event__scr_game @ 0x14005a400
function ___struct___20__tutorial_event__scr_game(self, other, x, y)
  self.x = x
  self.y = y
end

-- gml_Script_tutorial_event @ 0x14005a600
function tutorial_event(self, other, name, x, y)
  if gm.is_undefined(x) then x = 0 end
  if gm.is_undefined(y) then y = 0 end
  if not gm.variable_global_exists("tut") then
    return
  end
  global.___struct___20 = gm.method(gm.undefined, ___struct___20__tutorial_event__scr_game)
  gm.variable_struct_set(global.tut.events, name, gm.new(___struct___20__tutorial_event__scr_game, x, y))
end

-- gml_Script_tutorial_show @ 0x14005ab30
function tutorial_show(self, other, name, anchor, x, y, timer)
  local t = gm.variable_struct_get(global.tut.tips, name)
  if t.state ~= 0 then
    return
  end
  t.state = 1
  t.anchor = anchor
  t.x = x
  t.y = y
  t.timer = timer
end

-- gml_Script_tutorial_hide @ 0x14005b110
function tutorial_hide(self, other, name)
  local t = gm.variable_struct_get(global.tut.tips, name)
  if t.state == 1 then
    t.state = 2
  end
end

-- gml_Script_tutorial_show_nearest @ 0x14005b490
function tutorial_show_nearest(self, other, px, py)
  local r = 2
  local c = gm.floor(px / global.world.chunk_w)
  local food = gm.undefined
  local food_d = math.huge
  local lake = gm.undefined
  local lake_d = math.huge
  local ci = c - r
  while ci <= c + r do
    local ch = world_get_chunk(self, other, ci)
    local i = 0
    while i < gm.array_length(ch.foods) do
      local f = ch.foods[i]
      local d = gm.point_distance(px, py, f.x, f.y)
      if not gm.bool(f.collected) and d < food_d then
        food_d = d
        food = f
      end
      i = i + 1
    end
    i = 0
    while i < gm.array_length(ch.lakes) do
      local l = ch.lakes[i]
      local lx = l.body_x + l.body_w * 0.5
      local ly = l.body_y + l.body_h * 0.5
      local d = gm.point_distance(px, py, lx, ly)
      if not gm.bool(l.connected) and d < lake_d then
        lake_d = d
        lake = l
      end
      i = i + 1
    end
    ci = ci + 1
  end
  local oy = vis(self, other, "tutorial_world_offset_y")
  if not gm.bool(global.tut.ate) then
    if gm.is_undefined(food) then
      tutorial_show(self, other, "salt", "world", px, py - oy, -1)
    else
      tutorial_show(self, other, "salt", "world", food.x, food.y - oy, -1)
    end
  end
  if not gm.bool(global.tut.drank) then
    if gm.is_undefined(lake) then
      tutorial_show(self, other, "water", "world", px, py + oy, -1)
      return
    end
    tutorial_show(self, other, "water", "world", lake.body_x + lake.body_w * 0.5, lake.body_y - oy, -1)
  end
end

-- gml_Script_tutorial_wrap @ 0x14005d9f0
function tutorial_wrap(self, other, s)
  if gm.string_length(s) <= vis(self, other, "tutorial_wrap_chars") then
    return s
  end
  local mid = gm.string_length(s) * 0.5
  local best = -1
  local i = 1
  while i <= gm.string_length(s) do
    if gm.string_char_at(s, i) == " " then
      if best < 0 or gm.abs(i - mid) < gm.abs(best - mid) then
        best = i
      end
    end
    i = i + 1
  end
  if best < 0 then
    return s
  end
  return gm.string_copy(s, 1, best - 1) .. "\n" .. gm.string_copy(s, best + 1, gm.string_length(s) - best)
end

-- gml_Script_tutorial_state @ 0x14005e990
function tutorial_state(self, other, name)
  return gm.variable_struct_get(global.tut.tips, name).state
end

-- gml_Script_tutorial_step @ 0x14005ec10
function tutorial_step(self, other)
  local t = global.tut
  if gm.bool(global.title_open) or gm.bool(global.intro_active) then
    t.events = gm.struct({})
    return
  end
  if gm.bool(global.pause_open) or gm.bool(global.debug_open) or gm.bool(global.logo_open) then
    return
  end
  local ev = t.events
  local tip_frames = vis(self, other, "tutorial_tip_frames")
  t.events = gm.struct({})
  tutorial_show(self, other, "shoot", "world", global.start_x,
    wld(self, other, "surface_height_px") - vis(self, other, "tutorial_shoot_offset_y"), -1)
  if not gm.is_undefined(gm.variable_struct_get(ev, "aim")) then
    t.hand_done = true
  end
  if gm.bool(t.hand_done) then
    t.hand_alpha = gm.max(0, t.hand_alpha - 1 / gm.max(1, vis(self, other, "tutorial_hand_fade_frames")))
  else
    t.hand_alpha = t.tips.shoot.alpha
  end
  if t.hand_alpha > 0 then
    t.hand_frame = t.hand_frame + vis(self, other, "tutorial_hand_fps") / gm.game_get_speed(gm.gamespeed_fps)
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "shot")) then
    tutorial_hide(self, other, "shoot")
    tutorial_show(self, other, "node", "world", ev.shot.x, ev.shot.y + vis(self, other, "tutorial_world_offset_y"), -1)
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "node")) then
    tutorial_hide(self, other, "node")
    tutorial_show_nearest(self, other, ev.node.x, ev.node.y)
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "food")) then
    t.ate = true
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "water")) then
    t.drank = true
  end
  if gm.bool(t.ate) then
    tutorial_hide(self, other, "salt")
  end
  local cam_show = false
  if tutorial_state(self, other, "salt") ~= 0 or tutorial_state(self, other, "water") ~= 0 then
    cam_show = not (gm.bool(t.ate) and gm.bool(t.drank))
  end
  -- TODO(port): xmm constants lost in decompilation; assumed 1 / max(1, ...) and clamp(..., 0, 1), -fade when hidden
  local cam_fade = 1 / gm.max(1, vis(self, other, "tutorial_fade_frames"))
  local ca = gm.variable_struct_get(t, "camera_alpha")
  if gm.is_undefined(ca) then ca = 0 end
  if cam_show then
    t.camera_alpha = gm.clamp(ca + cam_fade, 0, 1)
  else
    t.camera_alpha = gm.clamp(ca + -cam_fade, 0, 1)
  end
  local cf = gm.variable_struct_get(t, "camera_frame")
  if gm.is_undefined(cf) then cf = 0 end
  t.camera_frame = cf + vis(self, other, "tutorial_camera_fps") / gm.game_get_speed(gm.gamespeed_fps)
  if gm.bool(t.drank) then
    tutorial_hide(self, other, "water")
  end
  -- TODO(port): helper FUN_14002f610 assumed to be ">="
  if mycelium_cm(self, other) >= vis(self, other, "tutorial_xp_cm") then
    tutorial_show(self, other, "xp", "screen", 0.5, 0.22, vis(self, other, "tutorial_short_frames"))
  end
  if tutorial_state(self, other, "xp") == 2 and t.tips.xp.alpha <= 0 then
    tutorial_show(self, other, "upgrade", "upgrades", 0, 0, tip_frames)
  end
  local menu = global.menu_open
  if gm.bool(menu) then
    tutorial_hide(self, other, "upgrade")
  end
  if not gm.bool(menu) then
    if gm.bool(mushroom_type_unlocked(self, other, 0)) or gm.bool(mushroom_type_unlocked(self, other, 1))
      or gm.bool(mushroom_type_unlocked(self, other, 2)) or gm.bool(mushroom_type_unlocked(self, other, 3))
      or gm.bool(mushroom_type_unlocked(self, other, 4)) or gm.bool(mushroom_type_unlocked(self, other, 5))
      or gm.bool(mushroom_type_unlocked(self, other, 6)) or gm.bool(mushroom_type_unlocked(self, other, 7)) then
      tutorial_show(self, other, "mushrooms", "screen", 0.5, 0.22, tip_frames)
    end
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "mushroom")) then
    tutorial_hide(self, other, "mushrooms")
  end
  if not gm.is_undefined(gm.variable_struct_get(ev, "clay")) then
    tutorial_show(self, other, "clay", "world", ev.clay.x, ev.clay.y + vis(self, other, "tutorial_world_offset_y"),
      vis(self, other, "tutorial_clay_seconds") * gm.game_get_speed(gm.gamespeed_fps))
  end
  if gm.bool(weather_is_raining(self, other)) then
    tutorial_show(self, other, "rain", "screen", 0.5, 0.3, -1)
  else
    tutorial_hide(self, other, "rain")
  end
  local all_done = true
  local i = 0
  while i < gm.array_length(t.order) do
    local name = t.order[i]
    if name ~= "final" and name ~= "clay" then
      if tutorial_state(self, other, name) ~= 2 or gm.variable_struct_get(t.tips, name).alpha > 0 then
        all_done = false
        break
      end
    end
    i = i + 1
  end
  if all_done then
    tutorial_show(self, other, "final", "screen", 0.5, 0.4, vis(self, other, "tutorial_final_frames"))
  end
  local fade = 1 / gm.max(1, vis(self, other, "tutorial_fade_frames"))
  i = 0
  while i < gm.array_length(t.order) do
    local tip = gm.variable_struct_get(t.tips, t.order[i])
    if tip.state == 1 then
      if tip.timer > 0 then
        tip.timer = tip.timer - 1
        if tip.timer <= 0 then
          tip.state = 2
        end
      end
      tip.alpha = gm.min(1, tip.alpha + fade)
    else
      tip.alpha = gm.max(0, tip.alpha - fade)
    end
    i = i + 1
  end
end

-- gml_Script_tutorial_draw @ 0x140062830
function tutorial_draw(self, other, box, col)
  if not gm.variable_global_exists("tut") then
    return
  end
  local t = global.tut
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  local vx = 0
  if gm.instance_exists(obj_forest_controller) then
    vx = gm.instance_find(obj_forest_controller, 0).camera_view_x
  end
  local vy = 0
  if gm.instance_exists(obj_forest_controller) then
    vy = gm.instance_find(obj_forest_controller, 0).camera_view_y
  end
  local pad = 24
  local nf = gm.array_length(t.hand_frames)
  if nf > 0 and t.hand_alpha > 0 and gm.instance_exists(obj_mycelium_controller) then
    local spr = t.hand_frames[gm.mod(gm.floor(t.hand_frame), nf)]
    local sc = vis(self, other, "tutorial_hand_scale")
    local myc = gm.instance_find(obj_mycelium_controller, 0)
    gm.draw_sprite_ext(spr, 0,
      myc.x - vx + vis(self, other, "tutorial_hand_offset_x"),
      myc.y - vy + vis(self, other, "tutorial_hand_offset_y"),
      sc, sc, 0, 16777215, t.hand_alpha)
  end
  local cs = gm.variable_struct_get(t, "camera_sprite")
  if gm.is_undefined(cs) then cs = -1 end
  if cs >= 0 and gm.sprite_exists(cs) then
    local ca = gm.variable_struct_get(t, "camera_alpha")
    if gm.is_undefined(ca) then ca = 0 end
    if ca > 0 and gm.instance_exists(obj_mycelium_controller) then
      local sc = vis(self, other, "tutorial_camera_scale")
      local sw = gm.sprite_get_width(cs)
      local sh = gm.sprite_get_height(cs)
      local myc = gm.instance_find(obj_mycelium_controller, 0)
      local cx = myc.x - vx - (sw * 0.5 - gm.sprite_get_xoffset(cs)) * sc
      local cy = myc.y - vy - vis(self, other, "tutorial_camera_offset_y") - (sh - gm.sprite_get_yoffset(cs)) * sc
      gm.draw_sprite_ext(cs, gm.mod(gm.floor(t.camera_frame), gm.max(1, gm.sprite_get_number(cs))),
        cx, cy, sc, sc, 0, 16777215, t.camera_alpha)
    end
  end
  gm.draw_set_font(ui_font(self, other))
  gm.draw_set_halign(gm.fa_center)
  gm.draw_set_valign(gm.fa_middle)
  local i = 0
  while i < gm.array_length(t.order) do
    local name = t.order[i]
    local tip = gm.variable_struct_get(t.tips, name)
    if tip.alpha > 0 then
      local txt = tutorial_wrap(self, other, loc(self, other, "tut_" .. name))
      local sc
      if name == "final" then
        sc = vis(self, other, "tutorial_final_scale")
      else
        sc = vis(self, other, "tutorial_text_scale")
      end
      local tx = tip.x
      local ty = tip.y
      local anchor = tip.anchor
      if anchor == "world" then
        tx = tx - vx
        ty = ty - vy
      elseif anchor == "screen" then
        tx = tx * gw
        ty = ty * gh
      elseif anchor == "upgrades" then
        tx = (box.x1 + box.x2) * 0.5
        ty = box.y2 + 40
      end
      sc = goal_fit(self, other, txt, sc, gw - pad * 2, 0, -1)
      if tip.anchor ~= "world" then
        local hw = gm.string_width(txt) * sc * 0.5
        local hh = gm.string_height(txt) * sc * 0.5
        tx = gm.clamp(tx, pad + hw, gw - pad - hw)
        ty = gm.clamp(ty, pad + hh, gh - pad - hh)
      end
      gm.draw_set_colour(0)
      gm.draw_set_alpha(tip.alpha * 0.6)
      gm.draw_text_transformed(tx + 3, ty + 3, txt, sc, sc, 0)
      gm.draw_set_colour(col)
      gm.draw_set_alpha(tip.alpha)
      gm.draw_text_transformed(tx, ty, txt, sc, sc, 0)
    end
    i = i + 1
  end
  -- TODO(port): draw_set_alpha argument passed in xmm and lost in decompilation; assumed 1
  gm.draw_set_alpha(1)
  gm.draw_set_font(ui_font_small(self, other))
  gm.draw_set_halign(gm.fa_left)
  gm.draw_set_valign(gm.fa_top)
end

-- gml_Script_tasks_defs @ 0x140067570
local tasks_defs_static = {}
function tasks_defs(self, other)
  if tasks_defs_static._defs == nil then
    tasks_defs_static._defs = gm.array("absorption", "shot_range", "water", "rain_point", "spores", "food", "build",
      "hedgehog", "grow", "trees", "stinky")
  end
  return tasks_defs_static._defs
end

-- gml_Script____struct___24_____struct___23__tasks_init__scr_game @ 0x140067800
function ___struct___24_____struct___23__tasks_init__scr_game(self, other)
  self.water = 0
  self.food = 0
  self.rain_points = 0
  self.spores = 0
  self.built = 0
  self.tree_built = 0
  self.stinky_built = 0
  self.hedgehogs = 0
end

-- gml_Script____struct___26_____struct___23__tasks_init__scr_game @ 0x140067a70
function ___struct___26_____struct___23__tasks_init__scr_game(self, other)
  self.migration = true
  self.magic = true
end

-- gml_Script____struct___29_____struct___23__tasks_init__scr_game @ 0x140067c20
function ___struct___29_____struct___23__tasks_init__scr_game(self, other)
  self.migration = false
  self.magic = false
end

-- gml_Script____struct___23__tasks_init__scr_game @ 0x140067dd0
function ___struct___23__tasks_init__scr_game(self, other)
  global.___struct___24 = gm.method(gm.undefined, ___struct___24_____struct___23__tasks_init__scr_game)
  self.stats = gm.new(___struct___24_____struct___23__tasks_init__scr_game)
  self.done = gm.struct({})
  self.open = true
  global.___struct___26 = gm.method(gm.undefined, ___struct___26_____struct___23__tasks_init__scr_game)
  self.goal_open = gm.new(___struct___26_____struct___23__tasks_init__scr_game)
  self.shown = false
  self.fold_k = gm.struct({})
  self.alpha = 0
  self.fill = gm.struct({})
  self.stage = 0
  global.___struct___29 = gm.method(gm.undefined, ___struct___29_____struct___23__tasks_init__scr_game)
  self.goals = gm.new(___struct___29_____struct___23__tasks_init__scr_game)
  self.popup = ""
end

-- gml_Script_tasks_init @ 0x140068710
function tasks_init(self, other)
  global.___struct___23 = gm.method(gm.undefined, ___struct___23__tasks_init__scr_game)
  global.tasks = gm.new(___struct___23__tasks_init__scr_game)
end

-- gml_Script_tasks_stat_add @ 0x140068960
function tasks_stat_add(self, other, key, n)
  if gm.is_undefined(n) then n = 1 end
  if not gm.variable_global_exists("tasks") then
    return
  end
  local s = global.tasks.stats
  local v = gm.variable_struct_get(s, key)
  if gm.is_undefined(v) then v = 0 end
  gm.variable_struct_set(s, key, v + n)
end

-- gml_Script_tasks_met @ 0x140068f00
function tasks_met(self, other, id)
  local s = global.tasks.stats
  local function stat(k)
    local v = gm.variable_struct_get(s, k)
    if gm.is_undefined(v) then v = 0 end
    return v
  end
  -- TODO(port): helper FUN_14004b9c0(a, n) assumed to be "a >= n"
  if id == "absorption" then
    return upg_is_owned(self, other, "unlock_absorption")
  elseif id == "shot_range" then
    return upg_is_owned(self, other, "shot_range")
  elseif id == "water" then
    return stat("water") >= 1000
  elseif id == "rain_point" then
    return stat("rain_points") >= 1
  elseif id == "spores" then
    return stat("spores") >= 1
  elseif id == "food" then
    return stat("food") >= 30
  elseif id == "build" then
    return stat("built") >= 5
  elseif id == "hedgehog" then
    return stat("hedgehogs") >= 1
  elseif id == "grow" then
    return mycelium_cm(self, other) >= 10000
  elseif id == "trees" then
    return stat("tree_built") >= 2
  elseif id == "stinky" then
    return stat("stinky_built") >= 2
  end
  return false
end
