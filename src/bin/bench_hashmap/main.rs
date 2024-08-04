mod memory;
use memory::{Manager, NodeIdx};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    let n = 1e7 as u32;

    let mut mem = Manager::new();

    let mut rng =  ChaCha8Rng::seed_from_u64(42);
    let indices = (0..n).map(|_| rng.gen::<u32>()).collect::<Vec<_>>();
    assert_eq!(mem.storage_size - 1, 0);

    let t = std::time::Instant::now();

    for i in indices.iter() {
        mem.find_node(
            NodeIdx::new(i + 0),
            NodeIdx::new(i + 1),
            NodeIdx::new(i + 2),
            NodeIdx::new(i + 3),
        );
    }

    println!("{} ns per insert", t.elapsed().as_nanos() / n as u128);
    assert_eq!(mem.storage_size - 1, 8380430 /* n as usize */);

    let t = std::time::Instant::now();

    for i in indices.iter() {
        mem.find_node(
            NodeIdx::new(i + 0),
            NodeIdx::new(i + 1),
            NodeIdx::new(i + 2),
            NodeIdx::new(i + 3),
        );
    }

    println!("{} ns per find", t.elapsed().as_nanos() / n as u128);
    assert_eq!(mem.storage_size - 1, 8380430 /* n as usize */);
}
