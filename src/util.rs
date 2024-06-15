use rand::prelude::SliceRandom;
use std::{
    collections::HashMap,
    path::Path,
    sync::{LazyLock, Mutex},
    time::{Duration, Instant},
};

use crate::ARGS;

/// Return the current CPU frequency in Hz.
pub(crate) fn get_cpu_freq() -> Option<f64> {
    let cur_cpu = get_cpu()?;
    let path = format!("/sys/devices/system/cpu/cpu{cur_cpu}/cpufreq/scaling_cur_freq");
    let path = &Path::new(&path);
    if !path.exists() {
        return None;
    }

    let val = std::fs::read_to_string(path).ok()?;
    Some(val.trim().parse::<f64>().ok()? * 1000.)
}

pub(crate) fn get_cpu() -> Option<i32> {
    #[cfg(not(target_os = "macos"))]
    {
        Some(unsafe { libc::sched_getcpu() })
    }
    #[cfg(target_os = "macos")]
    {
        None
    }
}

static DERANGEMENT: LazyLock<Mutex<HashMap<usize, (Vec<usize>, Vec<usize>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn derangement(len: usize) -> Vec<usize> {
    derangement_with_inv(len).0
}

pub fn derangement_with_inv(len: usize) -> (Vec<usize>, Vec<usize>) {
    let mut d = DERANGEMENT.lock().unwrap();
    if let Some(v) = d.get(&len) {
        return v.clone();
    }
    let mut inv = (0..len).collect::<Vec<_>>();
    inv.shuffle(&mut rand::thread_rng());
    let mut v = vec![0; len];
    for i in 0..len - 1 {
        v[inv[i]] = inv[i + 1];
    }
    v[inv[len - 1]] = inv[0];
    d.insert(len, (v.clone(), inv.clone()));
    (v, inv)
}

/// Return an iterator over sizes to iterate over.
/// Starts at 32B and goes up to ~1GB.
pub fn sizes() -> Vec<usize> {
    let mut v = vec![];
    let from = ARGS.from.unwrap_or(14);
    let to = ARGS.to.unwrap_or(25);
    let dense = ARGS.dense;
    for b in from..=to {
        let base = 1 << b;
        v.push(base);
        if dense {
            v.push(base * 5 / 4);
            v.push(base * 3 / 2);
            v.push(base * 7 / 4);
        }
    }
    v
}

pub fn run_experiment(f: impl Fn(usize) -> Result, results: &mut Vec<Result>) {
    let name = std::any::type_name_of_val(&f);
    let name = name.strip_prefix("perf::").unwrap();
    let name = name.replace("_", "-");
    eprintln!("Running experiment: {}", name);
    results.extend(sizes().iter().map(|size| {
        let mut r = f(*size);
        r.name = name.to_string();
        r
    }));
}

pub fn save_results(results: &Vec<Result>, name: &str) {
    let dir = Path::new("results").to_owned();
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(name).with_extension("json");
    let f = std::fs::File::create(f).unwrap();
    serde_json::to_writer(f, &results).unwrap();
}
#[derive(serde::Serialize)]
pub struct Result {
    /// Experiment name
    name: String,
    /// Input size in bytes.
    size: usize,
    /// Batch size.
    batch: usize,
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
    pub fn new(size: usize, steps: usize, batch: usize, f: impl Fn()) -> Result {
        let start = Instant::now();
        f();
        let duration = start.elapsed();
        let freq = get_cpu_freq().unwrap();
        let latency = duration.as_nanos() as f64 / steps as f64;
        let cycles = latency / 1000000000. * freq;

        println!(
            "n = {size:>12}B, s/it: {latency:>6.2?}ns cycles/it: {cycles:>7.2} freq: {freq:>10}",
        );
        Result {
            name: String::default(),
            size,
            batch,
            steps,
            duration,
            latency,
            cycles,
            freq,
        }
    }
}

/// Prefetch the given cacheline into L1 cache.
pub fn prefetch_index<T>(s: &[T], index: usize) {
    let ptr = unsafe { s.as_ptr().add(index) as *const u64 };
    #[cfg(target_arch = "x86_64")]
    unsafe {
        std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T0);
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        std::arch::x86::_mm_prefetch(ptr as *const i8, std::arch::x86::_MM_HINT_T0);
    }
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // TODO: Put this behind a feature flag.
        // std::arch::aarch64::_prefetch(ptr as *const i8, std::arch::aarch64::_PREFETCH_LOCALITY3);
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
    {
        // Do nothing.
    }
}
