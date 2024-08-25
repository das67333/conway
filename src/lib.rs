mod gui;
mod pattern_oblivious;
pub mod quadtree; // pub for benchmarks in separate binaries
mod utils;

pub use gui::{App, Config};
pub use pattern_oblivious::PatternObliviousEngine;
pub use quadtree::HashLifeEngine;
pub use utils::{parse_rle, Engine, NiceInt, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};
