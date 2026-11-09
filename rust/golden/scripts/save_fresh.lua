-- Fresh Exploration save (what save_game writes right after "new game"), for
-- rust/crates/myco_core/tests/save.rs. Run with MYCO_SEED=31607 (world seed). The animal timers
-- are jittered by the RNG, so the script also reports the jitter-free values the Rust model uses
-- as defaults for a save without state.timers (load_game itself has no default except boar).
local gm = require("gm")

save_start_new(global, global, 0, "Fresh")
game_mode_start(global, global, "exploration")
global.title_open, global.intro_active = false, false

local captured
local saved_write = save_write_text
save_write_text = function(self, other, path, text)
  if not captured then captured = text end
end
local ok, err = pcall(save_game, global, global)
save_write_text = saved_write
assert(ok and captured, "save_game failed: " .. tostring(err))

local fps = global.fps
local function v(k) return vis(global, global, k) end
local function b(k) return bal(global, global, k) end
return {
  save_json = captured,
  timer_defaults = {
    hedgehog = (b("hedgehog_spawn_interval") + (b("hedgehog_first_delay_min") + b("hedgehog_first_delay_max")) / 2) * fps,
    squirrel = b("squirrel_spawn_interval") * fps,
    boar = b("boar_spawn_interval") * fps,
    crow = b("crow_spawn_interval") * fps,
    experience_crows = { b("experience_crow_interval") * fps,
      (b("experience_crow_interval") + b("experience_crow_offset")) * fps },
    leaf = b("leaf_spawn_interval"),
    worm = 60,
  },
  -- jitter-free wind for a save without state.wind: direction 1, mid strength, full interval
  wind_default = { 1, (v("wind_min_strength") + v("wind_max_strength")) / 2, v("wind_change_interval") },
}
