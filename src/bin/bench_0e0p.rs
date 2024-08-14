use conway::{Engine, HashLifeEngine, Topology};
use std::time::Instant;

fn main() {
    const STEPS_LOG2: u32 = 12;

    let timer = Instant::now();

    let data = std::fs::read("res/0e0p-metaglider.mc").unwrap();
    let mut engine = crate::HashLifeEngine::from_macrocell(&data);
    println!("Time on building field: {:?}", timer.elapsed());

    let timer = Instant::now();
    engine.update(STEPS_LOG2, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());

    println!("{}", engine.stats_fast());
}
