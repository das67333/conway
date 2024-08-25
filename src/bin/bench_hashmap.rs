use conway::quadtree::*;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

fn bench_with_capacity(n: usize, m: usize) -> (f64, f64) {
    let mut mem = MemoryManager::with_capacity(m);

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut indices = (0..n)
        .map(|_| [0; 4].map(|_| rng.gen::<u32>() & (m - 1) as u32))
        .collect::<Vec<_>>();
    assert_eq!(mem.ht_size, 0);

    let t = std::time::Instant::now();

    for arr in indices.iter() {
        mem.find_node(
            NodeIdx(arr[0]),
            NodeIdx(arr[1]),
            NodeIdx(arr[2]),
            NodeIdx(arr[3]),
        );
    }

    let ns_per_insert = t.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(mem.ht_size, n);
    println!("{}", mem.stats_fast());
    println!("{:/<1$}", "", 64);

    indices.shuffle(&mut rng);
    let t = std::time::Instant::now();

    for arr in indices.iter() {
        mem.find_node(
            NodeIdx(arr[0]),
            NodeIdx(arr[1]),
            NodeIdx(arr[2]),
            NodeIdx(arr[3]),
        );
    }

    let ns_per_find = t.elapsed().as_secs_f64() * 1e9 / n as f64;
    assert_eq!(mem.ht_size, n);

    (ns_per_insert, ns_per_find)
}

fn main() {
    let m = 1usize << 26;
    for k in 7..=7 {
        let n = m * k / 10;
        let (ns_per_insert, ns_per_find) = bench_with_capacity(n, m);
        println!("{} {} {:.2} {:.2}", n, m, ns_per_insert, ns_per_find);
    }
}
