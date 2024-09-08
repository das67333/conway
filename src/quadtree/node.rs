#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NodeIdx(pub u32);

#[repr(align(8))]
#[derive(Clone, Default)]
pub struct QuadTreeNode<Meta> {
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub next: NodeIdx,
    pub cache: NodeIdx, // cached result of update
    pub has_cache: bool,
    pub meta: Meta, // metadata for engine: () for hashlife and u64 for streamlife
}

impl<Meta> QuadTreeNode<Meta> {
    // For blank nodes (without population) must return zero
    #[inline]
    pub fn hash(nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> usize {
        let h = 0u32
            .wrapping_add(nw.0.wrapping_mul(5))
            .wrapping_add(ne.0.wrapping_mul(17))
            .wrapping_add(sw.0.wrapping_mul(257))
            .wrapping_add(se.0.wrapping_mul(65537));
        h.wrapping_add(h >> 11) as usize
    }

    /// Returns the cells of a leaf node row by row.
    pub fn leaf_cells(&self) -> [u8; 8] {
        (self.nw.0 as u64 | (self.ne.0 as u64) << 32).to_le_bytes()
    }

    pub fn leaf_nw(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.nw.0 >> (i * 8) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_ne(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.nw.0 >> (i * 8 + 4) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_sw(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.ne.0 >> (i * 8) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_se(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.ne.0 >> (i * 8 + 4) & 0xF) << (i * 4);
        }
        result as u16
    }
}
