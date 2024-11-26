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

impl<T> ETTTests<T>
where
    T: ImplicitBST<ETAggregated<AggSum, Weak<T>>>,
{
    fn build(n: usize) -> Vec<Node<T>> {
        (0..n).map(|i| ETT::new(i as i32)).collect()
    }

    // -1 for going up tree
    fn assert_node_order(root: &Node<T>, order: &[i32]) {
        let root = root.inner_bst().root();
        use dynamic_2core::euler_tour_tree::ETData::*;
        let mut prev_is_out = true;
        assert_eq!(root.len(), (order.len() + 1) / 2 * 3 - 2);
        let mut all = (0..root.len()).filter_map(|i| {
            let node = root.find_kth(i);
            let data = node.node_data();
            // Well formatted list
            assert_eq!(matches!(data, Node(_)), prev_is_out);
            prev_is_out = matches!(data, EdgeOut { .. });
            match data {
                Node(x) => Some(*x),
                EdgeOut { .. } => None,
                EdgeIn => Some(-1),
            }
        });
        for x in order.iter() {
            assert_eq!(all.next(), Some(*x));
        }
        assert_eq!(all.next(), None);
    }

    fn assert_subtree_sizes(nodes: &Vec<Node<T>>, sizes: &[usize]) {
        for (i, size) in sizes.iter().enumerate() {
            assert_eq!(nodes[i].subtree_size(), *size);
        }
    }

    fn assert_all_connections(nodes: &Vec<Node<T>>, is_conn: &[&str]) {
        for (i, conn) in is_conn.iter().enumerate() {
            for (j, c) in conn.chars().enumerate() {
                assert_eq!(nodes[i].is_connected(&nodes[j]), c == '1');
            }
        }
    }

    fn assert_all_descendants(nodes: &Vec<Node<T>>, descendants: &[&str]) {
        for (i, conn) in descendants.iter().enumerate() {
            for (j, c) in conn.chars().enumerate() {
                assert_eq!(nodes[j].is_descendant_of(&nodes[i]), c == '1');
            }
        }
    }

    fn test_simple() {
        let nodes = Self::build(5);
        let mut edges = vec![];
        for i in 0..4 {
            dbg!(i);
            assert!(!nodes[i].is_connected(&nodes[i + 1]),);
            edges.push(nodes[i].connect(&nodes[i + 1], i as i32).unwrap());
            assert!(nodes[i].is_connected(&nodes[i + 1]));
            dbg!(&nodes[i]);
        }
        Self::assert_node_order(&nodes[0], &[0, 1, 2, 3, 4, -1, -1, -1, -1]);
        Self::assert_subtree_sizes(&nodes, &[5, 4, 3, 2, 1]);
        assert!(nodes[0].connect(&nodes[2], 0).is_none());
        Self::assert_all_descendants(&nodes, &["11111", "01111", "00111", "00011", "00001"]);
        Self::assert_all_connections(&nodes, &["11111", "11111", "11111", "11111", "11111"]);
        edges[1].disconnect(); // 1-2
        Self::assert_node_order(&nodes[0], &[0, 1, -1]);
        Self::assert_node_order(&nodes[2], &[2, 3, 4, -1, -1]);
        Self::assert_subtree_sizes(&nodes, &[2, 1, 3, 2, 1]);
        Self::assert_all_descendants(&nodes, &["11000", "01000", "00111", "00011", "00001"]);
        Self::assert_all_connections(&nodes, &["11000", "11000", "00111", "00111", "00111"]);
        nodes[3].reroot();
        Self::assert_node_order(&nodes[2], &[3, 2, -1, 4, -1]);
        Self::assert_subtree_sizes(&nodes, &[2, 1, 1, 3, 1]);
        Self::assert_all_descendants(&nodes, &["11000", "01000", "00100", "00111", "00001"]);
        Self::assert_all_connections(&nodes, &["11000", "11000", "00111", "00111", "00111"]);
    }

    fn test_all() {
        Self::test_simple();
    }
}

#[test]
fn test_with_slow_bst() {
    ETTTests::<SlowET>::test_all();
}
