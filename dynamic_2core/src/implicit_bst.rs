use std::ops::RangeBounds;

pub trait AggregatedData: Sized + Clone + Default {
    type Data: Sized + Clone;
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

pub trait NodeReference<BST, Ag>
where
    BST: ImplicitBST<Ag>,
    Ag: AggregatedData,
    Self: Clone,
{
    /// BST currently owning the reference.
    fn bst(&self) -> &BST;
    /// Position of a reference in the BST, 0-indexed. Panic if reference is not valid.
    fn order(&self) -> usize;
    /// Data associated with the node in the bst.
    fn data(&self) -> &Ag::Data;
}

pub trait WeakRef: Clone {
    type StrongRef;
    /// Upgrade to strong reference
    fn upgrade(&self) -> Option<Self::StrongRef>;
}

pub trait ImplicitBST<Ag>
where
    Ag: AggregatedData,
    Self: Sized + Clone + Eq,
{
    type WeakRef: WeakRef<StrongRef = Self::StrongRef>;
    type StrongRef: NodeReference<Self, Ag>;
    fn new_empty() -> Self;
    /// List from a single element, plus the reference to that element, which can be used after concating with other lists.
    fn new(data: Ag::Data) -> (Self, Self::WeakRef);
    /// Concat with other list, assume all elements are larger.
    fn concat(&self, other: &Self) -> Self;
    /// Split first range from left and right parts. Returns (left, range, right)
    fn split(&self, range: impl RangeBounds<usize>) -> (Self, Self, Self);
    fn total_agg(&self) -> Ag;
    fn range_agg(&self, range: impl RangeBounds<usize>) -> Ag;
    fn find_kth(&self, k: usize) -> Option<Self::WeakRef>;
    /// Find an element by giving a search strategy.
    fn find_element(
        &self,
        search_strategy: impl FnMut(usize, &Ag::Data, &Ag) -> SearchDirection,
    ) -> Self::WeakRef;
    /// Change the element at a given index.
    fn modify(&self, index: &Self::WeakRef, mod_func: impl FnOnce(&mut Ag::Data)) -> Option<Self>;
    fn contains(&self, index: &Self::WeakRef) -> bool;
}
