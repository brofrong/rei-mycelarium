import sys
from pathlib import Path

import pytest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

from myco_extract.datawin import DataWin, game_dir  # noqa: E402


@pytest.fixture(scope="session")
def datawin():
    path = game_dir() / "data.win"
    if not path.exists():
        pytest.skip(f"game not installed at {path}")
    return DataWin.open(str(path))
