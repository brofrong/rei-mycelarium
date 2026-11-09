-- gpu_set_tex_filter / gpu_get_tex_filter state (no love dependency). GameMaker answers the
-- filter the last set left; the game saves and restores it around repeller_clouds_draw and
-- cave_perk_icons_draw (ruling R27). Starts linear, LÖVE's default filter.
local M = {}
local on = true

local function truthy(v) return v == true or (type(v) == "number" and v >= 0.5) end

function M.set(v) on = truthy(v) end
function M.get() return on end
function M.reset() on = true end

return M
