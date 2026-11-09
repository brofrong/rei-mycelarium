#!/usr/bin/env python3
"""LÖVE-vs-Rust conformance: run one scenario in both ports and compare the saved state.

Usage: python3 -I tools/conformance.py <scenario> [--keep DIR] [--rust-bin PATH]
       python3 -I tools/conformance.py --all

A scenario is rust/conformance/scenarios/<name>.txt, `key: value` lines (`#` comments):
`seed`, `dump` (frame), `input` (MYCO_INPUT entries, `;`-separated, several lines append),
`input.love` / `input.rust` (entries for one port only), `fixture` (a save JSON placed in slot 1),
`cross_from: rust, love` + `cross_input` + `cross_dump` (cross_save: each producer's dump becomes
the slot 1 save both ports load), `timeout` (seconds per run). Both ports run
with the same MYCO_SEED, the same MYCO_INPUT and MYCO_DUMP at the scenario's frame, in a
1280x720 window (the same logical view in both), with MYCO_FOCUS=1, MYCO_SHOT_EXACT=1 and
MYCO_RNG_SEED (game_init's clock-seeded randomise() reseeds the same way in both):

- LÖVE runs with MYCO_CONFORM=1 (game/gm/conform.lua): one logic step per frame on a step
  clock, with the music/ambience shuffle off the game's RNG; its save folder is the
  MYCO_IDENTITY=MycelariumTest-t22 folder (MYCO_CONFORM_IDENTITY=MycelariumTest-<x> picks another,
  e.g. per worktree), wiped before each run.
- Rust runs rust/target/release/myco_app with MYCO_SAVE_DIR set to a fresh temp folder.

The dumps (what save_game would write at that frame) are compared on the deterministic fields:
state.food, state.water_free, state.length_px, state.upgrade_levels, state.tasks,
mycelium.growths / mycelium.knots (positions within 0.5 px), mushrooms and world.chunks.
Every difference is printed with its field path; the exit code is 1 if there is any. The game
RNG is traced after every step in both ports (MYCO_TRACE_RNG) and the report says whether the
sequences stayed identical, or the first step where they part (tools/rng_compare.py then shows
the per-step draw counts).
"""
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCENARIOS = ROOT / "rust" / "conformance" / "scenarios"
LOVE = Path(os.environ.get("LOVE", ROOT / "build" / "tools" / "love.app" / "Contents" / "MacOS" / "love"))
RUST_BIN = ROOT / "rust" / "target" / "release" / "myco_app"
IDENTITY = os.environ.get("MYCO_CONFORM_IDENTITY", "MycelariumTest-t22")
assert IDENTITY.startswith("MycelariumTest"), "never the real LOVE/Mycelarium folder"
LOVE_SAVE_DIR = Path.home() / "Library" / "Application Support" / "LOVE" / IDENTITY
SETTINGS_INI = '[display]\nfullscreen="0.000000"\nwindow_w="1280.000000"\nwindow_h="720.000000"\n'

# compared fields: (path, position tolerance, relative tolerance). The absolute position
# tolerance applies only to position keys (POS_KEYS: x, y, x1..y2, bx1..by2); every other number
# under the path (timers, amounts, born, ...) is compared with the relative tolerance alone.
FIELDS = [
    ("state.food", 0.0, 1e-9),
    ("state.water_free", 0.0, 1e-9),
    ("state.length_px", 0.0, 1e-9),
    ("state.upgrade_levels", 0.0, 0.0),
    ("state.tasks", 0.0, 1e-9),
    ("mycelium.growths", 0.5, 1e-9),
    ("mycelium.knots", 0.5, 1e-9),
    ("mushrooms", 0.5, 1e-9),
    ("world.chunks", 0.0, 1e-9),
]


# ---------------------------------------------------------------------------------------------
# diff


def get_path(doc, path: str):
    cur = doc
    for part in path.split("."):
        if isinstance(cur, dict) and part in cur:
            cur = cur[part]
        else:
            return _MISSING
    return cur


