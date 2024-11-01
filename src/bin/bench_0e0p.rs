use conway::{DefaultEngine, Engine, NiceInt, Topology};
use std::time::Instant;

fn main() {
    // let timer = Instant::now();
    let data = std::fs::read("res/0e0p-metaglider.mc").unwrap();

    for steps_log2 in 22..=26 {
        let mut engine = DefaultEngine::from_macrocell(&data);
        // println!("Time spent on building field: {:?}", timer.elapsed());

        let timer = Instant::now();
        engine.update(steps_log2, Topology::Unbounded);
        let elapsed = timer.elapsed();
        println!(
            "steps_log2={}\tpopulation={}\tmemory_mb={}\ttime={:?}",
            steps_log2,
            NiceInt::from_f64(engine.population()),
            engine.bytes_total() >> 20,
            elapsed
        );
        println!("{}", engine.statistics());
    }
}
