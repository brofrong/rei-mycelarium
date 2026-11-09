local gm = require("gm")

-- YYC inlines array_length as "0 unless the value is an array"
local function alen(a)
  if gm.is_array(a) then return gm.array_length(a) end
  return 0
end

-- gml_Script_world_draw_water_fill @ 0x140371740
function world_draw_water_fill(self, other, x, y, s)
  gm.draw_set_color(vis(self, other, "water_reservoir_colour"))
  gm.draw_rectangle(x, y, x + s - 1, y + s - 1, false)
  gm.draw_set_color(0xffffff)
end

-- gml_Script____struct___365__world_draw_food_glow__scr_world @ 0x14037df20
function ___struct___365__world_draw_food_glow__scr_world(self, other, x, y, step, light)
  self.x = x
  self.y = y
  self.step = step
  self.light = light
end

-- gml_Script_food_glow_pulse @ 0x140381360
function food_glow_pulse(self, other)
  local lo = gm.max(0, vis(self, other, "food_glow_pulse_low"))
  local hi = gm.max(lo, vis(self, other, "food_glow_pulse_high"))
  local period = gm.max(0.1, vis(self, other, "food_glow_pulse_seconds")) * 2000
  return gm.lerp(lo, hi, (math.sin(6.283185307179586 * gm.current_time / period) + 1) * 0.5)
end

-- gml_Script_world_draw_food_glow_inner @ 0x140381b30
function world_draw_food_glow_inner(self, other, g)
  if gm.bool(gm.is_undefined(g)) then
    return
  end
  local a = gm.clamp(g.light * vis(self, other, "food_glow_inner"), 0, 1)
  if a <= 0 then
    return
  end
  gm.gpu_set_tex_filter(true)
  gm.gpu_set_blendmode(1)
  gm.draw_surface_ext(global.food_glow_seed, g.x, g.y, g.step, g.step, 0, vis(self, other, "food_glow_colour"), a)
  gm.gpu_set_blendmode(0)
  gm.gpu_set_tex_filter(false)
end

-- gml_Script_world_draw_lakes @ 0x140371c00
function world_draw_lakes(self, other, w, vx, vy, vw, vh)
  if not gm.bool(lakes_ready(self, other)) then
    return
  end
  local lakes = w.lakes
  local fw = gm.div(gm.sprite_get_width(water_flare), 3)
  local fh = gm.sprite_get_height(water_flare)
  local ew = gm.sprite_get_width(water_edge)
  local eh = gm.sprite_get_height(water_edge)
  local now = gm.current_time
  local ffps = gm.max(1, vis(self, other, "water_flare_fps"))
  local life = lake_flare_life_ms(self, other)
  local i = 0
  while i < alen(lakes) do
    local lk = lakes[i]
    local kd = global.lake_kinds[lk.kind]
    if lk.x + kd.width < vx or lk.x > vx + vw then
      -- off screen
    elseif lk.y + kd.height < vy or lk.y > vy + vh then
      -- off screen
    else
      gm.draw_sprite_part(water_shade, 0, kd.left, 0, kd.width, kd.height, lk.x, lk.y)
      if lk.amount > 0 then
        local cut = lake_cut_row(self, other, lk, kd)
        if cut < kd.height then
          gm.draw_sprite_part(water, 0, kd.left, cut, kd.width, kd.height - cut, lk.x, lk.y + cut)
        end
        if gm.bool(gm.is_undefined(lk.flares)) then
          lake_init_flares(self, other, lk, kd, cut)
        end
        local j = 0
        while j < alen(lk.flares) do
          local f = lk.flares[j]
          local respawn
          if gm.bool(f.placed) then
            respawn = now - f.born >= life
          else
            respawn = now - f.born >= 500
          end
          if respawn then
            local spot = lake_flare_spot(self, other, lk, kd, j, cut)
            f.born = now
            f.placed = not gm.bool(gm.is_undefined(spot))
            if gm.bool(f.placed) then
              f.x = spot.x
              f.y = spot.y
            end
          end
          if gm.bool(f.placed) then
            local age = now - f.born
            local a = lake_flare_alpha(self, other, age)
            if a > 0 then
              local top = gm.max(0, cut - f.y)
              if not (top >= fh) then
                local frame = math.fmod(gm.floor(age * ffps / 1000), 3)
                gm.draw_sprite_part_ext(water_flare, 0, frame * fw, top, fw, fh - top,
                  lk.x + f.x, lk.y + f.y + top, 1, 1, 16777215, a)
              end
            end
          end
          j = j + 1
        end
        if not (lk.amount >= lk.full) then
          local row = vis(self, other, "water_edge_surface_row")
          local r = 0
          while r < eh do
            local yy = cut - row + r
            if not (yy < 0 or yy >= kd.height) then
              local spans = kd.spans[gm.max(yy, gm.min(cut, kd.height - 1))]
              local s = 0
              while s < alen(spans) do
                local x1 = spans[s + 1]
                local xx = spans[s]
                while xx <= x1 do
                  local ox = gm.mod(xx, ew)
                  local cw = gm.min(ew - ox, x1 + 1 - xx)
                  gm.draw_sprite_part(water_edge, 0, ox, r, cw, 1, lk.x + xx, lk.y + yy)
                  xx = xx + cw
                end
                s = s + 2
              end
            end
            r = r + 1
          end
        end
      end
    end
    i = i + 1
  end
