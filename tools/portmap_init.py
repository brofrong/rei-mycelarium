#!/usr/bin/env python3
"""Initial fill of rust/portmap.tsv: every translated game function -> owning task.

Prints `name<TAB>alias<TAB>task` for each `^function NAME` in game/myco/gen/*.lua (except the
`___struct___*` data shapes). `task` is `T<N>`, `skip:<reason>` or `T?` when no rule applies; the
`T?` rows are resolved by hand afterwards (they are also listed on stderr).

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
    """Return [(name, chunk_file_stem)] in file order, and {name: alias}."""
    funcs, aliases = [], {}
    for path in sorted(gen.glob("*.lua")):
        text = path.read_text()
        for name in FUNC_RE.findall(text):
            if not name.startswith("___struct___"):
                funcs.append((name, path.stem))
        for alias, name in ALIAS_RE.findall(text):
            if alias != name and name not in aliases:
                aliases[name] = alias
    return funcs, aliases


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


def main():
    funcs, aliases = collect()
    unresolved = []
    for name, chunk in funcs:
        alias = aliases.get(name)
        task = classify(name, alias, chunk)
        print(f"{name}\t{alias or '-'}\t{task}")
        if task == "T?":
            unresolved.append(f"{chunk}: {name} ({alias or '-'})")
    if unresolved:
        print(f"{len(unresolved)} rows need a task (T?):", file=sys.stderr)
        for row in unresolved:
            print("  " + row, file=sys.stderr)


if __name__ == "__main__":
    main()
