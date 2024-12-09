use std::collections::{BTreeMap, BTreeSet};

use common::{init_logger, log_traces, slow_lists::SlowLists, AggDigit, AggSum};
use dynamic_2core::lists::*;
use rand::prelude::*;
use scopeguard::{OnUnwind, ScopeGuard};
use treap::Treaps;

mod common;

#[allow(unused_imports)]
use dynamic_2core::lists::treap::PrettyIdx as I;

struct LTests<T: Lists<AggSum>>(std::marker::PhantomData<T>);

fn assert_data<L: Lists<impl AggregatedData<Data = i32>>>(l: &L, u: usize, data: &[i32]) {
    assert_eq!(l.len(u), data.len(), "{l:?}");
    let mut cur_u = l.first(u);
    assert!(l.is_first(cur_u));
    for i in 0..data.len() {
        assert_eq!(l.order(cur_u), i);
        assert_eq!(l.find_kth(u, i), cur_u, "i = {i}");
        assert_eq!(l.data(cur_u), &data[i], "element {i}={cur_u} is incorrect");
        if i == data.len() - 1 {
            assert!(l.is_last(cur_u));
        }
        cur_u = l.next(cur_u);
    }
    assert_eq!(cur_u, L::EMPTY);
}

fn guard<L: std::fmt::Debug>(l: L) -> ScopeGuard<L, impl FnOnce(L), OnUnwind> {
    scopeguard::guard_on_unwind(l, |l| log::error!("Crash with {l:?}"))
}

impl<L: Lists<AggSum>> LTests<L> {
    fn build(v: &[i32]) -> ScopeGuard<L, impl FnOnce(L), OnUnwind> {
        let l = guard(L::from_iter(v.iter().copied()));
        Self::assert_data(&l, 0, v);
        l
    }
    fn add_list(l: &mut L, v: &[i32]) -> Idx {
        let u = l.total_size();
        let mut last_root = u;
        for (i, &vi) in v.iter().enumerate() {
            let r = l.create(vi);
            assert_eq!(r, u + i);
            if i > 0 {
                last_root = l.concat(u + i - 1, u + i);
            }
        }
        Self::assert_data(&l, last_root, v);
        u
    }

    fn assert_data(l: &L, u: usize, data: &[i32]) {
        assert_data(l, u, data)
    }

    fn assert_conn(l: &L, lists: &[&[usize]]) {
        let u_to_li: BTreeMap<usize, usize> = lists
            .iter()
            .enumerate()
            .flat_map(|(i, li)| li.iter().copied().zip(std::iter::repeat(i)))
            .collect();
        for (&u, &u_list) in &u_to_li {
            for (&v, &v_list) in &u_to_li {
                assert_eq!(
                    l.on_same_list(u, v),
                    u_list == v_list,
                    "u {u} v {v}\n{u_to_li:?}\n{l:?}"
                );
            }
        }
    }

    fn assert_same_content(l: &L, u: usize, v: usize) -> bool {
        if l.len(u) != l.len(v) {
            return false;
        }
        (0..l.len(u)).all(|i| {
            let n1 = l.data(l.find_kth(u, i));
            let n2 = l.data(l.find_kth(v, i));
            n1 == n2
        })
    }

    fn test_new_empty() {
        let l = L::new(0);
        assert_eq!(l.total_agg(usize::MAX).0, 0);
    }

    fn test_new() {
        let mut l = L::new(1);
        let root = l.create(1);
        assert_eq!(root, 0);
        assert_eq!(l.total_agg(root), 1);
        assert_eq!(l.data(root), &1);
    }

    fn test_concat() {
        let (mut l, r1) = (Self::build(&[1, 2, 3]), 0);
        let r2 = Self::add_list(&mut l, &[8, 12, 10]);
        let r = l.concat(r1, r2);
        assert_eq!(l.total_agg(r), 36);
        assert_eq!(l.data(l.find_kth(r, 3)), &8);
        assert_eq!(l.data(l.find_kth(r, 2)), &3);
        assert_eq!(l.data(l.find_kth(r, 0)), &1);
        assert_eq!(l.find_kth(r, 6), L::EMPTY);
        Self::assert_data(&l, r, &[1, 2, 3, 8, 12, 10]);
        let (r3, r4, r5) = (
            Self::add_list(&mut l, &[15, 20]),
            Self::add_list(&mut l, &[-12]),
            Self::add_list(&mut l, &[99, 98, 97]),
        );
        let r = l.concat_all([r4, r, r5, r3]);
        Self::assert_data(&l, r, &[-12, 1, 2, 3, 8, 12, 10, 99, 98, 97, 15, 20]);
    }

