use super::QuadTreeNode;
use std::collections::LinkedList;

const CHUNK_SIZE: usize = (1 << 20) / std::mem::size_of::<QuadTreeNode>();

// impl QuadTreeNode {
//     pub fn nw() {}
//     pub fn ne() {}
//     pub fn sw() {}
//     pub fn se() {}
// }

// struct GcGuard {
//     // nodes whose subtrees are protected from garbage collection
//     gc_protected: *mut Vec<*const QuadTreeNode>,
//     // size of `gc_protected` before the guard was created
//     sp: usize,
// }
// impl GcGuard {
//     pub fn new(gc_protected: &mut Vec<*const QuadTreeNode>) -> Self {
//         let sp = gc_protected.len();
//         Self { gc_protected, sp }
//     }
// }
// impl Drop for GcGuard {
//     fn drop(&mut self) {
//         unsafe {
//             (*self.gc_protected).truncate(self.sp);
//         }
//     }
// }

pub struct NodesManager {
    // linked list of all allocated chunks
    allocated_chunks: LinkedList<[QuadTreeNode; CHUNK_SIZE]>,
    // linked list of all free nodes
    free_nodes: *mut QuadTreeNode,
    // counter used for periodic garbage collection
    new_node_count: u64,
}

impl NodesManager {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<usize>() >= 8, "64-bit system required");
        Self {
            allocated_chunks: LinkedList::new(),
            free_nodes: std::ptr::null_mut(),
            new_node_count: 0,
        }
    }

    pub fn new_node(&mut self) -> *mut QuadTreeNode {
        self.new_node_count += 1;
        if self.free_nodes.is_null() {
            // allocate a new chunk
            let arr = std::array::from_fn(|_| QuadTreeNode::default());
            self.allocated_chunks.push_back(arr);
            // link all nodes in the new chunk into the free list
            let arr = self.allocated_chunks.back_mut().unwrap();
            for i in 0..CHUNK_SIZE - 1 {
                arr[i].next = &mut arr[i + 1] as *mut QuadTreeNode;
            }
            self.free_nodes = &mut arr[0] as *mut QuadTreeNode;
        }
        // pop a node from the free list
        let node = self.free_nodes;
        unsafe { self.free_nodes = (*node).next };
        node
    }

    pub fn stats(&self) -> String {
        let mem = self.allocated_chunks.len() * CHUNK_SIZE * std::mem::size_of::<QuadTreeNode>();
        format!("memory on nodes: {} MB", mem >> 20)
    }
}
