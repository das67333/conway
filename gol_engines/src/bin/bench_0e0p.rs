use gol_engines::*;

fn main() {
    let timer = std::time::Instant::now();
    let data = std::fs::read("../res/0e0p-metaglider.mc").unwrap();

    let mut engine = HashLifeEngineAsync::from_macrocell(&data);
    println!("Time spent on building field: {:?}", timer.elapsed());
    assert_eq!(engine.population(), 93235805.0);

    let timer = std::time::Instant::now();
    let steps_log2 = 10;
    engine.update(steps_log2, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());
    eprintln!("{}", engine.statistics());
    assert_eq!(engine.hash(), 0x5e1805e773c45a65);
    assert_eq!(engine.population(), 93_236_670.0);
}
