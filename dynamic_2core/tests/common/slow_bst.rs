use std::{
    marker::PhantomData,
    ops::RangeBounds,
    sync::{Arc, RwLock, Weak},
    usize,
};

use dynamic_2core::{dynamic_2core::AgData, euler_tour_tree::ETAggregated, implicit_bst::*};

use super::AggSum;

#[derive(Debug, Clone)]
pub struct SlowBst<Ag: AggregatedData> {
    node_idx: usize,
    _phantom: PhantomData<Ag>,
}

#[derive(Clone, Debug)]
pub struct GroupEntry<Ag: AggregatedData> {
    node_idx: usize,
    arc: Arc<SlowBst<Ag>>,
    node_data: Ag::Data,
}

#[derive(Debug)]
pub struct Group<Ag: AggregatedData>(Vec<GroupEntry<Ag>>);

// Fucking hack because it's hard to do globals in Rust
pub trait SlowBstData: AggregatedData + 'static {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<Self>>>>>;
}

impl<Ag: SlowBstData> SlowBst<Ag> {
    // node id to group id

    fn create(idx: usize) -> Arc<Self> {
        Arc::new(SlowBst {
            node_idx: idx,
            _phantom: PhantomData,
        })
    }

    fn find(idx: usize) -> Arc<Self> {
        Ag::map().read().unwrap()[idx]
            .read()
            .unwrap()
            .0
            .iter()
            .find(|x| x.node_idx == idx)
            .unwrap()
            .arc
            .clone()
    }

    fn group(&self) -> Arc<RwLock<Group<Ag>>> {
        Ag::map().read().unwrap()[self.node_idx].clone()
    }
    fn group_entry_idx(&self, idx: usize) -> Arc<Self> {
        Ag::map().read().unwrap()[self.node_idx]
            .read()
            .unwrap()
            .0
            .get(idx)
            .map_or_else(Self::new_empty, |d| d.arc.clone())
    }
    fn map_len() -> usize {
        Ag::map().read().unwrap().len()
    }
}

impl<Ag: SlowBstData> ImplicitBST<Ag> for SlowBst<Ag> {
    fn new_empty() -> Arc<Self> {
        SlowBst::create(usize::MAX)
    }

    fn new(data: Ag::Data) -> Arc<Self> {
        let node = SlowBst::create(Self::map_len());
        let g = Group(vec![GroupEntry {
            node_idx: node.node_idx,
            arc: node.clone(),
            node_data: data,
        }]);
        Ag::map().write().unwrap().push(Arc::new(RwLock::new(g)));
        node
    }

    fn from_iter(data: impl IntoIterator<Item = Ag::Data>) -> impl Iterator<Item = Arc<Self>> {
        let cur = Self::map_len();
        let entries = data
            .into_iter()
            .enumerate()
            .map(|(i, d)| GroupEntry {
                node_idx: cur + i,
                arc: SlowBst::create(cur + i),
                node_data: d,
            })
            .collect::<Vec<_>>();
        let nodes = entries.iter().map(|e| e.arc.clone()).collect::<Vec<_>>();
        let added = entries.len();
        let g = Arc::new(RwLock::new(Group(entries)));
        Ag::map()
            .write()
            .unwrap()
            .extend(std::iter::repeat(g).take(added));
        nodes.into_iter()
    }

