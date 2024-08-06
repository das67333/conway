use super::{NodeIdx, QuadTreeNode};

// const HASHTABLE_MAX_LOAD_FACTOR: f64 = 0.7;

/// Hashtable for finding nodes (to avoid duplicates)
pub struct Manager {
    // all allocated nodes
    storage: Vec<QuadTreeNode>,
    // buffer where heads of linked lists are stored
    hashtable: Vec<NodeIdx>,
    // total number of elements in the hashtable
    pub ht_size: usize,
    // how many times elements were found in the hashtable
    hits: u64,
    // how many times elements were inserted into the hashtable
    misses: u64,
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
    /// Create a new memory manager with a given capacity.
    /// 
    /// `cap` must be a power of two!
    #[must_use]
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap.is_power_of_two(), "Capacity must be a power of two");
        Self {
            // first node must be reserved for null
            storage: vec![QuadTreeNode::default(); cap],
            hashtable: vec![NodeIdx::null(); cap],
            ht_size: 1,
            hits: 0,
            misses: 0,
        }
    }

    #[inline]
    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        &self.storage[idx.get()]
    }

    #[inline]
    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        &mut self.storage[idx.get()]
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
        }
        self.insert(index, node);
        node
    }

    #[inline]
    /// Find a node with the given parts.
    /// If the node is not found, it is created.
    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
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

    /// Get statistics about the memory manager.
    pub fn stats(&self, verbose: bool) -> String {
        let mut s = String::new();

        let mem = self.storage.len() * std::mem::size_of::<QuadTreeNode>();
        s.push_str(&format!("memory on nodes: {} MB\n", mem >> 20));

        s.push_str(&format!(
            "memory on hashtable: {} MB\n",
            (self.hashtable.len() * 4) >> 20
        ));

        s.push_str(&format!(
            "hashtable elements / buckets: {} / {}\n",
            self.ht_size,
            self.hashtable.len()
        ));

        s.push_str(&format!(
            "hashtable misses / hits: {} / {}\n",
            self.misses, self.hits
        ));

        if verbose {
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
            s.push_str("Chain lengths distribution:\n");
            s.push_str(&format!(
                "0-{}%  1-{}%  2-{}%  >2-{}%\n",
                lengths[0] * 100 / sum,
                lengths[1] * 100 / sum,
                lengths[2] * 100 / sum,
                lengths[3..].iter().sum::<usize>() * 100 / sum
            ));
        }
        s
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
    pub fn find_node_prefetched(&mut self, prefetched: &PrefetchedNode) -> NodeIdx {
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
        node = self.new_node();
        let n = self.get_mut(node);
        n.nw = nw;
        n.ne = ne;
        n.sw = sw;
        n.se = se;
        self.insert(index, node);
        node
    }

    fn new_node(&mut self) -> NodeIdx {
        self.ht_size += 1;
        assert!(self.ht_size <= self.storage.len(), "Node storage overflow, realloc disabled");
        NodeIdx::new(
            (self.ht_size - 1)
                .try_into()
                .expect("Nodes storage overflowed u32"),
        )
    }

    /// Insert a node into the hashtable.
    /// index must be hash(node) % buf.len(); node must not be present in the hashtable
    fn insert(&mut self, index: usize, node: NodeIdx) {
        self.get_mut(node).next = self.hashtable[index];
        self.hashtable[index] = node;
        // if self.ht_size as f64 > self.hashtable.len() as f64 * HASHTABLE_MAX_LOAD_FACTOR {
        //     self.rehash();
        // }
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
}
