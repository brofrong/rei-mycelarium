local gm = require("gm")

local function argv(v)
  if v == nil then return gm.undefined end
  return v
end

-- gml_Script____struct___249__upgrades_init__scr_upgrades @ 0x140290700
function ___struct___249__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "shot_range"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 1.0
  self.px = argv(a0)
  self.py = 103.0
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_shot_range_max_level"
  self.step_key = "upg_shot_range_per_level"
end

-- gml_Script____struct___250__upgrades_init__scr_upgrades @ 0x140290ba0
function ___struct___250__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "shot_width"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = 185.0
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_shot_width_max_level"
  self.step_key = "upg_shot_width_per_level"
end

-- gml_Script____struct___251__upgrades_init__scr_upgrades @ 0x140291050
function ___struct___251__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "water_efficiency"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = 245.0
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_water_efficiency_max_level"
  self.step_key = "upg_water_efficiency_per_level"
end

-- gml_Script____struct___252__upgrades_init__scr_upgrades @ 0x140291500
function ___struct___252__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "budding_chance"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 2.0
  self.px = 289.0
  self.py = 245.0
  self.available = true
  self.icon = argv(a0)
  self.max_key = "upg_budding_chance_max_level"
  self.step_key = "upg_budding_chance_per_level"
end

-- gml_Script____struct___253__upgrades_init__scr_upgrades @ 0x140291990
function ___struct___253__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "node_efficiency"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 3.0
  self.px = 554.0
  self.py = 185.0
  self.available = true
  self.icon = argv(a0)
  self.max_key = "upg_node_efficiency_max_level"
  self.step_key = "upg_node_efficiency_per_level"
end

-- gml_Script____struct___254__upgrades_init__scr_upgrades @ 0x140291e20
function ___struct___254__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "soil_penetration"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 3.0
  self.px = 780.0
  self.py = 103.0
  self.available = true
  self.icon = argv(a0)
  self.max_key = "upg_soil_penetration_max_level"
  self.step_key = "upg_soil_penetration_per_level"
end

-- gml_Script____struct___255__upgrades_init__scr_upgrades @ 0x1402922b0
function ___struct___255__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_absorption"
  self.trunk = "enzymes"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 1.0
  self.px = 0
  self.py = 290.0
  self.available = true
  self.icon = leaf
  self.cost_key = "upg_unlock_absorption_cost"
end

-- gml_Script____struct___256__upgrades_init__scr_upgrades @ 0x1402926e0
function ___struct___256__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "absorption_speed"
  self.trunk = "enzymes"
  self.kind = "level"
  self.parent = "unlock_absorption"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = 405.0
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_absorption_speed_max_level"
  self.step_key = "upg_absorption_speed_per_level"
end

-- gml_Script____struct___257__upgrades_init__scr_upgrades @ 0x140292b90
function ___struct___257__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "absorption_yield"
  self.trunk = "enzymes"
  self.kind = "level"
  self.parent = "unlock_absorption"
  self.req = 1.0
  self.tier = 2.0
  self.px = 0
  self.py = 407.0
  self.available = true
  self.icon = argv(a0)
  self.max_key = "upg_absorption_yield_max_level"
  self.step_key = "upg_absorption_yield_per_level"
end

-- gml_Script____struct___258__upgrades_init__scr_upgrades @ 0x140293010
function ___struct___258__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_protein"
  self.trunk = "enzymes"
  self.kind = "unlock"
  self.parent = "unlock_absorption"
  self.req = 1.0
  self.tier = 3.0
  self.px = 166.0
  self.py = 405.0
  self.available = true
  self.icon = argv(a0)
  self.cost_key = "upg_unlock_protein_cost"
end

-- gml_Script____struct___259__upgrades_init__scr_upgrades @ 0x140293450
function ___struct___259__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_repeller"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.bx = argv(a2)
  self.by = 40.0
  self.available = true
  self.icon = explosiveshroom
  self.cost_key = "upg_unlock_repeller_cost"
end

-- gml_Script____struct___260__upgrades_init__scr_upgrades @ 0x140293980
function ___struct___260__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "repeller_autoregen"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_repeller"
  self.req = 1.0
  self.tier = 3.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_repeller_autoregen_max_level"
  self.step_key = "upg_repeller_autoregen_per_level"
end

-- gml_Script____struct___261__upgrades_init__scr_upgrades @ 0x140293e50
function ___struct___261__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "repeller_regen_cost"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_repeller"
  self.req = 1.0
  self.tier = 3.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_repeller_regen_cost_max_level"
  self.step_key = "upg_repeller_regen_cost_per_level"
end

-- gml_Script____struct___262__upgrades_init__scr_upgrades @ 0x140294320
function ___struct___262__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_predator"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 1.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = predatorshroom
  self.cost_key = "upg_unlock_predator_cost"
end

-- gml_Script____struct___263__upgrades_init__scr_upgrades @ 0x1402947a0
function ___struct___263__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "predator_digestion"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_predator"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_predator_digestion_max_level"
  self.step_key = "upg_predator_digestion_per_level"
end

-- gml_Script____struct___264__upgrades_init__scr_upgrades @ 0x140294c70
function ___struct___264__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "predator_yield"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_predator"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_predator_yield_max_level"
  self.step_key = "upg_predator_yield_per_level"
end

-- gml_Script____struct___265__upgrades_init__scr_upgrades @ 0x140295140
function ___struct___265__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_tree"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 2.0
  self.px = 407.0
  self.py = argv(a0)
  self.available = true
  self.icon = treeshroom
  self.cost_key = "upg_unlock_tree_cost"
end

-- gml_Script____struct___266__upgrades_init__scr_upgrades @ 0x1402955b0
function ___struct___266__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "tree_water_yield"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_tree"
  self.req = 1.0
  self.tier = 3.0
  self.px = 328.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_tree_water_yield_max_level"
  self.step_key = "upg_tree_water_yield_per_level"
