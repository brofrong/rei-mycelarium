import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from check_chunk import check, lua_name  # noqa: E402


def test_lua_names():
    assert lua_name("gml_Script_hedgehog_frame_h") == "hedgehog_frame_h"
    assert lua_name("gml_Object_obj_hud_Create_0") == "gml_Object_obj_hud_Create_0"


def test_missing_and_stub_detection():
    rows = [("gml_Script_a", 10), ("gml_Script_b", 400), ("gml_Script_c", 300)]
    lua = ("function a(self, other)\n  return 1\nend\n"
           "function b(self, other)\n  -- TODO(port): not translated\n  return gm.undefined\nend\n")
    problems = check(rows, lua)
    assert any("c: missing" in p for p in problems)
    assert any(p.startswith("b: stub") for p in problems)
    assert not any(p.startswith("a:") for p in problems)
