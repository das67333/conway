use gol_engines::*;
use num_bigint::BigInt;

fn main() {
    let timer = std::time::Instant::now();
    set_memory_manager_cap_log2(26);
    let data = std::fs::read("../res/0e0p-metaglider.mc").unwrap();

    let pattern = Pattern::from_format(PatternFormat::Macrocell, &data).unwrap();
    let mut engine = HashLifeEngineSmall::from_pattern(&pattern, Topology::Unbounded).unwrap();
    assert_eq!(pattern.population(), BigInt::from(93_235_805));
    println!("Time spent on building field: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let generations_log2 = 10;
    engine.update(generations_log2);
    println!("Time on big update: {:?}", timer.elapsed());

    let updated = engine.current_state();
    print!("{}", engine.statistics());
    assert_eq!(updated.population(), BigInt::from(93_236_670));
    assert_eq!(updated.hash(), 0x5e1805e773c45a65);
}
