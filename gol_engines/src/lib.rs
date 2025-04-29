#![warn(clippy::all, clippy::cargo)]

mod pattern;
// mod quadtree;
mod config;
mod quadtree_async;
mod simd;
mod topology;
mod traits;
mod utils;

pub use pattern::{Pattern, PatternFormat, PatternNode};
pub use topology::Topology;
pub use traits::GoLEngine;

pub use config::{get_config, set_memory_manager_cap_log2};
// pub use quadtree::{HashLifeEngine, StreamLifeEngine};
pub use quadtree_async::HashLifeEngineAsync;
pub use simd::SIMDEngine;
pub use utils::NiceInt;

pub type DefaultEngine = SIMDEngine;

pub const VERSION: &str = "1.0";
