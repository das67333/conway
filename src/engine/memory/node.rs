#[repr(align(8))]
#[derive(Clone)]
pub struct QuadTreeNode {
    // 1) nw == null means that the node is a leaf
    // then ne is the cells of the leaf
    // 2) nw != null means that the node is a composite
    // then nw, ne, sw, se are the pointers to the children
    pub nw: *mut QuadTreeNode,
    pub ne: *mut QuadTreeNode,
    pub sw: *mut QuadTreeNode,
    pub se: *mut QuadTreeNode,
    // for using in linked list
    pub next: *mut QuadTreeNode,
    // cached result of update; for leaf nodes is unused
    pub cache: *mut QuadTreeNode,
    // total number of alive cells in the subtree
    pub population: f64,
}

impl QuadTreeNode {
    pub fn leaf_hash(cells: u64) -> usize {
        let mut h = cells;
        h ^= h >> 33;
        h = h.wrapping_mul(0xff51afd7ed558ccd);
        h ^= h >> 33;
        h = h.wrapping_mul(0xc4ceb9fe1a85ec53);
        h ^= h >> 33;
        h as usize
    }

    pub fn node_hash(
        nw: *mut QuadTreeNode,
        ne: *mut QuadTreeNode,
        sw: *mut QuadTreeNode,
        se: *mut QuadTreeNode,
    ) -> usize {
        let (nw, ne, sw, se) = (nw as usize, ne as usize, sw as usize, se as usize);
        let mut h = 5 * nw + 17 * ne + 257 * sw + 65537 * se;
        h += h >> 11;
        h
    }
}

impl Default for QuadTreeNode {
    fn default() -> Self {
        Self {
            nw: std::ptr::null_mut(),
            ne: std::ptr::null_mut(),
            sw: std::ptr::null_mut(),
            se: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
            cache: std::ptr::null_mut(),
            population: 0.0,
        }
    }
}
