pub trait Engine {
    /// Create a blank field with dimensions `2^{n_log2} x 2^{n_log2}`
    ///
    /// `6 <= n_log2 < 64`
    fn blank(n_log2: u32) -> Self
    where
        Self: Sized;

    /// Create a field with random cells
    /// 
    /// `fill_rate` - probability of cell being alive
    /// `seed` - random seed (if `None`, then random seed is generated)
    fn random(n_log2: u32, fill_rate: f64, seed: Option<u64>) -> Self
    where
        Self: Sized,
    {
        use rand::{Rng, SeedableRng};
        let mut field = Self::blank(n_log2);
        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        for y in 0..(1 << n_log2) {
            for x in 0..(1 << n_log2) {
                field.set_cell(x, y, rng.gen::<f64>() < fill_rate);
            }
        }
        field
    }

    /// Parse RLE format into the field
    fn parse_rle(data: &[u8]) -> Self
    where
        Self: Sized;

    /// Get the size of the field
    fn side_length(&self) -> u64;

    fn get_cell(&self, x: u64, y: u64) -> bool;

    fn set_cell(&mut self, x: u64, y: u64, state: bool);

    /// Update the field `2^{iters_log2}` times
    fn update(&mut self, iters_log2: u32);

    /// Fills the texture of given resolution with a part of field
    /// (from `viewport_x`, `viewport_y` to `viewport_x + size`, `viewport_y + size`)
    ///
    /// It's allowed to fill the texture with a bigger part of the field by
    /// changing `viewport_x`, `viewport_y`, `size` and `resolution`.
    ///
    /// `dst` - buffer of texture; it should be resized to `resolution * resolution`.
    fn fill_texture(
        &self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    );

    fn print_stats(&self);
}
