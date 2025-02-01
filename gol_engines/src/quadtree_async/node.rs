use std::sync::OnceLock;

/// Location of a node is determined by its `idx` and `size_log2`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct NodeIdx(pub u32);

unsafe impl Send for NodeIdx {}

/// A node of the quadtree.
///
/// If the node is a leaf, `nw` and `ne` are the data.
// #[repr(align(4))]
#[derive(Default)]
pub struct QuadTreeNode {
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub next: NodeIdx,            // next item in hashtable bucket, 0 is the end
    pub cache: OnceLock<NodeIdx>, // cached result of update
    pub gc_marked: bool,
    // pub meta: Meta, // metadata for engine: () for hashlife and u64 for streamlife // TODO
}

impl QuadTreeNode {
    /// For blank nodes (without population) must return zero.
    /// They are guaranteed to have all parts equal to NodeIdx(0).
    pub fn hash(nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> usize {
        let h = 0u32
            .wrapping_add(nw.0.wrapping_mul(5))
            .wrapping_add(ne.0.wrapping_mul(17))
            .wrapping_add(sw.0.wrapping_mul(257))
            .wrapping_add(se.0.wrapping_mul(65537));
        h.wrapping_add(h >> 11) as usize
    }

    pub fn parts(&self) -> [NodeIdx; 4] {
        [self.nw, self.ne, self.sw, self.se]
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
