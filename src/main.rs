use std::time::{Duration, Instant};

use util::*;

mod util;

#[derive(serde::Serialize)]
struct Result {
    /// Input size in bytes.
    size: usize,
    /// Number of iterations.
    steps: usize,
    /// Total duration of the experiment.
    duration: Duration,
    /// Latency (or reverse throughput) of each operation, in nanoseconds.
    latency: f64,
    /// Number of clock cycles per operation.
    cycles: f64,
    /// CPU frequency in Hz.
    freq: f64,
}

impl Result {
    fn new(size: usize, steps: usize, f: impl Fn()) -> Result {
        let start = Instant::now();
        f();
        let duration = start.elapsed();
        let freq = util::get_cpu_freq().unwrap();
        let latency = duration.as_nanos() as f64 / steps as f64;
        let cycles = latency / 1000000000. * freq;

        println!(
            "n = {size:>12}B, duration = {duration:>8.2?}, /it: {latency:>6.2?}ns cycles/it: {cycles:>7.2} freq: {freq:>10}",
        );
        Result {
            size,
            steps,
            duration,
            latency,
            cycles,
            freq,
        }
    }
}

/// Pointer-chase a derangement.
fn pointer_chasing(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let v = util::derangement(len);
    let steps = 100_000_000usize.next_multiple_of(len);

    Result::new(size, steps, || {
        let mut i = 0;
        for _ in 0..steps {
            i = v[i];
        }
        assert_eq!(i, 0);
    })

}

fn main() {
    run_experiment(pointer_chasing);
}