end

-- gml_Script____struct___267__upgrades_init__scr_upgrades @ 0x140295a60
function ___struct___267__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "tree_drop_rate"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_tree"
  self.req = 1.0
  self.tier = 3.0
  self.px = 488.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_tree_drop_rate_max_level"
  self.step_key = "upg_tree_drop_rate_per_level"
end

-- gml_Script____struct___268__upgrades_init__scr_upgrades @ 0x140295f10
function ___struct___268__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_parasitism"
  self.trunk = "enzymes"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 3.0
  self.px = 820.0
  self.py = argv(a0)
  self.bx = 406.0
  self.by = 40.0
  self.available = true
  self.icon = consumptorshroom
  self.cost_key = "upg_unlock_parasitism_cost"
end

-- gml_Script____struct___269__upgrades_init__scr_upgrades @ 0x1402963f0
function ___struct___269__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "parasitism_speed"
  self.trunk = "enzymes"
  self.kind = "level"
  self.parent = "unlock_parasitism"
  self.req = 1.0
  self.tier = 4.0
  self.px = 741.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_parasitism_speed_max_level"
  self.step_key = "upg_parasitism_speed_per_level"
end

-- gml_Script____struct___270__upgrades_init__scr_upgrades @ 0x1402968a0
function ___struct___270__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "parasitism_yield"
  self.trunk = "enzymes"
  self.kind = "level"
  self.parent = "unlock_parasitism"
  self.req = 1.0
  self.tier = 4.0
  self.px = 901.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_parasitism_yield_max_level"
  self.step_key = "upg_parasitism_yield_per_level"
end

-- gml_Script____struct___271__upgrades_init__scr_upgrades @ 0x140296d50
function ___struct___271__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_spore"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 1.0
  self.px = 0
  self.py = argv(a0)
  self.available = true
  self.icon = sporeshroom
  self.cost_key = "upg_unlock_spore_cost"
end

-- gml_Script____struct___272__upgrades_init__scr_upgrades @ 0x1402971b0
function ___struct___272__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "spore_count"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_spore"
  self.req = 1.0
  self.tier = 2.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_spore_count_max_level"
  self.step_key = "upg_spore_count_per_level"
end

-- gml_Script____struct___273__upgrades_init__scr_upgrades @ 0x140297680
function ___struct___273__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "spore_quality"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_spore"
  self.req = 1.0
  self.tier = 2.0
  self.px = 0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_spore_quality_max_level"
  self.step_key = "upg_spore_quality_per_level"
end

-- gml_Script____struct___274__upgrades_init__scr_upgrades @ 0x140297b20
function ___struct___274__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "spore_lightness"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_spore"
  self.req = 1.0
  self.tier = 3.0
  self.px = 127.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_spore_lightness_max_level"
  self.step_key = "upg_spore_lightness_per_level"
end

-- gml_Script____struct___275__upgrades_init__scr_upgrades @ 0x140297fd0
function ___struct___275__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_experience"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 3.0
  self.px = 0
  self.py = argv(a0)
  self.available = true
  self.icon = experienceshroom
  self.cost_key = "upg_unlock_experience_cost"
end

-- gml_Script____struct___276__upgrades_init__scr_upgrades @ 0x140298430
function ___struct___276__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_digger"
  self.trunk = "hyphae"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 4.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = diggershroom
  self.cost_key = "upg_unlock_digger_cost"
end

-- gml_Script____struct___277__upgrades_init__scr_upgrades @ 0x1402988c0
function ___struct___277__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "digger_length"
  self.trunk = "hyphae"
  self.kind = "level"
  self.parent = "unlock_digger"
  self.req = 1.0
  self.tier = 5.0
  self.px = argv(a0)
  self.py = argv(a1)
  self.available = true
  self.icon = argv(a2)
  self.max_key = "upg_digger_length_max_level"
  self.step_key = "upg_digger_length_per_level"
end

-- gml_Script____struct___278__upgrades_init__scr_upgrades @ 0x140298d90
function ___struct___278__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "unlock_stinky"
  self.trunk = "fruiting"
  self.kind = "unlock"
  self.parent = "origin"
  self.req = 1.0
  self.tier = 4.0
  self.px = 397.0
  self.py = argv(a0)
  self.available = true
  self.icon = stinkyshroom
  self.cost_key = "upg_unlock_stinky_cost"
end

-- gml_Script____struct___279__upgrades_init__scr_upgrades @ 0x140299200
function ___struct___279__upgrades_init__scr_upgrades(self, other, a0, a1, a2)
  self.id = "stinky_width"
  self.trunk = "fruiting"
  self.kind = "level"
  self.parent = "unlock_stinky"
  self.req = 1.0
  self.tier = 5.0
  self.px = 397.0
  self.py = argv(a0)
  self.available = true
  self.icon = argv(a1)
  self.max_key = "upg_stinky_width_max_level"
  self.step_key = "upg_stinky_width_per_level"
