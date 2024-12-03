use std::{fmt::Debug, marker::PhantomData};

use crate::lists::{AggregatedData, Idx, Lists, SearchData, SearchDirection};

#[derive(Clone)]
pub enum ETData<Data> {
    Node(Data),
    Edge {
        data: Data,
        /// Reference to matching edge
        other: Idx,
    },
}

impl<Data: std::fmt::Debug> std::fmt::Debug for ETData<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ETData::Node(data) => write!(f, "Node({:?})", data),
            ETData::Edge { data, .. } => write!(f, ".{:?}.", data),
            //ETData::Edge { .. } => write!(f, "."),
        }
    }
}

impl<Data> ETData<Data> {
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
    fn unwrap_edge(&self) -> (&Data, &Idx) {
        match self {
            ETData::Edge { data, other } => (data, other),
            _ => panic!("Expected Edge"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ETAggregated<Ag: AggregatedData> {
    data: Ag,
    subtree_size: usize,
}

impl<Ag: AggregatedData> Default for ETAggregated<Ag> {
    fn default() -> Self {
        Self {
            data: Ag::default(),
            subtree_size: 0,
        }
    }
}

impl<Ag: AggregatedData> AggregatedData for ETAggregated<Ag> {
    type Data = ETData<Ag::Data>;
    fn from(data: &Self::Data) -> Self {
        Self {
            data: Ag::from(data.data()),
            subtree_size: matches!(data, ETData::Node(_)).into(),
        }
    }
    fn merge(self, right: Self) -> Self {
        Self {
            data: self.data.merge(right.data),
            subtree_size: self.subtree_size + right.subtree_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NodeRef(Idx);
// Edges will be idx and idx + 1
#[derive(Debug, Clone, Copy)]
pub struct EdgeRef(Idx);

pub struct EulerTourTree<L, Ag>
where
    L: Lists<ETAggregated<Ag>>,
    Ag: AggregatedData,
{
    l: L,
    _phantom: PhantomData<Ag>,
}

impl<BST, Ag> std::fmt::Debug for EulerTourTree<BST, Ag>
where
    BST: Lists<ETAggregated<Ag>>,
    Ag: AggregatedData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nodes: ")?;
        for u in 0..self.l.total_size() {
            if self.l.root(u) == u {
                write!(f, "<")?;
                for i in 0..self.l.len(u) {
                    let j = self.l.find_kth(u, i);
                    match self.l.data(j) {
                        ETData::Node(d) => write!(f, "{j}{d:?} ")?,
                        ETData::Edge { data, other } => write!(f, "{j}[{data:?}] ")?,
                    }
                }
                write!(f, "> ")?;
            }
        }

        Ok(())
    }
}

impl NodeRef {
    /// Inner index for the node
    pub fn inner_idx(&self) -> Idx {
        self.0
    }
}

impl EdgeRef {
    /// Inner indices for the two direction of the edge
    pub fn inner_idx(&self) -> [Idx; 2] {
        [self.0, self.0 + 1]
    }
}

impl<L, Ag> EulerTourTree<L, Ag>
where
    L: Lists<ETAggregated<Ag>>,
    Ag: AggregatedData,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            l: L::new(capacity),
            _phantom: PhantomData,
        }
    }

    pub fn create_node(&mut self, node_data: Ag::Data) -> NodeRef {
        NodeRef(self.l.create(ETData::Node(node_data)))
    }

    /// Makes the given node the root of its tree.
    pub fn reroot(&mut self, u: NodeRef) {
        if !self.l.is_first(u.0) {
            let (bef, aft, _) = self.l.split(u.0, self.l.order(u.0)..);
            self.l.concat(aft, bef);
        }
    }
    /// Adds an edge between the root of self and the root of other. Panics if they are on the same tree.
    fn link_root(
        &mut self,
        u: NodeRef,
        root_w: NodeRef, // w
        uw_data: Ag::Data,
        wu_data: Ag::Data,
    ) -> EdgeRef {
        assert!(!self.l.on_same_list(u.0, root_w.0));
        assert!(self.l.is_root(root_w.0));
        let mx = self.l.total_size();
        let uw = self.l.create(ETData::Edge {
            data: uw_data,
            other: mx + 1,
        }); // uw
        let wu = self.l.create(ETData::Edge {
            data: wu_data,
            other: uw,
        }); // wu
        assert_eq!(uw, mx);
        assert_eq!(wu, mx + 1);

        // "AAA u BBB" and "w CCC" (it is root) becomes
        // AAA u uw w CCC wu BBB
        let (_, until_u, after_u) = self.l.split(u.0, 0..=self.l.order(u.0));
        self.l.concat_all([until_u, uw, root_w.0, wu, after_u]);
        EdgeRef(uw)
    }
    /// Remove the edge and return the root of the current tree and then the root of the new tree the edge removal created.
    pub fn disconnect(&mut self, edge: EdgeRef) -> (NodeRef, NodeRef) {
        let (edge, other_e) = (edge.0, edge.0 + 1);
        assert!(
            self.l.on_same_list(edge, other_e),
            "edge {edge} {other_e} must be connected {self:?}"
        );
        let (a, b) = (self.l.order(edge), self.l.order(other_e));
        let (left, middle, right) = self.l.split(edge, a.min(b)..=a.max(b));
        // Remove the first and last items, which is the edge which no longer exists
        let (_, middle, _) = self.l.split(middle, 1..self.l.len(middle) - 1);
        assert_eq!(self.l.len(edge), 1);
        assert_eq!(self.l.len(other_e), 1);
        let rest = self.l.concat(left, right);
        (NodeRef(self.l.first(rest)), NodeRef(self.l.first(middle)))
    }
    pub fn is_connected(&self, u: NodeRef, v: NodeRef) -> bool {
        self.l.on_same_list(u.0, v.0)
    }

    /// Connects the two nodes with an edge. The root of the first tree remais the root. Returns None if they are already connected.
    pub fn connect(
        &mut self,
        u: NodeRef,
        w: NodeRef,
        uw_data: Ag::Data,
        wu_data: Ag::Data,
    ) -> Option<EdgeRef> {
        if self.l.on_same_list(u.0, w.0) {
            // Already connected
            None
        } else {
            self.reroot(w);
            Some(self.link_root(u, w, uw_data, wu_data))
        }
    }

    /// Finds an element in the tree containing this node. Returns the inner idx to be used with self.inner_lists.
    pub fn find_element(
        &self,
        u: NodeRef,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
        self.l.find_element(u.0, |d| {
            search_strategy(SearchData {
                current_data: d.current_data.data(),
                left_agg: &d.left_agg.data,
                right_agg: &d.right_agg.data,
            })
        })
    }

    /// Number of nodes in the whole tree this node is contained in.
    pub fn tree_size(&self, u: NodeRef) -> usize {
        (self.l.len(u.0) + 2) / 3
    }

    /// Lists structure for this euler tour tree.
    pub fn inner_lists(&self) -> &L {
        &self.l
    }

    pub fn data(&self, u: NodeRef) -> &Ag::Data {
        self.l.data(u.0).data()
    }

    pub fn data_mut(&mut self, u: NodeRef) -> &mut Ag::Data {
        self.l.data_mut(u.0).data_mut()
    }

    pub fn edata(&self, e: EdgeRef) -> [&Ag::Data; 2] {
        [self.l.data(e.0).data(), self.l.data(e.0 + 1).data()]
    }

    pub fn edata_mut(&mut self, e: EdgeRef, direction: bool) -> &mut Ag::Data {
        self.l.data_mut(e.0 + (direction as usize)).data_mut()
    }

    pub fn try_node(&self, u: Idx) -> Option<NodeRef> {
        if matches!(self.l.data(u), ETData::Node(_)) {
            Some(NodeRef(u))
        } else {
            None
        }
    }

    pub fn deb_ord(&self, u: Idx, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    where
        Ag::Data: Ord,
    {
        let r = self.l.root(u);
        let mut all_data: Vec<_> = (0..self.l.len(r))
            .filter_map(|i| match self.l.data(self.l.find_kth(r, i)) {
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
