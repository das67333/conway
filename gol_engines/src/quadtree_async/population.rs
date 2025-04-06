use super::{MemoryManager, NodeIdx, LEAF_SIZE_LOG2};
use ahash::AHashMap as HashMap;

/// Calculates population of a node and caches the result.
#[derive(Default)]
pub struct PopulationManager {
    cache: HashMap<NodeIdx, f64>,
}

impl PopulationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, idx: NodeIdx, size_log2: u32, mem: &MemoryManager) -> f64 {
        if let Some(val) = self.cache.get(&idx) {
            *val
        } else {
            let n = mem.get(idx);
            let population = if size_log2 == LEAF_SIZE_LOG2 {
                u64::from_le_bytes(n.leaf_cells()).count_ones() as f64
            } else {
                self.get(n.nw, size_log2 - 1, mem)
                    + self.get(n.ne, size_log2 - 1, mem)
                    + self.get(n.sw, size_log2 - 1, mem)
                    + self.get(n.se, size_log2 - 1, mem)
            };
            self.cache.insert(idx, population);
            population
        }
    }

    pub fn bytes_total(&self) -> usize {
        self.cache.capacity() * size_of::<(NodeIdx, f64)>()
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
