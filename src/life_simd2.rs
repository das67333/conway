type Chunk = u64;

pub struct ConwayField {
    data_curr: Vec<Chunk>,
    data_next: Vec<Chunk>,
    width: usize,
    height: usize,
    width_effective: usize,
}

impl ConwayField {
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 8;

    #[target_feature(enable = "avx2")]
    unsafe fn update_row(&mut self, y: usize, y1: usize, y2: usize) {
        let shift = Self::CELLS_IN_CHUNK - 1;

        let (x, x1, x2) = (0, self.width_effective - 1, 1);

        let b = self.data_curr[x + y1 * self.width_effective];
        let a = (b << 1) | (self.data_curr[x1 + y1 * self.width_effective] >> shift);
        let c = (b >> 1) | (self.data_curr[x2 + y1 * self.width_effective] << shift);
        let i = self.data_curr[x + y * self.width_effective];
        let h = (i << 1) | (self.data_curr[x1 + y * self.width_effective] >> shift);
        let d = (i >> 1) | (self.data_curr[x2 + y * self.width_effective] << shift);
        let f = self.data_curr[x + y2 * self.width_effective];
        let g = (f << 1) | (self.data_curr[x1 + y2 * self.width_effective] >> shift);
        let e = (f >> 1) | (self.data_curr[x2 + y2 * self.width_effective] << shift);

        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);

        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);

        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);

        self.data_next[x + y * self.width_effective] = (i & i2) | i3;

        for x in 1..self.width_effective - 1 {
            let (x, x1, x2) = (x, x - 1, x + 1);

            let b = self.data_curr[x + y1 * self.width_effective];
            let a = (b << 1) | (self.data_curr[x1 + y1 * self.width_effective] >> shift);
            let c = (b >> 1) | (self.data_curr[x2 + y1 * self.width_effective] << shift);
            let i = self.data_curr[x + y * self.width_effective];
            let h = (i << 1) | (self.data_curr[x1 + y * self.width_effective] >> shift);
            let d = (i >> 1) | (self.data_curr[x2 + y * self.width_effective] << shift);
            let f = self.data_curr[x + y2 * self.width_effective];
            let g = (f << 1) | (self.data_curr[x1 + y2 * self.width_effective] >> shift);
            let e = (f >> 1) | (self.data_curr[x2 + y2 * self.width_effective] << shift);

            let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
            let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);

            let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
            let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);

            let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
            let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
            let z = !ah23 & ah1;
            let (i2, i3) = (!ah0 & z, ah0 & z);

            self.data_next[x + y * self.width_effective] = (i & i2) | i3;
        }
        let (x, x1, x2) = (self.width_effective - 1, self.width_effective - 2, 0);

        let b = self.data_curr[x + y1 * self.width_effective];
        let a = (b << 1) | (self.data_curr[x1 + y1 * self.width_effective] >> shift);
        let c = (b >> 1) | (self.data_curr[x2 + y1 * self.width_effective] << shift);
        let i = self.data_curr[x + y * self.width_effective];
        let h = (i << 1) | (self.data_curr[x1 + y * self.width_effective] >> shift);
        let d = (i >> 1) | (self.data_curr[x2 + y * self.width_effective] << shift);
        let f = self.data_curr[x + y2 * self.width_effective];
        let g = (f << 1) | (self.data_curr[x1 + y2 * self.width_effective] >> shift);
        let e = (f >> 1) | (self.data_curr[x2 + y2 * self.width_effective] << shift);

        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);

        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);

        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);

        self.data_next[x + y * self.width_effective] = (i & i2) | i3;
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_inner(&mut self) {
        for y in 0..self.height {
            let y1 = self.height * (y == 0) as usize + y - 1;
            let y2 = y + 1 - self.height * (y == self.height - 1) as usize;
            self.update_row(y, y1, y2);
        }
        std::mem::swap(&mut self.data_curr, &mut self.data_next);
    }
}

impl crate::CellularAutomaton for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert!(width % Self::CELLS_IN_CHUNK == 0);
        let width_effective = width / Self::CELLS_IN_CHUNK;
        // assert!(width_effective >= 2 && height >= 2); todo
        let size = width_effective * height;
        Self {
            data_curr: vec![0; size],
            data_next: vec![0; size],
            width,
            height,
            width_effective,
        }
    }

    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_cell(&self, x: usize, y: usize) -> bool {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset = x % Self::CELLS_IN_CHUNK;
        self.data_curr[pos] >> offset & 1 != 0
    }

    fn get_cells(&self) -> Vec<bool> {
        self.data_curr
            .iter()
            .flat_map(|x| (0..Self::CELLS_IN_CHUNK).map(|i| (*x >> i & 1 != 0)))
            .collect()
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let mask = 1 << x % Self::CELLS_IN_CHUNK;
        if state {
            self.data_curr[pos] |= mask;
        } else {
            self.data_curr[pos] &= !mask;
        }
    }

    fn set_cells(&mut self, states: &[bool]) {
        assert_eq!(states.len(), self.width * self.height);
        for (dst, src) in self
            .data_curr
            .iter_mut()
            .zip(states.chunks_exact(Self::CELLS_IN_CHUNK))
        {
            *dst = src
                .iter()
                .enumerate()
                .map(|(i, &x)| (x as Chunk) << i)
                .sum::<Chunk>();
        }
    }

    fn update(&mut self, iters_cnt: usize) {
        for _ in 0..iters_cnt {
            unsafe { self.update_inner() }
        }
    }
}
