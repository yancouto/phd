use std::rc::{Rc, Weak};

use crate::implicit_bst::*;

#[derive(Debug, Clone)]
pub struct SlowBst<Ag: AggregatedData>(Rc<Vec<Ag::Data>>);

#[derive(Debug, Clone)]
pub struct StrongSlow<Ag: AggregatedData> {
    bst: SlowBst<Ag>,
    index: usize,
}

impl<Ag: AggregatedData> NodeReference<SlowBst<Ag>, Ag> for StrongSlow<Ag> {
    fn bst(&self) -> &SlowBst<Ag> {
        &self.bst
    }

    fn order(&self) -> usize {
        self.index
    }

    fn data(&self) -> &Ag::Data {
        &self.bst.0[self.index]
    }
}

#[derive(Debug, Clone)]
pub struct WeakSlow<Ag: AggregatedData> {
    bst: Weak<Vec<Ag::Data>>,
    index: usize,
}

impl<Ag: AggregatedData> WeakRef for WeakSlow<Ag> {
    type StrongRef = StrongSlow<Ag>;

    fn upgrade(&self) -> Option<Self::StrongRef> {
        self.bst.upgrade().map(|bst| StrongSlow {
            bst: SlowBst(bst),
            index: self.index,
        })
    }
}

impl<Ag: AggregatedData> FromIterator<Ag::Data> for SlowBst<Ag> {
    fn from_iter<T: IntoIterator<Item = Ag::Data>>(iter: T) -> Self {
        Self(Rc::new(iter.into_iter().collect()))
    }
}

impl<Ag: AggregatedData> ImplicitBST<Ag> for SlowBst<Ag> {
    type WeakRef = WeakSlow<Ag>;

    type StrongRef = StrongSlow<Ag>;

    fn new_empty() -> Self {
        Self(Rc::new(Vec::new()))
    }

    fn new(data: <Ag as AggregatedData>::Data) -> (Self, Self::WeakRef) {
        let bst = Self(Rc::new(vec![data]));
        let weak = WeakSlow {
            bst: Rc::downgrade(&bst.0),
            index: 0,
        };
        (bst, weak)
    }

    fn concat(&self, other: &Self) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(self.0.as_slice());
        v.extend_from_slice(other.0.as_slice());
        return SlowBst(Rc::new(v));
    }

    fn split(&self, range: impl std::ops::RangeBounds<usize>) -> (Self, Self, Self) {
        let (mut left, mut middle, mut right) = (Vec::new(), Vec::new(), Vec::new());
        let mut reached_middle = false;
        for (i, data) in self.0.iter().enumerate() {
            reached_middle = reached_middle || range.contains(&i);
            match (reached_middle, range.contains(&i)) {
                (false, false) => left.push(data.clone()),
                (_, true) => middle.push(data.clone()),
                (true, false) => right.push(data.clone()),
            }
        }
        (
            SlowBst(Rc::new(left)),
            SlowBst(Rc::new(middle)),
            SlowBst(Rc::new(right)),
        )
    }

    fn total_agg(&self) -> Ag {
        self.0.iter().map(Ag::from).fold(Ag::default(), Ag::merge)
    }

    fn range_agg(&self, range: impl std::ops::RangeBounds<usize>) -> Ag {
        let mut agg = Ag::default();
        for (i, data) in self.0.iter().enumerate() {
            if range.contains(&i) {
                agg = Ag::merge(agg, Ag::from(data));
            }
        }
        agg
    }

    fn find_kth(&self, k: usize) -> Option<Self::WeakRef> {
        if k < self.0.len() {
            Some(WeakSlow {
                bst: Rc::downgrade(&self.0),
                index: k,
            })
        } else {
            None
        }
    }

    fn find_element(
        &self,
        _search_strategy: impl FnMut(usize, &<Ag as AggregatedData>::Data, &Ag) -> SearchDirection,
    ) -> Self::WeakRef {
        unimplemented!()
    }

    fn contains(&self, index: &Self::WeakRef) -> bool {
        index.upgrade().is_some_and(|n| n.bst().same_as(self))
    }

    fn same_as(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}
