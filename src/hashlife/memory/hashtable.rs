use super::{manager::NodesManager, QuadTreeNode};

const HASHTABLE_BUF_INITIAL_SIZE: usize = 1;
const HASHTABLE_MAX_LOAD_FACTOR: f64 = 1.2;

/// Hashtable for finding nodes (to avoid duplicates)
pub struct HashTable {
    // buffer where heads of linked lists are stored
    buf: Vec<*mut QuadTreeNode>,
    // number of elements in the hashtable
    size: usize,
    // how many times elements were found
    hits: u64,
    // how many times elements were inserted
    misses: u64,
    // storage for nodes
    mem_manager: NodesManager,
}

#[cfg(feature = "prefetch")]
pub struct PrefetchedNode {
    pub nw: *mut QuadTreeNode,
    pub ne: *mut QuadTreeNode,
    pub sw: *mut QuadTreeNode,
    pub se: *mut QuadTreeNode,
    pub hash: usize,
}

impl HashTable {
    pub fn new() -> Self {
        assert!(HASHTABLE_BUF_INITIAL_SIZE.is_power_of_two());
        Self {
            buf: vec![std::ptr::null_mut(); HASHTABLE_BUF_INITIAL_SIZE],
            size: 0,
            hits: 0,
            misses: 0,
            mem_manager: NodesManager::new(),
        }
    }

    // index must be hash(node) % buf.len()
    unsafe fn insert(&mut self, index: usize, node: *mut QuadTreeNode) {
        self.size += 1;
        (*node).next = self.buf[index];
        self.buf[index] = node;
        if self.size as f64 > self.buf.len() as f64 * HASHTABLE_MAX_LOAD_FACTOR {
            let new_size = self.buf.len() * 2;
            let mut new_buf = vec![std::ptr::null_mut(); new_size];
            for i in 0..self.buf.len() {
                let mut node = self.buf[i];
                while !node.is_null() {
                    let next = (*node).next;
                    let hash = if (*node).nw.is_null() {
                        QuadTreeNode::leaf_hash((*node).ne as u64)
                    } else {
                        QuadTreeNode::node_hash((*node).nw, (*node).ne, (*node).sw, (*node).se)
                    };
                    let index = hash % new_size;
                    (*node).next = new_buf[index];
                    new_buf[index] = node;
                    node = next;
                }
            }
            self.buf = new_buf;
        }
    }

    pub fn find_leaf(&mut self, cells: u64) -> *mut QuadTreeNode {
        let index = QuadTreeNode::leaf_hash(cells) & (self.buf.len() - 1);
        let mut node = self.buf[index];
        let mut prev: *mut QuadTreeNode = std::ptr::null_mut();
        while !node.is_null() {
            let next = unsafe { (*node).next };
            if unsafe { (*node).nw.is_null() && (*node).ne as u64 == cells } {
                // move the node to the front of the list
                if !prev.is_null() {
                    unsafe {
                        (*prev).next = (*node).next;
                        (*node).next = self.buf[index]
                    };
                    self.buf[index] = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = next;
        }
        self.misses += 1;
        node = self.mem_manager.new_node();
        unsafe {
            (*node).nw = std::ptr::null_mut();
            (*node).ne = cells as *mut QuadTreeNode;
            (*node).population = cells.count_ones() as f64;
            self.insert(index, node);
            node
        }
    }

    pub fn find_node(
        &mut self,
        nw: *mut QuadTreeNode,
        ne: *mut QuadTreeNode,
        sw: *mut QuadTreeNode,
        se: *mut QuadTreeNode,
    ) -> *mut QuadTreeNode {
        let index = QuadTreeNode::node_hash(nw, ne, sw, se) & (self.buf.len() - 1);
        let mut node = self.buf[index];
        let mut prev: *mut QuadTreeNode = std::ptr::null_mut();
        // search for the node in the linked list
        while !node.is_null() {
            let next = unsafe { (*node).next };
            if unsafe {
                (*node).nw == nw && (*node).ne == ne && (*node).sw == sw && (*node).se == se
            } {
                // move the node to the front of the list
                if !prev.is_null() {
                    unsafe {
                        (*prev).next = (*node).next;
                        (*node).next = self.buf[index]
                    };
                    self.buf[index] = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = next;
        }
        self.misses += 1;
        node = self.mem_manager.new_node();
        unsafe {
            (*node).nw = nw;
            (*node).ne = ne;
            (*node).sw = sw;
            (*node).se = se;
            (*node).population =
                ((*nw).population + (*ne).population) + ((*sw).population + (*se).population);
            self.insert(index, node);
            node
        }
    }

    pub fn stats(&self, verbose: bool) -> String {
        let mut s = format!(
            "{}
memory on hashtable: {} MB
hashtable elements / buckets: {} / {}
hashtable hits: {}
hashtable misses: {}
",
            self.mem_manager.stats(),
            (self.buf.len() * std::mem::size_of::<usize>()) >> 20,
            self.size,
            self.buf.len(),
            self.hits,
            self.misses,
        );

        if verbose {
            let mut lengths = vec![];
            for chain in self.buf.iter() {
                let mut len = 0;
                let mut node = *chain;
                while !node.is_null() {
                    len += 1;
                    node = unsafe { (*node).next };
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
        nw: *mut QuadTreeNode,
        ne: *mut QuadTreeNode,
        sw: *mut QuadTreeNode,
        se: *mut QuadTreeNode,
    ) -> PrefetchedNode {
        let hash = QuadTreeNode::node_hash(nw, ne, sw, se);
        let idx = hash & (self.buf.len() - 1);
        unsafe {
            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                &self.buf[idx] as *const *mut QuadTreeNode as *const i8,
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
    pub fn find_node_prefetched(&mut self, prefetched: &PrefetchedNode) -> *mut QuadTreeNode {
        let index = prefetched.hash & (self.buf.len() - 1);
        let (nw, ne, sw, se) = (prefetched.nw, prefetched.ne, prefetched.sw, prefetched.se);
        let mut node = self.buf[index];
        let mut prev: *mut QuadTreeNode = std::ptr::null_mut();
        // search for the node in the linked list
        while !node.is_null() {
            let next = unsafe { (*node).next };
            if unsafe {
                (*node).nw == nw && (*node).ne == ne && (*node).sw == sw && (*node).se == se
            } {
                // move the node to the front of the list
                if !prev.is_null() {
                    unsafe {
                        (*prev).next = (*node).next;
                        (*node).next = self.buf[index]
                    };
                    self.buf[index] = node;
                }
                self.hits += 1;
                return node;
            }
            prev = node;
            node = next;
        }
        self.misses += 1;
        node = self.mem_manager.new_node();
        unsafe {
            (*node).nw = nw;
            (*node).ne = ne;
            (*node).sw = sw;
            (*node).se = se;
            (*node).population =
                ((*nw).population + (*ne).population) + ((*sw).population + (*se).population);
            self.insert(index, node);
            node
        }
    }
}
