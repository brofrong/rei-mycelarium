-- Stone world for seed 31607: `MYCO_SEED=31607 tools/golden.sh stones_s31607` (stones_world_common.lua).
assert(os.getenv("MYCO_SEED") == "31607", "run with MYCO_SEED=31607")
return dofile("rust/golden/scripts/stones_world_common.lua")
