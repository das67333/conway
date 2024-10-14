use super::{NodeIdx, QuadTreeNode};
use std::{
    alloc::{alloc_zeroed, dealloc, Layout},
    ops::{Index, IndexMut},
};

pub struct ChunkVec<const CHUNK_SIZE: usize, Meta> {
    chunks: Vec<*mut QuadTreeNode<Meta>>,
    next_free_node: NodeIdx,
    len: usize,
}

impl<const CHUNK_SIZE: usize, Meta: Default> ChunkVec<CHUNK_SIZE, Meta> {
    pub fn new() -> Self {
        let chunk = Self::new_chunk();
        unsafe {
            // reserving NodeIdx(0) for blank node
            (*chunk).has_cache = true;
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

    #[inline]
    pub fn allocate(&mut self) -> NodeIdx {
        if self.next_free_node == NodeIdx(0) {
            let chunk = Self::new_chunk();
            for i in 0..CHUNK_SIZE - 1 {
                let next = self.capacity() + i + 1;
                unsafe { (*chunk.add(i)).next = NodeIdx(next as u32) };
            }
            self.next_free_node = NodeIdx(self.capacity() as u32);
            self.chunks.push(chunk);
        }

        let allocated = self.next_free_node;
        self.next_free_node = self[allocated].next;
        self.len += 1;
        allocated
    }

    pub fn drop_caches(&mut self) {
        for i in (1..self.capacity()).map(|i| NodeIdx(i as u32)) {
            self[i].has_cache = false;
        }
    }

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
            self[idx].has_cache = false;
        }
        self.next_free_node = next_free_node;
        self.len = self.capacity() - 1 - free_nodes_cnt;
    }

    fn new_chunk() -> *mut QuadTreeNode<Meta> {
        let layout = Layout::array::<QuadTreeNode<Meta>>(CHUNK_SIZE).unwrap();
        unsafe { alloc_zeroed(layout) as *mut QuadTreeNode<Meta> }
    }

    pub fn bytes_total(&self) -> usize {
        self.chunks.len() * (size_of::<usize>() + CHUNK_SIZE * size_of::<QuadTreeNode<Meta>>())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.chunks.len() * CHUNK_SIZE
    }
}

impl<const CHUNK_SIZE: usize, Meta> Index<NodeIdx> for ChunkVec<CHUNK_SIZE, Meta> {
    type Output = QuadTreeNode<Meta>;
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

impl<const CHUNK_SIZE: usize, Meta> IndexMut<NodeIdx> for ChunkVec<CHUNK_SIZE, Meta> {
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

impl<const CHUNK_SIZE: usize, Meta> Drop for ChunkVec<CHUNK_SIZE, Meta> {
    fn drop(&mut self) {
        let layout = Layout::array::<QuadTreeNode<Meta>>(CHUNK_SIZE).unwrap();
        for ptr in self.chunks.iter().copied() {
            unsafe {
                dealloc(ptr as *mut u8, layout);
            }
        }
    }
}
