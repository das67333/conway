use gol_engines::*;
use num_bigint::BigInt;

fn main() {
    let timer = std::time::Instant::now();
    let data = std::fs::read("../res/very_large_patterns/0e0p-metaglider.mc.gz").unwrap();
    WORKER_THREADS.store(1, std::sync::atomic::Ordering::Relaxed);

    let pattern = Pattern::from_format(PatternFormat::CompressedMacrocell, &data).unwrap();
    let mut engine = HashLifeEngineAsync::new(16 << 10);
    engine.load_pattern(&pattern, Topology::Unbounded).unwrap();
    assert_eq!(pattern.population(), BigInt::from(93_235_805));
    println!("Time spent on building field: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    engine.update(12).unwrap();
    println!("Time on big update: {:?}", timer.elapsed());

    let updated = engine.current_state();
    print!("{}", engine.statistics());
    // assert_eq!(updated.population(), BigInt::from(93_236_670));
    // assert_eq!(updated.hash(), 0x5e1805e773c45a65);
    assert_eq!(updated.population(), BigInt::from(93_237_300));
    assert_eq!(updated.hash(), 206505887300519070);
}
