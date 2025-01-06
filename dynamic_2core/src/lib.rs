pub mod dynamic_2core;
pub mod euler_tour_tree;
pub mod link_cut_tree;
pub mod lists;
pub use dynamic_2core::Dynamic2CoreSolver;

/// Return the fastest implemented solver for an empty graph on n nodes.
pub fn dynamic_2core_solver(n: usize) -> impl Dynamic2CoreSolver {
    use dynamic_2core::{AgData, D2CSolver};
    use euler_tour_tree::ETT;
    use link_cut_tree::LCT;
    use lists::treap::Treaps;
    D2CSolver::<ETT<Treaps<AgData>, AgData>, LCT<Treaps>>::new(n)
}
