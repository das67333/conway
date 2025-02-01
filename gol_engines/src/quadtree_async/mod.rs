mod chunk_vec;
mod fixed_vec;
mod hashlife;
mod memory;
mod node;
mod population;

const LEAF_SIZE: u64 = 8;
const LEAF_SIZE_LOG2: u32 = LEAF_SIZE.ilog2();

pub use chunk_vec::ChunkVec;
use fixed_vec::FixedVec;
use memory::MemoryManager;
pub use node::{NodeIdx, QuadTreeNode};
use population::PopulationManager;

pub use hashlife::HashLifeEngineAsync;