end

-- gml_Script_world_draw_dirt @ 0x1403761f0
function world_draw_dirt(self, other, vx, vy, vw, vh)
  local w = global.world
  local dh = gm.sprite_get_height(dirt1)
  local sp = gm.max(dh, wld(self, other, "dirt_spacing_px"))
  local chance = wld(self, other, "dirt_chance")
  local y0 = gm.max(vy, w.surface_y)
  local y1 = gm.min(vy + vh, w.bottom_y)
  if y1 <= y0 then
    return
  end
  local cx0 = gm.floor(vx / sp) - 1
  local cx1 = gm.floor((vx + vw) / sp)
  local cy0 = gm.floor((y0 - w.surface_y) / sp) - 1
  local cy1 = gm.floor((y1 - w.surface_y) / sp)
  local jit = sp - dh
  local cy = cy0
  while cy <= cy1 do
    local cx = cx0
    while cx <= cx1 do
      if world_hash01(self, other, cx, cy, 71) < chance then
        local py = w.surface_y + cy * sp + world_hash01(self, other, cx, cy, 73) * jit
        if not (py < w.surface_y) and py + dh <= w.bottom_y then
          local px = cx * sp + world_hash01(self, other, cx, cy, 72) * jit
          local spr = world_dirt_sprite(self, other, gm.floor((py + dh * 0.5 - w.surface_y) / w.tile))
          local sh = gm.sprite_get_height(spr)
          local nf = gm.max(1, gm.div(gm.sprite_get_width(spr), sh))
          local f = gm.min(nf - 1, gm.floor(world_hash01(self, other, cx, cy, 74) * nf))
          gm.draw_sprite_part(spr, 0, f * sh, 0, sh, sh, gm.floor(px), gm.floor(py))
        end
      end
      cx = cx + 1
    end
    cy = cy + 1
  end
end

