use super::{ChunkVec, NodeIdx, QuadTreeNode, LEAF_SIDE_LOG2};
use crate::NiceInt;
use std::{cell::UnsafeCell, sync::Mutex};

const CHUNK_SIZE: usize = 1 << 13;

/// Hashtable that stores nodes of the quadtree.
pub struct KIVMap {
    // all allocated nodes
    storage: ChunkVec<CHUNK_SIZE>,
    // buffer where heads of linked lists are stored
    hashtable: Vec<Mutex<NodeIdx>>,
    // how many times elements were found
    pub hits: u64,
    // how many times elements were inserted
    pub misses: u64,
}

pub struct MemoryManager {
    layers: UnsafeCell<Vec<KIVMap>>,
}

impl KIVMap {
    pub fn new() -> Self {
        assert!(CHUNK_SIZE.is_power_of_two(), "important for performance");
        assert!(u32::try_from(CHUNK_SIZE).is_ok(), "u32 is insufficient");
        // reserving NodeIdx(0) for blank node
        Self {
            storage: ChunkVec::new(),
            hashtable: (0..CHUNK_SIZE).map(|_| Mutex::new(NodeIdx(0))).collect(),
            hits: 0,
            misses: 0,
        }
    }

    pub unsafe fn rehash(&mut self) {
        let new_size = self.hashtable.len() << 1;
        assert!(u32::try_from(new_size).is_ok(), "u32 is insufficient");
        let new_buf = (0..new_size)
            .map(|_| Mutex::new(NodeIdx(0)))
            .collect::<Vec<_>>();
        for slot in std::mem::take(&mut self.hashtable) {
            let mut node = slot.lock().unwrap();
            while *node != NodeIdx(0) {
                let n = &self.storage[*node];
                let hash = QuadTreeNode::hash(n.nw, n.ne, n.sw, n.se);
                let next = n.next;
                let index = hash & (new_size - 1);
                let mut new_node = new_buf.get_unchecked(index).lock().unwrap();
                self.storage[*node].next = *new_node;
                *new_node = *node;
                *node = next;
            }
        }
        self.hashtable = new_buf;
    }

