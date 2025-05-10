use gol_engines::*;
use num_bigint::BigInt;
use std::sync::atomic::*;

fn main() {
    for x in 10..=15 {
        MIN_COROUTINE_SPAWN_SIZE_LOG2.store(x, Ordering::Relaxed);
        println!(
            "MIN_COROUTINE_SPAWN_SIZE_LOG2: {}",
            MIN_COROUTINE_SPAWN_SIZE_LOG2.load(Ordering::Relaxed)
        );
        let timer = std::time::Instant::now();
        let data = std::fs::read("../res/very_large_patterns/0e0p-metaglider.mc.gz").unwrap();
        WORKER_THREADS.store(16, std::sync::atomic::Ordering::Relaxed);

        let pattern = Pattern::from_format(PatternFormat::CompressedMacrocell, &data).unwrap();
        let mut engine = HashLifeEngineAsync::new(16 << 10);
        engine.load_pattern(&pattern, Topology::Unbounded).unwrap();
        assert_eq!(pattern.population(), BigInt::from(93_235_805));
        println!("Time spent on building field: {:?}", timer.elapsed());

        let timer = std::time::Instant::now();
        engine.update(12).unwrap();
        println!("Time on big update: {:?}", timer.elapsed());

        let timer = std::time::Instant::now();
        engine.run_gc();
        println!("Time on GC: {:?}", timer.elapsed());

        let updated = engine.current_state();
        println!(
            "COROUTINES_SPAWN_COUNT: {}",
            COROUTINES_SPAWN_COUNT.load(std::sync::atomic::Ordering::Relaxed)
        );
        println!(
            "NODES_CREATED_COUNT: {}",
            NODES_CREATED_COUNT.load(std::sync::atomic::Ordering::Relaxed)
        );
        // assert_eq!(updated.population(), BigInt::from(93_236_670));
        // assert_eq!(updated.hash(), 0x5e1805e773c45a65);
        assert_eq!(updated.population(), BigInt::from(93_237_300));
        assert_eq!(updated.hash(), 206505887300519070);
        COROUTINES_SPAWN_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
        NODES_CREATED_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}
