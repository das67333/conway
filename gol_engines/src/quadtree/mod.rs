mod chunk_vec;
mod hashlife;
mod memory;
mod node;
mod population;
mod streamlife;

const LEAF_SIDE: u64 = 8;
const LEAF_SIDE_LOG2: u32 = LEAF_SIDE.ilog2();

use chunk_vec::ChunkVec;
use memory::{MemoryManager, PrefetchedNode};
use node::{NodeIdx, QuadTreeNode};
use population::PopulationManager;

pub type HashLifeEngine = hashlife::HashLifeEngine<()>;
pub use streamlife::StreamLifeEngine;
