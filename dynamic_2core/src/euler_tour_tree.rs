use std::{
    marker::PhantomData,
    rc::{Rc, Weak},
};

use crate::implicit_bst::{AggregatedData, ImplicitBST};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    type Data = ETData<Ag::Data, Weak<BST>>;
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
pub struct EulerTourTree<BST, Ag>(Rc<BST>, PhantomData<Ag>)
where
    BST: ImplicitBST<ETAggregated<BST, Ag>>,
    Ag: AggregatedData;

fn alg_panic() -> ! {
    panic!("EulerTourTree algorithm incorrect")
}
#[allow(dead_code)]
fn or_alg_panic<T>(opt: Option<T>) -> T {
    opt.expect("EulerTourTree algorithm incorrect")
}

impl<BST, Ag> EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<BST, Ag>>,
    Ag: AggregatedData,
{
    fn from_bst(bst: Rc<BST>) -> Self {
        Self(bst, PhantomData)
    }
    /// Creates a new EulerTourTree with a single node.
    pub fn new(node_data: Ag::Data) -> NodeRef<Self> {
        let bst = BST::new(ETData::Node(node_data));
        NodeRef(Self::from_bst(bst))
    }
    pub fn from_iter(
        data: impl IntoIterator<Item = Ag::Data>,
    ) -> impl Iterator<Item = NodeRef<Self>> {
        BST::from_iter(data.into_iter().map(ETData::Node))
            .map(Self::from_bst)
            .map(NodeRef)
    }
    /// Makes the given node the root.
    pub fn reroot(node: &NodeRef<Self>) {
        Self::reroot_raw(&node.0 .0)
    }
    fn reroot_raw(node: &Rc<BST>) {
        let k = match node.order().checked_sub(1) {
            Some(k) => k,
            // Already the root.
            None => return,
        };
        let out_edge = node.find_kth(k);
        let (prev_root, new_root, in_edge) = Self::disconnect_raw(&out_edge, None);
        // reuse even the edges so it's easier to keep references to them
        Self::link_roots_raw(&new_root, &prev_root, &out_edge, &in_edge);
    }
    /// Adds an edge between the root of self and the root of other.
    pub fn link_roots(
        node1: &NodeRef<Self>, // u
        node2: &NodeRef<Self>, // w
        edge_data: Ag::Data,
    ) -> EdgeRef<Self> {
        assert!(!node1.0 .0.on_same_tree(&node2.0 .0));
        let inp = BST::new(ETData::EdgeIn); // wu
        let out = BST::new(ETData::EdgeOut {
            data: edge_data,
            in_ref: Rc::downgrade(&inp),
        }); // uw
        Self::link_roots_raw(&node1.0 .0, &node2.0 .0, &out, &inp);
        EdgeRef(Self::from_bst(out), Self::from_bst(inp))
    }
    fn link_roots_raw(
        node1: &Rc<BST>, // u
        node2: &Rc<BST>, // w
        out_edge: &Rc<BST>,
        in_edge: &Rc<BST>,
    ) {
        node1.concat(out_edge).concat(node2).concat(in_edge);
    }
    /// BST used to store the euler tour.
    pub fn inner_bst(&self) -> &BST {
        &self.0
    }
    pub fn is_connected(node1: &Self, node2: &Self) -> bool {
        node1.0.on_same_tree(&node2.0)
    }
    /// Returns the first elements of each tree, which are the roots. And then the removed in_edge.
    fn disconnect_raw(
        out_edge: &Rc<BST>,
        in_edge: Option<&Rc<BST>>,
    ) -> (Rc<BST>, Rc<BST>, Rc<BST>) {
        let in_edge = in_edge.cloned().unwrap_or_else(|| {
            if let ETData::EdgeOut { in_ref, .. } = out_edge.node_data() {
                or_alg_panic(in_ref.upgrade())
            } else {
                alg_panic()
            }
        });
        let (left, middle, right) = out_edge.split(out_edge.order()..=in_edge.order());
        // Remove the first and last items, which is the edge which no longer exists
        let (_, middle, _) = middle.split(1..middle.len() - 1);
        assert_eq!(out_edge.root().len(), 1);
        assert_eq!(in_edge.root().len(), 1);
        (left.concat(&right).first(), middle.first(), in_edge.clone())
    }
    /// Remove the edge and return the root of the current tree and then the root of the new tree the edge removal created.
    pub fn disconnect(edge: &EdgeRef<Self>) -> (NodeRef<Self>, NodeRef<Self>) {
        let (a, b, _) = Self::disconnect_raw(&edge.0 .0, Some(&edge.1 .0));
        (NodeRef(Self::from_bst(a)), NodeRef(Self::from_bst(b)))
    }

    /// Connects the two nodes with an edge. Returns None if they are already connected.
    pub fn connect(
        node1: &NodeRef<Self>,
        node2: &NodeRef<Self>,
        edge_data: Ag::Data,
    ) -> Option<EdgeRef<Self>> {
        if node1.0 .0.on_same_tree(&node2.0 .0) {
            // Already connected
            None
        } else {
            Self::reroot(node1);
            Self::reroot(node2);
            Some(Self::link_roots(node1, node2, edge_data))
        }
    }
}
