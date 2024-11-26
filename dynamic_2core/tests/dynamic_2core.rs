use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

use common::slow_bst::SlowET;
use dynamic_2core::dynamic_2core::{AgData, Dynamic2CoreSolver, ETTSolver};

mod common;

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

    fn test_all() {
        Self::test_dyn_con();
    }

    fn compare_with<T2: Dynamic2CoreSolver + std::fmt::Debug>()
    where
        T: std::fmt::Debug,
    {
        let mut t1 = T::new(30);
        let mut t2 = T2::new(30);
        let mut edges = vec![];
        let mut rng = rand::rngs::StdRng::seed_from_u64(2012);
        for q in 0..10000 {
            if edges.is_empty() || rng.gen_bool(0.90) {
                let u = rng.gen_range(0..29);
                let v = rng.gen_range(u..30);
                let added = t1.add_edge(u, v);
                assert_eq!(added, t2.add_edge(u, v));
                if added {
                    edges.push((u, v));
                }
            } else {
                let idx = rng.gen_range(0..edges.len());
                let (u, v) = edges[idx];
                assert_eq!(t1.remove_edge(u, v), t2.remove_edge(u, v));
                edges.swap_remove(idx);
            }
            if q % 10 == 0 {
                for u in 0..30 {
                    for v in 0..30 {
                        if t1.is_connected(u, v) != t2.is_connected(u, v) {
                            dbg!(q, u, v, &t1, &t2);
                        }
                        assert_eq!(t1.is_connected(u, v), t2.is_connected(u, v));
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
struct Dumb {
    adj: Vec<BTreeSet<usize>>,
}

impl Dynamic2CoreSolver for Dumb {
    fn new(n: usize) -> Self {
        Self {
            adj: vec![BTreeSet::new(); n],
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) -> bool {
        self.adj[u].insert(v) && self.adj[v].insert(u)
    }

    fn remove_edge(&mut self, u: usize, v: usize) -> bool {
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

    fn is_in_2core(&self, u: usize) -> bool {
        todo!()
    }

    fn is_in_1core(&self, u: usize) -> bool {
        todo!()
    }
}

#[test]
fn test_dumb() {
    D2CTests::<Dumb>::test_all();
}

// Can't run these in parallel because we used ugly globals.
#[test]
fn test_slow() {
    D2CTests::<ETTSolver<SlowET<AgData>>>::test_all();
    D2CTests::<ETTSolver<SlowET<AgData>>>::compare_with::<Dumb>();
}