    fn root(&self) -> Arc<Self> {
        let root_idx = self.group_entry_idx(0);
        root_idx
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

    fn find_kth(&self, k: usize) -> Arc<Self> {
        let cur_k = self.order();
        let the_idx = self.group_entry_idx(cur_k + k);
        the_idx
    }

    fn len(&self) -> usize {
        if self.node_idx == usize::MAX {
            return 0;
        }
        self.group().read().unwrap().0.len()
    }

    fn same_node(self: &Arc<Self>, other: &Arc<Self>) -> bool {
        self.node_idx == other.node_idx
    }

    fn is_empty(&self) -> bool {
        self.node_idx == usize::MAX
    }

    fn concat(&self, other: &Self) -> Arc<Self> {
        if self.is_empty() {
            return Self::find(other.node_idx);
        } else if other.is_empty() {
            return Self::find(self.node_idx);
        }
        let g1 = self.group();
        let g2 = other.group();
        assert!(!Arc::ptr_eq(&g1, &g2));
        g1.write().unwrap().0.append(&mut g2.write().unwrap().0);
        let root = g1.read().unwrap().0[0].arc.clone();
        Ag::map().write().unwrap().iter_mut().for_each(|g| {
            if Arc::ptr_eq(g, &g2) {
                *g = g1.clone();
            }
        });
        root
    }

    fn split(&self, range: impl RangeBounds<usize>) -> (Arc<Self>, Arc<Self>, Arc<Self>) {
        if self.is_empty() {
            return (Self::new_empty(), Self::new_empty(), Self::new_empty());
        }
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
            left.first().map_or_else(Self::new_empty, |n| n.arc.clone()),
            middle
                .first()
                .map_or_else(Self::new_empty, |n| n.arc.clone()),
            right
                .first()
                .map_or_else(Self::new_empty, |n| n.arc.clone()),
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
        mut search_strategy: impl FnMut(SearchData<'_, Ag>) -> SearchDirection,
    ) -> Arc<Self> {
        if self.is_empty() {
            return Self::new_empty();
        }
        let g = self.group();
        let mut left_agg = Ag::default();
        for (i, data) in g.read().unwrap().0.iter().enumerate() {
            use SearchDirection::*;
            match search_strategy(SearchData {
                current_data: &data.node_data,
                left_agg: left_agg.clone(),
                right_agg: self.range_agg(i + 1..),
            }) {
                Found => return data.arc.clone(),
                NotFound | Left => return Self::new_empty(),
                Right => {}
            }
            left_agg = left_agg.merge(Ag::from(&data.node_data));
        }
        Self::new_empty()
    }

    fn change_data(&self, f: impl FnOnce(&mut Ag::Data)) {
        assert!(!self.is_empty());
        let order = self.order();
        let g = self.group();
        f(&mut g.write().unwrap().0[order].node_data);
    }
}

#[derive(Debug)]
pub struct SlowET<Ag: AggregatedData = AggSum>(Arc<SlowBst<ETAggregated<Ag, Weak<SlowET<Ag>>>>>);

static GROUPS2: RwLock<Vec<Arc<RwLock<Group<ETAggregated<AggSum, Weak<SlowET<AggSum>>>>>>>> =
    RwLock::new(vec![]);

impl SlowBstData for ETAggregated<AggSum, Weak<SlowET<AggSum>>> {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<Self>>>>> {
        &GROUPS2
    }
}

impl<Ag: AggregatedData> SlowET<Ag> {
    fn from(bst: Arc<SlowBst<ETAggregated<Ag, Weak<SlowET<Ag>>>>>) -> Arc<Self> {
        let p = Arc::new(Self(bst));
        // Let's cheat because we don't use pointers properly
        unsafe {
            Arc::increment_strong_count(Arc::as_ptr(&p));
        }
        p
    }
}

static GROUPS3: RwLock<Vec<Arc<RwLock<Group<ETAggregated<AgData, Weak<SlowET<AgData>>>>>>>> =
    RwLock::new(vec![]);

impl SlowBstData for ETAggregated<AgData, Weak<SlowET<AgData>>> {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<Self>>>>> {
        &GROUPS3
    }
}

impl<Ag: AggregatedData> ImplicitBST<ETAggregated<Ag, Weak<SlowET<Ag>>>> for SlowET<Ag>
where
    ETAggregated<Ag, Weak<SlowET<Ag>>>: SlowBstData,
{
    fn new_empty() -> Arc<Self> {
        Self::from(SlowBst::new_empty())
    }

    fn new(data: <ETAggregated<Ag, Weak<Self>> as AggregatedData>::Data) -> Arc<Self> {
        Self::from(SlowBst::new(data))
    }

    fn from_iter(
        data: impl IntoIterator<Item = <ETAggregated<Ag, Weak<Self>> as AggregatedData>::Data>,
    ) -> impl Iterator<Item = Arc<Self>> {
        SlowBst::from_iter(data).map(Self::from)
    }

    fn root(&self) -> Arc<Self> {
        Self::from(self.0.root())
    }

    fn node_data(&self) -> &<ETAggregated<Ag, Weak<Self>> as AggregatedData>::Data {
        self.0.node_data()
    }

    fn order(&self) -> usize {
        self.0.order()
    }

    fn range_agg(&self, range: impl RangeBounds<usize>) -> ETAggregated<Ag, Weak<Self>> {
        self.0.range_agg(range)
    }

    fn find_element(
        &self,
        search_strategy: impl FnMut(SearchData<'_, ETAggregated<Ag, Weak<Self>>>) -> SearchDirection,
    ) -> Arc<Self> {
        Self::from(self.0.find_element(search_strategy))
    }

    fn find_kth(&self, k: usize) -> Arc<Self> {
        Self::from(self.0.find_kth(k))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn concat(&self, other: &Self) -> Arc<Self> {
        Self::from(self.0.concat(&other.0))
    }

    fn split(&self, range: impl RangeBounds<usize>) -> (Arc<Self>, Arc<Self>, Arc<Self>) {
        let (a, b, c) = self.0.split(range);
        (Self::from(a), Self::from(b), Self::from(c))
    }

    fn same_node(self: &Arc<Self>, other: &Arc<Self>) -> bool {
        self.0.same_node(&other.0)
    }

    fn change_data(
        &self,
        f: impl FnOnce(&mut <ETAggregated<Ag, Weak<Self>> as AggregatedData>::Data),
    ) {
        self.0.change_data(f);
    }
}
