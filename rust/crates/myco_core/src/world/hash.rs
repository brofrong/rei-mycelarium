//! World hashes, bit-exact with the LÖVE port: every generated chunk, lake, food and many
//! animations derive from them, so they are what makes worlds (and saves) compatible.

/// LuaJIT `bit.tobit` on an integral double: the value modulo 2^32 as `i32`.
#[inline]
fn tobit(x: f64) -> i32 {
    (x as i64) as i32
}

/// `to_i64` of the Lua port: truncation toward zero.
#[inline]
fn trunc(x: f64) -> f64 {
    x.trunc()
}

// port: world_hash01
/// `world_hash01(a, b, c)` with `global.world.seed` passed in: a value in [0, 1) (negative
/// intermediate values follow LuaJIT's 32-bit `bit` semantics exactly).
pub fn world_hash01(seed: i64, a: f64, b: f64, c: f64) -> f64 {
    let ia = trunc(a);
    let ib = trunc(b);
    let mut h = ((tobit(ia) >> 16) & 0xffff) as f64 * 0x5215 as f64
        + (tobit(ia) & 0xffff) as f64 * 0x9e37 as f64
        + (tobit(ib) & 0xffff) as f64 * 0x9069 as f64
        + c * 12983.0
        + seed as f64;
    h %= 65536.0; // gm.mod = fmod
    h = trunc(h);
    let x = tobit(h);
    h = (x ^ (x >> 8)) as f64;
    h = (h * 0x9e37 as f64) % 65536.0;
    let x = tobit(h);
    h = (x ^ (x >> 7)) as f64;
    h = (h * 0x9069 as f64) % 65536.0;
    let x = tobit(h);
    h = (x ^ (x >> 9)) as f64;
    h / 65536.0
}

// port: cave_hash
/// `cave_hash(a, b, c)`: the classic sine hash, fractional part in [0, 1).
pub fn cave_hash(a: f64, b: f64, c: f64) -> f64 {
    let v = (a * 12.9898 + b * 78.233 + c * 37.719).sin() * 43758.5453;
    v - v.floor()
}