-- gml_Script_world_draw @ 0x140378a80
function world_draw(self, other, vx, vy, vw, vh)
  local w = global.world
  local t = w.tile
  stones_init(self, other)
  lakes_init(self, other)
  stone_tiles_init(self, other)
  if vy + vh < w.surface_y then
    return
  end
  local cx0 = gm.floor(vx / w.chunk_w)
  local cx1 = gm.floor((vx + vw) / w.chunk_w)
  local r0 = gm.clamp(gm.floor((vy - w.surface_y) / t), 0, w.rows - 1)
  local r1 = gm.clamp(gm.floor((vy + vh - w.surface_y) / t), 0, w.rows - 1)
  local shade = gm.merge_colour(0, 16777215, vis(self, other, "food_shade"))
  local r = r0
  while r <= r1 do
    local yy = w.surface_y + r * t
    local spr = world_soil_sprite(self, other, r)
    local xs = gm.floor(vx / t) * t
    local xx = xs
    while xx < vx + vw do
      gm.draw_sprite(spr, 0, xx, yy)
      xx = xx + t
    end
    r = r + 1
  end
  world_draw_soil_edges(self, other, vx, vy, vw, vh)
  world_draw_dirt(self, other, vx, vy, vw, vh)
  world_draw_bones(self, other, vx, vy, vw, vh)
  world_draw_cave(self, other, vx, vy, vw, vh)
  local ci = cx0
  while ci <= cx1 do
    local ch = world_get_chunk(self, other, ci)
    local bx = ci * w.chunk_w
    world_chunk_reach(self, other, ch, r1)
    world_paint_clusters_late(self, other, ch)
    r = r0
    while r <= r1 do
      local yy = w.surface_y + r * t
      local c = 0
      while c < w.cols do
        local xx = bx + c * t
        if xx + t < vx or xx > vx + vw then
          -- off screen
        else
          local tile = ch.tiles[r * w.cols + c]
          if tile == 1 then
            gm.draw_sprite(hardsoil_tile, 0, xx, yy)
          elseif tile == 2 then
            gm.draw_sprite(stone_tile, 0, xx, yy)
          elseif tile == 3 then
            world_draw_water_fill(self, other, xx, yy, t)
          end
        end
        c = c + 1
      end
      r = r + 1
    end
    ci = ci + 1
  end
  ci = cx0
  while ci <= cx1 do
    world_draw_stone_deposits(self, other, world_get_chunk(self, other, ci), vx, vy, vw, vh)
    ci = ci + 1
  end
  ci = cx0
  while ci <= cx1 do
    world_draw_lakes(self, other, world_get_chunk(self, other, ci), vx, vy, vw, vh)
    ci = ci + 1
  end
  ci = cx0
  while ci <= cx1 do
    world_draw_stones(self, other, world_get_chunk(self, other, ci), vx, vy, vw, vh)
    ci = ci + 1
  end
  world_draw_ocean(self, other, vx, vy, vw, vh)
  local vis_foods = gm.array()
  ci = cx0
  while ci <= cx1 do
    local foods = world_get_chunk(self, other, ci).foods
    local k = 0
    while k < alen(foods) do
      local f = foods[k]
      if gm.bool(f.collected) then
        -- skip
      elseif f.x < vx - 48 or f.x > vx + vw + 48 then
        -- off screen
      elseif f.y < vy - 48 or f.y > vy + vh + 48 then
        -- off screen
      else
        gm.array_push(vis_foods, f)
      end
      k = k + 1
    end
    ci = ci + 1
  end
  local glow = world_draw_food_glow(self, other, vis_foods, vx, vy, vw, vh)
  local k = 0
  while k < alen(vis_foods) do
    local f = vis_foods[k]
    local spr = world_food_sprite(self, other, f.kind)
    local sw = gm.sprite_get_width(spr)
    local half = sw * 0.5
    gm.draw_sprite_part_ext(spr, 0, 0, f.cell * sw, sw, sw, gm.floor(f.x - half), gm.floor(f.y - half), 1, 1, shade, 1)
    k = k + 1
  end
  world_draw_food_glow_inner(self, other, glow)
end

