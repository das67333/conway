use conway::{Config, Engine, HashLifeEngine};
use std::time::Instant;

fn main() {
    let timer = Instant::now();
    // be careful with deadlocks
    let depth = Config::get().otca_depth;
    let top_pattern = Config::get().top_pattern.clone();
    let mut engine = crate::HashLifeEngine::from_recursive_otca_metapixel(depth, top_pattern);
    println!("Time to build engine: {:?}", timer.elapsed());

    let timer = Instant::now();
    engine.update(engine.side_length_log2() - 1);
    println!("Time on big update: {:?}", timer.elapsed());
}
