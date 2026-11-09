#!/usr/bin/env python3
"""Check that every original function owned by a task has a `// port: NAME` marker in rust/.

rust/portmap.tsv rows: `name<TAB>alias<TAB>task` (alias `-` when none, task `T<N>`, `T?` or
`skip:<reason>`). A row is covered when some .rs file contains `// port: <name>` or
`// port: <alias>`. `skip:*` rows need no marker.

Usage: python3 -I tools/port_coverage.py --task T5 | --all  [--portmap FILE] [--src DIR]
Exit code 1 when a required function has no marker, 2 for a task absent from the map.
Markers under target/ and tests/ directories do not count.
"""
import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MARKER_RE = re.compile(r"//\s*port:\s*([A-Za-z_][A-Za-z0-9_]*)")


def read_portmap(path):
    rows = []
    for n, line in enumerate(Path(path).read_text().splitlines(), 1):
        if not line.strip() or line.startswith("#"):
            continue
        cols = line.split("\t")
        if len(cols) != 3:
            raise SystemExit(f"{path}:{n}: expected 3 tab-separated columns, got {len(cols)}")
        rows.append(tuple(c.strip() for c in cols))
    return rows


def markers(src):
    found = set()
    for path in Path(src).rglob("*.rs"):
        parts = path.relative_to(src).parts
        if "target" in parts or "tests" in parts:  # tests do not port anything
            continue
        found.update(MARKER_RE.findall(path.read_text(errors="replace")))
    return found


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sel = ap.add_mutually_exclusive_group(required=True)
    sel.add_argument("--task", help="check only rows owned by this task, e.g. T5")
    sel.add_argument("--all", action="store_true", help="check every non-skip row")
    ap.add_argument("--portmap", default=str(ROOT / "rust" / "portmap.tsv"))
    ap.add_argument("--src", default=str(ROOT / "rust"))
    args = ap.parse_args(argv)

    rows = read_portmap(args.portmap)
    if args.task and not any(r[2] == args.task for r in rows):
        print(f"unknown task {args.task}: no rows in {args.portmap}", file=sys.stderr)
        return 2
    have = markers(args.src)
    wanted = [r for r in rows if not r[2].startswith("skip:") and (args.all or r[2] == args.task)]
    missing = [r for r in wanted if r[0] not in have and (r[1] == "-" or r[1] not in have)]
    for name, alias, task in missing:
        print(f"{task}\t{name}" + (f"\t(alias {alias})" if alias != "-" else ""))
    scope = "all tasks" if args.all else args.task
    print(f"{scope}: {len(wanted) - len(missing)}/{len(wanted)} covered", file=sys.stderr)
    return 1 if missing else 0


if __name__ == "__main__":
    sys.exit(main())
