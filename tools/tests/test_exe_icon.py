import struct
import zlib

import pytest

from exe_icon import PNG_SIG, dib_rgba, largest_icon, png
from myco_extract.datawin import game_dir

RED, GREEN, BLUE, CLEAR = b"\xff\0\0\xff", b"\0\xff\0\xff", b"\0\0\xff\xff", b"\xff\xff\xff\0"


def _dib() -> bytes:
    # 2x2, bottom-up BGRA (bottom row: blue, clear white; top row: red, green) + an all-zero AND mask
    hdr = struct.pack("<IiiHHIIiiII", 40, 2, 4, 1, 32, 0, 0, 0, 0, 0, 0)
    bgra = lambda p: bytes([p[2], p[1], p[0], p[3]])  # noqa: E731
    return hdr + bgra(BLUE) + bgra(CLEAR) + bgra(RED) + bgra(GREEN) + bytes(8)


def _pe(dib: bytes) -> bytes:
    """A minimal PE32+ whose .rsrc has RT_ICON 1 (three levels deep) and RT_GROUP_ICON 7 (two levels)."""
    rva, raw = 0x1000, 0x400
    d = lambda *e: struct.pack("<12xHH", 0, len(e)) + b"".join(struct.pack("<II", *x) for x in e)  # noqa: E731
    group = struct.pack("<HHH", 0, 1, 1) + struct.pack("<BBBBHHIH", 2, 2, 0, 0, 1, 32, len(dib), 1)
    rsrc = (
        d((3, 0x80000000 | 32), (14, 0x80000000 | 80))  # root @0
        + d((1, 0x80000000 | 56))  # RT_ICON names @32
        + d((0x409, 104))  # languages @56
        + d((7, 120))  # RT_GROUP_ICON names @80
        + struct.pack("<IIII", rva + 136, len(dib), 0, 0)  # data entry @104
        + struct.pack("<IIII", rva + 136 + len(dib), len(group), 0, 0)  # data entry @120
        + dib
        + group
    )
    opt = struct.pack("<H", 0x20B) + bytes(110) + bytes(16) + struct.pack("<II", rva, len(rsrc)) + bytes(13 * 8)
    coff = struct.pack("<HHIIIHH", 0x8664, 1, 0, 0, 0, len(opt), 0)
    section = b".rsrc\0\0\0" + struct.pack("<IIII16x", len(rsrc), rva, len(rsrc), raw)
    head = b"MZ" + bytes(0x3A) + struct.pack("<I", 0x40) + b"PE\0\0" + coff + opt + section
    return head.ljust(raw, b"\0") + rsrc


def _png_pixels(data: bytes) -> tuple[int, int, bytes]:
    assert data.startswith(PNG_SIG)
    w, h = struct.unpack_from(">II", data, 16)
    idat = data.index(b"IDAT")
    raw = zlib.decompress(data[idat + 4:idat + 4 + struct.unpack_from(">I", data, idat - 4)[0]])
    rows = [raw[y * (w * 4 + 1):(y + 1) * (w * 4 + 1)] for y in range(h)]
    assert all(r[0] == 0 for r in rows)
    return w, h, b"".join(r[1:] for r in rows)


def test_largest_icon_reads_the_group_through_both_directory_depths():
    assert largest_icon(_pe(_dib())) == _dib()


def test_dib_is_flipped_top_down_and_turned_rgba():
    assert dib_rgba(_dib()) == (2, 2, [RED + GREEN, BLUE + CLEAR])


def test_png_scales_nearest_neighbour():
    w, h, px = _png_pixels(png(*dib_rgba(_dib()), scale=2))
    assert (w, h) == (4, 4)
    assert px == (RED * 2 + GREEN * 2) * 2 + (BLUE * 2 + CLEAR * 2) * 2


def test_real_exe_icon_is_256():
    exe = game_dir() / "Mycelarium.exe"
    if not exe.exists():
        pytest.skip(f"game not installed at {exe}")
    w, h, _ = dib_rgba(largest_icon(exe.read_bytes()))
    assert (w, h) == (256, 256)
