use crate::implicit_bst::{AggregatedData, ImplicitBST};

#[derive(Clone)]
struct ETData<Data> {
    data: Data,
    subtree_size: usize,
}

impl<Aggregated: AggregatedData> AggregatedData for ETData<Aggregated> {
    type Data = Aggregated::Data;
    fn from(data: &Self::Data) -> Self {
        Self {
            data: Aggregated::from(data),
            subtree_size: 1,
        }
    }
    fn merge(self, right: Self) -> Self {
        Self {
            data: self.data.merge(right.data),
            subtree_size: self.subtree_size + right.subtree_size,
        }
    }
}

impl<Data> AsRef<Data> for ETData<Data> {
    fn as_ref(&self) -> &Data {
        &self.data
    }
}

trait EulerTourTree<Aggregated, BST>
where
    Aggregated: AggregatedData,
    BST: ImplicitBST<ETData<Aggregated>>,
    Self: Sized,
{
    /// Creates a new EulerTourTree with a single node.
    fn new(node_data: Aggregated::Data) -> (Self, BST::Reference);
    /// Panics if node is not a valid reference.
    fn reroot(self, node: &BST::Reference) -> Self;
    /// Adds an edge between the root of self and the root of other.
    fn link_roots(self, other: Self, edge_data: Aggregated::Data) -> (Self, BST::Reference);
    /// BST used to store the euler tour.
    fn inner_bst(&self) -> &BST;
}