class _Missing:
    def __repr__(self):
        return "<missing>"


_MISSING = _Missing()


POS_KEYS = {"x", "y", "x1", "y1", "x2", "y2", "bx1", "by1", "bx2", "by2"}


def _last_key(path: str) -> str:
    return path.rsplit(".", 1)[-1].split("[", 1)[0]


def _num(v):
    return isinstance(v, (int, float)) and not isinstance(v, bool)


def diff(a, b, path: str = "", atol: float = 0.0, rtol: float = 0.0, out: list | None = None,
         limit: int = 200) -> list:
    """Differences between two JSON values as (field path, a, b). Numbers match within
    `rtol * max(1, |a|, |b|)`, plus `atol` when the number is a position (its key is in
    POS_KEYS); booleans only match booleans. Lists compare element by element (`path[i]`, plus
    `path.length`), objects key by key (`path.key`)."""
    if out is None:
        out = []
    if len(out) >= limit:
        return out
    if _num(a) and _num(b):
        scale = max(1.0, abs(a), abs(b))
        tol = (atol if _last_key(path) in POS_KEYS else 0.0) + rtol * scale
        if not (a == b or abs(a - b) <= tol):
            out.append((path, a, b))
        return out
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b), key=str):
            p = f"{path}.{k}" if path else str(k)
            if k not in a:
                out.append((p, _MISSING, b[k]))
            elif k not in b:
                out.append((p, a[k], _MISSING))
            else:
                diff(a[k], b[k], p, atol, rtol, out, limit)
        return out
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            out.append((f"{path}.length", len(a), len(b)))
        for i, (x, y) in enumerate(zip(a, b)):
            diff(x, y, f"{path}[{i}]", atol, rtol, out, limit)
        return out
    if type(a) is not type(b) or a != b:
        out.append((path, a, b))
    return out


def compare_dumps(love: dict, rust: dict, fields=FIELDS, limit: int = 200) -> list:
    out: list = []
    for path, atol, rtol in fields:
        a, b = get_path(love, path), get_path(rust, path)
        if a is _MISSING or b is _MISSING:
            # a compared field absent from a dump is a failure, even when absent from both
            out.append((path, a, b))
            continue
        diff(a, b, path, atol, rtol, out, limit)
    return out


def short(v, n: int = 90) -> str:
    s = json.dumps(v) if not isinstance(v, _Missing) else repr(v)
    return s if len(s) <= n else s[: n - 3] + "..."


# ---------------------------------------------------------------------------------------------
# scenarios


@dataclass
class Scenario:
    name: str
    seed: int = 31607
    dump: int = 0
    inputs: list = field(default_factory=list)
    # per-port extra input (e.g. the node: Rust's right click is the owner's quick node, LÖVE
    # places the same node with Alt+LMB, both through myc_node_quick)
    port_inputs: dict = field(default_factory=lambda: {"love": [], "rust": []})
    # save fixture (JSON file, relative to the scenario folder) placed in slot 1 before the run
    fixture: str | None = None
    # cross_save: producer ports ("rust,love"); each one's dump at cross_dump (played with
    # cross_input) becomes the slot 1 save that both ports then load through the menu
    cross_from: str | None = None
    cross_inputs: list = field(default_factory=list)
    cross_dump: int = 0
    timeout: int = 900

    def input_for(self, port: str) -> str:
        entries = self.inputs + self.port_inputs[port]
        return ";".join(sorted(entries, key=lambda e: int(e.split(":", 1)[0])))


def parse_scenario(text: str, name: str = "") -> Scenario:
    sc = Scenario(name)
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        key, _, val = line.partition(":")
        key, val = key.strip(), val.strip()
        if key == "seed":
            sc.seed = int(val)
        elif key == "dump":
            sc.dump = int(val)
        elif key == "input":
            sc.inputs += [v.strip() for v in val.split(";") if v.strip()]
        elif key in ("input.love", "input.rust"):
            sc.port_inputs[key.split(".")[1]] += [v.strip() for v in val.split(";") if v.strip()]
        elif key == "fixture":
            sc.fixture = val
        elif key == "cross_from":
            sc.cross_from = val
        elif key == "cross_input":
            sc.cross_inputs += [v.strip() for v in val.split(";") if v.strip()]
        elif key == "cross_dump":
            sc.cross_dump = int(val)
        elif key == "timeout":
            sc.timeout = int(val)
        else:
            raise ValueError(f"{name}: unknown key {key!r}")
    return sc


