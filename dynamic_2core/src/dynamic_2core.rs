use std::collections::{BTreeMap, BTreeSet};

use crate::{
    euler_tour_tree::{ETAggregated, ETData, EdgeRef, EulerTourTree, NodeRef},
    lists::{AggregatedData, Idx, Lists, SearchDirection},
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

type Level = usize;
type Node = usize;
type EdgeId = usize;
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Data {
    Node {
        idx: Node,
        /// Extra edges ON THIS LEVEL only
        extra_edges: usize,
    },
    Edge {
        e_id: EdgeId,
        // Level of this tree edge
        level: Level,
    },
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::Node {
                extra_edges: _,
                idx,
            } => write!(f, "({})", idx),
            Data::Edge { level, e_id: _ } => write!(f, "l{}", level),
        }
    }
}

impl Data {
    fn unwrap_node(&self) -> (&Node, &usize) {
        match self {
            Data::Node { idx, extra_edges } => (idx, extra_edges),
            _ => panic!("Expected Node"),
        }
    }
    fn unwrap_node_mut(&mut self) -> (&mut Node, &mut usize) {
        match self {
            Data::Node { idx, extra_edges } => (idx, extra_edges),
            _ => panic!("Expected Node"),
        }
    }
    fn unwrap_edge(&self) -> (&EdgeId, &Level) {
        match self {
            Data::Edge { e_id, level } => (e_id, level),
            _ => panic!("Expected Edge"),
        }
    }
    fn unwrap_edge_mut(&mut self) -> (&mut EdgeId, &mut Level) {
        match self {
            Data::Edge { e_id, level } => (e_id, level),
            _ => panic!("Expected Edge"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgData {
    /// Minimum level of edge in range
    min_edge_level: Level,
    /// Total extra edges in this level in this range
    total_extra_edges: usize,
}

impl Default for AgData {
    fn default() -> Self {
        Self {
            min_edge_level: usize::MAX,
            total_extra_edges: 0,
        }
    }
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

#[derive(Debug)]
struct EdgeInfo {
    /// u < v
    e: (Node, Node),
    /// Level of the edge
    level: Level,
    /// One reference for each level. If None, it is an extra edge.
    levels: Option<Vec<EdgeRef>>,
}

impl EdgeInfo {
    fn is_extra(&self) -> bool {
        self.levels.is_none()
    }
}

pub struct ETTSolver<L>
where
    L: Lists<ETAggregated<AgData>>,
{
    // We can make this a single ETT, but it's easier to debug this way
    ett: Vec<EulerTourTree<L, AgData>>,
    // lg levels
    levels: Vec<Vec<NodeRef>>,
    edge_info: Vec<EdgeInfo>,
    // (u, v) -> id
    e_to_id: BTreeMap<(Node, Node), usize>,
    /// Only exists for extra edges
    u_level_to_extras: BTreeMap<(Node, Level), BTreeSet<EdgeId>>,
}

impl<BST> std::fmt::Debug for ETTSolver<BST>
where
    BST: Lists<ETAggregated<AgData>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.dbg(f, 0, AllEdges)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DbgMode {
    NoEdges,
    AllEdges,
    TreeEdges,
}
use DbgMode::*;

struct Dbg<T>(T, Level, DbgMode);

impl<BST> std::fmt::Debug for Dbg<&ETTSolver<BST>>
where
    BST: Lists<ETAggregated<AgData>>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.dbg(f, self.1, self.2)
    }
}

impl<L> ETTSolver<L>
where
    L: Lists<ETAggregated<AgData>>,
{
    fn dbg(&self, f: &mut std::fmt::Formatter<'_>, i: Level, mode: DbgMode) -> std::fmt::Result {
        write!(f, "ETTSolver Level {}:", i)?;
        let l = self.ett[i].inner_lists();
        for x in self.levels[i].iter() {
            let idx = x.inner_idx();
            if l.root(idx) == idx {
                if l.len(idx) >= 1 {
                    write!(f, " [")?;
                    self.ett[i].deb_ord(idx, f)?;
                    write!(f, "]")?;
                }
                // TODO: Remove
                write!(f, " - {:?}", x)?;
            }
        }
        if mode != NoEdges {
            write!(f, ", edges:\n")?;
            let mut es = self
                .e_to_id
                .iter()
                .filter_map(|((u, v), e_id)| {
                    let info = &self.edge_info[*e_id];
                    if mode == TreeEdges && info.is_extra() {
                        None
                    } else {
                        Some((*u, *v, info.level, !info.is_extra()))
                    }
                })
                .collect::<Vec<_>>();
            es.sort_by_key(|(_, _, l, is_t)| (*l, !is_t));
            for (u, v, l, is_tree) in es {
                write!(
                    f,
                    "{} {} {}{}\n",
                    u,
                    v,
                    if is_tree && mode == AllEdges { "T" } else { "" },
                    l
                )?;
            }
        }
        Ok(())
    }

    fn assert_extra(&self, e_id: EdgeId, lvl: Level) {
        let ((u, v), e_lvl) = self.edge(e_id);
        assert!(self.edge_info[e_id].is_extra(), "edge is not extra");
        assert_eq!(e_lvl, lvl, "edge has wrong level");
        // assert!(
        //     self.levels[lvl][u].is_connected(&self.levels[lvl][v]),
        //     "extra edge but not connected"
        // );
        for u in [u, v] {
            assert!(
                self.u_level_to_extras[&(u, lvl)].contains(&e_id),
                "edge not in extra list"
            );
        }
    }
    fn assert_data(&self, node: Idx, lvl: Level) {
        if node == L::EMPTY {
            return;
        }
        use ETData::*;
        match self.ett[lvl].inner_lists().data(node) {
            Node(Data::Node { idx, extra_edges }) => {
                assert_eq!(
                    self.u_level_to_extras
                        .get(&(*idx, lvl))
                        .map_or(0, BTreeSet::len),
                    *extra_edges,
                    "wrong extra edge count for {idx} at l{lvl}"
                );
            }
            Edge {
                data: Data::Edge { e_id, level },
                other: _,
            } => {
                assert!(*level >= lvl, "tree edge has level smaller than ETT level");
                assert_eq!(
                    self.edge_info[*e_id].level, *level,
                    "tree edge has diff level in info and data"
                );
                assert!(!self.edge_info[*e_id].is_extra(), "tree edge is extra");
            }
            _ => panic!("Invalid data"),
        }
    }
    fn find_level_i_tree_edge(&self, i: Level, u: NodeRef) -> Option<EdgeId> {
        // log::trace!("Looking for tree edge at level {}", i);
        let found = self.ett[i].find_element(u, |d| {
            // log::trace!("Checking {:?}", d);
            if matches!(d.current_data, Data::Edge { level, .. } if *level == i) {
                // println!("edge level {}", d.current_data.unwrap_edge().1);
                SearchDirection::Found
            } else if d.left_agg.min_edge_level <= i {
                SearchDirection::Left
            } else if d.right_agg.min_edge_level <= i {
                SearchDirection::Right
            } else {
                SearchDirection::NotFound
            }
        });
        if found != L::EMPTY {
            let (id, _) = self.ett[i].inner_lists().data(found).data().unwrap_edge();
            // log::trace!("Found edge {} at level {}", id, i);
            self.assert_data(found, i);
            return Some(*id);
        }
        None
    }
    fn find_level_i_extra_edge(&self, i: Level, u: NodeRef) -> Option<EdgeId> {
        let found = self.ett[i].find_element(u, |d| {
            if matches!(d.current_data, Data::Node { extra_edges, .. } if *extra_edges > 0) {
                SearchDirection::Found
            } else if d.left_agg.total_extra_edges > 0 {
                SearchDirection::Left
            } else if d.right_agg.total_extra_edges > 0 {
                SearchDirection::Right
            } else {
                SearchDirection::NotFound
            }
        });
        if found != L::EMPTY {
            self.assert_data(found, i);
            let (u, _) = self.ett[i].inner_lists().data(found).data().unwrap_node();
            let id = self.u_level_to_extras[&(*u, i)]
                .first()
                .expect("missing extra edge");
            self.assert_extra(*id, i);
            return Some(*id);
        }
        None
    }
    fn extra_edges_mut(&mut self, u: Node, lvl: Level) -> &mut usize {
        self.ett[lvl]
            .data_mut(self.levels[lvl][u])
            .unwrap_node_mut()
            .1
    }
    // Does not affect the Data::Edge.levels field
    fn add_edge_id(&mut self, e_id: EdgeId) {
        let ((u, v), lvl) = self.edge(e_id);
        assert!(self.e_to_id.insert((u, v), e_id).is_none());
        if self.edge_info[e_id].is_extra() {
            for w in [u, v] {
                assert!(self
                    .u_level_to_extras
                    .entry((w, lvl))
                    .or_default()
                    .insert(e_id));
                *self.extra_edges_mut(w, lvl) += 1;
                if w == 10 {
                    log::info!(
                        "[l{lvl}] adding edge {e_id} ({u}, {v}) to {w} ({} == {:?})",
                        self.ett[lvl].data(self.levels[lvl][w]).unwrap_node().1,
                        self.u_level_to_extras[&(w, lvl)],
                    );
                }
            }
            self.assert_extra(e_id, lvl);
        }
    }
    /// Does not affect the Data::Edge.levels field
    fn rem_edge_id(&mut self, e_id: EdgeId) {
        let ((u, v), lvl) = self.edge(e_id);
        assert!(self.e_to_id.remove(&self.edge_info[e_id].e).is_some());
        if self.edge_info[e_id].is_extra() {
            for w in [u, v] {
                assert!(self
                    .u_level_to_extras
                    .get_mut(&(w, lvl))
                    .unwrap()
                    .remove(&e_id));
                *self.extra_edges_mut(w, lvl) -= 1;
                if w == 10 {
                    log::info!(
                        "[l{lvl}] removing edge {e_id} ({u}, {v}) from {w} ({} == {:?})",
                        self.ett[lvl].data(self.levels[lvl][w]).unwrap_node().1,
                        self.u_level_to_extras[&(w, lvl)],
                    );
                }
            }
        }
    }
    fn add_level_to_edge(&mut self, e_id: EdgeId) {
        let ((u, v), lvl) = self.edge(e_id);
        self.rem_edge_id(e_id);
        self.edge_info[e_id].level = lvl + 1;
        self.add_edge_id(e_id);
        if let Some(levels) = &mut self.edge_info[e_id].levels {
            for (elvl, e, dir) in levels
                .iter()
                .enumerate()
                .flat_map(|(elvl, e)| [(elvl, e, false), (elvl, e, true)])
            {
                *self.ett[elvl].edata_mut(*e, dir).unwrap_edge_mut().1 = lvl + 1;
            }
            let e = Data::Edge {
                level: lvl + 1,
                e_id,
            };
            levels.push(
                self.ett[lvl + 1]
                    .connect(
                        self.levels[lvl + 1][u],
                        self.levels[lvl + 1][v],
                        e.clone(),
                        e,
                    )
                    .expect("shouldn't be connected at next level"),
            );
            assert_eq!(levels.len(), lvl + 2, "edge has wrong number of levels");
        } else {
            self.assert_extra(e_id, lvl + 1);
            assert!(
                self.ett[lvl + 1].is_connected(self.levels[lvl + 1][u], self.levels[lvl + 1][v]),
                "extra edge but not connected"
            );
        }
    }
    fn edge(&self, e_id: EdgeId) -> ((Node, Node), Level) {
        (self.edge_info[e_id].e, self.edge_info[e_id].level)
    }
}

impl<BST> Dynamic2CoreSolver for ETTSolver<BST>
where
    BST: Lists<ETAggregated<AgData>>,
{
    fn new(n: usize) -> Self {
        let log2n = (n.next_power_of_two().trailing_zeros() as usize) + 1;
        let mut ett = (0..log2n)
            .map(|_| EulerTourTree::new(n))
            .collect::<Vec<_>>();
        let levels = (0..log2n)
            .map(|lvl| {
                (0..n)
                    .map(|idx| {
                        ett[lvl].create_node(Data::Node {
                            extra_edges: 0,
                            idx,
                        })
                    })
                    .collect()
            })
            .collect();
        Self {
            ett,
            levels,
            edge_info: Vec::new(),
            e_to_id: BTreeMap::new(),
            u_level_to_extras: BTreeMap::new(),
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) -> bool {
        if u > v {
            return self.add_edge(v, u);
        }
        if u == v || self.e_to_id.contains_key(&(u, v)) {
            return false;
        }
        let e_id = self.edge_info.len();
        let e = Data::Edge { level: 0, e_id };
        let added = self.ett[0].connect(self.levels[0][u], self.levels[0][v], e.clone(), e);
        self.edge_info.push(EdgeInfo {
            e: (u, v),
            level: 0,
            levels: added.map(|e| vec![e]),
        });
        self.add_edge_id(e_id);
        true
    }

    // test failing
    fn remove_edge(&mut self, u: usize, v: usize) -> bool {
        if u > v {
            return self.remove_edge(v, u);
        }
        let e_id = if let Some(id) = self.e_to_id.get(&(u, v)) {
            *id
        } else {
            return false;
        };
        if let Some(levels) = self.edge_info[e_id].levels.clone() {
            log::trace!(
                "Removing tree edge {} = ({}, {}) at level {}",
                e_id,
                u,
                v,
                self.edge_info[e_id].level
            );
            let smallest_comp: Vec<_> = levels
                .into_iter()
                .enumerate()
                .map(|(lvl, e)| {
                    let ett = &self.ett[lvl];
                    log::trace!(
                        "[lvl {lvl}] before: {:?} b {:?}",
                        Dbg(self as &_, lvl, AllEdges),
                        e
                    );
                    assert!(ett.is_connected(self.levels[lvl][u], self.levels[lvl][v]));
                    let (tu, tv) = self.ett[lvl].disconnect(e);
                    let ett = &self.ett[lvl];
                    log::trace!(
                        "[lvl {lvl}] after: {:?} u {:?} v {:?}",
                        Dbg(self as &_, lvl, NoEdges),
                        tu,
                        tv
                    );
                    assert!(!ett.is_connected(tu, tv));
                    assert!(!ett.is_connected(self.levels[lvl][u], self.levels[lvl][v]));
                    self.assert_data(tu.inner_idx(), lvl);
                    self.assert_data(tv.inner_idx(), lvl);
                    if ett.tree_size(tu) < ett.tree_size(tv) {
                        tu
                    } else {
                        tv
                    }
                })
                .collect();
            self.rem_edge_id(e_id);

            for (i, small) in smallest_comp.into_iter().enumerate().rev() {
                log::trace!("Looking for edge at level {}: smol {:?}", i, small);
                // Move all tree edges of level i to i + 1
                while let Some(f_id) = self.find_level_i_tree_edge(i, small) {
                    assert!(!self.edge_info[f_id].is_extra(), "tree edge is extra");
                    assert_eq!(self.edge_info[f_id].level, i, "edge has wrong level");
                    log::trace!(
                        "Tree edge {:?} at level {} will move",
                        self.edge_info[f_id].e,
                        i
                    );
                    self.add_level_to_edge(f_id);
                }
                log::trace!("After tree edges pushed, smol: {:?}", small);
                // For all extra edges of level i, check if they replace the removed edge, and move them to level i + 1
                while let Some(f_id) = self.find_level_i_extra_edge(i, small) {
                    let (a, b) = self.edge_info[f_id].e;
                    if !self.ett[i].is_connected(self.levels[i][a], self.levels[i][b]) {
                        log::trace!("Extra edge ({}, {}) at level {} will replace", a, b, i);
                        self.rem_edge_id(f_id);
                        let mut rs = vec![];
                        // This is a replacement edge, add it to the tree in this and previous levels, then exit.
                        let e = Data::Edge {
                            level: i,
                            e_id: f_id,
                        };
                        for j in (0..=i).rev() {
                            assert!(!self.ett[j].is_connected(self.levels[j][u], self.levels[j][v]));
                            assert!(
                                !self.ett[j].is_connected(self.levels[j][a], self.levels[j][b]),
                                "({}, {}) shouldn't be connected at level {}: {:?}",
                                a,
                                b,
                                j,
                                self
                            );
                            let r = self.ett[j]
                                .connect(self.levels[j][a], self.levels[j][b], e.clone(), e.clone())
                                .expect("shouldn't be connected at previous level");
                            log::trace!("[lvl {}] after: {:?}", j, Dbg(self as &_, j, NoEdges));
                            assert!(self.ett[j].is_connected(self.levels[j][a], self.levels[j][b]));
                            assert!(
                                self.ett[j].is_connected(self.levels[j][u], self.levels[j][v]),
                                "({}, {}) should be connected at level {}: {:?}",
                                u,
                                v,
                                j,
                                Dbg(self as &_, j, AllEdges),
                            );
                            rs.push(r);
                        }
                        assert!(self.edge_info[f_id].levels.is_none());
                        self.edge_info[f_id].levels = Some(rs);
                        self.add_edge_id(f_id);
                        return true;
                    }
                    log::trace!("Extra edge ({}, {}) at level {} will move", a, b, i);
                    self.add_level_to_edge(f_id);
                }
            }
        } else {
            self.rem_edge_id(e_id);
        }
        // TODO swap with last to save space. May be tricky to keep all indices
        true
    }

    fn is_connected(&self, u: usize, v: usize) -> bool {
        self.ett[0].is_connected(self.levels[0][u], self.levels[0][v])
    }

    fn is_in_2core(&self, u: usize) -> bool {
        todo!()
    }

    fn is_in_1core(&self, u: usize) -> bool {
        let u = self.levels[0][u];
        // Definitely can be more efficient, O(1), but this works
        self.ett[0].tree_size(u) > 1
    }
}
