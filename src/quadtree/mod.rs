mod hashlife;
mod memory;
mod node;
mod population;
// mod streamlife;

pub const LEAF_SIZE: u64 = 8;

pub use memory::{MemoryManager, PrefetchedNode};
pub use node::{NodeIdx, QuadTreeNode};
pub use population::PopulationManager;

pub use hashlife::HashLifeEngine;
