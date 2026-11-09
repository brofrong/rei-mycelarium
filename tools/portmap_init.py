#!/usr/bin/env python3
"""Initial fill of rust/portmap.tsv: every translated game function -> owning task.

Prints `name<TAB>alias<TAB>task` for each `^function NAME` in game/myco/gen/*.lua (except the
`___struct___*` data shapes). `task` is `T<N>`, `skip:<reason>` or `T?` when no rule applies; the
`T?` rows are listed on stderr. Hand decisions for rows no rule covers live in MANUAL; a
function that calls drawing builtins goes to the render task of its logic task (ruling R7).

Usage: python3 -I tools/portmap_init.py > rust/portmap.tsv
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GEN = ROOT / "game" / "myco" / "gen"

FUNC_RE = re.compile(r"^function ([A-Za-z_][A-Za-z0-9_]*)\s*\(", re.M)
ALIAS_RE = re.compile(r"self\.([A-Za-z_][A-Za-z0-9_]*) = gm\.method\(self, ([A-Za-z_][A-Za-z0-9_]*)\)")


def collect(gen=GEN):
    """Return [(name, chunk_file_stem, body)] in file order, and {name: alias}.

    `body` is the text from `function NAME(` to the first column-0 `end`.
    """
    funcs, aliases = [], {}
    for path in sorted(gen.glob("*.lua")):
        text = path.read_text()
        for m in FUNC_RE.finditer(text):
            name = m.group(1)
            if name.startswith("___struct___"):
                continue
            end = re.compile(r"^end\b", re.M).search(text, m.end())
            funcs.append((name, path.stem, text[m.start():end.end() if end else len(text)]))
        for alias, name in ALIAS_RE.findall(text):
            if alias != name and name not in aliases:
                aliases[name] = alias
    return funcs, aliases


# Ruling R7: a function whose own body calls a drawing builtin (draw/gpu/surface/shader/texture,
# plus vertex-buffer building and freeing) belongs to a render task.
DRAW_RE = re.compile(r"\bgm\.((?:draw|gpu|surface|shader|texture|vertex)_[A-Za-z0-9_]*)\s*\(")
# pure size/existence/handle queries do not make a function a drawing one
DRAW_QUERY = {"surface_exists", "surface_get_width", "surface_get_height", "texture_get_width",
              "texture_get_height", "texture_get_texel_width", "texture_get_texel_height",
              "shader_is_compiled", "shader_get_uniform", "shader_get_sampler_index"}
# mixed functions that stay with their logic task (the render task ports their GPU part unmarked):
# load_game frees the old mycelium vertex buffers; the controller Create defines all mycelium state
# and also builds the vertex format
DRAW_KEEP = {"load_game", "gml_Object_obj_mycelium_controller_Create_0"}
# logic task -> render task that takes its drawing rows
DRAW_TASK = {"T3": "T17", "T5": "T13", "T6": "T13", "T7": "T13", "T8": "T14", "T9": "T15",
             "T10": "T15", "T11": "T17", "T12": "T16"}


def draw_calls(body):
    """Drawing builtins called in a Lua function body (size/existence queries excluded)."""
    return sorted({c for c in DRAW_RE.findall(body) if c not in DRAW_QUERY})


def draw_reassign(task, body, name=""):
    if name in DRAW_KEEP or not draw_calls(body):
        return task
    return DRAW_TASK.get(task, task)


# Hand decisions for rows no table rule covers (Task 1 step 6, rulings R5/R8).
MANUAL = {}
for _task, _names in {
    "T3": "balance_defaults balance_init worldgen_defaults worldgen_init "
          "game_mode_apply game_mode_start visual_defaults visual_init ui_font ui_font_small "
          "gml_Object_obj_game_Create_0 gml_Object_obj_game_Other_3",
    "T4": "display_init display_fit display_apply focus_step art_preload_step display_step "
          "display_window_size display_save display_set_fullscreen display_cycle_size",
    "T15": "post_layer_draw surface_fit bw_shader_set gml_Object_obj_game_Draw_74 gml_Object_obj_game_Draw_75 "
           "gml_Object_obj_game_Draw_77 gml_Object_obj_life_Draw_0",
    "T17": "ui_blocking animal_marker_list animal_markers_update animal_markers_draw animal_marker_at "
           "gml_Object_obj_hud_Create_0 gml_Object_obj_hud_Step_1 gml_Object_obj_hud_Draw_75",
    "T10": "squirrel_grab gml_Object_obj_life_Create_0 gml_Object_obj_life_Step_0 "
           "game_init game_step gml_Object_obj_game_Step_0",
    "T8": "anon__49789__anon__49515__gml_Object_obj_mycelium_controller_Create_0 "
          "anon__86877__anon__86458__gml_Object_obj_mycelium_controller_Create_0 "
          "gml_Object_obj_mycelium_controller_Create_0 gml_Object_obj_mycelium_controller_Step_0 "
          "gml_Object_obj_mycelium_controller_CleanUp_0",
    "T14": "gml_Object_obj_mycelium_controller_Draw_0 gml_Object_obj_mycelium_controller_Draw_64",
}.items():
    for _n in _names.split():
        MANUAL[_n] = _task


def starts(name, *prefixes):
    return any(name.startswith(p) for p in prefixes)


def classify(name, alias, chunk):
    """First matching rule wins. Mirrors the table in the Task 1 brief, with rulings R3/R5."""
    a = alias or ""
    # GameMaker / Steam wrappers and script registration bodies (only gm.method assignments)
    if starts(name, "Steam", "ExtensionCore", "steam_"):
        return "skip:steam"
    if starts(name, "ach_"):
        return "skip:achievements"
    if starts(name, "gml_GlobalScript_"):
        return "skip:script-registration"
    if starts(name, "gml_Object_obj_gm_"):
        return "skip:unused-gm-template"
    # aliases of anonymous controller methods
    if a.startswith("hud_"):
        return "T17"
    if a.startswith("forest_"):
        return "T16"
    if a.startswith("myc_"):
        return "T8"
    # balance / locale / saves (T3), plus resource accounting per ruling R3
    if name in ("bal", "vis", "wld", "game_mode", "perk_cost_mult", "water_total") or starts(
            name, "loc", "save", "load", "experience_price"):
        return "T3"
    # world rendering
    if starts(name, "world_draw", "food_glow"):
        return "T13"
    if name.startswith("food"):
        return "T3"
    # world_paint* write chunk tiles during generation (no draw calls): owned by the generator task
    if name.startswith("world_paint"):
        if "bone" in name:
            return "T6"
        if chunk.startswith("scr_stones") or "stone" in name or "cluster" in name or "maze" in name:
            return "T7"
        return "T5"
    if starts(name, "sound", "sfx", "audio", "music", "ambience", "hyphae_sound", "rain_sound",
              "forest_sound", "water_touch") or name == "mushroom_grow_sound":
        return "T20"
    if starts(name, "tasks_popup", "tasks_block", "hover_bar"):
        return "T17"
    if starts(name, "backdrop", "leaf_fg"):
        return "T16"
    if name.startswith("tree_") and chunk == "ui_1":
        return "T19"
    if name.startswith("logo"):
        return "T18"
    if starts(name, "cine", "intro", "camera_"):
        return "T21"
    if starts(name, "weather", "rain"):
        return "T12"
    if starts(name, "upg", "upgrades", "tasks", "goal", "tutorial", "cheat", "balance"):
        return "T11"
    if starts(name, "myc_", "mycelium_", "length", "reach", "pixels_to"):
        return "T8"
    if starts(name, "mushroom", "shroom", "repeller", "stinky", "consum", "tree_shroom", "experience"):
        return "T9"
    if name.startswith("worldgen_") or name.startswith("world_food"):
        return "T5"
    if chunk.startswith("scr_world_b") and starts(name, "world_", "lake", "water_"):
        return "T5"
    if starts(name, "cave", "relic", "perk", "bone", "caves_"):
        return "T6"
    if chunk.startswith("scr_world_a"):
        if name == "world_init" or name.startswith("world_start_"):
            return "T5"
        return "T6"
    if chunk.startswith("scr_stones"):
        return "T7"
    if chunk.startswith("scr_life") and not starts(name, "intro", "backdrop", "camera_fly", "animal_marker"):
        return "T10"
    return "T?"


def assign(name, alias, chunk, body):
    """Final task of a row: hand decision, else table rule; then the R7 drawing rule."""
    task = MANUAL.get(name) or classify(name, alias, chunk)
    if task == "T?":
        if "world_clear_ocean" in name:
            task = "T5"
        elif "obj_forest_controller" in name:
            task = "T16"
    return draw_reassign(task, body, name)


def main():
    funcs, aliases = collect()
    unresolved = []
    for name, chunk, body in funcs:
        alias = aliases.get(name)
        task = assign(name, alias, chunk, body)
        print(f"{name}\t{alias or '-'}\t{task}")
        if task == "T?":
            unresolved.append(f"{chunk}: {name} ({alias or '-'})")
    if unresolved:
        print(f"{len(unresolved)} rows need a task (T?):", file=sys.stderr)
        for row in unresolved:
            print("  " + row, file=sys.stderr)


if __name__ == "__main__":
    main()
