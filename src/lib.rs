mod gui;
mod lifes;

pub use lifes::{
    ca_trait::CellularAutomaton, hash::ConwayFieldHash, naive::ConwayFieldNaive,
    shader::ConwayFieldShader, simd1::ConwayFieldSimd1, simd2::ConwayFieldSimd2,
};

pub use gui::navigation::InteractionManager;
