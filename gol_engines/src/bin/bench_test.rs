use std::cell::UnsafeCell;

use gol_engines::{ChunkVec, QuadTreeNode};

fn main() {
    let n = 100_000_000;

    // let mut cv = ChunkVec::<8192>::new();
    // let timer = std::time::Instant::now();
    // for _ in 0..n {
    //     cv.push(QuadTreeNode::default());
    // }
    // println!("Time: {:?}", timer.elapsed());

    struct Helper(UnsafeCell<ChunkVec<CHUNK_SIZE>>);
    unsafe impl Send for Helper {}
    unsafe impl Sync for Helper {}
    impl Helper {
        fn new() -> Self {
            Self(UnsafeCell::new(ChunkVec::new()))
        }
        fn push(&self, node: QuadTreeNode) {
            let cv = unsafe { &mut *self.0.get() };
            cv.push(node);
        }
    }

    const CHUNK_SIZE: usize = 1 << 13;
    for k in 8..=8 {
        let cv = Helper::new();
        let timer = std::time::Instant::now();
        std::thread::scope(|s| {
            for _ in 0..k {
                s.spawn(|| {
                    for _ in 0..(n / k) {
                        cv.push(QuadTreeNode::default());
                    }
                });
            }
        });
        println!("Time on k={}: {:?}", k, timer.elapsed());
    }
}
