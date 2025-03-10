use crate::{parse_rle, Topology};

/// Game engine for Game of Life
pub trait Engine {
    /// Create a blank field with dimensions `2^{size_log2} x 2^{size_log2}`.
    ///
    /// `MIN_SIDE_LOG2 <= size_log2 <= MAX_SIDE_LOG2`
    fn blank(size_log2: u32) -> Self
    where
        Self: Sized;

    /// Create a field with random cells.
    ///
    /// `seed` - random seed (if `None`, then random seed is generated)
    fn random(size_log2: u32, seed: Option<u64>) -> Self
    where
        Self: Sized,
    {
        use rand::{Rng, SeedableRng};
        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let cells = (0..(1 << (size_log2 * 2 - 6)))
            .map(|_| rng.gen::<u64>())
            .collect();
        Self::from_cells_array(size_log2, cells)
    }

    /// Recursively builds OTCA megapixels `depth` times, using `top_pattern` as the top level.
    ///
    /// If `depth` == 0, every cell is a regular cell, if 1 it is
    /// an OTCA build from regular cells and so on.
    ///
    /// `top_pattern` must consist of zeros and ones.
    fn from_recursive_otca_metapixel(depth: u32, top_pattern: Vec<Vec<u8>>) -> Self
    where
        Self: Sized;

    /// Parse RLE format into the field.
    fn from_rle(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let (size_log2, cells) = parse_rle(data);
        Self::from_cells_array(size_log2, cells)
    }

    /// Parse MacroCell format into the field.
    fn from_macrocell(_data: &[u8]) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Create a square field from a vector of cells.
    fn from_cells_array(size_log2: u32, cells: Vec<u64>) -> Self
    where
        Self: Sized;

    /// Save the field in MacroCell format.
    fn save_as_macrocell(&mut self) -> Vec<u8> {
        unimplemented!()
    }

    /// Get cells of the field as a single vector of packed cells.
    fn get_cells(&self) -> Vec<u64>;

    /// Get log2 side length of the field.
    fn side_length_log2(&self) -> u32;

    /// Get cell state at (x, y).
    fn get_cell(&self, x: u64, y: u64) -> bool;

    /// Set cell state at (x, y).
    fn set_cell(&mut self, x: u64, y: u64, state: bool);

    /// Update the field `2^{iters_log2}` times.
    /// This function may change the size of the field.
    ///
    /// Returns coordinate shift caused by the update.
    fn update(&mut self, steps_log2: u32, topology: Topology) -> [i64; 2];

    /// Fills the texture of given resolution with a part of field
    /// (from `viewport_x`, `viewport_y` to `viewport_x + size`, `viewport_y + size`).
    ///
    /// It's allowed to fill the texture with a bigger part of the field by
    /// changing `viewport_x`, `viewport_y`, `size` and `resolution`.
    ///
    /// `dst` - buffer of texture; it should be resized to `resolution * resolution`.
    fn fill_texture(
        &mut self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    );

    /// Total number of alive cells in the field.
    fn population(&mut self) -> f64;

    /// Hash of the field's content.
    fn hash(&self) -> u64 {
        unimplemented!()
    }

    /// Heap memory used by the engine.
    fn bytes_total(&self) -> usize;

    /// Returns multiline string reporting engine stats.
    ///
    /// This function is fast enough to be called every frame.
    fn statistics(&mut self) -> String;

    /// Some engines accumulate cache that can be freed.
    fn run_gc(&mut self) {}
}

/// Async Game engine for Game of Life
pub trait AsyncEngine {
    /// Create a blank field with dimensions `2^{size_log2} x 2^{size_log2}`.
    ///
    /// `MIN_SIDE_LOG2 <= size_log2 <= MAX_SIDE_LOG2`
    fn blank(size_log2: u32) -> Self
    where
        Self: Sized;

    /// Create a field with random cells.
    ///
    /// `seed` - random seed (if `None`, then random seed is generated)
    fn random(size_log2: u32, seed: Option<u64>) -> Self
    where
        Self: Sized,
    {
        use rand::{Rng, SeedableRng};
        let mut rng = if let Some(x) = seed {
            rand_chacha::ChaCha8Rng::seed_from_u64(x)
        } else {
            rand_chacha::ChaCha8Rng::from_entropy()
        };
        let cells = (0..(1 << (size_log2 * 2 - 6)))
            .map(|_| rng.gen::<u64>())
            .collect();
        Self::from_cells_array(size_log2, cells)
    }

    /// Recursively builds OTCA megapixels `depth` times, using `top_pattern` as the top level.
    ///
    /// If `depth` == 0, every cell is a regular cell, if 1 it is
    /// an OTCA build from regular cells and so on.
    ///
    /// `top_pattern` must consist of zeros and ones.
    fn from_recursive_otca_metapixel(depth: u32, top_pattern: Vec<Vec<u8>>) -> Self
    where
        Self: Sized;

    /// Parse RLE format into the field.
    fn from_rle(data: &[u8]) -> Self
    where
        Self: Sized,
    {
        let (size_log2, cells) = parse_rle(data);
        Self::from_cells_array(size_log2, cells)
    }

    /// Parse MacroCell format into the field.
    fn from_macrocell(_data: &[u8]) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }

    /// Create a square field from a vector of cells.
    fn from_cells_array(size_log2: u32, cells: Vec<u64>) -> Self
    where
        Self: Sized;

    /// Save the field in MacroCell format.
    fn save_as_macrocell(&mut self) -> Vec<u8> {
        unimplemented!()
    }

    /// Get cells of the field as a single vector of packed cells.
    fn get_cells(&self) -> Vec<u64>;

    /// Get log2 side length of the field.
    fn side_length_log2(&self) -> u32;

    /// Get cell state at (x, y).
    fn get_cell(&self, x: u64, y: u64) -> bool;

    /// Set cell state at (x, y).
    fn set_cell(&mut self, x: u64, y: u64, state: bool);

    /// Update the field `2^{iters_log2}` times.
    /// This function may change the size of the field.
    ///
    /// Returns coordinate shift caused by the update.
    fn update(&mut self, steps_log2: u32, topology: Topology) -> [i64; 2];

    /// Fills the texture of given resolution with a part of field
    /// (from `viewport_x`, `viewport_y` to `viewport_x + size`, `viewport_y + size`).
    ///
    /// It's allowed to fill the texture with a bigger part of the field by
    /// changing `viewport_x`, `viewport_y`, `size` and `resolution`.
    ///
    /// `dst` - buffer of texture; it should be resized to `resolution * resolution`.
    fn fill_texture(
        &mut self,
        viewport_x: &mut f64,
        viewport_y: &mut f64,
        size: &mut f64,
        resolution: &mut f64,
        dst: &mut Vec<f64>,
    );

    /// Total number of alive cells in the field.
    fn population(&mut self) -> f64;

    /// Hash of the field's content.
    fn hash(&self) -> u64 {
        unimplemented!()
    }

    /// Heap memory used by the engine.
    fn bytes_total(&self) -> usize;

    /// Returns multiline string reporting engine stats.
    ///
    /// This function is fast enough to be called every frame.
    fn statistics(&mut self) -> String;

    /// Some engines accumulate cache that can be freed.
    fn run_gc(&mut self) {}
}
