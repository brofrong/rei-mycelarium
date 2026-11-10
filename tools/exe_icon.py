"""Extract the game's own icon (the largest image of the exe's first RT_GROUP_ICON) from a Windows PE.

python3 -I tools/exe_icon.py [--exe Mycelarium.exe] [--scale N] OUT.png

Writes OUT.png, enlarged N times with nearest-neighbour so the pixel art stays crisp. Stdlib only.
"""

import argparse
import struct
import sys
import zlib
from pathlib import Path

RT_ICON, RT_GROUP_ICON = 3, 14
PNG_SIG = b"\x89PNG\r\n\x1a\n"


def _sections(pe: bytes) -> tuple[int, list[tuple[int, int, int, int]]]:
    """(resource-table RVA, [(rva, virtual size, raw offset, raw size)]) of a PE32/PE32+ image."""
    if pe[:2] != b"MZ":
        raise ValueError("not an MZ executable")
    nt = struct.unpack_from("<I", pe, 0x3C)[0]
    if pe[nt:nt + 4] != b"PE\0\0":
        raise ValueError("no PE header")
    n_sections, opt_size = struct.unpack_from("<H12xH", pe, nt + 6)
    opt = nt + 24
    magic = struct.unpack_from("<H", pe, opt)[0]
    dirs = opt + {0x10B: 96, 0x20B: 112}[magic]
    res_rva = struct.unpack_from("<I", pe, dirs + 2 * 8)[0]
    table = opt + opt_size
    secs = []
    for i in range(n_sections):
        vsize, rva, raw_size, raw_off = struct.unpack_from("<4I", pe, table + i * 40 + 8)
        secs.append((rva, vsize, raw_off, raw_size))
    return res_rva, secs


def _offset(secs, rva: int) -> int:
    for s_rva, vsize, raw_off, raw_size in secs:
        if s_rva <= rva < s_rva + max(vsize, raw_size):
            return raw_off + rva - s_rva
    raise ValueError(f"RVA {rva:#x} outside every section")


def _entries(pe: bytes, base: int, off: int) -> list[tuple[int, int, bool]]:
    """[(id, offset from the resource base, is a subdirectory)] of one resource directory (ids only)."""
    named, ids = struct.unpack_from("<12xHH", pe, base + off)
    out = []
    for i in range(named + ids):
        name, target = struct.unpack_from("<II", pe, base + off + 16 + i * 8)
        if not name & 0x80000000:
            out.append((name, target & 0x7FFFFFFF, bool(target & 0x80000000)))
    return out


def _first_leaf(pe: bytes, base: int, off: int) -> int:
    """Offset of the data entry under a name/id directory, taking the first language."""
    _, off, is_dir = _entries(pe, base, off)[0]
    return _first_leaf(pe, base, off) if is_dir else off


def resources(pe: bytes, rtype: int) -> dict[int, bytes]:
    """{resource id: data} of one resource type."""
    res_rva, secs = _sections(pe)
    base = _offset(secs, res_rva)
    out = {}
    for tid, toff, _ in _entries(pe, base, 0):
        if tid != rtype:
            continue
        for rid, roff, is_dir in _entries(pe, base, toff):
            leaf = _first_leaf(pe, base, roff) if is_dir else roff
            rva, size = struct.unpack_from("<II", pe, base + leaf)
            start = _offset(secs, rva)
            out[rid] = pe[start:start + size]
    return out


def largest_icon(pe: bytes) -> bytes:
    """The RT_ICON data (a BITMAPINFOHEADER DIB or PNG bytes) of the first group's biggest image."""
    groups = resources(pe, RT_GROUP_ICON)
    if not groups:
        raise ValueError("no icon group in the executable")
    group = groups[min(groups)]
    icons = resources(pe, RT_ICON)
    count = struct.unpack_from("<4xH", group)[0]
    best = None
    for i in range(count):
        w, _, _, _, _, bpp, _, icon_id = struct.unpack_from("<BBBBHHIH", group, 6 + i * 14)
        if icon_id in icons and (best is None or (w or 256, bpp) > best[0]):
            best = ((w or 256, bpp), icons[icon_id])
    if best is None:
        raise ValueError("icon group names no RT_ICON present")
    return best[1]


def dib_rgba(dib: bytes) -> tuple[int, int, list[bytes]]:
    """(width, height, top-down RGBA rows) of a 32-bit icon DIB (height field counts the AND mask too)."""
    hdr, w, h2, _, bpp = struct.unpack_from("<IiiHH", dib)
    if bpp != 32:
        raise ValueError(f"{bpp}-bit icon DIBs are not supported")
    h = abs(h2) // 2
    stride = w * 4
    rows = []
    for y in range(h):
        src = dib[hdr + y * stride:hdr + (y + 1) * stride]
        rgba = bytearray(stride)
        rgba[0::4], rgba[1::4], rgba[2::4], rgba[3::4] = src[2::4], src[1::4], src[0::4], src[3::4]
        rows.append(bytes(rgba))
    return w, h, rows[::-1] if h2 > 0 else rows


def png(w: int, h: int, rows: list[bytes], scale: int = 1) -> bytes:
    """An RGBA PNG of the rows, each pixel repeated `scale` times both ways."""
    raw = bytearray()
    for row in rows:
        wide = b"".join(row[x:x + 4] * scale for x in range(0, len(row), 4))
        raw += (b"\0" + wide) * scale

    def chunk(tag: bytes, data: bytes) -> bytes:
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    ihdr = struct.pack(">IIBBBBB", w * scale, h * scale, 8, 6, 0, 0, 0)
    return PNG_SIG + chunk(b"IHDR", ihdr) + chunk(b"IDAT", zlib.compress(bytes(raw), 9)) + chunk(b"IEND", b"")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--exe", type=Path, default=Path.home() / "GameHub/steamapps/common/Mycelarium/Mycelarium.exe")
    ap.add_argument("--scale", type=int, default=1)
    ap.add_argument("out", type=Path)
    args = ap.parse_args()
    data = largest_icon(args.exe.read_bytes())
    if data.startswith(PNG_SIG):
        if args.scale != 1:
            raise SystemExit("the icon is PNG-compressed; --scale needs a DIB icon")
        args.out.write_bytes(data)
    else:
        args.out.write_bytes(png(*dib_rgba(data), scale=args.scale))
    return 0


if __name__ == "__main__":
    sys.exit(main())
