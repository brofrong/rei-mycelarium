local gm = require("gm")

-- gml_Script_logo_draw_blade @ 0x1401cc610
function logo_draw_blade(self, other, blade, ox, oy, scale, prog)
  if not (prog > 0) then
    return
  end
  local pts = blade.pts
  local n = gm.array_length(pts)
  local lim = 1 + prog * (n - 1)
  gm.draw_primitive_begin(gm.pr_trianglestrip)
  local i = 0
  while i < n do
    if i > lim then
      break
    end
    local t = i / (n - 1)
    local x = ox + pts[i][0] * scale
    local y = oy + pts[i][1] * scale
    local lo = gm.max(0, i - 1)
    local hi = gm.min(n - 1, i + 1)
    local dir = gm.point_direction(pts[lo][0], pts[lo][1], pts[hi][0], pts[hi][1])
    local run = vis(self, other, "logo_collar_run")
    local w1 = blade.width * 0.5 * gm.power(1 - t, 1.35)
    local w2 = blade.root * vis(self, other, "logo_collar") * gm.power(1 - gm.min(1, t / run), 2)
    local w = gm.max(w1, w2) * scale
    if prog < 1 then
      w = w * gm.clamp(lim - i, 0, 1)
    end
    w = gm.max(w, vis(self, other, "logo_min_pixels"))
    gm.draw_vertex(x + gm.lengthdir_x(w, dir + 90), y + gm.lengthdir_y(w, dir + 90))
    gm.draw_vertex(x + gm.lengthdir_x(w, dir - 90), y + gm.lengthdir_y(w, dir - 90))
    i = i + 1
  end
  gm.draw_primitive_end()
end

-- gml_Script_logo_paint @ 0x1401cef50
function logo_paint(self, other, cx, cy, w, h, col)
  local m = global.logo_model
  local mw = gm.max(0.6, m.x2 - m.x1) + 0.2
  local mh = gm.max(0.6, m.y2 - m.y1) + 0.2
  local s = gm.min(w / mw, h / mh)
  local ox = cx - (m.x1 + m.x2) * 0.5 * s
  local oy = cy - (m.y1 + m.y2) * 0.5 * s
  local p = global.logo_progress
  gm.draw_set_alpha(1)
  gm.draw_set_colour(col)
  local i = 0
  while i < gm.array_length(m.blades) do
    local b = m.blades[i]
    logo_draw_blade(self, other, b, ox, oy, s, gm.clamp((p - b.birth) / b.span, 0, 1))
    i = i + 1
  end
  i = 0
  while i < gm.array_length(m.ribbons) do
    local r = m.ribbons[i]
    logo_draw_ribbon(self, other, r, ox, oy, s, gm.clamp((p - r.letter * 0.5) / 0.9, 0, 1))
    i = i + 1
  end
  gm.draw_set_alpha(1)
end

-- gml_Script_logo_draw @ 0x1401d0730
function logo_draw(self, other, cx, cy, w, h, col)
  local m = global.logo_model
  if gm.is_undefined(m) then
    return
  end
  if m.letters == 0 then
    return
  end
  if global.logo_progress < m.duration then
    logo_drop_surface(self, other)
    logo_paint(self, other, cx, cy, w, h, col)
    return
  end
  local gw = gm.display_get_gui_width()
  local gh = gm.display_get_gui_height()
  if not gm.surface_exists(global.logo_surface) then
    global.logo_surface = gm.surface_create(gm.max(1, gw), gm.max(1, gh))
    if not gm.surface_exists(global.logo_surface) then
      logo_paint(self, other, cx, cy, w, h, col)
      return
    end
    gm.surface_set_target(global.logo_surface)
    gm.draw_clear_alpha(0, 0)
    logo_paint(self, other, cx, cy, w, h, col)
    gm.surface_reset_target()
  end
  gm.draw_surface(global.logo_surface, 0, 0)
end

-- gml_Script_logo_drop_surface @ 0x1401d17c0
function logo_drop_surface(self, other)
  if gm.surface_exists(global.logo_surface) then
    gm.surface_free(global.logo_surface)
  end
  global.logo_surface = -1
end

-- gml_Script_logo_read_typing @ 0x1401d1a30
function logo_read_typing(self, other)
  local src = gm.string_upper(gm.keyboard_string)
  local s = ""
  local i = 1
  while i <= gm.string_length(src) do
    local ch = gm.string_char_at(src, i)
    local c = gm.ord(ch)
    if (c >= 65 and c <= 90) or c == 32 then
      s = s .. ch
    end
    i = i + 1
  end
  if gm.string_length(s) > 18 then
    s = gm.string_copy(s, 1, 18)
  end
  gm.keyboard_string = s
  global.logo_text = s
end

-- gml_Script_logo_has_letters @ 0x1401d2310
function logo_has_letters(self, other)
  return gm.string_length(gm.string_replace_all(global.logo_text, " ", "")) > 0
end

-- gml_Script_logo_grow_start @ 0x1401d25e0
function logo_grow_start(self, other)
  logo_drop_surface(self, other)
  global.logo_model = logo_build(self, other, global.logo_text, global.logo_seed + logo_text_seed(self, other, global.logo_text))
  if global.logo_model.letters == 0 then
    return
  end
  global.logo_phase = "grow"
  global.logo_progress = 0
end

-- gml_Script_logo_edit @ 0x1401d2aa0
function logo_edit(self, other)
  logo_drop_surface(self, other)
  global.logo_phase = "type"
  gm.keyboard_string = global.logo_text
end

