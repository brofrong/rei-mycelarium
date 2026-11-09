-- Cave layout for seed 7: run as `MYCO_SEED=7 tools/golden.sh caves_s7` (see caves_gen_common.lua).
assert(os.getenv("MYCO_SEED") == "7", "run with MYCO_SEED=7")
return dofile("rust/golden/scripts/caves_gen_common.lua")
