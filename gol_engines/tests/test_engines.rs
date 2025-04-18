#[cfg(test)]
mod tests {
    use gol_engines::{GoLEngine, HashLifeEngine, SimdEngine, StreamLifeEngine, Topology};

    const SEED: u64 = 42;

    fn randomly_filled(size_log2: u32, seed: u64) -> Vec<Box<dyn GoLEngine>> {
        let engines: Vec<Box<dyn GoLEngine>> = vec![
            Box::new(SimdEngine::random(size_log2, Some(seed))),
            Box::new(HashLifeEngine::random(size_log2, Some(seed))),
            Box::new(StreamLifeEngine::random(size_log2, Some(seed))),
        ];

        assert_fields_equal(&engines);
        engines
    }

    fn assert_fields_equal(engines: &Vec<Box<dyn GoLEngine>>) {
        if engines.is_empty() {
            return;
        }

        let example = engines[0].as_ref();
        for engine in engines.iter().skip(1) {
            assert_eq!(engine.side_length_log2(), example.side_length_log2());
            let n = 1 << engine.side_length_log2();
            if engine.get_cells() == example.get_cells() {
                continue;
            }

            let (mut cells_curr, mut cells_example) = (vec![], vec![]);
            for y in 0..n {
                for x in 0..n {
                    cells_curr.push(engine.get_cell(x, y) as u8);
                    cells_example.push(example.get_cell(x, y) as u8);
                }
            }
            const K: u64 = 10;
            for (i, _) in cells_curr.iter().zip(cells_example.iter()).enumerate() {
                if cells_curr[i] != cells_example[i] {
                    let (x, y) = (i as u64 % n, i as u64 / n);
                    let (x1, y1) = (x.max(K) - K, y.max(K) - K);
                    let (x2, y2) = (x.min(n - K) + K, y.min(n - K) + K);
                    let mut picture = String::new();
                    for y in y1..y2 {
                        picture.push('|');
                        picture.extend(
                            cells_curr[(y * n + x1) as usize..(y * n + x2) as usize]
                                .iter()
                                .map(|&c| if c == 0 { ' ' } else { '#' }),
                        );
                        picture.push('|');
                        picture.extend(
                            cells_example[(y * n + x1) as usize..(y * n + x2) as usize]
                                .iter()
                                .map(|&c| if c == 0 { ' ' } else { '#' }),
                        );
                        picture.push_str("|\n");
                    }
                    panic!("Mismatch at ({}, {}):\n{}", x, y, picture);
                }
            }
        }
    }

    #[test]
    fn test_single_updates() {
        for size_log2 in [7, 9] {
            for steps_log2 in 0..size_log2 {
                let mut engines = randomly_filled(size_log2, SEED);

                for engine in engines.iter_mut() {
                    engine.update(steps_log2, Topology::Torus);
                }

                assert_fields_equal(&engines);
            }
        }
    }

    #[test]
    fn test_repetitive_updates_without_gc() {
        for size_log2 in [7, 9] {
            let mut engines = randomly_filled(size_log2, SEED);

            for steps_log2 in 0..size_log2 {
                for engine in engines.iter_mut() {
                    engine.update(steps_log2, Topology::Torus);
                }

                assert_fields_equal(&engines);
            }
        }
    }

    #[test]
    fn test_repetitive_updates_with_gc() {
        for size_log2 in [7, 9] {
            let mut engines = randomly_filled(size_log2, SEED);

            for steps_log2 in 0..size_log2 {
                for engine in engines.iter_mut() {
                    engine.update(steps_log2, Topology::Torus);
                    engine.run_gc();
                }

                assert_fields_equal(&engines);
            }
        }
    }
}
