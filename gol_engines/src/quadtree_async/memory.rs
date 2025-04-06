use std::cell::UnsafeCell;

use super::{NodeIdx, QuadTreeNode, LEAF_SIZE_LOG2};
use crate::NiceInt;

/// Wrapper around MemoryManager::find_or_create_node that prefetches the node from the hashtable.
pub struct PrefetchedNode {
    mem: *mut MemoryManager,
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub is_leaf: bool,
    pub hash: usize,
}

impl PrefetchedNode {
    pub fn new(
        mem: &MemoryManager,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> Self {
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        let idx = hash & (unsafe { (*mem.base.get()).hashtable.len() } - 1);

        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::*;
            _mm_prefetch::<_MM_HINT_T0>(
                (*mem.base.get()).hashtable.get_unchecked(idx) as *const QuadTreeNode as *const i8
            );
        }
        Self {
            mem: mem as *const MemoryManager as *mut MemoryManager,
            nw,
            ne,
            sw,
            se,
            is_leaf: size_log2 == LEAF_SIZE_LOG2,
            hash,
        }
    }

    pub fn find(&self) -> NodeIdx {
        unsafe {
            (*(*self.mem).base.get()).find_or_create_inner(
                self.nw,
                self.ne,
                self.sw,
                self.se,
                self.hash,
                self.is_leaf,
            )
        }
    }
}

pub struct MemoryManager {
    base: UnsafeCell<MemoryManagerRaw>,
}

impl MemoryManager {
    /// Create a new memory manager with a default capacity.
    pub fn new() -> Self {
        Self::with_capacity(1 << 26)
    }

