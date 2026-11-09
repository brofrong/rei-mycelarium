import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import portmap_init as P  # noqa: E402


def test_draw_builtins_detected():
    body = "function f(self, other)\n  gm.draw_set_alpha(1)\n  gm.gpu_set_blendmode(0)\nend"
    assert P.draw_calls(body) == ["draw_set_alpha", "gpu_set_blendmode"]


def test_size_queries_are_not_drawing():
    body = ("function f(self, other, s)\n  if gm.surface_exists(s) then return gm.surface_get_width(s) end\n"
            "  local u = gm.shader_get_uniform(sh, 'k')\nend")
    assert P.draw_calls(body) == []


def test_drawing_row_moves_to_render_task():
    body = "function lakes_init(self, other)\n  local s = gm.surface_create(4, 4)\nend"
    assert P.draw_reassign("T5", body, "lakes_init") == "T13"
    assert P.draw_reassign("T8", "gm.vertex_begin(b, f)", "x") == "T14"
    assert P.draw_reassign("T10", "gm.draw_sprite(s, 0, 0, 0)", "x") == "T15"
    assert P.draw_reassign("T12", "gm.draw_primitive_begin(4)", "x") == "T16"
    assert P.draw_reassign("T3", "gm.draw_set_font(f)", "x") == "T17"
    assert P.draw_reassign("T11", "gm.draw_text(0, 0, s)", "x") == "T17"


def test_non_drawing_and_render_rows_unchanged():
    assert P.draw_reassign("T5", "local x = gm.round(1.5)", "x") == "T5"
    assert P.draw_reassign("T17", "gm.draw_text(0, 0, s)", "x") == "T17"
    assert P.draw_reassign("T3", "gm.vertex_delete_buffer(b)", "load_game") == "T3"


def test_body_stops_at_column_zero_end(tmp_path):
    (tmp_path / "a.lua").write_text(
        "function a(self, other)\n  gm.with(self, o, function(self, other)\n    return 1\n  end)\nend\n"
        "function b(self, other)\n  gm.draw_text(0, 0, 'x')\nend\n")
    funcs, _ = P.collect(tmp_path)
    bodies = {n: b for n, _, b in funcs}
    assert P.draw_calls(bodies["a"]) == []
    assert P.draw_calls(bodies["b"]) == ["draw_text"]
