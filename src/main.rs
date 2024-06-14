use std::{path::Path, time::Instant};

mod util;

#[derive(serde::Serialize)]
struct Result {
    /// Input size in bytes.
    n: usize,
    /// Latency of each operation in nanoseconds.
    latency: f64,
    /// Number of clock cycles per operation.
    cycles: f64,
    /// CPU frequency in Hz.
    freq: f64,
}

/// Return latency per iteration and cycles per iteration.
fn pointer_chasing(n: usize) -> Result {
    let len = n / std::mem::size_of::<usize>();
    let v = util::derangement(len);
    assert_eq!(std::mem::size_of_val(&v[..]), n);

    let steps = 100_000_000usize.next_multiple_of(len);

    let start = Instant::now();
    let mut i = 0;
    for _ in 0..steps {
        i = v[i];
    }
    assert_eq!(i, 0);
    let duration = start.elapsed();
    let freq = util::get_cpu_freq().unwrap();
    let latency = duration.as_nanos() as f64 / steps as f64;
    let cycles = latency / 1000000000. * freq;
    println!(
        "n = {n:>12}B, duration = {duration:>8.2?}, /it: {latency:>6.2?}ns cycles/it: {cycles:>7.2} freq: {freq:>10}",
    );
    Result {
        n,
        latency,
        cycles,
        freq,
    }
}

fn pointer_chasing_experiment() {
    let mut results = vec![];
    // Start at ~10kb, go up to 1GB.
    for b in 13..30 {
        let base = 1 << b;
        for n in [base, base * 5 / 4, base * 3 / 2, base * 7 / 4] {
            results.push(pointer_chasing(n));
        }
    }

    let dir = Path::new("results").to_owned();
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("pointer-chasing.json");
    let f = std::fs::File::create(f).unwrap();
    serde_json::to_writer(f, &results).unwrap();
}

fn main() {
    pointer_chasing_experiment();
}
