#![feature(test)]
extern crate test;
use rand::{thread_rng, Rng, SeedableRng};
use std::collections::BTreeSet;

use common::{init_logger, slow_lct::SlowLCT, slow_lists::SlowLists};
use dynamic_2core::{
    dynamic_2core::{AgData, D2CSolver, Dynamic2CoreSolver},
    euler_tour_tree::ETAggregated,
    link_cut_tree::LCT,
    lists::treap::Treaps,
};

mod common;

trait ToEdge {
    fn to_edge(&self) -> (usize, usize);
}
impl ToEdge for (usize, usize) {
    fn to_edge(&self) -> (usize, usize) {
        *self
    }
}
impl ToEdge for usize {
    fn to_edge(&self) -> (usize, usize) {
        assert!(*self < 100);
        (self / 10, self % 10)
    }
}

struct D2CTests<T>(std::marker::PhantomData<T>)
where
    T: Dynamic2CoreSolver;

impl<T> D2CTests<T>
where
    T: Dynamic2CoreSolver,
{
    fn assert_all_connections(t: &T, groups: &[&[usize]]) {
        for g1 in groups {
            for u in g1.iter().copied() {
                for g2 in groups {
                    for v in g2.iter().copied() {
                        assert_eq!(t.is_connected(u, v), g1 == g2);
                    }
                }
            }
        }
    }

    fn map_core_numbers<T2: Dynamic2CoreSolver>(t: &mut T2, n: usize) -> Vec<usize> {
        (0..n)
            .map(|u| match (t.is_in_1core(u), t.is_in_2core(u)) {
                (true, true) => 2,
                (true, false) => 1,
                (false, false) => 0,
                (false, true) => panic!("In 2core but not in 1core"),
            })
            .collect()
    }

    fn assert_core_numbers(t: &mut T, cores: &[usize]) {
        let all = Self::map_core_numbers(t, cores.len());
        assert_eq!(all, cores);
    }

    fn add_edges(t: &mut T, edges: &[impl ToEdge]) {
        for (u, v) in edges.iter().map(ToEdge::to_edge) {
            assert!(t.add_edge(u, v));
        }
    }

    fn test_dyn_con() {
        let mut t = T::new(5);
        Self::assert_all_connections(&t, &[&[0], &[1], &[2], &[3], &[4]]);
        assert!(t.add_edge(0, 1));
        assert!(t.add_edge(0, 2));
        assert!(!t.add_edge(0, 1));
        assert!(!t.remove_edge(1, 2));
        Self::assert_all_connections(&t, &[&[0, 1, 2], &[3], &[4]]);
        assert!(t.add_edge(1, 4));
        Self::assert_all_connections(&t, &[&[0, 1, 2, 4], &[3]]);
        assert!(t.remove_edge(1, 0));
        Self::assert_all_connections(&t, &[&[0, 2], &[1, 4], &[3]]);
    }

    fn test_2core() {
        let mut t = T::new(11);
        Self::assert_core_numbers(&mut t, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        Self::add_edges(&mut t, &[01, 02, 03, 14, 15, 26, 27, 58, 59]);
        t.add_edge(7, 10);
        Self::assert_core_numbers(&mut t, &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
        t.add_edge(4, 9);
        Self::assert_core_numbers(&mut t, &[1, 2, 1, 1, 2, 2, 1, 1, 1, 2, 1]);
        t.add_edge(6, 7);
        Self::assert_core_numbers(&mut t, &[2, 2, 2, 1, 2, 2, 2, 2, 1, 2, 1]);
        t.remove_edge(4, 9);
        Self::assert_core_numbers(&mut t, &[1, 1, 2, 1, 1, 1, 2, 2, 1, 1, 1]);
        t.add_edge(10, 3);
        Self::assert_core_numbers(&mut t, &[2, 1, 2, 2, 1, 1, 2, 2, 1, 1, 2]);
        t.remove_edge(6, 7);
        Self::assert_core_numbers(&mut t, &[2, 1, 2, 2, 1, 1, 1, 2, 1, 1, 2]);
    }

    fn test_all() {
        Self::test_dyn_con();
        Self::test_2core();
    }

    fn compare_with_slow(seed: u64)
    where
        T: std::fmt::Debug,
    {
        const N: usize = 25;
        let mut t1 = T::new(N);
        let mut t2 = Slow::new(N);
        let mut edges = vec![];
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        for q in 0..10000 {
            if q % 100 == 0 {
                log::debug!("q {q}");
            }
            if edges.is_empty() || rng.gen_bool(0.66) {
                let mut u = rng.gen_range(0..N);
                let mut v = rng.gen_range(0..N - 1);
                if v >= u {
                    v += 1;
                } else {
                    std::mem::swap(&mut u, &mut v);
                }
                let added = t1.add_edge(u, v);
                assert_eq!(added, t2.add_edge(u, v));
                if added {
                    edges.push((u, v));
                }
            } else {
                let idx = rng.gen_range(0..edges.len());
                let (u, v) = edges.swap_remove(idx);
                assert_eq!(t1.remove_edge(u, v), t2.remove_edge(u, v));
            }
            if q % 10 == 0 {
                let gs = t2.groups();
                for u in 0..N {
                    for v in 0..N {
                        assert_eq!(t1.is_connected(u, v), (gs[u] == gs[v]));
                    }
                }
                assert_eq!(
                    Self::map_core_numbers(&mut t1, N),
                    Self::map_core_numbers(&mut t2, N)
                );
            }
        }
    }
}

struct Slow {
    adj: Vec<BTreeSet<usize>>,
    saved_core: Vec<bool>,
    invalidated: bool,
}

impl std::fmt::Debug for Slow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v_to_id = self.groups();
        let mut gs = vec![vec![]; v_to_id.iter().copied().max().unwrap_or(0)];
        for (v, &id) in v_to_id.iter().enumerate() {
            gs[id - 1].push(v);
        }
        f.debug_struct("Slow").field("groups", &gs).finish()
    }
}

