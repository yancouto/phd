# Dynamic 2-Core Solver

This project implements a dynamic 2-core solver for graphs. The solver supports dynamic addition and removal of edges while maintaining the 2-core structure of the graph. See the trait `Dynamic2CoreSolver`.

The 2-core of a graph is its maximal subgraph where all vertices have degree at least 2. This solver also implements connectivity queries using the HDT algorithm.

All operations take O(lg² n) amortized time.

## Usage

To use the dynamic 2-core solver, create an instance using the `dynamic_2core_solver` function and use the provided methods to add or remove edges and query the graph. 

```rust
use dynamic_2core::{dynamic_2core_solver, Dynamic2CoreSolver};

let mut solver = dynamic_2core_solver(10);
solver.add_edge(1, 2);
solver.add_edge(2, 3);
assert!(solver.is_connected(1, 3));
assert!(!solver.is_in_2core(2));
solver.add_edge(1, 3);
assert!(solver.is_in_2core(2));
```

You can see example usage at `src/bin/example.rs` and run it with `cargo run`.

## Implementation

This uses Euler Tour Trees and Link Cut Trees, which in turn also use Treaps. All data structures can be used independently from the 2-core solver. If we use Splay Trees on the Link Cut Trees instead of Treaps, the time complexity of all operations except `remove_edge` can be reduced to amortized O(lg n).

## Testing

Run the tests using:

```sh
cargo test
```

Add `-- --ignored` to run the stress tests (which run indefinitely with random data until they fail).