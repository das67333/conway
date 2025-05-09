use gol_engines::*;
use num_bigint::BigInt;

fn main() {
    for x in 15..20 {
        let timer = std::time::Instant::now();
        let data = std::fs::read("../res/very_large_patterns/0e0p-metaglider.mc.gz").unwrap();
        WORKER_THREADS.store(16, std::sync::atomic::Ordering::Relaxed);

        let pattern = Pattern::from_format(PatternFormat::CompressedMacrocell, &data).unwrap();
        let mut engine = HashLifeEngineAsync::new(16 << 10);
        unsafe {
            crate::OPTION_1 = x;
        }
        engine.load_pattern(&pattern, Topology::Unbounded).unwrap();
        assert_eq!(pattern.population(), BigInt::from(93_235_805));
        println!("Time spent on building field: {:?}", timer.elapsed());

        println!("OPTION_1: {}", unsafe { crate::OPTION_1 });
        let timer = std::time::Instant::now();
        engine.update(12).unwrap();
        println!("Time on big update: {:?}", timer.elapsed());

        let updated = engine.current_state();
        println!(
            "COUNTER_1: {}",
            COUNTER_1.load(std::sync::atomic::Ordering::Relaxed)
        );
        // assert_eq!(updated.population(), BigInt::from(93_236_670));
        // assert_eq!(updated.hash(), 0x5e1805e773c45a65);
        assert_eq!(updated.population(), BigInt::from(93_237_300));
        assert_eq!(updated.hash(), 206505887300519070);
        crate::COUNTER_1.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}
