# Dynamic 2-Core Solver

This project implements a dynamic 2-core solver for graphs. The solver supports dynamic addition and removal of edges while maintaining the 2-core structure of the graph. See the trait `Dynamic2CoreSolver`.

The 2-core of a graph is its maximal subgraph where all vertices have degree at least 2. This solver also implements connectivity queries using the HDT algorithm.

All operations take O(lg n) amortized time, except `remove_edge` which takes O(lg² n).

## Usage

To use the dynamic 2-core solver, create an instance using `FastDynamic2CoreSolver::new` and use the methods provided on `Dynamic2CoreSolver` to add, remove edges and query the graph. 

```rust
use dynamic_2core::{FastDynamic2CoreSolver, Dynamic2CoreSolver};

let mut solver = FastDynamic2CoreSolver::new(10);
solver.add_edge(1, 2);
solver.add_edge(2, 3);
assert!(solver.is_connected(1, 3));
assert!(!solver.is_in_2core(2));
solver.add_edge(1, 3);
assert!(solver.is_in_2core(2));
```

You can see example usage at `src/bin/example.rs` and run it with `cargo run`.

## Implementation

This uses Euler Tour Trees and Link Cut Trees, which in turn also use Splay Trees and Treaps (Cartesian Trees). All data structures can be used independently of the 2-core solver. In theory Link Cut Trees only have improved time guarantees (O(lg n) vs O(lg² n)) when used with Splay Trees, but in practice they work faster with Treaps.

To read the implementation of the algorithm, see `impl Dynamic2CoreSolve for D2CSolver` in `src/dynamic_2core.rs`.
For the data structures:
- Treaps: see `impl Lists for Treaps` in `src/lists/treap.rs`.
- Splay Trees: see `impl Lists for Splays` in `src/lists/splay.rs`.
- Link Cut Tree: see `impl LinkCutTree for LCT` in `src/link_cut_tree.rs`.
- Euler Tour Tree: see `impl EulerTourTree for ETT` in `src/euler_tour_tree.rs`.

## Testing

Run the tests using:

```sh
cargo test
```

Add `-- --ignored` to run the stress tests (which run indefinitely with random data until they fail). And use `cargo bench` to see the benchmarks.