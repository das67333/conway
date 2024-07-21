#[cfg(test)]
mod tests {
    use conway::{Engine, HashLifeEngine, PatternObliviousEngine, Topology};

    const SEED: u64 = 42;

    fn randomly_filled(n_log2: u32, seed: u64) -> (PatternObliviousEngine, HashLifeEngine) {
        let life_simd = PatternObliviousEngine::random(n_log2, Some(seed));
        let life_hash = HashLifeEngine::random(n_log2, Some(seed));

        assert_fields_equal(&life_simd, &life_hash);
        (life_simd, life_hash)
    }

    fn assert_fields_equal(life_simd: &PatternObliviousEngine, life_hash: &HashLifeEngine) {
        assert_eq!(life_simd.side_length_log2(), life_hash.side_length_log2());
        let n = 1 << life_simd.side_length_log2();
        if life_simd.get_cells() == life_hash.get_cells() {
            return;
        }
        let (mut cells_simd, mut cells_hash) = (vec![], vec![]);
        for y in 0..n {
            for x in 0..n {
                cells_simd.push(life_simd.get_cell(x, y) as u8);
                cells_hash.push(life_hash.get_cell(x, y) as u8);
            }
        }
        const K: u64 = 10;
        for (i, _) in cells_simd.iter().zip(cells_hash.iter()).enumerate() {
            if cells_simd[i] != cells_hash[i] {
                let (x, y) = (i as u64 % n, i as u64 / n);
                let (x1, y1) = (x.max(K) - K, y.max(K) - K);
                let (x2, y2) = (x.min(n - K) + K, y.min(n - K) + K);
                let mut picture = String::new();
                for y in y1..y2 {
                    picture.push('|');
                    picture.extend(
                        cells_simd[(y * n + x1) as usize..(y * n + x2) as usize]
                            .iter()
                            .map(|&c| if c == 0 { ' ' } else { '#' }),
                    );
                    picture.push('|');
                    picture.extend(
                        cells_hash[(y * n + x1) as usize..(y * n + x2) as usize]
                            .iter()
                            .map(|&c| if c == 0 { ' ' } else { '#' }),
                    );
                    picture.push_str("|\n");
                }
                panic!("Mismatch at ({}, {}):\n{}", x, y, picture,);
            }
        }
    }

    #[test]
    fn test_update_nodes() {
        for n_log2 in [7, 9] {
            for step in 0..n_log2 {
                let (mut life_simd, mut life_hash) = randomly_filled(n_log2, SEED);

                life_simd.update(step, Topology::Torus);
                life_hash.update(step, Topology::Torus);

                assert_fields_equal(&life_simd, &life_hash);
            }
        }
    }
}
