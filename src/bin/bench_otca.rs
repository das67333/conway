use conway::{Config, Engine, HashLifeEngine, Topology};
use std::time::Instant;

fn main() {
    let timer = Instant::now();

    let depth = Config::OTCA_DEPTH;
    let top_pattern = Config::TOP_PATTERN.iter().map(|row| row.to_vec()).collect();
    let mut engine = crate::HashLifeEngine::from_recursive_otca_metapixel(depth, top_pattern);
    println!("Time on building field: {:?}", timer.elapsed());

    let timer = Instant::now();
    engine.update(engine.side_length_log2() - 1, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());
}
