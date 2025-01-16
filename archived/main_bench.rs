use conway::*;
use criterion::{criterion_group, criterion_main, Criterion};

const SEED: u64 = 42;
const FILL_RATE: f64 = 0.5;

fn bench_life_update<Life: CellularAutomaton, const N: usize, const K: usize>(c: &mut Criterion) {
    let mut life = Life::blank(N, N);
    let id = format!("{}_update_{}_{}", Life::id(), N, K);
    life.randomize(Some(SEED), FILL_RATE);
    c.bench_function(&id, |b| b.iter(|| life.update(K)));
}

criterion_group!(
    benches,
    bench_life_update::<ConwayFieldNaive, 1024, 1>,
    bench_life_update::<ConwayFieldSimd1, 2048, 1>,
    bench_life_update::<ConwayFieldSimd1, 4096, 1>,
    bench_life_update::<ConwayFieldSimd2, 2048, 1>,
    bench_life_update::<ConwayFieldSimd2, 4096, 1>,
    // bench_life_update::<ConwayFieldShader, 4096, 1>,
    // bench_life_update::<ConwayFieldShader, 4096, 16>,
);
criterion_main!(benches);
