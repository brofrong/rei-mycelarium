-- GML math builtins (no love dependency).
local M = { infinity = math.huge, NaN = 0 / 0, pi = math.pi }

-- GameMaker's round() rounds halves to the nearest even integer.
function M.round(x)
  local f = math.floor(x)
  local d = x - f
  if d > 0.5 then return f + 1 end
  if d < 0.5 then return f end
  return f % 2 == 0 and f or f + 1
end

function M.floor(x) return math.floor(x) end
function M.ceil(x) return math.ceil(x) end
function M.frac(x) return x - (x >= 0 and math.floor(x) or math.ceil(x)) end
function M.abs(x) return math.abs(x) end
function M.sign(x) return x > 0 and 1 or x < 0 and -1 or 0 end
function M.sqrt(x) return math.sqrt(x) end
function M.sqr(x) return x * x end
function M.power(a, b) return a ^ b end
function M.exp(x) return math.exp(x) end
function M.ln(x) return math.log(x) end
function M.log2(x) return math.log(x) / math.log(2) end
function M.log10(x) return math.log10(x) end
function M.min(...) return math.min(...) end
function M.max(...) return math.max(...) end
function M.clamp(x, a, b) return math.max(a, math.min(b, x)) end
function M.lerp(a, b, t) return a + (b - a) * t end
function M.sin(r) return math.sin(r) end
function M.cos(r) return math.cos(r) end
function M.tan(r) return math.tan(r) end
function M.dsin(d) return math.sin(math.rad(d)) end
function M.dcos(d) return math.cos(math.rad(d)) end
function M.dtan(d) return math.tan(math.rad(d)) end
function M.darcsin(x) return math.deg(math.asin(x)) end
function M.darccos(x) return math.deg(math.acos(x)) end
function M.darctan2(y, x) return math.deg(math.atan2(y, x)) end
function M.arctan2(y, x) return math.atan2(y, x) end
function M.arctan(x) return math.atan(x) end
function M.degtorad(d) return math.rad(d) end
function M.radtodeg(r) return math.deg(r) end
function M.point_distance(x1, y1, x2, y2) return math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2) end
function M.point_direction(x1, y1, x2, y2) return (math.deg(math.atan2(y1 - y2, x2 - x1))) % 360 end
function M.lengthdir_x(len, dir) return len * math.cos(math.rad(dir)) end
function M.lengthdir_y(len, dir) return -len * math.sin(math.rad(dir)) end
function M.point_in_rectangle(px, py, x1, y1, x2, y2) return px >= x1 and px <= x2 and py >= y1 and py <= y2 end
function M.angle_difference(a, b) return ((a - b + 180) % 360) - 180 end
function M.is_nan(x) return x ~= x end
function M.is_infinity(x) return x == math.huge or x == -math.huge end

return M
