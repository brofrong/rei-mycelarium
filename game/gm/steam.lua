-- Steamworks stubs: the port runs without Steam (no achievements, cloud or overlay).
-- Signatures follow the translated GML convention fn(self, other, ...).
local M = {}

function M.steam_api_is_initialized() return false end
function M.steam_step() end
function M.steam_userstats_get_achievement() return false end
function M.steam_userstats_set_achievement() return false end
function M.steam_userstats_store_stats() return false end
function M.steam_api_last_error() return "" end

return M
