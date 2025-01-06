use std::fmt::Debug;
use std::ops::RangeBounds;

pub mod treap;

pub type Idx = usize;

/// This data structure stores multiple ordered lists of values. Use keys in 0..n.
pub trait Lists<Ag = ()>
where
    Ag: AggregatedData,
    Self: Debug,
{
    /// Returned when the node doesn't exist.
    const EMPTY: Idx;
    /// New Lists with given capacity.
    fn new(capacity: usize) -> Self;
    /// New Lists with given items already in a list.
    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> Self
    where
        Self: Sized,
    {
        let data = data.into_iter();
        let mut lists = Self::new(data.size_hint().0);
        for (i, data) in data.enumerate() {
            lists.create(data);
            if i > 0 {
                lists.concat(i - 1, i);
            }
        }
        lists
    }
    /// Create a new node with given data. Returns its index, which increases from 0.
    fn create(&mut self, data: Ag::Data) -> Idx;
    /// Number of nodes in all lists.
    fn total_size(&self) -> usize;

    // OPERATIONS
    // They panic if the node doesn't exist.

    /// Returns the root of the list containing u. All nodes in the list have the same root.
    fn root(&self, u: Idx) -> Idx;
    /// Data associated with u. Panics if u doesn't exist.
    fn data(&self, u: Idx) -> &Ag::Data;
    /// Data associated with u.
    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data));
    /// Position of u in its list, 0-indexed.
    fn order(&self, u: Idx) -> usize;
    fn is_first(&self, u: Idx) -> bool {
        u == self.first(u)
    }
    fn is_last(&self, u: Idx) -> bool {
        self.order(u) == self.len(u) - 1
    }
    /// Node after u in its list.
    fn next(&self, u: Idx) -> Idx {
        self.find_kth(u, self.order(u) + 1)
    }
    /// Node before u in its list.
    fn prev(&self, u: Idx) -> Idx {
        let k = self.order(u);
        if k == 0 {
            Self::EMPTY
        } else {
            self.find_kth(u, k - 1)
        }
    }
    /// Are the two nodes on the same tree?
    fn on_same_list(&self, u: Idx, v: Idx) -> bool {
        self.root(u) == self.root(v)
    }
    /// Checks if the current node is the root of the tree.
    fn is_root(&self, u: Idx) -> bool {
        self.root(u) == u
    }
    /// Find an element in the list containing u using a search strategy.
    fn find_element(
        &self,
        u: Idx,
        search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx;
    /// K-th element in the list containing u. (0-indexed)
    fn find_kth(&self, u: Idx, k: usize) -> Idx;
    /// First element in the list containing u.
    fn first(&self, u: Idx) -> Idx {
        self.find_kth(u, 0)
    }
    /// Size of the list containing u.
    fn len(&self, u: Idx) -> usize;
    /// Aggregated data of the list containing u.
    fn total_agg(&self, u: Idx) -> Ag {
        self.range_agg(u, ..)
    }
    /// Aggregated data of a range of the list containing u. (0-indexed)
    fn range_agg(&self, u: Idx, range: impl RangeBounds<usize>) -> Ag {
        let [l, r] = range_to_lr(range, || self.len(u));
        self.range_agg_lr(u, l, r)
    }
    /// XXX: Use range_agg(u, l..r) instead.
    fn range_agg_lr(&self, u: Idx, l: usize, r: usize) -> Ag;

    /// Concats the lists containing u and v. Returns the new root.
    fn concat(&mut self, u: Idx, v: Idx) -> Idx;
    /// Concats all given lists. Returns the new root.
    fn concat_all(&mut self, all: impl IntoIterator<Item = Idx>) -> Idx {
        let mut u = Self::EMPTY;
        for v in all {
            u = self.concat(u, v);
        }
        u
    }
    /// Splits the list containing u with the given range from the left and right parts. Returns (left, range, right), which may be EMPTY.
    fn split(&mut self, u: Idx, range: impl RangeBounds<usize>) -> (Idx, Idx, Idx) {
        let [l, r] = range_to_lr(range, || self.len(u));
        self.split_lr(u, l, r)
    }
    /// XXX: Use range_agg(u, l..r) instead.
    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx);
    /// Reverse the whole list containing u.
    fn reverse(&mut self, u: Idx);
}

pub trait AggregatedData: Debug + Clone + Default {
    // Should Data::reverse exist? Not necessary for us, but more generic.
    type Data: Debug + Clone;
    /// Create aggregated data from a single data item
    fn from(data: &Self::Data) -> Self;
    /// Merge two aggregated data items. The other item contains data of some (not necessarily all) items to the right.
    fn merge(self, right: Self) -> Self;
    /// Reverses the aggregated data. Used for reversing the list.
    fn reverse(self) -> Self;
}

#[derive(Debug)]
pub struct SearchData<'a, Ag: AggregatedData> {
    /// Data of the current node being looked at.
    pub current_data: &'a Ag::Data,
    /// Aggregated data of the left subtree.
    pub left_agg: &'a Ag,
    /// Aggregated data of the right subtree.
    pub right_agg: &'a Ag,
}

#[derive(Debug)]
pub enum SearchDirection {
    Found,
    NotFound,
    Left,
    Right,
}

fn range_to_lr(range: impl RangeBounds<usize>, len: impl FnOnce() -> usize) -> [usize; 2] {
    use std::ops::Bound::*;
    let start = match range.start_bound() {
        Included(start) => *start,
        Excluded(start) => *start + 1,
        Unbounded => 0,
    };
    let end = match range.end_bound() {
        Included(end) => *end + 1,
        Excluded(end) => *end,
        Unbounded => len(),
    };
    [start, end]
}

impl AggregatedData for () {
    type Data = ();
    fn from(_: &Self::Data) -> Self {
        ()
    }
    fn merge(self, _: Self) -> Self {
        ()
    }
    fn reverse(self) -> Self {
        ()
    }
}