-- gml_Script_logo_reroll @ 0x1401d2c50
function logo_reroll(self, other)
  global.logo_seed = global.logo_seed + 1
  logo_grow_start(self, other)
end

-- gml_Script_logo_open @ 0x1401d2d60
function logo_open(self, other)
  logo_drop_surface(self, other)
  global.logo_open = true
  global.logo_phase = "type"
  gm.keyboard_string = global.logo_text
end

-- gml_Script_logo_step @ 0x1401d2f90
function logo_step(self, other)
  if global.logo_phase == "type" then
    logo_read_typing(self, other)
    if gm.keyboard_check_pressed(13) and logo_has_letters(self, other) then
      logo_grow_start(self, other)
    end
  else
    local dur
    if gm.is_undefined(global.logo_model) then
      dur = 1
    else
      dur = global.logo_model.duration
    end
    global.logo_progress = gm.min(dur, global.logo_progress + dur / gm.max(1, vis(self, other, "logo_grow_frames")))
  end
end

-- gml_GlobalScript_scr_logo @ 0x1401d37f0
function gml_GlobalScript_scr_logo(self, other)
  self.logo_init = gm.method(self, logo_init)
  self.logo_noise = gm.method(self, logo_noise)
  self.logo_text_seed = gm.method(self, logo_text_seed)
  self.logo_make_path = gm.method(self, logo_make_path)
  self.logo_path_at = gm.method(self, logo_path_at)
  self.logo_path_nearest = gm.method(self, logo_path_nearest)
  self.logo_half_width = gm.method(self, logo_half_width)
  self.logo_reach = gm.method(self, logo_reach)
  self.logo_stretch = gm.method(self, logo_stretch)
  self.logo_push_blade = gm.method(self, logo_push_blade)
  self.logo_anchor = gm.method(self, logo_anchor)
  self.logo_grow_strand = gm.method(self, logo_grow_strand)
  self.logo_grow_thorn = gm.method(self, logo_grow_thorn)
  self.logo_grow_web = gm.method(self, logo_grow_web)
  self.logo_seed_points = gm.method(self, logo_seed_points)
  self.logo_outline = gm.method(self, logo_outline)
  self.logo_build = gm.method(self, logo_build)
  self.logo_draw_ribbon = gm.method(self, logo_draw_ribbon)
  self.logo_draw_blade = gm.method(self, logo_draw_blade)
  self.logo_paint = gm.method(self, logo_paint)
  self.logo_draw = gm.method(self, logo_draw)
  self.logo_drop_surface = gm.method(self, logo_drop_surface)
  self.logo_read_typing = gm.method(self, logo_read_typing)
  self.logo_has_letters = gm.method(self, logo_has_letters)
  self.logo_grow_start = gm.method(self, logo_grow_start)
  self.logo_edit = gm.method(self, logo_edit)
  self.logo_reroll = gm.method(self, logo_reroll)
  self.logo_open = gm.method(self, logo_open)
  self.logo_step = gm.method(self, logo_step)
end

-- gml_Script_save_init @ 0x1401d47f0
function save_init(self, other)
  global.save_slot = -1
  global.save_name = ""
  global.save_timer = 0
end

-- gml_Script_save_path @ 0x1401d4950
function save_path(self, other, slot)
  return "save_" .. gm.string(slot + 1) .. ".json"
end

-- gml_Script_save_meta_path @ 0x1401d4ce0
function save_meta_path(self, other, slot)
  return "save_" .. gm.string(slot + 1) .. "_meta.json"
end

-- gml_Script_save_write_text @ 0x1401d5070
function save_write_text(self, other, path, text)
  local tmp = path .. ".tmp"
  local buf = gm.buffer_create(gm.string_byte_length(text) + 1, 0, 1)
  gm.buffer_write(buf, 11, text)
  gm.buffer_save(buf, tmp)
  gm.buffer_delete(buf)
  if gm.file_exists(path) then
    gm.file_delete(path)
  end
  gm.file_rename(tmp, path)
end

-- gml_Script_save_read_text @ 0x1401d5840
function save_read_text(self, other, path)
  if not gm.file_exists(path) then
    return gm.undefined
  end
  local buf = gm.buffer_load(path)
  if buf < 0 then
    return gm.undefined
  end
  local s = gm.buffer_read(buf, 11)
  gm.buffer_delete(buf)
  return s
end

-- gml_Script_save_slot_meta @ 0x1401d5cc0
function save_slot_meta(self, other, slot)
  local txt = save_read_text(self, other, save_meta_path(self, other, slot))
  if gm.is_undefined(txt) then
    return gm.undefined
  end
  -- TODO(port): catch funclet not in decomp; assumed `catch (e) { return undefined; }`
  local ok, res = pcall(gm.json_parse, txt)
  if ok then
    return res
  end
  return gm.undefined
end

-- gml_Script_save_any_exists @ 0x1401d6240
function save_any_exists(self, other)
  local i = 0
  while i < 3 do
    if not gm.is_undefined(save_slot_meta(self, other, i)) then
      return true
    end
    i = i + 1
  end
  return false
end

-- gml_Script_save_start_new @ 0x1401d6530
function save_start_new(self, other, slot, name)
  global.save_slot = slot
  global.save_name = name
  global.save_timer = global.fps * 2
end

-- gml_Script_save_step @ 0x1401d67b0
function save_step(self, other)
  if global.save_slot < 0 or gm.bool(global.title_open) or gm.bool(global.intro_active) or gm.bool(global.pause_open) then
    return
  end
  global.save_timer = global.save_timer - 1
  if global.save_timer <= 0 then
    save_game(self, other)
  end
