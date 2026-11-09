"""GameMaker texture pages: BZ2+QOI ("2zoq"), QOI ("fioq") and PNG."""

import bz2
import io
import struct

from .datawin import DataWin, DataWinError


def _s(v: int, bits: int) -> int:
    return v - (1 << bits) if v & (1 << (bits - 1)) else v


def decode_gm_qoi(buf: bytes) -> tuple[int, int, bytes]:
    """Decode GameMaker's QOI variant (old QOI draft) into RGBA bytes."""
    if buf[:4] != b"fioq":
        raise DataWinError("not a GameMaker QOI stream")
    w, h, length = struct.unpack_from("<HHI", buf, 4)
    p, end = 12, min(12 + length, len(buf))
    r, g, b, a = 0, 0, 0, 255
    index = [(0, 0, 0, 0)] * 64
    out = bytearray()
    run = 0
    for _ in range(w * h):
        if run > 0:
            run -= 1
        elif p < end:
            b1 = buf[p]
            p += 1
            if b1 & 0xC0 == 0x00:
                r, g, b, a = index[b1 & 63]
            elif b1 & 0xE0 == 0x40:
                run = b1 & 31
            elif b1 & 0xE0 == 0x60:
                run = (((b1 & 31) << 8) | buf[p]) + 32
                p += 1
            elif b1 & 0xC0 == 0x80:
                r = (r + _s((b1 >> 4) & 3, 2)) & 255
                g = (g + _s((b1 >> 2) & 3, 2)) & 255
                b = (b + _s(b1 & 3, 2)) & 255
            elif b1 & 0xE0 == 0xC0:
                m = (b1 << 8) | buf[p]
                p += 1
                r = (r + _s((m >> 8) & 31, 5)) & 255
                g = (g + _s((m >> 4) & 15, 4)) & 255
                b = (b + _s(m & 15, 4)) & 255
            elif b1 & 0xF0 == 0xE0:
                m = (b1 << 16) | (buf[p] << 8) | buf[p + 1]
                p += 2
                r = (r + _s((m >> 15) & 31, 5)) & 255
                g = (g + _s((m >> 10) & 31, 5)) & 255
                b = (b + _s((m >> 5) & 31, 5)) & 255
                a = (a + _s(m & 31, 5)) & 255
            else:
                if b1 & 8:
                    r = buf[p]
                    p += 1
                if b1 & 4:
                    g = buf[p]
                    p += 1
                if b1 & 2:
                    b = buf[p]
                    p += 1
                if b1 & 1:
                    a = buf[p]
                    p += 1
            index[(r ^ g ^ b ^ a) & 63] = (r, g, b, a)
        out += bytes((r, g, b, a))
    return w, h, bytes(out)


def texture_pages(dw: DataWin) -> list[dict]:
    pages = []
    for i, ptr in enumerate(dw.chunk_list("TXTR")):
        pages.append({"index": i, "w": dw.u32(ptr + 12), "h": dw.u32(ptr + 16), "data_ptr": dw.u32(ptr + 24)})
    return pages


def decode_texture(dw: DataWin, txtr_entry_ptr: int) -> tuple[int, int, bytes]:
    off = dw.u32(txtr_entry_ptr + 24)
    magic = bytes(dw.mv[off:off + 4])
    if magic == b"2zoq":
        raw = bz2.BZ2Decompressor().decompress(bytes(dw.mv[off + 12:]))
        return decode_gm_qoi(raw)
    if magic == b"fioq":
        return decode_gm_qoi(bytes(dw.mv[off:]))
    if magic == b"\x89PNG":
        from PIL import Image

        length = dw.u32(txtr_entry_ptr + 8)
        img = Image.open(io.BytesIO(bytes(dw.mv[off:off + length]))).convert("RGBA")
        return img.width, img.height, img.tobytes()
    raise DataWinError(f"unknown texture format {magic!r} at {off}")
