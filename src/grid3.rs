pub type Chunk = u64;

#[derive(Clone)]
pub struct ConwayGrid<const WIDTH: usize, const HEIGHT: usize> {
    data: Vec<Chunk>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    const BITS_PER_CELL: usize = 4;
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;
    const WIDTH_REAL: usize = WIDTH / Self::CELLS_IN_CHUNK + 2;
    const HEIGHT_REAL: usize = HEIGHT + 2;

    pub fn empty() -> Self {
        let size = Self::WIDTH_REAL * Self::HEIGHT_REAL;
        assert!(WIDTH % Self::CELLS_IN_CHUNK == 0);
        Self {
            data: vec![0; size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_PER_CELL;
        self.data[pos] >> offset & 1 == 1
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_PER_CELL;
        let mask = 1 << offset;
        if value {
            self.data[pos] |= mask;
        } else {
            self.data[pos] &= !mask;
        }
    }

    /// 0xE -> 0xEEEEE...
    const fn repeat_hex(x: u8) -> Chunk {
        assert!(x < 0x10);
        let mut ans = 0;
        let mask = x as Chunk;
        let mut i = 0;
        while i != Self::CELLS_IN_CHUNK {
            ans |= mask << i * Self::BITS_PER_CELL;
            i += 1;
        }
        ans
    }

    pub fn update(&mut self) {
        unsafe { self.update_inner() }
    }

    fn calc_seq_sums(&self, row: &mut Vec<Chunk>, y_real: usize) {
        let l = y_real * Self::WIDTH_REAL;
        for x in 1..=WIDTH / Self::CELLS_IN_CHUNK {
            row[x] = self.data[x + l]
                + (self.data[x + l] >> Self::BITS_PER_CELL)
                + (self.data[x + l] << Self::BITS_PER_CELL)
                + (self.data[x + l - 1] >> Self::BITS_PER_CELL * (Self::CELLS_IN_CHUNK - 1))
                + (self.data[x + l + 1] << Self::BITS_PER_CELL * (Self::CELLS_IN_CHUNK - 1));
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn update_inner(&mut self) {
        let (mut seq_sums_prev, mut seq_sums_curr, mut seq_sums_next) = (
            vec![0; Self::WIDTH_REAL],
            vec![0; Self::WIDTH_REAL],
            vec![0; Self::WIDTH_REAL],
        );
        self.calc_seq_sums(&mut seq_sums_next, 1);

        for y in 0..HEIGHT {
            std::mem::swap(&mut seq_sums_prev, &mut seq_sums_curr);
            std::mem::swap(&mut seq_sums_curr, &mut seq_sums_next);
            self.calc_seq_sums(&mut seq_sums_next, y + 2);
            let l = (y + 1) * Self::WIDTH_REAL;
            for x in 1..=WIDTH / Self::CELLS_IN_CHUNK {
                let neighbours =
                    seq_sums_prev[x] + seq_sums_curr[x] + seq_sums_next[x] - self.data[x + l];

                let mask = neighbours | (self.data[x + l] << 3);
                let keep = {
                    let mut temp = (mask & Self::repeat_hex(0xE)) ^ Self::repeat_hex(0x5);
                    temp &= temp >> 2;
                    temp &= temp >> 1;
                    temp
                };
                let create = {
                    let mut temp = mask ^ Self::repeat_hex(0xC);
                    temp &= temp >> 2;
                    temp &= temp >> 1;
                    temp
                };
                self.data[x + l] = (keep | create) & Self::repeat_hex(0x1);
            }
        }
    }
}
