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
    pub ctrl: u8, // control byte for hashmap
}

impl QuadTreeNode {
    // control byte:
    // 11111111     -> empty
    // 11000000     -> deleted
    // 10<hash>     -> full (leaf)
    // 0<hash>      -> full (node)
    pub const CTRL_EMPTY: u8 = !0;
    pub const CTRL_DELETED: u8 = 3 << 6;
    pub const CTRL_LEAF_BASE: u8 = 2 << 6;
    pub const CTRL_LEAF_MASK: u8 = (1 << 6) - 1;
    pub const CTRL_NODE_MASK: u8 = (1 << 7) - 1;

    #[inline]
    pub fn is_leaf(&self) -> bool {
        self.ctrl >> 6 == 2
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
            ctrl: Self::CTRL_EMPTY,
        }
    }
}
