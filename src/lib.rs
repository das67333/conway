mod lifes;

pub use lifes::{
    ca_trait::CellularAutomaton, hash_256x256::ConwayFieldHash256, hash_4x4::ConwayFieldHash,
    naive::ConwayFieldNaive, simd1::ConwayFieldSimd1, simd2::ConwayFieldSimd2,
};

// pub use gui::navigation::InteractionManager;