use dynamic_2core::link_cut_tree::*;

#[derive(Debug)]
pub struct SlowLCT {
    parent: Vec<usize>,
}

impl SlowLCT {
    fn path_from_root(&self, mut u: Node) -> Vec<Node> {
        let mut path = vec![u];
        while self.parent[u] != u {
            u = self.parent[u];
            path.push(u);
        }
        path.reverse();
        path
    }
}

impl LinkCutTree for SlowLCT {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }

    fn root(&mut self, u: Node) -> Node {
        if self.parent[u] == u {
            u
        } else {
            self.root(self.parent[u])
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

    fn lca(&mut self, u: Node, v: Node) -> Option<Node> {
        let pu = self.path_from_root(u);
        let pv = self.path_from_root(v);
        if pu[0] != pv[0] {
            None
        } else {
            Some(
                pu.into_iter()
                    .zip(pv)
                    .take_while(|(a, b)| a == b)
                    .last()
                    .unwrap()
                    .0,
            )
        }
    }
}
