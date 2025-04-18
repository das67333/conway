#![warn(clippy::all, clippy::cargo)]

mod quadtree;
mod quadtree_async;
mod simd;
mod utils;

pub use quadtree::{HashLifeEngine, StreamLifeEngine};
pub use quadtree_async::HashLifeEngineAsync;
pub use quadtree_async::{NodeIdx, QuadTreeNode};
pub use simd::SimdEngine;
pub use utils::{parse_rle, GoLEngine, NiceInt, Topology};

pub type DefaultEngine = StreamLifeEngine;

pub const MIN_SIDE_LOG2: u32 = 7;
pub const MAX_SIDE_LOG2: u32 = 62;
