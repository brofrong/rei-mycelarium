"""Recover YYC runtime-initialised constants (strings etc.) from Mycelarium.exe.

Static initialisers look like: lea rdx,[value]; lea rcx,[dst]; call <setter>.
Writes re/constants.json: {"0x14137a210": "logo_surface", ...} for string setters.
Usage: python3 -I tools/yyc_constants.py [--exe PATH] [--out re/constants.json]
"""

import argparse
import collections
import json
import re
import struct
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
STRING_SETTER = 0x1406DAAE0


def load_pe(path: Path):
    d = path.read_bytes()
    pe = struct.unpack_from("<I", d, 0x3C)[0]
    optsz = struct.unpack_from("<H", d, pe + 20)[0]
    base = struct.unpack_from("<Q", d, pe + 24 + 24)[0]
    o = pe + 24 + optsz
    secs = [struct.unpack_from("<IIII", d, o + 8 + 40 * i) for i in range(struct.unpack_from("<H", d, pe + 6)[0])]
    return d, base, secs


def triples(d, base, secs):
    vsz, va, rsz, raw = secs[0]
    text = d[raw:raw + rsz]
    pat = re.compile(rb"\x48\x8d\x15(....)\x48\x8d\x0d(....)\xe8(....)", re.S)
    for m in pat.finditer(text):
        at = base + va + m.start()
        src = at + 7 + struct.unpack("<i", m.group(1))[0]
        dst = at + 14 + struct.unpack("<i", m.group(2))[0]
        call = at + 19 + struct.unpack("<i", m.group(3))[0]
        yield src, dst, call


def cstring(d, base, secs, va):
    v = va - base
    for vsz, sva, rsz, raw in secs:
        if sva <= v < sva + rsz:
            p = raw + v - sva
            e = d.find(b"\0", p)
            try:
                return d[p:e].decode("utf-8")
            except UnicodeDecodeError:
                return None
    return None


def recover(exe: Path) -> tuple[dict, collections.Counter]:
    d, base, secs = load_pe(exe)
    out, calls = {}, collections.Counter()
    for src, dst, call in triples(d, base, secs):
        calls[call] += 1
        if call == STRING_SETTER:
            s = cstring(d, base, secs, src)
            if s is not None:
                out[f"0x{dst:x}"] = s
    return out, calls


def numeric_constants(exe: Path, addrs) -> dict:
    """Decode 16-byte RValue constants in initialised data: real -> float, bool, undefined -> None."""
    d, base, secs = load_pe(exe)
    out = {}
    for a in addrs:
        v = a - base
        for vsz, sva, rsz, raw in secs:
            if sva <= v < sva + min(vsz, rsz) - 16:
                p = raw + v - sva
                val, flags, kind = struct.unpack_from("<QII", d, p)
                if flags != 0:
                    break
                if kind == 0:
                    out[f"0x{a:x}"] = struct.unpack_from("<d", d, p)[0]
                elif kind == 13:
                    out[f"0x{a:x}"] = val != 0
                elif kind == 5:
                    out[f"0x{a:x}"] = None
                break
    return out


def referenced_addrs(decomp: Path) -> set[int]:
    addrs = set()
    for p in decomp.glob("*/*.c"):
        addrs.update(int(a, 16) for a in re.findall(r"\b_?DAT_(14[0-9a-f]{7})\b", p.read_text()))
    return addrs


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--exe", type=Path, default=Path.home() / "GameHub/steamapps/common/Mycelarium/Mycelarium.exe")
    ap.add_argument("--out", type=Path, default=REPO / "re" / "constants.json")
    args = ap.parse_args()
    consts, calls = recover(args.exe)
    for k, v in numeric_constants(args.exe, referenced_addrs(REPO / "re" / "decomp")).items():
        consts.setdefault(k, v)
    args.out.write_text(json.dumps(consts, ensure_ascii=False, indent=1))
    print(f"{len(consts)} string constants -> {args.out}; setters: {[(hex(k), v) for k, v in calls.most_common(5)]}")


if __name__ == "__main__":
    main()
