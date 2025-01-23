use criterion::{black_box, criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use dynamic_2core::lists::{splay::Splays, treap::Treaps, Lists};
use rand::{Rng, SeedableRng};

fn bench_list_impl<L: Lists<()>>(b: &mut Bencher, seed: u64, n: usize, q: usize) {
    b.iter(|| {
        let mut l = black_box(L::new(n));
        for _i in 0..n {
            l.create(black_box(()));
        }
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for _q in 0..q {
            match rng.gen_range(0..100) {
                // concat
                0..33 => {
                    let u = rng.gen_range(0..n);
                    let v = rng.gen_range(0..n);
                    black_box(l.concat(u, v));
                }
                // split
                33..66 => {
                    let u = rng.gen_range(0..n);
                    let sz = l.len(u);
                    let ql = rng.gen_range(0..sz);
                    let qr = rng.gen_range(ql..=sz);
                    black_box(l.split(u, ql..qr));
                }
                // reverse
                66..77 => {
                    let u = rng.gen_range(0..n);
                    black_box(l.reverse(u));
                }
                // Same list query
                77..88 => {
                    let (u, v) = (rng.gen_range(0..n), rng.gen_range(0..n));
                    black_box(l.on_same_list(u, v));
                }
                // range query
                _ => {
                    let u = rng.gen_range(0..n);
                    let sz = l.len(u);
                    let ql = rng.gen_range(0..sz);
                    let qr = rng.gen_range(ql..=sz);
                    black_box(l.range_agg(u, ql..qr));
                }
            }
        }
    });
}

fn bench_list(c: &mut Criterion) {
    let mut g = c.benchmark_group("List");
    let mut rng = rand::rngs::StdRng::seed_from_u64(4815162342);
    for q in [25usize, 50, 100] {
        g.throughput(criterion::Throughput::Elements(q as u64));
        let input_str = format!("N 25 Q {q}");
        let seed = rng.gen();
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
