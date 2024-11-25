use std::sync::{Arc, RwLock};

use dynamic_2core::implicit_bst::AggregatedData;
use slow_bst::{Group, SlowBstData};

pub mod slow_bst;

#[derive(Debug, Clone, Default)]
pub struct AggSum(pub i32);

impl AggregatedData for AggSum {
    type Data = i32;

    fn from(data: &Self::Data) -> Self {
        Self(*data)
    }

    fn merge(self, right: Self) -> Self {
        Self(self.0 + right.0)
    }
}

static GROUPS: RwLock<Vec<Arc<RwLock<Group<AggSum>>>>> = RwLock::new(vec![]);

impl SlowBstData for AggSum {
    fn map() -> &'static RwLock<Vec<Arc<RwLock<Group<AggSum>>>>> {
        &GROUPS
    }
}

impl PartialEq<i32> for AggSum {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}
