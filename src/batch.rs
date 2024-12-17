use super::*;
use std::{array::from_fn, hint::black_box};

fn setup_batch<const B: usize>(
    size: usize,
) -> (AlignedVec<Padded<*const usize, 64>>, [*const usize; B]) {
    let (v_usize, order) = util::derangement_with_order::<PaddedUsize>(size);
    let v: Vec<*const usize> = vec![0 as *const usize; v_usize.len()];
    let mut v = AlignedVec::<PaddedPointer>::from(&v);
    let offset = v.as_ptr();
    for i in 0..v.len() {
        v[i] = (unsafe { offset.offset(v_usize[i] as isize) } as *const usize).into();
    }
    let i0: [_; B] = from_fn(|i| *v[order[i * v.len() / B]]);
    drop(order);
    (v, i0)
}

pub fn batch_inner<const B: usize, const PREFETCH: bool, const WORK: bool>(size: usize) -> Result {
    let (v, i0) = setup_batch::<B>(size);
    let offset = v.as_ptr() as *const usize;
    Result::new(size, *STEPS, B, &v, || {
        let mut is = i0;
        let mut sum = 0;
        for _ in 0..*STEPS / B {
            for i in &mut is {
                *i = unsafe { **i } as *const usize;
                if PREFETCH {
                    prefetch_ptr(i);
                }
                if WORK {
                    sum += v.len() / (unsafe { i.offset_from(offset) } + 1) as usize;
                }
            }
        }
        black_box(is);
        black_box(sum);
    })
}

pub fn batch<const B: usize>(size: usize) -> Result {
    batch_inner::<B, false, false>(size)
}
pub fn batch_prefetch<const B: usize>(size: usize) -> Result {
    batch_inner::<B, true, false>(size)
}
pub fn batch_work<const B: usize>(size: usize) -> Result {
    batch_inner::<B, false, true>(size)
}
pub fn batch_prefetch_work<const B: usize>(size: usize) -> Result {
    batch_inner::<B, true, true>(size)
}

pub fn batch_exp() {
    let results = &mut vec![];
    run_experiment(batch::<1>, results);
    run_experiment(batch::<2>, results);
    run_experiment(batch::<4>, results);
    run_experiment(batch::<8>, results);
    run_experiment(batch::<16>, results);
    run_experiment(batch::<32>, results);
    run_experiment(batch::<64>, results);
    run_experiment(batch::<128>, results);

    run_experiment(batch_prefetch::<1>, results);
    run_experiment(batch_prefetch::<2>, results);
    run_experiment(batch_prefetch::<4>, results);
    run_experiment(batch_prefetch::<8>, results);
    run_experiment(batch_prefetch::<16>, results);
    run_experiment(batch_prefetch::<32>, results);
    run_experiment(batch_prefetch::<64>, results);
    run_experiment(batch_prefetch::<128>, results);

    run_experiment(batch_work::<1>, results);
    run_experiment(batch_work::<2>, results);
    run_experiment(batch_work::<4>, results);
    run_experiment(batch_work::<8>, results);
    run_experiment(batch_work::<16>, results);
    run_experiment(batch_work::<32>, results);
    run_experiment(batch_work::<64>, results);
    run_experiment(batch_work::<128>, results);

    run_experiment(batch_prefetch_work::<1>, results);
    run_experiment(batch_prefetch_work::<2>, results);
    run_experiment(batch_prefetch_work::<4>, results);
    run_experiment(batch_prefetch_work::<8>, results);
    run_experiment(batch_prefetch_work::<16>, results);
    run_experiment(batch_prefetch_work::<32>, results);
    run_experiment(batch_prefetch_work::<64>, results);
    run_experiment(batch_prefetch_work::<128>, results);

    save_results(results, "batching");
}
