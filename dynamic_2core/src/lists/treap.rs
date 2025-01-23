//! A Treap, or Cartesian Tree, is a BST that is also a heap for randomized priorities.
//! It is expected to be balanced and have logarithmic time complexity for all operations.

use std::fmt::{Debug, Display, Formatter};

use debug_tree::{add_branch_to, add_leaf_to, AsTree, TreeBuilder};
use derivative::Derivative;
use rand::{rngs, Rng, SeedableRng};

use super::{AggregatedData, Idx, Lists, SearchData, SearchDirection};

pub(crate) fn node_fmt(u: &Idx, f: &mut Formatter) -> std::fmt::Result {
    if *u == usize::MAX {
        write!(f, "∅")
    } else {
        write!(f, "{u}")
    }
}
pub(crate) fn node2_fmt([u, v]: &[Idx; 2], f: &mut Formatter) -> std::fmt::Result {
    write!(f, "[")?;
    node_fmt(u, f)?;
    write!(f, ", ")?;
    node_fmt(v, f)?;
    write!(f, "]")
}

/// Used to pretty print a Idx, outputting ∅ if it is EMPTY.
pub struct PrettyIdx(pub Idx);

impl Display for PrettyIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        node_fmt(&self.0, f)
    }
}

impl Debug for PrettyIdx {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}

#[allow(unused_imports)]
use PrettyIdx as I;

#[derive(Derivative)]
#[derivative(Debug)]
struct Node<Ag: AggregatedData> {
    #[derivative(Debug(format_with = "node_fmt"))]
    parent: Idx,
    /// Left and right child
    #[derivative(Debug(format_with = "node2_fmt"))]
    child: [Idx; 2],
    /// This nodes children and aggregated data should be flipped.
    flip_subtree: bool,
    /// Data for this node
    data: Ag::Data,
    /// Aggregated data for this node's subtree
    ag_data: Ag,
    size: usize,
    #[derivative(Debug = "ignore")]
    priority: u32,
}

impl<Ag: AggregatedData> Node<Ag> {
    fn new(data: Ag::Data, priority: u32) -> Self {
        Self {
            ag_data: Ag::from(&data),
            data,
            child: [Treaps::<Ag>::EMPTY; 2],
            parent: Treaps::<Ag>::EMPTY,
            size: 1,
            priority,
            flip_subtree: false,
        }
    }
    fn flip(&self, flipped: bool) -> bool {
        self.flip_subtree ^ flipped
    }
}

/// Data structure that maintains multiple treaps.
pub struct Treaps<Ag: AggregatedData = ()> {
    nodes: Vec<Node<Ag>>,
    rng: rngs::StdRng,
}

impl<Ag: AggregatedData> Debug for Treaps<Ag> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let builder = TreeBuilder::new();
        add_branch_to!(builder, "Treaps");
        for u in 0..self.nodes.len() {
            if self.nodes[u].parent == Self::EMPTY {
                self.tree_inorder_dbg(u, &builder);
            }
        }
        writeln!(f, "{}", builder.string())
    }
}

trait ReverseF: AggregatedData {
    fn reverseif(self, flipped: bool) -> Self;
}

impl<T: AggregatedData> ReverseF for T {
    fn reverseif(self, flipped: bool) -> Self {
        if flipped {
            self.reverse()
        } else {
            self
        }
    }
}

