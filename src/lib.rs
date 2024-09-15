mod gui;
mod pattern_oblivious;
mod quadtree;
mod utils;

pub use gui::{App, Config};
pub use pattern_oblivious::PatternObliviousEngine;
pub use quadtree::{HashLifeEngine, StreamLifeEngine};
pub use utils::{parse_rle, Engine, NiceInt, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};
