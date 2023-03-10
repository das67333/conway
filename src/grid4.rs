pub type Chunk = std::simd::u64x4;
type Lane = u64;

#[derive(Clone)]
pub struct ConwayGrid<const WIDTH: usize, const HEIGHT: usize> {
    data: Vec<Chunk>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    const BITS_PER_CELL: usize = 4;
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;
    const WIDTH_REAL: usize = WIDTH / Self::CELLS_IN_CHUNK + 2;
    const HEIGHT_REAL: usize = HEIGHT + 2;

    const CELLS_IN_LANE: usize = Self::CELLS_IN_CHUNK / Chunk::LANES;

    pub fn empty() -> Self {
        let size = Self::WIDTH_REAL * Self::HEIGHT_REAL;
        assert!(WIDTH % Self::CELLS_IN_CHUNK == 0);
        Self {
            data: vec![Chunk::default(); size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset1 = x % Self::CELLS_IN_CHUNK;
        let lane = self.data[pos][offset1 / Self::CELLS_IN_LANE];
        let offset2 = offset1 % Self::CELLS_IN_LANE * Self::BITS_PER_CELL;
        lane >> offset2 & 1 == 1
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset1 = x % Self::CELLS_IN_CHUNK;
        let lane = &mut self.data[pos][offset1 / Self::CELLS_IN_LANE];
        let offset2 = offset1 % Self::CELLS_IN_LANE * Self::BITS_PER_CELL;
        let mask = 1 << offset2;
        if value {
            *lane |= mask;
        } else {
            *lane &= !mask;
        }
    }

    /// 0xE -> 0xEEEEE...
    const fn repeat_hex(x: u8) -> Lane {
        assert!(x < 0x10);
        let mut ans = 0;
        let mask = x as Lane;
        let mut i = 0;
        while i != Self::CELLS_IN_LANE {
            ans |= mask << i * Self::BITS_PER_CELL;
            i += 1;
        }
        ans
    }

    pub fn update(&mut self) {
        unsafe { self.update_inner() }
    }

    fn calc_seq_sums(&self, row: &mut Vec<Chunk>, y_real: usize) {
        let shift1 = Chunk::splat(Self::BITS_PER_CELL as Lane);
        let shift2 = Chunk::splat((Self::BITS_PER_CELL * (Self::CELLS_IN_LANE - 1)) as Lane);

        let l = y_real * Self::WIDTH_REAL;
        for x in 1..=WIDTH / Self::CELLS_IN_CHUNK {
            let curr = self.data[x + l];
            let (mut prev, mut next);
            prev = curr.rotate_lanes_right::<1>();
            next = curr.rotate_lanes_left::<1>();
            prev[0] = self.data[x + l - 1][Chunk::LANES - 1];
            next[Chunk::LANES - 1] = self.data[x + l + 1][0];
            row[x] =
                curr + (curr >> shift1) + (curr << shift1) + (prev >> shift2) + (next << shift2);
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn update_inner(&mut self) {
        let (mut seq_sums_prev, mut seq_sums_curr, mut seq_sums_next) = (
            vec![Chunk::default(); Self::WIDTH_REAL],
            vec![Chunk::default(); Self::WIDTH_REAL],
            vec![Chunk::default(); Self::WIDTH_REAL],
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

                let mask = neighbours | (self.data[x + l] << Chunk::splat(0x3));
                let keep = {
                    let mut temp = (mask & Chunk::splat(Self::repeat_hex(0xE)))
                        ^ Chunk::splat(Self::repeat_hex(0x5));
                    temp &= temp >> Chunk::splat(0x2);
                    temp &= temp >> Chunk::splat(0x1);
                    temp
                };
                let create = {
                    let mut temp = mask ^ Chunk::splat(Self::repeat_hex(0xC));
                    temp &= temp >> Chunk::splat(0x2);
                    temp &= temp >> Chunk::splat(0x1);
                    temp
                };
                self.data[x + l] = (keep | create) & Chunk::splat(Self::repeat_hex(0x1));
            }
        }
    }
}
