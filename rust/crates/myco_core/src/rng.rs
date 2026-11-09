//! GML random numbers on xorshift64*, bit-identical to the LÖVE port's `game/gm/random.lua`
//! (LuaJIT `uint64_t` arithmetic = `u64` wrapping here).

const MUL_SEED: u64 = 6364136223846793005;
const ADD_SEED: u64 = 1442695040888963407;
const ZERO_STATE: u64 = 88172645463325252;
const MUL_OUT: u64 = 2685821657736338717;
const TWO_POW_53: f64 = 9007199254740992.0;

/// `array_shuffle` (GML; the LÖVE shim `game/gm/value.lua` draws from the game RNG the same
/// way): Fisher–Yates from the end with `j = floor(random(1) * (i + 1))`.
pub fn array_shuffle<T>(rng: &mut Rng, v: &mut [T]) {
    let n = v.len();
    for i in (1..n).rev() {
        let j = (rng.random(1.0) * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rng {
    state: u64,
    seed: i64,
}

impl Default for Rng {
    /// The generator state at load time (`random.lua` ends with `random_set_seed(0)`).
    fn default() -> Self {
        Rng::new(0)
    }
}

impl Rng {
    /// `random_set_seed(seed)`. Negative seeds wrap to `u64` like LuaJIT's `ffi.cast("uint64_t", n)`.
    pub fn new(seed: i64) -> Self {
        let mut state = (seed as u64).wrapping_mul(MUL_SEED).wrapping_add(ADD_SEED);
        if state == 0 {
            state = ZERO_STATE;
        }
        Rng { state, seed }
    }

    /// `random_get_seed()`.
    pub fn seed(&self) -> i64 {
        self.seed
    }

    /// Uniform in [0, 1) with 53 random bits.
    pub fn next01(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        (x.wrapping_mul(MUL_OUT) >> 11) as f64 / TWO_POW_53
    }

    pub fn random(&mut self, n: f64) -> f64 {
        self.next01() * n
    }

    pub fn random_range(&mut self, a: f64, b: f64) -> f64 {
        a + self.next01() * (b - a)
    }

    /// Integer in [0, floor(n)].
    pub fn irandom(&mut self, n: f64) -> i64 {
        (self.next01() * (n.floor() + 1.0)).floor() as i64
    }

    /// Integer in [floor(min(a, b)), floor(max(a, b))].
    pub fn irandom_range(&mut self, a: f64, b: f64) -> i64 {
        let lo = a.min(b).floor();
        let hi = a.max(b).floor();
        (lo + (self.next01() * (hi - lo + 1.0)).floor()) as i64
    }

    /// `choose(...)`; panics on an empty slice (GML would return the argument count error).
    pub fn choose<T: Copy>(&mut self, items: &[T]) -> T {
        assert!(!items.is_empty(), "choose: no items");
        let k = (self.next01() * items.len() as f64).floor() as usize;
        items[k]
    }

    /// `randomise()`: reseed from the clock, returns the new seed (in [0, 2147483647)).
    pub fn randomise(&mut self) -> i64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        // os.time() * 1000 + os.clock() * 1e6 in the Lua original; any clock-derived value will do
        let v = now.as_secs() as f64 * 1000.0 + now.subsec_micros() as f64;
        let seed = (v - (v / 2147483647.0).floor() * 2147483647.0).floor() as i64;
        *self = Rng::new(seed);
        seed
    }
}
