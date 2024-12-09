use std::fmt::Debug;

use dynamic_2core::lists::*;

/// Dummy implementation, most of the operations take linear time.
#[derive(Clone)]
pub struct SlowLists<Ag: AggregatedData = ()> {
    lists: Vec<Vec<Entry<Ag>>>,
    u_to_list: Vec<usize>,
}

impl<Ag: AggregatedData> Debug for SlowLists<Ag> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SlowLists:")?;
        for l in &self.lists {
            if l.len() > 1 {
                write!(f, " [")?;
                for e in l {
                    write!(f, "{}({:?}) ", e.idx, e.data)?;
                }
                write!(f, "]\n")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Entry<Ag: AggregatedData> {
    idx: Idx,
    data: Ag::Data,
}

impl<Ag: AggregatedData> SlowLists<Ag> {
    fn list(&self, u: Idx) -> &Vec<Entry<Ag>> {
        if u == Self::EMPTY {
            &self.lists[0]
        } else {
            &self.lists[self.u_to_list[u]]
        }
    }
    fn entry(&self, u: Idx) -> &Entry<Ag> {
        self.list(u).iter().find(|e| e.idx == u).unwrap()
    }
    pub fn lists(&self) -> Vec<Vec<Idx>> {
        self.lists
            .iter()
            .filter_map(|v| {
                if v.is_empty() {
                    None
                } else {
                    Some(v.iter().map(|e| e.idx).collect())
                }
            })
            .collect()
    }
}

impl<Ag: AggregatedData> Lists<Ag> for SlowLists<Ag> {
    fn new(capacity: usize) -> Self {
        let mut lists = Vec::with_capacity(capacity + 1);
        // SENTINEL for EMPTY
        lists.push(vec![]);
        Self {
            lists,
            u_to_list: Vec::with_capacity(capacity),
        }
    }

    fn create(&mut self, data: Ag::Data) -> Idx {
        let idx = self.total_size();
        self.lists.push(vec![Entry { idx, data }]);
        self.u_to_list.push(self.lists.len() - 1);
        idx
    }

    fn total_size(&self) -> usize {
        self.u_to_list.len()
    }

    fn root(&self, u: Idx) -> Idx {
        if u == Self::EMPTY {
            return Self::EMPTY;
        }
        self.list(u)[0].idx
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        &self.entry(u).data
    }

    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data)) {
        f(&mut self.lists[self.u_to_list[u]]
            .iter_mut()
            .find(|e| e.idx == u)
            .unwrap()
            .data)
    }

    fn order(&self, u: Idx) -> usize {
        if u == Self::EMPTY {
            return 0;
        }
        self.list(u).iter().position(|e| e.idx == u).unwrap()
    }

    fn find_element(
        &self,
        u: Idx,
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Idx {
        let left_agg = Ag::default();
        use SearchDirection::*;
        for i in 0..self.list(u).len() {
            let right_agg = self.range_agg(u, i + 1..);
            let e = &self.list(u)[i];
            match search_strategy(SearchData {
                current_data: &e.data,
                left_agg: &left_agg,
                right_agg: &right_agg,
            }) {
                Found => return e.idx,
                NotFound => return Self::EMPTY,
                Left => panic!("Should never go left"),
                Right => {}
            }
        }
        Self::EMPTY
    }

    fn find_kth(&self, u: Idx, k: usize) -> Idx {
        self.list(u).get(k).map_or(Self::EMPTY, |e| e.idx)
    }

    fn len(&self, u: Idx) -> usize {
        if u == Self::EMPTY {
            return 0;
        }
        self.list(u).len()
    }

    fn range_agg_lr(&self, u: Idx, l: usize, r: usize) -> Ag {
        self.list(u)
            .iter()
            .enumerate()
            .filter(|(i, _)| *i >= l && *i < r)
            .map(|(_, d)| &d.data)
            .fold(Ag::default(), |agg, data| agg.merge(Ag::from(data)))
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        if v == Self::EMPTY || self.on_same_list(u, v) {
            return u;
        } else if u == Self::EMPTY {
            return v;
        }
        let lu = self.u_to_list[u];
        let lv = self.u_to_list[v];
        for w in self.lists[lv].iter() {
            self.u_to_list[w.idx] = lu;
        }
        let mut nv = vec![];
        nv.append(&mut self.lists[lv]);
        self.lists[lu].append(&mut nv);
        self.root(u)
    }

    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx) {
        if u == Self::EMPTY {
            return (Self::EMPTY, Self::EMPTY, Self::EMPTY);
        }
        let lu = self.u_to_list[u];
        log::trace!("u {u} split {l}..{r}");
        assert!(
            l <= r && r <= self.lists[lu].len(),
            "Invalid range {l}..{r}: {self:?}"
        );
        let mut it = self.lists[lu].drain(..r);
        let gl: Vec<_> = (0..l).map(|_| it.next().unwrap()).collect();
        let gm: Vec<_> = it.collect();
        assert_eq!(gm.len(), r - l);
        let ig = self.lists.len();
        for e in gl.iter() {
            self.u_to_list[e.idx] = ig;
        }
        for e in gm.iter() {
            self.u_to_list[e.idx] = ig + 1;
        }
        self.lists.push(gl);
        self.lists.push(gm);
        (
            self.lists[ig].get(0).map_or(Self::EMPTY, |e| e.idx),
            self.lists[ig + 1].get(0).map_or(Self::EMPTY, |e| e.idx),
            self.lists[lu].get(0).map_or(Self::EMPTY, |e| e.idx),
        )
    }

    fn reverse(&mut self, u: Idx) {
        let l = self.u_to_list[u];
        self.lists[l].reverse();
    }
}