end
-- gml_Script_upgrades_init @ 0x1402996b0
-- (decompiled separately: original Ghidra export timed out)
function upgrades_init(self, other)
  global.upgrade_points = 0
  global.upgrade_levels = gm.struct({})
  global.upgrade_levels.origin = 1.0
  global.upgrade_trunks = gm.array()
  global.upgrade_defs = gm.array(
    gm.new(___struct___248__upgrades_init__scr_upgrades, -0.5),
    gm.new(___struct___249__upgrades_init__scr_upgrades, -775.0, -1.0),
    gm.new(___struct___250__upgrades_init__scr_upgrades, -558.0, -1.0),
    gm.new(___struct___251__upgrades_init__scr_upgrades, -288.0, -1.0),
    gm.new(___struct___252__upgrades_init__scr_upgrades, -1.0),
    gm.new(___struct___253__upgrades_init__scr_upgrades, -1.0),
    gm.new(___struct___254__upgrades_init__scr_upgrades, -1.0),
    gm.new(___struct___255__upgrades_init__scr_upgrades),
    gm.new(___struct___256__upgrades_init__scr_upgrades, -161.0, -1.0),
    gm.new(___struct___257__upgrades_init__scr_upgrades, -1.0),
    gm.new(___struct___258__upgrades_init__scr_upgrades, -1.0),
    gm.new(___struct___259__upgrades_init__scr_upgrades, -820.0, -118.0, -406.0),
    gm.new(___struct___260__upgrades_init__scr_upgrades, -904.0, -272.0, -1.0),
    gm.new(___struct___261__upgrades_init__scr_upgrades, -739.0, -275.0, -1.0),
    gm.new(___struct___262__upgrades_init__scr_upgrades, -408.0, -113.0),
    gm.new(___struct___263__upgrades_init__scr_upgrades, -486.0, -275.0, -1.0),
    gm.new(___struct___264__upgrades_init__scr_upgrades, -328.0, -275.0, -1.0),
    gm.new(___struct___265__upgrades_init__scr_upgrades, -105.0),
    gm.new(___struct___266__upgrades_init__scr_upgrades, -272.0, -1.0),
    gm.new(___struct___267__upgrades_init__scr_upgrades, -272.0, -1.0),
    gm.new(___struct___268__upgrades_init__scr_upgrades, -105.0),
    gm.new(___struct___269__upgrades_init__scr_upgrades, -272.0, -1.0),
    gm.new(___struct___270__upgrades_init__scr_upgrades, -272.0, -1.0),
    gm.new(___struct___271__upgrades_init__scr_upgrades, -178.0),
    gm.new(___struct___272__upgrades_init__scr_upgrades, -126.0, -345.0, -1.0),
    gm.new(___struct___273__upgrades_init__scr_upgrades, -345.0, -1.0),
    gm.new(___struct___274__upgrades_init__scr_upgrades, -345.0, -1.0),
    gm.new(___struct___275__upgrades_init__scr_upgrades, -520.0),
    gm.new(___struct___276__upgrades_init__scr_upgrades, -397.0, -467.0),
    gm.new(___struct___277__upgrades_init__scr_upgrades, -397.0, -632.0, -1.0),
    gm.new(___struct___278__upgrades_init__scr_upgrades, -467.0),
    gm.new(___struct___279__upgrades_init__scr_upgrades, -632.0, -1.0)
  )
  global.upgrade_grow = gm.struct({})
  local i = 0
  while i < gm.array_length(global.upgrade_defs) do
    local id = global.upgrade_defs[i].id
    gm.variable_struct_set(global.upgrade_grow, id, upg_is_owned(self, other, id) and 1.0 or 0)
    i = i + 1
  end
end

-- gml_Script_upg_def @ 0x14029e860
function upg_def(self, other, id)
  local i = 0
  while i < gm.array_length(global.upgrade_defs) do
    if global.upgrade_defs[i].id == id then
      return global.upgrade_defs[i]
    end
    i = i + 1
  end
  return gm.undefined
end

-- gml_Script____struct___281__upg_tree_pos__scr_upgrades @ 0x14029eb90
function ___struct___281__upg_tree_pos__scr_upgrades(self, other, a0, a1)
  self.x = argv(a0)
  self.y = argv(a1)
end

-- gml_Script_upg_tree_pos @ 0x14029ed90
function upg_tree_pos(self, other, def)
  return gm.new(___struct___281__upg_tree_pos__scr_upgrades,
    def.px * vis(self, other, "tree_layout_scale_x"),
    def.py * vis(self, other, "tree_layout_scale_y"))
end

-- gml_Script____struct___282__upg_tree_via__scr_upgrades @ 0x14029f2b0
function ___struct___282__upg_tree_via__scr_upgrades(self, other, a0, a1)
  self.x = argv(a0)
  self.y = argv(a1)
end

-- gml_Script_upg_tree_via @ 0x14029f4b0
function upg_tree_via(self, other, def)
  if not gm.variable_struct_exists(def, "bx") then
    return gm.undefined
  end
  local bx = def.bx
  local by = def.by
  local x = bx * vis(self, other, "tree_layout_scale_x")
  local y = by * vis(self, other, "tree_layout_scale_y")
  return gm.new(___struct___282__upg_tree_via__scr_upgrades, x, y)
end

-- gml_Script_upg_children @ 0x14029fb40
function upg_children(self, other, id)
  local out = gm.array()
  local i = 0
  while i < gm.array_length(global.upgrade_defs) do
    if global.upgrade_defs[i].parent == id then
      gm.array_push(out, global.upgrade_defs[i])
    end
    i = i + 1
  end
  return out
end

-- gml_Script_upg_level @ 0x1402a0020
function upg_level(self, other, id)
  local v = gm.variable_struct_get(global.upgrade_levels, id)
  if gm.is_undefined(v) then
    return 0
  end
  return v
end

-- gml_Script_upg_max_level @ 0x1402a0380
function upg_max_level(self, other, id)
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) then
    return 0
  end
  if def.kind == "level" then
    return bal(self, other, def.max_key)
  end
  return 1
end

-- gml_Script_upg_bonus @ 0x1402a07b0
function upg_bonus(self, other, id)
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) or def.kind ~= "level" then
    return 0
  end
  local step = bal(self, other, def.step_key)
  return upg_level(self, other, id) * step
end

-- gml_Script_upg_is_owned @ 0x1402a0cf0
function upg_is_owned(self, other, id)
  return upg_level(self, other, id) > 0
end

-- gml_Script_upg_requirement_met @ 0x1402a0ec0
function upg_requirement_met(self, other, id)
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) then
    return false
  end
  if def.parent == "" then
    return true
  end
  return upg_level(self, other, def.parent) >= def.req
end

