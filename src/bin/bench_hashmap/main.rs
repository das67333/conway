mod memory;
use memory::{Manager, NodeIdx};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn main() {
    let n = 50_000_000;

    let mut mem = Manager::with_capacity((n as usize).next_power_of_two());

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut indices = (0..n)
        .map(|_| [0; 4].map(|_| rng.gen::<u32>()))
        .collect::<Vec<_>>();
    assert_eq!(mem.storage_size - 1, 0);

    let t = std::time::Instant::now();

    for arr in indices.iter() {
        mem.find_node(
            NodeIdx::new(arr[0]),
            NodeIdx::new(arr[1]),
            NodeIdx::new(arr[2]),
            NodeIdx::new(arr[3]),
        );
    }

    println!("{} ns per insert", t.elapsed().as_nanos() / n as u128);
    assert_eq!(mem.storage_size - 1, n as usize);

    indices.shuffle(&mut rng);
    let t = std::time::Instant::now();

    for arr in indices.iter() {
        mem.find_node(
            NodeIdx::new(arr[0]),
            NodeIdx::new(arr[1]),
            NodeIdx::new(arr[2]),
            NodeIdx::new(arr[3]),
        );
    }

    println!("{} ns per find", t.elapsed().as_nanos() / n as u128);
    assert_eq!(mem.storage_size - 1, n as usize);
}
