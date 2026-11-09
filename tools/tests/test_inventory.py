import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from inventory import builtins_used, callees  # noqa: E402

SRC = "void gml_Script_x(void){ F_draw_sprite_ext(a,b); gml_Script_mycelium_add_length(c); F_draw_sprite_ext(d); }"


def test_builtins_used():
    assert builtins_used(SRC) == {"draw_sprite_ext"}


def test_callees():
    assert callees(SRC) == {"gml_Script_mycelium_add_length"}


def test_callees_ignores_own_definition_and_strings():
    src = '// gml_Script_x @ 0x1\nvoid gml_Script_x(void){ s = "gml_Script_y"; gml_Script_z(); }'
    assert callees(src) == {"gml_Script_z"}


def test_builtins_via_vid_ids():
    src = "void gml_Script_x(void){ YYGML_CallBuiltin(a,b,&r,1,CONCAT44(u,vid_draw_sprite),args); p = vid_chunk_w; }"
    assert builtins_used(src, known={"draw_sprite", "draw_text"}) == {"draw_sprite"}
