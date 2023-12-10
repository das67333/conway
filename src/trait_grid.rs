pub trait Grid: Sized {
    fn blank(width: usize, height: usize) -> Self;

    fn size(&self) -> (usize, usize);

    fn get(&self, x: usize, y: usize) -> bool;

    fn set(&mut self, x: usize, y: usize, state: bool);

    fn update(&mut self, n: usize);

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

    fn draw(&self, screen: &mut [u8]) {
        const BYTES_IN_PIXEL: usize = 4;

        let (w, h) = self.size();
        assert_eq!(screen.len(), BYTES_IN_PIXEL * w * h);
        for (i, pixel) in screen.chunks_exact_mut(BYTES_IN_PIXEL).enumerate() {
            let value = self.get(i % w, i / w);
            let color = if value {
                [0, 0xff, 0xff, 0xff]
            } else {
                [0, 0, 0, 0xff]
            };
            pixel.copy_from_slice(&color);
        }
    }

    fn println(&self) {
        let (w, h) = self.size();
        for y in 0..h {
            for x in 0..w {
                print!("{}", self.get(x, y) as u8);
                if x + 1 == w {
                    println!();
                }
            }
        }
        println!();
    }
}