def load_scenario(name: str) -> Scenario:
    return parse_scenario((SCENARIOS / f"{name}.txt").read_text(), name)


# ---------------------------------------------------------------------------------------------
# running the ports


def meta_for(save: dict) -> dict:
    st = save.get("state", {})
    return {"length_cm": 0, "mode": st.get("mode", "exploration"), "name": save.get("name", ""), "saved": 46000.0}


def prepare_save_dir(d: Path, save_text: str | None):
    if d.exists():
        shutil.rmtree(d)
    d.mkdir(parents=True)
    (d / "settings.ini").write_text(SETTINGS_INI)
    if save_text is not None:
        save = json.loads(save_text.rstrip("\0"))
        (d / "save_1.json").write_bytes(save_text.rstrip("\0").encode() + b"\0")
        (d / "save_1_meta.json").write_bytes(json.dumps(meta_for(save)).encode() + b"\0")


TRACE_RNG = True


def run_port(port: str, seed: int, script: str, dump_frame: int, out: Path, save_text: str | None,
             timeout: int, rust_bin: Path, log: Path) -> dict:
    env = dict(os.environ)
    for k in list(env):
        if k.startswith("MYCO_"):
            del env[k]
    env.update({
        "MYCO_SEED": str(seed),
        # the clock-seeded game RNG (game_init's randomise) gets the same seed in both ports
        "MYCO_RNG_SEED": str(seed),
        "MYCO_INPUT": script,
        "MYCO_DUMP": f"{dump_frame}:{out}",
        "MYCO_FOCUS": "1",
        "MYCO_SHOT_EXACT": "1",
    })
    if TRACE_RNG:
        env["MYCO_TRACE_RNG"] = "1"
    if out.exists():
        out.unlink()
    if port == "love":
        assert LOVE_SAVE_DIR.name == IDENTITY  # never the real LOVE/Mycelarium folder
        prepare_save_dir(LOVE_SAVE_DIR, save_text)
        env["MYCO_IDENTITY"] = IDENTITY
        env["MYCO_CONFORM"] = "1"
        cmd = [str(LOVE), "game"]
    else:
        save_dir = Path(tempfile.mkdtemp(prefix="myco-conf-"))
        prepare_save_dir(save_dir, save_text)
        env["MYCO_SAVE_DIR"] = str(save_dir)
        env["MYCO_ASSETS"] = str(ROOT / "build" / "assets")
        cmd = [str(rust_bin)]
    with open(log, "wb") as lf:
        try:
            subprocess.run(cmd, cwd=ROOT, env=env, stdout=lf, stderr=subprocess.STDOUT, timeout=timeout)
        except subprocess.TimeoutExpired:
            raise RuntimeError(f"{port}: timed out after {timeout}s (log {log})")
    if not out.exists():
        raise RuntimeError(f"{port}: no dump written (log {log})")
    return json.loads(out.read_text().rstrip("\0"))


def run_both(sc: Scenario, work: Path, rust_bin: Path, save_text: str | None, tag: str, verbose: bool) -> dict:
    dumps = {}
    for port in ("love", "rust"):
        if verbose:
            print(f"[{sc.name}{tag}] {port}: running to frame {sc.dump}")
        dumps[port] = run_port(port, sc.seed, sc.input_for(port), sc.dump, work / f"{port}{tag}.json", save_text,
                               sc.timeout, rust_bin, work / f"{port}{tag}.log")
    if TRACE_RNG:
        first_rng_divergence(work / f"love{tag}.log", work / f"rust{tag}.log")
    return dumps