end

-- gml_Script_save_copy_struct @ 0x1401d6a10
function save_copy_struct(self, other, src)
  local r = gm.struct({})
  local names = gm.variable_struct_get_names(src)
  local i = 0
  while i < gm.array_length(names) do
    gm.variable_struct_set(r, names[i], gm.variable_struct_get(src, names[i]))
    i = i + 1
  end
  return r
end

-- gml_Script____struct___160__save_game__scr_save @ 0x1401d6ff0
function ___struct___160__save_game__scr_save(self, other, id, sprite, x, y, flip, roots, consume)
  self.id = id
  self.sprite = sprite
  self.x = x
  self.y = y
  self.flip = flip
  self.roots = roots
  self.consume = consume
end

-- gml_Script____struct___161__save_game__scr_save @ 0x1401d7410
function ___struct___161__save_game__scr_save(self, other, sprite, x, y, flip, roots)
  self.sprite = sprite
  self.x = x
  self.y = y
  self.flip = flip
  self.roots = roots
end

-- gml_Script____struct___162__save_game__scr_save @ 0x1401d7740
function ___struct___162__save_game__scr_save(self, other, id, part, x, y, flip, roots, consume, start)
  self.id = id
  self.part = part
  self.x = x
  self.y = y
  self.flip = flip
  self.roots = roots
  self.consume = consume
  self.start = start
end

-- gml_Script____struct___163__save_game__scr_save @ 0x1401d7bd0
function ___struct___163__save_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___164__save_game__scr_save @ 0x1401d7e60
function ___struct___164__save_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___165__save_game__scr_save @ 0x1401d80f0
function ___struct___165__save_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___166__save_game__scr_save @ 0x1401d8380
function ___struct___166__save_game__scr_save(self, other, trees, small_trees, bushes, plants, decor_trees, stumps)
  self.trees = trees
  self.small_trees = small_trees
  self.bushes = bushes
  self.plants = plants
  self.decor_trees = decor_trees
  self.stumps = stumps
end

-- gml_Script____struct___167__save_game__scr_save @ 0x1401d86e0
function ___struct___167__save_game__scr_save(self, other, points, start, width, shoved, swept)
  self.points = points
  self.start = start
  self.width = width
  self.shoved = shoved
  self.swept = swept
end

-- gml_Script____struct___168__save_game__scr_save @ 0x1401d89f0
function ___struct___168__save_game__scr_save(self, other, branches)
  self.branches = branches
end

-- gml_Script____struct___169__save_game__scr_save @ 0x1401d8b50
function ___struct___169__save_game__scr_save(self, other, lake_cx, lake)
  self.lake_cx = lake_cx
  self.lake = lake
end

-- gml_Script____struct___170__save_game__scr_save @ 0x1401d8d10
function ___struct___170__save_game__scr_save(self, other, cx, idx)
  self.cx = cx
  self.idx = idx
end

-- gml_Script____struct___172_____struct___171__save_game__scr_save @ 0x1401d8ed0
function ___struct___172_____struct___171__save_game__scr_save(self, other, chunks, camera_x, camera_y)
  self.chunks = chunks
  self.camera_x = camera_x
  self.camera_y = camera_y
end

-- gml_Script____struct___173_____struct___171__save_game__scr_save @ 0x1401d9110
function ___struct___173_____struct___171__save_game__scr_save(self, other, growths, knots)
  self.growths = growths
  self.knots = knots
end

-- gml_Script____struct___175_____struct___174_____struct___171__save_game__scr_save @ 0x1401d92e0
function ___struct___175_____struct___174_____struct___171__save_game__scr_save(self, other, hedgehog, boar, squirrel, crow, experience_crows, leaf, worm)
  self.hedgehog = hedgehog
  self.boar = boar
  self.squirrel = squirrel
  self.crow = crow
  self.experience_crows = experience_crows
  self.leaf = leaf
  self.worm = worm
end

-- gml_Script____struct___176_____struct___174_____struct___171__save_game__scr_save @ 0x1401d96c0
function ___struct___176_____struct___174_____struct___171__save_game__scr_save(self, other, tips, events, ate, drank, hand_done)
  self.tips = tips
  self.events = events
  self.ate = ate
  self.drank = drank
  self.hand_done = hand_done
end

-- gml_Script____struct___177_____struct___174_____struct___171__save_game__scr_save @ 0x1401d99b0
function ___struct___177_____struct___174_____struct___171__save_game__scr_save(self, other, stats, done, open, shown, stage, goals, popup, goal_open)
  self.stats = stats
  self.done = done
  self.open = open
  self.shown = shown
  self.stage = stage
  self.goals = goals
  self.popup = popup
  self.goal_open = goal_open
end

-- gml_Script____struct___174_____struct___171__save_game__scr_save @ 0x1401d9de0
function ___struct___174_____struct___171__save_game__scr_save(self, other, ...)
  local a = { ... }
  self.start_x = a[1]
  self.food = a[2]
  self.water_free = a[3]
  self.water_infinite = a[4]
  self.cave_perks = a[5]
  self.cave_taken = a[6]
  self.length_px = a[7]
  self.play_frames = a[8]
  self.cave_yellow = a[9]
  self.relic_taken = a[10]
  self.mode = a[11]
  self.points_awarded = a[12]
  self.upgrade_points = a[13]
  self.upgrade_levels = a[14]
  self.cam_box = a[15]
  self.wind = a[16]
  self.timers = gm.new(___struct___175_____struct___174_____struct___171__save_game__scr_save,
    a[17], a[18], a[19], a[20], a[21], a[22], a[23])
  self.tutorial = gm.new(___struct___176_____struct___174_____struct___171__save_game__scr_save,
    a[24], a[25], a[26], a[27], a[28])
  self.tasks = gm.new(___struct___177_____struct___174_____struct___171__save_game__scr_save,
    a[29], a[30], a[31], a[32], a[33], a[34], a[35], a[36])
