local json = require("gm.json")
local gm = require("gm.core")

return {
  decode_plain = function()
    local t = json.decode('{"a": [1, 2.5, "x\\n\\u0416"], "b": true, "c": null}')
    assert(t.a[1] == 1 and t.a[2] == 2.5 and t.a[3] == "x\nЖ" and t.b == true and t.c == nil)
  end,
  decode_gml = function()
    local t = json.decode('{"list": [10, {"k": "v"}], "n": -3e2}', json.gml)
    assert(gm.is_struct(t) and gm.is_array(t.list) and t.list[0] == 10 and t.list[1].k == "v" and t.n == -300)
  end,
  encode_gml_roundtrip = function()
    local s = gm.struct({ name = "Мицелий", vals = gm.array(1, 2.5, true), inner = gm.struct({ u = gm.undefined }) })
    local back = json.decode(json.encode(s), json.gml)
    assert(back.name == "Мицелий" and back.vals[1] == 2.5 and back.vals[2] == true and gm.array_length(back.vals) == 3)
  end,
  encode_numbers = function()
    assert(json.encode(gm.array(3, 0.5, -2)) == "[3,0.5,-2]")
  end,
}
