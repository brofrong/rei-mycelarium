//! Conformance debugging: the callers of the game-RNG draws while the title sim boots and runs
//! `steps` empty steps. `cargo run -p myco_core --features trace-draws --example rng_trace --
//! <seed> <from> <to> [steps]` (set MYCO_RNG_SEED=<seed> too, as the conformance harness does).
use myco_core::sim::{Sim, StepInput};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: i64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(31607);
    let from: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let to: u64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(500);
    let steps: u64 = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(2);
    myco_core::rng::draw_trace::set_range(from, to);
    let mut sim = Sim::title_screen(Some(seed));
    println!("RNG 0 {}", sim.rng.state());
    let input = StepInput {
        view: myco_core::sim::ViewRect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        },
        ..Default::default()
    };
    for _ in 0..steps {
        sim.step(&input);
        println!("RNG {} {}", sim.frame, sim.rng.state());
    }
}