end

-- gml_Script____struct___171__save_game__scr_save @ 0x1401db0c0
function ___struct___171__save_game__scr_save(self, other, ...)
  local a = { ... }
  self.version = 1
  self.name = a[1]
  self.world = a[2]
  self.water = a[3]
  self.forest = gm.new(___struct___172_____struct___171__save_game__scr_save, a[4], a[5], a[6])
  self.mycelium = gm.new(___struct___173_____struct___171__save_game__scr_save, a[7], a[8])
  self.mushrooms = a[9]
  self.state = gm.new(___struct___174_____struct___171__save_game__scr_save,
    a[10], -- state.start_x
    a[11], -- state.food
    a[12], -- state.water_free
    a[13], -- state.water_infinite
    a[14], -- state.cave_perks
    a[15], -- state.cave_taken
    a[16], -- state.length_px
    a[17], -- state.play_frames
    a[18], -- state.cave_yellow
    a[19], -- state.relic_taken
    a[20], -- state.mode
    a[21], -- state.points_awarded
    a[22], -- state.upgrade_points
    a[23], -- state.upgrade_levels
    a[24], -- state.cam_box
    a[25], -- state.wind
    a[26], -- state.timers.hedgehog
    a[27], -- state.timers.boar
    a[28], -- state.timers.squirrel
    a[29], -- state.timers.crow
    a[30], -- state.timers.experience_crows
    a[31], -- state.timers.leaf
    a[32], -- state.timers.worm
    a[33], -- state.tutorial.tips
    a[34], -- state.tutorial.events
    a[35], -- state.tutorial.ate
    a[36], -- state.tutorial.drank
    a[37], -- state.tutorial.hand_done
    a[38], -- state.tasks.stats
    a[39], -- state.tasks.done
    a[40], -- state.tasks.open
    a[41], -- state.tasks.shown
    a[42], -- state.tasks.stage
    a[43], -- state.tasks.goals
    a[44], -- state.tasks.popup
    a[45]) -- state.tasks.goal_open
end

-- gml_Script____struct___178__save_game__scr_save @ 0x1401dc730
function ___struct___178__save_game__scr_save(self, other, name, saved, length_cm, mode)
  self.name = name
  self.saved = saved
  self.length_cm = length_cm
  self.mode = mode
end

