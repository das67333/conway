#[derive(Clone)]
pub struct ConwayGrid<const WIDTH: usize, const HEIGHT: usize> {
    cells: Vec<bool>,
    neibs_count: Vec<u8>,
}

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    const WIDTH_REAL: usize = WIDTH + 2;
    const HEIGHT_REAL: usize = HEIGHT + 2;

    pub fn empty() -> Self {
        let size = Self::WIDTH_REAL * Self::HEIGHT_REAL;
        Self {
            cells: vec![false; size],
            neibs_count: vec![0; size],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[x + 1 + (y + 1) * Self::WIDTH_REAL]
    }

    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.cells[x + 1 + (y + 1) * Self::WIDTH_REAL] = value;
    }

    pub fn update(&mut self) {
        self.update_neighbours();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = x + y * WIDTH;
                let neibs = self.neibs_count[idx];
                self.cells[idx] = if self.cells[idx] {
                    neibs == 2 || neibs == 3
                } else {
                    neibs == 3
                };
            }
        }
    }

    fn update_neighbours(&mut self) {
        self.neibs_count.fill(0);
        let w: isize = WIDTH as isize;
        let h: isize = HEIGHT as isize;
        for shift in [-w - 1, -w, -w + 1, -1, 1, w - 1, w, w + 1] {
            for i in w + 1..h * w - w - 1 {
                self.neibs_count[(i + shift) as usize] += self.cells[i as usize] as u8;
            }
        }
    }
}
