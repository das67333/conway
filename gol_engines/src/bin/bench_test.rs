use std::cell::UnsafeCell;

use gol_engines::{thread_id, ChunkVec, QuadTreeNode};

fn main() {
    let n = 100_000_000;

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

    // struct Helper(Mutex<ChunkVec<CHUNK_SIZE>>);
    // unsafe impl Send for Helper {}
    // unsafe impl Sync for Helper {}
    // impl Helper {
    //     fn new() -> Self {
    //         Self(Mutex::new(ChunkVec::new()))
    //     }
    //     fn push(&self, node: QuadTreeNode) {
    //         let cv = &mut *self.0.lock().unwrap();
    //         cv.push(node);
    //     }
    // }

    const CHUNK_SIZE: usize = 1 << 12;

    let mut baseline = None;
    for k in 1..=16 {
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
        let elapsed = timer.elapsed();
        let mpps = n as f64 / elapsed.as_secs_f64() * 1e-6;
        if baseline.is_none() {
            baseline.replace(mpps);
        }
        thread_id::reset_next_id();
        println!(
            "k={}: {:.2} Mpps, {:.0}%",
            k,
            mpps,
            100.0 * mpps / baseline.unwrap()
        );
    }
}
