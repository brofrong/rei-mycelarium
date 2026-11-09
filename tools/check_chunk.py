"""Check a translated chunk: every function present, no stubs.

Usage: python3 -I tools/check_chunk.py <chunk>   (reads re/chunks/<chunk>.txt and game/myco/gen/<chunk>.lua)
Exit code 0 when clean; prints one line per problem otherwise.
"""

import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
STUB_MARKERS = re.compile(r"not translated|simplified|placeholder|stub", re.I)


def lua_name(gml: str) -> str:
    return gml[len("gml_Script_"):] if gml.startswith("gml_Script_") else gml


def bodies(lua: str) -> dict[str, str]:
    out = {}
    starts = [(m.start(), m.group(1)) for m in re.finditer(r"^function ([A-Za-z_][A-Za-z0-9_]*)\s*\(", lua, re.M)]
    for i, (pos, name) in enumerate(starts):
        end = starts[i + 1][0] if i + 1 < len(starts) else len(lua)
        out[name] = lua[pos:end]
    return out


def check(rows: list[tuple[str, int]], lua: str) -> list[str]:
    defs = bodies(lua)
    problems = []
    for gml, size in rows:
        name = lua_name(gml)
        body = defs.get(name)
        if body is None:
            problems.append(f"{name}: missing")
            continue
        code_lines = [l for l in body.splitlines()[1:] if l.strip() and not l.strip().startswith("--") and l.strip() != "end"]
        if STUB_MARKERS.search(body) or (size >= 120 and len(code_lines) * 25 < size):
            problems.append(f"{name}: stub? ({len(code_lines)} Lua lines for {size} lifted lines)")
    return problems


def main() -> int:
    chunk = sys.argv[1]
    rows = []
    for line in (REPO / "re" / "chunks" / f"{chunk}.txt").read_text().splitlines():
        name, _, size = line.split("\t")
        rows.append((name, int(size)))
    lua_file = REPO / "game" / "myco" / "gen" / f"{chunk}.lua"
    if not lua_file.exists():
        print(f"{lua_file} missing")
        return 1
    problems = check(rows, lua_file.read_text())
    r = subprocess.run(["luajit", "-bl", str(lua_file)], capture_output=True, text=True)
    if r.returncode != 0:
        problems.append("syntax: " + r.stderr.strip().splitlines()[-1])
    for p in problems:
        print(p)
    print(f"{chunk}: {len(rows) - len(problems)}/{len(rows)} ok")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
