#[derive(Clone)]
pub struct ConwayGrid<const WIDTH: usize, const HEIGHT: usize> {
    cells_curr: Vec<bool>,
    cells_next: Vec<bool>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    const WIDTH_REAL: usize = WIDTH + 2;
    const HEIGHT_REAL: usize = HEIGHT + 2;

    pub fn empty() -> Self {
        let size = Self::WIDTH_REAL * Self::HEIGHT_REAL;
        Self {
            cells_curr: vec![false; size],
            cells_next: vec![false; size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells_curr[x + 1 + (y + 1) * Self::WIDTH_REAL]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.cells_curr[x + 1 + (y + 1) * Self::WIDTH_REAL] = value;
    }

    fn count_neibs(&self, x: usize, y: usize) -> usize {
        self.get(x - 1, y - 1) as usize
            + self.get(x, y - 1) as usize
            + self.get(x + 1, y - 1) as usize
            + self.get(x - 1, y) as usize
            + self.get(x + 1, y) as usize
            + self.get(x - 1, y + 1) as usize
            + self.get(x, y + 1) as usize
            + self.get(x + 1, y + 1) as usize
    }

    pub fn update(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let neibs = self.count_neibs(x, y);
                let idx = x + 1 + (y + 1) * Self::WIDTH_REAL;
                let next = if self.cells_curr[idx] {
                    neibs == 2 || neibs == 3
                } else {
                    neibs == 3
                };
                self.cells_next[idx] = next;
            }
        }
        std::mem::swap(&mut self.cells_next, &mut self.cells_curr);
    }
}
