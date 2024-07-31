pub const MIN_SIDE_LOG2: u32 = 7;
pub const MAX_SIDE_LOG2: u32 = 62;

use super::Topology;

/// Engine trait for Game of Life with edges stitched together.
pub trait Engine {
    /// Create a blank field with dimensions `2^{n_log2} x 2^{n_log2}`
    ///
    /// `MIN_SIDE_LOG2 <= n_log2 <= MAX_SIDE_LOG2`
    fn blank(n_log2: u32) -> Self
    where
        Self: Sized;

    /// Create a field with random cells
    ///
    /// `seed` - random seed (if `None`, then random seed is generated)
    fn random(n_log2: u32, seed: Option<u64>) -> Self
    where
        Self: Sized,
    {
        use rand::{Rng, SeedableRng};
        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let cells = (0..(1 << (n_log2 * 2 - 6)))
            .map(|_| rng.gen::<u64>())
            .collect();
        Self::from_cells(n_log2, cells)
    }

    /// Parse RLE format into the field
    fn from_rle(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let (n_log2, cells) = crate::parse_rle(data);
        Self::from_cells(n_log2, cells)
    }

    fn from_macrocell(_data: &[u8]) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Create a square field from a vector of cells
    fn from_cells(n_log2: u32, cells: Vec<u64>) -> Self
    where
        Self: Sized;

    /// Save the field in MacroCell format
    fn save_into_macrocell(&self) -> Vec<u8> {
        unimplemented!()
    }

    fn get_cells(&self) -> Vec<u64>;

    /// Get the side length of the field in log2
    fn side_length_log2(&self) -> u32;

    fn get_cell(&self, x: u64, y: u64) -> bool;

    fn set_cell(&mut self, x: u64, y: u64, state: bool);

    /// Update the field `2^{iters_log2}` times
    ///
    /// If `unbounded` is `false`, then the field's topology is a torus.
    ///
    /// Returns the coordinate offset after the update.
    fn update(&mut self, steps_log2: u32, topology: Topology) -> [u64; 2];

    /// Fills the texture of given resolution with a part of field
    /// (from `viewport_x`, `viewport_y` to `viewport_x + size`, `viewport_y + size`)
    ///
    /// It's allowed to fill the texture with a bigger part of the field by
    /// changing `viewport_x`, `viewport_y`, `size` and `resolution`.
    ///
    /// `dst` - buffer of texture; it should be resized to `resolution * resolution`.
    ///
    /// Returns log2 of the number of cells per pixel side.
    fn fill_texture(
        &self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    ) -> u32;

    /// Returns multiline string reporting engine stats.
    ///
    /// This function is fast enough to be called every frame.
    fn stats_fast(&self) -> String {
        String::new()
    }

    /// Additional stats that are slow to compute.
    fn stats_slow(&self) -> String {
        String::new()
    }
}
