#[cfg(test)]
mod tests {
    use crate::{Engine, HashLifeEngine, PatternObliviousEngine};

    fn randomly_filled(
        n_log2: u32,
        seed: u64,
        fill_rate: f64,
    ) -> (PatternObliviousEngine, HashLifeEngine) {
        use rand::{Rng, SeedableRng};

        let mut life_simd = PatternObliviousEngine::blank(n_log2);
        let mut life_hash = HashLifeEngine::blank(n_log2);

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        let n = 1 << n_log2;
        for y in 0..n {
            for x in 0..n {
                let state = rng.gen_bool(fill_rate);
                life_simd.set_cell(x, y, state);
                life_hash.set_cell(x, y, state);
            }
        }

        assert_fields_equal(&life_simd, &life_hash);
        (life_simd, life_hash)
    }

    fn assert_fields_equal(life_simd: &PatternObliviousEngine, life_hash: &HashLifeEngine) {
        assert_eq!(life_simd.side_length_log2(), life_hash.side_length_log2());
        let n = 1 << life_simd.side_length_log2();
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
    fn test_update_nodes_double() {
        const N_LOG2: u32 = 9;

        let (mut life_simd, mut life_hash) = randomly_filled(N_LOG2, 42, 0.6);

        life_simd.update(N_LOG2 - 1);
        life_hash.update(N_LOG2 - 1);

        assert_fields_equal(&life_simd, &life_hash);
    }

    #[test]
    fn test_update_nodes_single() {
        const N_LOG2: u32 = 9;

        let (mut life_simd, mut life_hash) = randomly_filled(N_LOG2, 42, 0.6);

        life_simd.update(0);
        life_hash.update(0);

        assert_fields_equal(&life_simd, &life_hash);
    }

    #[test]
    fn test_update_nodes_different_steps() {
        const N_LOG2: u32 = 9;

        for step in 0..N_LOG2 {
            let (mut life_simd, mut life_hash) = randomly_filled(N_LOG2, 42, 0.6);

            life_simd.update(step);
            life_hash.update(step);

            assert_fields_equal(&life_simd, &life_hash);
        }
    }
}
