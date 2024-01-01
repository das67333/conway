pub trait CellularAutomaton: Sized {
    /// Name that is used in benchmarks
    fn id<'a>() -> &'a str;
    /// Creates a field filled with dead cells
    fn blank(width: usize, height: usize) -> Self;
    /// [`(width, height)`] of the field
    fn get_size(&self) -> (usize, usize);
    fn get_cell(&self, x: usize, y: usize) -> bool;
    fn get_cells(&self) -> Vec<bool>;
    fn set_cell(&mut self, x: usize, y: usize, state: bool);
    fn set_cells(&mut self, states: &[bool]);
    fn update(&mut self, iters_cnt: usize);

    /// Fills the field with random cells
    fn randomize(&mut self, seed: Option<u64>, fill_rate: f64) {
        use rand::{Rng, SeedableRng};

        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let (w, h) = self.get_size();
        let states = (0..w * h)
            .map(|_| rng.gen_bool(fill_rate))
            .collect::<Vec<_>>();
        self.set_cells(&states);
    }

    /// Draws the field (TODO: deprecated and will be removed)
    fn draw(&self, screen: &mut [u8]) {
        const BYTES_IN_PIXEL: usize = 4;

        let (w, h) = self.get_size();
        assert_eq!(screen.len(), BYTES_IN_PIXEL * w * h);
        for (i, pixel) in screen.chunks_exact_mut(BYTES_IN_PIXEL).enumerate() {
            let value = self.get_cell(i % w, i / w);
            let color = if value {
                [0x00, 0xff, 0xff, 0xff]
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };
            pixel.copy_from_slice(&color);
        }
    }

    /// Prints the field to the stdout
    fn println(&self) {
        let (w, h) = self.get_size();
        for y in 0..h {
            for x in 0..w {
                print!("{}", self.get_cell(x, y) as u8);
                if x + 1 == w {
                    println!();
                }
            }
        }
        println!();
    }
}
