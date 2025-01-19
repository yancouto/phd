//! Euler Tour Tree implementation, with custom aggregated data.

use std::{fmt::Debug, marker::PhantomData};

use crate::lists::{AggregatedData, Idx, Lists, SearchData, SearchDirection};

// Edges will be idx and idx + 1
#[derive(Debug, Clone, Copy)]
pub struct EdgeRef(Idx);

/// Interface of an Euler Tour Tree
/// It maintains a collection of euler tours on a forest of trees. Each node and edge might have associated data, which can be aggregated.
pub trait EulerTourTree<Ag: AggregatedData> {
    const EMPTY: Idx;
    /// Creates a new Euler Tour Tree with nodes given by the data and no edges.
    fn new(node_data: Vec<Ag::Data>) -> Self;
    /// Makes the given node the root of its tree.
    fn reroot(&mut self, u: Idx);
    /// Returns the root of the euler tour tree containing u.
    fn root(&self, u: Idx) -> Idx;
    /// Remove the edge and return the root of the current tree and then the root of the new tree the edge removal created.
    fn disconnect(&mut self, edge: EdgeRef) -> (Idx, Idx);
    /// Connects the two nodes with an edge. The root of the first tree remais the root. Returns None if they are already connected.
    fn connect(&mut self, u: Idx, w: Idx, uw_data: Ag::Data, wu_data: Ag::Data) -> Option<EdgeRef>;
    fn is_connected(&self, u: Idx, v: Idx) -> bool;
    /// Number of nodes in the whole tree this node is contained in.
    fn tree_size(&self, u: Idx) -> usize;
    /// Finds an element in the tree containing this node and return it. It may be a node or an edge.
    fn find_element(
        &self,
        u: Idx,
        search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx;
    /// Returns data of the node. Can be used for normal nodes, or from Idx of edges returned by find_element.
    fn data(&self, u: Idx) -> &Ag::Data;
    /// Modifies the data on a given node
    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data));
    /// Returns the data of the edge.
    fn edata(&self, e: EdgeRef) -> [&Ag::Data; 2];
    /// Modifies the data of the edge. The direction is given by a boolean.
    fn mutate_edata(&mut self, e: EdgeRef, direction: bool, f: impl FnOnce(&mut Ag::Data));
}

pub struct ETT<L, Ag>
where
    L: Lists<Ag>,
    Ag: AggregatedData,
{
    l: L,
    _phantom: PhantomData<Ag>,
}

impl<BST, Ag> std::fmt::Debug for ETT<BST, Ag>
where
    BST: Lists<Ag>,
    Ag: AggregatedData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nodes: ")?;
        for u in 0..self.l.total_size() {
            if self.l.root(u) == u {
                write!(f, "<")?;
                for i in 0..self.l.len(u) {
                    let j = self.l.find_kth(u, i);
                    write!(f, "{j}{d:?} ", d = self.l.data(j))?;
                }
                write!(f, "> ")?;
            }
        }

        Ok(())
    }
}

impl EdgeRef {
    /// Inner indices for the two direction of the edge
    pub fn inner_idx(&self) -> [Idx; 2] {
        [self.0, self.0 + 1]
    }
}

impl<L, Ag> ETT<L, Ag>
where
    L: Lists<Ag>,
    Ag: AggregatedData,
{
    /// Adds an edge between the root of self and the root of other. Panics if they are on the same tree.
    fn link_root(
        &mut self,
        u: Idx,
        root_w: Idx, // w
        uw_data: Ag::Data,
        wu_data: Ag::Data,
    ) -> EdgeRef {
        debug_assert!(!self.l.on_same_list(u, root_w));
        debug_assert!(self.l.is_first(root_w));
        let uw = self.l.create(uw_data); // uw
        let wu = self.l.create(wu_data); // wu

        // "AAA u BBB" and "w CCC" (it is root) becomes
        // AAA u uw w CCC wu BBB
        let (_, until_u, after_u) = self.l.split(u, 0..=self.l.order(u));
        self.l.concat_all([until_u, uw, root_w, wu, after_u]);
        EdgeRef(uw)
    }
    pub fn inner_lists(&self) -> &L {
        &self.l
    }
}

impl<L, Ag> EulerTourTree<Ag> for ETT<L, Ag>
where
    L: Lists<Ag>,
    Ag: AggregatedData,
{
    const EMPTY: Idx = L::EMPTY;
    fn new(node_data: Vec<Ag::Data>) -> Self {
        let mut l = L::new(node_data.len());
        for (i, data) in node_data.into_iter().enumerate() {
            assert_eq!(l.create(data), i);
        }
        Self {
            l,
            _phantom: PhantomData,
        }
    }
    fn reroot(&mut self, u: Idx) {
        if !self.l.is_first(u) {
            let (before_u, u_and_after, _) = self.l.split(u, self.l.order(u)..);
            self.l.concat(u_and_after, before_u);
        }
    }
    fn root(&self, u: Idx) -> Idx {
        self.l.first(u)
    }
    fn disconnect(&mut self, edge: EdgeRef) -> (Idx, Idx) {
        let (edge, other_e) = (edge.0, edge.0 + 1);
        debug_assert!(self.l.on_same_list(edge, other_e));
        let (a, b) = (self.l.order(edge), self.l.order(other_e));
        let (left, middle, right) = self.l.split(edge, a.min(b)..=a.max(b));
        // Remove the first and last items, which is the edge which no longer exists
        let (_, middle, _) = self.l.split(middle, 1..self.l.len(middle) - 1);
        debug_assert_eq!(self.l.len(edge), 1);
        debug_assert_eq!(self.l.len(other_e), 1);
        let rest = self.l.concat(left, right);
        (self.l.first(rest), self.l.first(middle))
    }
    fn is_connected(&self, u: Idx, v: Idx) -> bool {
        self.l.on_same_list(u, v)
    }
    fn connect(&mut self, u: Idx, w: Idx, uw_data: Ag::Data, wu_data: Ag::Data) -> Option<EdgeRef> {
        if self.l.on_same_list(u, w) {
            // Already connected
            None
        } else {
            self.reroot(w);
            Some(self.link_root(u, w, uw_data, wu_data))
        }
    }

    fn find_element(
        &self,
        u: Idx,
        search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
        self.l.find_element(u, search_strategy)
    }
    fn data(&self, u: Idx) -> &Ag::Data {
        self.l.data(u)
    }

    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data)) {
        self.l.mutate_data(u, f)
    }

    fn edata(&self, e: EdgeRef) -> [&Ag::Data; 2] {
        [self.l.data(e.0), self.l.data(e.0 + 1)]
    }

    fn mutate_edata(&mut self, e: EdgeRef, direction: bool, f: impl FnOnce(&mut Ag::Data)) {
        self.l.mutate_data(e.0 + (direction as usize), f)
    }
    fn tree_size(&self, u: Idx) -> usize {
        (self.l.len(u) + 2) / 3
    }
}
