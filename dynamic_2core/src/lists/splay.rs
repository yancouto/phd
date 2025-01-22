use super::{AggregatedData, Idx, Lists};

#[derive(Debug)]
pub struct Splays<Ag> {
    _phantom: std::marker::PhantomData<Ag>,
}

impl<Ag> Lists<Ag> for Splays<Ag>
where
    Ag: AggregatedData,
{
    const EMPTY: Idx = usize::MAX;

    fn new(capacity: usize) -> Self {
        todo!()
    }

    fn create(&mut self, data: Ag::Data) -> Idx {
        todo!()
    }

    fn total_size(&self) -> usize {
        todo!()
    }

    fn root(&self, u: Idx) -> Idx {
        todo!()
    }

    fn data(&self, u: Idx) -> &Ag::Data {
        todo!()
    }

    fn mutate_data(&mut self, u: Idx, f: impl FnOnce(&mut Ag::Data)) {
        todo!()
    }

    fn order(&self, u: Idx) -> usize {
        todo!()
    }

    fn find_element(
        &self,
        u: Idx,
        search_strategy: impl FnMut(super::SearchData<'_, Ag>) -> super::SearchDirection,
    ) -> Idx {
        todo!()
    }

    fn find_kth(&self, u: Idx, k: usize) -> Idx {
        todo!()
    }

    fn len(&self, u: Idx) -> usize {
        todo!()
    }

    fn range_agg_lr(&self, u: Idx, l: usize, r: usize) -> Ag {
        todo!()
    }

    fn concat(&mut self, u: Idx, v: Idx) -> Idx {
        todo!()
    }

    fn split_lr(&mut self, u: Idx, l: usize, r: usize) -> (Idx, Idx, Idx) {
        todo!()
    }

    fn reverse(&mut self, u: Idx) {
        todo!()
    }
}
