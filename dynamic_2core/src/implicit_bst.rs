pub trait AggregatedData: Sized + Clone {
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

pub trait ImplicitBST<Aggregated>
where
    Aggregated: AggregatedData,
    Self: Sized,
{
    type Reference;
    fn new_empty() -> Self;
    /// List from a single element, plus the reference to that element, which can be used after concating with other lists.
    fn new(data: Aggregated::Data) -> (Self, Self::Reference);
    /// Concat with other list, assume all elements are larger.
    fn concat(self, other: Self) -> Self;
    /// Split first k elements from the rest
    fn split(self, k: usize) -> (Self, Self);
    // TODO: This probably needs to be different, I need to think about lifetimes, or use Rcs everywhere.
    fn find_bst(index: &Self::Reference) -> &Self;
    /// Position of a reference in the list. Panic if reference is not valid.
    fn order_from_reference(&self, index: &Self::Reference) -> usize;
    /// Find an element by giving a search strategy.
    fn find(
        &self,
        search_strategy: impl FnMut(usize, &Aggregated::Data, &Aggregated) -> SearchDirection,
    ) -> Self::Reference;
    /// Change the element at a given index. Panics if index is out of bounds.
    fn modify(&mut self, index: &Self::Reference, mod_func: impl FnOnce(&mut Aggregated::Data));
}
