use super::{NodeIdx, QuadTreeNode};

const HASHTABLE_BUF_INITIAL_SIZE: usize = 1;
const HASHTABLE_MAX_LOAD_FACTOR: f64 = 1.2;

/// Hashtable for finding nodes (to avoid duplicates)
pub struct Manager {
    // all allocated nodes
    nodes: Vec<QuadTreeNode>,
    // buffer where heads of linked lists are stored
    hashtable: Vec<NodeIdx>,
    // total number of elements in the hashtable
    ht_size: usize,
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
    pub fn new() -> Self {
        assert!(std::mem::size_of::<usize>() >= 8, "64-bit system required");
        assert!(HASHTABLE_BUF_INITIAL_SIZE.is_power_of_two());
        Self {
            // first node must be reserved for null
            nodes: vec![QuadTreeNode::default()],
            hashtable: vec![NodeIdx::null(); HASHTABLE_BUF_INITIAL_SIZE],
            ht_size: 0,
            hits: 0,
            misses: 0,
        }
    }

    // index must be hash(node) % buf.len(); node must not be present in the hashtable
    fn insert(&mut self, index: usize, node: NodeIdx) {
        self.ht_size += 1;
        self.get_mut(node).next = self.hashtable[index];
        self.hashtable[index] = node;
        if self.ht_size as f64 > self.hashtable.len() as f64 * HASHTABLE_MAX_LOAD_FACTOR {
            let new_size = self.hashtable.len() * 2;
            let mut new_buf = vec![NodeIdx::null(); new_size];
            for i in 0..self.hashtable.len() {
                let mut node = self.hashtable[i];
                while !node.is_null() {
                    let next = self.get(node).next;
                    let hash = if self.get(node).nw.is_null() {
                        QuadTreeNode::leaf_hash(self.get(node).cells())
                    } else {
                        QuadTreeNode::node_hash(
                            self.get(node).nw,
                            self.get(node).ne,
                            self.get(node).sw,
                            self.get(node).se,
                        )
                    };
                    let index = hash % new_size;
                    self.get_mut(node).next = new_buf[index];
                    new_buf[index] = node;
                    node = next;
                }
            }
            self.hashtable = new_buf;
        }
    }

    pub fn get(&self, idx: NodeIdx) -> &QuadTreeNode {
        &self.nodes[idx.get()]
    }

    pub fn get_mut(&mut self, idx: NodeIdx) -> &mut QuadTreeNode {
        &mut self.nodes[idx.get()]
    }

    pub fn find_leaf(&mut self, cells: [u8; 8]) -> NodeIdx {
        let index = QuadTreeNode::leaf_hash(cells) & (self.hashtable.len() - 1);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        while !node.is_null() {
            let next = self.get(node).next;
            if self.get(node).nw.is_null() && self.get(node).cells() == cells {
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
        let n = self.get_mut(node);
        n.nw = NodeIdx::null();
        let cells = u64::from_le_bytes(cells);
        n.ne = NodeIdx::new(cells as u32);
        n.sw = NodeIdx::new((cells >> 32) as u32);
        n.population = cells.count_ones() as f64;
        self.insert(index, node);
        node
    }

    pub fn find_node(&mut self, nw: NodeIdx, ne: NodeIdx, sw: NodeIdx, se: NodeIdx) -> NodeIdx {
        let index = QuadTreeNode::node_hash(nw, ne, sw, se) & (self.hashtable.len() - 1);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        // search for the node in the linked list
        while !node.is_null() {
            let next = self.get(node).next;
            if self.get(node).nw == nw
                && self.get(node).ne == ne
                && self.get(node).sw == sw
                && self.get(node).se == se
            {
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

        let population = (self.get(nw).population + self.get(ne).population)
            + (self.get(sw).population + self.get(se).population);

        node = self.new_node();
        let n = self.get_mut(node);
        n.nw = nw;
        n.ne = ne;
        n.sw = sw;
        n.se = se;
        n.population = population;
        self.insert(index, node);
        node
    }

    pub fn stats(&self, verbose: bool) -> String {
        let mem = self.nodes.capacity() * std::mem::size_of::<QuadTreeNode>();
        let mut s = format!(
            "
memory on nodes: {} MB
memory on hashtable: {} MB
hashtable elements / buckets: {} / {}
hashtable hits: {}
hashtable misses: {}
",
            mem >> 20,
            (self.hashtable.len() * std::mem::size_of::<usize>()) >> 20,
            self.ht_size,
            self.hashtable.len(),
            self.hits,
            self.misses,
        );

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

            for (i, count) in lengths.iter().enumerate() {
                if *count > 0 {
                    s.extend(format!("buckets of size {}: {}\n", i, count).chars());
                }
            }
        }
        s
    }

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
                &(self.get(self.hashtable[idx]).cache) as *const NodeIdx as *const i8,
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

    #[cfg(feature = "prefetch")]
    pub fn find_node_prefetched(&mut self, prefetched: &PrefetchedNode) -> NodeIdx {
        let index = prefetched.hash & (self.hashtable.len() - 1);
        let (nw, ne, sw, se) = (prefetched.nw, prefetched.ne, prefetched.sw, prefetched.se);
        let mut node = self.hashtable[index];
        let mut prev = NodeIdx::null();
        // search for the node in the linked list
        while !node.is_null() {
            let next = self.get(node).next;
            if self.get(node).nw == nw
                && self.get(node).ne == ne
                && self.get(node).sw == sw
                && self.get(node).se == se
            {
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
        let population = (self.get(nw).population + self.get(ne).population)
            + (self.get(sw).population + self.get(se).population);
        node = self.new_node();
        let n = self.get_mut(node);
        n.nw = nw;
        n.ne = ne;
        n.sw = sw;
        n.se = se;
        n.population = population;
        self.insert(index, node);
        node
    }

    fn new_node(&mut self) -> NodeIdx {
        self.nodes.push(QuadTreeNode::default());
        NodeIdx::new(
            (self.nodes.len() - 1)
                .try_into()
                .expect("Nodes storage overflowed u32"),
        )
    }
}
