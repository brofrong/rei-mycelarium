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


def test_propagation_moves_drawing_wrappers():
    tasks = {"weather_draw_front": "T12", "rain_gloom_draw": "T16", "wld": "T3", "weather_step": "T12",
             "draw_event": "T16", "draw_event2": "T16", "edge_x": "T10", "trunk_edges": "T15", "climb_x": "T10",
             "chain_a": "T6", "chain_b": "T6", "wrap_draw": "T13", "chain_c": "T6", "anon__9__chain_c": "T6", "steam_x": "skip:steam"}
    graph = {
        # calls render + a logic query, called only by a render event -> moves (T12 -> T16)
        "weather_draw_front": {"rain_gloom_draw", "wld"}, "draw_event": {"weather_draw_front", "chain_a"},
        # weather_step calls only logic -> stays
        "weather_step": {"wld"},
        # edge_x reads data built by render-owned code but is called by logic -> stays
        "edge_x": {"trunk_edges"}, "climb_x": {"edge_x"},
        # chain: chain_b wraps render but is called by logic chain_a -> both stay (conservative);
        # chain_c moves (wrapper), then its closure anon__9__chain_c follows -> fixed point
        "chain_a": {"chain_b", "steam_x"}, "chain_b": {"wrap_draw"},
        "draw_event2": {"chain_c"}, "chain_c": {"wrap_draw", "wld"},
    }
    out = P.propagate(dict(tasks), graph)
    assert out["weather_draw_front"] == "T16"
    assert out["weather_step"] == "T12"
    assert out["edge_x"] == "T10"
    assert out["chain_b"] == "T6" and out["chain_a"] == "T6"
    assert out["chain_c"] == "T13" and out["anon__9__chain_c"] == "T13"


def test_propagation_closures_and_fixed_rows():
    tasks = {"caves_debug_draw": "T13", "anon__68900__caves_debug_draw__scr_world": "T6",
             "anon__5__parent_fn": "T8", "parent_fn": "T8", "kept": "T3", "r": "T14"}
    graph = {"kept": {"r"}}
    out = P.propagate(dict(tasks), graph, fixed={"kept"})
    assert out["anon__68900__caves_debug_draw__scr_world"] == "T13"
    assert out["anon__5__parent_fn"] == "T8"
    assert out["kept"] == "T3"


def test_call_graph_uses_aliases_and_skips_gm():
    funcs = [("anon__1__x", "c", "function anon__1__x(self, other)\n  self.myc_b(self, other)\n  gm.draw_text(0)\nend"),
             ("anon__2__x", "c", "function anon__2__x(self, other)\n  return 1\nend"),
             ("f", "c", "function f(self, other)\n  f(self, other)\n  anon__2__x(self, other)\nend")]
    graph = P.call_graph(funcs, {"anon__2__x": "myc_b"})
    assert graph["anon__1__x"] == {"anon__2__x"}
    assert graph["f"] == {"anon__2__x"}


def test_forest_non_drawing_rows_go_to_core():
    tasks = {"bush_draw": "T16", "stump_make": "T16", "surface_get_chunk": "T16", "weather_draw_front": "T16",
             "rain_gloom_draw": "T16", "fg_init": "T16", "art": "T15", "hud_thing": "T17", "logic": "T8"}
    graph = {"stump_make": {"surface_get_chunk"},          # state only -> T5
             "weather_draw_front": {"rain_gloom_draw"},     # wrapper of a drawing T16 row -> stays
             "fg_init": {"art"},                            # calls another render task -> stays
             "logic": {"stump_make"}}
    draws = {"bush_draw", "rain_gloom_draw"}
    out = P.forest_logic_to_core(dict(tasks), graph, draws)
    assert out["stump_make"] == "T5" and out["surface_get_chunk"] == "T5"
    assert out["bush_draw"] == "T16" and out["rain_gloom_draw"] == "T16"
    assert out["weather_draw_front"] == "T16" and out["fg_init"] == "T16"
    assert out["hud_thing"] == "T17" and out["logic"] == "T8"
