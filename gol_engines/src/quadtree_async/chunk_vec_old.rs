use super::{FixedVec, NodeIdx, QuadTreeNode};

/// Deque-like structure storing QuadTreeNode elements.
/// It is chosen instead of a vector to avoid reallocation and better utilize memory.
///
/// First element should always be reserved for blank node.
pub struct ChunkVec<const CHUNK_SIZE: usize> {
    chunks: Vec<FixedVec<QuadTreeNode, CHUNK_SIZE>>,
}

impl<const CHUNK_SIZE: usize> ChunkVec<CHUNK_SIZE> {
    pub fn new() -> Self {
        // reserving NodeIdx(0) for blank node
        let node = QuadTreeNode::default();
        node.cache.set(NodeIdx(0)).unwrap();

        // every thread has its own chunk
        let mut chunks = vec![FixedVec::new()];
        unsafe { chunks[0].push(node) };
        Self { chunks }
    }

    /// Allocate memory for a new node and return its NodeIdx.
    pub fn push(&mut self, node: QuadTreeNode) -> NodeIdx {
        let idx = self.chunks.len() - 1;
        let (mut chunk, mut shift) = unsafe { (self.chunks.get_unchecked(idx).weak_ref(), idx * CHUNK_SIZE) };

        // if full, allocate new chunk
        if chunk.len() == CHUNK_SIZE {
            let new_chunk = FixedVec::new();
            chunk = new_chunk.weak_ref();
            self.chunks.push(new_chunk);
            let chunk_idx = {
                self.chunks.len() - 1
            };
            assert!(
                (chunk_idx * CHUNK_SIZE) >> 30 < 3,
                "Close to overflowing u32"
            );
            shift = chunk_idx * CHUNK_SIZE;
        }

        let idx = (shift + chunk.len()) as u32;
        unsafe { chunk.push(node) };
        NodeIdx(idx)
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
                .get(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> std::ops::IndexMut<NodeIdx> for ChunkVec<CHUNK_SIZE> {
    fn index_mut(&mut self, index: NodeIdx) -> &mut Self::Output {
        let i = index.0 as usize;
        unsafe {
            self.chunks
                .get_unchecked_mut(i / CHUNK_SIZE)
                .get_mut(i % CHUNK_SIZE)
        }
    }
}

impl<const CHUNK_SIZE: usize> Default for ChunkVec<CHUNK_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}
