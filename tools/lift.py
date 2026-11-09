"""Shrink Ghidra's YYC decompilation into a compact pseudo-C for translation.

Removes RValue bookkeeping (free/copy/kind checks, stack-trace frames, declarations),
names known runtime helpers and turns variable/builtin access into readable forms:
  &global.x, &self.x, &other.x, &V->x, name!(argc, args), // L<gml line>, 1080.0

Usage: python3 -I tools/lift.py [--decomp re/decomp] [--out re/lifted]
"""

import argparse
import json
import re
import struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

HELPERS = {
    "FUN_14002eee0": "OP_MUL",
    "FUN_14002ebd0": "OP_DIV",
    "FUN_140024b00": "OP_SUB",
    "FUN_14072b5f0": "clamp",
    "FUN_14072b950": "min_n",
    "FUN_14072b7b0": "max_n",
    "FUN_1406c0800": "TO_BOOL",
    "FUN_1406c2750": "TO_REAL",
    "FUN_1406f8660": "GET_BUILTIN_VAR",
}

_DECL = re.compile(r"^\s*(?:undefined\d*|int|uint|bool|byte|char|double|float|longlong|ulonglong|code|short|ushort|"
                   r"undefined1|undefined2|undefined4|undefined8)\b[^=;()]*;\s*$")
_VARREF = re.compile(r"\(\*\*\(code \*\*\)\(\*(\w+) \+ (?:8|0x10)\)\)\(\1,vid_(\w+)\)")
_CONCAT = re.compile(r"CONCAT44\(\w+,(vid_\w+)\)")
_BUILTIN = re.compile(r"YYGML_CallBuiltin\(param_1,param_2,&?\w+,(\d+),vid_(\w+),(&?\w+)\)")
_FREE_BLOCK = re.compile(r"\n[ \t]*if \(\(1 << \([^\n]*& 0x46U\) != 0\) \{\n[ \t]*FREE_RValue\([^\n]*\);\n[ \t]*\}")
_COPY_BLOCK = re.compile(r"\n([ \t]*)if \(\(1 << \([^\n]*& 0x46U\) == 0\) \{\n[ \t]*([^\n]*;)\n[ \t]*\}\n"
                         r"[ \t]*else \{\n[ \t]*COPY_RValue_Post\([^\n]*\);\n[ \t]*\}")
_HEXDBL = re.compile(r"(?:\(undefined8 \*\))?0x([0-9a-f]{16})\b")
_CMP = re.compile(r"YYCompareVal\(([^,]+),([^,]+),DAT_140f4d210,\d+\)")


def _double(m: re.Match) -> str:
    v = struct.unpack(">d", bytes.fromhex(m.group(1)))[0]
    if v != v or abs(v) > 1e15 or (v != 0 and abs(v) < 1e-9):
        return m.group(0)
    return repr(v)


def _line_var(src: str) -> str | None:
    m = re.search(r"g_pCurrentStackTrace = &\w+;\n\s*(\w+) = (?:0x[0-9a-f]+|\d+);", src)
    return m.group(1) if m else None


_VID_ASSIGN = re.compile(r"^\s*(\w+) = (vid_\w+);$")
_INDIRECT = re.compile(r"YYGML_CallBuiltin\(param_1,param_2,&?\w+,(\d+),(\w+),(&?\w+)\)")


def _indirect_builtins(src: str) -> str:
    """uVar8 = vid_f; ... YYGML_CallBuiltin(..., uVar8, ...) -> f!(argc, args)."""
    held, out = {}, []
    for line in src.splitlines():
        m = _VID_ASSIGN.match(line)
        if m:
            held[m.group(1)] = m.group(2)[4:]
            continue
        line = _INDIRECT.sub(lambda c: f"{held[c.group(2)]}!({c.group(1)}, {c.group(3)})" if c.group(2) in held else c.group(0), line)
        out.append(line)
    return "\n".join(out)


_CONST = re.compile(r"(?:CONCAT44\(uRam[0-9a-f]+,)?\b_?DAT_(14[0-9a-f]{7})\b\)?")
_ASSET = re.compile(r"\b0x(10000(?:0[0-9a-f]))(0000[0-9a-f]{4})\b")
ASSET_KINDS = {0: "object", 1: "sprite", 2: "sound", 3: "room", 6: "font", 8: "shader"}


