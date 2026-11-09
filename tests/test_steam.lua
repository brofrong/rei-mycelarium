local steam = require("gm.steam")

return {
  steam_is_offline = function()
    assert(steam.steam_api_is_initialized() == false)
    assert(steam.steam_step() == nil)
  end,
  achievements_are_noops = function()
    assert(steam.steam_userstats_get_achievement(nil, nil, "ACH_X") == false)
    assert(steam.steam_userstats_set_achievement(nil, nil, "ACH_X") == false)
    assert(steam.steam_userstats_store_stats() == false)
  end,
}
