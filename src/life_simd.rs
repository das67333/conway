type Chunk = u64;

pub struct ConwayField {
    data: Vec<Chunk>,
    width: usize,
    height: usize,
    width_effective: usize,
}

impl ConwayField {
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;
    const BITS_PER_CELL: usize = 4;

    /// 0xE -> 0xEEEEE...
    const fn repeat_hex(x: u8) -> Chunk {
        assert!(x <= 0x10);
        Chunk::from_ne_bytes([x * 0x11; std::mem::size_of::<Chunk>()])
    }

    #[target_feature(enable = "avx2")]
    unsafe fn calc_sums(&self, sums: &mut [Chunk], y: usize) {
        let w = self.width_effective;
        let row = &self.data[y * w..(y + 1) * w];
        let shift_1 = Self::BITS_PER_CELL;
        let shift_2 = (Self::CELLS_IN_CHUNK - 1) * Self::BITS_PER_CELL;

        let (prev, curr, next) = (
            row.get_unchecked(w - 1),
            row.get_unchecked(0),
            row.get_unchecked(1),
        );
        *sums.get_unchecked_mut(0) =
            curr + (curr >> shift_1) + (curr << shift_1) + (prev >> shift_2) + (next << shift_2);
        for x in 1..w - 1 {
            let (prev, curr, next) = (
                row.get_unchecked(x - 1),
                row.get_unchecked(x),
                row.get_unchecked(x + 1),
            );
            *sums.get_unchecked_mut(x) = curr
                + (curr >> shift_1)
                + (curr << shift_1)
                + (prev >> shift_2)
                + (next << shift_2);
        }
        let (prev, curr, next) = (
            row.get_unchecked(w - 2),
            row.get_unchecked(w - 1),
            row.get_unchecked(0),
        );
        *sums.get_unchecked_mut(w - 1) =
            curr + (curr >> shift_1) + (curr << shift_1) + (prev >> shift_2) + (next << shift_2);
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_row(
        &mut self,
        sums_prev: &[Chunk],
        sums_curr: &[Chunk],
        sums_next: &[Chunk],
        y: usize,
    ) {
        let w = self.width_effective;
        let row = &mut self.data[y * w..(y + 1) * w];
        for x in 0..w {
            let neighbours = sums_prev.get_unchecked(x)
                + sums_curr.get_unchecked(x)
                + sums_next.get_unchecked(x)
                - row.get_unchecked(x);

            let mask = neighbours | (row.get_unchecked(x) << 3);
            let keep = {
                let (m0, m2) = (mask, mask >> 2);
                !m2 & (m0 & m2) >> 1
            };
            let create = {
                let mut temp = mask ^ Self::repeat_hex(0b1100);
                temp &= temp >> 2;
                temp &= temp >> 1;
                temp
            };
            *row.get_unchecked_mut(x) = (keep | create) & Self::repeat_hex(0b0001);
        }
    }

    unsafe fn update_inner(&mut self) {
        let (mut sums_prev, mut sums_curr, mut sums_next) = (
            vec![0; self.width_effective],
            vec![0; self.width_effective],
            vec![0; self.width_effective],
        );
        self.calc_sums(&mut sums_prev, self.height - 1);
        self.calc_sums(&mut sums_curr, 0);
        let mut preserved = sums_curr.clone();
        self.calc_sums(&mut sums_next, 1);
        self.update_row(&sums_prev, &sums_curr, &sums_next, 0);

        for y in 1..(self.height - 1) {
            std::mem::swap(&mut sums_prev, &mut sums_curr);
            std::mem::swap(&mut sums_curr, &mut sums_next);
            self.calc_sums(&mut sums_next, y + 1);
            self.update_row(&sums_prev, &sums_curr, &sums_next, y);
        }
        std::mem::swap(&mut sums_prev, &mut sums_curr);
        std::mem::swap(&mut sums_curr, &mut sums_next);
        std::mem::swap(&mut sums_next, &mut preserved);
        self.update_row(&sums_prev, &sums_curr, &sums_next, self.height - 1);
    }
}

impl crate::CellularAutomaton for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert!(width % Self::CELLS_IN_CHUNK == 0);
        let width_effective = width / Self::CELLS_IN_CHUNK;
        assert!(width_effective >= 2 && height >= 2);
        let size = width_effective * height;
        Self {
            data: vec![0; size],
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
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_PER_CELL;
        (self.data[pos] >> offset) & 1 != 0
    }

    fn get_cells(&self) -> Vec<bool> {
        let states: &[u8] = bytemuck::cast_slice(&self.data);
        states
            .iter()
            .flat_map(|x| [x & 0x01 != 0, x & 0x10 != 0])
            .collect()
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_PER_CELL;
        let mask = 1 << offset;
        if state {
            self.data[pos] |= mask;
        } else {
            self.data[pos] &= !mask;
        }
    }

    fn set_cells(&mut self, states: &[bool]) {
        assert_eq!(states.len(), self.width * self.height);
        let data: &mut [u8] = bytemuck::cast_slice_mut(&mut self.data);
        let states = states.iter().map(|x| *x as u8).collect::<Vec<_>>();
        for (x, y) in data.iter_mut().zip(states.chunks_exact(2)) {
            *x = y[0] + (y[1] << Self::BITS_PER_CELL);
        }
    }

    fn update(&mut self, iters_cnt: usize) {
        for _ in 0..iters_cnt {
            unsafe { self.update_inner() }
        }
    }
}