-- gml_Script_upg_cost @ 0x1402a1310
function upg_cost(self, other, id)
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) or def.kind == "root" or def.kind == "ghost" then
    return 0
  end
  if def.kind == "unlock" then
    return bal(self, other, def.cost_key)
  end
  local tier_mult = gm.power(bal(self, other, "upgrade_cost_tier_mult"), gm.max(0, def.tier - 1))
  local growth = bal(self, other, "upgrade_cost_growth")
  local override = gm.variable_struct_get(global.balance, "upg_" .. id .. "_cost_growth")
  if not gm.is_undefined(override) then
    growth = override
  end
  local lvl = upg_level(self, other, id)
  return gm.ceil(bal(self, other, "upgrade_cost_base") * tier_mult * gm.power(growth, lvl))
end

-- gml_Script_upg_demo_locked @ 0x1402a2410
local upg_demo_locked_static = nil
function upg_demo_locked(self, other, id)
  if upg_demo_locked_static == nil then
    upg_demo_locked_static = {}
    upg_demo_locked_static._open = gm.array("shot_range", "budding_chance", "unlock_spore",
      "unlock_predator", "unlock_absorption", "absorption_yield")
  end
  local demo_only = bal(self, other, "demo_upgrades_only")
  if type(demo_only) == "number" and demo_only <= 0 then
    return false
  end
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) or def.kind == "root" then
    return false
  end
  return not gm.array_contains(upg_demo_locked_static._open, id)
end

-- gml_Script_upg_can_buy @ 0x1402a2aa0
function upg_can_buy(self, other, id)
  local def = upg_def(self, other, id)
  if upg_demo_locked(self, other, id) then
    return false
  end
  if gm.is_undefined(def) or not gm.bool(def.available) or def.kind == "root" or def.kind == "ghost" then
    return false
  end
  if not gm.bool(upg_requirement_met(self, other, id)) then
    return false
  end
  local mx = upg_max_level(self, other, id)
  if upg_level(self, other, id) >= mx then
    return false
  end
  return global.upgrade_points >= upg_cost(self, other, id)
end

-- gml_Script_upg_buy @ 0x1402a3280
function upg_buy(self, other, id)
  if not gm.bool(upg_can_buy(self, other, id)) then
    return false
  end
  global.upgrade_points = global.upgrade_points - upg_cost(self, other, id)
  gm.variable_struct_set(global.upgrade_levels, id, upg_level(self, other, id) + 1)
  return true
end

-- gml_Script_upg_state @ 0x1402a3640
function upg_state(self, other, id)
  local def = upg_def(self, other, id)
  if gm.is_undefined(def) then
    return "locked"
  end
  if def.kind == "root" then
    return "root"
  end
  if upg_demo_locked(self, other, id) then
    return "demo"
  end
  if def.kind == "ghost" or not gm.bool(def.available) then
    return "ghost"
  end
  if not gm.bool(upg_requirement_met(self, other, id)) then
    return "locked"
  end
  local mx = upg_max_level(self, other, id)
  if upg_level(self, other, id) >= mx then
    return "maxed"
  end
  if global.upgrade_points >= upg_cost(self, other, id) then
    return "ready"
  end
  if upg_is_owned(self, other, id) then
    return "owned"
  end
  return "short"
end

-- gml_Script_upgrades_grow_step @ 0x1402a40d0
function upgrades_grow_step(self, other)
  local i = 0
  while i < gm.array_length(global.upgrade_defs) do
    local id = global.upgrade_defs[i].id
    local target = upg_is_owned(self, other, id) and 1.0 or 0
    local len = gm.max(1.0, tree_hypha_length(self, other, id))
    local rate = vis(self, other, "tree_grow_speed") / len
    local cur = gm.variable_struct_get(global.upgrade_grow, id)
    local v
    if type(cur) ~= "number" or not (target > cur) then
      v = target
    else
      v = gm.min(target, gm.variable_struct_get(global.upgrade_grow, id) + rate)
    end
    gm.variable_struct_set(global.upgrade_grow, id, v)
    i = i + 1
  end
end

-- gml_Script_upg_grow @ 0x1402a4cd0
function upg_grow(self, other, id)
  local v = gm.variable_struct_get(global.upgrade_grow, id)
  if gm.is_undefined(v) then
    return 0
  end
  return v
end

-- gml_Script_myc_shot_range @ 0x1402a5030
function myc_shot_range(self, other)
  local m = 1.0 + upg_bonus(self, other, "shot_range")
  return bal(self, other, "base_shot_range") * m
end

-- gml_Script_myc_shot_range_max @ 0x1402a52c0
function myc_shot_range_max(self, other)
  local per = bal(self, other, "upg_shot_range_per_level")
  local m = 1.0 + bal(self, other, "upg_shot_range_max_level") * per
  return bal(self, other, "base_shot_range") * m
end

-- gml_Script_myc_digger_range @ 0x1402a5650
function myc_digger_range(self, other)
  local m = 1.0 + upg_bonus(self, other, "digger_length")
  return myc_shot_range_max(self, other) * m
end

-- gml_Script_myc_shot_spread @ 0x1402a58c0
function myc_shot_spread(self, other)
  local m = 1.0 + upg_bonus(self, other, "shot_width")
  return bal(self, other, "base_shot_spread") * m
end

-- gml_Script_myc_shot_water_cost @ 0x1402a5b50
function myc_shot_water_cost(self, other)
  if gm.bool(weather_is_raining(self, other)) then
    return 0
  end
  local m = gm.max(0.25, 1.0 - upg_bonus(self, other, "water_efficiency"))
  return bal(self, other, "base_shot_water_cost") * m
end

-- gml_Script_myc_shot_water_rate @ 0x1402a5f60
function myc_shot_water_rate(self, other)
  local d = gm.max(1.0, bal(self, other, "base_shot_range"))
  return myc_shot_water_cost(self, other) / d
end

