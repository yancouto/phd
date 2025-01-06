use dynamic_2core::{dynamic_2core_solver, Dynamic2CoreSolver};

fn add_edge(t: &mut impl Dynamic2CoreSolver, u: usize, v: usize) {
    println!("Adding edge from {} to {}", u, v);
    t.add_edge(u, v);
}

fn rem_edge(t: &mut impl Dynamic2CoreSolver, u: usize, v: usize) {
    println!("Removing edge from {} to {}", u, v);
    t.remove_edge(u, v);
}

fn is_2_core(t: &mut impl Dynamic2CoreSolver, u: usize) {
    println!(
        "Is {} in the 2-core? {}",
        u,
        if t.is_in_2core(u) { "Yes" } else { "No" }
    );
}

fn main() {
    let mut t = dynamic_2core_solver(10);
    for u in 0..9 {
        t.add_edge(u, u + 1);
    }
    println!("Created a path of length 10 (vertices 0 to 9)");
    is_2_core(&mut t, 2);
    add_edge(&mut t, 0, 3);
    is_2_core(&mut t, 2);
    is_2_core(&mut t, 4);
    add_edge(&mut t, 7, 9);
    is_2_core(&mut t, 4);
    rem_edge(&mut t, 3, 0);
    is_2_core(&mut t, 4);
    is_2_core(&mut t, 7);
}
