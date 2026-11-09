#!/usr/bin/env python3
"""Per-step game-RNG draw counts of two MYCO_TRACE_RNG logs (conformance debugging).

Usage: python3 -I tools/rng_compare.py <seed> <love.log> <rust.log> [--steps N]
Prints the steps where the number of draws differs between the ports.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from rng_index import seeded, step  # noqa: E402


def trace(path):
    out = {}
    for line in Path(path).read_text(errors="replace").splitlines():
        p = line.split()
        if len(p) == 3 and p[0] == "RNG":
            out[int(p[1])] = int(p[2])
    return out


def indices(seed, states, limit=20_000_000):
    want = set(states.values())
    found = {}
    x = seeded(seed)
    for n in range(limit + 1):
        if x in want:
            found[x] = n
            if len(found) == len(want):
                break
        x = step(x)
    return {f: found.get(s) for f, s in states.items()}


def main(argv):
    seed, a, b = int(argv[0]), trace(argv[1]), trace(argv[2])
    ia, ib = indices(seed, a), indices(seed, b)
    prev = (None, None)
    shown = 0
    for f in sorted(set(ia) & set(ib)):
        da = None if ia[f] is None or prev[0] is None else ia[f] - prev[0]
        db = None if ib[f] is None or prev[1] is None else ib[f] - prev[1]
        if f == 0 or da != db:
            print(f"step {f}: love idx {ia[f]} (+{da}) rust idx {ib[f]} (+{db})")
            shown += 1
            if shown > 40:
                break
        prev = (ia[f], ib[f])


if __name__ == "__main__":
    main(sys.argv[1:])
