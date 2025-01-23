use common::{slow_lists::SlowLists, AggSum};
use dynamic_2core::{
    euler_tour_tree::{EdgeRef, EulerTourTree},
    lists::{splay::Splays, treap::Treaps, Idx, Lists},
};

mod common;

struct ETTTests<L>(std::marker::PhantomData<L>)
where
    L: Lists<AggSum>;

type ETT<T> = dynamic_2core::euler_tour_tree::ETT<T, AggSum>;

fn e(u: usize, v: usize) -> i32 {
    (10 * u + v) as i32
}

impl<L> ETTTests<L>
where
    L: Lists<AggSum>,
{
    fn build(n: usize) -> ETT<L> {
        ETT::new((0..(n as i32)).collect())
    }

    // Uses the data for nodes and for edges.
    fn assert_node_order(t: &mut ETT<L>, root: Idx, order: &[i32]) {
        let l = t.inner_lists();
        let mut node = l.first(root);
        assert_eq!(l.len(node), order.len());
        for (i, x) in order.iter().enumerate() {
            assert_eq!(l.data(node), x, "i = {}", i);
            node = l.next(node);
        }
        assert_eq!(node, L::EMPTY);
    }

    fn assert_all_connections(t: &mut ETT<L>, is_conn: &[&str]) {
        for (i, conn) in is_conn.iter().enumerate() {
            for (j, c) in conn.chars().enumerate() {
                assert_eq!(t.is_connected(i, j), c == '1');
            }
        }
    }

    fn connect(t: &mut ETT<L>, u: usize, v: usize) -> EdgeRef {
        t.connect(u, v, e(u, v), e(v, u)).unwrap()
    }

    fn test_simple() {
        let t = &mut Self::build(5);
        let mut edges = vec![];
        for i in 0..4 {
            dbg!(i);
            assert!(!t.is_connected(i, i + 1),);
            edges.push(Self::connect(t, i, i + 1));
            assert!(t.is_connected(i, i + 1));
        }
        Self::assert_node_order(t, 0, &[0, 01, 1, 12, 2, 23, 3, 34, 4, 43, 32, 21, 10]);
        assert!(t.connect(0, 2, 0, 0).is_none());
        Self::assert_all_connections(t, &["11111", "11111", "11111", "11111", "11111"]);
        t.disconnect(edges[1]); // 1-2
        Self::assert_node_order(t, 0, &[0, 01, 1, 10]);
        Self::assert_node_order(t, 2, &[2, 23, 3, 34, 4, 43, 32]);
        Self::assert_all_connections(t, &["11000", "11000", "00111", "00111", "00111"]);
        t.reroot(3);
        Self::assert_node_order(t, 2, &[3, 34, 4, 43, 32, 2, 23]);
        Self::assert_all_connections(t, &["11000", "11000", "00111", "00111", "00111"]);
    }

    fn test_reroot() {
        let mut t = &mut Self::build(5);
        for (u, v) in [(0, 4), (0, 1), (1, 2), (2, 3)] {
            Self::connect(&mut t, u, v);
        }
        Self::assert_node_order(t, 2, &[0, 01, 1, 12, 2, 23, 3, 32, 21, 10, 04, 4, 40]);
        t.reroot(3);
        Self::assert_node_order(t, 2, &[3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12, 2, 23]);
        t.reroot(2);
        Self::assert_node_order(t, 2, &[2, 23, 3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12]);
    }

    fn test_all() {
        Self::test_simple();
        Self::test_reroot();
    }
}

#[test]
fn test_ett_with_slow_lists() {
    ETTTests::<SlowLists<_>>::test_all();
}

#[test]
fn test_ett_with_treap() {
    ETTTests::<Treaps<_>>::test_all();
}

#[test]
fn test_ett_with_splay() {
    ETTTests::<Splays<_>>::test_all();
}
