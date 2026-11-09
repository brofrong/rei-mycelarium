-- Stone world for seed 7: `MYCO_SEED=7 tools/golden.sh stones_s7` (stones_world_common.lua).
assert(os.getenv("MYCO_SEED") == "7", "run with MYCO_SEED=7")
return dofile("rust/golden/scripts/stones_world_common.lua")
