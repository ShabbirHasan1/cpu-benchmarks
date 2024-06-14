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
    let mut v = (0..len).collect::<Vec<_>>();
    v.shuffle(&mut rand::thread_rng());
    let mut inv = vec![0; len];
    for i in 0..len {
        inv[v[i]] = i;
    }
    for i in 0..len - 1 {
        v[inv[i]] = inv[i + 1];
    }
    v[inv[len - 1]] = inv[0];
    inv.clear();
    v
}
