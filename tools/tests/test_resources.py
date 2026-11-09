import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1]


def test_rooms(assets):
    r = assets["rooms"]
    assert {k: (v["w"], v["h"]) for k, v in r.items()} == {"Room1": (35040, 3744), "Main_menu": (1920, 1080), "UI": (1920, 1080)}
    assert any(i["object"] == "obj_game" for room in r.values() for i in room["instances"])


def test_room_order(assets):
    assert assets["game"]["room_order"] == ["Room1", "Main_menu", "UI"]
    assert assets["game"]["speed"] == 60


def test_room1_layers_and_view(assets):
    room = assets["rooms"]["Room1"]
    assert [(l["name"], l["type"], l["depth"]) for l in room["layers"]] == [("Instances", "instances", 0), ("ForestScene", "instances", 100), ("Background", "background", 200)]
    assert room["views_enabled"] and room["views"][0]["view"] == [0, 0, 1920, 1080]


def test_objects(assets):
    assert sorted(assets["objects"]) == ["obj_forest_controller", "obj_game", "obj_gm_button", "obj_gm_button1", "obj_gm_text",
                                         "obj_gm_text1", "obj_gm_textbox", "obj_hud", "obj_life", "obj_mycelium_controller"]
    assert [3, 1] in assets["objects"]["obj_hud"]["events"]  # Step(3), Begin(1) -> gml_Object_obj_hud_Step_1
    assert assets["objects"]["obj_game"]["parent"] is None


def test_sprites(assets):
    s = assets["sprites"]
    assert len(s) == 167 and "apple" in s and "background_B_layer1" in s
    assert s["apple"]["bbox"] == [3, 2, 60, 63] and s["apple"]["speed"] == 30
    for spr in s.values():
        for fr in spr["frames"]:
            assert 0 <= fr["page"] < 5 and fr["src"][2] > 0


def test_sounds_shaders_fonts(assets, out_dir):
    assert len(assets["sounds"]) == 24
    assert all((out_dir / "sounds" / v["file"]).exists() for v in assets["sounds"].values())
    assert assets["sounds"]["hang_guitar_01"]["streamed"] is True
    assert sorted(p.stem for p in (out_dir / "shaders").glob("*.fsh")) == ["sh_blackmetal", "sh_glow_blur", "sh_stinky"]
    assert sorted(assets["fonts"]) == ["Font1", "fnt_gm_15", "fnt_gm_20", "fnt_gm_30"]
    f = assets["fonts"]["fnt_gm_15"]
    assert f["size"] == 15 and f["glyphs"]["33"] == {"x": 2, "y": 83, "w": 3, "h": 25, "shift": 6, "offset": 1}


def test_pages_and_externals(out_dir):
    assert sorted(p.name for p in (out_dir / "pages").glob("*.png")) == [f"page{i}.png" for i in range(5)]
    for name in ["localization.csv", "basic_items.json", "splash.png", "fonts/cyrillic-ger-spa-port.ttf"]:
        assert (out_dir / name).exists()


def test_cli_missing_game(tmp_path):
    r = subprocess.run([sys.executable, "-I", str(TOOLS / "extract.py"), "--game-dir", str(tmp_path), "--out", str(tmp_path / "o")],
                       capture_output=True, text=True)
    assert r.returncode == 2 and "game not found" in r.stderr
