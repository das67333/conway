use super::{MemoryManager, NodeIdx};
use std::collections::HashMap;

#[derive(Default)]
pub struct PopulationManager {
    cache: HashMap<NodeIdx, f64>,
}

impl PopulationManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&mut self, node: NodeIdx, mem: &MemoryManager) -> f64 {
        if let Some(val) = self.cache.get(&node) {
            *val
        } else {
            let n = mem.get(node);
            let population = if n.is_leaf() {
                (n.nw.0.count_ones() + n.ne.0.count_ones()) as f64
            } else {
                self.get(n.nw, mem)
                    + self.get(n.ne, mem)
                    + self.get(n.sw, mem)
                    + self.get(n.se, mem)
            };
            self.cache.insert(node, population);
            population
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}