-- gml_Script_myc_node_food_cost @ 0x1402a6230
function myc_node_food_cost(self, other)
  local m = gm.max(0.2, 1.0 - upg_bonus(self, other, "node_efficiency"))
  return gm.max(1.0, gm.round(bal(self, other, "base_node_food_cost") * m))
end

-- gml_Script_myc_budding_chance @ 0x1402a66b0
function myc_budding_chance(self, other)
  local b = upg_bonus(self, other, "budding_chance")
  return gm.clamp(gm.real(bal(self, other, "base_random_node_chance") + b), 0, 1.0)
end

-- gml_Script_myc_soil_penetration @ 0x1402a6920
function myc_soil_penetration(self, other)
  local b = upg_bonus(self, other, "soil_penetration")
  return bal(self, other, "base_hardsoil_penetration") + b
end

-- gml_GlobalScript_scr_upgrades @ 0x1402a6b70
function gml_GlobalScript_scr_upgrades(self, other)
  self.upgrades_init = gm.method(self, upgrades_init)
  self.upg_def = gm.method(self, upg_def)
  self.upg_tree_pos = gm.method(self, upg_tree_pos)
  self.upg_tree_via = gm.method(self, upg_tree_via)
  self.upg_children = gm.method(self, upg_children)
  self.upg_level = gm.method(self, upg_level)
  self.upg_max_level = gm.method(self, upg_max_level)
  self.upg_bonus = gm.method(self, upg_bonus)
  self.upg_is_owned = gm.method(self, upg_is_owned)
  self.upg_requirement_met = gm.method(self, upg_requirement_met)
  self.upg_cost = gm.method(self, upg_cost)
  self.upg_demo_locked = gm.method(self, upg_demo_locked)
  self.upg_can_buy = gm.method(self, upg_can_buy)
  self.upg_buy = gm.method(self, upg_buy)
  self.upg_state = gm.method(self, upg_state)
  self.upgrades_grow_step = gm.method(self, upgrades_grow_step)
  self.upg_grow = gm.method(self, upg_grow)
  self.myc_shot_range = gm.method(self, myc_shot_range)
  self.myc_shot_range_max = gm.method(self, myc_shot_range_max)
  self.myc_digger_range = gm.method(self, myc_digger_range)
  self.myc_shot_spread = gm.method(self, myc_shot_spread)
  self.myc_shot_water_cost = gm.method(self, myc_shot_water_cost)
  self.myc_shot_water_rate = gm.method(self, myc_shot_water_rate)
  self.myc_node_food_cost = gm.method(self, myc_node_food_cost)
  self.myc_budding_chance = gm.method(self, myc_budding_chance)
  self.myc_soil_penetration = gm.method(self, myc_soil_penetration)
end

-- gml_Script____struct___284__weather_init__scr_weather @ 0x1402b0600
function ___struct___284__weather_init__scr_weather(self, other, a0, a1, a2, a3, a4, a5, a6)
  self.phase = 0
  self.timer = argv(a0)
  self.intensity = 0
  self.active = 0
  self.wind = 0
  self.gust = 1.0
  self.gust_target = 1.0
  self.gust_timer = 0
  self.gloom = 0
  self.colour = argv(a1)
  self.gloom_colour = argv(a2)
  self.bands = argv(a3)
  self.drops = argv(a4)
  self.splashes = argv(a5)
  self.spray = argv(a6)
end

-- gml_Script____struct___285__weather_init__scr_weather @ 0x1402b0bc0
function ___struct___285__weather_init__scr_weather(self, other)
  self.speed = 13.0
  self.drift = 0.55
  self.len = 13.0
  self.width = 1.0
  self.alpha = 0.2
  self.splash = 0.55
end

-- gml_Script____struct___286__weather_init__scr_weather @ 0x1402b0e30
function ___struct___286__weather_init__scr_weather(self, other)
  self.speed = 19.0
  self.drift = 0.85
  self.len = 22.0
  self.width = 1.5
  self.alpha = 0.32
  self.splash = 0.85
end

-- gml_Script____struct___287__weather_init__scr_weather @ 0x1402b10a0
function ___struct___287__weather_init__scr_weather(self, other)
  self.speed = 26.0
  self.drift = 1.15
  self.len = 34.0
  self.width = 2.3
  self.alpha = 0.46
  self.splash = 1.2
end

-- gml_Script____struct___288__weather_init__scr_weather @ 0x1402b1310
function ___struct___288__weather_init__scr_weather(self, other, a0)
  self.band = argv(a0)
  self.x = 0
  self.y = 0
  self.ground = 0
  self.speed_scale = 1.0
  self.len_scale = 1.0
  self.alpha_scale = 1.0
end

-- gml_Script_weather_init @ 0x1402b1630
function weather_init(self, other)
  local b1 = gm.new(___struct___285__weather_init__scr_weather)
  local b2 = gm.new(___struct___286__weather_init__scr_weather)
  local b3 = gm.new(___struct___287__weather_init__scr_weather)
  local bands = gm.array(b1, b2, b3)
  global.weather = gm.new(___struct___284__weather_init__scr_weather,
    vis(self, other, "rain_first_gap_frames"), 15521990.0, 11701370.0,
    bands, gm.array(), gm.array(), gm.array())
  local pattern = gm.array(0, 1.0, 2.0, 0, 1.0, 0)
  local n = gm.max(1.0, gm.round(vis(self, other, "rain_max_drops")))
  local drops = gm.array_create(n)
  local i = 0
  while i < n do
    local idx = math.floor(math.fmod(i, gm.array_length(pattern)))
    drops[i] = gm.new(___struct___288__weather_init__scr_weather, pattern[idx])
    i = i + 1
  end
  global.weather.drops = drops
end

-- gml_Script_weather_is_raining @ 0x1402b2780
function weather_is_raining(self, other)
  return global.weather.intensity > 0
end

