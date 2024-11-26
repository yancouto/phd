use std::{
    collections::{btree_map::Entry, BTreeMap},
    sync::Weak,
};

use crate::{
    euler_tour_tree::{ETAggregated, EdgeRef, EulerTourTree, NodeRef},
    implicit_bst::{AggregatedData, ImplicitBST, SearchDirection},
};

pub trait Dynamic2CoreSolver {
    /// New instance for an empty graph on n nodes
    fn new(n: usize) -> Self;
    /// Add an edge between u and v. Returns whether is was added.
    fn add_edge(&mut self, u: usize, v: usize) -> bool;
    /// Remove an edge between u and v. Returns whether it was removed.
    fn remove_edge(&mut self, u: usize, v: usize) -> bool;
    /// Check if u and v are connected.
    fn is_connected(&self, u: usize, v: usize) -> bool;
    /// Check if u is in the 2-core.
    fn is_in_2core(&self, u: usize) -> bool;
    /// Check if u is in the 1-core.
    fn is_in_1core(&self, u: usize) -> bool;
}

#[derive(Debug, Clone)]
pub enum Data {
    Node {
        /// Extra edges ON THIS LEVEL only
        extra_edges: usize,
        idx: usize,
    },
    Edge {
        // Level of this tree edge
        level: usize,
        e_id: usize,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AgData {
    /// Minimum level of edge in range
    min_edge_level: usize,
    /// Total extra edges in this level in this range
    total_extra_edges: usize,
}

impl AggregatedData for AgData {
    type Data = Data;
    fn from(data: &Self::Data) -> Self {
        match data {
            Data::Node {
                extra_edges,
                idx: _,
            } => Self {
                total_extra_edges: *extra_edges,
                min_edge_level: usize::MAX,
            },
            Data::Edge { level, e_id: _ } => Self {
                min_edge_level: *level,
                total_extra_edges: 0,
            },
        }
    }
    fn merge(self, right: Self) -> Self {
        Self {
            min_edge_level: self.min_edge_level.min(right.min_edge_level),
            total_extra_edges: self.total_extra_edges + right.total_extra_edges,
        }
    }
}

struct EdgeInfo<BST>
where
    BST: ImplicitBST<ETAggregated<AgData, Weak<BST>>>,
{
    /// u < v
    e: (usize, usize),
    /// Level of the edge
    level: usize,
    /// One reference for each level. If None, it is an extra edge.
    levels: Option<Vec<EdgeRef<EulerTourTree<BST, AgData>>>>,
}

impl<BST> EdgeInfo<BST>
where
    BST: ImplicitBST<ETAggregated<AgData, Weak<BST>>>,
{
    fn add_level(&mut self, solver_levels: &Vec<Vec<NodeRef<EulerTourTree<BST, AgData>>>>) {
        let (u, v) = self.e;
        self.level += 1;
        if let Some(levels) = &mut self.levels {
            let mut e_id = None;
            for r in levels.iter() {
                r.inner_bst().change_data(|d| {
                    if let Some(Data::Edge { e_id: id, level }) = d.data_mut() {
                        e_id = Some(*id);
                        *level += 1;
                    } else {
                        panic!("Algorithm error: found a node that is not an edge");
                    }
                });
            }
            levels.push(
                solver_levels[self.level][u]
                    .connect(
                        &solver_levels[self.level][v],
                        Data::Edge {
                            level: self.level,
                            e_id: e_id.expect("must have found id"),
                        },
                    )
                    .expect("shouldn't be connected at next level"),
            );
        }
    }
}

pub struct ETTSolver<BST>
where
    BST: ImplicitBST<ETAggregated<AgData, Weak<BST>>>,
{
    // lg levels
    levels: Vec<Vec<NodeRef<EulerTourTree<BST, AgData>>>>,
    edge_info: Vec<EdgeInfo<BST>>,
    // (u, v) -> id
    e_to_id: BTreeMap<(usize, usize), usize>,
}

impl<BST> ETTSolver<BST>
where
    BST: ImplicitBST<ETAggregated<AgData, Weak<BST>>>,
{
    fn find_level_i_tree_edge(
        &self,
        i: usize,
        node: &NodeRef<EulerTourTree<BST, AgData>>,
    ) -> Option<usize> {
        assert!(node.inner_bst().is_root());
        let found = node.find_element(
            |d| {
                if matches!(d.current_data, Data::Edge { level, .. } if *level == i) {
                    SearchDirection::Found
                } else if d.left_agg.min_edge_level <= i {
                    SearchDirection::Left
                } else if d.right_agg.min_edge_level <= i {
                    SearchDirection::Right
                } else {
                    SearchDirection::NotFound
                }
            },
            Data::Node {
                extra_edges: 0,
                idx: usize::MAX,
            },
        );
        if !found.is_empty() {
            if let Some(Data::Edge { e_id, .. }) = found.node_data().data() {
                return Some(*e_id);
            }
            panic!("Algorithm error: found a node that is not an edge");
        }
        None
    }
    fn find_level_i_extra_edge(
        &self,
        _i: usize,
        node: &NodeRef<EulerTourTree<BST, AgData>>,
    ) -> Option<usize> {
        assert!(node.inner_bst().is_root());
        let found = node.find_element(
            |d| {
                if matches!(d.current_data, Data::Node { extra_edges, .. } if *extra_edges > 0) {
                    SearchDirection::Found
                } else if d.left_agg.total_extra_edges > 0 {
                    SearchDirection::Left
                } else if d.right_agg.total_extra_edges > 0 {
                    SearchDirection::Right
                } else {
                    SearchDirection::NotFound
                }
            },
            Data::Node {
                extra_edges: 0,
                idx: usize::MAX,
            },
        );
        if !found.is_empty() {
            if let Some(Data::Node { idx, .. }) = found.node_data().data() {
                return Some(*idx);
            }
            panic!("Algorithm error: found a node that is not a node");
        }
        None
    }
}

impl<BST> Dynamic2CoreSolver for ETTSolver<BST>
where
    BST: ImplicitBST<ETAggregated<AgData, Weak<BST>>>,
{
    fn new(n: usize) -> Self {
        let log2n = (n.next_power_of_two().trailing_zeros() as usize) + 1;
        Self {
            levels: (0..log2n)
                .map(|_| {
                    (0..n)
                        .map(|idx| {
                            EulerTourTree::new(Data::Node {
                                extra_edges: 0,
                                idx,
                            })
                        })
                        .collect()
                })
                .collect(),
            edge_info: Vec::new(),
            e_to_id: BTreeMap::new(),
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) -> bool {
        if u > v {
            return self.add_edge(v, u);
        }
        let entry = self.e_to_id.entry((u, v));
        if u == v || matches!(entry, Entry::Occupied(_)) {
            return false;
        }
        let e_id = entry.or_insert(self.edge_info.len());
        let added = self.levels[0][u].connect(
            &self.levels[0][v],
            Data::Edge {
                level: 0,
                e_id: *e_id,
            },
        );
        self.edge_info.push(EdgeInfo {
            e: (u, v),
            level: 0,
            levels: added.map(|e| vec![e]),
        });
        true
    }

    fn remove_edge(&mut self, u: usize, v: usize) -> bool {
        if let Some(id) = self.e_to_id.remove(&(u, v)) {
            if let Some(levels) = self.edge_info[id].levels.take() {
                for (i, e) in levels.iter().enumerate().rev() {
                    let (tu, tv) = e.disconnect();
                    let small = if tu.subtree_size() < tv.subtree_size() {
                        tu
                    } else {
                        tv
                    };
                    // Move all tree edges of level i to i + 1
                    while let Some(f_id) = self.find_level_i_tree_edge(i, &small) {
                        self.edge_info[f_id].add_level(&self.levels);
                    }
                    // For all extra edges of level i, check if they replace the removed edge, and move them to level i + 1
                    while let Some(f_id) = self.find_level_i_extra_edge(i, &small) {
                        let (a, b) = self.edge_info[f_id].e;
                        if !self.levels[i][a].is_connected(&self.levels[i][b]) {
                            // This is a replacement edge, add it to the tree in this and previous levels, then exit.
                            for j in (0..=i).rev() {
                                self.levels[j][a]
                                    .connect(
                                        &self.levels[j][b],
                                        Data::Edge {
                                            level: i,
                                            e_id: f_id,
                                        },
                                    )
                                    .expect("shouldn't be connected at previous level");
                            }
                            return true;
                        }
                        self.edge_info[f_id].add_level(&self.levels);
                    }
                }
            }
            // TODO swap with last to save space
            true
        } else {
            false
        }
    }

    fn is_connected(&self, u: usize, v: usize) -> bool {
        self.levels[0][u].is_connected(&self.levels[0][v])
    }

    fn is_in_2core(&self, u: usize) -> bool {
        todo!()
    }

    fn is_in_1core(&self, u: usize) -> bool {
        todo!()
    }
}
