use super::{NodeIdx, QuadTreeNode};
use crate::NiceInt;

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

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_leaf_from_parts(&mut self, nw: u16, ne: u16, sw: u16, se: u16) -> NodeIdx {
        let cells = Self::demorton_u64(nw, ne, sw, se).to_le_bytes();
        self.find_leaf(cells)
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf(&mut self, cells: [u8; 8]) -> NodeIdx {
        let nw = NodeIdx::new(u32::from_le_bytes(cells[0..4].try_into().unwrap()));
        let ne = NodeIdx::new(u32::from_le_bytes(cells[4..8].try_into().unwrap()));
        let [sw, se] = [NodeIdx::null(); 2];
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { self.find_inner(nw, ne, sw, se, hash, true) }
    }

    #[inline]
    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { self.find_inner(nw, ne, sw, se, hash, false) }
    }

    #[inline]
    /// Find an item in hashtable; if it is not present, it is created and its index in hashtable is returned.
    unsafe fn find_inner(
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
        let mut step = 1u8;

        // 1<ones>          -> empty
        // 1<zeros>         -> deleted
        // 0<is_leaf><hash> -> full
        let ctrl_full = {
            let hash_compressed = {
                let mut h = hash;
                h ^= h >> 16;
                h ^= h >> 8;
                h as u8
            };
            if is_leaf {
                QuadTreeNode::CTRL_LEAF_BASE | (QuadTreeNode::CTRL_LEAF_MASK & hash_compressed)
            } else {
                QuadTreeNode::CTRL_NODE_MASK & hash_compressed
            }
        };

        loop {
            let n = unsafe { self.hashtable.get_unchecked(index) };
            if n.ctrl == ctrl_full && n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                self.hits += 1;
                break;
            }

            if n.ctrl == QuadTreeNode::CTRL_EMPTY {
                self.hashtable[index] = QuadTreeNode {
                    nw,
                    ne,
                    sw,
                    se,
                    next: NodeIdx::null(),
                    has_next: false,
                    ctrl: ctrl_full,
                };
                self.ht_size += 1;
                self.misses += 1;
                break;
            }

            index = (index + step as usize) & mask;
            step = step.wrapping_add(1);
        }

        NodeIdx::new(index as u32)
    }

    /// Get statistics about the memory manager.
    pub fn stats(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!(
            "hashtable size/capacity: {}/{} MB\n",
            NiceInt::from_usize((self.ht_size * std::mem::size_of::<QuadTreeNode>()) >> 20),
            NiceInt::from_usize((self.hashtable.len() * std::mem::size_of::<QuadTreeNode>()) >> 20),
        ));

        s.push_str(&format!(
            "hashtable load factor: {:.3}\n",
            self.ht_size as f64 / self.hashtable.len() as f64,
        ));

        s.push_str(&format!(
            "hashtable misses / hits: {} / {}\n",
            NiceInt::from(self.misses),
            NiceInt::from(self.hits),
        ));

        s
    }

    // /// Prefetch the node with the given parts.
    // #[cfg(feature = "prefetch")]
    // pub fn setup_prefetch(
    //     &self,
    //     nw: NodeIdx,
    //     ne: NodeIdx,
    //     sw: NodeIdx,
    //     se: NodeIdx,
    // ) -> PrefetchedNode {
    //     let hash = QuadTreeNode::node_hash(nw, ne, sw, se);
    //     let idx = hash & (self.hashtable.len() - 1);
    //     unsafe {
    //         std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
    //             self.get(self.hashtable[idx]) as *const QuadTreeNode as *const i8,
    //         );
    //     }
    //     PrefetchedNode {
    //         nw,
    //         ne,
    //         sw,
    //         se,
    //         hash,
    //     }
    // }

    // /// Find a node with the given parts; use the prefetched node to speed up the search.
    // /// If the node is not found, it is created.
    // #[cfg(feature = "prefetch")]
    // pub fn find_node_prefetched(&mut self, prefetched: &PrefetchedNode) -> NodeIdx {
    //     let index = prefetched.hash & (self.hashtable.len() - 1);
    //     let (nw, ne, sw, se) = (prefetched.nw, prefetched.ne, prefetched.sw, prefetched.se);
    //     let mut node = self.hashtable[index];
    //     let mut prev = NodeIdx::null();
    //     // search for the node in the linked list
    //     while !node.is_null() {
    //         let n = self.get(node);
    //         let next = n.next;
    //         if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
    //             // // prefetch cache
    //             // unsafe {
    //             //     std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
    //             //         self.get(n.cache) as *const QuadTreeNode as *const i8,
    //             //     );
    //             // }
    //             // move the node to the front of the list
    //             if !prev.is_null() {
    //                 self.get_mut(prev).next = n.next;
    //                 self.get_mut(node).next = self.hashtable[index];
    //                 self.hashtable[index] = node;
    //             }
    //             self.hits += 1;
    //             return node;
    //         }
    //         prev = node;
    //         node = next;
    //     }
    //     self.misses += 1;
    //     node = self.new_node();
    //     let n = self.get_mut(node);
    //     n.nw = nw;
    //     n.ne = ne;
    //     n.sw = sw;
    //     n.se = se;
    //     self.insert(index, node);
    //     node
    // }

    fn demorton_u64(nw: u16, ne: u16, sw: u16, se: u16) -> u64 {
        let (mut nw, mut ne, mut sw, mut se) = (nw as u64, ne as u64, sw as u64, se as u64);
        let mut cells = 0;
        let mut shift = 0;
        for _ in 0..4 {
            cells |= (nw & 0xF) << shift;
            nw >>= 4;
            shift += 4;
            cells |= (ne & 0xF) << shift;
            ne >>= 4;
            shift += 4;
        }
        for _ in 0..4 {
            cells |= (sw & 0xF) << shift;
            sw >>= 4;
            shift += 4;
            cells |= (se & 0xF) << shift;
            se >>= 4;
            shift += 4;
        }
        cells
    }
}

#[test]
fn f() {}
