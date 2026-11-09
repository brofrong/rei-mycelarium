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
  array_concat_contains_filter = function()
    local a = gm.array_concat(gm.array(1, 2), gm.array(3))
    assert(gm.array_length(a) == 3 and a[2] == 3 and gm.array_contains(a, 2) and not gm.array_contains(a, 9))
    local f = gm.array_filter(a, function(_, _, v) return v ~= 2 end)
    assert(gm.array_length(f) == 2 and f[1] == 3)
  end,
  variable_clone_deep = function()
    local s = gm.struct({ list = gm.array(1, gm.struct({ k = 1 })) })
    local c = gm.variable_clone(s)
    c.list[1].k = 5
    assert(s.list[1].k == 1 and c.list[1].k == 5 and gm.is_struct(c) and gm.is_array(c.list))
  end,
  new_runs_constructor_on_struct = function()
    local function Point(self, other, x, y) self.x, self.y = x, y end
    local p = gm.new(Point, 2, 3)
    assert(gm.is_struct(p) and p.x == 2 and p.y == 3 and gm.instanceof(p) == Point)
  end,
  method_call_matches_function_call_convention = function()
    local s, caller = gm.struct({ k = 10 }), gm.struct({})
    local m = gm.method(s, function(self, other, a, b) return self.k + a * b, other end)
    local r, o = m(caller, nil, 2, 3) -- called like f(self, other, a, b)
    assert(r == 16 and o == caller)
  end,
  fractional_index_floors = function()
    local a = gm.array(10, 20, 30)
    assert(a[1.7] == 20)
    a[2.2] = 99
    assert(a[2] == 99 and gm.array_length(a) == 3)
  end,
}

