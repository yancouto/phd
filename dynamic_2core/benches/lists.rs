use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use dynamic_2core::lists::{splay::Splays, treap::Treaps, AggregatedData, Lists};
use flexi_logger::Logger;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

#[derive(Clone, Copy, Debug)]
enum Operation {
    Concat,
    Split,
    Reverse,
    SameList,
    RangeAgg,
}

#[derive(Clone, Copy, Debug)]
enum OperationDistribution {
    Default,
}

impl OperationDistribution {
    fn get_op(&self, rng: &mut impl Rng) -> Operation {
        let weights = match self {
            Self::Default => [3, 3, 1, 2, 3],
        };
        use Operation::*;
        *[Concat, Split, Reverse, SameList, RangeAgg]
            .choose_weighted(rng, |&o| weights[o as usize])
            .unwrap()
    }
}

fn single_op<L: Lists<Ag>, Ag: AggregatedData>(
    l: &mut L,
    rng: &mut impl Rng,
    op_dist: OperationDistribution,
) {
    let n = l.total_size();
    use Operation::*;
    match op_dist.get_op(rng) {
        Concat => {
            let u = rng.gen_range(0..n);
            let v = rng.gen_range(0..n);
            log::trace!("concat {} {}", u, v);
            black_box(l.concat(u, v));
        }
        Split => {
            let u = rng.gen_range(0..n);
            let sz = l.len(u);
            let ql = rng.gen_range(0..sz);
            let qr = rng.gen_range(ql..=sz);
            log::trace!("split {} {}..{}", u, ql, qr);
            black_box(l.split(u, ql..qr));
        }
        Reverse => {
            let u = rng.gen_range(0..n);
            log::trace!("reverse {}", u);
            black_box(l.reverse(u));
        }
        SameList => {
            let (u, v) = (rng.gen_range(0..n), rng.gen_range(0..n));
            log::trace!("on_same_list {} {}", u, v);
            black_box(l.on_same_list(u, v));
        }
        Operation::RangeAgg => {
            let u = rng.gen_range(0..n);
            let sz = l.len(u);
            let ql = rng.gen_range(0..sz);
            let qr = rng.gen_range(ql..=sz);
            log::trace!("range_agg {} {}..{}", u, ql, qr);
            black_box(l.range_agg(u, ql..qr));
        }
    }
}

fn same_operations_impl<L: Lists>(b: &mut Bencher, seed: u64, n: usize, q: usize) {
    b.iter(|| {
        let mut l = black_box(L::new(n));
        for _i in 0..n {
            l.create(black_box(()));
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for _q in 0..q {
            single_op(&mut l, &mut rng, OperationDistribution::Default);
        }
    });
}

fn same_operations(c: &mut Criterion) {
    let _ = &*LOGGER;
    let mut g = c.benchmark_group("Per fixed batch");
    let mut rng = rand::rngs::StdRng::seed_from_u64(4815162342);
    for q in [25usize, 50, 100] {
        g.throughput(criterion::Throughput::Elements(q as u64));
        let input_str = format!("N 25 Batch size {q}");
        let seed = rng.gen();
        log::debug!("Using seed {seed}");
        g.bench_with_input(BenchmarkId::new("splay", &input_str), &q, |b, &q| {
            same_operations_impl::<Splays>(b, seed, 25, q)
        });
        g.bench_with_input(BenchmarkId::new("treap", &input_str), &q, |b, &q| {
            same_operations_impl::<Treaps>(b, seed, 25, q)
        });
    }
    g.finish();
}

fn each_operation_impl<L: Lists<AggSum>>(b: &mut Bencher, seed: u64, dist: OperationDistribution) {
    const N: usize = 1000000;
    let mut l = black_box(L::new(N));
    let mut cur_block_size = 1;
    let mut left_in_block = cur_block_size;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    for i in 0..N {
        l.create(i as i32);
        left_in_block -= 1;
        if left_in_block == 0 {
            cur_block_size =
                ((cur_block_size + rng.gen_range(1..5)) as f64 * rng.gen_range(1.1..2.1)) as usize;
            left_in_block = cur_block_size;
        } else if i > 0 {
            if rng.gen() {
                l.concat(i, i - 1);
            } else {
                l.concat(i - 1, i);
            }
        }
    }
    b.iter(|| {
        single_op(&mut l, &mut rng, dist);
    });
}

fn each_operation(c: &mut Criterion) {
    let _ = &*LOGGER;
    let mut g = c.benchmark_group("Per operation N = 10^6");
    let mut rng = rand::rngs::StdRng::seed_from_u64(4815162342);
    g.throughput(criterion::Throughput::Elements(1));
    g.measurement_time(Duration::from_secs(30));
    g.warm_up_time(Duration::from_secs(10));
    for dist in [OperationDistribution::Default] {
        let seed = rng.gen();
        log::debug!("Using seed {seed}");
        let input_str = format!("{dist:?}").to_lowercase();
        g.bench_with_input(BenchmarkId::new("splay", &input_str), &dist, |b, &dist| {
            each_operation_impl::<Splays<AggSum>>(b, seed, dist)
        });
        g.bench_with_input(BenchmarkId::new("treap", &input_str), &dist, |b, &dist| {
            each_operation_impl::<Treaps<AggSum>>(b, seed, dist);
        });
    }
    g.finish();
}

criterion_group!(benches, same_operations, each_operation);
criterion_main!(benches);

pub static LOGGER: LazyLock<Mutex<flexi_logger::LoggerHandle>> = LazyLock::new(|| {
    Mutex::new(
        Logger::try_with_env_or_str("info")
            .unwrap()
            .write_mode(flexi_logger::WriteMode::SupportCapture)
            .log_to_stdout()
            .set_palette("196;208;3;7;8".to_owned())
            .format(|w, now, record| {
                let style = flexi_logger::style(record.level());
                write!(
                    w,
                    "{} {pref}[{}] {}{suf}",
                    now.format("%H:%M:%S"),
                    &record.level().as_str()[0..1],
                    record.args(),
                    pref = style.prefix(),
                    suf = style.suffix(),
                )
            })
            .start()
            .unwrap(),
    )
});

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AggSum(pub i32);

impl AggregatedData for AggSum {
    type Data = i32;

    fn from(data: &Self::Data) -> Self {
        Self(*data)
    }

    fn merge(self, right: Self) -> Self {
        Self(self.0 + right.0)
    }

    fn reverse(self) -> Self {
        self
    }
}
