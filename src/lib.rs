mod gui;
mod quadtree;
mod simd;
mod utils;

pub use gui::{App, Config};
pub use quadtree::{HashLifeEngine, StreamLifeEngine};
pub use simd::SimdEngine;
pub use utils::{parse_rle, Engine, NiceInt, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};

pub type DefaultEngine = StreamLifeEngine;