impl<Ag: AggregatedData> Treaps<Ag> {
    #[allow(dead_code)]
    fn tree_preorder_dbg<T: AsTree>(&self, u: Idx, tree: &T) {
        let nu = &self.nodes[u];
        add_branch_to!(*tree, "[{u}] {nu:?}");
        if nu.child == [Self::EMPTY, Self::EMPTY] {
            return;
        }
        for c in nu.child {
            if c != Self::EMPTY {
                self.tree_preorder_dbg(c, tree);
            } else {
                add_leaf_to!(*tree, "<no edge>");
            }
        }
    }
    #[allow(dead_code)]
    fn tree_inorder_dbg<T: AsTree>(&self, u: Idx, tree: &T) {
        let nu = &self.nodes[u];
        if nu.child[0] != Self::EMPTY {
            add_branch_to!(*tree, "left child of {u}");
            self.tree_inorder_dbg(nu.child[0], tree);
        }
        add_branch_to!(*tree, "[{u}] {nu:?}");
        if nu.child[1] != Self::EMPTY {
            self.tree_inorder_dbg(nu.child[1], tree);
        }
    }
    fn n(&self, u: Idx) -> Option<&Node<Ag>> {
        // Even safer than just self.nodes.get(u)
        if u == Self::EMPTY {
            None
        } else {
            Some(&self.nodes[u])
        }
    }
    fn child(&self, u: Idx, flipped: bool) -> [usize; 2] {
        self.n(u).map_or([Self::EMPTY; 2], |n| {
            if n.flip(flipped) {
                [n.child[1], n.child[0]]
            } else {
                n.child
            }
        })
    }
    fn range(&self, u: Idx, ql: usize, qr: usize) -> [usize; 2] {
        self.n(u).map_or([ql, qr], |n| {
            if n.flip_subtree {
                [n.size - qr, n.size - ql]
            } else {
                [ql, qr]
            }
        })
    }
    /// Panics if u is empty. Returns old value.
    fn change_left(&mut self, u: Idx, new_l: Idx, flipped: bool) -> Idx {
        let n = &mut self.nodes[u];
        let li = n.flip(flipped) as usize;
        let old_l = self.nodes[u].child[li];
        if old_l != Self::EMPTY {
            self.nodes[old_l].parent = Self::EMPTY;
        }
        self.nodes[u].child[li] = new_l;
        if new_l != Self::EMPTY {
            self.nodes[new_l].parent = u;
        }
        self.recalc(u);
        old_l
    }
    fn change_right(&mut self, u: Idx, new_r: Idx, flipped: bool) -> Idx {
        self.change_left(u, new_r, !flipped)
    }
    fn size(&self, u: Idx) -> usize {
        self.n(u).map_or(0, |n| n.size)
    }
    fn parent(&self, u: Idx) -> Idx {
        self.n(u).map_or(Self::EMPTY, |n| n.parent)
    }
    fn ag_data(&self, u: Idx, flipped: bool) -> Ag {
        self.n(u).map_or_else(Ag::default, |n| {
            if n.flip(flipped) {
                n.ag_data.clone().reverse()
            } else {
                n.ag_data.clone()
            }
        })
    }
    /// Call when children are changed. Not necessary for flip_subtree if using the methods above.
    fn recalc(&mut self, u: Idx) -> Idx {
        if u == Self::EMPTY {
            return Self::EMPTY;
        }
        let f = self.nodes[u].flip_subtree;
        let [l, r] = self.child(u, false);
        self.nodes[u].size = self.size(l) + 1 + self.size(r);
        let ag = self
            .ag_data(l, f)
            .merge(Ag::from(&self.nodes[u].data))
            .merge(self.ag_data(r, f))
            // agg may actually stored reverse if the flip bit is set.
            .reverseif(f);
        self.nodes[u].ag_data = ag;
        u
    }
    fn unlaze_flip(&mut self, u: Idx) {
        if u == Self::EMPTY {
            return;
        }
        let n = &mut self.nodes[u];
        if n.flip_subtree {
            n.flip_subtree = false;
            n.ag_data = n.ag_data.clone().reverse();
            n.child.swap(0, 1);
            for c in n.child {
                if c != Self::EMPTY {
                    self.nodes[c].flip_subtree ^= true;
                }
            }
        }
    }
    /// (First k, rest)
    fn split_k(&mut self, u: Idx, k: usize) -> (Idx, Idx) {
        self.unlaze_flip(u);
        if u == Self::EMPTY || k == 0 {
            // If k == 0 the node is fully returned on the right
            return (Self::EMPTY, u);
        }
        let [l, r] = self.child(u, false);
        let szl = self.size(l);
        if k <= szl {
            self.change_left(u, Self::EMPTY, false);
            let (ll, lr) = self.split_k(l, k);
            (ll, self.concat(lr, u))
        } else {
            self.change_right(u, Self::EMPTY, false);
            let (rl, rr) = self.split_k(r, k - szl - 1);
            (self.concat(u, rl), rr)
        }
    }
    #[allow(dead_code)]
    fn dbg_node(&self, u: Idx) {
        if u == Self::EMPTY {
            log::trace!("Node ∅");
        } else {
            let t = TreeBuilder::new();
            self.tree_preorder_dbg(u, &t);
            log::trace!("\n{}", t.string());
        }
    }
    fn concat_inner(&mut self, u: Idx, v: Idx) -> Idx {
        self.unlaze_flip(u);
        self.unlaze_flip(v);
        if u == Self::EMPTY {
            return v;
        } else if v == Self::EMPTY {
            return u;
        }
        if self.nodes[u].priority > self.nodes[v].priority {
            let old_r = self.change_right(u, Self::EMPTY, false);
            let new_r = self.concat_inner(old_r, v);
            self.change_right(u, new_r, false);
            u
        } else {
            let old_l = self.change_left(v, Self::EMPTY, false);
            let new_l = self.concat_inner(u, old_l);
            self.change_left(v, new_l, false);
            v
        }
    }
    fn range_agg_lr_inner(&self, u: Idx, ql: usize, qr: usize) -> Ag {
        if u == Self::EMPTY || ql >= qr {
            return Ag::default();
        }
        if ql == 0 && qr >= self.size(u) {
            return self.ag_data(u, false);
        }
        let [ql, qr] = self.range(u, ql, qr);
        let [l, r] = self.nodes[u].child;
        let szl = self.size(l);
        let mut ag = Ag::default();
        if ql < szl {
            ag = self.range_agg_lr_inner(l, ql, qr.min(szl));
        }
        if ql <= szl && qr > szl {
            ag = ag.merge(Ag::from(&self.nodes[u].data));
        }
        if qr > szl + 1 {
            let rag = self.range_agg_lr_inner(r, ql.saturating_sub(szl + 1), qr - (szl + 1));
            ag = ag.merge(rag);
        }
        ag.reverseif(self.nodes[u].flip_subtree)
    }
}

