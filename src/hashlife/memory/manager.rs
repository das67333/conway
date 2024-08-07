use crate::NiceInt;

use super::{NodeIdx, QuadTreeNode};

const HASHTABLE_BUF_INITIAL_SIZE: usize = 1;
const HASHTABLE_MAX_LOAD_FACTOR: f64 = 1.2;
const CHUNK_SIZE: usize = (1 << 18) / std::mem::size_of::<QuadTreeNode>();

/// Hashtable for finding nodes (to avoid duplicates)
pub struct Manager {
    // all allocated nodes
    storage: Vec<Box<[QuadTreeNode; CHUNK_SIZE]>>,
    // total number of initiallized nodes in the storage
    storage_size: usize,
    // buffer where heads of linked lists are stored
    hashtable: Vec<NodeIdx>,
    // total number of elements in the hashtable
    ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were inserted into the hashtable
    misses: u64,
    // i-th element is the number of nodes with size 2^i.
    pub size_distribution_history: Vec<u64>,
}

#[cfg(feature = "prefetch")]
pub struct PrefetchedNode {
    pub nw: NodeIdx,
    pub ne: NodeIdx,
    pub sw: NodeIdx,
    pub se: NodeIdx,
    pub hash: usize,
}

impl Manager {
    /// Create a new memory manager.
    pub fn new() -> Self {
        assert!(HASHTABLE_BUF_INITIAL_SIZE.is_power_of_two());
        Self {
            // first node must be reserved for null
            storage: vec![Box::new(std::array::from_fn(|_| QuadTreeNode::default()))],
            storage_size: 1,
            hashtable: vec![NodeIdx::null(); HASHTABLE_BUF_INITIAL_SIZE],
            ht_size: 0,
            hits: 0,
            misses: 0,
            size_distribution_history: vec![],
        }
    }

    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        let (i, j) = (idx.get() / CHUNK_SIZE, idx.get() % CHUNK_SIZE);
        &self.storage[i][j]
    }

    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        let (i, j) = (idx.get() / CHUNK_SIZE, idx.get() % CHUNK_SIZE);
        &mut self.storage[i][j]
    }

    /// Find a leaf node with the given parts.
    /// If the node is not found, it is created.
    ///
    /// `nw`, `ne`, `sw`, `se` are 16-bit integers, where each 4 bits represent a row of 4 cells.
    pub fn find_leaf_from_parts(
        &mut self,
        nw: u16,
        ne: u16,
        sw: u16,
        se: u16,
        size_log2: u32,
    ) -> NodeIdx {
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
        self.find_leaf(cells.to_le_bytes(), size_log2)
    }

    /// Find a leaf node with the given cells.
    /// If the node is not found, it is created.
    ///
    /// `cells` is an array of 8 bytes, where each byte represents a row of 8 cells.
    pub fn find_leaf(&mut self, cells: [u8; 8], size_log2: u32) -> NodeIdx {
        let index = QuadTreeNode::leaf_hash(cells) & (self.hashtable.len() - 1);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        while !node.is_null() {
            let next = self.get(node).next;
            if self.get(node).nw.is_null() && self.get(node).leaf_cells() == cells {
                // move the node to the front of the list
                if !prev.is_null() {
                    self.get_mut(prev).next = self.get(node).next;
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
            n.nw = NodeIdx::null();
            let cells = u64::from_le_bytes(cells);
            n.ne = NodeIdx::new(cells as u32);
            n.sw = NodeIdx::new((cells >> 32) as u32);
            n.population = cells.count_ones() as f64;
        }
        assert!(size_log2 == 3);
        self.insert(index, node, 3);
        node
    }

    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    pub fn find_node(
        &mut self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
        size_log2: u32,
    ) -> NodeIdx {
        let index = QuadTreeNode::node_hash(nw, ne, sw, se) & (self.hashtable.len() - 1);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        // search for the node in the linked list
        while !node.is_null() {
            let next = self.get(node).next;
            let n = self.get(node);
            if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // move the node to the front of the list
                if !prev.is_null() {
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

        let population = (self.get(nw).population + self.get(ne).population)
            + (self.get(sw).population + self.get(se).population);

        node = self.new_node();
        {
            let n = self.get_mut(node);
            n.nw = nw;
            n.ne = ne;
            n.sw = sw;
            n.se = se;
            n.population = population;
        }
        self.insert(index, node, size_log2);
        node
    }

    /// Prefetch the node with the given parts.
    #[cfg(feature = "prefetch")]
    pub fn setup_prefetch(
        &self,
        nw: NodeIdx,
        ne: NodeIdx,
        sw: NodeIdx,
        se: NodeIdx,
    ) -> PrefetchedNode {
        let hash = QuadTreeNode::node_hash(nw, ne, sw, se);
        let idx = hash & (self.hashtable.len() - 1);
        unsafe {
            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                self.get(self.hashtable[idx]) as *const QuadTreeNode as *const i8,
            );
        }
        PrefetchedNode {
            nw,
            ne,
            sw,
            se,
            hash,
        }
    }

    /// Find a node with the given parts; use the prefetched node to speed up the search.
    /// If the node is not found, it is created.
    #[cfg(feature = "prefetch")]
    pub fn find_node_prefetched(&mut self, prefetched: &PrefetchedNode, size_log2: u32) -> NodeIdx {
        let index = prefetched.hash & (self.hashtable.len() - 1);
        let (nw, ne, sw, se) = (prefetched.nw, prefetched.ne, prefetched.sw, prefetched.se);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        // search for the node in the linked list
        while !node.is_null() {
            let n = self.get(node);
            let next = n.next;
            if n.nw == nw && n.ne == ne && n.sw == sw && n.se == se {
                // // prefetch cache
                // unsafe {
                //     std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                //         self.get(n.cache) as *const QuadTreeNode as *const i8,
                //     );
                // }
                // move the node to the front of the list
                if !prev.is_null() {
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
        let population = (self.get(nw).population + self.get(ne).population)
            + (self.get(sw).population + self.get(se).population);
        node = self.new_node();
        let n = self.get_mut(node);
        n.nw = nw;
        n.ne = ne;
        n.sw = sw;
        n.se = se;
        n.population = population;
        self.insert(index, node, size_log2);
        node
    }

    fn new_node(&mut self) -> NodeIdx {
        if self.storage_size % CHUNK_SIZE == 0 {
            self.storage
                .push(Box::new(std::array::from_fn(|_| QuadTreeNode::default())));
        }
        self.storage_size += 1;
        NodeIdx::new(
            (self.storage_size - 1)
                .try_into()
                .expect("Nodes storage overflowed u32"),
        )
    }

    /// Insert a node into the hashtable.
    /// index must be hash(node) % buf.len(); node must not be present in the hashtable
    fn insert(&mut self, index: usize, node: NodeIdx, size_log2: u32) {
        if self.size_distribution_history.len() <= size_log2 as usize {
            self.size_distribution_history
                .resize(size_log2 as usize + 1, 0);
        }
        self.size_distribution_history[size_log2 as usize] += 1;

        self.ht_size += 1;
        self.get_mut(node).next = self.hashtable[index];
        self.hashtable[index] = node;
        if self.ht_size as f64 > self.hashtable.len() as f64 * HASHTABLE_MAX_LOAD_FACTOR {
            self.rehash();
        }
    }

    fn rehash(&mut self) {
        let new_size = self.hashtable.len() * 2;
        let mut new_buf = vec![NodeIdx::null(); new_size];
        for i in 0..self.hashtable.len() {
            let mut node = self.hashtable[i];
            while !node.is_null() {
                let n = self.get(node);
                let hash = if n.nw.is_null() {
                    QuadTreeNode::leaf_hash(n.leaf_cells())
                } else {
                    QuadTreeNode::node_hash(n.nw, n.ne, n.sw, n.se)
                };
                let next = n.next;
                let index = hash % new_size;
                self.get_mut(node).next = new_buf[index];
                new_buf[index] = node;
                node = next;
            }
        }
        self.hashtable = new_buf;
    }

    /// Get statistics about the memory manager.
    pub fn stats_fast(&self) -> String {
        let mut s = String::new();

        let mem = self.storage.len() * CHUNK_SIZE * std::mem::size_of::<QuadTreeNode>();
        s.push_str(&format!("memory on nodes: {} MB\n", mem >> 20));

        s.push_str(&format!(
            "memory on hashtable: {} MB\n",
            NiceInt::from_usize((self.hashtable.len() * 4) >> 20)
        ));

        s.push_str(&format!(
            "hashtable elements / buckets: {} / {}\n",
            NiceInt::from_usize(self.ht_size),
            NiceInt::from_usize(self.hashtable.len())
        ));

        s.push_str(&format!(
            "hashtable misses / hits: {} / {}\n",
            NiceInt::from(self.misses),
            NiceInt::from(self.hits)
        ));

        s
    }

    pub fn stats_slow(&self) -> String {
        let mut lengths = vec![];
        for chain in self.hashtable.iter() {
            let mut len = 0;
            let mut node = *chain;
            while !node.is_null() {
                len += 1;
                node = self.get(node).next;
            }
            if len >= lengths.len() {
                lengths.resize(len + 1, 0);
            }
            lengths[len] += 1;
        }

        let sum = lengths.iter().sum::<usize>();
        let mut s = "Chain lengths distribution:\n".to_string();
        for (i, &len) in lengths.iter().enumerate() {
            s.push_str(&format!(
                "{:<2}-{:>3}% ({})\n",
                i,
                len * 100 / sum,
                NiceInt::from_usize(len),
            ));
        }

        s
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}
