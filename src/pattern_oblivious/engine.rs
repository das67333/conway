use crate::{Engine, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};

pub struct PatternObliviousEngine {
    data: Vec<u64>,
    n: u64,
}

impl PatternObliviousEngine {
    const CELLS_IN_CHUNK: u64 = 64;

    fn update_row(row_prev: &[u64], row_curr: &[u64], row_next: &[u64], dst: &mut [u64]) {
        // TODO: double word technique
        // TODO: use avx2 if available
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

    fn update_inner(&mut self) {
        let (w, h) = (
            self.n as usize >> Self::CELLS_IN_CHUNK.ilog2(),
            self.n as usize,
        );
        let mut row_prev = self.data[(h - 1) * w..].to_vec();
        let mut row_curr = self.data[..w].to_vec();
        let row_preserved = row_curr.to_vec();
        let mut row_next = self.data[w..2 * w].to_vec();
        let dst = &mut self.data[..w];
        Self::update_row(&row_prev, &row_curr, &row_next, dst);

        for y in 1..self.n as usize - 1 {
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
}

impl Engine for PatternObliviousEngine {
    fn blank(n_log2: u32) -> Self {
        assert!((MIN_SIDE_LOG2..=MAX_SIDE_LOG2).contains(&n_log2));
        let n: u64 = 1 << n_log2;
        Self {
            data: vec![0; 1 << (n_log2 * 2 - Self::CELLS_IN_CHUNK.ilog2())],
            n,
        }
    }

    fn from_macrocell(_data: &[u8]) -> Self {
        unimplemented!()
    }

    fn from_cells(n_log2: u32, cells: Vec<u64>) -> Self
    where
        Self: Sized,
    {
        assert_eq!(
            cells.len(),
            1 << (n_log2 * 2 - Self::CELLS_IN_CHUNK.ilog2())
        );
        Self {
            data: cells,
            n: 1 << n_log2,
        }
    }

    fn save_into_macrocell(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn get_cells(&self) -> Vec<u64> {
        self.data.clone()
    }

    fn side_length_log2(&self) -> u32 {
        self.n.ilog2()
    }

    fn get_cell(&self, x: u64, y: u64) -> bool {
        let pos = (x + y * self.n) >> Self::CELLS_IN_CHUNK.ilog2();
        let offset = x & (Self::CELLS_IN_CHUNK - 1);
        self.data[pos as usize] >> offset & 1 != 0
    }

    fn set_cell(&mut self, x: u64, y: u64, state: bool) {
        let pos = (x + y * self.n) >> Self::CELLS_IN_CHUNK.ilog2();
        let mask = 1 << (x & (Self::CELLS_IN_CHUNK - 1));
        if state {
            self.data[pos as usize] |= mask;
        } else {
            self.data[pos as usize] &= !mask;
        }
    }

    fn update(&mut self, steps_log2: u32, topology: Topology) -> [u64; 2] {
        assert!(
            matches!(topology, Topology::Torus),
            "not supported ty this engine"
        );
        for _ in 0..1u64 << steps_log2 {
            self.update_inner();
        }
        [0; 2]
    }

    fn fill_texture(
        &self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    ) {
        *viewport_x = 0.;
        *viewport_y = 0.;
        *size = self.n as f64;
        *resolution = self.n as f64;
        dst.clear();
        dst.resize((self.n.max(64) * self.n) as usize, 0.);
        for y in 0..self.n {
            for x in 0..self.n {
                dst[(x + y * self.n) as usize] = self.get_cell(x, y) as u8 as f64;
            }
        }
    }

    fn stats_fast(&self) -> String {
        format!("memory on field: {} bytes", self.data.len() * 8)
    }
}

impl Default for PatternObliviousEngine {
    fn default() -> Self {
        Self::blank(MIN_SIDE_LOG2)
    }
}
