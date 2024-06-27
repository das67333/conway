#[cfg(test)]
mod tests {
    use crate::Engine;

    #[test]
    fn test_consistency() {
        use rand::{Rng, SeedableRng};

        const N: u64 = 512;
        const SEED: u64 = 42;
        const FILL_RATE: f64 = 0.6;

        let mut life_simd = crate::PatternObliviousEngine::blank(N.ilog2());
        let mut life_hash = crate::HashLifeEngine::blank(N.ilog2());

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEED);
        for y in 0..N {
            for x in 0..N {
                let state = rng.gen_bool(FILL_RATE);
                life_simd.set_cell(x, y, state);
                life_hash.set_cell(x, y, state);
            }
        }

        life_simd.update(N.ilog2() - 1);
        life_hash.update(N.ilog2() - 1);

        let (mut cells_simd, mut cells_hash) = (vec![], vec![]);
        for y in 0..N {
            for x in 0..N {
                cells_simd.push(life_simd.get_cell(x, y));
                cells_hash.push(life_hash.get_cell(x, y));
            }
        }
        assert_eq!(
            cells_simd.iter().map(|t| *t as usize).sum::<usize>(),
            cells_hash.iter().map(|t| *t as usize).sum::<usize>()
        );
        assert_eq!(cells_simd, cells_hash);
    }
}
