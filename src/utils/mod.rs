mod parse_rle;
mod topology;
mod traits;

pub use parse_rle::parse_rle;
pub use topology::Topology;
pub use traits::{Engine, MAX_SIDE_LOG2, MIN_SIDE_LOG2};
