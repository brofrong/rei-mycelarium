"""Build re/inventory.md and re/builtins.json from the Ghidra export in re/decomp/.

Usage: python3 -I tools/inventory.py [--decomp re/decomp] [--out re]
"""

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

_COMMENT = re.compile(r"//[^\n]*|/\*.*?\*/", re.S)
_STRING = re.compile(r'"(?:\\.|[^"\\])*"')
_BUILTIN_CALL = re.compile(r"\bF_([A-Za-z0-9_]+)\s*\(")
_VID = re.compile(r"\bvid_([A-Za-z0-9_]+)\b")
_GML_CALL = re.compile(r"\b(gml_(?:Script|Object|GlobalScript|RoomCC)_[A-Za-z0-9_]+)\s*\(")
_GML_REF = re.compile(r"\b(gml_(?:Script|Object|GlobalScript|RoomCC)_[A-Za-z0-9_]+)\b")
_DEFINITION = re.compile(r"^[A-Za-z_][\w \*]*?\b(gml_\w+)\s*\(", re.M)


def _code(src: str) -> str:
    return _STRING.sub('""', _COMMENT.sub("", src))


def _builtin_hits(code: str, known: set[str] | None) -> list[str]:
    hits = _BUILTIN_CALL.findall(code)
    if known:
        hits += [v for v in _VID.findall(code) if v in known]
    return hits


def builtins_used(c_source: str, known: set[str] | None = None) -> set[str]:
    """Builtins called directly (F_name(...)) or by id (vid_name, when name is a registered builtin)."""
    return set(_builtin_hits(_code(c_source), known))


def _defined(code: str) -> set[str]:
    return set(_DEFINITION.findall(code))


def callees(c_source: str) -> set[str]:
    code = _code(c_source)
    return set(_GML_CALL.findall(code)) - _defined(code)


def references(c_source: str) -> set[str]:
    """All gml_* functions mentioned (calls plus method/function pointers)."""
    code = _code(c_source)
    return set(_GML_REF.findall(code)) - _defined(code)


def load_index(decomp: Path) -> list[dict]:
    rows = []
    lines = (decomp / "index.tsv").read_text().splitlines()[1:]
    for line in lines:
        name, addr, size, status, rel = line.split("\t")
        rows.append({"name": name, "address": int(addr, 16), "size": int(size), "status": status, "file": rel})
    return rows


def script_file(name: str) -> str | None:
    m = re.search(r"__(scr_[a-z_]+?)(?:_|$)", name) or re.search(r"_40_(scr_[a-z_]+?)$", name)
    return m.group(1) if m else None


def object_of(name: str) -> str | None:
    m = re.match(r"gml_Object_(obj_[A-Za-z0-9_]+?)_(Create|Step|Draw|CleanUp|Other|Destroy|Alarm|Mouse|KeyPress|Keyboard|KeyRelease|Collision|Async)_\d+$", name)
    return m.group(1) if m else None


def reachable(graph: dict[str, set[str]], roots: list[str]) -> set[str]:
    seen, stack = set(), list(roots)
    while stack:
        n = stack.pop()
        if n in seen:
            continue
        seen.add(n)
        stack.extend(graph.get(n, ()))
    return seen


def build(decomp: Path, out: Path) -> None:
    rows = load_index(decomp)
    sources = {}
    for r in rows:
        p = decomp / r["file"]
        sources[r["name"]] = p.read_text() if p.exists() else ""

    known_file = decomp.parent / "runtime_builtins.txt"
    known = set(known_file.read_text().split()) if known_file.exists() else None
    builtins: dict[str, dict] = defaultdict(lambda: {"calls": 0, "functions": []})
    graph = {}
    for name, src in sources.items():
        code = _code(src)
        hits = _builtin_hits(code, known)
        for b in hits:
            builtins[b]["calls"] += 1
        for b in sorted(set(hits)):
            builtins[b]["functions"].append(name)
        graph[name] = references(src)

    # group scripts into scr_* files: explicit suffix, then GlobalScript references, then address proximity
    group = {}
    for r in rows:
        f = script_file(r["name"])
        if r["name"].startswith("gml_GlobalScript_"):
            f = r["name"][len("gml_GlobalScript_"):]
        if f:
            group[r["name"]] = f
    for gs, refs in graph.items():
        if gs.startswith("gml_GlobalScript_"):
            for ref in refs:
                group.setdefault(ref, gs[len("gml_GlobalScript_"):])
    guessed = set()
    by_addr = sorted(rows, key=lambda r: r["address"])
    for i, r in enumerate(by_addr):
        if r["name"] in group or object_of(r["name"]):
            continue
        near = [x for x in by_addr[max(0, i - 8):i + 9] if x["name"] in group and x["name"] not in guessed]
        if near:
            best = min(near, key=lambda x: abs(x["address"] - r["address"]))
            group[r["name"]] = group[best["name"]]
            guessed.add(r["name"])

    out.mkdir(parents=True, exist_ok=True)
    (out / "builtins.json").write_text(json.dumps(dict(sorted(builtins.items())), indent=1))

    md = ["# Mycelarium: инвентарь функций", "",
          f"Функций в выгрузке: {len(rows)}, декомпилировано: {sum(r['status'] == 'ok' for r in rows)}, "
          f"встроенных функций используется: {len(builtins)}.", "",
          "## Объекты и события", ""]
    events = defaultdict(list)
    for r in rows:
        o = object_of(r["name"])
        if o:
            events[o].append(r["name"])
    for o in sorted(events):
        md.append(f"### {o}")
        for ev in sorted(events[o]):
            reach = reachable(graph, [ev]) - {ev}
            md.append(f"- `{ev}` — вызывает напрямую: {len(graph.get(ev, ()))}, транзитивно: {len(reach)}")
        md.append("")
    md += ["## Скрипты по файлам", "", "`(?)` — файл угадан по соседству адресов.", ""]
    files = defaultdict(list)
    for r in rows:
        if not object_of(r["name"]):
            files[group.get(r["name"], "unknown")].append(r)
    for f in sorted(files):
        md.append(f"### {f} ({len(files[f])})")
        for r in sorted(files[f], key=lambda r: r["address"]):
            mark = " (?)" if r["name"] in guessed else ""
            md.append(f"- `{r['name']}`{mark} @ 0x{r['address']:x}, {r['size']} B, {r['status']}")
        md.append("")
    md += ["## Встроенные функции по частоте", "", "| функция | вызовов | в функциях |", "|---|---|---|"]
    for b, info in sorted(builtins.items(), key=lambda kv: -kv[1]["calls"]):
        md.append(f"| `{b}` | {info['calls']} | {len(info['functions'])} |")
    (out / "inventory.md").write_text("\n".join(md) + "\n")
    (out / "callgraph.json").write_text(json.dumps({k: sorted(v) for k, v in sorted(graph.items())}, indent=1))
    (out / "groups.json").write_text(json.dumps(dict(sorted(group.items())), indent=1))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--decomp", type=Path, default=REPO / "re" / "decomp")
    ap.add_argument("--out", type=Path, default=REPO / "re")
    args = ap.parse_args()
    build(args.decomp, args.out)
    print(f"inventory written to {args.out}")


if __name__ == "__main__":
    main()
