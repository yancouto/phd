use std::{
    marker::PhantomData,
    ops::RangeBounds,
    rc::Rc,
    sync::{Arc, RwLock},
    usize,
};

use crate::implicit_bst::*;

#[derive(Debug, Clone)]
pub struct SlowBst<Ag: AggregatedData> {
    node_idx: usize,
    _phantom: PhantomData<Ag>,
}

#[derive(Clone, Debug)]
pub struct GroupEntry<Ag: AggregatedData> {
    node_idx: usize,
    node_data: Ag::Data,
}

#[derive(Debug)]
pub struct Group<Ag: AggregatedData>(Vec<GroupEntry<Ag>>);

// Fucking hack because it's hard to do globals in Rust
pub trait SlowBstData: AggregatedData + 'static {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<Self>>>>>;
}

impl<Ag: SlowBstData> SlowBst<Ag>
where
    Ag::Data: Default,
{
    // node id to group id

    fn from(idx: usize) -> Rc<Self> {
        Rc::new(SlowBst {
            node_idx: idx,
            _phantom: PhantomData,
        })
    }

    fn group(&self) -> Arc<RwLock<Group<Ag>>> {
        Ag::map().read().unwrap()[self.node_idx].clone()
    }
    fn group_entry_idx(&self, idx: usize) -> usize {
        Ag::map().read().unwrap()[self.node_idx]
            .read()
            .unwrap()
            .0
            .get(idx)
            .map_or(usize::MAX, |d| d.node_idx)
    }
    fn map_len() -> usize {
        Ag::map().read().unwrap().len()
    }
}

impl<Ag: SlowBstData> ImplicitBST<Ag> for SlowBst<Ag>
where
    Ag::Data: Default,
{
    fn new_empty() -> Rc<Self> {
        SlowBst::from(usize::MAX)
    }

    fn new(data: Ag::Data) -> Rc<Self> {
        let node = SlowBst::from(Self::map_len());
        let g = Group(vec![GroupEntry {
            node_idx: node.node_idx,
            node_data: data,
        }]);
        Ag::map().write().unwrap().push(Arc::new(RwLock::new(g)));
        node
    }

    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> impl Iterator<Item = Rc<Self>> {
        let cur = Self::map_len();
        let entries = data
            .into_iter()
            .enumerate()
            .map(|(i, d)| GroupEntry {
                node_idx: cur + i,
                node_data: d,
            })
            .collect::<Vec<_>>();
        let added = entries.len();
        let g = Arc::new(RwLock::new(Group(entries)));
        Ag::map()
            .write()
            .unwrap()
            .extend(std::iter::repeat(g).take(added));
        (cur..cur + added).map(SlowBst::from)
    }

    fn root(&self) -> Rc<Self> {
        let root_idx = self.group_entry_idx(0);
        SlowBst::from(root_idx)
    }

    fn node_data(&self) -> &Ag::Data {
        // TRUST ME
        unsafe { &*(&self.group().read().unwrap().0[self.order()].node_data as *const _) }
    }

    fn order(&self) -> usize {
        self.group()
            .read()
            .unwrap()
            .0
            .iter()
            .position(|x| x.node_idx == self.node_idx)
            .unwrap()
    }

    fn range_agg(&self, range: impl RangeBounds<usize>) -> Ag {
        if self.node_idx == usize::MAX {
            return Ag::default();
        }
        let cur_k = self.order();
        self.group()
            .read()
            .unwrap()
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, d)| {
                (i >= cur_k && range.contains(&(i - cur_k))).then(|| Ag::from(&d.node_data))
            })
            .fold(Ag::default(), Ag::merge)
    }

    fn find_kth(&self, k: usize) -> Rc<Self> {
        let cur_k = self.order();
        let the_idx = self.group_entry_idx(cur_k + k);
        SlowBst::from(the_idx)
    }

    fn len(&self) -> usize {
        if self.node_idx == usize::MAX {
            return 0;
        }
        self.group().read().unwrap().0.len()
    }

    fn same_node(self: &Rc<Self>, other: &Rc<Self>) -> bool {
        self.node_idx == other.node_idx
    }

    fn concat(&self, other: &Self) -> Rc<Self> {
        let g1 = self.group();
        let g2 = other.group();
        assert!(!Arc::ptr_eq(&g1, &g2));
        g1.write().unwrap().0.append(&mut g2.write().unwrap().0);
        let root = SlowBst::from(g1.read().unwrap().0[0].node_idx);
        Ag::map().write().unwrap().iter_mut().for_each(|g| {
            if Arc::ptr_eq(g, &g2) {
                *g = g1.clone();
            }
        });
        root
    }

    fn split(&self, range: impl RangeBounds<usize>) -> (Rc<Self>, Rc<Self>, Rc<Self>) {
        let g = self.group();
        let mut g = g.write().unwrap();
        let mut v = vec![];
        v.append(&mut g.0);
        let (mut left, mut middle, mut right) = (Vec::new(), Vec::new(), Vec::new());
        let mut reached_middle = false;
        for (i, data) in v.into_iter().enumerate() {
            reached_middle = reached_middle || range.contains(&i);
            match (reached_middle, range.contains(&i)) {
                (false, false) => left.push(data.clone()),
                (_, true) => middle.push(data.clone()),
                (true, false) => right.push(data.clone()),
            }
        }
        let ret = (
            SlowBst::from(left.first().map_or(usize::MAX, |n| n.node_idx)),
            SlowBst::from(middle.first().map_or(usize::MAX, |n| n.node_idx)),
            SlowBst::from(right.first().map_or(usize::MAX, |n| n.node_idx)),
        );
        g.0.append(&mut middle);
        let gl = Arc::new(RwLock::new(Group(left.clone())));
        let gr = Arc::new(RwLock::new(Group(right.clone())));
        let mut map = Ag::map().write().unwrap();
        for il in left {
            map[il.node_idx] = gl.clone();
        }
        for ir in right {
            map[ir.node_idx] = gr.clone();
        }
        ret
    }

    fn find_element(
        &self,
        _search_strategy: impl FnMut(usize, &<Ag as AggregatedData>::Data, &Ag) -> SearchDirection,
    ) -> Rc<Self> {
        todo!()
    }
}