impl Slow {
    fn groups(&self) -> Vec<usize> {
        let mut groups = vec![0; self.adj.len()];
        let mut group_id = 0;
        for u in 0..self.adj.len() {
            if groups[u] == 0 {
                group_id += 1;
                groups[u] = group_id;
                let mut stack = vec![u];
                while let Some(u) = stack.pop() {
                    groups[u] = group_id;
                    stack.extend(self.adj[u].iter().copied().filter(|&v| {
                        if groups[v] == 0 {
                            groups[v] = group_id;
                            true
                        } else {
                            false
                        }
                    }));
                }
            }
        }
        groups
    }
}

impl Dynamic2CoreSolver for Slow {
    fn new(n: usize) -> Self {
        Self {
            adj: vec![BTreeSet::new(); n],
            saved_core: vec![false; n],
            invalidated: false,
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) -> bool {
        self.invalidated = true;
        self.adj[u].insert(v) && self.adj[v].insert(u)
    }

    fn remove_edge(&mut self, u: usize, v: usize) -> bool {
        self.invalidated = true;
        self.adj[u].remove(&v) && self.adj[v].remove(&u)
    }

    fn is_connected(&self, u: usize, v: usize) -> bool {
        let mut seen = BTreeSet::new();
        let mut stack = vec![u];
        while let Some(u) = stack.pop() {
            if u == v {
                return true;
            }
            if seen.insert(u) {
                stack.extend(self.adj[u].iter().copied());
            }
        }
        false
    }

    fn is_in_2core(&mut self, u: usize) -> bool {
        if !self.invalidated {
            return self.saved_core[u];
        }
        self.invalidated = false;
        let mut new_adj = self.adj.clone();
        let mut to_rem: Vec<_> = (0..self.adj.len())
            .filter(|&v| self.adj[v].len() <= 1)
            .collect();
        let mut seen = BTreeSet::from_iter(to_rem.iter().copied());
        while let Some(v) = to_rem.pop() {
            for w in new_adj[v].clone() {
                new_adj[w].remove(&v);
                if new_adj[w].len() <= 1 && seen.insert(w) {
                    to_rem.push(w);
                }
            }
        }
        for u in 0..self.adj.len() {
            self.saved_core[u] = !seen.contains(&u);
        }
        self.saved_core[u]
    }

    fn is_in_1core(&self, u: usize) -> bool {
        !self.adj[u].is_empty()
    }
}

#[test]
fn test_dumb() {
    init_logger();
    D2CTests::<Slow>::test_all();
}

#[test]
fn test_slow() {
    init_logger();
    D2CTests::<D2CSolver<SlowLists<ETAggregated<AgData>>, SlowLCT>>::test_all();
}

#[test]
fn test_lct_with_slow() {
    init_logger();
    D2CTests::<D2CSolver<SlowLists<ETAggregated<AgData>>, LCT<SlowLists>>>::test_all();
}

#[test]
fn test_lct_with_treap() {
    init_logger();
    D2CTests::<D2CSolver<Treaps<ETAggregated<AgData>>, LCT<Treaps>>>::test_all();
}

#[test]
fn test_cmp_slow() {
    init_logger();
    D2CTests::<D2CSolver<SlowLists<ETAggregated<AgData>>, LCT<SlowLists>>>::compare_with_slow(
        9232345,
    );
}
#[test]
fn test_dyn2core_cmp1() {
    init_logger();
    D2CTests::<D2CSolver<Treaps<ETAggregated<AgData>>, LCT<Treaps>>>::compare_with_slow(9232345);
}
#[test]
fn test_dyn2core_cmp2() {
    D2CTests::<D2CSolver<Treaps<ETAggregated<AgData>>, LCT<Treaps>>>::compare_with_slow(100000007);
}
#[test]
fn test_dyn2core_cmp3() {
    D2CTests::<D2CSolver<Treaps<ETAggregated<AgData>>, LCT<Treaps>>>::compare_with_slow(3);
}

fn stress_iter() {
    let seed: u64 = thread_rng().gen();
    log::info!("seed = {seed}");
    D2CTests::<D2CSolver<Treaps<ETAggregated<AgData>>, LCT<Treaps>>>::compare_with_slow(seed);
}

#[test]
#[ignore]
fn test_dyn2core_stress() {
    init_logger();
    loop {
        stress_iter();
    }
}

#[bench]
fn test_dyn2core_bench(b: &mut test::Bencher) {
    init_logger();
    b.iter(stress_iter)
}