-- gml_Script_save_game @ 0x1401dc9e0
function save_game(self, other)
  if global.save_slot < 0 or gm.bool(global.title_open) or gm.bool(global.intro_active) then
    return false
  end
  if not (gm.instance_exists(obj_forest_controller) and gm.instance_exists(obj_mycelium_controller)) then
    return false
  end
  global.save_timer = bal(self, other, "save_autosave_seconds") * global.fps
  local fc = gm.instance_find(obj_forest_controller, 0)
  local mc = gm.instance_find(obj_mycelium_controller, 0)
  local sid = 0
  local chunks = gm.struct({})
  local keys = gm.variable_struct_get_names(fc.surface_chunks)
  local k = 0
  while k < gm.array_length(keys) do
    local ch = gm.variable_struct_get(fc.surface_chunks, keys[k])
    local trees = gm.array()
    local small = gm.array()
    local stumps = gm.array()
    local j = 0
    while j < gm.array_length(ch.trees) do
      local t = ch.trees[j]
      t.save_id = sid
      sid = sid + 1
      local consume = t.consume
      if gm.is_undefined(consume) then
        consume = 0
      end
      gm.array_push(trees, gm.new(___struct___160__save_game__scr_save,
        t.save_id, gm.sprite_get_name(t.sprite), t.x, t.y, t.flip, t.roots, consume))
      j = j + 1
    end
    j = 0
    while j < gm.array_length(ch.small_trees) do
      local st = ch.small_trees[j]
      gm.array_push(small, gm.new(___struct___161__save_game__scr_save,
        gm.sprite_get_name(st.sprite), st.x, st.y, st.flip, st.roots))
      j = j + 1
    end
    j = 0
    while j < gm.array_length(ch.stumps) do
      local s = ch.stumps[j]
      local pi = 0
      local q = 0
      while q < gm.array_length(fc.stump_parts) do
        if fc.stump_parts[q].left == s.part.left then
          pi = q
        end
        q = q + 1
      end
      s.save_id = sid
      sid = sid + 1
      local consume = s.consume
      if gm.is_undefined(consume) then
        consume = 0
      end
      gm.array_push(stumps, gm.new(___struct___162__save_game__scr_save,
        s.save_id, pi, s.x, s.y, s.flip, s.roots, consume, s == fc.start_stump))
      j = j + 1
    end
    local bushes = gm.array()
    local src = ch.bushes
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local b = src[j]
      gm.array_push(bushes, gm.new(___struct___163__save_game__scr_save, b.part, b.x, b.y))
      j = j + 1
    end
    local plants = gm.array()
    src = ch.plants
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local p = src[j]
      gm.array_push(plants, gm.new(___struct___164__save_game__scr_save, p.part, p.x, p.y))
      j = j + 1
    end
    local decor = gm.array()
    src = ch.decor_trees
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local d = src[j]
      gm.array_push(decor, gm.new(___struct___165__save_game__scr_save, d.part, d.x, d.y))
      j = j + 1
    end
    chunks[keys[k]] = gm.new(___struct___166__save_game__scr_save, trees, small, bushes, plants, decor, stumps)
    k = k + 1
  end
  local growths = gm.array()
  local i = 0
  while i < gm.array_length(mc.myc_growths) do
    local gr = mc.myc_growths[i]
    local brs = gm.array()
    local q = 0
    while q < gm.array_length(gr.branches) do
      local b = gr.branches[q]
      gm.array_push(brs, gm.new(___struct___167__save_game__scr_save, b.points, b.start, b.width, b.shoved, b.swept))
      q = q + 1
    end
    gm.array_push(growths, gm.new(___struct___168__save_game__scr_save, brs))
    i = i + 1
  end
  local mush = gm.array()
  i = 0
  while i < gm.array_length(global.mushrooms) do
    local m = global.mushrooms[i]
    if not gm.bool(m.stolen) then
      local c = save_copy_struct(self, other, m)
      if gm.is_undefined(m.host) then
        c.host = -1
      else
        local hid = m.host.save_id
        if gm.is_undefined(hid) then
          hid = -1
        end
        c.host = hid
      end
      c.stench = gm.array()
      c.crow_target = false
      if m.carried_by ~= -1 then
        c.y = wld(self, other, "surface_height_px")
        c.carried_by = -1
      end
      gm.array_push(mush, c)
    end
    i = i + 1
  end
  local waters = gm.array()
  i = 0
  while i < gm.array_length(global.water_sources) do
    local w = global.water_sources[i]
    if gm.is_struct(w.lake) then
      local lakes = world_get_chunk(self, other, w.lake.cx).lakes
      local q = 0
      while q < gm.array_length(lakes) do
        if lakes[q] == w.lake then
          gm.array_push(waters, gm.new(___struct___169__save_game__scr_save, w.lake.cx, q))
        end
        q = q + 1
      end
    else
      gm.array_push(waters, gm.new(___struct___170__save_game__scr_save, w.cx, w.idx))
    end
    i = i + 1
  end
  local ck = gm.variable_struct_get_names(global.world.chunks)
  k = 0
  while k < gm.array_length(ck) do
    local lakes = gm.variable_struct_get(global.world.chunks, ck[k]).lakes
    local q = 0
    while q < gm.array_length(lakes) do
      lakes[q].flares = gm.undefined
      q = q + 1
    end
    k = k + 1
  end
  local tut = global.tut
  local tasks = global.tasks
  local data = gm.new(___struct___171__save_game__scr_save,
    global.save_name, global.world, waters, chunks, fc.camera_x, fc.camera_y, growths, mc.myc_knots, mush,
    global.start_x, global.food, global.water_free, global.water_infinite, global.cave_perks, global.cave_taken,
    global.mycelium_length_px, global.play_frames, global.cave_yellow, global.relic_taken, game_mode(self, other),
    global.mycelium_points_awarded, global.upgrade_points, global.upgrade_levels,
    gm.array(global.cam_box_x1, global.cam_box_y1, global.cam_box_x2, global.cam_box_y2),
    gm.array(global.wind_dir, global.wind_speed, global.wind_timer),
    global.hedgehog_timer, global.boar_timer, global.squirrel_timer, global.crow_timer,
    global.experience_crow_timers, global.leaf_timer, global.worm_timer,
    tut.tips, tut.events, tut.ate, tut.drank, tut.hand_done,
    tasks.stats, tasks.done, tasks.open, tasks.shown, tasks.stage, tasks.goals, tasks.popup, tasks.goal_open)
  save_write_text(self, other, save_path(self, other, global.save_slot), gm.json_stringify(data))
  save_write_text(self, other, save_meta_path(self, other, global.save_slot), gm.json_stringify(
    gm.new(___struct___178__save_game__scr_save, global.save_name, gm.date_current_datetime(), mycelium_cm(self, other), game_mode(self, other))))
  return true
end

-- gml_Script____struct___180__load_game__scr_save @ 0x1401e3240
function ___struct___180__load_game__scr_save(self, other, lake)
  self.lake = lake
  self.cx = 0
  self.idx = 0
end

-- gml_Script____struct___181__load_game__scr_save @ 0x1401e3400
function ___struct___181__load_game__scr_save(self, other, lake, cx, idx)
  self.lake = lake
  self.cx = cx
  self.idx = idx
end

-- gml_Script____struct___188__load_game__scr_save @ 0x1401e3630
function ___struct___188__load_game__scr_save(self, other, sprite, x, y, roots, consume)
  self.sprite = sprite
  self.x = x
  self.y = y
  self.flip = 1
  self.roots = roots
  self.consume = consume
end

-- gml_Script____struct___189__load_game__scr_save @ 0x1401e3980
function ___struct___189__load_game__scr_save(self, other, sprite, x, y, roots)
  self.sprite = sprite
  self.x = x
  self.y = y
  self.flip = 1
  self.roots = roots
end

-- gml_Script____struct___190__load_game__scr_save @ 0x1401e3c70
function ___struct___190__load_game__scr_save(self, other, part, x, y, roots, consume)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
  self.roots = roots
  self.consume = consume
end

-- gml_Script____struct___191__load_game__scr_save @ 0x1401e3fc0
function ___struct___191__load_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___192__load_game__scr_save @ 0x1401e4250
function ___struct___192__load_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___193__load_game__scr_save @ 0x1401e44e0
function ___struct___193__load_game__scr_save(self, other, part, x, y)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
end

