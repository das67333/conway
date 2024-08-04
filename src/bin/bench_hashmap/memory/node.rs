#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIdx(u32);

impl NodeIdx {
    pub fn new(idx: u32) -> Self {
        NodeIdx(idx)
    }

    pub fn null() -> Self {
        NodeIdx(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn get(&self) -> usize {
        self.0 as usize
    }
}

#[repr(align(32))]
#[derive(Clone)]
pub struct QuadTreeNode {
    // 1) nw == null means that the node is a leaf
    // then (ne + sw * 2^32) are the cells of the leaf
    // 2) nw != null means that the node is a composite
    // then nw, ne, sw, se are the pointers to the children
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    // for using in linked list
    pub next: NodeIdx,
    // cached result of update; for leaf nodes is unused
    pub cache: NodeIdx,
    // total number of alive cells in the subtree
    pub population: f64,
}

impl QuadTreeNode {
    pub fn leaf_hash(cells: [u8; 8]) -> usize {
        let mut h = u64::from_le_bytes(cells);
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h as usize
    }

    pub fn node_hash(nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> usize {
        let mut h = 5 * nw.0 as u64 + 17 * ne.0 as u64 + 257 * sw.0 as u64 + 65537 * se.0 as u64;
        h += h >> 11;
        h as usize
    }

    /// Returns the cells of a leaf node row by row.
    pub fn leaf_cells(&self) -> [u8; 8] {
        (self.ne.0 as u64 | (self.sw.0 as u64) << 32).to_le_bytes()
    }

    pub fn leaf_nw(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.ne.0 >> (i * 8) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_ne(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.ne.0 >> (i * 8 + 4) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_sw(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.sw.0 >> (i * 8) & 0xF) << (i * 4);
        }
        result as u16
    }

    pub fn leaf_se(&self) -> u16 {
        let mut result = 0;
        for i in 0..4 {
            result |= (self.sw.0 >> (i * 8 + 4) & 0xF) << (i * 4);
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
            cache: NodeIdx::null(),
            population: 0.,
        }
    }
}
