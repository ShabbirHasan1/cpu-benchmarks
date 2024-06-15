use std::sync::LazyLock;

use clap::Parser;

use util::*;

mod util;

/// Pointer-chase a derangement.
fn pointer_chasing(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let v = util::derangement(len);
    let steps = STEPS.next_multiple_of(len);

    Result::new(size, steps, 1, || {
        let mut i = 0;
        for _ in 0..steps {
            i = v[i];
        }
        assert_eq!(i, 0);
    })
}

/// Pointer-chase a derangement, B pointers at a time.
/// Pointers are initialized to be exactly spread out along the cycle.
fn pointer_chasing_batch<const B: usize>(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let (v, inv) = util::derangement_with_inv(len);
    let steps = STEPS.next_multiple_of(B * len);

    let i0: [usize; B] = std::array::from_fn(|j| inv[j * len / B]);
    drop(inv);
    Result::new(size, steps, B, || {
        let mut i = i0;
        for _ in 0..steps / B {
            for j in 0..B {
                i[j] = v[i[j]];
            }
        }
        assert_eq!(i, i0);
    })
}

/// Pointer-chase a derangement, B pointers at a time.
/// Pointers are initialized to be exactly spread out along the cycle.
/// Prefetch the location for the next iteration.
fn pointer_chasing_prefetch<const B: usize>(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let (v, inv) = util::derangement_with_inv(len);
    let steps = STEPS.next_multiple_of(B * len);

    let i0: [usize; B] = std::array::from_fn(|j| inv[j * len / B]);
    drop(inv);
    Result::new(size, steps, B, || {
        let mut i = i0;
        for _ in 0..steps / B {
            for j in 0..B {
                i[j] = v[i[j]];
                prefetch_index(&v, i[j]);
            }
        }
        assert_eq!(i, i0);
    })
}

/// Pointer-chase a derangement, B pointers at a time.
/// Pointers are initialized to be exactly spread out along the cycle.
/// Each iteration does some 'heavy' work, to block pipelining.
fn pointer_chasing_batch_with_work<const B: usize>(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let (v, inv) = util::derangement_with_inv(len);
    let steps = STEPS.next_multiple_of(B * len);

    let i0: [usize; B] = std::array::from_fn(|j| inv[j * len / B]);
    drop(inv);
    Result::new(size, steps, B, || {
        let mut i = i0;
        let mut sum = 0;
        for _ in 0..steps / B {
            for j in 0..B {
                i[j] = v[i[j]];
                sum += len / (i[j] + 1);
            }
        }
        assert_eq!(i, i0);
        assert!(sum > len);
    })
}

/// Pointer-chase a derangement, B pointers at a time.
/// Pointers are initialized to be exactly spread out along the cycle.
/// Prefetch the location for the next iteration.
fn pointer_chasing_prefetch_with_work<const B: usize>(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let (v, inv) = util::derangement_with_inv(len);
    let steps = STEPS.next_multiple_of(B * len);

    let i0: [usize; B] = std::array::from_fn(|j| inv[j * len / B]);
    drop(inv);
    Result::new(size, steps, B, || {
        let mut i = i0;
        let mut sum = 0;
        for _ in 0..steps / B {
            for j in 0..B {
                i[j] = v[i[j]];
                sum += len / (i[j] + 1);
                prefetch_index(&v, i[j]);
            }
        }
        assert_eq!(i, i0);
        assert!(sum > len);
    })
}

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    from: Option<usize>,
    #[clap(short, long)]
    to: Option<usize>,
    #[clap(short, long)]
    dense: bool,
    #[clap(short, long)]
    experiment: Option<Experiment>,
}

static ARGS: LazyLock<Args> = LazyLock::new(|| Args::parse());
static STEPS: LazyLock<usize> = LazyLock::new(|| if ARGS.dense { 100_000_000 } else { 20_000_000 });

#[derive(clap::ValueEnum, Copy, Clone)]
enum Experiment {
    PointerChasing,
    PointerChasingWithWork,
}

fn pointer_chasing_exp() {
    let results = &mut vec![];
    run_experiment(pointer_chasing, results);
    run_experiment(pointer_chasing_batch::<2>, results);
    run_experiment(pointer_chasing_batch::<4>, results);
    run_experiment(pointer_chasing_batch::<8>, results);
    run_experiment(pointer_chasing_batch::<9>, results);
    run_experiment(pointer_chasing_batch::<10>, results);
    run_experiment(pointer_chasing_batch::<11>, results);
    run_experiment(pointer_chasing_batch::<12>, results);
    run_experiment(pointer_chasing_batch::<13>, results);
    run_experiment(pointer_chasing_batch::<14>, results);
    run_experiment(pointer_chasing_batch::<15>, results);
    run_experiment(pointer_chasing_batch::<16>, results);
    run_experiment(pointer_chasing_batch::<17>, results);
    run_experiment(pointer_chasing_batch::<18>, results);
    run_experiment(pointer_chasing_batch::<19>, results);
    run_experiment(pointer_chasing_batch::<32>, results);
    run_experiment(pointer_chasing_batch::<64>, results);
    run_experiment(pointer_chasing_prefetch::<2>, results);
    run_experiment(pointer_chasing_prefetch::<4>, results);
    run_experiment(pointer_chasing_prefetch::<8>, results);
    run_experiment(pointer_chasing_prefetch::<16>, results);
    run_experiment(pointer_chasing_prefetch::<32>, results);
    run_experiment(pointer_chasing_prefetch::<64>, results);
    save_results(results, "pointer-chasing");
}

fn pointer_chasing_with_work_exp() {
    let results = &mut vec![];
    run_experiment(pointer_chasing_batch_with_work::<1>, results);
    run_experiment(pointer_chasing_batch_with_work::<2>, results);
    run_experiment(pointer_chasing_batch_with_work::<4>, results);
    run_experiment(pointer_chasing_batch_with_work::<8>, results);
    run_experiment(pointer_chasing_batch_with_work::<16>, results);
    run_experiment(pointer_chasing_batch_with_work::<32>, results);
    run_experiment(pointer_chasing_batch_with_work::<64>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<2>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<4>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<5>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<6>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<7>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<8>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<9>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<10>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<11>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<12>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<13>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<14>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<15>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<16>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<17>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<18>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<19>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<20>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<24>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<32>, results);
    run_experiment(pointer_chasing_prefetch_with_work::<64>, results);
    save_results(results, "pointer-chasing-with-work");
}

fn main() {
    let experiments = if let Some(e) = ARGS.experiment {
        vec![e]
    } else {
        vec![
            Experiment::PointerChasing,
            Experiment::PointerChasingWithWork,
        ]
    };
    for e in experiments {
        match e {
            Experiment::PointerChasing => pointer_chasing_exp(),
            Experiment::PointerChasingWithWork => pointer_chasing_with_work_exp(),
        }
    }
}
