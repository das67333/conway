#![warn(clippy::all, clippy::cargo)]

mod quadtree;
mod quadtree_async;
mod simd;
mod utils;

pub use quadtree::{HashLifeEngine, StreamLifeEngine};
pub use quadtree_async::HashLifeEngineAsync;
pub use quadtree_async::{ChunkVec, NodeIdx, QuadTreeNode, thread_id};
pub use simd::SimdEngine;
pub use utils::{parse_rle, AsyncEngine, Engine, NiceInt, Topology};

pub type DefaultEngine = StreamLifeEngine;

pub const MIN_SIDE_LOG2: u32 = 7;
pub const MAX_SIDE_LOG2: u32 = 62;
