-- The love-free part of the GML compatibility layer: one table `gm` with
-- values (gm.value) and the instance/event/room runtime (gm.instance).
local gm = require("gm.instance")
for k, v in pairs(require("gm.value")) do
  if gm[k] == nil then gm[k] = v end
end
return gm
