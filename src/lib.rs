mod gui;
pub mod hashlife; // pub for benchmarks in separate binaries
mod pattern_oblivious;
mod utils;

pub use gui::{App, Config};
pub use hashlife::HashLifeEngine;
pub use pattern_oblivious::PatternObliviousEngine;
pub use utils::{parse_rle, Engine, NiceInt, Topology, MAX_SIDE_LOG2, MIN_SIDE_LOG2};