    fn test_split() {
        let mut l = Self::build(&[1, 2, 3, 7, 9, 2]);
        assert_eq!(l.range_agg(0, 0..1), 1);
        assert_eq!(l.range_agg(0, 1..4), 12);
        assert_eq!(l.range_agg(0, 4..), 11);
        assert_eq!(l.range_agg(0, 0..0), 0);
        let (left, mid, right) = l.split(0, 1..=3);
        assert_eq!(l.total_agg(left), 1);
        assert_eq!(l.total_agg(mid), 12);
        assert_eq!(l.total_agg(right), 11);
        Self::assert_data(&l, left, &[1]);
        Self::assert_data(&l, mid, &[2, 3, 7]);
        Self::assert_data(&l, right, &[9, 2]);
        let (left, mid, right) = l.split(mid, 1..1);
        Self::assert_data(&l, left, &[2]);
        Self::assert_data(&l, mid, &[]);
        Self::assert_data(&l, right, &[3, 7]);
    }

    fn test_same_as_not_content() {
        let (mut l, r1) = (Self::build(&[1, 2, 3]), 0);
        let r2 = Self::add_list(&mut l, &[1, 2, 3]);
        assert!(Self::assert_same_content(&l, r1, r2));
        assert!(!l.on_same_list(r1, r2));
        assert!(l.on_same_list(r1, r1));
        assert!(l.on_same_list(r2, r2));
        let r3 = Self::add_list(&mut l, &[1, 2, 3, 4]);
        let (_, mid, _) = l.split(r3, 0..3);
        assert!(Self::assert_same_content(&l, r1, mid));
        assert!(!l.on_same_list(r1, mid));
    }

    fn test_dsu() {
        let mut l = L::new(4);
        for i in 0..4 {
            l.create(i);
        }
        Self::assert_conn(&l, &[&[0], &[1], &[2], &[3]]);
        let root1 = l.concat(0, 1);
        let root2 = l.concat(2, 3);
        assert!(l.on_same_list(root1, 1));
        assert!(!l.on_same_list(root1, root2));
        assert!(l.on_same_list(root2, 3));
        assert!(!l.on_same_list(1, root2));
        Self::assert_conn(&l, &[&[0, 1], &[2, 3]]);
        let root = l.concat(root1, root2);
        assert!(l.on_same_list(root, root2));
        assert!(l.on_same_list(root, 3));
        assert!(l.on_same_list(root, 3));
        assert!(l.on_same_list(root, root2));
        Self::assert_conn(&l, &[&[0, 1, 2, 3]]);
        Self::assert_data(&l, root2, &[0, 1, 2, 3]);
        let (_, m, _) = l.split(root, 0..=1);
        assert!(!l.on_same_list(m, root2));
        Self::assert_conn(&l, &[&[0, 1], &[2, 3]]);
        let root = l.concat(root2, m);
        Self::assert_conn(&l, &[&[0, 1, 2, 3]]);
        assert!(l.on_same_list(root, 3));
        Self::assert_data(&l, root, &[2, 3, 0, 1]);
    }

    fn test_change_data() {
        let (mut l, r) = (Self::build(&[1, 2, 4]), 0);
        Self::assert_data(&l, r, &[1, 2, 4]);
        assert_eq!(l.total_agg(r), 7);
        assert_eq!(l.range_agg(r, 1..), 6);
        let node = l.find_kth(r, 1);
        assert_eq!(node, 1);
        l.mutate_data(node, |d| *d = 10);
        Self::assert_data(&l, r, &[1, 10, 4]);
        assert_eq!(l.total_agg(r), 15);
        assert_eq!(l.range_agg(r, 1..), 14);
        l.mutate_data(r, |d| *d = 100);
        Self::assert_data(&l, r, &[100, 10, 4]);
        assert_eq!(l.total_agg(r), 114);
        assert_eq!(l.range_agg(r, 1..), 14);
        let node = l.find_kth(r, 1);
        l.mutate_data(node, |d| *d = 1000);
        Self::assert_data(&l, r, &[100, 1000, 4]);
        assert_eq!(l.total_agg(r), 1104);
        assert_eq!(l.range_agg(r, 1..), 1004);
        assert_eq!(l.range_agg(r, 2..), 4);
    }

