use std::vec;

use dynamic_2core::{implicit_bst::*, slow_bst::SlowBst};

#[derive(Debug, Clone, Default)]
struct AggSum(i32);

impl AggregatedData for AggSum {
    type Data = i32;

    fn from(data: &Self::Data) -> Self {
        Self(*data)
    }

    fn merge(self, right: Self) -> Self {
        Self(self.0 + right.0)
    }
}

impl PartialEq<i32> for AggSum {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

struct BSTTests<T: ImplicitBST<AggSum>> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: ImplicitBST<AggSum>> BSTTests<T> {
    fn test_new_empty() {
        let bst = T::new_empty();
        assert_eq!(bst.total_agg().0, 0);
    }

    fn test_new() {
        let (bst, weak) = T::new(1);
        assert_eq!(bst.total_agg(), 1);
        let strong = weak.upgrade().unwrap();
        assert_eq!(strong.data(), &1);
    }

    fn test_concat() {
        let bst1 = T::from_iter(vec![1, 2, 3]);
        let bst2 = T::from_iter(vec![8, 12, 10]);
        let bst = bst1.concat(&bst2);
        assert_eq!(bst.total_agg(), 36);
        assert_eq!(bst.find_kth(3).unwrap().upgrade().unwrap().data(), &8);
        assert_eq!(bst.find_kth(2).unwrap().upgrade().unwrap().data(), &3);
        assert_eq!(bst.find_kth(0).unwrap().upgrade().unwrap().data(), &1);
        assert!(bst.find_kth(6).is_none());
    }

    fn test_split() {
        let bst = T::from_iter(vec![1, 2, 3, 7, 9, 2]);
        let (left, mid, right) = bst.split(1..=3);
        assert_eq!(left.total_agg(), 1);
        assert_eq!(bst.range_agg(0..1), 1);
        assert_eq!(mid.total_agg(), 12);
        assert_eq!(bst.range_agg(1..4), 12);
        assert_eq!(right.total_agg(), 11);
        assert_eq!(bst.range_agg(4..), 11);
        assert_eq!(bst.range_agg(0..0), 0);
    }

    fn same_content(bst1: &T, bst2: &T) -> bool {
        if bst1.len() != bst2.len() {
            return false;
        }
        for i in 0..bst1.len() {
            let n1 = *bst1.find_kth(i).unwrap().upgrade().unwrap().data();
            let n2 = *bst2.find_kth(i).unwrap().upgrade().unwrap().data();
            if n1 != n2 {
                return false;
            }
        }
        true
    }

    fn test_same_as_not_content() {
        let bst1 = T::from_iter(vec![1, 2, 3]);
        let bst2 = T::from_iter(vec![1, 2, 3]);
        assert!(Self::same_content(&bst1, &bst2));
        assert!(!bst1.same_as(&bst2));
        assert!(bst1.same_as(&bst1));
        assert!(bst2.same_as(&bst2));
        let bst3 = T::from_iter(vec![1, 2, 3, 4]);
        let (_, bst4, _) = bst3.split(0..4);
        assert!(Self::same_content(&bst1, &bst4));
        assert!(!bst1.same_as(&bst4));
    }

    fn test_dsu() {
        let bsts = vec![T::new(1), T::new(2), T::new(3), T::new(4)];
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
