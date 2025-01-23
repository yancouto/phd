use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use dynamic_2core::lists::{splay::Splays, treap::Treaps, Lists};
use flexi_logger::Logger;
use rand::{Rng, SeedableRng};

fn bench_list_impl<L: Lists<()>>(b: &mut Bencher, seed: u64, n: usize, q: usize) {
    b.iter(|| {
        let mut l = black_box(L::new(n));
        for _i in 0..n {
            l.create(black_box(()));
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for _q in 0..q {
            log::trace!("q {qi}/{q}", qi = _q + 1);
            match rng.gen_range(0..100) {
                // concat
                0..33 => {
                    let u = rng.gen_range(0..n);
                    let v = rng.gen_range(0..n);
                    log::trace!("concat {} {}", u, v);
                    black_box(l.concat(u, v));
                }
                // split
                33..66 => {
                    let u = rng.gen_range(0..n);
                    let sz = l.len(u);
                    let ql = rng.gen_range(0..sz);
                    let qr = rng.gen_range(ql..=sz);
                    log::trace!("split {} {}..{}", u, ql, qr);
                    black_box(l.split(u, ql..qr));
                }
                // reverse
                66..77 => {
                    let u = rng.gen_range(0..n);
                    log::trace!("reverse {}", u);
                    black_box(l.reverse(u));
                }
                // Same list query
                77..88 => {
                    let (u, v) = (rng.gen_range(0..n), rng.gen_range(0..n));
                    log::trace!("on_same_list {} {}", u, v);
                    black_box(l.on_same_list(u, v));
                }
                // range query
                _ => {
                    let u = rng.gen_range(0..n);
                    let sz = l.len(u);
                    let ql = rng.gen_range(0..sz);
                    let qr = rng.gen_range(ql..=sz);
                    log::trace!("range_agg {} {}..{}", u, ql, qr);
                    black_box(l.range_agg(u, ql..qr));
                }
            }
        }
    });
}

fn bench_list(c: &mut Criterion) {
    let mut g = c.benchmark_group("List");
    let mut rng = rand::rngs::StdRng::seed_from_u64(4815162342);
    let _logger = Logger::try_with_env().unwrap().start().unwrap();
    for q in [25usize, 50, 100] {
        g.throughput(criterion::Throughput::Elements(q as u64));
        let input_str = format!("N 25 Q {q}");
        let seed = rng.gen();
        log::debug!("Using seed {seed}");
        g.bench_with_input(BenchmarkId::new("splay", &input_str), &q, |b, &q| {
            bench_list_impl::<Splays>(b, seed, 25, q)
        });
        g.bench_with_input(BenchmarkId::new("treap", &input_str), &q, |b, &q| {
            bench_list_impl::<Treaps>(b, seed, 25, q)
        });
    }
    g.finish();
}

criterion_group!(benches, bench_list);
criterion_main!(benches);
