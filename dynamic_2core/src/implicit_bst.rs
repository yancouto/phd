use std::fmt::Debug;
use std::ops::RangeBounds;
use std::sync::Arc;

pub trait AggregatedData: Debug + Sized + Clone + Default {
    type Data: Debug + Sized + Clone;
    /// Create aggregated data from a single data item
    fn from(data: &Self::Data) -> Self;
    /// Merge two aggregated data items. The other item contains data of some (not necessarily all) items to the right.
    fn merge(self, right: Self) -> Self;
}

#[derive(Debug)]
pub struct SearchData<'a, Ag: AggregatedData> {
    /// Data of the current node being looked at.
    pub current_data: &'a Ag::Data,
    /// Aggregated data of the left subtree.
    pub left_agg: Ag,
    /// Aggregated data of the right subtree.
    pub right_agg: Ag,
}

#[derive(Debug)]
pub enum SearchDirection {
    Found,
    NotFound,
    Left,
    Right,
}

/// A node of a BST with implicit keys, and values that can be aggregated.
pub trait ImplicitBST<Ag>
where
    Ag: AggregatedData,
    Self: Debug,
{
    /// Empty BST
    fn new_empty() -> Arc<Self>;
    /// BST from a single element.
    fn new(data: Ag::Data) -> Arc<Self>;
    /// BST from list of items
    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> impl Iterator<Item = Arc<Self>>;

    // NODE OPERATIONS - The following don't require the node to be a root.

    /// Returns the root of the tree containing this node.
    fn root(&self) -> Arc<Self>;
    /// Data associated with this node only.
    fn node_data(&self) -> &Ag::Data;
    /// Change data associated with this node.
    fn change_data(&self, f: impl FnOnce(&mut Ag::Data));
    /// Replace data associated with this node.
    fn replace_data(&self, new_data: Ag::Data) {
        self.change_data(|data| *data = new_data);
    }
    /// Position of the node in the full BST, 0-indexed. Panics if empty.
    fn order(&self) -> usize;
    fn is_first(&self) -> bool {
        self.order() == 0
    }
    fn next(&self) -> Arc<Self> {
        self.root().find_kth(self.order() + 1)
    }
    /// Aggregated data of the subtree.
    fn total_agg(&self) -> Ag {
        self.range_agg(..)
    }
    /// Aggregated data of a range in the subtree. (0-indexed on the subtree)
    fn range_agg(&self, range: impl RangeBounds<usize>) -> Ag;
    /// Find an element by giving a search strategy.
    fn find_element(
        &self,
        search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Arc<Self>;
    /// K-th element in the subtree. (0-indexed on the subtree)
    fn find_kth(&self, k: usize) -> Arc<Self>;
    fn first(&self) -> Arc<Self> {
        self.find_kth(0)
    }
    /// Size of the subtree.
    fn len(&self) -> usize;
    /// Is the node empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Same node, not content equality.
    fn same_node(self: &Arc<Self>, other: &Arc<Self>) -> bool {
        Arc::ptr_eq(self, other)
    }
    /// Are the two nodes on the same tree?
    fn on_same_tree(&self, other: &Self) -> bool {
        self.root().same_node(&other.root())
    }
    /// Checks if the current node is the root of the tree.
    fn is_root(self: &Arc<Self>) -> bool {
        self.root().same_node(self)
    }

    // Whole tree operations - These are applied to the root of the tree, not the current node.

    /// Concat the BST containing this node with the one containing the other, assume all elements on it come after. Returns the new root.
    fn concat(&self, other: &Self) -> Arc<Self>;
    /// Splits the given range from the left and right parts. Index is on the WHOLE TREE, not on the subtree. Returns (left, range, right)
    fn split(&self, range: impl RangeBounds<usize>) -> (Arc<Self>, Arc<Self>, Arc<Self>);
}
