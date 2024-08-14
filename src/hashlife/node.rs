#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIdx(u32);

impl NodeIdx {
    #[inline]
    pub fn new(idx: u32) -> Self {
        NodeIdx(idx)
    }

    /// null() might refer to a real node! It is not equivalent to None!
    #[inline]
    pub fn null() -> Self {
        NodeIdx(0)
    }

    #[inline]
    pub fn get(&self) -> usize {
        self.0 as usize
    }
}

#[repr(align(8))]
#[derive(Clone)]
pub struct QuadTreeNode {
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub next: NodeIdx, // cached result of update
    pub has_next: bool,
    // metadata for hashmap:
    // 1<ones>          -> empty
    // 1<zeros>         -> deleted
    // 0<is_leaf><hash> -> full
    pub metadata: u16,
}

impl QuadTreeNode {
    pub const METADATA_EMPTY: u16 = !0;
    pub const METADATA_DELETED: u16 = 1 << 15;

    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.metadata & (1 << 14) != 0
    }

    // Only lower 32 bits are used
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

impl Default for QuadTreeNode {
    fn default() -> Self {
        Self {
            nw: NodeIdx::null(),
            ne: NodeIdx::null(),
            sw: NodeIdx::null(),
            se: NodeIdx::null(),
            next: NodeIdx::null(),
            has_next: false,
            metadata: Self::METADATA_EMPTY,
        }
    }
}
