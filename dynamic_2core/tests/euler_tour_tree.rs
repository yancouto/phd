use std::sync::Weak;

use common::{slow_bst::SlowET, AggSum};
use dynamic_2core::{
    euler_tour_tree::{ETAggregated, EdgeRef, EulerTourTree, NodeRef},
    implicit_bst::ImplicitBST,
};

mod common;

struct ETTTests<T>(std::marker::PhantomData<T>)
where
    T: ImplicitBST<ETAggregated<AggSum, Weak<T>>>;

type ETT<T> = EulerTourTree<T, AggSum>;
type Node<T> = NodeRef<ETT<T>>;
#[allow(dead_code)]
type Edge<T> = EdgeRef<ETT<T>>;

fn e(u: usize, v: usize) -> i32 {
    (10 * u + v) as i32
}

impl<T> ETTTests<T>
where
    T: ImplicitBST<ETAggregated<AggSum, Weak<T>>>,
{
    fn build(n: usize) -> Vec<Node<T>> {
        (0..n).map(|i| ETT::new(i as i32)).collect()
    }

    // Uses the data for nodes and for edges.
    fn assert_node_order(root: &Node<T>, order: &[i32]) {
        let mut node = root.inner_bst().root().first();
        assert_eq!(node.root().len(), order.len());
        for (i, x) in order.iter().enumerate() {
            assert_eq!(node.node_data().data(), x, "i = {}", i);
            node = node.next();
        }
        assert!(node.is_empty());
    }

    fn assert_subtree_sizes(_nodes: &Vec<Node<T>>, _sizes: &[usize]) {
        // for (i, size) in sizes.iter().enumerate() {
        //     assert_eq!(nodes[i].subtree_size(), *size);
        // }
    }

    fn assert_all_connections(nodes: &Vec<Node<T>>, is_conn: &[&str]) {
        for (i, conn) in is_conn.iter().enumerate() {
            for (j, c) in conn.chars().enumerate() {
                assert_eq!(nodes[i].is_connected(&nodes[j]), c == '1');
            }
        }
    }

    fn assert_all_descendants(_nodes: &Vec<Node<T>>, _descendants: &[&str]) {
        // for (i, conn) in descendants.iter().enumerate() {
        //     for (j, c) in conn.chars().enumerate() {
        //         assert_eq!(nodes[j].is_descendant_of(&nodes[i]), c == '1');
        //     }
        // }
    }

    fn connect(u: usize, v: usize, nodes: &Vec<Node<T>>) -> Edge<T> {
        nodes[u].connect(&nodes[v], e(u, v), e(v, u)).unwrap()
    }

    fn test_simple() {
        let nodes = Self::build(5);
        let mut edges = vec![];
        for i in 0..4 {
            dbg!(i);
            assert!(!nodes[i].is_connected(&nodes[i + 1]),);
            edges.push(Self::connect(i, i + 1, &nodes));
            assert!(nodes[i].is_connected(&nodes[i + 1]));
            dbg!(&nodes[i]);
        }
        Self::assert_node_order(&nodes[0], &[0, 01, 1, 12, 2, 23, 3, 34, 4, 43, 32, 21, 10]);
        Self::assert_subtree_sizes(&nodes, &[5, 4, 3, 2, 1]);
        assert!(nodes[0].connect(&nodes[2], 0, 0).is_none());
        Self::assert_all_descendants(&nodes, &["11111", "01111", "00111", "00011", "00001"]);
        Self::assert_all_connections(&nodes, &["11111", "11111", "11111", "11111", "11111"]);
        edges[1].disconnect(); // 1-2
        Self::assert_node_order(&nodes[0], &[0, 01, 1, 10]);
        Self::assert_node_order(&nodes[2], &[2, 23, 3, 34, 4, 43, 32]);
        Self::assert_subtree_sizes(&nodes, &[2, 1, 3, 2, 1]);
        Self::assert_all_descendants(&nodes, &["11000", "01000", "00111", "00011", "00001"]);
        Self::assert_all_connections(&nodes, &["11000", "11000", "00111", "00111", "00111"]);
        nodes[3].reroot();
        Self::assert_node_order(&nodes[2], &[3, 34, 4, 43, 32, 2, 23]);
        Self::assert_subtree_sizes(&nodes, &[2, 1, 1, 3, 1]);
        Self::assert_all_descendants(&nodes, &["11000", "01000", "00100", "00111", "00001"]);
        Self::assert_all_connections(&nodes, &["11000", "11000", "00111", "00111", "00111"]);
    }

    fn test_reroot() {
        let nodes = Self::build(5);
        for (u, v) in [(0, 4), (0, 1), (1, 2), (2, 3)] {
            Self::connect(u, v, &nodes);
        }
        Self::assert_node_order(&nodes[2], &[0, 01, 1, 12, 2, 23, 3, 32, 21, 10, 04, 4, 40]);
        nodes[3].reroot();
        Self::assert_node_order(&nodes[2], &[3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12, 2, 23]);
        nodes[2].reroot();
        Self::assert_node_order(&nodes[2], &[2, 23, 3, 32, 21, 10, 04, 4, 40, 0, 01, 1, 12]);
    }

    fn test_all() {
        Self::test_simple();
        Self::test_reroot();
    }
}

#[test]
fn test_ett_with_slow_bst() {
    ETTTests::<SlowET>::test_all();
}
