#[cfg(test)]
mod tests {
    #[test]
    fn test_consistency() {
        use rand::{Rng, SeedableRng};

        const N: usize = 512;
        const SEED: u64 = 42;
        const FILL_RATE: f64 = 0.6;

        let mut life_simd = ConwayFieldSimd2::blank(N, N);
        let mut life_hash = crate::ConwayFieldHash256::blank(N.ilog2());

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(SEED);
        for y in 0..N {
            for x in 0..N {
                let state = rng.gen_bool(FILL_RATE);
                life_simd.set_cell(x, y, state);
                life_hash.set_cell(x, y, state);
            }
        }

        life_simd.update(N / 2);
        life_hash.update(N / 2);

        let (mut cells_simd, mut cells_hash) =
            (Vec::with_capacity(N * N), Vec::with_capacity(N * N));
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

    type Chunk = u64;

    pub struct ConwayFieldSimd2 {
        data: Vec<Chunk>,
        height: usize,
        width_effective: usize,
    }

    impl ConwayFieldSimd2 {
        const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 8;

        #[target_feature(enable = "avx2")]
        unsafe fn update_row(
            row_prev: &[Chunk],
            row_curr: &[Chunk],
            row_next: &[Chunk],
            dst: &mut [Chunk],
        ) {
            // TODO: double word technique
            let (w, shift) = (row_prev.len(), Self::CELLS_IN_CHUNK - 1);
            let (x, x1, x2) = (0, w - 1, 1);

            let b = row_prev[x];
            let a = (b << 1) | (row_prev[x1] >> shift);
            let c = (b >> 1) | (row_prev[x2] << shift);
            let i = row_curr[x];
            let h = (i << 1) | (row_curr[x1] >> shift);
            let d = (i >> 1) | (row_curr[x2] << shift);
            let f = row_next[x];
            let g = (f << 1) | (row_next[x1] >> shift);
            let e = (f >> 1) | (row_next[x2] << shift);
            let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
            let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
            let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
            let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
            let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
            let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
            let z = !ah23 & ah1;
            let (i2, i3) = (!ah0 & z, ah0 & z);
            dst[x] = (i & i2) | i3;

            for x in 1..w - 1 {
                let (x, x1, x2) = (x, x - 1, x + 1);

                let b = row_prev[x];
                let a = (b << 1) | (row_prev[x1] >> shift);
                let c = (b >> 1) | (row_prev[x2] << shift);
                let i = row_curr[x];
                let h = (i << 1) | (row_curr[x1] >> shift);
                let d = (i >> 1) | (row_curr[x2] << shift);
                let f = row_next[x];
                let g = (f << 1) | (row_next[x1] >> shift);
                let e = (f >> 1) | (row_next[x2] << shift);
                let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
                let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
                let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
                let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
                let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
                let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
                let z = !ah23 & ah1;
                let (i2, i3) = (!ah0 & z, ah0 & z);
                dst[x] = (i & i2) | i3;
            }
            let (x, x1, x2) = (w - 1, w - 2, 0);

            let b = row_prev[x];
            let a = (b << 1) | (row_prev[x1] >> shift);
            let c = (b >> 1) | (row_prev[x2] << shift);
            let i = row_curr[x];
            let h = (i << 1) | (row_curr[x1] >> shift);
            let d = (i >> 1) | (row_curr[x2] << shift);
            let f = row_next[x];
            let g = (f << 1) | (row_next[x1] >> shift);
            let e = (f >> 1) | (row_next[x2] << shift);
            let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
            let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
            let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
            let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
            let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
            let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
            let z = !ah23 & ah1;
            let (i2, i3) = (!ah0 & z, ah0 & z);
            dst[x] = (i & i2) | i3;
        }

        #[target_feature(enable = "avx2")]
        unsafe fn update_inner(&mut self) {
            let (w, h) = (self.width_effective, self.height);
            let mut row_prev = self.data[(h - 1) * w..].to_vec();
            let mut row_curr = self.data[..w].to_vec();
            let row_preserved = row_curr.to_vec();
            let mut row_next = self.data[w..2 * w].to_vec();
            let dst = &mut self.data[..w];
            Self::update_row(&row_prev, &row_curr, &row_next, dst);

            for y in 1..self.height - 1 {
                std::mem::swap(&mut row_prev, &mut row_curr);
                std::mem::swap(&mut row_curr, &mut row_next);
                row_next.copy_from_slice(&self.data[(y + 1) * w..(y + 2) * w]);
                let dst = &mut self.data[y * w..(y + 1) * w];
                Self::update_row(&row_prev, &row_curr, &row_next, dst);
            }

            std::mem::swap(&mut row_prev, &mut row_curr);
            std::mem::swap(&mut row_curr, &mut row_next);
            let dst = &mut self.data[(h - 1) * w..];
            Self::update_row(&row_prev, &row_curr, &row_preserved, dst);
        }

        fn blank(width: usize, height: usize) -> Self {
            assert!(is_x86_feature_detected!("avx2"));
            assert!(width % Self::CELLS_IN_CHUNK == 0);
            let width_effective = width / Self::CELLS_IN_CHUNK;
            assert!(width_effective >= 2 && height >= 2);
            Self {
                data: vec![0; width_effective * height],
                height,
                width_effective,
            }
        }

        fn get_cell(&self, x: usize, y: usize) -> bool {
            let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
            let offset = x % Self::CELLS_IN_CHUNK;
            self.data[pos] >> offset & 1 != 0
        }

        fn set_cell(&mut self, x: usize, y: usize, state: bool) {
            let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
            let mask = 1 << (x % Self::CELLS_IN_CHUNK);
            if state {
                self.data[pos] |= mask;
            } else {
                self.data[pos] &= !mask;
            }
        }

        fn update(&mut self, iters_cnt: usize) {
            for _ in 0..iters_cnt {
                unsafe { self.update_inner() }
            }
        }
    }
}
