import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from lift import lift  # noqa: E402

HEAD = '''void gml_Script_f(undefined8 param_1,undefined8 param_2)
{
  undefined8 local_68;
  local_30 = "gml_Script_f";
  local_38 = g_pCurrentStackTrace;
  g_pCurrentStackTrace = &local_38;
  local_28 = 0x2f;
'''


def test_variable_refs():
    out = lift(HEAD + "  p = (**(code **)(*g_pGlobal + 8))(g_pGlobal,vid_volume_music);\n"
               "  q = (**(code **)(*param_1 + 0x10))(param_1,vid_hp);\n"
               "  r = (**(code **)(*param_2 + 8))(param_2,vid_x);\n}\n")
    assert "p = &global.volume_music;" in out and "q = &self.hp;" in out and "r = &other.x;" in out


def test_builtin_call_and_line_markers():
    out = lift(HEAD + "  local_28 = 0x30;\n  uVar8 = CONCAT44(uVar14,vid_window_get_width);\n"
               "  ppuVar7 = (undefined8 **)YYGML_CallBuiltin(param_1,param_2,&local_1f0,0,uVar8,0);\n"
               "  uVar14 = (undefined4)((ulonglong)uVar8 >> 0x20);\n"
               "  YYGML_CallBuiltin(param_1,param_2,&local_1f0,2,CONCAT44(uVar14,vid_display_set_gui_size),&local_178);\n}\n")
    assert "// L48" in out and "// L47" in out
    assert "display_set_gui_size!(2, &local_178)" in out
    assert "ppuVar7 = (undefined8 **)window_get_width!(0, 0);" in out and "uVar8 = vid_window_get_width;" not in out
    assert "uVar14 = (undefined4)" not in out and "g_pCurrentStackTrace" not in out and "undefined8 local_68;" not in out


def test_rvalue_noise_removed():
    out = lift(HEAD + "  if ((1 << ((byte)local_5c & 0x1f) & 0x46U) != 0) {\n    FREE_RValue(&local_68);\n  }\n"
               "  if ((1 << ((byte)*(undefined4 *)((longlong)puVar4 + 0xc) & 0x1f) & 0x46U) == 0) {\n    local_78 = *puVar4;\n  }\n"
               "  else {\n    COPY_RValue_Post(&local_78,puVar4);\n  }\n  uStack_1fc = 0;\n  COPY_RValue(&local_118,&local_208);\n}\n")
    assert "FREE_RValue" not in out and "0x46U" not in out and "uStack_1fc" not in out
    assert "local_78 = *puVar4;" in out and "local_118 = local_208;" in out


def test_double_constants_and_operators():
    out = lift(HEAD + "  local_158 = (undefined8 *)0x4090e00000000000;\n  FUN_14002eee0(&local_208,&local_88);\n"
               "  iVar2 = YYCompareVal(&local_118,&local_208,DAT_140f4d210,1);\n}\n")
    assert "local_158 = 1080.0;" in out and "OP_MUL(&local_208,&local_88)" in out and "CMP(&local_118,&local_208)" in out


def test_constants_and_assets_inlined():
    consts = {"0x14137b080": "hedgehog_scale", "0x140fd78a0": 1.0, "0x140fd77c0": False}
    assets = {1: ["apple", "boar"], 2: ["bird"]}
    out = lift(HEAD + "  local_a8 = _DAT_14137b080;\n  local_c8 = (undefined1 *)CONCAT44(uRam0000000140fd78a4,_DAT_140fd78a0);\n"
               "  x = DAT_140fd77c0;\n  local_b8 = 0x100000100000001;\n  s = 0x100000200000000;\n}\n", consts, assets)
    assert 'local_a8 = "hedgehog_scale";' in out and "local_c8 = (undefined1 *)1.0;" in out and "x = false;" in out
    assert "local_b8 = sprite(boar);" in out and "s = sound(bird);" in out
