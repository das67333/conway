use super::{ChunkVec, NodeIdx, QuadTreeNode, LEAF_SIZE_LOG2};
use crate::NiceInt;

const CHUNK_SIZE: usize = 1 << 13;

/// Wrapper around MemoryManager::find_node that prefetches the node from the hashtable.
pub struct PrefetchedNode<Meta> {
    kiv: *mut KIVMap<Meta>,
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub hash: usize,
}

/// Hashtable that stores nodes of the quadtree.
pub struct KIVMap<Meta> {
    // all allocated nodes
    storage: ChunkVec<CHUNK_SIZE, Meta>,
    // buffer where heads of linked lists are stored
    hashtable: Vec<NodeIdx>,
    // how many times elements were found
    pub hits: u64,
    // how many times elements were inserted
    pub misses: u64,
}

pub struct MemoryManager<Meta> {
    layers: Vec<KIVMap<Meta>>,
}

impl<Meta: Clone + Default> PrefetchedNode<Meta> {
    pub fn new(
        mem: &MemoryManager<Meta>,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> Self {
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        let kiv = unsafe {
            mem.layers
                .get_unchecked((size_log2 - LEAF_SIZE_LOG2) as usize)
        };
        let idx = hash & (kiv.hashtable.len() - 1);
        unsafe {
            use std::arch::x86_64::*;
            _mm_prefetch::<_MM_HINT_T0>(
                kiv.hashtable.get_unchecked(idx) as *const NodeIdx as *const i8
            );
        }
        Self {
            kiv: kiv as *const KIVMap<Meta> as *mut KIVMap<Meta>,
            nw,
            ne,
            sw,
            se,
            hash,
        }
    }

    pub fn find(&self) -> NodeIdx {
        unsafe { (*self.kiv).find(self.nw, self.ne, self.sw, self.se, self.hash) }
    }
}

impl<Meta: Clone + Default> KIVMap<Meta> {
    pub fn new() -> Self {
        assert!(CHUNK_SIZE.is_power_of_two(), "important for performance");
        assert!(u32::try_from(CHUNK_SIZE).is_ok(), "u32 is insufficient");
        // reserving NodeIdx(0) for blank node
        Self {
            storage: ChunkVec::new(),
            hashtable: vec![NodeIdx(0); CHUNK_SIZE],
            hits: 0,
            misses: 0,
        }
    }

    pub unsafe fn rehash(&mut self) {
        let new_size = self.hashtable.len() << 1;
        assert!(u32::try_from(new_size).is_ok(), "u32 is insufficient");
        let mut new_buf = vec![NodeIdx(0); new_size];
        for mut node in std::mem::take(&mut self.hashtable) {
            while node != NodeIdx(0) {
                let n = &self.storage[node];
                let hash = QuadTreeNode::<Meta>::hash(n.nw, n.ne, n.sw, n.se);
                let next = n.next;
                let index = hash & (new_size - 1);
                self.storage[node].next = *new_buf.get_unchecked(index);
                *new_buf.get_unchecked_mut(index) = node;
                node = next;
            }
        }
        self.hashtable = new_buf;
    }

