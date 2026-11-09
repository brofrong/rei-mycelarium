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


@pytest.fixture(scope="session")
def out_dir(tmp_path_factory, datawin):
    from myco_extract.extract import extract_all

    out = tmp_path_factory.mktemp("assets")
    extract_all(game_dir(), out, link=False)
    return out


@pytest.fixture(scope="session")
def assets(out_dir):
    import json

    return {k: json.loads((out_dir / f"{k}.json").read_text()) for k in ["rooms", "objects", "sprites", "sounds", "fonts", "game"]}
