use super::{NodeIdx, QuadTreeNode};

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
                mem.hashtable.get_unchecked(idx) as *const NodeIdx as *const i8
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
        unsafe { (*self.mem).find_inner(self.nw, self.ne, self.sw, self.se, self.hash) }
    }
}

const HASHTABLE_BUF_INITIAL_SIZE: usize = 1;
const CHUNK_SIZE: usize = 1 << 13;

/// Hashtable for finding nodes (to avoid duplicates)
pub struct MemoryManager<Meta> {
    // all allocated nodes
    // storage: Vec<Box<[QuadTreeNode<Meta>; CHUNK_SIZE]>>,
    storage: Vec<QuadTreeNode<Meta>>,
    // total number of initiallized nodes in the storage
    // storage_size: usize,
    // buffer where heads of linked lists are stored
    hashtable: Vec<NodeIdx>,
    // total number of elements in the hashtable
    ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were inserted into the hashtable
    misses: u64,
}

impl<Meta: Clone + Default> MemoryManager<Meta> {
    /// Create a new memory manager.
    pub fn new() -> Self {
        assert!(HASHTABLE_BUF_INITIAL_SIZE.is_power_of_two());
        Self {
            // first node must be reserved for null
            // storage: vec![Box::new(std::array::from_fn(|_| {
            //     QuadTreeNode::<Meta>::default()
            // }))],
            storage: vec![QuadTreeNode::<Meta>::default()],
            // storage_size: 1,
            hashtable: vec![NodeIdx(0); HASHTABLE_BUF_INITIAL_SIZE],
            ht_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    pub fn clear_cache(&mut self) {
        // for chunk in self.storage.iter_mut() {
        //     for x in chunk.iter_mut() {
        //         x.has_cache = false;
        //         x.cache = NodeIdx(0);
        //     }
        // }
        for x in self.storage.iter_mut() {
            x.has_cache = false;
            x.cache = NodeIdx(0);
        }
    }

    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode<Meta> {
        // let (i, j) = (idx.0 as usize / CHUNK_SIZE, idx.0 as usize % CHUNK_SIZE);
        unsafe { self.storage.get_unchecked(idx.0 as usize) }
    }

    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode<Meta> {
        // let (i, j) = (idx.0 as usize / CHUNK_SIZE, idx.0 as usize % CHUNK_SIZE);
        unsafe { self.storage.get_unchecked_mut(idx.0 as usize) }
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
        self.find_leaf(cells.to_le_bytes())
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf(&mut self, cells: [u8; 8]) -> NodeIdx {
        let [nw, se] = [NodeIdx(0); 2];
        let v = u64::from_le_bytes(cells);
        let ne = NodeIdx(v as u32);
        let sw = NodeIdx((v >> 32) as u32);
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        self.find_inner(nw, ne, sw, se, hash)
    }

    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    #[inline]
    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
        let hash = QuadTreeNode::<Meta>::hash(nw, ne, sw, se);
        self.find_inner(nw, ne, sw, se, hash)
    }

    #[inline]
    pub fn find_inner(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        hash: usize,
    ) -> NodeIdx {
        let index = hash & (self.hashtable.len() - 1);
        let mut node = unsafe { *self.hashtable.get_unchecked(index) };
        let mut prev = NodeIdx(0);
        // search for the node in the linked list
        while node.0 != 0 {
            let next = self.get(node).next;
            let n = self.get(node);
            if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // move the node to the front of the list
                if prev.0 != 0 {
                    self.get_mut(prev).next = n.next;
                    self.get_mut(node).next = self.hashtable[index];
                    self.hashtable[index] = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = next;
        }
        self.misses += 1;

        node = self.new_node();
        {
            let n = self.get_mut(node);
            n.nw = nw;
            n.ne = ne;
            n.sw = sw;
            n.se = se;
        }
        self.insert(index, node);
        node
    }

    fn new_node(&mut self) -> NodeIdx {
        // if self.storage_size % CHUNK_SIZE == 0 {
        //     self.storage
        //         .push(Box::new(std::array::from_fn(|_| QuadTreeNode::default())));
        // }
        self.storage.push(QuadTreeNode::default());
        // self.storage_size += 1;
        NodeIdx(
            // (self.storage_size - 1)
            (self.storage.len() - 1)
                .try_into()
                .expect("Nodes storage overflowed u32"),
        )
    }

    /// Insert a node into the hashtable.
    /// index must be hash(node) % buf.len(); node must not be present in the hashtable
    fn insert(&mut self, index: usize, node: NodeIdx) {
        self.ht_size += 1;
        self.get_mut(node).next = self.hashtable[index];
        self.hashtable[index] = node;
        if self.ht_size > self.hashtable.len() / 2 {
            self.rehash();
        }
    }

    fn rehash(&mut self) {
        let new_size = self.hashtable.len() * 2;
        let mut new_buf = vec![NodeIdx(0); new_size];
        for i in 0..self.hashtable.len() {
            let mut node = self.hashtable[i];
            while node.0 != 0 {
                let n = self.get(node);
                let hash = QuadTreeNode::<Meta>::hash(n.nw, n.ne, n.sw, n.se);
                let next = n.next;
                let index = hash & (new_size - 1);
                self.get_mut(node).next = new_buf[index];
                new_buf[index] = node;
                node = next;
            }
        }
        self.hashtable = new_buf;
    }

    /// Get statistics about the memory manager.
    pub fn stats_fast(&self) -> String {
        // let mut s = String::new();

        // let mem = self.storage.len() * CHUNK_SIZE * std::mem::size_of::<QuadTreeNode<Meta>>();
        // s.push_str(&format!("memory on nodes: {} MB\n", mem >> 20));

        // s.push_str(&format!(
        //     "memory on hashtable: {} MB\n",
        //     NiceInt::from_usize((self.hashtable.len() * 4) >> 20)
        // ));

        // s.push_str(&format!(
        //     "hashtable elements / buckets: {} / {}\n",
        //     NiceInt::from_usize(self.ht_size),
        //     NiceInt::from_usize(self.hashtable.len())
        // ));

        // s.push_str(&format!(
        //     "hashtable misses / hits: {} / {}\n",
        //     NiceInt::from(self.misses),
        //     NiceInt::from(self.hits)
        // ));

        // s
        String::new()
    }

    pub fn stats_slow(&self) -> String {
        // let mut lengths = vec![];
        // for chain in self.hashtable.iter() {
        //     let mut len = 0;
        //     let mut node = *chain;
        //     while node != NodeIdx(0) {
        //         len += 1;
        //         node = self.get(node).next;
        //     }
        //     if len >= lengths.len() {
        //         lengths.resize(len + 1, 0);
        //     }
        //     lengths[len] += 1;
        // }

        // let sum = lengths.iter().sum::<usize>();
        // let mut s = "Chain lengths distribution:\n".to_string();
        // for (i, &len) in lengths.iter().enumerate() {
        //     s.push_str(&format!(
        //         "{:<2}-{:>3}% ({})\n",
        //         i,
        //         len * 100 / sum,
        //         NiceInt::from_usize(len),
        //     ));
        // }

        // s
        String::new()
    }
}

impl<Meta: Clone + Default> Default for MemoryManager<Meta> {
    fn default() -> Self {
        Self::new()
    }
}