-- gml_Script_weather_force_rain @ 0x1402b28b0
function weather_force_rain(self, other)
  local w = global.weather
  if w.phase == 0 or w.phase == 3.0 then
    rain_begin(self, other, w)
    return true
  end
  w.phase = 3.0
  w.timer = vis(self, other, "rain_fade_out_frames") * w.intensity
  return false
end

-- gml_Script_rain_begin @ 0x1402b2dd0
function rain_begin(self, other, w)
  w.phase = 1.0
  local fade = gm.max(1.0, vis(self, other, "rain_fade_in_frames"))
  w.timer = fade * (1.0 - w.intensity)
  w.gust = 1.0
  w.gust_target = 1.0
  w.gust_timer = 0
end

-- gml_Script_rain_phase_step @ 0x1402b3360
function rain_phase_step(self, other, w)
  w.timer = w.timer - 1.0
  local phase = w.phase
  if phase == 0 then
    w.intensity = 0
    if w.timer <= 0 then
      rain_begin(self, other, w)
    end
  elseif phase == 1.0 then
    local fade = gm.max(1.0, vis(self, other, "rain_fade_in_frames"))
    w.intensity = gm.clamp(gm.real(1.0 - w.timer / fade), 0, 1.0)
    if w.timer <= 0 then
      w.phase = 2.0
      local r = gm.random_range(0.7, 1.4)
      w.timer = vis(self, other, "rain_hold_frames") * r
    end
  elseif phase == 2.0 then
    w.intensity = 1.0
    if w.timer <= 0 then
      w.phase = 3.0
      w.timer = gm.max(1.0, vis(self, other, "rain_fade_out_frames"))
    end
  elseif phase == 3.0 then
    local fade = gm.max(1.0, vis(self, other, "rain_fade_out_frames"))
    w.intensity = gm.clamp(gm.real(w.timer / fade), 0, 1.0)
    if w.timer <= 0 then
      w.phase = 0
      local gmax = vis(self, other, "rain_gap_max_frames")
      local gmin = vis(self, other, "rain_gap_min_frames")
      w.timer = gm.irandom_range(gmin, gmax)
    end
  end
  rain_gloom_step(self, other, w)
end

-- gml_Script_rain_gloom_step @ 0x1402b4660
function rain_gloom_step(self, other, w)
  local target = 0
  local phase = w.phase
  if phase == 0 then
    local lead = vis(self, other, "rain_gloom_lead_frames")
    if w.timer <= lead then
      target = 1.0
    else
      target = 0
    end
  elseif phase == 1.0 or phase == 2.0 then
    target = 1.0
  end
  if target > 0 and w.gloom <= 0 then
    sfx_play(self, other, thunder)
  end
  local rate = 1.0 / gm.max(1.0, vis(self, other, "rain_gloom_fade_frames"))
  local step = gm.clamp(target - w.gloom, -rate, rate)
  w.gloom = gm.clamp(w.gloom + step, 0, 1.0)
end

-- gml_Script_rain_drop_reset @ 0x1402b5130
function rain_drop_reset(self, other, d, vx, vy, vw, anywhere)
  local surf = wld(self, other, "surface_height_px")
  d.x = (vx - 140) + gm.random(vw + 280)
  d.ground = (surf - 4) + gm.random(7.0)
  d.speed_scale = gm.random_range(0.85, 1.15)
  d.len_scale = gm.random_range(0.75, 1.35)
  d.alpha_scale = gm.random_range(0.6, 1.3)
  if gm.bool(anywhere) then
    d.y = (vy - 60) + gm.random(gm.max(80.0, (d.ground - vy) + 60))
  else
    d.y = (vy - 40) - gm.random(220.0)
  end
end

-- gml_Script____struct___289__rain_splash_add__scr_weather @ 0x1402b5dd0
function ___struct___289__rain_splash_add__scr_weather(self, other, a0, a1, a2, a3)
  self.x = argv(a0)
  self.y = argv(a1)
  self.t = 0
  self.life = argv(a2)
  self.size = argv(a3)
end

-- gml_Script____struct___290__rain_splash_add__scr_weather @ 0x1402b60d0
function ___struct___290__rain_splash_add__scr_weather(self, other, a0, a1, a2, a3, a4, a5)
  self.x = argv(a0)
  self.y = argv(a1)
  self.vx = argv(a2)
  self.vy = argv(a3)
  self.ground = argv(a4)
  self.t = 0
  self.life = argv(a5)
end

-- gml_Script_rain_splash_add @ 0x1402b64a0
function rain_splash_add(self, other, w, x, y, scale)
  if gm.array_length(w.splashes) >= vis(self, other, "rain_max_splashes") then
    return
  end
  local splashes = w.splashes
  local life = gm.irandom_range(15, 26)
  local size = gm.random_range(4.0, 9.0) * scale
  gm.array_push(splashes, gm.new(___struct___289__rain_splash_add__scr_weather, x, y, life, size))
  if gm.array_length(w.spray) >= vis(self, other, "rain_max_spray") then
    return
  end
  local n = gm.irandom_range(1, 2)
  local i = 0
  while i < n do
    local spray = w.spray
    local wind = w.wind
    local vx = gm.random_range(-1.3, 1.3) + wind * 0.12
    local vy = -gm.random_range(1.1, 2.7) * scale
    local ground = y + gm.random(3.0)
    local plife = gm.irandom_range(10, 20)
    gm.array_push(spray, gm.new(___struct___290__rain_splash_add__scr_weather, x, y, vx, vy, ground, plife))
    i = i + 1
  end
end

-- gml_Script_rain_splashes_step @ 0x1402b73e0
function rain_splashes_step(self, other, w)
  local list = w.splashes
  local keep = gm.array()
  local i = 0
  while i < gm.array_length(list) do
    local s = list[i]
    s.t = s.t + 1
    if s.t < s.life then
      gm.array_push(keep, s)
    end
    i = i + 1
  end
  w.splashes = keep
  local spray = w.spray
  keep = gm.array()
  i = 0
  while i < gm.array_length(spray) do
    local p = spray[i]
    p.t = p.t + 1
    p.vy = p.vy + 0.17
    p.x = p.x + p.vx
    p.y = p.y + p.vy
    if p.t < p.life and p.y < p.ground then
      gm.array_push(keep, p)
    end
    i = i + 1
  end
  w.spray = keep
