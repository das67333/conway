use criterion::{criterion_group, criterion_main, Criterion};

const N: usize = 1_000_000;

fn hash_std(c: &mut Criterion) {
    c.bench_function("std::hash", |b| {
        b.iter(|| {
            let mut m = std::collections::HashMap::new();
            for i in 0..N {
                m.insert(i, i);
            }
        })
    });
}

fn ahash(c: &mut Criterion) {
    c.bench_function("ahash", |b| {
        b.iter(|| {
            let mut m = ahash::AHashMap::new();
            for i in 0..N {
                m.insert(i, i);
            }
        })
    });
}

fn lru<const K: usize>(c: &mut Criterion) {
    use std::num::NonZeroUsize;

    c.bench_function(&format!("lru_{}", K), |b| {
        b.iter(|| {
            let mut m = lru::LruCache::new(NonZeroUsize::new(K).unwrap());
            for i in 0..N {
                m.put(i, i);
            }
        })
    });
}

fn hashlink_lru<const K: usize>(c: &mut Criterion) {
    c.bench_function(&format!("hashlink_lru_{}", K), |b| {
        b.iter(|| {
            let mut m = hashlink::LruCache::new(K);
            for i in 0..N {
                m.insert(i, i);
            }
        })
    });
}

criterion_group!(benches, hash_std, ahash, lru<1>, lru<1_000>, lru<N>, hashlink_lru<1>, hashlink_lru<1_000>, hashlink_lru<N>);
criterion_main!(benches);
