use std::fmt::{Debug, Formatter};

use derivative::Derivative;

use crate::lists::SearchData;

use super::{
    treap::{node2_fmt, node_fmt},
    AggregatedData, Idx, Lists,
};

// Used for debugging
#[allow(unused_imports)]
use super::treap::PrettyIdx as I;

#[derive(Derivative)]
#[derivative(Debug)]
struct Node<Ag: AggregatedData> {
    /// Data in this node
    data: Ag::Data,
    #[derivative(Debug(format_with = "node2_fmt"))]
    child: [Idx; 2],
    #[derivative(Debug(format_with = "node_fmt"))]
    parent: Idx,
    /// Delta flipped. Flipped is the xor of all d_flip starting at the root.
    d_flip: bool,
    /// Aggregated data in subtree
    subtree_agg: Ag,
    /// Size of subtree
    subtree_size: usize,
}

impl<Ag: AggregatedData> Node<Ag> {
    const EMPTY: Idx = usize::MAX;

    /// To use this, guarantee: you don't look at data, you don't modify anything.
    unsafe fn null() -> Self {
        Self {
            // We never use this
            data: std::mem::zeroed(),
            child: [Node::<Ag>::EMPTY; 2],
            parent: Node::<Ag>::EMPTY,
            d_flip: false,
            subtree_agg: Ag::default(),
            subtree_size: 0,
        }
    }

    fn new(data: Ag::Data) -> Self {
        Self {
            child: [Self::EMPTY; 2],
            parent: Self::EMPTY,
            d_flip: false,
            subtree_agg: Ag::from(&data),
            subtree_size: 1,
            data,
        }
    }

    fn child(&self, flip: bool) -> [Idx; 2] {
        if flip ^ self.d_flip {
            [self.child[1], self.child[0]]
        } else {
            self.child
        }
    }

    fn side_of(&self, u: Idx, flip: bool) -> Option<bool> {
        let flip = flip ^ self.d_flip;
        match self.child {
            [l, _] if l == u => Some(flip),
            [_, r] if r == u => Some(!flip),
            _ => None,
        }
    }

    fn agg(&self) -> Ag {
        let mut ag = self.subtree_agg.clone();
        if self.d_flip {
            ag = ag.reverse()
        }
        ag
    }
}

pub struct Splays<Ag: AggregatedData = ()> {
    n: Vec<Node<Ag>>,
    null: Node<Ag>,
}

impl<Ag: AggregatedData> Debug for Splays<Ag> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut seen = vec![false; self.n.len()];
        for u in 0..self.n.len() {
            if self.n[u].parent == Self::EMPTY {
                self.print_rec(f, u, false, &mut seen)?;
                write!(f, " --- ")?;
            }
        }
        Ok(())
    }
}

impl<Ag: AggregatedData> Splays<Ag> {
    fn print_rec(
        &self,
        f: &mut Formatter<'_>,
        u: Idx,
        flipped: bool,
        seen: &mut Vec<bool>,
    ) -> std::fmt::Result {
        if u == Self::EMPTY {
            return Ok(());
        }
        if seen[u] {
            write!(f, "<<ERR: Loop including vx {u}>>")?;
            return Ok(());
        }
        seen[u] = true;
        write!(f, "(")?;
        let [l, r] = self.n[u].child(flipped);
        let flipped = flipped ^ self.n[u].d_flip;
        self.print_rec(f, l, flipped, seen)?;
        write!(
            f,
            " {u}[{:?}]{} ",
            self.n[u].data,
            if self.n[u].d_flip { "<>" } else { "" }
        )?;
        self.print_rec(f, r, flipped, seen)?;
        seen[u] = false;
        write!(f, ")")
    }

    fn n(&self, u: Idx) -> &Node<Ag> {
        (u == Self::EMPTY)
            .then_some(&self.null)
            .unwrap_or_else(|| &self.n[u])
    }

    fn unlaze_flip(&mut self, u: Idx) {
        if u == Self::EMPTY {
            return;
        }
        let nu = &mut self.n[u];
        if nu.d_flip {
            nu.d_flip = false;
            nu.child.swap(0, 1);
            nu.subtree_agg = nu.subtree_agg.clone().reverse();
            for v in nu.child {
                if v != Self::EMPTY {
                    self.n[v].d_flip ^= true;
                }
            }
        }
    }