def run_scenario(sc: Scenario, work: Path, rust_bin: Path, verbose: bool = True) -> list:
    work.mkdir(parents=True, exist_ok=True)
    if not sc.cross_from:
        save_text = (SCENARIOS / sc.fixture).read_text() if sc.fixture else None
        dumps = run_both(sc, work, rust_bin, save_text, "", verbose)
        return compare_dumps(dumps["love"], dumps["rust"])
    # cross_save: each producer port plays and dumps (the text save_game writes); both ports
    # then load that file through the title's Load Game and dump right after
    diffs = []
    for prod in [p.strip() for p in sc.cross_from.split(",")]:
        other = "love" if prod == "rust" else "rust"
        if verbose:
            print(f"[{sc.name}] {prod}: producing the save (dump at {sc.cross_dump})")
        made_path = work / f"{prod}_made.json"
        run_port(prod, sc.seed, ";".join(sc.cross_inputs), sc.cross_dump, made_path, None, sc.timeout, rust_bin,
                 work / f"{prod}_made.log")
        save_text = made_path.read_text()
        made = json.loads(save_text.rstrip("\0"))
        dumps = run_both(sc, work, rust_bin, save_text, f"_from_{prod}", verbose)
        for p, a, b in compare_dumps(dumps["love"], dumps["rust"]):
            diffs.append((f"[{prod} save, love vs rust after load] {p}", a, b))
        # the loading port keeps every compared field of the other port's save, except the
        # lakes' flare_seq: load_game drops the sparkles (lk.flares) and the first draw after it
        # restarts them (lake_init_flares sets flare_seq = 0 and counts again), in both ports
        for p, a, b in compare_dumps(made, dumps[other]):
            if not p.endswith(".flare_seq"):
                diffs.append((f"[{prod} save vs {other} after load] {p}", a, b))
    return diffs


def rng_trace(log: Path) -> dict:
    out = {}
    for line in log.read_text(errors="replace").splitlines():
        parts = line.split()
        if len(parts) == 3 and parts[0] == "RNG":
            out[int(parts[1])] = parts[2]
    return out


def first_rng_divergence(love_log: Path, rust_log: Path):
    a, b = rng_trace(love_log), rng_trace(rust_log)
    for f in sorted(set(a) & set(b)):
        if a[f] != b[f]:
            print(f"  RNG first differs after step {f}: love {a[f]} rust {b[f]}")
            return f
    print(f"  RNG identical over {len(set(a) & set(b))} steps")
    return None


def report(name: str, diffs: list) -> int:
    if not diffs:
        print(f"[{name}] PASS: no differences in the compared fields")
        return 0
    print(f"[{name}] FAIL: {len(diffs)} difference(s) (love vs rust)")
    for p, a, b in diffs:
        print(f"  {p}: {short(a)} != {short(b)}")
    return 1


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("scenario", nargs="?")
    ap.add_argument("--all", action="store_true")
    ap.add_argument("--keep", type=Path, help="folder for dumps and logs (default: a temp folder)")
    ap.add_argument("--rust-bin", type=Path, default=RUST_BIN)
    ap.add_argument("--no-trace-rng", action="store_true",
                    help="do not trace the game RNG (by default the report says where it first differs)")
    args = ap.parse_args(argv)
    global TRACE_RNG
    TRACE_RNG = not args.no_trace_rng
    names = sorted(p.stem for p in SCENARIOS.glob("*.txt")) if args.all else [args.scenario]
    if not names or names == [None]:
        ap.error("give a scenario or --all")
    base = args.keep or Path(tempfile.mkdtemp(prefix="myco-conformance-"))
    rc = 0
    for n in names:
        sc = load_scenario(n)
        try:
            diffs = run_scenario(sc, base / n, args.rust_bin)
        except RuntimeError as e:
            print(f"[{n}] ERROR: {e}")
            rc = 1
            continue
        rc |= report(n, diffs)
    print(f"dumps and logs: {base}")
    return rc


if __name__ == "__main__":
    sys.exit(main())