-- gml_Script_world_draw_food_glow @ 0x14037e1d0
function world_draw_food_glow(self, other, foods, vx, vy, vw, vh)
  local pulse = food_glow_pulse(self, other)
  local strength = vis(self, other, "food_glow_strength") * pulse
  if strength <= 0 or alen(foods) == 0 then
    return gm.undefined
  end
  local ds = gm.max(2, vis(self, other, "food_glow_downscale"))
  local sw = gm.max(1, gm.div(vw, ds)) + 2
  local sh = gm.max(1, gm.div(vh, ds)) + 2
  global.food_glow_seed = surface_fit(self, other, global.food_glow_seed, sw, sh)
  global.food_glow_blur = surface_fit(self, other, global.food_glow_blur, sw, sh)
  if not gm.bool(gm.surface_exists(global.food_glow_seed)) or not gm.bool(gm.surface_exists(global.food_glow_blur)) then
    return gm.undefined
  end
  local ox = gm.floor(vx / ds) * ds
  local oy = gm.floor(vy / ds) * ds
  local swell = gm.max(0.1, vis(self, other, "food_glow_swell")) / ds
  local inner = gm.clamp(vis(self, other, "food_glow_inner"), 0, 1)
  gm.gpu_set_tex_filter(true)
  gm.surface_set_target(global.food_glow_seed)
  gm.draw_clear_alpha(0, 0)
  local i = 0
  while i < alen(foods) do
    local f = foods[i]
    local spr = world_food_sprite(self, other, f.kind)
    local fw = gm.sprite_get_width(spr)
    local half = fw * swell * 0.5
    gm.draw_sprite_part_ext(spr, 0, 0, f.cell * fw, fw, fw,
      (f.x - ox) / ds - half, (f.y - oy) / ds - half, swell, swell, 16777215, 1)
    i = i + 1
  end
  gm.surface_reset_target()
  local rad = gm.max(0.5, vis(self, other, "food_glow_radius")) * 0.25
  gm.shader_set(sh_glow_blur)
  gm.gpu_set_blendmode_ext(2, 1)
  gm.surface_set_target(global.food_glow_blur)
  gm.shader_set_uniform_f(global.glow_blur_step_handle, rad / sw, 0)
  gm.draw_surface(global.food_glow_seed, 0, 0)
  gm.surface_reset_target()
  gm.surface_set_target(global.food_glow_seed)
  gm.shader_set_uniform_f(global.glow_blur_step_handle, 0, rad / sh)
  gm.draw_surface(global.food_glow_blur, 0, 0)
  gm.surface_reset_target()
  gm.gpu_set_blendmode(0)
  gm.shader_reset()
  gm.gpu_set_blendmode(1)
  gm.draw_surface_ext(global.food_glow_seed, ox, oy, ds, ds, 0, vis(self, other, "food_glow_colour"),
    gm.clamp(strength * (1 - inner), 0, 1))
  gm.gpu_set_blendmode(0)
  gm.gpu_set_tex_filter(false)
  return gm.new(___struct___365__world_draw_food_glow__scr_world, ox, oy, ds, strength)
end