impl<Ag: AggregatedData> Lists<Ag> for Treaps<Ag> {
    const EMPTY: Idx = usize::MAX;

    fn new(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            rng: rand::rngs::StdRng::seed_from_u64(2012),
        }
    }

    fn create(&mut self, data: Ag::Data) -> Idx {
        let idx = self.nodes.len();
        self.nodes.push(Node::new(data, self.rng.gen()));
        idx
    }

    fn total_size(&self) -> usize {
        self.nodes.len()
    }

    fn root(&mut self, mut u: Idx) -> Idx {
        while self.parent(u) != Self::EMPTY {
            u = self.nodes[u].parent;
        }
        u
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        &self.nodes[u].data
    }

    fn mutate_data(&mut self, mut u: Idx, f: impl FnOnce(&mut Ag::Data)) {
        f(&mut self.nodes[u].data);
        while u != Self::EMPTY {
            self.recalc(u);
            u = self.parent(u);
        }
    }

    fn order(&mut self, u: Idx) -> usize {
        if u == Self::EMPTY {
            return 0;
        }
        let mut path = vec![];
        let mut cur = u;
        while cur != Self::EMPTY {
            path.push(cur);
            cur = self.parent(cur);
        }
        path.reverse();
        let mut flipped = false;
        let mut ord = 0;
        for i in 0..(path.len() - 1) {
            let [p, u] = [path[i], path[i + 1]];
            let [l, r] = self.child(p, flipped);
            if u == r {
                ord += self.size(l) + 1
            }
            flipped = self.nodes[p].flip(flipped);
        }
        let [ul, _] = self.child(u, flipped);
        ord + self.size(ul)
    }

    fn find_element(
        &mut self,
        u: Idx,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
        let mut u = self.root(u);
        let mut flipped = false;
        use SearchDirection::*;
        while u != Self::EMPTY {
            let [l, r] = self.child(u, flipped);
            let p = u;
            match search_strategy(SearchData {
                current_data: self.data(u),
                left_agg: &self.ag_data(l, flipped),
                right_agg: &self.ag_data(r, flipped),
            }) {
                Found => return u,
                NotFound => return Self::EMPTY,
                Left => u = l,
                Right => u = r,
            }
            flipped = self.nodes[p].flip(flipped);
        }
        Self::EMPTY
    }

    fn find_kth(&mut self, mut u: Idx, mut k: usize) -> Idx {
        let mut flipped = false;
        u = self.root(u);
        while u != Self::EMPTY {
            let [l, r] = self.child(u, flipped);
            flipped = self.nodes[u].flip(flipped);
            let sl = self.size(l);
            if sl > k {
                u = l;
            } else if sl == k {
                return u;
            } else {
                k -= sl + 1;
                u = r;
            }
        }
        Self::EMPTY
    }

    fn len(&mut self, u: Idx) -> usize {
        if u == Self::EMPTY {
            0
        } else {
            let u = self.root(u);
            self.nodes[u].size
        }
    }

    fn total_agg(&mut self, u: Idx) -> Ag {
        let u = self.root(u);
        self.ag_data(u, false)
    }

    fn range_agg_lr(&mut self, u: Idx, ql: usize, qr: usize) -> Ag {
        let u = self.root(u);
        self.range_agg_lr_inner(u, ql, qr)
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        let (u, v) = (self.root(u), self.root(v));
        if u == v {
            self.unlaze_flip(u);
            return u;
        }
        self.concat_inner(u, v)
    }

    fn split_lr(&mut self, u: Idx, ql: usize, qr: usize) -> (Idx, Idx, Idx) {
        let u = self.root(u);
        let (l, mr) = self.split_k(u, ql);
        let (m, r) = self.split_k(mr, qr - ql);
        (l, m, r)
    }

    fn reverse(&mut self, u: Idx) {
        let u = self.root(u);
        self.nodes[u].flip_subtree ^= true;
    }

    fn is_root(&mut self, u: Idx) -> bool {
        self.parent(u) == Self::EMPTY
    }
}
