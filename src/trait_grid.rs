pub trait Grid {
    fn blank(width: usize, height: usize) -> Self;

    fn random(width: usize, height: usize, seed: Option<u64>, fill_rate: f64) -> Self;

    fn size(&self) -> (usize, usize);

    fn get(&self, x: usize, y: usize) -> bool;

    fn set(&mut self, x: usize, y: usize, state: bool);

    fn update(&mut self, n: usize);

    fn draw(&self, screen: &mut [u8]);

    fn println(&self) {
        for y in 0..self.size().1 {
            for x in 0..self.size().0 {
                print!("{}", self.get(x, y) as u8);
            }
            println!();
        }
        println!();
    }
}
