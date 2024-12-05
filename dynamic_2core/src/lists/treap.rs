use rand::random;
use std::ops::RangeBounds;

use super::{AggregatedData, Idx, Lists, SearchData, SearchDirection};

#[derive(Debug)]
struct Node<Ag: AggregatedData> {
    ag_data: Ag,
    data: Ag::Data,
    child: [Idx; 2],
    parent: Idx,
    size: usize,
    priority: u32,
    flip_subtree: bool,
}

impl<Ag: AggregatedData> Node<Ag> {
    fn new(data: Ag::Data) -> Self {
        Self {
            ag_data: Ag::from(&data),
            data,
            child: [Treaps::<Ag>::EMPTY; 2],
            parent: Treaps::<Ag>::EMPTY,
            size: 1,
            priority: random(),
            flip_subtree: false,
        }
    }
}

#[derive(Debug)]
pub struct Treaps<Ag: AggregatedData> {
    nodes: Vec<Node<Ag>>,
}

impl<Ag: AggregatedData> Treaps<Ag> {
    fn n(&self, u: Idx) -> Option<&Node<Ag>> {
        // Even safer than just self.nodes.get(u)
        if u == Self::EMPTY {
            None
        } else {
            Some(&self.nodes[u])
        }
    }
    fn child(&self, u: Idx) -> [usize; 2] {
        self.n(u).map_or([Self::EMPTY; 2], |n| n.child)
    }
    fn size(&self, u: Idx) -> usize {
        self.n(u).map_or(0, |n| n.size)
    }
    fn parent(&self, u: Idx) -> Idx {
        self.n(u).map_or(Self::EMPTY, |n| n.parent)
    }
    fn ag_data(&self, u: Idx) -> Ag {
        self.n(u).map_or_else(Ag::default, |n| n.ag_data.clone())
    }
    fn recalc(&mut self, u: Idx) {
        if u == Self::EMPTY {
            return;
        }
        if self.nodes[u].flip_subtree {
            self.nodes[u].flip_subtree = false;
            self.nodes[u].child.swap(0, 1);
            for c in self.nodes[u].child {
                self.reverse(c);
            }
        }
        let [l, r] = self.nodes[u].child;
        self.nodes[u].size = 1 + self.size(l) + self.size(r);
        self.nodes[u].ag_data = self
            .ag_data(l)
            .merge(Ag::from(&self.nodes[u].data))
            .merge(self.ag_data(r));
    }
}

impl<Ag: AggregatedData> Lists<Ag> for Treaps<Ag> {
    fn new(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    fn create(&mut self, data: Ag::Data) -> Idx {
        let idx = self.nodes.len();
        self.nodes.push(Node::new(data));
        idx
    }

    fn total_size(&self) -> usize {
        self.nodes.len()
    }

    fn root(&self, mut u: Idx) -> Idx {
        while self.parent(u) != Self::EMPTY {
            u = self.nodes[u].parent;
        }
        u
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        &self.nodes[u].data
    }

    fn data_mut(&mut self, u: Idx) -> &mut Ag::Data {
        &mut self.nodes[u].data
    }

    fn order(&self, u: Idx) -> usize {
        todo!()
    }

    fn find_element(
        &mut self,
        mut u: Idx,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
        use SearchDirection::*;
        while u != Self::EMPTY {
            self.recalc(u);
            let [l, r] = self.nodes[u].child;
            match search_strategy(SearchData {
                current_data: self.data(u),
                left_agg: &self.ag_data(l),
                right_agg: &self.ag_data(r),
            }) {
                Found => return u,
                NotFound => return Self::EMPTY,
                Left => u = l,
                Right => u = r,
            }
        }
        Self::EMPTY
    }

    fn find_kth(&self, mut u: Idx, mut k: usize) -> Idx {
        u = self.root(u);
        while u != Self::EMPTY {
            // TODO: Don't recalc, use flip boolean without modifying
            //self.recalc(u);
            let [l, r] = self.child(u);
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

    fn total_agg(&mut self, u: Idx) -> Ag {
        let u = self.root(u);
        self.recalc(u);
        self.nodes[u].ag_data.clone()
    }

    fn range_agg(&mut self, u: Idx, range: impl RangeBounds<usize>) -> Ag {
        let u = self.root(u);
        todo!()
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        let (u, v) = (self.root(u), self.root(v));
        if u == Self::EMPTY {
            return v;
        } else if v == Self::EMPTY {
            return u;
        }
        self.recalc(u);
        self.recalc(v);
        if self.nodes[u].priority > self.nodes[v].priority {
            self.nodes[u].child[1] = self.concat(self.nodes[u].child[1], v);
            self.recalc(u);
            u
        } else {
            self.nodes[v].child[0] = self.concat(u, self.nodes[v].child[0]);
            self.recalc(v);
            v
        }
    }

    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx) {
        todo!()
    }

    fn reverse(&mut self, u: Idx) {
        self.nodes[u].flip_subtree ^= true;
    }
}
