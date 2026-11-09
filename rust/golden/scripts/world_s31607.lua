-- World layout for seed 31607: run as `MYCO_SEED=31607 tools/golden.sh world_s31607` (see world_gen_common.lua).
assert(os.getenv("MYCO_SEED") == "31607", "run with MYCO_SEED=31607")
return dofile("rust/golden/scripts/world_gen_common.lua")
