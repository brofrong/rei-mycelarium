local gm = require("gm")

-- Functions missed by the original partition (grouped as Steam by mistake); translated by the integrator.

-- gml_Script____struct___367__gml_Object_obj_forest_controller_Create_0 @ 0x1405d39c0
function ___struct___367__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 6.0
  self.top = 39.0
  self.w = 187.0
  self.h = 35.0
  self.weight = 3.0
end

-- gml_Script____struct___368__gml_Object_obj_forest_controller_Create_0 @ 0x1405d3bd0
function ___struct___368__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 208.0
  self.top = 48.0
  self.w = 220.0
  self.h = 26.0
  self.weight = 3.0
end

-- gml_Script____struct___369__gml_Object_obj_forest_controller_Create_0 @ 0x1405d3de0
function ___struct___369__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 441.0
  self.top = 25.0
  self.w = 347.0
  self.h = 49.0
  self.weight = 2.0
end

-- gml_Script____struct___370__gml_Object_obj_forest_controller_Create_0 @ 0x1405d3ff0
function ___struct___370__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 0
  self.top = 99.0
  self.w = 713.0
  self.h = 49.0
  self.weight = 1.0
end

-- gml_Script____struct___371__gml_Object_obj_forest_controller_Create_0 @ 0x1405d41f0
function ___struct___371__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 16.0
  self.top = 155.0
  self.w = 123.0
  self.h = 67.0
  self.weight = 3.0
end

-- gml_Script____struct___372__gml_Object_obj_forest_controller_Create_0 @ 0x1405d4400
function ___struct___372__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 166.0
  self.top = 148.0
  self.w = 132.0
  self.h = 74.0
  self.weight = 3.0
end

-- gml_Script____struct___373__gml_Object_obj_forest_controller_Create_0 @ 0x1405d4610
function ___struct___373__gml_Object_obj_forest_controller_Create_0(self, other)
  self.left = 343.0
  self.top = 160.0
  self.w = 92.0
  self.h = 62.0
  self.weight = 3.0
end
-- gml_Script_anon__799__gml_Object_obj_forest_controller_Create_0 @ 0x1405d3280
-- forest_fit_view: re-centre the camera on the new view size and match view port 0 to the surface.
function anon__799__gml_Object_obj_forest_controller_Create_0(self, other)
  -- L28-L29 (FUN_140024840 = +=)
  self.camera_x = self.camera_x + (self.camera_width - global.view_w) * 0.5
  self.camera_y = self.camera_y + (self.camera_height - global.view_h) * 0.5
  -- L30-L31
  self.camera_width = global.view_w
  self.camera_height = global.view_h
  -- L33-L35
  gm.camera_set_view_size(self.forest_camera, self.camera_width, self.camera_height)
  gm.view_set_wport(0, global.surf_w)
  gm.view_set_hport(0, global.surf_h)
end
