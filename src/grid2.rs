pub type Chunk = u64;

#[derive(Clone)]
pub struct ConwayGrid<const WIDTH: usize, const HEIGHT: usize> {
    curr: Vec<Chunk>,
    next: Vec<Chunk>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    const BITS_IN_CELL: usize = 4;
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;
    const WIDTH_REAL: usize = WIDTH / Self::CELLS_IN_CHUNK + 2;
    const HEIGHT_REAL: usize = HEIGHT + 2;

    pub fn empty() -> Self {
        let size = Self::WIDTH_REAL * Self::HEIGHT_REAL;
        assert!(WIDTH % Self::CELLS_IN_CHUNK == 0);
        Self {
            curr: vec![0; size],
            next: vec![0; size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_IN_CELL;
        (self.curr[pos] >> offset) & 1 == 1
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + 1 + (y + 1) * Self::WIDTH_REAL;
        let offset = (x % Self::CELLS_IN_CHUNK) * Self::BITS_IN_CELL;
        let mask = 1 << offset;
        if value {
            self.curr[pos] |= mask;
        } else {
            self.curr[pos] &= !mask;
        }
    }

    /// 0xE -> 0xEEEEE...
    const fn repeat_hex(x: u8) -> Chunk {
        assert!(x < 0x10);
        let mut ans = 0;
        let mask = x as Chunk;
        let mut i = 0;
        while i != Self::CELLS_IN_CHUNK {
            ans |= mask << i * Self::BITS_IN_CELL;
            i += 1;
        }
        ans
    }

    pub fn update(&mut self) {
        unsafe { self.update_inner() }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn update_inner(&mut self) {
        // [0] [1] [2]
        // [3] [4] [5]
        // [6] [7] [8]
        for y in 0..HEIGHT {
            for x in 0..WIDTH / Self::CELLS_IN_CHUNK {
                let i = (x + 1) + (y + 1) * Self::WIDTH_REAL;
                let c1 = self.curr[i - Self::WIDTH_REAL];
                let c4 = self.curr[i];
                let c7 = self.curr[i + Self::WIDTH_REAL];
                let c0 = self.curr[i - Self::WIDTH_REAL - 1];
                let c3 = self.curr[i - 1];
                let c6 = self.curr[i + Self::WIDTH_REAL - 1];
                let c2 = self.curr[i - Self::WIDTH_REAL + 1];
                let c5 = self.curr[i + 1];
                let c8 = self.curr[i + Self::WIDTH_REAL + 1];

                let neighbours = (c1 >> Self::BITS_IN_CELL)
                    + c1
                    + (c1 << Self::BITS_IN_CELL)
                    + (c4 >> Self::BITS_IN_CELL)
                    + (c4 << Self::BITS_IN_CELL)
                    + (c7 >> Self::BITS_IN_CELL)
                    + c7
                    + (c7 << Self::BITS_IN_CELL)
                    + (c0 >> Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1))
                    + (c3 >> Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1))
                    + (c6 >> Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1))
                    + (c2 << Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1))
                    + (c5 << Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1))
                    + (c8 << Self::BITS_IN_CELL * (Self::CELLS_IN_CHUNK - 1));

                let mask = neighbours | (c4 << 3);
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
                self.next[i] = (keep | create) & Self::repeat_hex(0x1);
            }
        }
        std::mem::swap(&mut self.curr, &mut self.next);
    }
}
