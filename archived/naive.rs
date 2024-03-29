use super::ca_trait::CellularAutomaton;

pub struct ConwayFieldNaive {
    data_curr: Vec<bool>,
    data_next: Vec<bool>,
    width: usize,
    height: usize,
}

impl ConwayFieldNaive {
    fn count_neibs(&self, x: usize, y: usize) -> u8 {
        let x1 = if x == 0 { self.width - 1 } else { x - 1 };
        let x2 = if x == self.width - 1 { 0 } else { x + 1 };
        let y1 = if y == 0 { self.height - 1 } else { y - 1 };
        let y2 = if y == self.height - 1 { 0 } else { y + 1 };
        self.get_cell(x1, y1) as u8
            + self.get_cell(x, y1) as u8
            + self.get_cell(x2, y1) as u8
            + self.get_cell(x1, y) as u8
            + self.get_cell(x2, y) as u8
            + self.get_cell(x1, y2) as u8
            + self.get_cell(x, y2) as u8
            + self.get_cell(x2, y2) as u8
    }
}

impl CellularAutomaton for ConwayFieldNaive {
    fn id<'a>() -> &'a str {
        "naive"
    }

    fn blank(width: usize, height: usize) -> Self {
        assert!(width >= 1 && height >= 1);
        Self {
            data_curr: vec![false; width * height],
            data_next: vec![false; width * height],
            width,
            height,
        }
    }

    fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_cell(&self, x: usize, y: usize) -> bool {
        self.data_curr[x + y * self.width]
    }

    fn set_cell(&mut self, x: usize, y: usize, state: bool) {
        self.data_curr[x + y * self.width] = state;
    }

    fn update(&mut self, iters_cnt: usize) {
        for _ in 0..iters_cnt {
            for y in 0..self.height {
                for x in 0..self.width {
                    let neibs = self.count_neibs(x, y);
                    let next = if self.data_curr[x + y * self.width] {
                        neibs == 2 || neibs == 3
                    } else {
                        neibs == 3
                    };
                    self.data_next[x + y * self.width] = next;
                }
            }
            std::mem::swap(&mut self.data_next, &mut self.data_curr);
        }
    }
}
