use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    sync::OnceLock,
};

use super::{NodeIdx, QuadTreeNode};

/// Deque-like structure storing QuadTreeNode elements.
/// It is chosen instead of a vector to avoid reallocation statters.
///
/// First element should always be reserved for blank node.
pub struct ChunkVec<const CHUNK_SIZE: usize> {
    chunks: Vec<*mut QuadTreeNode>,
    next_free_node: NodeIdx,
    len: usize,
}

impl<const CHUNK_SIZE: usize> ChunkVec<CHUNK_SIZE> {
    pub fn new() -> Self {
        let chunk = Self::new_chunk();
        unsafe {
            // reserving NodeIdx(0) for blank node
            (*chunk).cache.set(NodeIdx(0)).unwrap();
            for i in 1..CHUNK_SIZE - 1 {
                (*chunk.add(i)).next = NodeIdx(i as u32 + 1);
            }
        };
        Self {
            chunks: vec![chunk],
            next_free_node: NodeIdx(1),
            len: 1,
        }
    }

    /// Allocate memory for a new node and return its NodeIdx.
    pub fn allocate(&mut self) -> NodeIdx {
        if self.next_free_node == NodeIdx(0) {
            let chunk = Self::new_chunk();
            for i in 0..CHUNK_SIZE - 1 {
                let next = NodeIdx((self.capacity() + i + 1) as u32);
                unsafe { (*chunk.add(i)).next = next };
            }
            self.next_free_node = NodeIdx(self.capacity() as u32);
            self.chunks.push(chunk);
        }

        let allocated = self.next_free_node;
        assert!(allocated.0 >> 30 != 3, "Close to overflowing u32");
        self.next_free_node = self[allocated].next;
        self.len += 1;
        allocated
    }

    /// Deallocate every unmarked node and leave all nodes unmarked.
    pub fn deallocate_unmarked_and_unmark(&mut self) {
        let mut next_free_node = NodeIdx(0);
        let mut free_nodes_cnt = 0;
        for idx in (1..self.capacity()).rev().map(|i| NodeIdx(i as u32)) {
            if self[idx].gc_marked {
                self[idx].gc_marked = false;
            } else {
                self[idx].next = next_free_node;
                next_free_node = idx;
                free_nodes_cnt += 1;
            }
            self[idx].cache = OnceLock::new();
        }
        self.next_free_node = next_free_node;
        self.len = self.capacity() - 1 - free_nodes_cnt;
    }

    fn new_chunk() -> *mut QuadTreeNode {
        let layout = Layout::array::<QuadTreeNode>(CHUNK_SIZE).unwrap();
        unsafe { alloc_zeroed(layout) as *mut QuadTreeNode }
    }

    pub fn bytes_total(&self) -> usize {
        self.chunks.len() * CHUNK_SIZE * size_of::<QuadTreeNode>()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.chunks.len() * CHUNK_SIZE
    }
}

impl<const CHUNK_SIZE: usize> std::ops::Index<NodeIdx> for ChunkVec<CHUNK_SIZE> {
    type Output = QuadTreeNode;
    fn index(&self, index: NodeIdx) -> &Self::Output {
        let i = index.0 as usize;
        unsafe {
            &*self
                .chunks
                .get_unchecked(i / CHUNK_SIZE)
                .add(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> std::ops::IndexMut<NodeIdx> for ChunkVec<CHUNK_SIZE> {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        let i = index.0 as usize;
        unsafe {
            &mut *self
                .chunks
                .get_unchecked_mut(i / CHUNK_SIZE)
                .add(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> Drop for ChunkVec<CHUNK_SIZE> {
    fn drop(&mut self) {
        let layout = Layout::array::<QuadTreeNode>(CHUNK_SIZE).unwrap();
        for ptr in self.chunks.iter().copied() {
            unsafe { dealloc(ptr as *mut u8, layout) }
        }
    }
}
