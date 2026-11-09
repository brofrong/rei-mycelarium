-- MYCO_CONFORM=1: lockstep mode for the LÖVE-vs-Rust conformance harness (tools/conformance.py).
-- - one logic step per love.update, so MYCO_INPUT/MYCO_DUMP frame N is logic step N, as in the
--   Rust port (the normal loop runs as many steps as the wall clock owes, up to 5);
-- - current_time follows the step count (floor(steps * 1000 / 60), like Rust's Sim::now_ms) and
--   get_timer stands still within a step (time budgets never run out, like the Rust port's);
-- - the music/ambience shuffle draws from a private generator instead of the game's RNG (the
--   Rust port's audio engine has its own), so the game's random sequence is the same in both.
local M = {}

M.enabled = os.getenv("MYCO_CONFORM") == "1"

-- current_time (ms) during logic step n (0-based count of the steps already run)
function M.time_ms(n) return math.floor(n * 1000 / 60) end

-- get_timer (us) during logic step n: constant within the step
function M.timer_us(n) return math.floor(n * 1e6 / 60) end

-- Scripts of scr_audio that roll the global RNG for music and ambience only.
M.AUDIO_RNG_FUNCS = { "audio_init", "audio_step", "music_next_track", "ambience_init", "ambience_rearm", "ambience_step" }

-- Wrap fn so its random draws come from a private generator state and leave the game's alone.
function M.isolate_rng(fn, random, private)
  private = private or { st = nil }
  return function(...)
    local game = random.get_state()
    if private.st then random.set_state(private.st) else random.random_set_seed(1) end
    local res = { pcall(fn, ...) }
    private.st = random.get_state()
    random.set_state(game)
    if not res[1] then error(res[2], 0) end
    return unpack(res, 2)
  end
end

-- Replace the audio scripts in env (the global table) by RNG-isolated wrappers.
function M.install_audio(env, random)
  local private = { st = nil }
  for _, name in ipairs(M.AUDIO_RNG_FUNCS) do
    local fn = rawget(env, name)
    if fn then env[name] = M.isolate_rng(fn, random, private) end
  end
end

return M
