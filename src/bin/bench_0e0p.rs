use conway::{DefaultEngine, Engine, Topology};
use std::time::Instant;

fn main() {
    // let timer = Instant::now();
    let data = std::fs::read("res/0e0p-metaglider.mc").unwrap();

    for steps_log2 in 0..=19 {
        let mut engine = DefaultEngine::from_macrocell(&data);
        // println!("Time spent on building field: {:?}", timer.elapsed());

        let timer = Instant::now();
        engine.update(steps_log2, Topology::Unbounded);
        println!("Time spent on update: {:?}", timer.elapsed());
        println!("{}", engine.population());
    }
}
