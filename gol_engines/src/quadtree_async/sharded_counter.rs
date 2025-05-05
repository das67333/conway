//! A thread-local counter with batched flush to a global atomic.
//! Uses 8-bit local counters for efficiency with fixed threshold of 256.

use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering};

// Enforce singleton: only one ThreadLocalCounter may be instantiated.
static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Global accumulated count flushed from all threads.
static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static LOCAL_COUNT: Cell<u8> = Cell::new(0);
}

/// A thread-safe thread-local counter that flushes to a global atomic
/// when local reaches 256.
pub(super) struct ThreadLocalCounter;

impl ThreadLocalCounter {
    /// Creates the singleton counter with fixed threshold of 256.
    /// Panics if more than one instance is created.
    pub(super) fn new() -> Self {
        let prev = INSTANCE_COUNT.fetch_add(1, Ordering::SeqCst);
        assert!(prev == 0, "ThreadLocalCounter must be a singleton");
        ThreadLocalCounter {}
    }

    /// Increments the thread-local counter and returns
    /// the previous global count only when flushing occurs.
    ///
    /// # Returns
    /// - When a flush occurs: returns the previous global count
    /// - Otherwise: returns 0
    pub(super) fn increment(&self) -> usize {
        let mut result = 0;
        LOCAL_COUNT.with(|cell| {
            let new_value = cell.get().wrapping_add(1);
            cell.set(new_value);

            // If we wrapped around (new_value is 0), flush 256 to global
            if new_value == 0 {
                result = GLOBAL_COUNT.fetch_add(256, Ordering::Relaxed);
            }
        });

        result
    }

    pub(super) fn reset(&self) {
        GLOBAL_COUNT.store(0, Ordering::Relaxed);
    }
}

impl Drop for ThreadLocalCounter {
    fn drop(&mut self) {
        INSTANCE_COUNT.fetch_sub(1, Ordering::SeqCst);
        GLOBAL_COUNT.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn get_approx() -> usize {
        GLOBAL_COUNT.load(Ordering::Relaxed)
    }

    #[test]
    #[serial]
    fn single_thread_increments() {
        let ctr = ThreadLocalCounter::new();
        let initial = get_approx();
        for _ in 0..256 {
            ctr.increment();
        }
        assert_eq!(get_approx(), initial + 256);
    }

    #[test]
    #[serial]
    fn multi_threaded_increments() {
        let ctr = ThreadLocalCounter::new();
        let initial = get_approx();

        std::thread::scope(|s| {
            for _ in 0..4 {
                let c = &ctr;
                s.spawn(move || {
                    for _ in 0..1024 {
                        c.increment();
                    }
                });
            }
        });
        assert_eq!(get_approx(), initial + 4 * 1024);
    }
}
