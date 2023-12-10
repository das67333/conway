use crate::trait_grid::Grid;

type Chunk = u64;

pub struct ConwayField {
    field: Vec<Chunk>,
    width: usize,
    height: usize,
    width_effective: usize,
}

impl ConwayField {
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;

    /// 0xE -> 0xEEEEE...
    const fn repeat_hex(x: u8) -> Chunk {
        assert!(x < 0x10);
        Chunk::from_ne_bytes([x * 17; std::mem::size_of::<Chunk>()])
    }

    #[inline(always)]
    unsafe fn calc_sums(&self, sums: &mut [Chunk], y: usize) {
        let w = self.width_effective;
        let field = &self.field[y * w..(y + 1) * w];
        {
            *sums.get_unchecked_mut(0) = field.get_unchecked(0)
                + (field.get_unchecked(0) >> 4)
                + (field.get_unchecked(0) << 4)
                + (field.get_unchecked(w - 1) >> (Self::CELLS_IN_CHUNK * 4 - 4))
                + (field.get_unchecked(1) << (Self::CELLS_IN_CHUNK * 4 - 4));
        }
        for x in 1..w - 1 {
            *sums.get_unchecked_mut(x) = field.get_unchecked(x)
                + (field.get_unchecked(x) >> 4)
                + (field.get_unchecked(x) << 4)
                + (field.get_unchecked(x - 1) >> (Self::CELLS_IN_CHUNK * 4 - 4))
                + (field.get_unchecked(x + 1) << (Self::CELLS_IN_CHUNK * 4 - 4));
        }
        {
            *sums.get_unchecked_mut(w - 1) = field.get_unchecked(w - 1)
                + (field.get_unchecked(w - 1) >> 4)
                + (field.get_unchecked(w - 1) << 4)
                + (field.get_unchecked(w - 2) >> (Self::CELLS_IN_CHUNK * 4 - 4))
                + (field.get_unchecked(0) << (Self::CELLS_IN_CHUNK * 4 - 4));
        }
    }

    #[inline(always)]
    unsafe fn update_row(
        &mut self,
        sums_prev: &[Chunk],
        sums_curr: &[Chunk],
        sums_next: &[Chunk],
        y: usize,
    ) {
        let w = self.width_effective;
        let field = &mut self.field[y * w..(y + 1) * w];
        for x in 0..w {
            let neighbours = sums_prev.get_unchecked(x)
                + sums_curr.get_unchecked(x)
                + sums_next.get_unchecked(x)
                - field.get_unchecked(x);

            let mask = neighbours | (field.get_unchecked(x) << 3);
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
            *field.get_unchecked_mut(x) = (keep | create) & Self::repeat_hex(0x1);
        }
    }

    #[target_feature(enable = "avx2")]
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

impl Grid for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert!(width % Self::CELLS_IN_CHUNK == 0);
        let width_effective = width / Self::CELLS_IN_CHUNK;
        assert!(width_effective >= 2 && height >= 2);
        let size = width_effective * height;
        Self {
            field: vec![0; size],
            width,
            height,
            width_effective,
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get(&self, x: usize, y: usize) -> bool {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset = (x % Self::CELLS_IN_CHUNK) * 4;
        (self.field[pos] >> offset) & 1 != 0
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset = (x % Self::CELLS_IN_CHUNK) * 4;
        let mask = 1 << offset;
        if value {
            self.field[pos] |= mask;
        } else {
            self.field[pos] &= !mask;
        }
    }

    fn update(&mut self, n: usize) {
        for _ in 0..n {
            unsafe { self.update_inner() }
        }
    }
}
