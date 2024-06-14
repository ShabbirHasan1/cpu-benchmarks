use super::Result;
use rand::prelude::SliceRandom;
use std::path::Path;

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

pub fn derangement(len: usize) -> Vec<usize> {
    derangement_with_inv(len).0
}

pub fn derangement_with_inv(len: usize) -> (Vec<usize>, Vec<usize>) {
    let mut inv = (0..len).collect::<Vec<_>>();
    inv.shuffle(&mut rand::thread_rng());
    let mut v = vec![0; len];
    for i in 0..len - 1 {
        v[inv[i]] = inv[i + 1];
    }
    v[inv[len - 1]] = inv[0];
    (v, inv)
}

/// Return an iterator over sizes to iterate over.
/// Starts at 32B and goes up to ~1GB.
pub fn sizes() -> impl Iterator<Item = usize> {
    (3..30).flat_map(move |b| {
        let base = 1 << b;
        [base, base * 5 / 4, base * 3 / 2, base * 7 / 4]
    })
}

pub fn run_experiment(f: impl Fn(usize) -> Result) {
    let name = std::any::type_name_of_val(&f);
    let name = name.strip_prefix("perf::").unwrap();
    let name = name.replace("_", "-");
    eprintln!("Running experiment: {}", name);
    let results = sizes().map(|size| f(size)).collect::<Vec<_>>();

    let dir = Path::new("results").to_owned();
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join(name).with_extension("json");
    let f = std::fs::File::create(f).unwrap();
    serde_json::to_writer(f, &results).unwrap();
}
