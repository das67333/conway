use super::{NodeIdx, QuadTreeNode, LEAF_SIZE};
use crate::NiceInt;

/// Wrapper around MemoryManager::find_node that prefetches the node from the hashtable.
pub struct PrefetchedNode<Meta> {
    mem: *mut MemoryManager<Meta>,
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub hash: usize,
}

impl<Meta: Clone + Default> PrefetchedNode<Meta> {
    pub fn new(
        mem: &MemoryManager<Meta>,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
    ) -> Self {
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        let idx = hash & (mem.hashtable.len() - 1);
        unsafe {
            use std::arch::x86_64::*;
            _mm_prefetch::<_MM_HINT_T0>(
                mem.hashtable.get_unchecked(idx) as *const QuadTreeNode<Meta> as *const i8
            );
        }
        Self {
            mem: mem as *const MemoryManager<Meta> as *mut MemoryManager<Meta>,
            nw,
            ne,
            sw,
            se,
            hash,
        }
    }

    pub fn find(&self) -> NodeIdx {
        unsafe { (*self.mem).find_inner(self.nw, self.ne, self.sw, self.se, self.hash, false) }
    }
}

/// Hashtable that stores nodes of the quadtree
pub struct MemoryManager<Meta> {
    // buffer where heads of linked lists are stored
    hashtable: Vec<QuadTreeNode<Meta>>,
    // total number of elements in the hashtable
    pub ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were not found and therefore inserted
    misses: u64,
}

impl<Meta: Clone + Default> MemoryManager<Meta> {
    // control byte:
    // 00000000     -> empty
    // 00111111     -> deleted
    // 01<hash>     -> full (leaf)
    // 1<hash>      -> full (node)
    const CTRL_EMPTY: u8 = 0;
    const CTRL_DELETED: u8 = (1 << 6) - 1;
    const CTRL_LEAF_BASE: u8 = 1 << 6;
    const CTRL_LEAF_MASK: u8 = (1 << 6) - 1;
    const CTRL_NODE_BASE: u8 = 1 << 7;

    /// Create a new memory manager with a default capacity.
    pub fn new() -> Self {
        Self::with_capacity(1 << 28)
    }

    /// Create a new memory manager with a given capacity.
    ///
    /// `cap` must be a power of two!
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap.is_power_of_two(), "Capacity must be a power of two");
        assert!(u32::try_from(cap).is_ok(), "Capacity must fit into 32 bits");
        Self {
            // first node must be reserved for null
            hashtable: vec![QuadTreeNode::<Meta>::default(); cap],
            ht_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Get a const reference to the node with the given index.
    #[inline]
    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode<Meta> {
        unsafe { self.hashtable.get_unchecked(idx.0 as usize) }
    }

    /// Get a mutable reference to the node with the given index.
    #[inline]
    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode<Meta> {
        unsafe { self.hashtable.get_unchecked_mut(idx.0 as usize) }
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_leaf_from_parts(&mut self, nw: u16, ne: u16, sw: u16, se: u16) -> NodeIdx {
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
        self.find_leaf(cells)
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf(&mut self, cells: [u8; 8]) -> NodeIdx {
        let nw = NodeIdx(u32::from_le_bytes(cells[0..4].try_into().unwrap()));
        let ne = NodeIdx(u32::from_le_bytes(cells[4..8].try_into().unwrap()));
        let [sw, se] = [NodeIdx(0); 2];
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        unsafe { self.find_inner(nw, ne, sw, se, hash, true) }
    }

    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    #[inline]
    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        unsafe { self.find_inner(nw, ne, sw, se, hash, false) }
    }

    /// Find an item in hashtable; if it is not present, it is created and its index in hashtable is returned.
    #[inline]
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

        let ctrl_full = {
            let hash_compressed = {
                let mut h = hash;
                h ^= h >> 16;
                h ^= h >> 8;
                h as u8
            };
            if is_leaf {
                Self::CTRL_LEAF_BASE | (Self::CTRL_LEAF_MASK & hash_compressed)
            } else {
                Self::CTRL_NODE_BASE | hash_compressed
            }
        };

        loop {
            let n = self.hashtable.get_unchecked(index);
            if n.ctrl == ctrl_full && n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                self.hits += 1;
                break;
            }

            if n.ctrl == Self::CTRL_EMPTY {
                self.hashtable[index] = QuadTreeNode {
                    nw,
                    ne,
                    sw,
                    se,
                    next: NodeIdx(0),
                    has_next: false,
                    ctrl: ctrl_full,
                    meta: Default::default(),
                };
                self.ht_size += 1;
                self.misses += 1;
                break;
            }

            index = (index + step as usize) & mask;
            step = step.wrapping_add(1);
        }

        NodeIdx(index as u32)
    }

    pub fn clear_cache(&mut self) {
        for n in self.hashtable.iter_mut() {
            n.next = NodeIdx(0);
            n.has_next = false;
        }
    }

    /// Statistics about the memory manager that are fast to compute.
    pub fn stats_fast(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!(
            "hashtable size/capacity: {}/{} MB\n",
            NiceInt::from_usize((self.ht_size * std::mem::size_of::<QuadTreeNode<Meta>>()) >> 20),
            NiceInt::from_usize(
                (self.hashtable.len() * std::mem::size_of::<QuadTreeNode<Meta>>()) >> 20
            ),
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
        let mut size_log2_cnt: Vec<u64> = vec![];

        for mut n in self.hashtable.iter() {
            if n.ctrl == Self::CTRL_EMPTY || n.ctrl == Self::CTRL_DELETED {
                continue;
            }
            let mut height = 0;
            while !n.is_leaf() {
                n = self.get(n.nw);
                height += 1;
            }
            if size_log2_cnt.len() <= height {
                size_log2_cnt.resize(height + 1, 0);
            }
            size_log2_cnt[height] += 1;
        }

        let sum = size_log2_cnt.iter().sum::<u64>();

        let mut s = "\nNodes' sizes (side lengths) distribution:\n".to_string();
        s.push_str(&format!("total - {}\n", NiceInt::from(sum)));
        for (height, count) in size_log2_cnt.iter().enumerate() {
            let percent = count * 100 / sum;
            if percent == 0 {
                continue;
            }
            s.push_str(&format!(
                "2^{:<2} -{:>3}%\n",
                LEAF_SIZE.ilog2() + height as u32,
                percent,
            ));
        }

        s.push('\n');

        s
    }
}

impl<Meta: Clone + Default> Default for MemoryManager<Meta> {
    fn default() -> Self {
        Self::new()
    }
}
