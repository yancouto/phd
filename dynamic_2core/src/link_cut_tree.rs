//! Link Cut Tree implementation, without aggregated data.

use crate::lists::Lists;

pub type Node = usize;

/// Interface of a Link Cut Tree.
/// It maintains a collection of trees dynamically. This implementation doesn't have any data associated with the nodes.
pub trait LinkCutTree {
    /// Create a new LinkCutTree with n vertices and no edges.
    fn new(n: usize) -> Self;
    /// Returns the root of the tree containing u.
    fn root(&mut self, u: Node) -> Node;
    /// Adds an edge between u and v. Returns false if they were in the same tree.
    /// Reroots v, and keeps the root of the tree containing u the same.
    fn link(&mut self, u: Node, v: Node) -> bool;
    /// Cuts u from its immediate parent. Returns the parent of u.
    fn cut(&mut self, u: Node) -> Option<Node>;
    /// Makes u the root of its current tree.
    fn reroot(&mut self, u: Node);
    /// The lowest common ancestor of u and v. None if they are in different trees.
    fn lca(&mut self, u: Node, v: Node) -> Option<Node>;
}

#[derive(Debug)]
pub struct LCT<L>
where
    L: Lists<()>,
{
    l: L,
    // Non-EMPTY iff the node is a root of a preferred path that is not the topmost.
    parent: Vec<usize>,
}

impl<L> LCT<L>
where
    L: Lists<()>,
{
    /// Returns the point where the access operation entered the topmost preferred path.
    /// That is, returns the LCA of u with the last node that called access.
    fn access(&mut self, mut u: Node) -> Node {
        let mut prev_topmost = L::EMPTY;
        let mut last_u = u;
        while u != L::EMPTY {
            let order = self.l.order(u);
            let (_, _, after) = self.l.split(u, ..=order);
            assert!(self.l.is_last(u));
            if after != L::EMPTY {
                self.parent[self.l.first(after)] = u;
            }
            self.l.concat(u, prev_topmost);
            last_u = u;
            u = self.l.first(u);
            (u, prev_topmost) = (std::mem::replace(&mut self.parent[u], L::EMPTY), u);
        }
        last_u
    }
}

impl<L> LinkCutTree for LCT<L>
where
    L: Lists,
{
    fn new(n: usize) -> Self {
        let mut l = L::new(n);
        for i in 0..n {
            assert_eq!(l.create(()), i);
        }
        Self {
            l,
            parent: vec![L::EMPTY; n],
        }
    }

    fn root(&mut self, u: Node) -> Node {
        self.access(u);
        self.l.first(u)
    }

    fn link(&mut self, u: Node, v: Node) -> bool {
        if self.root(u) == self.root(v) {
            return false;
        }
        self.reroot(v);
        self.parent[v] = u;
        true
    }

    fn cut(&mut self, u: Node) -> Option<Node> {
        self.access(u);
        if self.l.is_first(u) {
            return None;
        }
        let p = self.l.prev(u);
        // split ..p from u
        let order = self.l.order(u);
        self.l.split(u, ..order);
        Some(p)
    }

    fn reroot(&mut self, u: Node) {
        self.access(u);
        // u will be the new root
        self.l.reverse(u);
    }

    fn lca(&mut self, u: Node, v: Node) -> Option<Node> {
        self.access(u);
        let ru = self.l.first(u);
        let lca = self.access(v);
        let rv = self.l.first(v);
        (ru == rv).then_some(lca)
    }
}
