use clap::Parser;

use util::*;

mod util;

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

/// Pointer-chase a derangement, B pointers at a time.
/// Pointers are initialized to be exactly spread out along the cycle.
fn pointer_chasing_batch<const B: usize>(size: usize) -> Result {
    let len = size / std::mem::size_of::<usize>();
    let (v, inv) = util::derangement_with_inv(len);
    let steps = 100_000_000usize.next_multiple_of(B * len);

    let i0: [usize; B] = std::array::from_fn(|j| inv[j * len / B]);
    drop(inv);
    Result::new(size, steps, || {
        let mut i = i0;
        for _ in 0..steps / B {
            for j in 0..B {
                i[j] = v[i[j]];
            }
        }
        assert_eq!(i, i0);
    })
}

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    large: bool,
}

fn main() {
    let args = Args::parse();
    let mut results = vec![];
    results.extend(run_experiment(pointer_chasing, args.large));
    // results.extend(run_experiment(pointer_chasing_batch::<1>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<2>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<4>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<8>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<16>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<32>, args.large));
    results.extend(run_experiment(pointer_chasing_batch::<64>, args.large));
    // results.extend(run_experiment(pointer_chasing_batch::<128>, args.large));
    // results.extend(run_experiment(pointer_chasing_batch::<256>, args.large));
    save_results(results, "pointer-chasing");
}
