use std::sync::Mutex;

use super::{NodeIdx, QuadTreeNode};

mod thread_id {
    use std::{
        cell::Cell,
        sync::atomic::{AtomicU32, Ordering},
    };

    static NEXT_ID: AtomicU32 = AtomicU32::new(0);

    thread_local! {
        static THREAD_ID: Cell<Option<u32>> = const { Cell::new(None) };
    }

    pub fn get() -> usize {
        THREAD_ID.with(|id| {
            if let Some(x) = id.get() {
                x as usize
            } else {
                let x = NEXT_ID.fetch_add(1, Ordering::Relaxed);
                id.set(Some(x));
                x as usize
            }
        })
    }

    pub fn reset_next_id() {
        NEXT_ID.store(0, Ordering::Relaxed);
    }
}

const MAX_THREADS: usize = 8;

/// Deque-like structure storing QuadTreeNode elements.
/// It is chosen instead of a vector to avoid reallocation and better utilize memory.
///
/// First element should always be reserved for blank node.
pub struct ChunkVec<const CHUNK_SIZE: usize> {
    chunks: Vec<Vec<QuadTreeNode>>,
    lock: Mutex<()>,
    /// Index of the chunk
    index_by_thread: [u32; MAX_THREADS],
}

impl<const CHUNK_SIZE: usize> ChunkVec<CHUNK_SIZE> {
    pub fn new() -> Self {
        // reserving NodeIdx(0) for blank node
        let node = QuadTreeNode::default();
        node.cache.set(NodeIdx(0)).unwrap();
        let mut chunks = (0..MAX_THREADS)
            .map(|_| Vec::with_capacity(CHUNK_SIZE))
            .collect::<Vec<_>>();
        chunks[0].push(node);
        thread_id::reset_next_id();
        Self {
            chunks,
            lock: Mutex::new(()),
            index_by_thread: std::array::from_fn(|i| i as u32),
        }
    }

    /// Allocate memory for a new node and return its NodeIdx.
    pub fn push(&mut self, node: QuadTreeNode) -> NodeIdx {
        let thread_id = thread_id::get();
        let mut idx = self.index_by_thread[thread_id];
        if self.chunks[idx as usize].len() == CHUNK_SIZE {
            let new_chunk = Vec::with_capacity(CHUNK_SIZE);
            let _lock = self.lock.lock().unwrap();
            idx = self.chunks.len() as u32;
            self.chunks.push(new_chunk);
            drop(_lock);
            assert!(
                (idx as usize * CHUNK_SIZE) >> 30 != 3,
                "Close to overflowing u32"
            );
            self.index_by_thread[thread_id] = idx;
        }

        let chunk = &mut self.chunks[idx as usize];
        chunk.push(node);
        NodeIdx((idx as usize * CHUNK_SIZE + chunk.len() - 1) as u32)
    }

    /// Deallocate every unmarked node and leave all nodes unmarked.
    pub fn deallocate_unmarked_and_unmark(&mut self) {
        // let mut next_free_node = NodeIdx(0);
        // let mut free_nodes_cnt = 0;
        // for idx in (1..self.capacity()).rev().map(|i| NodeIdx(i as u32)) {
        //     if self[idx].gc_marked {
        //         self[idx].gc_marked = false;
        //     } else {
        //         self[idx].next = next_free_node;
        //         next_free_node = idx;
        //         free_nodes_cnt += 1;
        //     }
        //     self[idx].cache = OnceLock::new();
        // }
        // self.next_free_node = next_free_node;
        // self.len = self.capacity() - 1 - free_nodes_cnt;
    }

    pub fn bytes_total(&self) -> usize {
        self.chunks.len() * CHUNK_SIZE * size_of::<QuadTreeNode>()
    }

    pub fn len(&self) -> usize {
        (self.chunks.len() - 1) * CHUNK_SIZE + self.chunks.last().unwrap().len()
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
            self.chunks
                .get_unchecked(i / CHUNK_SIZE)
                .get_unchecked(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> std::ops::IndexMut<NodeIdx> for ChunkVec<CHUNK_SIZE> {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        let i = index.0 as usize;
        unsafe {
            self.chunks
                .get_unchecked_mut(i / CHUNK_SIZE)
                .get_unchecked_mut(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> Default for ChunkVec<CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
