use crate::{GoLEngine, Pattern, PatternFormat, Topology};
use anyhow::{anyhow, Result};
use num_bigint::BigInt;

/// A fast, SIMD-optimized engine for Conway's Game of Life that uses bitwise operations
/// for efficient cell state updates.
///
/// # Limitations
///
/// - Only supports torus topology (wrapping boundaries)
/// - Requires patterns with `size_log2` of at least 7 (128×128 cells)
///
/// # Example
///
/// ```rust
/// use gol_engines::{GoLEngine, Pattern, PatternFormat, SIMDEngine, Topology};
///
/// // Load a pattern (must be at least 128×128)
/// let pattern = Pattern::random(7, None).unwrap();
/// let mut expanded_pattern = pattern.clone();
/// expanded_pattern.expand(7); // Expand to at least 128×128
///
/// // Create the SIMD engine
/// let mut engine = SIMDEngine::from_pattern(&expanded_pattern, Topology::Torus).unwrap();
///
/// // Run for 2^10 = 1024 generations
/// engine.update(10, Topology::Torus);
///
/// // Get the current state
/// let result = engine.current_state();
/// ```
pub struct SIMDEngine {
    /// The packed cell data, where each bit represents a cell and 64 cells are stored in each u64
    data: Vec<u64>,
    /// The side length of the square grid (must be a power of 2)
    n: usize,
}

impl SIMDEngine {
    const CELLS_IN_CHUNK: usize = 64;

    fn update_row(row_prev: &[u64], row_curr: &[u64], row_next: &[u64], dst: &mut [u64]) {
        // TODO: double word technique
        // TODO: use avx2 if available
        let (w, shift) = (row_prev.len(), Self::CELLS_IN_CHUNK - 1);
        let (x, x1, x2) = (0, w - 1, 1);

        let b = row_prev[x];
        let a = (b << 1) | (row_prev[x1] >> shift);
        let c = (b >> 1) | (row_prev[x2] << shift);
        let i = row_curr[x];
        let h = (i << 1) | (row_curr[x1] >> shift);
        let d = (i >> 1) | (row_curr[x2] << shift);
        let f = row_next[x];
        let g = (f << 1) | (row_next[x1] >> shift);
        let e = (f >> 1) | (row_next[x2] << shift);
        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);
        dst[x] = (i & i2) | i3;

        for x in 1..w - 1 {
            let (x, x1, x2) = (x, x - 1, x + 1);

            let b = row_prev[x];
            let a = (b << 1) | (row_prev[x1] >> shift);
            let c = (b >> 1) | (row_prev[x2] << shift);
            let i = row_curr[x];
            let h = (i << 1) | (row_curr[x1] >> shift);
            let d = (i >> 1) | (row_curr[x2] << shift);
            let f = row_next[x];
            let g = (f << 1) | (row_next[x1] >> shift);
            let e = (f >> 1) | (row_next[x2] << shift);
            let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
            let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
            let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
            let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
            let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
            let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
            let z = !ah23 & ah1;
            let (i2, i3) = (!ah0 & z, ah0 & z);
            dst[x] = (i & i2) | i3;
        }
        let (x, x1, x2) = (w - 1, w - 2, 0);

        let b = row_prev[x];
        let a = (b << 1) | (row_prev[x1] >> shift);
        let c = (b >> 1) | (row_prev[x2] << shift);
        let i = row_curr[x];
        let h = (i << 1) | (row_curr[x1] >> shift);
        let d = (i >> 1) | (row_curr[x2] << shift);
        let f = row_next[x];
        let g = (f << 1) | (row_next[x1] >> shift);
        let e = (f >> 1) | (row_next[x2] << shift);
        let (ab0, ab1, cd0, cd1) = (a ^ b, a & b, c ^ d, c & d);
        let (ef0, ef1, gh0, gh1) = (e ^ f, e & f, g ^ h, g & h);
        let (ad0, ad1, ad2) = (ab0 ^ cd0, ab1 ^ cd1 ^ (ab0 & cd0), ab1 & cd1);
        let (eh0, eh1, eh2) = (ef0 ^ gh0, ef1 ^ gh1 ^ (ef0 & gh0), ef1 & gh1);
        let (ah0, xx, yy) = (ad0 ^ eh0, ad0 & eh0, ad1 ^ eh1);
        let (ah1, ah23) = (xx ^ yy, ad2 | eh2 | (ad1 & eh1) | (xx & yy));
        let z = !ah23 & ah1;
        let (i2, i3) = (!ah0 & z, ah0 & z);
        dst[x] = (i & i2) | i3;
    }

    fn update_inner(&mut self) {
        let (w, h) = (self.n >> Self::CELLS_IN_CHUNK.ilog2(), self.n);
        let mut row_prev = self.data[(h - 1) * w..].to_vec();
        let mut row_curr = self.data[..w].to_vec();
        let row_preserved = row_curr.to_vec();
        let mut row_next = self.data[w..2 * w].to_vec();
        let dst = &mut self.data[..w];
        Self::update_row(&row_prev, &row_curr, &row_next, dst);

        for y in 1..self.n - 1 {
            std::mem::swap(&mut row_prev, &mut row_curr);
            std::mem::swap(&mut row_curr, &mut row_next);
            row_next.copy_from_slice(&self.data[(y + 1) * w..(y + 2) * w]);
            let dst = &mut self.data[y * w..(y + 1) * w];
            Self::update_row(&row_prev, &row_curr, &row_next, dst);
        }

        std::mem::swap(&mut row_prev, &mut row_curr);
        std::mem::swap(&mut row_curr, &mut row_next);
        let dst = &mut self.data[(h - 1) * w..];
        Self::update_row(&row_prev, &row_curr, &row_preserved, dst);
    }
}

impl GoLEngine for SIMDEngine {
    fn from_pattern(pattern: &Pattern, topology: Topology) -> Result<Self> {
        if topology != Topology::Torus {
            return Err(anyhow!("Only torus topology is supported by SIMDEngine"));
        }
        if pattern.get_size_log2() < 7 {
            return Err(anyhow!("Pattern is too small for SIMDEngine"));
        }

        let packed_cells = pattern.to_format(PatternFormat::PackedCells)?;
        let n = 1 << pattern.get_size_log2();
        let mut data = Vec::with_capacity(n * n / 64);
        for chunk in packed_cells.chunks(8) {
            let mut bytes = [0; 8];
            bytes[..].copy_from_slice(chunk);
            data.push(u64::from_le_bytes(bytes));
        }

        Ok(Self { data, n })
    }

    fn current_state(&self) -> Pattern {
        let packed_cells = self
            .data
            .iter()
            .flat_map(|&chunk| chunk.to_le_bytes())
            .collect::<Vec<u8>>();
        Pattern::from_format(PatternFormat::PackedCells, &packed_cells)
            .expect("A bug in SIMDEngine")
    }

    fn update(&mut self, generations_log2: u32) -> [BigInt; 2] {
        if generations_log2 >= 64 {
            panic!(
                "Generation count 2^{} is too large and would cause excessive runtime",
                generations_log2
            );
        }
        for _ in 0..1u64 << generations_log2 {
            self.update_inner();
        }
        [BigInt::ZERO; 2]
    }

    fn bytes_total(&self) -> usize {
        self.data.capacity() * size_of::<u64>()
    }
}
