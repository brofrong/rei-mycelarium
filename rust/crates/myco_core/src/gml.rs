//! GML math semantics, exactly as the LÖVE port's `game/gm/mathx.lua` and `gm.mod`
//! (`game/gm/value.lua`). Angles are degrees with y pointing down, like GameMaker.

const DEG2RAD: f64 = std::f64::consts::PI / 180.0; // LuaJIT math.rad
const RAD2DEG: f64 = 180.0 / std::f64::consts::PI; // LuaJIT math.deg

/// Lua's `%` on numbers: floored modulo, `a - floor(a / b) * b` (result has the divisor's sign).
#[inline]
fn lua_mod(a: f64, b: f64) -> f64 {
    a - (a / b).floor() * b
}

/// GameMaker `round`: halves go to the nearest even integer.
pub fn round(x: f64) -> f64 {
    let f = x.floor();
    let d = x - f;
    // above the half, or exactly the half with an odd floor -> up
    if d > 0.5 || (d == 0.5 && lua_mod(f, 2.0) != 0.0) {
        f + 1.0
    } else {
        f
    }
}

pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// `math.max(lo, math.min(hi, x))`.
pub fn clamp(x: f64, lo: f64, hi: f64) -> f64 {
    let m = if x < hi { x } else { hi };
    if m > lo {
        m
    } else {
        lo
    }
}

/// `gm.power(a, b)` = Lua's `a ^ b`: the C library `pow`, called for every exponent. LLVM
/// would turn `pow(x, 2.0)` into `x * x`, but the macOS libm `pow` is not correctly rounded
/// (e.g. `pow(19.07197..., 2)` is one ulp above the product), so the exponent is hidden from
/// the optimizer.
pub fn power(a: f64, b: f64) -> f64 {
    a.powf(std::hint::black_box(b))
}

/// `math.sqrt((x2 - x1) ^ 2 + (y2 - y1) ^ 2)`, squares through `pow` like the original.
pub fn point_distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (power(x2 - x1, 2.0) + power(y2 - y1, 2.0)).sqrt()
}

/// Degrees in [0, 360), counter-clockwise on screen (y down).
pub fn point_direction(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    lua_mod((y1 - y2).atan2(x2 - x1) * RAD2DEG, 360.0)
}

pub fn lengthdir_x(len: f64, dir: f64) -> f64 {
    len * (dir * DEG2RAD).cos()
}

pub fn lengthdir_y(len: f64, dir: f64) -> f64 {
    -len * (dir * DEG2RAD).sin()
}

/// Signed difference a - b wrapped to [-180, 180).
pub fn angle_difference(a: f64, b: f64) -> f64 {
    lua_mod(a - b + 180.0, 360.0) - 180.0
}

/// GML `mod` / `gm.mod`: C `fmod`, the result has the dividend's sign.
pub fn gmod(a: f64, b: f64) -> f64 {
    a % b
}

/// Fractional part, toward zero: `x - trunc(x)`.
pub fn frac(x: f64) -> f64 {
    x - if x >= 0.0 { x.floor() } else { x.ceil() }
}

pub fn sign(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else if x < 0.0 {
        -1.0
    } else {
        0.0
    }
}

pub fn dsin(deg: f64) -> f64 {
    (deg * DEG2RAD).sin()
}

pub fn dcos(deg: f64) -> f64 {
    (deg * DEG2RAD).cos()
}

pub fn darctan2(y: f64, x: f64) -> f64 {
    y.atan2(x) * RAD2DEG
}

pub fn degtorad(deg: f64) -> f64 {
    deg * DEG2RAD
}

pub fn radtodeg(rad: f64) -> f64 {
    rad * RAD2DEG
}
