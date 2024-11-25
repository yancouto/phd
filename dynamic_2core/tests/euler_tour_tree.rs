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
        let root = ETT::inner_bst(root).root();
        use dynamic_2core::euler_tour_tree::ETData::*;
        let mut all = (0..root.len()).filter_map(|i| match root.find_kth(i).node_data() {
            Node(x) => Some(*x),
            EdgeOut { .. } => None,
            EdgeIn => Some(-1),
        });
        for x in order.iter() {
            assert_eq!(all.next(), Some(*x));
        }
        assert_eq!(all.next(), None);
    }

    fn test_simple() {
        let nodes = Self::build(5);
        let mut edges = vec![];
        for i in 0..4 {
            dbg!(i);
            assert!(!ETT::is_connected(&nodes[i], &nodes[i + 1]),);
            edges.push(ETT::connect(&nodes[i], &nodes[i + 1], i as i32).unwrap());
            assert!(ETT::is_connected(&nodes[i], &nodes[i + 1]));
            dbg!(&nodes[i]);
        }
        Self::assert_node_order(&nodes[0], &[0, 1, 2, 3, 4, -1, -1, -1, -1]);
        assert!(ETT::is_connected(&nodes[0], &nodes[4]));
        assert!(ETT::is_parent_of(&nodes[0], &nodes[4]));
        assert!(!ETT::is_parent_of(&nodes[4], &nodes[0]));
        assert!(ETT::connect(&nodes[0], &nodes[2], 0).is_none());
        ETT::disconnect(&edges[1]); // 1-2
        assert!(!ETT::is_connected(&nodes[0], &nodes[4]));
        assert!(!ETT::is_parent_of(&nodes[0], &nodes[4]));
        assert!(ETT::is_connected(&nodes[2], &nodes[4]));
        assert!(ETT::is_parent_of(&nodes[2], &nodes[3]));
        Self::assert_node_order(&nodes[0], &[0, 1, -1]);
        Self::assert_node_order(&nodes[2], &[2, 3, 4, -1, -1]);
        ETT::reroot(&nodes[3]);
        assert!(!ETT::is_parent_of(&nodes[2], &nodes[3]));
        assert!(ETT::is_parent_of(&nodes[3], &nodes[2]));
        Self::assert_node_order(&nodes[2], &[3, 2, -1, 4, -1]);
    }

    fn test_all() {
        Self::test_simple();
    }
}

#[test]
fn test_with_slow_bst() {
    ETTTests::<SlowET>::test_all();
}
