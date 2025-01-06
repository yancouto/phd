pub mod dynamic_2core;
pub mod euler_tour_tree;
pub mod link_cut_tree;
pub mod lists;
pub use dynamic_2core::Dynamic2CoreSolver;

use dynamic_2core::{AgData, D2CSolver};
use euler_tour_tree::ETT;
use link_cut_tree::LCT;
use lists::treap::Treaps;

/// The fastest implemented solver for dynamic 2-core on this crate.
pub type FastDynamic2CoreSolver = D2CSolver<ETT<Treaps<AgData>, AgData>, LCT<Treaps>>;
