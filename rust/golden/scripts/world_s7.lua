-- World layout for seed 7: run as `MYCO_SEED=7 tools/golden.sh world_s7` (see world_gen_common.lua).
assert(os.getenv("MYCO_SEED") == "7", "run with MYCO_SEED=7")
return dofile("rust/golden/scripts/world_gen_common.lua")
