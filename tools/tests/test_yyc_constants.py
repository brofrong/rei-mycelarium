import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from myco_extract.datawin import game_dir  # noqa: E402
from yyc_constants import recover  # noqa: E402


def test_known_string_constants():
    exe = game_dir() / "Mycelarium.exe"
    if not exe.exists():
        pytest.skip("game not installed")
    consts, _ = recover(exe)
    assert consts["0x14137a210"] == "logo_surface"
    assert len(consts) > 1000


def test_numeric_constants():
    exe = game_dir() / "Mycelarium.exe"
    if not exe.exists():
        pytest.skip("game not installed")
    from yyc_constants import numeric_constants
    nums = numeric_constants(exe, {0x140fd78a0, 0x140fd77c0, 0x140fd77d0})
    assert nums == {"0x140fd78a0": 1.0, "0x140fd77c0": False, "0x140fd77d0": None}
