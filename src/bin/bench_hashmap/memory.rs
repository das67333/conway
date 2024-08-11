use super::{NodeIdx, node::QuadTreeNode};
// const HASHTABLE_MAX_LOAD_FACTOR: f64 = 0.7;

/// Hashtable that stores nodes of the quadtree
pub struct MemoryManager {
    // buffer where heads of linked lists are stored
    hashtable: Vec<QuadTreeNode>,
    // total number of elements in the hashtable
    pub ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were not found and therefore inserted
    misses: u64,
}

// #[cfg(feature = "prefetch")]
// pub struct PrefetchedNode {
//     pub nw: NodeIdx,
//     pub ne: NodeIdx,
//     pub sw: NodeIdx,
//     pub se: NodeIdx,
//     pub hash: usize,
// }

impl MemoryManager {
    /// Create a new memory manager with a default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(1 << 28)
    }

    /// Create a new memory manager with a given capacity.
    ///
    /// `cap` must be a power of two!
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap.is_power_of_two(), "Capacity must be a power of two");
        assert!(u32::try_from(cap).is_ok(), "Capacity must fit into 32 bits");
        Self {
            // first node must be reserved for null
            hashtable: vec![QuadTreeNode::default(); cap],
            ht_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    #[inline]
    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { self.hashtable.get_unchecked(idx.get()) }
    }

    #[inline]
    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        unsafe { self.hashtable.get_unchecked_mut(idx.get()) }
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf(&mut self, cells: [u8; 8]) -> NodeIdx {
        let hash = QuadTreeNode::leaf_hash(cells);
        let nw = NodeIdx::new(u32::from_le_bytes(cells[0..4].try_into().unwrap()));
        let ne = NodeIdx::new(u32::from_le_bytes(cells[4..8].try_into().unwrap()));
        let [sw, se] = [NodeIdx::null(); 2];
        self.find_inner(nw, ne, sw, se, hash, true)
    }

    #[inline]
    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
        let hash = QuadTreeNode::node_hash(nw, ne, sw, se);
        self.find_inner(nw, ne, sw, se, hash, false)
    }

    #[inline]
    /// Find an item in hashtable; if it is not present, it is created and its index in hashtable is returned.
    fn find_inner(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        hash: usize,
        is_leaf: bool,
    ) -> NodeIdx {
        let mask = self.hashtable.len() - 1;
        let mut index = hash & mask;
        let mut step = 1;

        // 1<ones>          -> empty
        // 1<zeros>         -> tombstone
        // 0<is_leaf><hash> -> full
        let meta_full = {
            let mut t = hash as u16 & ((1 << 14) - 1);
            if is_leaf {
                t |= 1 << 14;
            }
            t
        };
        loop {
            let node = unsafe { self.hashtable.get_unchecked_mut(index) };
            if node.metadata == meta_full {
                if node.nw == nw && node.ne == ne && node.sw == sw && node.se == se {
                    self.hits += 1;
                    break;
                }
            }
            if node.metadata == QuadTreeNode::METADATA_EMPTY {
                node.nw = nw;
                node.ne = ne;
                node.sw = sw;
                node.se = se;
                assert!(!node.has_next);
                node.metadata = meta_full;
                self.ht_size += 1;
                self.misses += 1;
                break;
            }
            index = (index + step) & mask;
            step += 1;
        }
        NodeIdx::new(index as u32)
    }
}
