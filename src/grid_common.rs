use crate::ConwayGrid;

impl<const WIDTH: usize, const HEIGHT: usize> ConwayGrid<WIDTH, HEIGHT> {
    pub fn randomize(&mut self, seed: Option<u64>, fill_rate: Option<f64>) {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaCha8Rng;

        const DEFAULT_SEED: u64 = 42;
        const DEFAULT_FILL_RATE: f64 = 0.3;

        let seed = match seed {
            Some(val) => val,
            None => DEFAULT_SEED,
        };
        let fill_rate = match fill_rate {
            Some(val) => val,
            None => DEFAULT_FILL_RATE,
        };

        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                self.set(x, y, rng.gen_bool(fill_rate));
            }
        }
    }

    pub fn random() -> Self {
        let mut life = Self::empty();
        life.randomize(None, None);
        life
    }

    pub fn draw(&self, screen: &mut [u8]) {
        const BYTES_IN_PIXEL: usize = 4;

        assert_eq!(screen.len(), BYTES_IN_PIXEL * WIDTH * HEIGHT);
        for (i, pixel) in screen.chunks_exact_mut(BYTES_IN_PIXEL).enumerate() {
            let color = if self.get(i % WIDTH, i / WIDTH) {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, 0, 0xff]
            };
            pixel.copy_from_slice(&color);
        }
    }
}
