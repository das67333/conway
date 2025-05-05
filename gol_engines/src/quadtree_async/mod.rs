mod blank;
mod hashlife;
mod memory;
mod node;
mod sharded_counter;

const LEAF_SIZE: u64 = 8;
const LEAF_SIZE_LOG2: u32 = LEAF_SIZE.ilog2();

use blank::BlankNodes;
use memory::MemoryManager;
use node::{NodeIdx, QuadTreeNode};
use sharded_counter::ThreadLocalCounter;

pub use hashlife::HashLifeEngineAsync;
