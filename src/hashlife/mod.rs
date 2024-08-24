mod engine;
mod memory;
mod node;
mod population;

pub const LEAF_SIZE: u64 = 8;

pub use memory::{MemoryManager, PrefetchedNode};
pub use node::{NodeIdx, QuadTreeNode};
pub use population::PopulationManager;

pub use engine::HashLifeEngine;