    fn update(&mut self, u: Idx) {
        if u == Self::EMPTY {
            return;
        }
        self.unlaze_flip(u);
        let nu = self.n(u);
        let [l, r] = nu.child;
        let [nl, nr] = [self.n(l), self.n(r)];
        let agg = nl.agg().merge(Ag::from(&nu.data)).merge(nr.agg());
        let size = nl.subtree_size + 1 + nr.subtree_size;
        self.n[u].subtree_agg = agg;
        self.n[u].subtree_size = size;
    }

    fn side_in_parent(&self, u: Idx) -> bool {
        self.n[self.n[u].parent]
            .side_of(u, false)
            .expect("Node should be child of its parent")
    }

    fn replace_child(&mut self, u: Idx, right: bool, new_child: Idx) -> Idx {
        if new_child != Self::EMPTY {
            // To be safe, let's make sure the tree is ALWAYS valid
            assert_eq!(self.n[new_child].parent, Self::EMPTY);
            self.n[new_child].parent = u;
        }
        if u == Self::EMPTY {
            return Self::EMPTY;
        }
        let nu = &mut self.n[u];
        let prev_child = std::mem::replace(&mut nu.child[(right ^ nu.d_flip) as usize], new_child);
        if prev_child != Self::EMPTY {
            assert_eq!(self.n[prev_child].parent, u);
            self.n[prev_child].parent = Self::EMPTY;
        }
        self.update(u);
        prev_child
    }

    fn rotate_up(&mut self, u: Idx) {
        let p = self.n[u].parent;
        assert!(p != Self::EMPTY, "Can't rotate_up root");
        self.unlaze_flip(p);
        self.unlaze_flip(u);
        let u_side = self.side_in_parent(u);
        let b = std::mem::replace(&mut self.n[u].child[!u_side as usize], p);
        self.n[p].child[u_side as usize] = b;
        let pp = self.n[p].parent;
        self.n[u].parent = pp;
        if b != Self::EMPTY {
            self.n[b].parent = p;
        }
        if pp != Self::EMPTY {
            let p_side = self.side_in_parent(p);
            let pp_flipped = self.n[pp].d_flip;
            self.n[pp].child[(p_side ^ pp_flipped) as usize] = u;
        }
        self.n[p].parent = u;
        self.update(p); // Now lowest
        self.update(u); // mid
        self.update(pp); // top
    }

    /// Splay u to the root. u will be unlazed after this.
    fn splay(&mut self, u: Idx) {
        if u == Self::EMPTY {
            return;
        }
        loop {
            let p = self.n[u].parent;
            let pp = self.n(p).parent;
            self.unlaze_flip(pp);
            self.unlaze_flip(p);
            self.unlaze_flip(u);
            if pp == Self::EMPTY {
                if p != Self::EMPTY {
                    self.rotate_up(u);
                }
                break;
            }
            let u_side = self.side_in_parent(u);
            let p_side = self.side_in_parent(p);
            if u_side == p_side {
                self.rotate_up(p);
                self.rotate_up(u);
            } else {
                self.rotate_up(u);
                self.rotate_up(u);
            }
        }
    }

    fn check_rec(&self, u: Idx, seen: &mut Vec<bool>) -> (usize, Ag)
    where
        Ag: Eq,
    {
        if u == Self::EMPTY {
            return (0, Ag::default());
        }
        assert!(!seen[u], "Loop including vx {u}");
        seen[u] = true;
        let [l, r] = self.n[u].child;
        let (mut tot_sz, mut tot_agg) = self.check_rec(l, seen);
        for x in [l, r] {
            if x != Self::EMPTY {
                assert_eq!(self.n[x].parent, u, "parent of {x} wrong (not {u})");
            }
        }
        tot_sz += 1;
        tot_agg = tot_agg.merge(Ag::from(&self.n[u].data));
        let (sz, agg) = self.check_rec(r, seen);
        tot_sz += sz;
        tot_agg = tot_agg.merge(agg);
        assert_eq!(tot_sz, self.n[u].subtree_size, "size calculated wrong");
        assert_eq!(tot_agg, self.n[u].subtree_agg, "agg calculated wrong");
        if self.n[u].d_flip {
            tot_agg = tot_agg.reverse();
        }
        seen[u] = false;
        (tot_sz, tot_agg)
    }
    /// Used for debugging, makes sure the structure of the tree is correct.
    #[allow(dead_code)]
    fn check_all(&self)
    where
        Ag: Eq,
    {
        log::trace!("Checking {self:?}");
        let mut seen = vec![false; self.n.len()];
        for u in 0..self.n.len() {
            let p = self.n[u].parent;
            if p != Self::EMPTY {
                assert!(self.n[p].child.contains(&u));
            }
            self.check_rec(u, &mut seen);
        }
    }
}

