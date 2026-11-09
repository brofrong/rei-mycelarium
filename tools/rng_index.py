#!/usr/bin/env python3
"""How many draws after random_set_seed(seed) the game RNG is at (conformance debugging).

Usage: python3 -I tools/rng_index.py <seed> <state> [<state> ...] [--max N]
`state` is the xorshift word the MYCO_TRACE_RNG lines print (game/gm/random.lua, rng.rs).
"""
import sys

M = (1 << 64) - 1


def seeded(seed: int) -> int:
    s = (seed * 6364136223846793005 + 1442695040888963407) & M
    return s or 88172645463325252


def step(x: int) -> int:
    x ^= x >> 12
    x ^= (x << 25) & M
    x ^= x >> 27
    return x


def index_of(seed: int, targets, limit: int = 10_000_000) -> dict:
    want = {int(t) for t in targets}
    found = {}
    x = seeded(seed)
    for n in range(limit + 1):
        if x in want:
            found[x] = n
            if len(found) == len(want):
                break
        x = step(x)
    return found


def main(argv):
    limit = 10_000_000
    if "--max" in argv:
        i = argv.index("--max")
        limit = int(argv[i + 1])
        argv = argv[:i] + argv[i + 2:]
    seed, targets = int(argv[0]), argv[1:]
    found = index_of(seed, targets, limit)
    for t in targets:
        print(t, found.get(int(t), "not within %d draws" % limit))


if __name__ == "__main__":
    main(sys.argv[1:])