    /// Find an item in hashtable; if it is not present, it is created.
    /// Returns its index in hashtable.
    #[inline]
    pub unsafe fn find_or_create(
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
        let mut slot = self.hashtable.get_unchecked(i).lock().unwrap();
        let mut node = *slot;
        let mut prev = NodeIdx(0);
        // search for the node in the linked list
        while node != NodeIdx(0) {
            let n = &self.storage[node];
            if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // move the node to the front of the list
                if prev != NodeIdx(0) {
                    self.storage[prev].next = n.next;
                    self.storage[node].next = *slot;
                    *slot = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = n.next;
        }

        self.misses += 1;
        let idx = self.storage.allocate();
        self.storage[idx] = QuadTreeNode {
            nw,
            ne,
            sw,
            se,
            next: *slot,
            ..Default::default()
        };
        *slot = idx;
        if self.storage.len() > self.hashtable.len() {
            drop(slot);
            // TODO: стоит ли запускать gc
            self.rehash();
        }
        idx
    }

    pub fn filter_unmarked_from_hashtable(&mut self) {
        for slot in self.hashtable.iter_mut() {
            let (mut curr, mut marked) = (*slot.get_mut().unwrap(), NodeIdx(0));
            while curr != NodeIdx(0) {
                let next = self.storage[curr].next;
                if self.storage[curr].gc_marked {
                    self.storage[curr].next = marked;
                    marked = curr;
                }
                curr = next;
            }
            *slot = Mutex::new(marked);
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

impl MemoryManager {
    /// Create a new memory manager.
    pub fn new() -> Self {
        Self {
            layers: UnsafeCell::new(vec![KIVMap::new()]),
        }
    }

    /// Get a const reference to the node with the given index.
    #[inline]
    pub fn get(&self, idx: NodeIdx, size_log2: u32) -> &QuadTreeNode {
        let i = (size_log2 - LEAF_SIDE_LOG2) as usize;
        let layers = unsafe { &*self.layers.get() };
        debug_assert!(layers.len() > i && layers[i].capacity() > idx.0 as usize);
        unsafe {
            let storage = &layers.get_unchecked(i).storage;
            let p = &storage[idx] as *const QuadTreeNode;
            &*p
        }
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_or_create_leaf_from_array(&self, parts: [u16; 4]) -> NodeIdx {
        let [mut nw, mut ne, mut sw, mut se] = parts.map(|x| x as u64);
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
        self.find_or_create_leaf_from_u64(cells)
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is u64 built by concatenating rows of cells.
    pub fn find_or_create_leaf_from_u64(&self, cells: u64) -> NodeIdx {
        let nw = NodeIdx(cells as u32);
        let ne = NodeIdx((cells >> 32) as u32);
        let [sw, se] = [NodeIdx(0); 2];
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        unsafe {
            (*self.layers.get())
                .get_unchecked_mut(0)
                .find_or_create(nw, ne, sw, se, hash)
        }
    }

    pub fn find_or_create_node_from_array(&self, parts: [NodeIdx; 4], size_log2: u32) -> NodeIdx {
        let [nw, ne, sw, se] = parts;
        self.find_or_create_node(nw, ne, sw, se, size_log2)
    }

    /// Find a node with the given parts.
    ///
    /// `size_log2` is related to the result! `nw`, `ne`, `sw`, `se` are `size_log2 - 1`
    ///
    /// If the node is not found, it is created.
    #[inline]
    pub fn find_or_create_node(
        &self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let i = (size_log2 - LEAF_SIDE_LOG2) as usize;
        let hash = QuadTreeNode::hash(nw, ne, sw, se);
        let layers = unsafe { &mut *self.layers.get() };
        if layers.len() <= i {
            layers.resize_with(i + 1, KIVMap::new);
        }
        unsafe { layers.get_unchecked_mut(i).find_or_create(nw, ne, sw, se, hash) }
    }

    /// Recursively mark nodes to rescue them from garbage collection.
    pub fn gc_mark(&mut self, idx: NodeIdx, size_log2: u32) {
        // self.get_mut(idx, size_log2).gc_marked = true;
        if idx == NodeIdx(0) {
            return;
        }

        if size_log2 == LEAF_SIDE_LOG2 {
            return;
        }

        for x in self.get(idx, size_log2).parts() {
            self.gc_mark(x, size_log2 - 1);
        }
        unimplemented!()
    }

    pub fn gc_finish(&self) {
        let layers = unsafe { &mut *self.layers.get() };
        for kiv in layers {
            kiv.gc_finish();
        }
    }

    pub fn bytes_total(&self) -> usize {
        let layers = unsafe { &mut *self.layers.get() };
        layers.iter().map(|m| m.bytes_total()).sum::<usize>()
    }

    /// Get statistics about the memory manager.
    pub fn stats_fast(&self) -> String {
        let mut s = String::new();

        s += &format!(
            "Memory spent on kivtables: {} MB\n",
            NiceInt::from_usize(self.bytes_total() >> 20),
        );

        let layers = unsafe { &mut *self.layers.get() };
        let total_misses = layers.iter().map(|m| m.misses).sum::<u64>();
        let total_hits = layers.iter().map(|m| m.hits).sum::<u64>();
        s += &format!(
            "Hashtable misses / hits: {} / {}\n",
            NiceInt::from(total_misses),
            NiceInt::from(total_hits),
        );

        let nodes_total = layers.iter().map(|m| m.len()).sum::<usize>();
        s += "Nodes' sizes (side lengths) distribution:\n";
        s += &format!("total - {}\n", NiceInt::from_usize(nodes_total));
        for (i, m) in layers.iter().enumerate() {
            let percent = m.len() * 100 / nodes_total;
            if percent == 0 {
                continue;
            }
            s += &format!("2^{:<2} -{:>3}%\n", LEAF_SIDE_LOG2 + i as u32, percent,);
        }
        s
    }
}