end

-- gml_Script_weather_step @ 0x1402b82d0
function weather_step(self, other, vx, vy, vw)
  local w = global.weather
  rain_phase_step(self, other, w)
  rain_splashes_step(self, other, w)
  if w.intensity <= 0 then
    w.active = 0
    return
  end
  w.gust_timer = w.gust_timer - 1.0
  if w.gust_timer <= 0 then
    w.gust_timer = gm.irandom_range(90, 260)
    w.gust_target = gm.random_range(0.55, 1.7)
  end
  w.gust = w.gust + (w.gust_target - w.gust) * 0.018
  local wind = global.wind_dir * global.wind_speed * w.gust
  w.wind = wind * vis(self, other, "rain_wind_push")
  if not (vy < wld(self, other, "surface_height_px")) then
    w.active = 0
    return
  end
  local drops = w.drops
  local n = gm.round(gm.array_length(drops) * w.intensity)
  local bands = w.bands
  local left = vx - 160
  local right = vx + vw + 160
  local chance = vis(self, other, "rain_splash_chance")
  local i = 0
  while i < n do
    local d = drops[i]
    if not (i < w.active) then
      rain_drop_reset(self, other, d, vx, vy, vw, true)
    else
      local b = bands[d.band]
      d.x = d.x + w.wind * b.drift
      d.y = d.y + b.speed * d.speed_scale
      if d.y < d.ground then
        if d.x < left or d.x > right or d.y < vy - 520 then
          rain_drop_reset(self, other, d, vx, vy, vw, false)
        end
      else
        if d.x > vx - 32 and d.x < vx + vw + 32 and gm.random(1.0) < chance then
          rain_splash_add(self, other, w, d.x, d.ground, b.splash)
        end
        rain_drop_reset(self, other, d, vx, vy, vw, false)
      end
    end
    i = i + 1
  end
  w.active = n
end

-- gml_Script_rain_streak @ 0x1402ba7e0
function rain_streak(self, other, x, y, dx, dy, width, col, alpha)
  local len = gm.point_distance(0, 0, dx, dy)
  if len <= 0 then
    return
  end
  local hw = width * 0.5
  local nx = -dy / len * hw
  local ny = dx / len * hw
  local tx = x - dx
  local ty = y - dy
  local hx1 = x + nx
  local hy1 = y + ny
  local hx2 = x - nx
  local hy2 = y - ny
  local tx1 = tx + nx * 0.3
  local ty1 = ty + ny * 0.3
  local tx2 = tx - nx * 0.3
  local ty2 = ty - ny * 0.3
  gm.draw_vertex_colour(hx1, hy1, col, alpha)
  gm.draw_vertex_colour(hx2, hy2, col, alpha)
  gm.draw_vertex_colour(tx1, ty1, col, 0)
  gm.draw_vertex_colour(tx1, ty1, col, 0)
  gm.draw_vertex_colour(hx2, hy2, col, alpha)
  gm.draw_vertex_colour(tx2, ty2, col, 0)
end

-- gml_Script_rain_bands_draw @ 0x1402bc210
function rain_bands_draw(self, other, w, band_min, band_max, vx, vy, vw, vh)
  local drops = w.drops
  local n = gm.round(gm.array_length(drops) * w.intensity)
  if n <= 0 then
    return
  end
  local bands = w.bands
  local col = w.colour
  local inten = w.intensity
  local left = vx - 48
  local right = vx + vw + 48
  local top = vy - 64
  local bottom = gm.min(vy + vh, wld(self, other, "surface_height_px")) + 8
  gm.gpu_set_blendmode(gm.bm_add)
  gm.draw_primitive_begin(gm.pr_trianglelist)
  local i = 0
  while i < n do
    local d = drops[i]
    if not (d.band < band_min) and not (d.band > band_max) then
      if not (d.x < left) and not (d.x > right) and not (d.y < top) and not (d.y > bottom) then
        local b = bands[d.band]
        local dx = w.wind * b.drift
        local dy = b.speed * d.speed_scale
        local dl = gm.max(1.0, gm.point_distance(0, 0, dx, dy))
        local len = b.len * d.len_scale
        rain_streak(self, other, d.x, d.y, dx / dl * len, dy / dl * len, b.width, col,
          b.alpha * d.alpha_scale * inten)
      end
    end
    i = i + 1
  end
  gm.draw_primitive_end()
  gm.gpu_set_blendmode(gm.bm_normal)
end

-- gml_Script_rain_gloom_shade @ 0x1402be270
function rain_gloom_shade(self, other, w)
  local g = w.gloom
  local k = g * (g * 6 - 15) + 10
  return g * g * g * k
end

-- gml_Script_rain_gloom_draw @ 0x1402be6e0
function rain_gloom_draw(self, other, w, x, y, ww, hh)
  local strength = vis(self, other, "rain_gloom_strength")
  local s = rain_gloom_shade(self, other, w) * strength
  if s <= 0 then
    return
  end
  local gc = w.gloom_colour
  local r = gm.colour_get_red(gc)
  local g = gm.colour_get_green(gc)
  local b = gm.colour_get_blue(gc)
  local col = gm.make_colour_rgb(gm.lerp(255.0, r, s), gm.lerp(255.0, g, s), gm.lerp(255.0, b, s))
  gm.gpu_set_blendmode_ext(gm.bm_dest_colour, gm.bm_zero)
  gm.draw_set_colour(col)
  gm.draw_rectangle(x, y, x + ww, y + hh, false)
  gm.gpu_set_blendmode(gm.bm_normal)
  gm.draw_set_colour(gm.c_white)
