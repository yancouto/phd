use std::fmt::{Debug, Formatter};

use debug_tree::TreeBuilder;
use derivative::Derivative;
use rand::{rngs, Rng, SeedableRng};

use super::{AggregatedData, Idx, Lists, SearchData, SearchDirection};

fn node_fmt(u: &Idx, f: &mut Formatter) -> std::fmt::Result {
    if *u == usize::MAX {
        write!(f, "∅")
    } else {
        write!(f, "{u}")
    }
}
fn node2_fmt([u, v]: &[Idx; 2], f: &mut Formatter) -> std::fmt::Result {
    write!(f, "[")?;
    node_fmt(u, f)?;
    write!(f, ", ")?;
    node_fmt(v, f)?;
    write!(f, "]")
}

#[derive(Derivative)]
#[derivative(Debug)]
struct Node<Ag: AggregatedData> {
    #[derivative(Debug(format_with = "node_fmt"))]
    parent: Idx,
    #[derivative(Debug(format_with = "node2_fmt"))]
    /// Left and right child
    child: [Idx; 2],
    /// This nodes children and aggregated data should be flipped.
    flip_subtree: bool,
    /// Data for this node
    data: Ag::Data,
    /// Aggregated data for this node's subtree
    ag_data: Ag,
    #[derivative(Debug = "ignore")]
    priority: u32,
    #[derivative(Debug = "ignore")]
    size: usize,
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

pub struct Treaps<Ag: AggregatedData> {
    nodes: Vec<Node<Ag>>,
    rng: rngs::StdRng,
}

impl<Ag: AggregatedData> Debug for Treaps<Ag> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut builder = TreeBuilder::new();
        let _b = builder.add_branch("Treaps");
        for u in &self.nodes {
            if u.parent == Self::EMPTY {
                self.tree_dbg(u, &mut builder);
            }
        }
        writeln!(f, "{}", builder.string())
    }
}

