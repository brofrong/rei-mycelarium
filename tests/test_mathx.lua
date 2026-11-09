local m = require("gm.mathx")

return {
  round_half_even = function()
    assert(m.round(2.5) == 2 and m.round(3.5) == 4 and m.round(-2.5) == -2 and m.round(1.4) == 1 and m.round(1.6) == 2)
  end,
  angle_difference = function()
    assert(m.angle_difference(10, 350) == 20 and m.angle_difference(350, 10) == -20 and m.angle_difference(90, 90) == 0)
  end,
  degrees_trig = function()
    assert(math.abs(m.dsin(90) - 1) < 1e-12 and math.abs(m.dcos(180) + 1) < 1e-12 and math.abs(m.dtan(45) - 1) < 1e-12)
  end,
  misc = function()
    assert(m.point_distance(0, 0, 3, 4) == 5 and m.lerp(10, 20, 0.25) == 12.5 and m.frac(-2.75) == -0.75)
    assert(m.point_in_rectangle(5, 5, 0, 0, 10, 10) and not m.point_in_rectangle(11, 5, 0, 0, 10, 10))
    assert(m.sin(0) == 0 and m.cos(0) == 1 and m.infinity == math.huge and m.clamp(5, 0, 3) == 3 and m.sign(-2) == -1 and m.power(2, 10) == 1024)
  end,
}