    /// Create a new memory manager with a given capacity.
    ///
    /// `cap` must be a power of two!
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            base: UnsafeCell::new(MemoryManagerRaw::with_capacity(cap)),
        }
    }

    /// Get a const reference to the node with the given index.
    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { (*self.base.get()).get(idx) }
    }

    /// Get a mutable reference to the node with the given index.
    pub fn get_mut(&self, idx: NodeIdx) -> &mut QuadTreeNode {
        unsafe { (*self.base.get()).get_mut(idx) }
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_or_create_leaf_from_parts(&self, nw: u16, ne: u16, sw: u16, se: u16) -> NodeIdx {
        /// See Morton order: https://en.wikipedia.org/wiki/Z-order_curve
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

        let cells = demorton_u64(nw, ne, sw, se).to_le_bytes();
        self.find_or_create_leaf_from_array(cells)
    }

    pub fn find_or_create_leaf_from_u64(&self, value: u64) -> NodeIdx {
        self.find_or_create_leaf_from_array(value.to_le_bytes())
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_or_create_leaf_from_array(&self, cells: [u8; 8]) -> NodeIdx {
        let nw = NodeIdx(u32::from_le_bytes(cells[0..4].try_into().unwrap()));
        let ne = NodeIdx(u32::from_le_bytes(cells[4..8].try_into().unwrap()));
        let [sw, se] = [NodeIdx(0); 2];
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { (*self.base.get()).find_or_create_inner(nw, ne, sw, se, hash, true) }
    }

    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    #[inline]
    pub fn find_or_create_node(
        &self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
    ) -> NodeIdx {
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { (*self.base.get()).find_or_create_inner(nw, ne, sw, se, hash, false) }
    }

    pub fn clear_cache(&mut self) {
        self.base.get_mut().clear_cache();
    }

    pub fn bytes_total(&self) -> usize {
        unsafe { (*self.base.get()).bytes_total() }
    }

    /// Statistics about the memory manager that are fast to compute.
    pub fn stats_fast(&self) -> String {
        unsafe { (*self.base.get()).stats_fast() }
    }
}

/// Hashtable that stores nodes of the quadtree
struct MemoryManagerRaw {
    // buffer where heads of linked lists are stored
    hashtable: Vec<QuadTreeNode>,
    // total number of elements in the hashtable
    pub ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were not found and therefore inserted
    misses: u64,
}

impl MemoryManagerRaw {
    // control byte:
    // 00000000     -> empty
    // 00111111     -> deleted
    // 01<hash>     -> full (leaf)
    // 1<hash>      -> full (node)
    const CTRL_EMPTY: u8 = 0;
    const CTRL_DELETED: u8 = (1 << 6) - 1;
    const CTRL_LEAF_BASE: u8 = 1 << 6;
    const CTRL_NODE_BASE: u8 = 1 << 7;

    /// Create a new memory manager with a given capacity.
    ///
    /// `cap` must be a power of two!
    fn with_capacity(cap: usize) -> Self {
        // TODO: speed up
        assert!(cap.is_power_of_two(), "Capacity must be a power of two");
        assert!(u32::try_from(cap).is_ok(), "Capacity must fit into 32 bits");
        Self {
            // first node must be reserved for null
            hashtable: (0..cap).map(|_| QuadTreeNode::default()).collect(),
            ht_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Get a const reference to the node with the given index.
    #[inline]
    fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { self.hashtable.get_unchecked(idx.0 as usize) }
    }

    /// Get a mutable reference to the node with the given index.
    #[inline]
    fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        unsafe { self.hashtable.get_unchecked_mut(idx.0 as usize) }
    }

    /// Find an item in hashtable; if it is not present, it is created and its index in hashtable is returned.
    #[inline]
    unsafe fn find_or_create_inner(
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

        let ctrl = {
            let hash_compressed = {
                let mut h = hash;
                h ^= h >> 16;
                h ^= h >> 8;
                h as u8
            };
            if is_leaf {
                Self::CTRL_LEAF_BASE | ((Self::CTRL_LEAF_BASE - 1) & hash_compressed)
            } else {
                Self::CTRL_NODE_BASE | hash_compressed
            }
        };

        let mut first_deleted = None;
        loop {
            let n = self.hashtable.get_unchecked(index);
            if n.ctrl == ctrl && n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                self.hits += 1;
                break;
            }

            if n.ctrl == Self::CTRL_EMPTY {
                self.hashtable[index] = QuadTreeNode {
                    nw,
                    ne,
                    sw,
                    se,
                    cache: NodeIdx(0),
                    has_cache: false,
                    gc_marked: false,
                    ctrl,
                };
                self.ht_size += 1;
                self.misses += 1;
                break;
            }
            if n.ctrl == Self::CTRL_DELETED && first_deleted.is_none() {
                first_deleted = Some(index);
            }

            index = index.wrapping_add(1) & mask;
        }

        NodeIdx(index as u32)
    }

    pub fn clear_cache(&mut self) {
        for n in self.hashtable.iter_mut() {
            n.has_cache = false;
        }
    }

    pub fn bytes_total(&self) -> usize {
        self.hashtable.len() * std::mem::size_of::<QuadTreeNode>()
    }

    /// Statistics about the memory manager that are fast to compute.
    pub fn stats_fast(&self) -> String {
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

    /// Statistics about the memory manager that are slow to compute.
    pub fn stats_slow(&self) -> String {
        unimplemented!()
        // let mut size_log2_cnt: Vec<u64> = vec![];

        // for mut n in self.hashtable.iter() {
        //     if n.ctrl == Self::CTRL_EMPTY || n.ctrl == Self::CTRL_DELETED {
        //         continue;
        //     }
        //     let mut height = 0;
        //     while !n.is_leaf() {
        //         n = self.get(n.nw);
        //         height += 1;
        //     }
        //     if size_log2_cnt.len() <= height {
        //         size_log2_cnt.resize(height + 1, 0);
        //     }
        //     size_log2_cnt[height] += 1;
        // }

        // let sum = size_log2_cnt.iter().sum::<u64>();

        // let mut s = "\nNodes' sizes (side lengths) distribution:\n".to_string();
        // s.push_str(&format!("total - {}\n", NiceInt::from(sum)));
        // for (height, count) in size_log2_cnt.iter().enumerate() {
        //     let percent = count * 100 / sum;
        //     if percent == 0 {
        //         continue;
        //     }
        //     s.push_str(&format!(
        //         "2^{:<2} -{:>3}%\n",
        //         LEAF_SIZE.ilog2() + height as u32,
        //         percent,
        //     ));
        // }

        // s.push('\n');

        // s
    }
}