end

-- gml_Script_rain_splashes_draw @ 0x1402bf1e0
function rain_splashes_draw(self, other, w, vx, vw)
  local list = w.splashes
  local col = w.colour
  local left = vx - 32
  local right = vx + vw + 32
  gm.gpu_set_blendmode(gm.bm_add)
  gm.draw_set_colour(col)
  local i = 0
  while i < gm.array_length(list) do
    local s = list[i]
    if not (s.x < left) and not (s.x > right) then
      local k = s.t / s.life
      local r = s.size * (0.25 + k * 1.6)
      gm.draw_set_alpha((1.0 - k) * 0.45)
      gm.draw_ellipse(s.x - r, s.y - r * 0.3, s.x + r, s.y + r * 0.3, true)
    end
    i = i + 1
  end
  gm.draw_set_alpha(1.0)
  gm.draw_set_colour(gm.c_white)
  local spray = w.spray
  gm.draw_primitive_begin(gm.pr_trianglelist)
  i = 0
  while i < gm.array_length(spray) do
    local p = spray[i]
    if not (p.x < left) and not (p.x > right) then
      local f = 1.0 - p.t / p.life
      rain_streak(self, other, p.x, p.y, p.vx * 1.6, p.vy * 1.6, 1.6, col, 0.5 * f)
    end
    i = i + 1
  end
  gm.draw_primitive_end()
  gm.gpu_set_blendmode(gm.bm_normal)
end

-- gml_Script_weather_draw_back @ 0x1402c1020
function weather_draw_back(self, other, vx, vy, vw, vh)
  local w = global.weather
  if vy >= wld(self, other, "surface_height_px") then
    return
  end
  if w.intensity <= 0 then
    return
  end
  rain_bands_draw(self, other, w, 0, 0, vx, vy, vw, vh)
end

-- gml_Script_weather_draw_front @ 0x1402c1510
function weather_draw_front(self, other, vx, vy, vw, vh)
  local w = global.weather
  rain_gloom_draw(self, other, w, vx, vy, vw, vh)
  if vy >= wld(self, other, "surface_height_px") then
    return
  end
  if w.intensity > 0 then
    rain_bands_draw(self, other, w, 1.0, 2.0, vx, vy, vw, vh)
  end
  if gm.array_length(w.splashes) > 0 or gm.array_length(w.spray) > 0 then
    rain_splashes_draw(self, other, w, vx, vw)
  end
end

-- gml_GlobalScript_scr_weather @ 0x1402c1d70
function gml_GlobalScript_scr_weather(self, other)
  self.weather_init = gm.method(self, weather_init)
  self.weather_is_raining = gm.method(self, weather_is_raining)
  self.weather_force_rain = gm.method(self, weather_force_rain)
  self.rain_begin = gm.method(self, rain_begin)
  self.rain_phase_step = gm.method(self, rain_phase_step)
  self.rain_gloom_step = gm.method(self, rain_gloom_step)
  self.rain_drop_reset = gm.method(self, rain_drop_reset)
  self.rain_splash_add = gm.method(self, rain_splash_add)
  self.rain_splashes_step = gm.method(self, rain_splashes_step)
  self.weather_step = gm.method(self, weather_step)
  self.rain_streak = gm.method(self, rain_streak)
  self.rain_bands_draw = gm.method(self, rain_bands_draw)
  self.rain_gloom_shade = gm.method(self, rain_gloom_shade)
  self.rain_gloom_draw = gm.method(self, rain_gloom_draw)
  self.rain_splashes_draw = gm.method(self, rain_splashes_draw)
  self.weather_draw_back = gm.method(self, weather_draw_back)
  self.weather_draw_front = gm.method(self, weather_draw_front)
end

-- gml_Script_anon__1944__gml_Object_obj_forest_controller_Create_0 @ 0x1405d4820
function anon__1944__gml_Object_obj_forest_controller_Create_0(self, other)
  local total = 0
  local i = 0
  while i < gm.array_length(self.bush_parts) do
    total = total + self.bush_parts[i].weight
    i = i + 1
  end
  local r = gm.random(gm.real(total))
  i = 0
  while i < gm.array_length(self.bush_parts) do
    r = r - self.bush_parts[i].weight
    if r < 0 then
      return i
    end
    i = i + 1
  end
  return 0
end

-- gml_Script____struct___374__gml_Object_obj_forest_controller_Create_0 @ 0x1405d4d00
function ___struct___374__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 121.0
  self.top = 6.0
  self.w = 11.0
  self.h = 28.0
  self.weight = 3.0
end

-- gml_Script____struct___375__gml_Object_obj_forest_controller_Create_0 @ 0x1405d4f10
function ___struct___375__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 121.0
  self.top = 43.0
  self.w = 11.0
  self.h = 28.0
  self.weight = 3.0
end

-- gml_Script____struct___376__gml_Object_obj_forest_controller_Create_0 @ 0x1405d5120
function ___struct___376__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 121.0
  self.top = 80.0
  self.w = 11.0
  self.h = 28.0
  self.weight = 3.0
end

-- gml_Script____struct___377__gml_Object_obj_forest_controller_Create_0 @ 0x1405d5330
function ___struct___377__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 142.0
  self.top = 2.0
  self.w = 15.0
  self.h = 33.0
  self.weight = 3.0
end

-- gml_Script____struct___378__gml_Object_obj_forest_controller_Create_0 @ 0x1405d5540
function ___struct___378__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 142.0
  self.top = 39.0
  self.w = 15.0
  self.h = 33.0
  self.weight = 3.0
end

-- gml_Script____struct___379__gml_Object_obj_forest_controller_Create_0 @ 0x1405d5750
function ___struct___379__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 142.0
  self.top = 76.0
  self.w = 15.0
  self.h = 33.0
  self.weight = 3.0
end
