use std::fmt::Debug;
use std::ops::RangeBounds;

pub mod treap;

pub trait AggregatedData: Debug + Sized + Clone + Default {
    type Data: Debug + Sized + Clone;
    /// Create aggregated data from a single data item
    fn from(data: &Self::Data) -> Self;
    /// Merge two aggregated data items. The other item contains data of some (not necessarily all) items to the right.
    fn merge(self, right: Self) -> Self;
}

impl AggregatedData for () {
    type Data = ();
    fn from(_: &Self::Data) -> Self {
        ()
    }
    fn merge(self, _: Self) -> Self {
        ()
    }
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

pub type Idx = usize;

/// This stores multiple ordered lists of values. Use keys in 0..n.
pub trait Lists<Ag = ()>
where
    Ag: AggregatedData,
    Self: Sized + Debug,
{
    /// Returned when the node doesn't exist.
    const EMPTY: Idx = usize::MAX;
    /// New Lists with given capacity.
    fn new(capacity: usize) -> Self;
    /// New Lists with given items already in a list.
    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> Self {
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
    /// Create a new node with given data. Returns its index, which increase from 0.
    fn create(&mut self, data: Ag::Data) -> Idx;
    /// Number of nodes in all lists.
    fn total_size(&self) -> usize;

    // OPERATIONS
    // They panic if the node doesn't exist.

    /// Returns the root of the list containing u. All nodes in the list have the same root.
    fn root(&self, u: Idx) -> Idx;
    /// Data associated with u.
    fn data(&self, u: Idx) -> &Ag::Data;
    /// Data associated with u.
    fn data_mut(&mut self, u: Idx) -> &mut Ag::Data;
    /// Position of u in its list, 0-indexed.
    fn order(&self, u: Idx) -> usize;
    fn is_first(&self, u: Idx) -> bool {
        self.order(u) == 0
    }
    fn is_last(&self, u: Idx) -> bool {
        self.order(u) == self.len(u) - 1
    }
    /// Next node after u in its list.
    fn next(&self, u: Idx) -> Idx {
        self.find_kth(self.root(u), self.order(u) + 1)
    }
    fn prev(&self, u: Idx) -> Idx {
        self.find_kth(self.root(u), self.order(u) - 1)
    }
    /// Are the two nodes on the same tree?
    fn on_same_list(&self, u: Idx, v: Idx) -> bool {
        self.root(u) == self.root(v)
    }
    /// Checks if the current node is the root of the tree.
    fn is_root(&self, u: Idx) -> bool {
        self.root(u) == u
    }
    /// Is the node the empty node?
    fn is_empty(&self, u: Idx) -> bool {
        u == Self::EMPTY
    }
    /// Find an element in the list containing u using a search strategy.
    fn find_element(
        &mut self,
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
    fn total_agg(&mut self, u: Idx) -> Ag {
        self.range_agg(u, ..)
    }
    /// Aggregated data of a range of the list containing u. (0-indexed)
    fn range_agg(&mut self, u: Idx, range: impl RangeBounds<usize>) -> Ag;

    /// Concats the lists containing u and v. Returns the new root.
    fn concat(&mut self, u: Idx, v: Idx) -> Idx;
    fn concat_all(&mut self, all: impl IntoIterator<Item = Idx>) -> Idx {
        let mut u = Self::EMPTY;
        for v in all {
            u = self.concat(u, v);
        }
        u
    }
    /// Splits the list containing u with the given range from the left and right parts. Returns (left, range, right), which may be EMPTY.
    fn split(&mut self, u: Idx, range: impl RangeBounds<usize>) -> (Idx, Idx, Idx) {
        use std::ops::Bound::*;
        let start = match range.start_bound() {
            Included(start) => *start,
            Excluded(start) => *start + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(end) => *end + 1,
            Excluded(end) => *end,
            Unbounded => self.len(u),
        };
        self.split_lr(u, start, end)
    }
    /// Split with given (inclusive, exclusive( range.
    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx);
    /// Reverse the whole list containing u.
    fn reverse(&mut self, u: Idx);
}