impl<Ag: AggregatedData> Treaps<Ag> {
    fn tree_dbg(&self, u: &Node<Ag>, tree: &mut TreeBuilder) {
        let _b = tree.add_branch(&format!("{u:?}"));
        if u.child == [Self::EMPTY, Self::EMPTY] {
            return;
        }
        for c in u.child {
            if c != Self::EMPTY {
                self.tree_dbg(&self.nodes[c], tree);
            } else {
                tree.add_leaf("<no edge>");
            }
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
    // Panics if empty
    fn change_left(&mut self, u: Idx, new_l: Idx, flipped: bool) -> Idx {
        let n = &mut self.nodes[u];
        let li = n.flip(flipped) as usize;
        let old_l = self.nodes[u].child[li];
        if old_l != Self::EMPTY {
            self.nodes[old_l].parent = Self::EMPTY;
        }
        self.nodes[u].child[li] = new_l;
        self.recalc(u);
        if new_l != Self::EMPTY {
            self.nodes[new_l].parent = u;
        }
        new_l
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
    // Call when children are changed. Not necessary for flip_subtree if using the methods above.
    fn recalc(&mut self, u: Idx) -> Idx {
        if u == Self::EMPTY {
            return Self::EMPTY;
        }
        let f = self.nodes[u].flip_subtree;
        let [l, r] = self.child(u, false);
        self.nodes[u].size = self.size(l) + 1 + self.size(r);
        let mut ag = self
            .ag_data(l, f)
            .merge(Ag::from(&self.nodes[u].data))
            .merge(self.ag_data(r, f));
        if f {
            // agg is actually stored reverse because the flip bit is set.
            ag = ag.reverse();
        }
        self.nodes[u].ag_data = ag;
        u
    }
    /// (First k, rest)
    fn split_k(&mut self, u: Idx, k: usize, flipped: bool) -> (Idx, Idx) {
        if u == Self::EMPTY || k == 0 {
            return (Self::EMPTY, Self::EMPTY);
        }
        let [l, r] = self.child(u, flipped);
        let szl = self.size(l);
        if k <= szl {
            self.change_left(u, Self::EMPTY, flipped);
            let (ll, lr) = self.split_k(l, k, self.nodes[u].flip(flipped));
            (ll, self.concat(lr, u))
        } else {
            self.change_right(u, Self::EMPTY, flipped);
            let (rl, rr) = self.split_k(r, k - szl - 1, self.nodes[u].flip(flipped));
            (self.concat(u, rl), rr)
        }
    }
    fn concat_inner(&mut self, u: Idx, v: Idx) -> Idx {
        if u == Self::EMPTY {
            return v;
        } else if v == Self::EMPTY {
            return u;
        }
        if self.nodes[u].priority > self.nodes[v].priority {
            let new_r = self.concat_inner(self.child(u, false)[1], v);
            self.change_right(u, new_r, false);
            if self.nodes[u].flip_subtree {
                self.reverse(new_r);
            }
            u
        } else {
            let new_l = self.concat_inner(u, self.child(v, false)[0]);
            self.change_left(v, new_l, false);
            if self.nodes[v].flip_subtree {
                self.reverse(new_l);
            }
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
        let f = self.nodes[u].flip_subtree;
        let [l, r] = self.child(u, false);
        let szl = self.size(l);
        let mut ag = Ag::default();
        if ql < szl {
            ag = self.range_agg_lr_inner(l, ql, qr.min(szl));
            if f {
                ag = ag.reverse();
            }
        }
        if ql <= szl && qr > szl {
            ag = ag.merge(Ag::from(&self.nodes[u].data));
        }
        if qr > szl + 1 {
            let mut rag = self.range_agg_lr_inner(r, ql.saturating_sub(szl + 1), qr - (szl + 1));
            if f {
                rag = rag.reverse();
            }
            ag = ag.merge(rag);
        }
        ag
    }
}

impl<Ag: AggregatedData> Lists<Ag> for Treaps<Ag> {
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

    fn root(&self, mut u: Idx) -> Idx {
        let ou = u;
        while self.parent(u) != Self::EMPTY {
            u = self.nodes[u].parent;
        }
        log::info!("root({ou}) = {u}");
        u
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        &self.nodes[u].data
    }

    fn data_mut(&mut self, u: Idx) -> &mut Ag::Data {
        // TODO: this can't be like this
        &mut self.nodes[u].data
    }

    fn order(&self, mut u: Idx) -> usize {
        // XXX: This needs fixing due to flipping. Walk all the way to root, then walk down.
        let mut ord = 0;
        while self.parent(u) != Self::EMPTY {
            let prev = u;
            u = self.nodes[u].parent;
            let [l, r] = self.child(u, false);
            if prev == r {
                ord += self.size(l) + 1;
            }
        }
        ord
    }

    fn find_element(
        &self,
        mut u: Idx,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
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

    fn find_kth(&self, mut u: Idx, mut k: usize) -> Idx {
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

    fn len(&self, u: Idx) -> usize {
        if u == Self::EMPTY {
            0
        } else {
            self.nodes[self.root(u)].size
        }
    }

    fn total_agg(&self, u: Idx) -> Ag {
        let u = self.root(u);
        self.ag_data(u, false)
    }

    fn range_agg_lr(&self, u: Idx, ql: usize, qr: usize) -> Ag {
        self.range_agg_lr_inner(self.root(u), ql, qr)
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        log::info!("Concat {u} {v}");
        let (u, v) = (self.root(u), self.root(v));
        self.concat_inner(u, v)
    }

    fn split_lr(&mut self, u: Idx, ql: usize, qr: usize) -> (Idx, Idx, Idx) {
        let (l, mr) = self.split_k(u, ql, false);
        let (m, r) = self.split_k(mr, qr - ql, false);
        (l, m, r)
    }

    fn reverse(&mut self, u: Idx) {
        self.nodes[u].flip_subtree ^= true;
    }
}
