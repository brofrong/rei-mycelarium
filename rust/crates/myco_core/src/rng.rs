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

/// `MYCO_RNG_SEED`'s value as a seed (`None`: unset or not an integer).
pub fn fixed_seed_from(v: Option<&str>) -> Option<i64> {
    v?.trim().parse().ok()
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
        #[cfg(feature = "trace-draws")]
        draw_trace::reset();
        Rng { state, seed }
    }

    /// The xorshift word (conformance traces compare it with the LÖVE port's).
    pub fn state(&self) -> u64 {
        self.state
    }

    /// `random_get_seed()`.
    pub fn seed(&self) -> i64 {
        self.seed
    }

    /// Uniform in [0, 1) with 53 random bits.
    pub fn next01(&mut self) -> f64 {
        #[cfg(feature = "trace-draws")]
        draw_trace::draw();
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
    /// `MYCO_RNG_SEED=<int>` (conformance runs; the LÖVE port reads the same variable) fixes it.
    pub fn randomise(&mut self) -> i64 {
        let fixed = fixed_seed_from(std::env::var("MYCO_RNG_SEED").ok().as_deref());
        self.randomise_with(fixed)
    }

    /// [`Rng::randomise`] with the fixed seed given: `Some(s)` reseeds like `Rng::new(s)`.
    pub fn randomise_with(&mut self, fixed: Option<i64>) -> i64 {
        if let Some(seed) = fixed {
            *self = Rng::new(seed);
            return seed;
        }
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

/// Feature `trace-draws` (conformance debugging, `examples/rng_trace.rs`): print the callers of
/// draws `from..=to` counted from the last reseed, like the LÖVE port's MYCO_TRACE_DRAWS.
#[cfg(feature = "trace-draws")]
pub mod draw_trace {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    static FROM: AtomicU64 = AtomicU64::new(u64::MAX);
    static TO: AtomicU64 = AtomicU64::new(0);
    static PAUSED: AtomicBool = AtomicBool::new(false);

    pub fn set_range(from: u64, to: u64) {
        FROM.store(from, Ordering::Relaxed);
        TO.store(to, Ordering::Relaxed);
    }
    /// Draws of a generator other than the game's are not counted while paused.
    pub fn pause(p: bool) {
        PAUSED.store(p, Ordering::Relaxed);
    }
    pub(super) fn reset() {
        if !PAUSED.load(Ordering::Relaxed) {
            N.store(0, Ordering::Relaxed);
        }
    }
    pub(super) fn draw() {
        if PAUSED.load(Ordering::Relaxed) {
            return;
        }
        let n = N.fetch_add(1, Ordering::Relaxed);
        if n < FROM.load(Ordering::Relaxed) || n > TO.load(Ordering::Relaxed) {
            return;
        }
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let mut names = Vec::new();
        let mut lines = bt.lines().peekable();
        while let Some(l) = lines.next() {
            let l = l.trim();
            if let Some(pos) = l.find("myco_core::") {
                let name = &l[pos + "myco_core::".len()..];
                if name.starts_with("rng::") {
                    continue;
                }
                let at = lines
                    .peek()
                    .and_then(|n| n.trim().strip_prefix("at "))
                    .and_then(|p| p.rsplit('/').next())
                    .unwrap_or("");
                names.push(format!("{name}@{at}"));
                if names.len() >= 5 {
                    break;
                }
            }
        }
        println!("DRAW {n} {}", names.join(" < "));
    }
}
