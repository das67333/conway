use super::{NodeIdx, QuadTreeNode, LEAF_SIZE_LOG2};
use std::cell::UnsafeCell;

/// Wrapper around MemoryManager::find_or_create_node that prefetches the node from the hashtable.
pub(super) struct PrefetchedNode {
    mem: *mut MemoryManager,
    nw: NodeIdx,
    ne: NodeIdx,
    sw: NodeIdx,
    se: NodeIdx,
    is_leaf: bool,
    hash: usize,
}

impl PrefetchedNode {
    pub(super) fn new(
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

    pub(super) fn find_or_create(&self) -> NodeIdx {
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

pub(super) struct MemoryManager {
    base: UnsafeCell<MemoryManagerRaw>,
}

impl MemoryManager {
    /// Create a new memory manager with a capacity of `1 << cap_log2`.
    pub(super) fn with_capacity(cap_log2: u32) -> Self {
        Self {
            base: UnsafeCell::new(MemoryManagerRaw::with_capacity(cap_log2)),
        }
    }

    /// Get a const reference to the node at the given index.
    pub(super) fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { (*self.base.get()).get(idx) }
    }

    /// Get a mutable reference to the node at the given index.
    pub(super) fn get_mut(&self, idx: NodeIdx) -> &mut QuadTreeNode {
        unsafe { (*self.base.get()).get_mut(idx) }
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub(super) fn find_or_create_leaf_from_parts(
        &self,
        nw: u16,
        ne: u16,
        sw: u16,
        se: u16,
    ) -> NodeIdx {
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

    pub(super) fn find_or_create_leaf_from_u64(&self, value: u64) -> NodeIdx {
        self.find_or_create_leaf_from_array(value.to_le_bytes())
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub(super) fn find_or_create_leaf_from_array(&self, cells: [u8; 8]) -> NodeIdx {
        let nw = NodeIdx(u32::from_le_bytes(cells[0..4].try_into().unwrap()));
        let ne = NodeIdx(u32::from_le_bytes(cells[4..8].try_into().unwrap()));
        let [sw, se] = [NodeIdx(0); 2];
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { (*self.base.get()).find_or_create_inner(nw, ne, sw, se, hash, true) }
    }

    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    pub(super) fn find_or_create_node(
        &self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
    ) -> NodeIdx {
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe { (*self.base.get()).find_or_create_inner(nw, ne, sw, se, hash, false) }
    }

    pub(super) fn clear(&mut self) {
        self.base
            .get_mut()
            .hashtable
            .fill_with(QuadTreeNode::default);
        self.base.get_mut().len = 0;
        self.base.get_mut().poisoned = false;
    }

    pub(super) fn bytes_total(&self) -> usize {
        unsafe { (*self.base.get()).bytes_total() }
    }

    pub(super) fn poisoned(&self) -> bool {
        unsafe { (*self.base.get()).poisoned }
    }
}

/// Hashtable that stores nodes of the quadtree
struct MemoryManagerRaw {
    /// buffer where heads of linked lists are stored
    hashtable: Vec<QuadTreeNode>,
    /// number of nodes that were created
    len: usize,
    /// if true, the hashtable is poisoned and should be restored from the backup
    poisoned: bool,
}

impl MemoryManagerRaw {
    // control byte:
    // 00000000     -> empty
    // 00111111     -> deleted (not used)
    // 01<hash>     -> full (leaf)
    // 1<hash>      -> full (node)
    const CTRL_EMPTY: u8 = 0;
    const CTRL_LEAF_BASE: u8 = 1 << 6;
    const CTRL_NODE_BASE: u8 = 1 << 7;

    /// Create a new memory manager with a capacity of `1 << cap_log2`.
    fn with_capacity(cap_log2: u32) -> Self {
        assert!(
            cap_log2 <= 32,
            "Hashtables bigger than 2^32 are not supported"
        );
        Self {
            // first node must be reserved for null
            hashtable: (0..1u64 << cap_log2)
                .map(|_| QuadTreeNode::default())
                .collect(),
            len: 0,
            poisoned: false,
        }
    }

    /// Get a const reference to the node at the given index.
    #[inline]
    fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        unsafe { self.hashtable.get_unchecked(idx.0 as usize) }
    }

    /// Get a mutable reference to the node at the given index.
    #[inline]
    fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        unsafe { self.hashtable.get_unchecked_mut(idx.0 as usize) }
    }

    /// Find an item in hashtable; if it is not present, it is created and its index in hashtable is returned.
    unsafe fn find_or_create_inner(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        hash: usize,
        is_leaf: bool,
    ) -> NodeIdx {
        if self.poisoned {
            return NodeIdx(0);
        }

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

        loop {
            let n = self.hashtable.get_unchecked(index);
            if n.ctrl == ctrl && n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                break;
            }

            if n.ctrl == Self::CTRL_EMPTY {
                *self.hashtable.get_unchecked_mut(index) = QuadTreeNode {
                    nw,
                    ne,
                    sw,
                    se,
                    ctrl,
                    ..Default::default()
                };
                self.len += 1;
                if self.len > self.hashtable.len() * 3 / 4 {
                    self.poisoned = true;
                    return NodeIdx(0);
                }
                break;
            }

            index = index.wrapping_add(1) & mask;
        }

        NodeIdx(index as u32)
    }

    fn bytes_total(&self) -> usize {
        self.hashtable.len() * std::mem::size_of::<QuadTreeNode>()
    }
}