    /// Find an item in hashtable; if it is not present, it is created.
    /// Returns its index in hashtable.
    #[inline]
    pub unsafe fn find(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        hash: usize,
    ) -> NodeIdx {
        if nw == NodeIdx(0) && ne == NodeIdx(0) && sw == NodeIdx(0) && se == NodeIdx(0) {
            return NodeIdx(0);
        }

        let i = hash & (self.hashtable.len() - 1);
        let mut node = *self.hashtable.get_unchecked(i);
        let mut prev = NodeIdx(0);
        // search for the node in the linked list
        while node != NodeIdx(0) {
            let n = &self.storage[node];
            let next = n.next;
            if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // move the node to the front of the list
                if prev != NodeIdx(0) {
                    self.storage[prev].next = next;
                    self.storage[node].next = *self.hashtable.get_unchecked(i);
                    *self.hashtable.get_unchecked_mut(i) = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = next;
        }

        self.misses += 1;
        let idx = self.storage.allocate();
        self.storage[idx] = QuadTreeNode {
            nw,
            ne,
            sw,
            se,
            next: *self.hashtable.get_unchecked(i),
            ..Default::default()
        };
        *self.hashtable.get_unchecked_mut(i) = idx;
        if self.storage.len() > self.hashtable.len() {
            // TODO: как удалять мусор
            self.rehash();
        }
        idx
    }

    pub fn filter_unmarked_from_hashtable(&mut self) {
        for idx in self.hashtable.iter_mut() {
            let (mut curr, mut marked) = (*idx, NodeIdx(0));
            while curr != NodeIdx(0) {
                let next = self.storage[curr].next;
                if self.storage[curr].gc_marked {
                    self.storage[curr].next = marked;
                    marked = curr;
                }
                curr = next;
            }

            *idx = marked;
        }
    }

    pub fn gc_finish(&mut self) {
        self.filter_unmarked_from_hashtable();
        self.storage.deallocate_unmarked_and_unmark();
    }

    pub fn bytes_total(&self) -> usize {
        self.storage.bytes_total() + self.hashtable.capacity() * std::mem::size_of::<NodeIdx>()
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn capacity(&self) -> usize {
        self.storage.capacity()
    }
}

impl<Meta: Clone + Default> MemoryManager<Meta> {
    /// Create a new memory manager.
    pub fn new() -> Self {
        Self {
            layers: vec![KIVMap::new()],
        }
    }

    /// Get a const reference to the node with the given index.
    #[inline]
    pub fn get(&self, idx: NodeIdx, size_log2: u32) -> &QuadTreeNode<Meta> {
        let i = (size_log2 - LEAF_SIZE_LOG2) as usize;
        debug_assert!(self.layers.len() > i && self.layers[i].capacity() > idx.0 as usize);
        unsafe { &self.layers.get_unchecked(i).storage[idx] }
    }

    /// Get a mutable reference to the node with the given index.
    #[inline]
    pub fn get_mut(&mut self, idx: NodeIdx, size_log2: u32) -> &mut QuadTreeNode<Meta> {
        let i = (size_log2 - LEAF_SIZE_LOG2) as usize;
        debug_assert!(self.layers.len() > i && self.layers[i].capacity() > idx.0 as usize);
        unsafe { &mut self.layers.get_unchecked_mut(i).storage[idx] }
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_leaf_from_parts(&mut self, nw: u16, ne: u16, sw: u16, se: u16) -> NodeIdx {
        let [mut nw, mut ne, mut sw, mut se] = [nw as u64, ne as u64, sw as u64, se as u64];
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
        self.find_leaf_from_u64(cells)
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf_from_rows(&mut self, cells: [u8; 8]) -> NodeIdx {
        self.find_leaf_from_u64(u64::from_le_bytes(cells))
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is u64 built by concatenating rows of cells.
    pub fn find_leaf_from_u64(&mut self, cells: u64) -> NodeIdx {
        let nw = NodeIdx(cells as u32);
        let ne = NodeIdx((cells >> 32) as u32);
        let [sw, se] = [NodeIdx(0); 2];
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        unsafe { self.layers.get_unchecked_mut(0).find(nw, ne, sw, se, hash) }
    }

    /// Find a node with the given parts.
    ///
    /// `size_log2` is related to the result! `nw`, `ne`, `sw`, `se` are `size_log2 - 1`
    ///
    /// If the node is not found, it is created.
    #[inline]
    pub fn find_node(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let i = (size_log2 - LEAF_SIZE_LOG2) as usize;
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        if self.layers.len() <= i {
            self.layers.resize_with(i + 1, KIVMap::new);
        }
        unsafe { self.layers.get_unchecked_mut(i).find(nw, ne, sw, se, hash) }
    }

    pub fn gc_finish(&mut self) {
        for kiv in self.layers.iter_mut() {
            kiv.gc_finish();
        }
    }

    pub fn bytes_total(&self) -> usize {
        self.layers.iter().map(|m| m.bytes_total()).sum::<usize>()
    }

    /// Get statistics about the memory manager.
    pub fn stats_fast(&self) -> String {
        let mut s = String::new();

        s += &format!(
            "Memory spent on kivtables: {} MB\n",
            NiceInt::from_usize(self.bytes_total() >> 20),
        );

        let total_misses = self.layers.iter().map(|m| m.misses).sum::<u64>();
        let total_hits = self.layers.iter().map(|m| m.hits).sum::<u64>();
        s += &format!(
            "Hashtable misses / hits: {} / {}\n",
            NiceInt::from(total_misses),
            NiceInt::from(total_hits),
        );

        let nodes_total = self.layers.iter().map(|m| m.len()).sum::<usize>();
        s += "Nodes' sizes (side lengths) distribution:\n";
        s += &format!("total - {}\n", NiceInt::from_usize(nodes_total));
        for (i, m) in self.layers.iter().enumerate() {
            let percent = m.len() * 100 / nodes_total;
            if percent == 0 {
                continue;
            }
            s += &format!("2^{:<2} -{:>3}%\n", LEAF_SIZE_LOG2 + i as u32, percent,);
        }
        s
    }
}