    fn test_find_element() {
        let l = Self::build(&[0, 0, 1, 0, 3, 0, 2, 0, 1, 1000]);
        let idx_of_kth_value = |mut k: i32, expected: Idx| {
            let v = l.find_element(0, move |s: SearchData<'_, AggSum>| {
                if s.left_agg.0 >= k {
                    SearchDirection::Left
                } else if s.left_agg.0 + s.current_data >= k {
                    SearchDirection::Found
                } else {
                    k -= s.left_agg.0 + s.current_data;
                    SearchDirection::Right
                }
            });
            assert_eq!(v, expected, "idx of {k} was wrong");
        };
        idx_of_kth_value(1, 2);
        idx_of_kth_value(2, 4);
        idx_of_kth_value(3, 4);
        idx_of_kth_value(4, 4);
        idx_of_kth_value(5, 6);
        idx_of_kth_value(6, 6);
        idx_of_kth_value(7, 8);
        idx_of_kth_value(8, 9);
        idx_of_kth_value(255, 9);
        idx_of_kth_value(100000, L::EMPTY);
    }

    fn test_all() {
        Self::test_new_empty();
        Self::test_new();
        Self::test_concat();
        Self::test_split();
        Self::test_same_as_not_content();
        Self::test_dsu();
        Self::test_change_data();
        Self::test_find_element();
    }
}

#[allow(non_snake_case)]
fn random_compare_with_slow<L, Ag>(Q: usize, N: usize, range: std::ops::Range<i32>, seed: u64)
where
    Ag: AggregatedData<Data = i32> + Eq,
    L: Lists<Ag>,
{
    init_logger();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let rng = &mut rng;
    let mut l = guard(L::new(N));
    let l = &mut l as &mut L;
    type SL<Ag> = SlowLists<Ag>;
    let mut slow = SL::<Ag>::new(N);
    let sl = &mut slow;
    for i in 0..N {
        let x = rng.gen_range(range.clone());
        assert_eq!(i, l.create(x));
        sl.create(x);
    }
    for q in 1..=Q {
        if q % 100 == 0 {
            log::debug!("q {q}");
        }
        if q == 0 {
            log_traces();
        }
        let lists = sl.lists();
        let lists = &lists;
        let rnd_u = |mn_size: usize, rng: &mut StdRng| {
            *lists
                .iter()
                .filter(|v| v.len() >= mn_size)
                .choose(rng)
                .unwrap_or(&lists[0])
                .choose(rng)
                .unwrap()
        };
        let ln = lists.len();
        let concat = |l: &mut L, sl: &mut SL<Ag>, rng: &mut StdRng| {
            let [l1, l2]: [_; 2] = lists
                .choose_multiple(rng, 2)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let (u, v) = (*l1.choose(rng).unwrap(), *l2.choose(rng).unwrap());
            l.concat(u, v);
            sl.concat(u, v);
        };
        let split = |l: &mut L, sl: &mut SL<Ag>, rng: &mut StdRng| {
            let l1 = lists.choose(rng).unwrap();
            let (a, b) = (rng.gen_range(0..l1.len()), rng.gen_range(0..l1.len()));
            let range = (a.min(b))..(a.max(b));
            let &u = l1.choose(rng).unwrap();
            l.split(u, range.clone());
            sl.split(u, range);
        };
        match rng.gen_range(0..100) {
            // concat
            _ if ln > 20 => concat(l, sl, rng),
            0..35 if ln > 1 => concat(l, sl, rng),
            // split
            0..75 if ln == 1 => split(l, sl, rng),
            35..65 if ln != N => split(l, sl, rng),
            // reverse
            65..90 => {
                let u = rnd_u(2, rng);
                l.reverse(u);
                sl.reverse(u);
            }
            // mutate data
            _ => {
                let u = rnd_u(1, rng);
                let new_val = rng.gen_range(range.clone());
                let f = |v: &mut i32| *v = new_val;
                l.mutate_data(u, &f);
                sl.mutate_data(u, &f);
            }
        }
        if q % 30 == 0 {
            assert_eq!(l.total_size(), sl.total_size());
            let mut roots = BTreeSet::new();
            let lists = sl.lists();
            for (i, list) in lists.iter().enumerate() {
                let any_u = *list.choose(rng).unwrap();
                let root = l.root(any_u);
                assert!(l.is_root(root));
                for &r in &roots {
                    assert!(!l.on_same_list(any_u, r));
                }
                for j in 0..i {
                    for &v in lists[j].choose_multiple(rng, 5) {
                        assert!(!l.on_same_list(*list.choose(rng).unwrap(), v));
                    }
                }
                assert!(roots.insert(root));
                for &u in list {
                    assert_eq!(root, l.root(u), "all should have the same root");
                    for &v in list.choose_multiple(rng, 5) {
                        assert!(l.on_same_list(u, v), "on_same_list wrong");
                    }
                }
                assert_data(
                    l,
                    any_u,
                    &list.iter().map(|&u| *sl.data(u)).collect::<Vec<_>>(),
                );
                assert_eq!(l.total_agg(any_u), sl.total_agg(any_u));
                // Test range_agg
                for _ in 0..10 {
                    let mut ab = [rng.gen_range(0..=list.len()), rng.gen_range(0..=list.len())];
                    ab.sort();
                    let &u = list.choose(rng).unwrap();
                    assert_eq!(l.range_agg(u, ab[0]..ab[1]), sl.range_agg(u, ab[0]..ab[1]));
                }
            }
        }
    }
}