impl<Ag: AggregatedData> Lists<Ag> for Splays<Ag> {
    const EMPTY: Idx = Node::<Ag>::EMPTY;

    fn new(capacity: usize) -> Self {
        Self {
            n: Vec::with_capacity(capacity),
            null: unsafe { Node::null() },
        }
    }

    fn create(&mut self, data: Ag::Data) -> Idx {
        self.n.push(Node::new(data));
        self.n.len() - 1
    }

    fn total_size(&self) -> usize {
        self.n.len()
    }

    fn root(&mut self, mut u: Idx) -> Idx {
        if u == Self::EMPTY {
            return Self::EMPTY;
        }
        // Let's make the smallest the root, and then return it.
        // This way, it preserves the complexity, and the property that
        // u and v are in the same list iff root(u) == root(v)
        self.splay(u);
        loop {
            self.unlaze_flip(u);
            let [l, _] = self.n[u].child(false);
            if l == Self::EMPTY {
                break;
            }
            u = l;
        }
        self.splay(u);
        u
    }

    fn on_same_list(&mut self, u: Idx, v: Idx) -> bool {
        if u == v {
            return true;
        }
        self.splay(u);
        self.splay(v);
        // Splaying v made u not the root
        self.n(u).parent != Self::EMPTY
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        &self.n[u].data
    }

    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data)) {
        self.splay(u);
        f(&mut self.n[u].data);
        self.update(u);
    }

    fn order(&mut self, u: Idx) -> usize {
        self.splay(u);
        self.n(self.n(u).child(false)[0]).subtree_size
    }

    fn find_element(
        &mut self,
        mut u: Idx,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> super::SearchDirection,
    ) -> Idx {
        self.splay(u);
        let mut prev_u = u;
        let found = loop {
            if u == Self::EMPTY {
                break Self::EMPTY;
            }
            self.unlaze_flip(u);
            let [l, r] = self.n[u].child;
            let st = SearchData {
                current_data: &self.n[u].data,
                left_agg: &self.n(l).subtree_agg,
                right_agg: &self.n(r).subtree_agg,
            };
            use super::SearchDirection::*;
            prev_u = u;
            match search_strategy(st) {
                Found => break u,
                NotFound => break Self::EMPTY,
                Left => u = l,
                Right => u = r,
            }
        };
        // Splay last seen vertex
        self.splay(prev_u);
        found
    }

    fn find_kth(&mut self, mut u: Idx, mut k: usize) -> Idx {
        self.splay(u);
        if self.n(u).subtree_size <= k {
            return Self::EMPTY;
        }
        loop {
            self.unlaze_flip(u);
            let [l, r] = self.n[u].child;
            let szl = self.n(l).subtree_size;
            if szl == k {
                break;
            } else if szl > k {
                u = l;
            } else {
                k -= szl + 1;
                u = r;
            }
        }
        self.splay(u);
        u
    }

    fn first(&mut self, u: Idx) -> Idx {
        // We always return the smallest in the root
        self.root(u)
    }

    fn len(&mut self, u: Idx) -> usize {
        self.splay(u);
        self.n(u).subtree_size
    }

    fn range_agg_lr(&mut self, u: Idx, l: usize, r: usize) -> Ag {
        let (nl, nm, nr) = self.split_lr(u, l, r);
        let ans = self.n(nm).subtree_agg.clone();
        self.concat_all([nl, nm, nr]);
        ans
    }

    fn total_agg(&mut self, u: Idx) -> Ag {
        self.splay(u);
        self.n(u).subtree_agg.clone()
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        let v = self.first(v);
        self.splay(u);
        if v == Self::EMPTY {
            return u;
        }
        assert_eq!(self.replace_child(v, false, u), Self::EMPTY);
        v
    }

    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx) {
        let middle = self.find_kth(u, l);
        if middle == Self::EMPTY {
            return (u, Self::EMPTY, Self::EMPTY);
        }
        let left = self.replace_child(middle, false, Self::EMPTY);
        if r == l {
            return (left, Self::EMPTY, middle);
        }
        let last = self.find_kth(middle, r - l - 1);
        let right = self.replace_child(last, true, Self::EMPTY);
        // might not be the root anymore
        self.splay(middle);
        (left, middle, right)
    }

    fn reverse(&mut self, u: Idx) {
        self.splay(u);
        if u != Self::EMPTY {
            self.n[u].d_flip ^= true;
        }
    }
}
