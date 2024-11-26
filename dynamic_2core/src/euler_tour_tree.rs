use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::implicit_bst::{AggregatedData, ImplicitBST, SearchData, SearchDirection};

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

impl<Data: std::fmt::Debug, InRef> std::fmt::Debug for ETData<Data, InRef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ETData::Node(data) => write!(f, "Node({:?})", data),
            ETData::EdgeOut { data, .. } => write!(f, "EdgeOut({:?})", data),
            ETData::EdgeIn => write!(f, "EdgeIn"),
        }
    }
}

impl<Data, InRef> ETData<Data, InRef> {
    pub fn data(&self) -> Option<&Data> {
        match self {
            ETData::Node(data) => Some(data),
            ETData::EdgeOut { data, .. } => Some(data),
            ETData::EdgeIn => None,
        }
    }
    pub fn data_mut(&mut self) -> Option<&mut Data> {
        match self {
            ETData::Node(data) => Some(data),
            ETData::EdgeOut { data, .. } => Some(data),
            ETData::EdgeIn => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ETAggregated<Ag: AggregatedData, InRef> {
    data: Ag,
    subtree_size: usize,
    _phantom: PhantomData<InRef>,
}

impl<Ag: AggregatedData, InRef> Default for ETAggregated<Ag, InRef> {
    fn default() -> Self {
        Self {
            data: Ag::default(),
            subtree_size: 0,
            _phantom: PhantomData,
        }
    }
}

impl<Ag: AggregatedData, InRef: Clone + Debug> AggregatedData for ETAggregated<Ag, InRef> {
    type Data = ETData<Ag::Data, InRef>;
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

#[derive(Debug)]
pub struct NodeRef<N>(N);
#[derive(Debug)]
pub struct EdgeRef<N>(N, N);

impl<N> AsRef<N> for NodeRef<N> {
    fn as_ref(&self) -> &N {
        &self.0
    }
}

#[derive(Clone)]
pub struct EulerTourTree<BST, Ag>(Arc<BST>, PhantomData<Ag>)
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData;

fn alg_panic() -> ! {
    panic!("EulerTourTree algorithm incorrect")
}
#[allow(dead_code)]
fn or_alg_panic<T>(opt: Option<T>) -> T {
    opt.expect("EulerTourTree algorithm incorrect")
}

impl<BST, Ag> std::fmt::Debug for EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self.0.root();
        write!(f, "Euler tour:")?;
        for i in 0..r.len() {
            write!(f, " {:?}", r.find_kth(i).node_data())?;
        }
        write!(f, "\n")
    }
}
impl<BST, Ag> NodeRef<EulerTourTree<BST, Ag>>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    fn from_bst(bst: Arc<BST>) -> Self {
        Self(EulerTourTree::from_bst(bst))
    }
    /// Makes the given node the root.
    pub fn reroot(&self) {
        EulerTourTree::reroot_raw(&self.0 .0)
    }
    /// BST used to store the euler tour.
    pub fn inner_bst(&self) -> Arc<BST> {
        self.0 .0.clone()
    }
    pub fn is_connected(&self, node2: &Self) -> bool {
        self.0 .0.on_same_tree(&node2.0 .0)
    }

    /// Connects the two nodes with an edge. The root of the first tree remais the root. Returns None if they are already connected.
    pub fn connect(
        &self,
        node2: &Self,
        edge_data: Ag::Data,
    ) -> Option<EdgeRef<EulerTourTree<BST, Ag>>> {
        if self.0 .0.on_same_tree(&node2.0 .0) {
            // Already connected
            None
        } else {
            Self::reroot(node2);
            Some(EulerTourTree::link_root(self, node2, edge_data))
        }
    }

    /// Size of the current subtree of this node.
    pub fn subtree_size(&self) -> usize {
        let root = self.0 .0.root();
        match self.0 .0.order().checked_sub(1) {
            Some(k) => {
                if let ETData::EdgeOut { in_ref, .. } = root.find_kth(k).node_data() {
                    let in_ref = or_alg_panic(in_ref.upgrade());
                    (in_ref.order() - k - 1 + 2) / 3
                } else {
                    alg_panic()
                }
            }
            // It is the root
            None => (root.len() + 2) / 3,
        }
    }

    pub fn is_descendant_of(&self, ascendant: &Self) -> bool {
        if !self.0 .0.on_same_tree(&ascendant.0 .0) {
            return false;
        }
        let asc_subtree = ascendant.subtree_size();
        let asc_order = ascendant.0 .0.order();
        let desc_order = self.0 .0.order();
        desc_order >= asc_order && desc_order < asc_order + 3 * asc_subtree - 2
    }

    /// For in-edges, uses the given data.
    pub fn find_element(
        &self,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
        in_edge_data: Ag::Data,
    ) -> Arc<BST> {
        self.0 .0.find_element(|d| {
            search_strategy(SearchData {
                current_data: d.current_data.data().unwrap_or(&in_edge_data),
                left_agg: d.left_agg.data,
                right_agg: d.right_agg.data,
            })
        })
    }
}

impl<BST, Ag> EdgeRef<EulerTourTree<BST, Ag>>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    fn from_bst(out: Arc<BST>, inp: Arc<BST>) -> Self {
        Self(EulerTourTree::from_bst(out), EulerTourTree::from_bst(inp))
    }
    /// Remove the edge and return the root of the current tree and then the root of the new tree the edge removal created.
    pub fn disconnect(
        &self,
    ) -> (
        NodeRef<EulerTourTree<BST, Ag>>,
        NodeRef<EulerTourTree<BST, Ag>>,
    ) {
        let (a, b, _) = EulerTourTree::disconnect_raw(&self.0 .0, Some(&self.1 .0));
        (NodeRef::from_bst(a), NodeRef::from_bst(b))
    }
    /// BST used to store the euler tour. Reference to the out edge.
    pub fn inner_bst(&self) -> Arc<BST> {
        self.0 .0.clone()
    }
}

