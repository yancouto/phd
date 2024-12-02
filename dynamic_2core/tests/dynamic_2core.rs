use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;

use common::{init_logger, slow_bst::SlowET, LOGGER};
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

    fn compare_with_dumb()
    where
        T: std::fmt::Debug,
    {
        const N: usize = 25;
        let mut t1 = T::new(N);
        let mut t2 = Dumb::new(N);
        let mut edges = vec![];
        let mut rng = rand::rngs::StdRng::seed_from_u64(20178);
        for q in 0..3000 {
            if q % 100 == 0 || q == 1562 || q >= 163 {
                println!("q {}", q);
            }
            if q == 1562 || q == 163 {
                LOGGER
                    .lock()
                    .unwrap()
                    .parse_and_push_temp_spec("trace")
                    .unwrap();
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
                    // println!("Added edge {} {}\n{:?}\n", u, v, &t1);
                    edges.push((u, v));
                }
            } else {
                let idx = rng.gen_range(0..edges.len());
                let (u, v) = edges[idx];
                assert_eq!(t1.remove_edge(u, v), t2.remove_edge(u, v));
                edges.swap_remove(idx);
                // println!("Removed edge {} {}\n{:?}\n", u, v, &t1);
            }
            if q % 1 == 0 {
                let gs = t2.groups();
                for u in 0..N {
                    for v in 0..N {
                        assert_eq!(
                            t1.is_connected(u, v),
                            (gs[u] == gs[v]),
                            "q {} u {} v {}\nt1\n{:?}\n\nt2\n{:?}",
                            q,
                            u,
                            v,
                            &t1,
                            &t2
                        );
                    }
                }
            }
        }
    }
}

struct Dumb {
    adj: Vec<BTreeSet<usize>>,
}

impl std::fmt::Debug for Dumb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v_to_id = self.groups();
        let mut gs = vec![vec![]; v_to_id.iter().copied().max().unwrap_or(0)];
        for (v, &id) in v_to_id.iter().enumerate() {
            gs[id - 1].push(v);
        }
        f.debug_struct("Dumb").field("groups", &gs).finish()
    }
}

impl Dumb {
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
        !self.adj[u].is_empty()
    }
}

#[test]
fn test_dumb() {
    D2CTests::<Dumb>::test_all();
}

// Can't run these in parallel because we used ugly globals.
#[test]
fn test_slow() {
    init_logger();
    D2CTests::<ETTSolver<SlowET<AgData>>>::test_all();
    D2CTests::<ETTSolver<SlowET<AgData>>>::compare_with_dumb();
}
