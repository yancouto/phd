use common::{slow_lists::SlowLists, AggSum};
use dynamic_2core::lists::*;

mod common;

struct BSTTests<T: Lists<AggSum>>(std::marker::PhantomData<T>);

impl<L: Lists<AggSum>> BSTTests<L> {
    fn build(v: &[i32]) -> L {
        let l = L::from_iter(v.iter().copied());
        Self::assert_data(&l, 0, v);
        l
    }
    fn add_list(l: &mut L, v: &[i32]) -> Idx {
        let u = l.total_size();
        for (i, vi) in v.iter().enumerate() {
            let r = l.create(*vi);
            assert_eq!(r, u + i);
            if i > 0 {
                l.concat(u + i - 1, u + i);
            }
        }
        Self::assert_data(&l, u, v);
        u
    }

    fn assert_data(l: &L, u: usize, data: &[i32]) {
        assert_eq!(l.len(u), data.len());
        for i in 0..data.len() {
            assert_eq!(l.data(l.find_kth(u, i)), &data[i]);
        }
    }

    fn assert_same_content(l: &L, u: usize, v: usize) -> bool {
        if l.len(u) != l.len(v) {
            return false;
        }
        (0..l.len(u)).all(|i| {
            let n1 = l.data(l.find_kth(u, i));
            let n2 = l.data(l.find_kth(v, i));
            n1 == n2
        })
    }

    fn test_new_empty() {
        let l = L::new(0);
        assert_eq!(l.total_agg(usize::MAX).0, 0);
    }

    fn test_new() {
        let mut l = L::new(1);
        let root = l.create(1);
        assert_eq!(root, 0);
        assert_eq!(l.total_agg(root), 1);
        assert_eq!(l.data(root), &1);
    }

    fn test_concat() {
        let (mut l, r1) = (Self::build(&[1, 2, 3]), 0);
        let r2 = Self::add_list(&mut l, &[8, 12, 10]);
        let r = l.concat(r1, r2);
        assert_eq!(l.total_agg(r), 36);
        assert_eq!(l.data(l.find_kth(r, 3)), &8);
        assert_eq!(l.data(l.find_kth(r, 2)), &3);
        assert_eq!(l.data(l.find_kth(r, 0)), &1);
        assert!(l.is_empty(l.find_kth(r, 6)));
        Self::assert_data(&l, r, &[1, 2, 3, 8, 12, 10]);
    }

    fn test_split() {
        let mut l = Self::build(&[1, 2, 3, 7, 9, 2]);
        assert_eq!(l.range_agg(0, 0..1), 1);
        assert_eq!(l.range_agg(0, 1..4), 12);
        assert_eq!(l.range_agg(0, 4..), 11);
        assert_eq!(l.range_agg(0, 0..0), 0);
        let (left, mid, right) = l.split(0, 1..=3);
        assert_eq!(l.total_agg(left), 1);
        assert_eq!(l.total_agg(mid), 12, "l = {l:?}");
        assert_eq!(l.total_agg(right), 11);
        Self::assert_data(&l, left, &[1]);
        Self::assert_data(&l, mid, &[2, 3, 7]);
        Self::assert_data(&l, right, &[9, 2]);
    }

    #[allow(dead_code)]
    fn dbg(l: &L, u: usize) {
        for i in 0..l.len(u) {
            print!("{:?} ", l.data(l.find_kth(u, i)));
        }
        println!();
    }

    fn test_same_as_not_content() {
        let (mut l, r1) = (Self::build(&[1, 2, 3]), 0);
        let r2 = Self::add_list(&mut l, &[1, 2, 3]);
        assert!(Self::assert_same_content(&l, r1, r2));
        assert!(!l.on_same_list(r1, r2));
        assert!(l.on_same_list(r1, r1));
        assert!(l.on_same_list(r2, r2));
        let r3 = Self::add_list(&mut l, &[1, 2, 3, 4]);
        let (_, mid, _) = l.split(r3, 0..3);
        assert!(Self::assert_same_content(&l, r1, mid));
        assert!(!l.on_same_list(r1, mid));
    }

    fn test_dsu() {
        let mut l = L::new(4);
        for i in 0..4 {
            l.create(i + 1);
        }
        assert!(!l.on_same_list(0, 1));
        assert!(!l.on_same_list(0, 2));
        assert!(!l.on_same_list(0, 3));
        let root1 = l.concat(0, 1);
        let root2 = l.concat(2, 3);
        assert!(l.on_same_list(root1, 1));
        assert!(!l.on_same_list(root1, root2));
        assert!(l.on_same_list(root2, 3));
        assert!(!l.on_same_list(1, root2));
        assert!(!l.on_same_list(1, 3));
        let root = l.concat(root1, root2);
        assert!(l.on_same_list(root, root2));
        assert!(l.on_same_list(root, 3));
        assert!(l.on_same_list(root, 3));
        assert!(l.on_same_list(root, root2));
        Self::assert_data(&l, root2, &[1, 2, 3, 4]);
        let (_, m, _) = l.split(root, 0..=1);
        assert!(!l.on_same_list(m, root2));
        let root = l.concat(root2, m);
        assert!(l.on_same_list(root, 3));
        Self::assert_data(&l, root, &[3, 4, 1, 2]);
    }

    fn test_change_data() {
        let (mut l, r) = (Self::build(&[1, 2, 4]), 0);
        Self::assert_data(&l, r, &[1, 2, 4]);
        assert_eq!(l.total_agg(r), 7);
        assert_eq!(l.range_agg(r, 1..), 6);
        let node = l.find_kth(r, 1);
        assert_eq!(node, 1);
        *l.data_mut(node) = 10;
        Self::assert_data(&l, r, &[1, 10, 4]);
        assert_eq!(l.total_agg(r), 15);
        assert_eq!(l.range_agg(r, 1..), 14);
        *l.data_mut(r) = 100;
        Self::assert_data(&l, r, &[100, 10, 4]);
        assert_eq!(l.total_agg(r), 114);
        assert_eq!(l.range_agg(r, 1..), 14);
        let node = l.find_kth(r, 1);
        *l.data_mut(node) = 1000;
        Self::assert_data(&l, r, &[100, 1000, 4]);
        assert_eq!(l.total_agg(r), 1104);
        assert_eq!(l.range_agg(r, 1..), 1004);
        assert_eq!(l.range_agg(r, 2..), 4);
    }

    fn test_all() {
        Self::test_new_empty();
        Self::test_new();
        Self::test_concat();
        Self::test_split();
        Self::test_same_as_not_content();
        Self::test_dsu();
        Self::test_change_data();
    }
}

#[test]
fn test_slow_bst() {
    BSTTests::<SlowLists<AggSum>>::test_all();
}
