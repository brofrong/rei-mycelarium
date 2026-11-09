local gm = require("gm.core")

local function fresh()
  gm.reset()
  return {}
end

return {
  event_order = function()
    local log = fresh()
    local obj = gm.object("obj_t", {
      Step_1 = function() log[#log + 1] = "Step_1" end,
      Step_0 = function() log[#log + 1] = "Step_0" end,
      Draw_0 = function() log[#log + 1] = "Draw_0" end,
      Draw_64 = function() log[#log + 1] = "Draw_64" end,
    })
    gm.instance_create_depth(0, 0, 0, obj)
    gm.step()
    gm.draw()
    assert(table.concat(log, ",") == "Step_1,Step_0,Draw_0,Draw_64", table.concat(log, ","))
  end,
  draw_depth = function()
    local log = fresh()
    local obj = gm.object("obj_d", { Draw_0 = function(self) log[#log + 1] = self.depth end })
    gm.instance_create_depth(0, 0, -10, obj)
    gm.instance_create_depth(0, 0, 10, obj)
    gm.draw()
    assert(log[1] == 10 and log[2] == -10)
  end,
  create_runs_and_sets_fields = function()
    fresh()
    local obj = gm.object("obj_c", { Create_0 = function(self) self.hp = 3 end })
    local inst = gm.instance_create_depth(5, 6, 0, obj)
    assert(inst.x == 5 and inst.y == 6 and inst.hp == 3 and inst.object_index == obj and inst.id >= 100000)
  end,
  with_other = function()
    fresh()
    local a = gm.object("obj_a", {})
    local b = gm.object("obj_b", {})
    local ia = gm.instance_create_depth(0, 0, 0, a)
    gm.instance_create_depth(0, 0, 0, b)
    gm.instance_create_depth(0, 0, 0, b)
    local seen = 0
    gm.with(ia, b, function(self, other)
      assert(other == ia and self.object_index == b)
      seen = seen + 1
    end)
    assert(seen == 2)
  end,
  parent_events_and_inherited = function()
    local log = fresh()
    local p = gm.object("obj_p", { Create_0 = function() log[#log + 1] = "p" end })
    local c = gm.object("obj_ch", { parent = p, Create_0 = function(self, other) gm.event_inherited(self, other); log[#log + 1] = "c" end })
    local c2 = gm.object("obj_ch2", { parent = p })
    gm.instance_create_depth(0, 0, 0, c)
    gm.instance_create_depth(0, 0, 0, c2)
    assert(table.concat(log, ",") == "p,c,p")
    assert(gm.instance_number(p) == 2 and gm.instance_exists(c2))
  end,
  destroy_removes_after_cleanup = function()
    local log = fresh()
    local obj = gm.object("obj_x", { CleanUp_0 = function() log[#log + 1] = "cleanup" end })
    local inst = gm.instance_create_depth(0, 0, 0, obj)
    gm.instance_destroy(inst)
    assert(log[1] == "cleanup" and not gm.instance_exists(obj))
  end,
  room_goto_creates_instances = function()
    fresh()
    local obj = gm.object("obj_r", { Create_0 = function(self) self.made = true end })
    gm.define_room("R1", { w = 100, h = 50, instances = { { object = "obj_r", x = 1, y = 2, scale_x = 1, scale_y = 1, image_speed = 1, image_index = 0, color = 4294967295, angle = 0 } }, layers = {} })
    gm.room_goto("R1")
    gm.step() -- room change applies at end of step
    local inst = gm.instance_find(obj, 0)
    assert(gm.room == "R1" and gm.room_width == 100 and inst and inst.made and inst.x == 1)
  end,
  game_end_runs_events = function()
    local log = fresh()
    local obj = gm.object("obj_e", {
      Other_3 = function() log[#log + 1] = "end" end,
      CleanUp_0 = function() log[#log + 1] = "cleanup" end,
    })
    gm.instance_create_depth(0, 0, 0, obj)
    local quit = false
    gm.on_game_end = function() quit = true end
    gm.game_end()
    assert(table.concat(log, ",") == "end,cleanup" and quit)
  end,
}
