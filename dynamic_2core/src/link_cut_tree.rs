use crate::lists::Lists;

pub type Node = usize;

/// Maintains a collection of trees dynamically.
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
    /// The k-th vertex from the root of the tree containing u to u.
    fn kth_in_path_from_root(&mut self, u: Node, k: usize) -> Option<Node>;
}

const NULL: usize = usize::MAX;

#[derive(Debug)]
pub struct LCT<L>
where
    L: Lists<()>,
{
    l: L,
    // Non-NULL iff the node is a root of a preferred path that is not the topmost.
    parent: Vec<usize>,
}

impl<L> LCT<L>
where
    L: Lists<()>,
{
    fn access(&mut self, mut u: Node) {
        let mut prev = L::EMPTY;
        while u != NULL {
            let (_, _, after) = self.l.split(u, ..=self.l.order(u));
            assert!(self.l.is_last(u));
            if after != L::EMPTY {
                self.parent[self.l.first(after)] = u;
            }
            self.l.concat(u, prev);
            u = self.l.first(u);
            (u, prev) = (std::mem::replace(&mut self.parent[u], NULL), u);
        }
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
            parent: vec![NULL; n],
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
        self.l.split(u, ..self.l.order(u));
        Some(p)
    }

    fn reroot(&mut self, u: Node) {
        self.access(u);
        // u will be the new root
        self.l.reverse(u);
    }

    fn kth_in_path_from_root(&mut self, u: Node, k: usize) -> Option<Node> {
        self.access(u);
        let v = self.l.find_kth(u, k);
        (v != L::EMPTY).then_some(v)
    }
}
