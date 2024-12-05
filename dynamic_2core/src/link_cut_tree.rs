pub type Node = usize;

/// Maintains a collection of trees dynamically.
pub trait LinkCutTree {
    /// Create a new LinkCutTree with n vertices and no edges.
    fn new(n: usize) -> Self;
    /// Adds an edge between u and v. Returns false if they were in the same tree.
    /// Reroots v, and keeps the root of the tree containing u the same.
    fn link(&mut self, u: Node, v: Node) -> bool;
    /// Cuts u from its immediate parent. Returns the parent of u.
    fn cut(&mut self, u: Node) -> Option<Node>;
    /// Makes u the root of its current tree.
    fn reroot(&mut self, u: Node);
    /// The k-th vertex from the root of the tree containing u to u.
    fn kth_in_path_from_root(&self, u: Node, k: usize) -> Option<Node>;
}