fn test_digits<L: Lists<AggDigit>>() {
    init_logger();
    let mut t = guard(L::from_iter([0, 1, 2, 3, 4, 5, 6, 7].into_iter()));
    assert_eq!(t.total_agg(0), 1234567);
    assert_eq!(t.range_agg(0, 3..=5), 345);
    assert_eq!(t.range_agg(0, 2..7), 23456);
    t.reverse(0);
    assert_eq!(t.total_agg(0), 76543210);
    assert_eq!(t.range_agg(0, 3..=5), 432);
    assert_eq!(t.range_agg(0, 2..7), 54321);
    t.reverse(0);
    assert_eq!(t.range_agg(0, 3..=6), 3456);
    t.split(0, 2..=4);
    t.reverse(0);
    assert_eq!(t.total_agg(0), 10);
    t.concat(0, 2);
    assert_data(&*t, 0, &[1, 0, 2, 3, 4]);
    assert_eq!(t.range_agg(0, 1..=3), 23);
    assert_eq!(t.range_agg(0, 0..=2), 102);
    t.reverse(0);
    assert_data(&*t, 0, &[4, 3, 2, 0, 1]);
    assert_eq!(t.range_agg(0, 1..=2), 32);
    t.concat(5, 0);
    assert_data(&*t, 0, &[5, 6, 7, 4, 3, 2, 0, 1]);
    assert_eq!(t.range_agg(0, 1..=3), 674);
    assert_eq!(t.range_agg(0, 0..7), 5674320);
    let (l, m, r) = t.split(0, 2..=4);
    assert_data(&*t, l, &[5, 6]);
    assert_data(&*t, m, &[7, 4, 3]);
    assert_data(&*t, r, &[2, 0, 1]);
}

#[test]
fn test_slow_lists() {
    init_logger();
    LTests::<SlowLists<AggSum>>::test_all();
    test_digits::<SlowLists<AggDigit>>();
}

#[test]
fn test_treap() {
    init_logger();
    LTests::<Treaps<AggSum>>::test_all();
    test_digits::<Treaps<AggDigit>>();
}

#[test]
fn test_treap_cmp1() {
    random_compare_with_slow::<Treaps<AggSum>, _>(5000, 100, -100000..100000, 10000);
}
#[test]
fn test_treap_cmp2() {
    random_compare_with_slow::<Treaps<AggSum>, _>(500, 1000, -100000..100000, 74828);
}
#[test]
fn test_treap_cmp3() {
    random_compare_with_slow::<Treaps<AggDigit>, _>(10000, 8, 0..10, 4635);
}

#[test]
#[ignore]
fn test_treap_stress() {
    init_logger();
    loop {
        let seed = thread_rng().gen();
        log::info!("seed = {seed}");
        random_compare_with_slow::<Treaps<AggSum>, _>(30000, 200, -100000..100000, seed);
    }
}
