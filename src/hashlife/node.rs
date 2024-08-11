#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIdx(u32);

impl NodeIdx {
    #[inline]
    pub fn new(idx: u32) -> Self {
        NodeIdx(idx)
    }

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
    // 1<zeros>         -> tombstone
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

    #[inline]
    pub fn leaf_hash(cells: [u8; 8]) -> usize {
        let mut h = u64::from_le_bytes(cells);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h as usize
    }

    #[inline]
    pub fn node_hash(nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> usize {
        // 6364136223846793005
        //
        // for (int i = 1; i < k64; i++) {
        //     h -= (h << 7);
        //     h += hcopy[i];
        // }
        // let (h0, h1) = (nw.0 as u64 | (ne.0 as u64) << 32, sw.0 as u64 | (se.0 as u64) << 32);
        // (h0 + h1 * 6364136223846793005) as usize

        // 1 + 2^2, 3 + 2^4, 5 + 2^8, 7 + 2^16
        let mut h = 5 * nw.0 as u64 + 17 * ne.0 as u64 + 257 * sw.0 as u64 + 65537 * se.0 as u64;
        h += h >> 11;
        h as usize
        //     let mut h = 0u32
        //     .wrapping_add(nw.0.wrapping_mul(5))
        //     .wrapping_add(ne.0.wrapping_mul(17))
        //     .wrapping_add(sw.0.wrapping_mul(257))
        //     .wrapping_add(se.0.wrapping_mul(65537));
        // h += h >> 11;
        // h as usize
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