-- gml_GlobalScript_scr_world @ 0x140382520
function gml_GlobalScript_scr_world(self, other)
  self.world_init = gm.method(self, world_init)
  self.world_start_ring = gm.method(self, world_start_ring)
  self.world_start_on_screen = gm.method(self, world_start_on_screen)
  self.world_start_angle = gm.method(self, world_start_angle)
  self.world_rect_distance = gm.method(self, world_rect_distance)
  self.cave_defs = gm.method(self, cave_defs)
  self.cave_def = gm.method(self, cave_def)
  self.cave_range = gm.method(self, cave_range)
  self.cave_place = gm.method(self, cave_place)
  self.cave_host_pick = gm.method(self, cave_host_pick)
  self.world_cave_rects = gm.method(self, world_cave_rects)
  self.cave_yellow_spot = gm.method(self, cave_yellow_spot)
  self.world_area_untouched = gm.method(self, world_area_untouched)
  self.cave_yellow_pick = gm.method(self, cave_yellow_pick)
  self.cave_yellow_step = gm.method(self, cave_yellow_step)
  self.cave_blue_place = gm.method(self, cave_blue_place)
  self.caves_extend = gm.method(self, caves_extend)
  self.world_clear_zones = gm.method(self, world_clear_zones)
  self.bone_kinds = gm.method(self, bone_kinds)
  self.bone_kind_weight = gm.method(self, bone_kind_weight)
  self.bone_kind_pick = gm.method(self, bone_kind_pick)
  self.world_paint_bones = gm.method(self, world_paint_bones)
  self.world_big_bones = gm.method(self, world_big_bones)
  self.world_draw_bones = gm.method(self, world_draw_bones)
  self.relic_def = gm.method(self, relic_def)
  self.world_relics = gm.method(self, world_relics)
  self.relic_taken = gm.method(self, relic_taken)
  self.relic_wrapped = gm.method(self, relic_wrapped)
  self.relics_touch_at = gm.method(self, relics_touch_at)
  self.relics_wrap_draw_visible = gm.method(self, relics_wrap_draw_visible)
  self.world_cave_hits = gm.method(self, world_cave_hits)
  self.caves_mask_init = gm.method(self, caves_mask_init)
  self.cave_hash = gm.method(self, cave_hash)
  self.cave_branch_points = gm.method(self, cave_branch_points)
  self.cave_outline_build = gm.method(self, cave_outline_build)
  self.cave_solid_at = gm.method(self, cave_solid_at)
  self.world_cave_solid_any = gm.method(self, world_cave_solid_any)
  self.world_clear_caves = gm.method(self, world_clear_caves)
  self.perk_has = gm.method(self, perk_has)
  self.cave_taken = gm.method(self, cave_taken)
  self.caves_touch_at = gm.method(self, caves_touch_at)
  self.cave_take = gm.method(self, cave_take)
  self.perk_time_mult = gm.method(self, perk_time_mult)
  self.perk_cost_mult = gm.method(self, perk_cost_mult)
  self.world_draw_cave = gm.method(self, world_draw_cave)
  self.caves_wrap_hit = gm.method(self, caves_wrap_hit)
  self.caves_wrap_draw_visible = gm.method(self, caves_wrap_draw_visible)
  self.cave_wrap_draw = gm.method(self, cave_wrap_draw)
  self.cave_popups_draw = gm.method(self, cave_popups_draw)
  self.cave_perk_icons_draw = gm.method(self, cave_perk_icons_draw)
  self.cave_hover_tip = gm.method(self, cave_hover_tip)
  self.perk_debug_next = gm.method(self, perk_debug_next)
  self.caves_debug_draw = gm.method(self, caves_debug_draw)
  self.world_start_clear = gm.method(self, world_start_clear)
  self.world_start_food_quota = gm.method(self, world_start_food_quota)
  self.world_food_value_in = gm.method(self, world_food_value_in)
  self.world_food_fill = gm.method(self, world_food_fill)
  self.world_food_spot_ok = gm.method(self, world_food_spot_ok)
  self.world_start_guarantee = gm.method(self, world_start_guarantee)
  self.world_start_add_lake = gm.method(self, world_start_add_lake)
  self.world_start_add_food = gm.method(self, world_start_add_food)
  self.world_hash01 = gm.method(self, world_hash01)
  self.world_chunk_key = gm.method(self, world_chunk_key)
  self.world_get_chunk = gm.method(self, world_get_chunk)
  self.world_chunk_set = gm.method(self, world_chunk_set)
  self.world_chunk_get = gm.method(self, world_chunk_get)
  self.world_paint_stone_clusters = gm.method(self, world_paint_stone_clusters)
  self.world_paint_stone_pillar = gm.method(self, world_paint_stone_pillar)
  self.lake_layout = gm.method(self, lake_layout)
  self.lakes_init = gm.method(self, lakes_init)
  self.lakes_ready = gm.method(self, lakes_ready)
  self.lake_solid_at = gm.method(self, lake_solid_at)
  self.lake_pick_kind = gm.method(self, lake_pick_kind)
  self.lake_amount = gm.method(self, lake_amount)
  self.lake_flare_count = gm.method(self, lake_flare_count)
  self.world_paint_lakes = gm.method(self, world_paint_lakes)
  self.world_lake_covers = gm.method(self, world_lake_covers)
  self.lake_flare_life_ms = gm.method(self, lake_flare_life_ms)
  self.lake_flare_alpha = gm.method(self, lake_flare_alpha)
  self.lake_flare_spot = gm.method(self, lake_flare_spot)
  self.lake_init_flares = gm.method(self, lake_init_flares)
  self.lake_cut_row = gm.method(self, lake_cut_row)
  self.world_paint_food = gm.method(self, world_paint_food)
  self.world_layer_top = gm.method(self, world_layer_top)
  self.world_layer_bottom = gm.method(self, world_layer_bottom)
  self.world_layer_key = gm.method(self, world_layer_key)
  self.world_generate_chunk = gm.method(self, world_generate_chunk)
  self.world_chunk_reach = gm.method(self, world_chunk_reach)
  self.world_survival_full_at = gm.method(self, world_survival_full_at)
  self.world_survival_thin = gm.method(self, world_survival_thin)
  self.world_paint_layer = gm.method(self, world_paint_layer)
  self.world_paint_layer_content = gm.method(self, world_paint_layer_content)
  self.world_clear_ocean = gm.method(self, world_clear_ocean)
  self.world_locate = gm.method(self, world_locate)
  self.world_tile_at = gm.method(self, world_tile_at)
  self.world_water_at = gm.method(self, world_water_at)
  self.world_water_shown_at = gm.method(self, world_water_shown_at)
  self.world_is_passable = gm.method(self, world_is_passable)
  self.world_is_deep_soil = gm.method(self, world_is_deep_soil)
  self.world_soil_water_factor = gm.method(self, world_soil_water_factor)
  self.world_travel_cost = gm.method(self, world_travel_cost)
  self.world_connect_lake_at = gm.method(self, world_connect_lake_at)
  self.world_connect_water_at = gm.method(self, world_connect_water_at)
  self.world_food_sheets = gm.method(self, world_food_sheets)
  self.world_food_kinds = gm.method(self, world_food_kinds)
  self.world_food_sprite = gm.method(self, world_food_sprite)
  self.world_food_cells = gm.method(self, world_food_cells)
  self.world_food_value = gm.method(self, world_food_value)
  self.world_food_columns = gm.method(self, world_food_columns)
  self.world_collect_food_at = gm.method(self, world_collect_food_at)
  self.water_source_amount = gm.method(self, water_source_amount)
  self.water_source_take = gm.method(self, water_source_take)
  self.water_total = gm.method(self, water_total)
  self.water_spend = gm.method(self, water_spend)
  self.water_shown = gm.method(self, water_shown)
  self.water_counter_step = gm.method(self, water_counter_step)
  self.world_draw_turf = gm.method(self, world_draw_turf)
  self.world_soil_band = gm.method(self, world_soil_band)
  self.world_seam_row_at_screens = gm.method(self, world_seam_row_at_screens)
  self.world_dense_row = gm.method(self, world_dense_row)
  self.world_deep_row = gm.method(self, world_deep_row)
  self.world_soil_tier = gm.method(self, world_soil_tier)
  self.world_deep_y = gm.method(self, world_deep_y)
  self.world_ocean_row = gm.method(self, world_ocean_row)
  self.world_ocean_y = gm.method(self, world_ocean_y)
  self.world_is_ocean_at = gm.method(self, world_is_ocean_at)
  self.world_dense_y = gm.method(self, world_dense_y)
  self.world_soil_sprite = gm.method(self, world_soil_sprite)
  self.world_dirt_sprite = gm.method(self, world_dirt_sprite)
  self.world_soil_edge_sprite = gm.method(self, world_soil_edge_sprite)
  self.world_draw_soil_edges = gm.method(self, world_draw_soil_edges)
  self.world_bottom_is_rock = gm.method(self, world_bottom_is_rock)
  self.world_draw_bottom_rock = gm.method(self, world_draw_bottom_rock)
  self.world_draw_ocean = gm.method(self, world_draw_ocean)
  self.world_draw_ocean_flares = gm.method(self, world_draw_ocean_flares)
  self.world_draw_water_fill = gm.method(self, world_draw_water_fill)
  self.world_draw_lakes = gm.method(self, world_draw_lakes)
  self.world_draw_dirt = gm.method(self, world_draw_dirt)
  self.world_draw = gm.method(self, world_draw)
  self.world_draw_food_glow = gm.method(self, world_draw_food_glow)
  self.food_glow_pulse = gm.method(self, food_glow_pulse)
  self.world_draw_food_glow_inner = gm.method(self, world_draw_food_glow_inner)
end
