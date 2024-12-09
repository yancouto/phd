use common::{slow_lists::SlowLists, AggSum};
use dynamic_2core::{
    euler_tour_tree::{EdgeRef, EulerTourTree, NodeRef},
    lists::{treap::Treaps, Lists},
};

mod common;

struct ETTTests<T>(std::marker::PhantomData<T>)
where
    T: Lists<AggSum>;

type ETT<T> = EulerTourTree<T, AggSum>;

fn e(u: usize, v: usize) -> i32 {
    (10 * u + v) as i32
}

impl<L> ETTTests<L>
where
    L: Lists<AggSum>,
{
    fn build(n: usize) -> (ETT<L>, Vec<NodeRef>) {
        let mut t = ETT::new(L::new(n));
        let v: Vec<_> = (0..n).map(|i| t.create_node(i as i32)).collect();
        (t, v)
    }

    // Uses the data for nodes and for edges.
    fn assert_node_order(t: &ETT<L>, root: &NodeRef, order: &[i32]) {
        let l = t.inner_lists();
        let mut node = l.first(root.inner_idx());
        assert_eq!(l.len(node), order.len());
        for (i, x) in order.iter().enumerate() {
            assert_eq!(l.data(node), x, "i = {}", i);
            node = l.next(node);
        }
        assert_eq!(node, L::EMPTY);
    }

    fn assert_all_connections(t: &ETT<L>, nodes: &Vec<NodeRef>, is_conn: &[&str]) {
        for (i, conn) in is_conn.iter().enumerate() {
            for (j, c) in conn.chars().enumerate() {
                assert_eq!(t.is_connected(nodes[i], nodes[j]), c == '1');
            }
        }
    }

    fn connect(t: &mut ETT<L>, u: usize, v: usize, nodes: &Vec<NodeRef>) -> EdgeRef {
        t.connect(nodes[u], nodes[v], e(u, v), e(v, u)).unwrap()
    }

    fn test_simple() {
        let (mut t, nodes) = Self::build(5);
        let mut edges = vec![];
        for i in 0..4 {
            dbg!(i);
            assert!(!t.is_connected(nodes[i], nodes[i + 1]),);
            edges.push(Self::connect(&mut t, i, i + 1, &nodes));
            assert!(t.is_connected(nodes[i], nodes[i + 1]));
            dbg!(&nodes[i]);
        }
        Self::assert_node_order(
            &t,
            &nodes[0],
            &[0, 01, 1, 12, 2, 23, 3, 34, 4, 43, 32, 21, 10],
        );
        assert!(t.connect(nodes[0], nodes[2], 0, 0).is_none());
        Self::assert_all_connections(&t, &nodes, &["11111", "11111", "11111", "11111", "11111"]);
        t.disconnect(edges[1]); // 1-2
        Self::assert_node_order(&t, &nodes[0], &[0, 01, 1, 10]);
        Self::assert_node_order(&t, &nodes[2], &[2, 23, 3, 34, 4, 43, 32]);
        Self::assert_all_connections(&t, &nodes, &["11000", "11000", "00111", "00111", "00111"]);
        t.reroot(nodes[3]);
        Self::assert_node_order(&t, &nodes[2], &[3, 34, 4, 43, 32, 2, 23]);
        Self::assert_all_connections(&t, &nodes, &["11000", "11000", "00111", "00111", "00111"]);
    }

    fn test_reroot() {
        let (mut t, nodes) = Self::build(5);
        for (u, v) in [(0, 4), (0, 1), (1, 2), (2, 3)] {
            Self::connect(&mut t, u, v, &nodes);
        }
        Self::assert_node_order(
            &t,
            &nodes[2],
            &[0, 01, 1, 12, 2, 23, 3, 32, 21, 10, 04, 4, 40],
        );
        t.reroot(nodes[3]);
        Self::assert_node_order(
            &t,
            &nodes[2],
            &[3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12, 2, 23],
        );
        t.reroot(nodes[2]);
        Self::assert_node_order(
            &t,
            &nodes[2],
            &[2, 23, 3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12],
        );
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
