mod memory;
use memory::{Manager, NodeIdx};

fn main() {
    let n = 1e7 as u32;

    let mut mem = Manager::new();

    let t = std::time::Instant::now();

    for i in 0..n {
        mem.find_node(
            NodeIdx::new(i),
            NodeIdx::new(i),
            NodeIdx::new(i),
            NodeIdx::new(i),
        );
    }

    println!("{} ns per insert", t.elapsed().as_nanos() / n as u128);

    let t = std::time::Instant::now();

    for i in 0..n {
        mem.find_node(
            NodeIdx::new(i),
            NodeIdx::new(i),
            NodeIdx::new(i),
            NodeIdx::new(i),
        );
    }

    println!("{} ns per find", t.elapsed().as_nanos() / n as u128);
}
