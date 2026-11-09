import struct

from myco_extract.qoi import decode_gm_qoi, decode_texture, texture_pages


def _fioq(w, h, ops: bytes) -> bytes:
    return b"fioq" + struct.pack("<HHI", w, h, len(ops)) + ops


def test_color_and_run():
    # COLOR(r,g,b,a all set) then RUN_8 with 2 repeats -> 3 equal pixels
    w, h, px = decode_gm_qoi(_fioq(3, 1, bytes([0xF0 | 0xF, 10, 20, 30, 40, 0x40 | 1])))
    assert (w, h) == (3, 1) and px == bytes([10, 20, 30, 40]) * 3


def test_diff8_and_index():
    # start (0,0,0,255) -> index 63; DIFF_8 r-1,g+1,b+1 -> (255,1,1,255) -> index 0; INDEX 63 -> start again
    start_hash = (0 ^ 0 ^ 0 ^ 255) & 63
    ops = bytes([0xF0 | 0x1, 255, 0x80 | (3 << 4) | (1 << 2) | 1, start_hash])
    _, _, px = decode_gm_qoi(_fioq(3, 1, ops))
    assert px == bytes([0, 0, 0, 255, 255, 1, 1, 255, 0, 0, 0, 255])


def test_real_page_dims(datawin):
    pages = texture_pages(datawin)
    assert [(p["w"], p["h"]) for p in pages] == [(64, 64), (2048, 2048), (2048, 2048), (2048, 2048), (2048, 1024)]
    w, h, px = decode_texture(datawin, datawin.chunk_list("TXTR")[4])
    assert (w, h, len(px)) == (2048, 1024, 2048 * 1024 * 4)
