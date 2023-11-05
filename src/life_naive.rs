use crate::trait_grid::Grid;

pub struct ConwayField {
    cells_curr: Vec<bool>,
    cells_next: Vec<bool>,
    width: usize,
    height: usize,
}

impl ConwayField {
    fn count_neibs(&self, x: usize, y: usize) -> usize {
        let x1 = if x == 0 { self.width - 1 } else { x - 1 };
        let x2 = if x == self.width - 1 { 0 } else { x + 1 };
        let y1 = if y == 0 { self.height - 1 } else { y - 1 };
        let y2 = if y == self.height - 1 { 0 } else { y + 1 };
        self.get(x1, y1) as usize
            + self.get(x, y1) as usize
            + self.get(x2, y1) as usize
            + self.get(x1, y) as usize
            + self.get(x2, y) as usize
            + self.get(x1, y2) as usize
            + self.get(x, y2) as usize
            + self.get(x2, y2) as usize
    }
}

impl Grid for ConwayField {
    fn blank(width: usize, height: usize) -> Self {
        assert!(width >= 1 && height >= 1);
        let size = width * height;
        Self {
            cells_curr: vec![false; size],
            cells_next: vec![false; size],
            width,
            height,
        }
    }

    fn random(width: usize, height: usize, seed: Option<u64>, fill_rate: f64) -> Self {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        let mut rng = if let Some(x) = seed {
            ChaCha8Rng::seed_from_u64(x)
        } else {
            ChaCha8Rng::from_entropy()
        };
        let mut result = Self::blank(width, height);
        for y in 0..height {
            for x in 0..width {
                result.set(x, y, rng.gen_bool(fill_rate));
            }
        }
        result
    }

    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get(&self, x: usize, y: usize) -> bool {
        self.cells_curr[x + y * self.width]
    }

    fn set(&mut self, x: usize, y: usize, value: bool) {
        self.cells_curr[x + y * self.width] = value;
    }

    fn update(&mut self, n: usize) {
        for _ in 0..n {
            for y in 0..self.height {
                for x in 0..self.width {
                    let neibs = self.count_neibs(x, y);
                    let next = if self.get(x, y) {
                        neibs == 2 || neibs == 3
                    } else {
                        neibs == 3
                    };
                    self.set(x, y, next);
                }
            }
            std::mem::swap(&mut self.cells_next, &mut self.cells_curr);
        }
    }

    fn draw(&self, screen: &mut [u8]) {
        const BYTES_IN_PIXEL: usize = 4;

        assert_eq!(screen.len(), BYTES_IN_PIXEL * self.width * self.height);
        for (pixel, &value) in screen
            .chunks_exact_mut(BYTES_IN_PIXEL)
            .zip(self.cells_curr.iter())
        {
            pixel.copy_from_slice(&if value {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, 0, 0xff]
            });
        }
    }
}
