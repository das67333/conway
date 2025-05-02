#[cfg(test)]
mod tests {
    use gol_engines::*;

    fn build_engines() -> Vec<Box<dyn GoLEngine>> {
        let data = std::fs::read("../res/otca_0.rle").unwrap();
        let pattern = Pattern::from_format(PatternFormat::RLE, &data).unwrap();
        let engines: Vec<Box<dyn GoLEngine>> = vec![
            Box::new(SIMDEngine::from_pattern(&pattern, Topology::Torus).unwrap()),
            Box::new(HashLifeEngineSmall::from_pattern(&pattern, Topology::Torus).unwrap()),
            Box::new(HashLifeEngineSync::from_pattern(&pattern, Topology::Torus).unwrap()),
            Box::new(HashLifeEngineAsync::from_pattern(&pattern, Topology::Torus).unwrap()),
        ];

        assert_fields_equal(&engines);
        engines
    }

    fn assert_fields_equal(engines: &Vec<Box<dyn GoLEngine>>) {
        let first = engines[0].current_state().hash();
        for engine in engines.iter().skip(1) {
            assert_eq!(engine.current_state().hash(), first, "Fields do not match");
        }
    }

    #[test]
    fn test_single_updates() {
        for size_log2 in [7, 9] {
            for generations_log2 in 0..size_log2 {
                let mut engines = build_engines();

                for engine in engines.iter_mut() {
                    engine.update(generations_log2);
                }

                assert_fields_equal(&engines);
            }
        }
    }

    #[test]
    fn test_repetitive_updates_without_gc() {
        for size_log2 in [7, 9, 8] {
            let mut engines = build_engines();

            for generations_log2 in 0..size_log2 {
                for engine in engines.iter_mut() {
                    engine.update(generations_log2);
                }

                assert_fields_equal(&engines);
            }
        }
    }

    #[test]
    fn test_repetitive_updates_with_gc() {
        for size_log2 in [7, 9, 8] {
            let mut engines = build_engines();

            for generations_log2 in 0..size_log2 {
                for engine in engines.iter_mut() {
                    engine.update(generations_log2);
                    engine.run_gc();
                }

                assert_fields_equal(&engines);
            }
        }
    }
}
