local r = require("gm.random")
local conform = require("gm.conform")

return {
  random_state_roundtrip = function()
    r.random_set_seed(99)
    r.random(1)
    local st = r.get_state()
    local a = { r.random(1), r.random(1) }
    r.random_set_seed(5)
    r.set_state(st)
    local b = { r.random(1), r.random(1) }
    assert(a[1] == b[1] and a[2] == b[2] and r.random_get_seed() == 99)
  end,
  isolated_function_leaves_game_rng_alone = function()
    local function audio() return r.random(1) + r.irandom(10) end
    local wrapped = conform.isolate_rng(audio, r)
    r.random_set_seed(42)
    local expect = { r.random(1), r.random(1) }
    r.random_set_seed(42)
    wrapped()
    local x = r.random(1)
    wrapped()
    wrapped()
    local y = r.random(1)
    assert(x == expect[1] and y == expect[2], "game sequence unchanged by audio draws")
  end,
  install_wraps_only_audio_scripts = function()
    local env = { audio_step = function() return r.random(1) end, myc_step = function() return r.random(1) end }
    local orig_myc = env.myc_step
    conform.install_audio(env, r)
    assert(env.myc_step == orig_myc)
    r.random_set_seed(1)
    local first = r.random(1)
    r.random_set_seed(1)
    env.audio_step()
    assert(r.random(1) == first)
  end,
  lockstep_clock_matches_rust_now_ms = function()
    assert(conform.time_ms(0) == 0 and conform.time_ms(1) == 16 and conform.time_ms(3) == 50)
    assert(conform.time_ms(59) == 983 and conform.time_ms(60) == 1000)
    assert(conform.timer_us(60) == 1000000)
  end,
}
