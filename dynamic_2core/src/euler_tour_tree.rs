use std::marker::PhantomData;

use crate::implicit_bst::{AggregatedData, ImplicitBST, NodeReference, WeakRef};

#[derive(Clone)]
pub enum ETData<Data, InRef> {
    Node(Data),
    EdgeOut {
        data: Data,
        /// Reference to matching in_edge
        in_ref: InRef,
    },
    EdgeIn,
}

impl<Data, InRef> ETData<Data, InRef> {
    fn data(&self) -> Option<&Data> {
        match self {
            ETData::Node(data) => Some(data),
            ETData::EdgeOut { data, .. } => Some(data),
            ETData::EdgeIn => None,
        }
    }
}

#[derive(Clone)]
pub struct ETAggregated<BST: ImplicitBST<Self>, Ag: AggregatedData> {
    data: Ag,
    subtree_size: usize,
    _phantom: PhantomData<BST>,
}

impl<BST: ImplicitBST<Self>, Ag: AggregatedData> Default for ETAggregated<BST, Ag> {
    fn default() -> Self {
        Self {
            data: Ag::default(),
            subtree_size: 0,
            _phantom: PhantomData,
        }
    }
}

impl<BST: ImplicitBST<Self>, Ag: AggregatedData> AggregatedData for ETAggregated<BST, Ag> {
    type Data = ETData<Ag::Data, BST::WeakRef>;
    fn from(data: &Self::Data) -> Self {
        Self {
            data: data.data().map(Ag::from).unwrap_or_default(),
            subtree_size: matches!(data, ETData::Node(_)).into(),
            _phantom: PhantomData,
        }
    }
    fn merge(self, right: Self) -> Self {
        Self {
            data: self.data.merge(right.data),
            subtree_size: self.subtree_size + right.subtree_size,
            _phantom: PhantomData,
        }
    }
}

pub struct NodeRef<N>(N);
pub struct EdgeRef<N>(N, N);

impl<N> AsRef<N> for NodeRef<N> {
    fn as_ref(&self) -> &N {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct EulerTourTree<BST, Ag>(BST, PhantomData<Ag>)
where
    BST: ImplicitBST<ETAggregated<BST, Ag>>,
    Ag: AggregatedData;

fn alg_panic() -> ! {
    panic!("EulerTourTree algorithm incorrect")
}
fn or_alg_panic<T>(opt: Option<T>) -> T {
    opt.expect("EulerTourTree algorithm incorrect")
}

impl<BST, Ag> EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<BST, Ag>>,
    Ag: AggregatedData,
{
    fn from_bst(bst: BST) -> Self {
        Self(bst, PhantomData)
    }
    /// Creates a new EulerTourTree with a single node.
    pub fn new(node_data: Ag::Data) -> (Self, NodeRef<BST::WeakRef>) {
        let (bst, r) = BST::new(ETData::Node(node_data));
        (Self::from_bst(bst), NodeRef(r))
    }
    /// Makes the given node reference. None if node is not a valid reference.
    pub fn reroot(node: &NodeRef<BST::WeakRef>) -> Option<Self> {
        Self::reroot_raw(node).map(Self::from_bst)
    }
    fn reroot_raw(node: &NodeRef<BST::WeakRef>) -> Option<BST> {
        let node = node.0.upgrade()?;
        let k = match node.order().checked_sub(1) {
            Some(k) => k,
            // Already the root.
            None => return Some(node.bst().clone()),
        };
        let (prev_root, new_root) = Self::disconnect_raw(&node.bst().find_kth(k)?)?;
        Some(new_root.concat(&prev_root))
    }
    /// Adds an edge between the root of self and the root of other.
    pub fn link_roots(
        &self,        // u
        other: &Self, // w
        edge_data: Ag::Data,
    ) -> (Self, EdgeRef<BST::WeakRef>) {
        let (inp, in_ref) = BST::new(ETData::EdgeIn); // wu
        let (out, out_r) = BST::new(ETData::EdgeOut {
            data: edge_data,
            in_ref: in_ref.clone(),
        }); // uw
        let bst = self.0.concat(&out).concat(&other.0).concat(&inp);
        (Self::from_bst(bst), EdgeRef(out_r, in_ref))
    }
    /// BST used to store the euler tour.
    pub fn inner_bst(&self) -> &BST {
        &self.0
    }
    pub fn is_connected(node1: &NodeRef<BST::WeakRef>, node2: &NodeRef<BST::WeakRef>) -> bool {
        node1
            .0
            .upgrade()
            .is_some_and(|r| Some(r.bst()) == node2.0.upgrade().as_ref().map(|n| n.bst()))
    }
    fn disconnect_raw(edge: &BST::WeakRef) -> Option<(BST, BST)> {
        let out_node = edge.upgrade()?;
        let in_node = if let ETData::EdgeOut { in_ref, .. } = out_node.data() {
            or_alg_panic(in_ref.upgrade())
        } else {
            alg_panic()
        };
        let (left, middle, right) = out_node.bst().split(in_node.order()..=out_node.order());
        Some((left.concat(&right), middle))
    }
    /// Remove the edge and return the current tree and then the new tree the edge removal created.
    pub fn disconnect(edge: &EdgeRef<BST::WeakRef>) -> Option<(Self, Self)> {
        let (a, b) = Self::disconnect_raw(&edge.0)?;
        Some((Self::from_bst(a), Self::from_bst(b)))
    }

    pub fn connect(
        node1: &NodeRef<BST::WeakRef>,
        node2: &NodeRef<BST::WeakRef>,
        edge_data: Ag::Data,
    ) -> Option<(Self, EdgeRef<BST::WeakRef>)> {
        let root1 = Self::reroot(node1)?;
        if root1.0.contains(&node2.0) {
            // Already connected
            None
        } else {
            Some(root1.link_roots(&Self::reroot(node2)?, edge_data))
        }
    }
}