-- gml_Script____struct___194__load_game__scr_save @ 0x1401e4770
function ___struct___194__load_game__scr_save(self, other, trees, small_trees, bushes, plants, decor_trees, stumps)
  self.trees = trees
  self.small_trees = small_trees
  self.bushes = bushes
  self.plants = plants
  self.decor_trees = decor_trees
  self.stumps = stumps
end

-- gml_Script____struct___195__load_game__scr_save @ 0x1401e4ad0
function ___struct___195__load_game__scr_save(self, other, part, x, y, roots)
  self.part = part
  self.x = x
  self.y = y
  self.flip = 1
  self.roots = roots
end

-- gml_Script____struct___196__load_game__scr_save @ 0x1401e4dc0
function ___struct___196__load_game__scr_save(self, other, age, duration, finish_age, branches, vbuf, bx1, by1, bx2, by2)
  self.age = age
  self.duration = duration
  self.finish_age = finish_age
  self.branches = branches
  self.scanned_length = 0
  self.finished = true
  self.budding = false
  self.no_length = true
  self.vbuf = vbuf
  self.bx1 = bx1
  self.by1 = by1
  self.bx2 = bx2
  self.by2 = by2
end

-- gml_Script_load_game @ 0x1401e53e0
-- (decompiled separately: the export of this function timed out in re/decomp)
function load_game(self, other, slot)
  local txt = save_read_text(self, other, save_path(self, other, slot))
  if gm.is_undefined(txt) then
    return false
  end
  local ok, data = pcall(gm.json_parse, txt)
  if not ok then
    -- TODO(port): catch funclet not in decomp; assumed `catch (e) { return false; }`
    return false
  end
  local ver = data.version
  if gm.is_undefined(ver) then
    ver = 0
  end
  if ver ~= 1 then
    return false
  end
  local fc = gm.instance_find(obj_forest_controller, 0)
  local mc = gm.instance_find(obj_mycelium_controller, 0)
  local st = data.state
  local mode = st.mode
  if gm.is_undefined(mode) then
    mode = "exploration"
  end
  game_mode_apply(self, other, mode)
  global.world = data.world
  global.world.turf = gm.asset_get_index("trintrava1")
  local cy = st.cave_yellow
  if gm.is_struct(cy) then
    global.cave_yellow = cy
  else
    global.cave_yellow = gm.undefined
  end
  local pf = st.play_frames
  if gm.is_undefined(pf) then
    pf = 0
  end
  global.play_frames = pf
  local rt = st.relic_taken
  if gm.is_struct(rt) then
    global.relic_taken = rt
  else
    global.relic_taken = gm.struct({})
  end
  global.stones_moving = gm.array()
  local keys = gm.variable_struct_get_names(global.world.chunks)
  local i = 0
  while i < gm.array_length(keys) do
    local c = gm.variable_struct_get(global.world.chunks, keys[i])
    local j = 0
    while j < gm.array_length(c.stones) do
      local s = c.stones[j]
      s.moving = false
      s.vx = 0
      s.vy = 0
      s.spin = 0
      j = j + 1
    end
    j = 0
    while j < gm.array_length(c.lakes) do
      c.lakes[j].flares = gm.undefined
      j = j + 1
    end
    i = i + 1
  end
  global.water_sources = gm.array()
  i = 0
  while i < gm.array_length(data.water) do
    local w = data.water[i]
    if gm.variable_struct_exists(w, "lake_cx") then
      gm.array_push(global.water_sources, gm.new(___struct___180__load_game__scr_save,
        world_get_chunk(self, other, w.lake_cx).lakes[w.lake]))
    else
      gm.array_push(global.water_sources, gm.new(___struct___181__load_game__scr_save, gm.undefined, w.cx, w.idx))
    end
    i = i + 1
  end
  global.start_x = st.start_x
  global.food = st.food
  global.water_free = st.water_free
  local v = st.water_infinite
  if gm.is_undefined(v) then
    v = false
  end
  global.water_infinite = v
  v = st.cave_perks
  if gm.is_undefined(v) then
    v = gm.struct({})
  end
  global.cave_perks = v
  global.cave_flash = gm.struct({})
  v = st.cave_taken
  if gm.is_undefined(v) then
    v = gm.struct({})
  end
  global.cave_taken = v
  global.animal_markers = gm.array()
  global.cave_wrap = gm.struct({})
  global.cave_popups = gm.array()
  global.cave_burst_pending = false
  global.water_dip = 0
  global.water_dip_hold = 0
  global.mycelium_length_px = st.length_px
  global.mycelium_points_awarded = st.points_awarded
  global.upgrade_points = st.upgrade_points
  global.upgrade_levels = st.upgrade_levels
  global.cam_box_x1 = st.cam_box[0]
  global.cam_box_y1 = st.cam_box[1]
  global.cam_box_x2 = st.cam_box[2]
  global.cam_box_y2 = st.cam_box[3]
  global.wind_dir = st.wind[0]
  global.wind_speed = st.wind[1]
  global.wind_timer = st.wind[2]
  i = 0
  while i < gm.array_length(global.upgrade_defs) do
    local d = global.upgrade_defs[i]
    local owned
    if upg_is_owned(self, other, d.id) then
      owned = 1
    else
      owned = 0
    end
    gm.variable_struct_set(global.upgrade_grow, d.id, owned)
    i = i + 1
  end
  local def_tips = global.tut.tips
  global.tut.tips = st.tutorial.tips
  i = 0
  while i < gm.array_length(global.tut.order) do
    local key = global.tut.order[i]
    if not gm.variable_struct_exists(global.tut.tips, key) then
      gm.variable_struct_set(global.tut.tips, key, gm.variable_struct_get(def_tips, key))
    end
    i = i + 1
  end
  global.tut.events = st.tutorial.events
  global.tut.ate = st.tutorial.ate
  global.tut.drank = st.tutorial.drank
  global.tut.hand_done = st.tutorial.hand_done
  tasks_init(self, other)
  local ts = st.tasks
  if gm.is_struct(ts) then
    local names = gm.variable_struct_get_names(ts.stats)
    i = 0
    while i < gm.array_length(names) do
      gm.variable_struct_set(global.tasks.stats, names[i], gm.variable_struct_get(ts.stats, names[i]))
      i = i + 1
    end
    global.tasks.done = ts.done
    global.tasks.open = ts.open
    global.tasks.fold_k.tasks = gm.bool(ts.open) and 1 or 0
    v = ts.goal_open
    if gm.is_undefined(v) then
      v = global.tasks.goal_open
    end
    global.tasks.goal_open = v
    v = global.tasks.goal_open.migration
    if gm.is_undefined(v) then
      v = true
    end
    global.tasks.fold_k.migration = gm.bool(v) and 1 or 0
    v = global.tasks.goal_open.magic
    if gm.is_undefined(v) then
      v = true
    end
    global.tasks.fold_k.magic = gm.bool(v) and 1 or 0
    global.tasks.shown = ts.shown
    global.tasks.alpha = gm.bool(ts.shown) and 1 or 0
    v = ts.stage
    if gm.is_undefined(v) then
      v = 0
    end
    global.tasks.stage = v
    v = ts.goals
    if gm.is_undefined(v) then
      v = global.tasks.goals
    end
    global.tasks.goals = v
    v = ts.popup
    if gm.is_undefined(v) then
      v = ""
    end
    global.tasks.popup = v
    if global.tasks.popup == "migration" then
      global.tasks.popup = "migration_ready"
    end
    if global.tasks.popup == "migration_end" then
      global.tasks.popup = ""
    end
  end
  local by_id = gm.struct({})
  local start = gm.undefined
  local chunks = gm.struct({})
  local names = gm.variable_struct_get_names(data.forest.chunks)
  i = 0
  while i < gm.array_length(names) do
    local c = gm.variable_struct_get(data.forest.chunks, names[i])
    local trees = gm.array()
    local small = gm.array()
    local stumps = gm.array()
    local j = 0
    while j < gm.array_length(c.trees) do
      local t = c.trees[j]
      local tr = gm.new(___struct___188__load_game__scr_save, gm.asset_get_index(t.sprite), t.x, t.y, t.roots, t.consume)
      gm.variable_struct_set(by_id, gm.string(t.id), tr)
      gm.array_push(trees, tr)
      j = j + 1
    end
    j = 0
    while j < gm.array_length(c.small_trees) do
      local t = c.small_trees[j]
      gm.array_push(small, gm.new(___struct___189__load_game__scr_save, gm.asset_get_index(t.sprite), t.x, t.y, t.roots))
      j = j + 1
    end
    j = 0
    while j < gm.array_length(c.stumps) do
      local s = c.stumps[j]
      local sn = gm.new(___struct___190__load_game__scr_save, fc.stump_parts[s.part], s.x, s.y, s.roots, s.consume)
      if gm.is_struct(sn.roots) and not gm.is_undefined(sn.part.root_ax) then
        sn.roots.x = s.x + (sn.part.root_ax - sn.part.w * 0.5)
        sn.roots.flip = 1
      end
      gm.variable_struct_set(by_id, gm.string(s.id), sn)
      gm.array_push(stumps, sn)
      if gm.bool(s.start) then
        start = sn
      end
      j = j + 1
    end
    local bushes = gm.array()
    local src = c.bushes
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local b = src[j]
      gm.array_push(bushes, gm.new(___struct___191__load_game__scr_save, b.part, b.x, b.y))
      j = j + 1
    end
    local plants = gm.array()
    src = c.plants
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local p = src[j]
      gm.array_push(plants, gm.new(___struct___192__load_game__scr_save, p.part, p.x, p.y))
      j = j + 1
    end
    local decor = gm.array()
    src = c.decor_trees
    if gm.is_undefined(src) then
      src = gm.array()
    end
    j = 0
    while j < gm.array_length(src) do
      local d = src[j]
      gm.array_push(decor, gm.new(___struct___193__load_game__scr_save, d.part, d.x, d.y))
      j = j + 1
    end
    chunks[names[i]] = gm.new(___struct___194__load_game__scr_save, trees, small, bushes, plants, decor, stumps)
    i = i + 1
  end
  gm.with(self, fc, function(self, other)
    self.surface_chunks = chunks
    self.camera_x = data.forest.camera_x
    self.camera_y = data.forest.camera_y
    self.camera_vx = 0
    self.camera_vy = 0
    if gm.is_undefined(start) then
      self.start_stump = gm.new(___struct___195__load_game__scr_save, self.stump_parts[0], -1000000000, self.surface_height, gm.undefined)
    else
      self.start_stump = start
    end
    if gm.is_undefined(start) then
      self.start_stumps = gm.array()
    else
      self.start_stumps = gm.array(start)
    end
  end)
  gm.with(self, mc, function(self, other)
    self.myc_tiles_clear(self, other)
    local q = 0
    while q < gm.array_length(self.myc_growths) do
      local g = self.myc_growths[q]
      if gm.variable_struct_exists(g, "sound") then
        hyphae_sound_stop(self, other, g.sound)
      end
      self.myc_growth_free_chunks(self, other, g)
      if g.vbuf >= 0 then
        gm.vertex_delete_buffer(g.vbuf)
      end
      q = q + 1
    end
    self.myc_growths = gm.array()
    q = 0
    while q < gm.array_length(data.mycelium.growths) do
      local brs = data.mycelium.growths[q].branches
      local nb = gm.array()
      local mx = 0
      local r = 0
      while r < gm.array_length(brs) do
        local b = brs[r]
        local nbr = self.myc_build_branch(self, other, b.points, b.start, b.width, false)
        nbr.shoved = b.shoved
        nbr.swept = b.swept
        mx = gm.max(mx, b.start)
        gm.array_push(nb, nbr)
        r = r + 1
      end
      if gm.array_length(nb) ~= 0 then
        local bx1 = nb[0].bx1
        local by1 = nb[0].by1
        local bx2 = nb[0].bx2
        local by2 = nb[0].by2
        r = 1
        while r < gm.array_length(nb) do
          bx1 = gm.min(bx1, nb[r].bx1)
          by1 = gm.min(by1, nb[r].by1)
          bx2 = gm.max(bx2, nb[r].bx2)
          by2 = gm.max(by2, nb[r].by2)
          r = r + 1
        end
        local age = self.myc_growth_duration + mx + 2
        gm.array_push(self.myc_growths, gm.new(___struct___196__load_game__scr_save,
          age, self.myc_growth_duration, age, nb, -1, bx1, by1, bx2, by2))
      end
      q = q + 1
    end
    local cx = global.start_x
    local cy = 0
    if gm.instance_exists(obj_forest_controller) then
      local fci = gm.instance_find(obj_forest_controller, 0)
      cx = fci.camera_x + fci.camera_width * 0.5
      cy = fci.camera_y + fci.camera_height * 0.5
    end
    self.myc_bake_queue_all(self, other, cx, cy)
    self.myc_knots = data.mycelium.knots
    self.myc_spores = gm.array()
    self.myc_dragging = false
    self.myc_has_aim = false
    self.myc_clear_prompts(self, other)
    self.x = global.start_x
    self.myc_grid_rebuild(self, other)
    self.myc_burst_queue = gm.array()
    self.myc_burst_next = 0
    self.myc_burst_show = gm.array()
    self.myc_burst_show_next = 0
    self.myc_burst_growing = gm.array()
    self.myc_digger_queue = gm.array()
    self.myc_digger_next = 0
  end)
  global.mushrooms = gm.array()
  i = 0
  while i < gm.array_length(data.mushrooms) do
    local m = data.mushrooms[i]
    if m.host >= 0 then
      m.host = gm.variable_struct_get(by_id, gm.string(m.host))
    else
      m.host = gm.undefined
    end
    m.stench = gm.array()
    m.stench_due = 0
    m.carried_by = -1
    m.stolen = false
    m.crow_target = false
    if gm.variable_struct_exists(m, "born") then
      gm.variable_struct_remove(m, "born")
    end
    gm.array_push(global.mushrooms, m)
    i = i + 1
  end
  global.animals = gm.array()
  global.boars = gm.array()
  global.boar_flung = gm.array()
  global.boar_pack = gm.array()
  global.repel_clouds = gm.array()
  global.squirrels = gm.array()
  global.crows = gm.array()
  global.crow_pending = gm.array(gm.undefined, gm.undefined, gm.undefined)
  global.leaves = gm.array()
  global.leaf_tendrils_out = gm.array()
  global.drops = gm.array()
  global.worms = gm.array()
  global.worm_trail = gm.array()
  global.backdrop_leaves = gm.undefined
  global.hedgehog_timer = st.timers.hedgehog
  v = st.timers.boar
  if gm.is_undefined(v) then
    v = bal(self, other, "boar_spawn_interval") * global.fps
  end
  global.boar_timer = v
  global.squirrel_timer = st.timers.squirrel
  global.crow_timer = st.timers.crow
  global.experience_crow_timers = st.timers.experience_crows
  global.leaf_timer = st.timers.leaf
  global.worm_timer = st.timers.worm
  life_far_init(self, other)
  weather_init(self, other)
  global.intro_active = false
  global.title_open = false
  global.save_slot = slot
  global.save_name = data.name
  -- TODO(port): target global's vid is missing in the decompilation; assumed save_timer (as in save_game)
  global.save_timer = bal(self, other, "save_autosave_seconds") * global.fps
  return true
end

-- gml_GlobalScript_scr_save @ 0x1401f48d0
function gml_GlobalScript_scr_save(self, other)
  self.save_init = gm.method(self, save_init)
  self.save_path = gm.method(self, save_path)
  self.save_meta_path = gm.method(self, save_meta_path)
  self.save_write_text = gm.method(self, save_write_text)
  self.save_read_text = gm.method(self, save_read_text)
  self.save_slot_meta = gm.method(self, save_slot_meta)
  self.save_any_exists = gm.method(self, save_any_exists)
  self.save_start_new = gm.method(self, save_start_new)
  self.save_step = gm.method(self, save_step)
  self.save_copy_struct = gm.method(self, save_copy_struct)
  self.save_game = gm.method(self, save_game)
  self.load_game = gm.method(self, load_game)
end