impl<BST, Ag> EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    /// Creates a new EulerTourTree with a single node.
    pub fn new(node_data: Ag::Data) -> NodeRef<Self> {
        let bst = BST::new(ETData::Node(node_data));
        NodeRef::from_bst(bst)
    }
    fn from_bst(bst: Arc<BST>) -> Self {
        Self(bst, PhantomData)
    }
    fn reroot_raw(node: &Arc<BST>) {
        let k = match node.order().checked_sub(1) {
            Some(k) => k,
            // Already the root.
            None => return,
        };
        let out_edge = node.root().find_kth(k);
        let (prev_root, new_root, in_edge) = Self::disconnect_raw(&out_edge, None);
        // reuse even the edges so it's easier to keep references to them
        Self::link_root_raw(&new_root, &prev_root, &out_edge, &in_edge);
    }
    /// Adds an edge between the root of self and the root of other. Panics if they are on the same tree.
    fn link_root(
        node1: &NodeRef<Self>, // u
        root2: &NodeRef<Self>, // w
        edge_data: Ag::Data,
    ) -> EdgeRef<Self> {
        assert!(!node1.0 .0.on_same_tree(&root2.0 .0));
        assert!(root2.0 .0.is_root());
        let inp = BST::new(ETData::EdgeIn); // wu
        let out = BST::new(ETData::EdgeOut {
            data: edge_data,
            in_ref: Arc::downgrade(&inp),
        }); // uw
        Self::link_root_raw(&node1.0 .0, &root2.0 .0, &out, &inp);
        EdgeRef::from_bst(out, inp)
    }
    fn link_root_raw(
        node1: &Arc<BST>, // u
        node2: &Arc<BST>, // w
        out_edge: &Arc<BST>,
        in_edge: &Arc<BST>,
    ) {
        // "AAA u BBB" and "w CCC" (it is root) becomes
        // AAA u uw w CCC wu BBB
        let (_, until_node1, after_node1) = node1.split(0..=node1.order());
        until_node1
            .concat(out_edge)
            .concat(node2)
            .concat(in_edge)
            .concat(&after_node1);
    }
    /// Returns the first elements of each tree, which are the roots. And then the removed in_edge.
    fn disconnect_raw(
        out_edge: &Arc<BST>,
        in_edge: Option<&Arc<BST>>,
    ) -> (Arc<BST>, Arc<BST>, Arc<BST>) {
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
}
