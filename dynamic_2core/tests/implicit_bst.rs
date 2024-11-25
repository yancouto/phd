use std::sync::Arc;

use common::slow_bst::SlowBst;
use common::AggSum;
use dynamic_2core::implicit_bst::*;

mod common;

struct BSTTests<T: ImplicitBST<AggSum>>(std::marker::PhantomData<T>);

impl<T: ImplicitBST<AggSum>> BSTTests<T> {
    fn build(v: &[i32]) -> Arc<T> {
        T::from_iter(v.iter().copied()).next().unwrap()
    }

    fn test_new_empty() {
        let bst = T::new_empty();
        assert_eq!(bst.total_agg().0, 0);
    }

    fn test_new() {
        let bst = T::new(1);
        assert_eq!(bst.total_agg(), 1);
        assert_eq!(bst.node_data(), &1);
    }

    fn test_concat() {
        let bst1 = Self::build(&[1, 2, 3]);
        let bst2 = Self::build(&[8, 12, 10]);
        let bst = bst1.concat(&bst2);
        assert_eq!(bst.total_agg(), 36);
        assert_eq!(bst.find_kth(3).node_data(), &8);
        assert_eq!(bst.find_kth(2).node_data(), &3);
        assert_eq!(bst.find_kth(0).node_data(), &1);
        assert!(bst.find_kth(6).is_empty());
    }

    fn test_split() {
        let bst = Self::build(&[1, 2, 3, 7, 9, 2]);
        assert_eq!(bst.range_agg(0..1), 1);
        assert_eq!(bst.range_agg(1..4), 12);
        assert_eq!(bst.range_agg(4..), 11);
        assert_eq!(bst.range_agg(0..0), 0);
        let (left, mid, right) = bst.split(1..=3);
        assert_eq!(left.total_agg(), 1);
        assert_eq!(mid.total_agg(), 12);
        assert_eq!(right.total_agg(), 11);
    }

    fn same_content(bst1: &T, bst2: &T) -> bool {
        if bst1.len() != bst2.len() {
            return false;
        }
        for i in 0..bst1.len() {
            let n1 = *bst1.find_kth(i).node_data();
            let n2 = *bst2.find_kth(i).node_data();
            if n1 != n2 {
                return false;
            }
        }
        true
    }

    #[allow(dead_code)]
    fn dbg(bst: &T) {
        let bst = bst.root();
        for i in 0..bst.len() {
            print!("{:?} ", bst.find_kth(i).node_data());
        }
        println!();
    }

    fn test_same_as_not_content() {
        let bst1 = Self::build(&[1, 2, 3]);
        let bst2 = Self::build(&[1, 2, 3]);
        assert!(Self::same_content(&bst1, &bst2));
        assert!(!bst1.same_node(&bst2));
        assert!(bst1.same_node(&bst1));
        assert!(bst2.same_node(&bst2));
        let bst3 = Self::build(&[1, 2, 3, 4]);
        let (_, bst4, _) = bst3.split(0..3);
        assert!(Self::same_content(&bst1, &bst4));
        assert!(!bst1.same_node(&bst4));
    }

    fn test_dsu() {
        let n = vec![T::new(1), T::new(2), T::new(3), T::new(4)];
        assert!(!n[0].on_same_tree(&n[1]));
        assert!(!n[0].on_same_tree(&n[2]));
        assert!(!n[0].on_same_tree(&n[3]));
        n[0].concat(&n[1]);
        n[2].concat(&n[3]);
        assert!(n[0].on_same_tree(&n[1]));
        assert!(!n[0].on_same_tree(&n[2]));
        assert!(n[2].on_same_tree(&n[3]));
        assert!(!n[1].on_same_tree(&n[2]));
        assert!(!n[1].on_same_tree(&n[3]));
        n[0].concat(&n[3]);
        assert!(n[1].on_same_tree(&n[2]));
        assert!(n[1].on_same_tree(&n[3]));
        assert!(n[0].on_same_tree(&n[3]));
        assert!(n[0].on_same_tree(&n[2]));
        assert!(Self::same_content(
            &n[2].root(),
            &Self::build(&[1, 2, 3, 4])
        ));
        n[0].split(0..=1);
        assert!(!n[0].on_same_tree(&n[2]));
        n[2].concat(&n[0]);
        assert!(n[1].on_same_tree(&n[3]));
        assert!(Self::same_content(
            &n[2].root(),
            &Self::build(&[3, 4, 1, 2])
        ));
    }

    fn test_all() {
        Self::test_new_empty();
        Self::test_new();
        Self::test_concat();
        Self::test_split();
        Self::test_same_as_not_content();
        Self::test_dsu();
    }
}

#[test]
fn test_slow_bst() {
    BSTTests::<SlowBst<AggSum>>::test_all();
}
