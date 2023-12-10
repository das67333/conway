use crate::trait_grid::Grid;
use std::simd::*;

type Elem = u64;
type Chunk = Simd<Elem, 4>;

pub struct ConwayField {
    field: Vec<Chunk>,
    width: usize,
    height: usize,
    width_effective: usize,
}

impl ConwayField {
    const CELLS_IN_CHUNK: usize = std::mem::size_of::<Chunk>() * 2;
    const CELLS_IN_ELEM: usize = std::mem::size_of::<Elem>() * 2;

    /// 0xE -> 0xEEEEE...
    fn repeat_hex(x: u8) -> Chunk {
        assert!(x < 0x10);
        Chunk::splat(Elem::from_ne_bytes([x * 17; std::mem::size_of::<Elem>()]))
    }

    #[inline(always)]
    unsafe fn calc_sums(&self, sums: &mut [Chunk], y: usize) {
        let w = self.width_effective;
        let field = &self.field[y * w..(y + 1) * w];

        let shift1 = Chunk::splat(4);
        let shift2 = Chunk::splat(((Self::CELLS_IN_ELEM - 1) * 4) as Elem);
        {
            let curr = field[0];
            let (mut prev, mut next);
            prev = curr.rotate_elements_right::<1>();
            next = curr.rotate_elements_left::<1>();
            prev[0] = field[w - 1][Chunk::LEN - 1];
            next[Chunk::LEN - 1] = field[1][0];
            sums[0] =
                curr + (curr >> shift1) + (curr << shift1) + (prev >> shift2) + (next << shift2);
        }
        for x in 1..w - 1 {
            let curr = field[x];
            let (mut prev, mut next);
            prev = curr.rotate_elements_right::<1>();
            next = curr.rotate_elements_left::<1>();
            prev[0] = field[x - 1][Chunk::LEN - 1];
            next[Chunk::LEN - 1] = field[x + 1][0];
            sums[x] =
                curr + (curr >> shift1) + (curr << shift1) + (prev >> shift2) + (next << shift2);
        }
        {
            let curr = field[w - 1];
            let (mut prev, mut next);
            prev = curr.rotate_elements_right::<1>();
            next = curr.rotate_elements_left::<1>();
            prev[0] = field[w - 2][Chunk::LEN - 1];
            next[Chunk::LEN - 1] = field[0][0];
            sums[w - 1] =
                curr + (curr >> shift1) + (curr << shift1) + (prev >> shift2) + (next << shift2);
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

            let mask = neighbours | (field[x] << Chunk::splat(0x3));
            let keep = {
                let mut temp = (mask & Self::repeat_hex(0xE)) ^ Self::repeat_hex(0x5);
                temp &= temp >> Chunk::splat(0x2);
                temp &= temp >> Chunk::splat(0x1);
                temp
            };
            let create = {
                let mut temp = mask ^ Self::repeat_hex(0xC);
                temp &= temp >> Chunk::splat(0x2);
                temp &= temp >> Chunk::splat(0x1);
                temp
            };
            *field.get_unchecked_mut(x) = (keep | create) & Self::repeat_hex(0x1);
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_inner(&mut self) {
        let (mut sums_prev, mut sums_curr, mut sums_next) = (
            vec![Chunk::default(); self.width_effective],
            vec![Chunk::default(); self.width_effective],
            vec![Chunk::default(); self.width_effective],
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
            field: vec![Chunk::default(); size],
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
        let offset1 = x % Self::CELLS_IN_CHUNK;
        let elem = self.field[pos][offset1 / Self::CELLS_IN_ELEM];
        let offset2 = (offset1 % Self::CELLS_IN_ELEM) * 4;
        elem >> offset2 & 1 == 1
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        let pos = x / Self::CELLS_IN_CHUNK + y * self.width_effective;
        let offset1 = x % Self::CELLS_IN_CHUNK;
        let elem = &mut self.field[pos][offset1 / Self::CELLS_IN_ELEM];
        let offset2 = (offset1 % Self::CELLS_IN_ELEM) * 4;
        let mask = 1 << offset2;
        if value {
            *elem |= mask;
        } else {
            *elem &= !mask;
        }
    }

    fn update(&mut self, n: usize) {
        for _ in 0..n {
            unsafe { self.update_inner() }
        }
    }
}
