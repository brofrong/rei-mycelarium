local gm = require("gm.core")

return {
  zero_based = function()
    local a = gm.array(10, 20, 30)
    assert(a[0] == 10 and a[2] == 30 and gm.array_length(a) == 3)
  end,
  array_auto_extends = function()
    local a = gm.array()
    a[3] = 7
    assert(gm.array_length(a) == 4 and a[0] == 0 and a[3] == 7)
  end,
  array_push_delete_insert = function()
    local a = gm.array(1, 2, 3)
    gm.array_push(a, 4)
    gm.array_delete(a, 0, 1)
    gm.array_insert(a, 1, 9)
    assert(gm.array_length(a) == 4 and a[0] == 2 and a[1] == 9 and a[2] == 3 and a[3] == 4)
  end,
  array_create_fill = function()
    local a = gm.array_create(3, 5)
    assert(gm.array_length(a) == 3 and a[2] == 5 and gm.is_array(a))
  end,
  utf8_strings = function()
    assert(gm.string_length("Мицелий") == 7 and gm.string_char_at("Мицелий", 1) == "М")
    assert(gm.string_copy("Мицелий", 2, 3) == "иц" .. "е" and gm.string_pos("ли", "Мицелий") == 5)
  end,
  string_conversions = function()
    assert(gm.string(3) == "3" and gm.string(2.5) == "2.50" and gm.string(true) == "1")
    assert(gm.real("12.5") == 12.5)
  end,
  undefined_kept = function()
    local s = gm.struct({})
    s.x = gm.undefined
    assert(gm.variable_struct_exists(s, "x") and gm.is_undefined(s.x))
  end,
  method_binds_self = function()
    local s = gm.struct({ v = 5 })
    local m = gm.method(s, function(self) return self.v end)
    assert(m() == 5)
  end,
  method_other_is_caller = function()
    local s, caller = gm.struct({}), gm.struct({ tag = "caller" })
    local m = gm.method(s, function(self, other) return other end)
    assert(m(caller) == caller)
  end,
  typeof_values = function()
    assert(gm.typeof(1) == "number" and gm.typeof("a") == "string" and gm.typeof(gm.array()) == "array")
    assert(gm.typeof(gm.struct({})) == "struct" and gm.typeof(gm.undefined) == "undefined" and gm.typeof(true) == "bool")
    assert(gm.typeof(gm.method(nil, function() end)) == "method")
  end,
  div_mod = function()
    assert(gm.div(-7, 2) == -3 and gm.mod(-7, 2) == -1 and gm.mod(7.5, 2) == 1.5)
  end,
}
