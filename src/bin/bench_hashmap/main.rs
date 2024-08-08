#[allow(unused)]
mod memory;
use memory::{Manager, NodeIdx};
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn bench_with_capacity(n: usize, m: usize) -> (f64, f64) {
    let mut mem = Manager::with_capacity(m);

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut indices = (0..n)
        .map(|_| [0; 4].map(|_| rng.gen::<u32>() & (m - 1) as u32))
        .collect::<Vec<_>>();
    assert_eq!(mem.ht_size, 0);

    let t = std::time::Instant::now();

    for arr in indices.iter() {
        mem.find_node(
            NodeIdx::new(arr[0]),
            NodeIdx::new(arr[1]),
            NodeIdx::new(arr[2]),
            NodeIdx::new(arr[3]),
        );
    }

    let ns_per_insert = t.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(mem.ht_size, n as usize);

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

    let ns_per_find = t.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(mem.ht_size, n as usize);

    (ns_per_insert, ns_per_find)
}

fn main() {
    let m = 1usize << 23;
    let k_max = 1;
    for k in 1..=k_max {
        let n = m * k / k_max;
        let (ns_per_insert, ns_per_find) = bench_with_capacity(n, m);
        println!("{} {} {:.2} {:.2}", n, m, ns_per_insert, ns_per_find);
    }
}
