use dynamic_2core::link_cut_tree::*;

#[derive(Debug)]
pub struct SlowLCT {
    parent: Vec<usize>,
}

impl SlowLCT {
    fn root(&self, u: Node) -> Node {
        if self.parent[u] == u {
            u
        } else {
            self.root(self.parent[u])
        }
    }
}

impl LinkCutTree for SlowLCT {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
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
        let p = self.parent[u];
        self.parent[u] = u;
        (u != p).then_some(p)
    }

    fn reroot(&mut self, u: Node) {
        let p = self.parent[u];
        if p != u {
            self.reroot(p);
            self.parent[p] = u;
            self.parent[u] = u;
        }
    }

    fn kth_in_path_from_root(&self, u: Node, k: usize) -> Option<Node> {
        let mut path = vec![u];
        let mut last = u;
        while self.parent[last] != last {
            last = self.parent[last];
            path.push(last);
        }
        path.reverse();
        path.get(k).copied()
    }
}
