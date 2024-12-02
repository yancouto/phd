use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::implicit_bst::{AggregatedData, ImplicitBST, SearchData, SearchDirection};

#[derive(Clone)]
pub enum ETData<Data, Ref> {
    Node(Data),
    Edge {
        data: Data,
        /// Reference to matching edge
        other: Ref,
    },
}

impl<Data: std::fmt::Debug, InRef> std::fmt::Debug for ETData<Data, InRef> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ETData::Node(data) => write!(f, "Node({:?})", data),
            ETData::Edge { data, .. } => write!(f, ".{:?}.", data),
            //ETData::Edge { .. } => write!(f, "."),
        }
    }
}

impl<Data, Ref> ETData<Data, Ref> {
    pub fn data(&self) -> &Data {
        match self {
            ETData::Node(data) => data,
            ETData::Edge { data, .. } => data,
        }
    }
    pub fn data_mut(&mut self) -> &mut Data {
        match self {
            ETData::Node(data) => data,
            ETData::Edge { data, .. } => data,
        }
    }
    #[allow(dead_code)]
    fn unwrap_node(&self) -> &Data {
        match self {
            ETData::Node(data) => data,
            _ => panic!("Expected Node"),
        }
    }
    #[allow(dead_code)]
    fn unwrap_edge(&self) -> (&Data, &Ref) {
        match self {
            ETData::Edge { data, other } => (data, other),
            _ => panic!("Expected Edge"),
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
            data: Ag::from(data.data()),
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

impl<BST, Ag> EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    pub fn deb_ord(node: &Arc<BST>, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Ag::Data: Ord,
    {
        let r = node.root();
        let mut all_data: Vec<_> = (0..r.len())
            .filter_map(|i| match r.find_kth(i).node_data() {
                ETData::Node(d) => Some(d),
                _ => None,
            })
            .collect();
        all_data.sort();
        for d in all_data {
            write!(f, " {:?}", d)?;
        }
        Ok(())
    }
}

impl<BST, Ag> std::fmt::Debug for EulerTourTree<BST, Ag>
where
    BST: ImplicitBST<ETAggregated<Ag, Weak<BST>>>,
    Ag: AggregatedData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self.0.root();
        // write!(f, "Euler tour:")?;
        // for i in 0..r.len() {
        //     write!(f, " {:?}", r.find_kth(i).node_data())?;
        // }
        write!(f, "Nodes: ")?;
        for i in 0..r.len() {
            match r.find_kth(i).node_data() {
                ETData::Node(d) => write!(f, "{:?} ", d)?,
                _ => {}
            }
        }
        Ok(())
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
        EulerTourTree::reroot_raw(&self.0 .0);
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
        &self, // u
        node_w: &Self,
        uw_data: Ag::Data,
        wu_data: Ag::Data,
    ) -> Option<EdgeRef<EulerTourTree<BST, Ag>>> {
        if self.0 .0.on_same_tree(&node_w.0 .0) {
            // Already connected
            None
        } else {
            Self::reroot(node_w);
            Some(EulerTourTree::link_root(self, node_w, uw_data, wu_data))
        }
    }

    /// Finds an element in the tree containing this node.
    pub fn find_element(
        &self,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Arc<BST> {
        self.0 .0.root().find_element(|d| {
            search_strategy(SearchData {
                current_data: d.current_data.data(),
                left_agg: d.left_agg.data,
                right_agg: d.right_agg.data,
            })
        })
    }

    /// Number of nodes in the whole tree this node is contained in.
    pub fn tree_size(&self) -> usize {
        (self.0 .0.root().len() + 2) / 3
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
        let (a, b, _) = EulerTourTree::disconnect_raw(&self.0 .0, Some(self.1 .0.clone()));
        (NodeRef::from_bst(a), NodeRef::from_bst(b))
    }
    /// BST used to store the euler tour. Reference to the out edge.
    pub fn inner_bst(&self) -> [Arc<BST>; 2] {
        [self.0 .0.clone(), self.1 .0.clone()]
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
        if !node.is_first() {
            let (bef, aft, _) = node.split(node.order()..);
            aft.concat(&bef);
        }
    }
    /// Adds an edge between the root of self and the root of other. Panics if they are on the same tree.
    fn link_root(
        node_u: &NodeRef<Self>, // u
        root_w: &NodeRef<Self>, // w
        uw_data: Ag::Data,
        wu_data: Ag::Data,
    ) -> EdgeRef<Self> {
        assert!(!node_u.0 .0.on_same_tree(&root_w.0 .0));
        assert!(root_w.0 .0.is_root());
        let wu = BST::new(ETData::Edge {
            data: wu_data,
            other: Weak::new(),
        }); // wu
        let uw = BST::new(ETData::Edge {
            data: uw_data,
            other: Arc::downgrade(&wu),
        }); // uw
        wu.change_data(|data| {
            if let ETData::Edge { other, .. } = data {
                *other = Arc::downgrade(&uw);
            } else {
                alg_panic()
            }
        });
        Self::link_root_raw(&node_u.0 .0, &root_w.0 .0, &uw, &wu);
        EdgeRef::from_bst(uw, wu)
    }
    fn link_root_raw(
        u: &Arc<BST>, // u
        w: &Arc<BST>, // w
        uw: &Arc<BST>,
        wu: &Arc<BST>,
    ) {
        // "AAA u BBB" and "w CCC" (it is root) becomes
        // AAA u uw w CCC wu BBB
        let (_, until_u, after_u) = u.split(0..=u.order());
        until_u.concat(uw).concat(w).concat(wu).concat(&after_u);
    }
    /// Returns the first elements of each tree, which are the roots. And then the removed other_e.
    fn disconnect_raw(
        edge: &Arc<BST>,
        // hint, optional but makes it faster
        other_e: Option<Arc<BST>>,
    ) -> (Arc<BST>, Arc<BST>, Arc<BST>) {
        let other_e = other_e.unwrap_or_else(|| {
            if let ETData::Edge { other, .. } = edge.node_data() {
                or_alg_panic(other.upgrade())
            } else {
                alg_panic()
            }
        });
        let (a, b) = (edge.order(), other_e.order());
        let (left, middle, right) = edge.split(a.min(b)..=a.max(b));
        // Remove the first and last items, which is the edge which no longer exists
        let (_, middle, _) = middle.split(1..middle.len() - 1);
        assert_eq!(edge.root().len(), 1);
        assert_eq!(other_e.root().len(), 1);
        (left.concat(&right).first(), middle.first(), other_e)
    }
}
