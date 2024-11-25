use std::fmt::Debug;
use std::ops::RangeBounds;
use std::rc::Rc;

pub trait AggregatedData: Debug + Sized + Clone + Default {
    type Data: Debug + Sized + Clone;
    /// Create aggregated data from a single data item
    fn from(data: &Self::Data) -> Self;
    /// Merge two aggregated data items. The other item contains data of some (not necessarily all) items to the right.
    fn merge(self, right: Self) -> Self;
}

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
    Self: Clone + Debug,
{
    /// Empty BST
    fn new_empty() -> Rc<Self>;
    /// BST from a single element.
    fn new(data: Ag::Data) -> Rc<Self>;
    /// BST from list of items
    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> impl Iterator<Item = Rc<Self>>;

    // NODE OPERATIONS - The following don't require the node to be a root.

    /// Returns the root of the tree containing this node.
    fn root(&self) -> Rc<Self>;
    /// Data associated with this node only.
    fn node_data(&self) -> &Ag::Data;
    /// Position of the node in the full BST, 0-indexed. Panics if empty.
    fn order(&self) -> usize;
    /// Aggregated data of the subtree.
    fn total_agg(&self) -> Ag {
        self.range_agg(..)
    }
    /// Aggregated data of a range in the subtree. (0-indexed on the subtree)
    fn range_agg(&self, range: impl RangeBounds<usize>) -> Ag;
    /// Find an element by giving a search strategy.
    fn find_element(
        &self,
        search_strategy: impl FnMut(usize, &Ag::Data, &Ag) -> SearchDirection,
    ) -> Rc<Self>;
    /// K-th element in the subtree. (0-indexed on the subtree)
    fn find_kth(&self, k: usize) -> Rc<Self>;
    fn first(&self) -> Rc<Self> {
        self.find_kth(0)
    }
    /// Size of the subtree.
    fn len(&self) -> usize;
    /// Is the node empty?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Same node, not content equality.
    fn same_node(self: &Rc<Self>, other: &Rc<Self>) -> bool {
        Rc::ptr_eq(self, other)
    }
    /// Are the two nodes on the same tree?
    fn on_same_tree(&self, other: &Self) -> bool {
        self.root().same_node(&other.root())
    }
    /// Checks if the current node is the root of the tree.
    fn is_root(self: &Rc<Self>) -> bool {
        self.root().same_node(self)
    }

    // Whole tree operations - These are applied to the root of the tree, not the current node.

    /// Concat the BST containing this node with the one containing the other, assume all elements on it come after. Returns the new root.
    fn concat(&self, other: &Self) -> Rc<Self>;
    /// Splits the given range from the left and right parts. Index is on the WHOLE TREE, not on the subtree. Returns (left, range, right)
    fn split(&self, range: impl RangeBounds<usize>) -> (Rc<Self>, Rc<Self>, Rc<Self>);
}
