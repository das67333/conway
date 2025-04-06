use super::{MemoryManager, NodeIdx, LEAF_SIZE_LOG2};

pub struct BlankNodes {
    data: Vec<NodeIdx>,
}

impl BlankNodes {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn get(&mut self, size_log2: u32, mem: &MemoryManager) -> NodeIdx {
        let i = (size_log2 - LEAF_SIZE_LOG2) as usize;
        while self.data.len() <= i {
            if let Some(&b) = self.data.last() {
                self.data.push(mem.find_or_create_node(b, b, b, b));
            } else {
                self.data.push(mem.find_or_create_leaf_from_u64(0));
            };
        }
        self.data[i]
    }
}