def _inline_constants(src: str, consts: dict, assets: dict) -> str:
    def const(m: re.Match) -> str:
        key = "0x" + m.group(1)
        if key not in consts:
            return m.group(0)
        v = consts[key]
        text = m.group(0)
        tail = ")" if not text.startswith("CONCAT44") and text.endswith(")") else ""
        if isinstance(v, bool):
            return ("true" if v else "false") + tail
        if v is None:
            return "undefined" + tail
        if isinstance(v, str):
            return '"' + v.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"' + tail
        return repr(v) + tail

    def asset(m: re.Match) -> str:
        kind, idx = int(m.group(1), 16) & 0xFF, int(m.group(2), 16)
        names = assets.get(kind, [])
        label = names[idx] if idx < len(names) else str(idx)
        return f"{ASSET_KINDS.get(kind, 'asset' + str(kind))}({label})"

    src = _ASSET.sub(asset, src)
    return _CONST.sub(const, src)


def lift(src: str, consts: dict | None = None, assets: dict | None = None) -> str:
    line_var = _line_var(src)
    if consts is not None or assets is not None:
        src = _inline_constants(src, consts or {}, assets or {})
    src = _FREE_BLOCK.sub("", src)
    src = _COPY_BLOCK.sub(lambda m: "\n" + m.group(1) + m.group(2), src)
    src = _VARREF.sub(lambda m: {"g_pGlobal": "&global.", "param_1": "&self.", "param_2": "&other."}.get(m.group(1), f"&{m.group(1)}->") + m.group(2), src)
    src = _CONCAT.sub(r"\1", src)
    src = _BUILTIN.sub(r"\2!(\1, \3)", src)
    src = _indirect_builtins(src)
    src = _CMP.sub(r"CMP(\1,\2)", src)
    for k, v in HELPERS.items():
        src = src.replace(k + "(", v + "(")
    src = _HEXDBL.sub(_double, src)
    out = []
    for line in src.splitlines():
        s = line.strip()
        if not s or _DECL.match(line):
            continue
        if "g_pCurrentStackTrace" in s or s.startswith("_eh_vector_") or s.startswith("FREE_RValue("):
            continue
        if re.match(r"^uStack_\w+ = [^;]*;$", s) or re.match(r"^\w+ = 0xffffff;$", s):
            continue
        if re.match(r"^uVar\d+ = \(undefined4\)\(\(ulonglong\)\w+ >> 0x20\);$", s):
            continue
        if re.match(r"^FUN_1406d83d0\(g_pGlobal\);$", s) or s == "FUN_1406d8320(1);":
            continue
        if re.match(r'^\w+ = "gml_[^"]*";$', s):
            continue
        if line_var:
            m = re.match(rf"^{line_var} = (0x[0-9a-f]+|\d+);$", s)
            if m:
                out.append(line[: len(line) - len(line.lstrip())] + f"// L{int(m.group(1), 0)}")
                continue
        m = re.match(r"^COPY_RValue\(&?(\w+),&?(\w+)\);$", s)
        if m:
            out.append(line[: len(line) - len(line.lstrip())] + f"{m.group(1)} = {m.group(2)};")
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--decomp", type=Path, default=REPO / "re" / "decomp")
    ap.add_argument("--out", type=Path, default=REPO / "re" / "lifted")
    args = ap.parse_args()
    consts_file = REPO / "re" / "constants.json"
    consts = json.loads(consts_file.read_text()) if consts_file.exists() else {}
    assets = {}
    for kind, name in [(0, "objects"), (1, "sprites"), (2, "sounds"), (3, "rooms"), (6, "fonts")]:
        f = REPO / "build" / "assets" / f"{name}.json"
        if f.exists():
            assets[kind] = list(json.loads(f.read_text()))
    assets[8] = ["sh_blackmetal", "sh_glow_blur", "sh_stinky"]
    before = after = 0
    for p in sorted(args.decomp.glob("*/*.c")):
        src = p.read_text()
        res = lift(src, consts, assets)
        before += src.count("\n")
        after += res.count("\n")
        dest = args.out / p.parent.name / (p.stem + ".txt")
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(res)
    print(f"lifted {before} -> {after} lines into {args.out}")


if __name__ == "__main__":
    main()
