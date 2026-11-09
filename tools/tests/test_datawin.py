import pytest

from myco_extract.datawin import DataWin, DataWinError


def test_gen8_real(datawin):
    g = datawin.gen8()
    assert (g["name"], g["filename"], g["window_w"], g["window_h"], g["bytecode"]) == ("MYCO", "MYCO", 1920, 1080, 17)


def test_chunks_real(datawin):
    for name in ["GEN8", "SPRT", "ROOM", "OBJT", "TPAG", "TXTR", "AUDO", "SHDR", "FONT", "STRG"]:
        assert name in datawin.chunks
    assert "CODE" not in datawin.chunks  # YYC


def test_chunk_counts_real(datawin):
    counts = {n: len(datawin.chunk_list(n)) for n in ["OBJT", "ROOM", "SPRT", "SOND", "SHDR", "FONT", "TXTR"]}
    assert counts == {"OBJT": 10, "ROOM": 3, "SPRT": 167, "SOND": 24, "SHDR": 3, "FONT": 4, "TXTR": 5}


def test_not_datawin(tmp_path):
    p = tmp_path / "data.win"
    p.write_bytes(b"NOPE" + b"\0" * 8)
    with pytest.raises(DataWinError, match="not a GameMaker data.win"):
        DataWin.open(str(p))


def test_missing(tmp_path):
    with pytest.raises(DataWinError, match="game not found"):
        DataWin.open(str(tmp_path / "nope.win"))
