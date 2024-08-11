use super::{MemoryManager, NodeIdx};
use std::collections::HashMap;

pub struct PopulationManager {
    cache: HashMap<NodeIdx, f64>,
}

impl PopulationManager {
    pub fn new() -> Self {
        PopulationManager {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, node: NodeIdx, mem: &MemoryManager) -> f64 {
        if let Some(val) = self.cache.get(&node) {
            *val
        } else {
            let n = mem.get(node);
            if n.is_leaf() {
                return (n.nw.get().count_ones() + n.ne.get().count_ones()) as f64;
            }
            let population = self.get(n.nw, mem)
                + self.get(n.ne, mem)
                + self.get(n.sw, mem)
                + self.get(n.se, mem);
            self.cache.insert(node, population);
            population
        }
    }
}
