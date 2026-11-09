local text = require("gm.text_layout")
local shader_conv = require("gm.shader_conv")
local ini = require("gm.ini")
local buffer = require("gm.buffer")

local function mono(s) return #s * 10 end -- 10 px per byte

return {
  wrap_on_spaces = function()
    local lines = text.wrap("aaa bbb ccc", 75, mono)
    assert(#lines == 2 and lines[1] == "aaa bbb" and lines[2] == "ccc")
  end,
  wrap_keeps_newlines_and_long_words = function()
    local lines = text.wrap("x\nabcdefghij", 50, mono)
    assert(#lines == 2 and lines[1] == "x" and lines[2] == "abcdefghij")
  end,
  wrap_negative_width_means_no_wrap = function()
    local lines = text.wrap("aaa bbb", -1, mono)
    assert(#lines == 1)
  end,
  align_offsets = function()
    assert(text.align_x(1, 100) == -50 and text.align_x(2, 100) == -100 and text.align_x(0, 100) == 0)
    assert(text.align_y(1, 40) == -20 and text.align_y(2, 40) == -40)
  end,
  shader_conversion = function()
    local fsh = "varying vec2 v_vTexcoord;\nvarying vec4 v_vColour;\nuniform float u_t;\nvoid main()\n{\n    gl_FragColor = v_vColour * texture2D( gm_BaseTexture, v_vTexcoord );\n}\n"
    local out = shader_conv.fragment(fsh)
    assert(out:find("void effect%(%)") and out:find("love_PixelColor =") and out:find("uniform float u_t;"))
    assert(not out:find("varying vec2 v_vTexcoord;", 1, true) and not out:find("gl_FragColor", 1, true))
  end,
  shader_precision_removed = function()
    local out = shader_conv.fragment("precision mediump float;\n#define LOWPREC lowp\nvoid main()\n{\n gl_FragColor = vec4(1.0);\n}\n")
    assert(not out:find("precision mediump", 1, true) and not out:find("#define LOWPREC lowp", 1, true))
    assert(out:find("uniform Image MainTex;", 1, true))
  end,
  ini_roundtrip = function()
    local d = ini.parse('[display]\r\nfullscreen="1.000000"\r\n[audio]\r\nmusic="0.5"\r\n')
    assert(ini.read(d, "display", "fullscreen", "x") == "1.000000" and ini.read(d, "audio", "nope", "def") == "def")
    ini.write(d, "audio", "sfx", "0.25")
    local d2 = ini.parse(ini.serialize(d))
    assert(ini.read(d2, "audio", "sfx", "") == "0.25" and ini.read(d2, "audio", "music", "") == "0.5")
  end,
  buffer_rw = function()
    local b = buffer.create(4, buffer.grow, 1)
    buffer.write(b, buffer.u8, 200)
    buffer.write(b, buffer.s32, -5)
    buffer.write(b, buffer.f64, 2.5)
    buffer.write(b, buffer.string, "Жук")
    assert(buffer.get_size(b) >= 1 + 4 + 8 + 7)
    buffer.seek(b, buffer.seek_start, 0)
    assert(buffer.read(b, buffer.u8) == 200 and buffer.read(b, buffer.s32) == -5 and buffer.read(b, buffer.f64) == 2.5)
    assert(buffer.read(b, buffer.string) == "Жук")
    assert(buffer.peek(b, 0, buffer.u8) == 200)
    buffer.poke(b, 0, buffer.u8, 7)
    assert(buffer.peek(b, 0, buffer.u8) == 7)
  end,
}
