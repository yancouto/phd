//! This project implements a dynamic 2-core solver for graphs. The solver supports dynamic addition and removal of edges while maintaining the 2-core structure of the graph. See the trait [Dynamic2CoreSolver].
//!
//! The 2-core of a graph is its maximal subgraph where all vertices have degree at least 2. This solver also implements connectivity queries using the HDT algorithm.
//!
//! All operations with [FastDynamic2CoreSolver] take O(lg² n) amortized time.
//!
//! ## Usage
//!
//! To use the dynamic 2-core solver, create an instance using [FastDynamic2CoreSolver::new] and use the methods provided on [Dynamic2CoreSolver] to add, remove edges and query the graph.
//!
//! ```
//! use dynamic_2core::{FastDynamic2CoreSolver, Dynamic2CoreSolver};
//!
//! let mut solver = FastDynamic2CoreSolver::new(10);
//! solver.add_edge(1, 2);
//! solver.add_edge(2, 3);
//! assert!(solver.is_connected(1, 3));
//! assert!(!solver.is_in_2core(2));
//! solver.add_edge(1, 3);
//! assert!(solver.is_in_2core(2));
//! ```
//!
//! You can see example usage at `src/bin/example.rs` and run it with `cargo run`.
//!
//! ## Implementation
//!
//! This uses Euler Tour Trees and Link Cut Trees, which in turn also use Treaps. All data structures can be used independently of the 2-core solver. If we use Splay Trees on the Link Cut Trees instead of Treaps, the time complexity of all operations except `remove_edge` can be reduced to amortized O(lg n).
//!
//! To read the implementation of the algorithm, see `impl Dynamic2CoreSolve for D2CSolver` in `src/dynamic_2core.rs`.
//! For the data structures:
//! - Treaps: see `impl Lists for Treaps` in `src/lists/treap.rs`.
//! - Link Cut Tree: see `impl LinkCutTree for LCT` in `src/link_cut_tree.rs`.
//! - Euler Tour Tree: see `impl EulerTourTree for ETT` in `src/euler_tour_tree.rs`.
//!
//! ## Testing
//!
//! Run the tests using:
//!
//! ```skip
//! cargo test
//! ```
//!
//! Add `-- --ignored` to run the stress tests (which run indefinitely with random data until they fail).
pub mod dynamic_2core;
pub mod euler_tour_tree;
pub mod link_cut_tree;
pub mod lists;
pub use dynamic_2core::Dynamic2CoreSolver;

use dynamic_2core::{AgData, D2CSolver};
use euler_tour_tree::ETT;
use link_cut_tree::LCT;
use lists::treap::Treaps;

/// The fastest implemented solver for dynamic 2-core on this crate. It uses Link Cut Trees and Euler Tour Trees with Treaps.
pub type FastDynamic2CoreSolver = D2CSolver<ETT<Treaps<AgData>, AgData>, LCT<Treaps>>;
