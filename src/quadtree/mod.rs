mod deque;
mod hashlife;
mod memory;
mod node;
mod population;
// mod streamlife;

const LEAF_SIZE: u64 = 8;
const LEAF_SIZE_LOG2: u32 = LEAF_SIZE.ilog2();

use deque::Deque;
use memory::{MemoryManager, PrefetchedNode};
use node::{NodeIdx, QuadTreeNode};
use population::PopulationManager;

pub use hashlife::HashLifeEngine;