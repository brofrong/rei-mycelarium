-- Cave layout for seed 31607: run as `MYCO_SEED=31607 tools/golden.sh caves_s31607` (see caves_gen_common.lua).
assert(os.getenv("MYCO_SEED") == "31607", "run with MYCO_SEED=31607")
return dofile("rust/golden/scripts/caves_gen_common.lua")
