use alloc_madvise::Memory;
use rand::prelude::SliceRandom;
use std::{
    collections::HashMap,
    convert::From,
    ops::{Deref, DerefMut, Index, IndexMut},
    path::Path,
    slice::SliceIndex,
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

/// Cache derangements.
static DERANGEMENT: LazyLock<Mutex<HashMap<usize, (Vec<usize>, Vec<usize>)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// TODO: Alignment.
#[derive(Clone, Copy)]
#[repr(align(64))]
pub struct Padded<T, const S: usize>(T, [u8; S - std::mem::size_of::<T>()])
where
    [(); S - std::mem::size_of::<T>()]:;
impl<T, const S: usize> Deref for Padded<T, S>
where
    [(); S - std::mem::size_of::<T>()]:,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const S: usize> From<T> for Padded<T, S>
where
    [(); S - std::mem::size_of::<T>()]:,
{
    fn from(v: T) -> Self {
        Padded(v, [0; S - std::mem::size_of::<T>()])
    }
}

pub fn derangement<T>(size: usize) -> Vec<usize> {
    derangement_with_order::<T>(size).0
}

pub fn derangement_with_order<T>(size: usize) -> (Vec<usize>, Vec<usize>) {
    let len = size / std::mem::size_of::<T>();
    let mut d = DERANGEMENT.lock().unwrap();
    if let Some((v, order)) = d.get(&len) {
        return (v.clone(), order.clone());
    }
    let mut order = (0..len).collect::<Vec<_>>();
    order.shuffle(&mut rand::thread_rng());
    let mut v = vec![0; len];
    for i in 0..len - 1 {
        v[order[i]] = order[i + 1];
    }
    v[order[len - 1]] = order[0];
    d.insert(len, (v.clone(), order.clone()));
    (v, order)
}

pub struct AlignedVec<T> {
    v: Memory,
    len: usize,
    _t: std::marker::PhantomData<T>,
}

impl<T> AlignedVec<T> {
    pub fn from<U>(v: &Vec<U>) -> Self
    where
        U: Clone,
        T: From<U>,
    {
        let len = v.len();
        let size = len * std::mem::size_of::<T>();
        // Round size up to the next multiple of 4MB.
        let size = size.next_multiple_of(2 * 1024 * 1024);
        let mut mem = alloc_madvise::Memory::allocate(size, false, true).unwrap();
        let mem_mut: &mut [usize] = mem.as_mut();
        let (pref, mem_mut, suf) = unsafe { mem_mut.align_to_mut::<T>() };
        assert!(pref.is_empty());
        assert!(suf.is_empty());
        for i in 0..len {
            mem_mut[i] = v[i].clone().into();
        }

        assert_eq!(mem.len(), size);

        AlignedVec {
            v: mem,
            len,
            _t: std::marker::PhantomData,
        }
    }
}

impl<T> Deref for AlignedVec<T> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = self.v.as_ptr().cast::<T>();
            std::slice::from_raw_parts(ptr, self.len)
        }
    }
}

impl<T> DerefMut for AlignedVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr = self.v.as_ptr_mut().cast::<T>();
            std::slice::from_raw_parts_mut(ptr, self.len)
        }
    }
}
impl<T> Index<usize> for AlignedVec<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.get_unchecked(index) }
    }
}
impl<T> IndexMut<usize> for AlignedVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.get_unchecked_mut(index) }
    }
}

/// Return an iterator over sizes to iterate over.
/// Starts at 32B and goes up to ~1GB.
pub fn sizes() -> Vec<usize> {
    let mut v = vec![];
    let from = ARGS.from.unwrap_or(13);
    let release = ARGS.release;
    let dense = release || ARGS.dense;
    let to = ARGS.to.unwrap_or(if release { 28 } else { 26 });
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
    let name = name.rsplit_once("::").unwrap().1;
    let runs = if ARGS.release { 3 } else { 1 };
    eprintln!("Running experiment: {}", name);
    for run in 0..runs {
        results.extend(sizes().iter().map(|size| {
            let mut r = f(*size);
            r.name = name.to_string();
            r.run = run;
            r
        }))
    }
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
    pub name: String,
    /// Input size in bytes.
    pub size: usize,
    /// Run id.
    pub run: usize,
    /// Batch size.
    pub batch: usize,
    /// Number of iterations.
    pub steps: usize,
    /// Total duration of the experiment.
    pub duration: Duration,
    /// Latency (or reverse throughput) of each operation, in nanoseconds.
    pub latency: f64,
    /// Number of clock cycles per operation.
    pub cycles: f64,
    /// CPU frequency in Hz.
    pub freq: f64,
    /// Alignment of the target vector.
    pub alignment: usize,
}

impl Result {
    pub fn new<T>(size: usize, steps: usize, batch: usize, v: &[T], f: impl Fn()) -> Result {
        let start = Instant::now();
        f();
        let duration = start.elapsed();
        let freq = get_cpu_freq().unwrap();
        let latency = duration.as_nanos() as f64 / steps as f64;
        let cycles = latency / 1000000000. * freq;

        let sz = size::Size::from_bytes(size);
        let sz = format!("{}", sz);

        println!("n = {sz:>8}, s/it: {latency:>6.2?}ns cycles/it: {cycles:>7.2} freq: {freq:>10}",);
        Result {
            name: String::default(),
            run: 0,
            size,
            batch,
            steps,
            duration,
            latency,
            cycles,
            freq,
            alignment: (v.as_ptr() as usize).trailing_zeros() as usize,
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

pub struct Unchecked<T>(pub T);
impl<T: std::ops::Deref> Deref for Unchecked<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: std::ops::Deref> DerefMut for Unchecked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<V, T> Index<usize> for Unchecked<V>
where
    V: std::ops::Deref<Target = [T]>,
    usize: SliceIndex<[T], Output = T>,
{
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let x: &T = unsafe {
            let slice = self.0.deref();
            let y = index.get_unchecked(slice);
            &*y
        };
        x
    }
}
