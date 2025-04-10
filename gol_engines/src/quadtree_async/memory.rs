use std::cell::UnsafeCell;

use super::{NodeIdx, QuadTreeNode};

pub struct MemoryManager {
    base: UnsafeCell<MemoryManagerRaw>,
}

unsafe impl Sync for MemoryManager {}

impl MemoryManager {
    /// Create a new memory manager with a default capacity.
    pub fn new() -> Self {
        Self::with_capacity(26)
    }

    /// Create a new memory manager with a capacity of `1 << cap_log2`.
    pub fn with_capacity(cap_log2: u32) -> Self {
        Self {
            base: UnsafeCell::new(MemoryManagerRaw::with_capacity(cap_log2)),
        }
    }

    /// Get a const reference to the node with the given index.
    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { (*self.base.get()).get(idx) }
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
    /// buffer where heads of linked lists are stored
    hashtable: Vec<QuadTreeNode>,
    /// striped locks for the hashtable
    locks: Vec<std::sync::Mutex<()>>,
    /// log2 of hashtable's capacity
    ht_cap_log2: u32,
    // /// total number of elements in the hashtable
    // pub ht_size: u32,
    // /// how many times elements were found in the hashtable
    // hits: AtomicU64,
    // /// how many times elements were not found and therefore inserted
    // misses: AtomicU64,
}

impl MemoryManagerRaw {
    const STRIPED_LOCKS_CNT_LOG2: u32 = 10;

    // control byte:
    // 00000000     -> empty
    // 00111111     -> deleted
    // 01<hash>     -> full (leaf)
    // 1<hash>      -> full (node)
    const CTRL_EMPTY: u8 = 0;
    const CTRL_DELETED: u8 = (1 << 6) - 1;
    const CTRL_LEAF_BASE: u8 = 1 << 6;
    const CTRL_NODE_BASE: u8 = 1 << 7;

    /// Create a new memory manager with a capacity of `1 << cap_log2`.
    fn with_capacity(cap_log2: u32) -> Self {
        // TODO: speed up
        assert!(
            cap_log2 <= 32,
            "Hashtables bigger than 2^32 are not supported"
        );
        Self {
            // first node must be reserved for null
            hashtable: (0..1u64 << cap_log2)
                .map(|_| QuadTreeNode::default())
                .collect(),
            locks: (0..(1 << Self::STRIPED_LOCKS_CNT_LOG2))
                .map(|_| std::sync::Mutex::new(()))
                .collect(),
            ht_cap_log2: cap_log2,
            // ht_size: 0,
            // hits: AtomicU64::new(0),
            // misses: AtomicU64::new(0),
        }
    }

    /// Get a const reference to the node with the given index.
    #[inline]
    fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { self.hashtable.get_unchecked(idx.0 as usize) }
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

        // let mut first_deleted = None;
        let shift = self.ht_cap_log2 - Self::STRIPED_LOCKS_CNT_LOG2;
        let mut lock = self.locks.get_unchecked(index >> shift).lock().unwrap();
        loop {
            let n = self.hashtable.get_unchecked(index);
            if n.ctrl == ctrl && n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // self.hits.fetch_add(1, Ordering::Relaxed);
                break;
            }

            if n.ctrl == Self::CTRL_EMPTY {
                *self.hashtable.get_unchecked_mut(index) = QuadTreeNode {
                    nw,
                    ne,
                    sw,
                    se,
                    cache: tokio::sync::OnceCell::new(),
                    gc_marked: false,
                    ctrl,
                };
                // self.ht_size += 1;
                // self.misses.fetch_add(1, Ordering::Relaxed);
                break;
            }
            // if n.ctrl == Self::CTRL_DELETED && first_deleted.is_none() {
            //     first_deleted = Some(index);
            // }

            let next_index = index.wrapping_add(1) & mask;
            if index >> shift != next_index >> shift {
                drop(lock);
                lock = self.locks[next_index >> shift].lock().unwrap();
            }
            index = next_index;
        }

        NodeIdx(index as u32)
    }

    pub fn clear_cache(&mut self) {
        for n in self.hashtable.iter_mut() {
            n.cache = tokio::sync::OnceCell::new();
        }
    }

    pub fn bytes_total(&self) -> usize {
        self.hashtable.len() * std::mem::size_of::<QuadTreeNode>()
    }

    /// Statistics about the memory manager that are fast to compute.
    pub fn stats_fast(&self) -> String {
        let mut s = String::new();

        // s.push_str(&format!(
        //     "hashtable size/capacity: {}/{} MB\n",
        //     NiceInt::from_usize(
        //         (self.ht_size as usize * std::mem::size_of::<QuadTreeNode>()) >> 20
        //     ),
        //     NiceInt::from_usize((self.hashtable.len() * std::mem::size_of::<QuadTreeNode>()) >> 20),
        // ));

        // s.push_str(&format!(
        //     "hashtable load factor: {:.3}\n",
        //     self.ht_size as f64 / self.hashtable.len() as f64,
        // ));

        // s.push_str(&format!(
        //     "hashtable misses / hits: {} / {}\n",
        //     NiceInt::from(self.misses.load(Ordering::Relaxed)),
        //     NiceInt::from(self.hits.load(Ordering::Relaxed)),
        // ));

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
