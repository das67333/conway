use conway::CellularAutomaton;
use criterion::{criterion_group, criterion_main, Criterion};

const SEED: u64 = 42;
const FILL_RATE: f64 = 0.5;

fn bench_life_naive_update_1024_16(c: &mut Criterion) {
    let mut life = conway::life_naive::ConwayField::blank(1024, 1024);
    life.randomize(Some(SEED), FILL_RATE);
    c.bench_function("life_naive_update_1024_16", |b| b.iter(|| life.update(8)));
}

fn bench_life_simd1_update_4096_16(c: &mut Criterion) {
    let mut life = conway::life_simd1::ConwayField::blank(4096, 4096);
    life.randomize(Some(SEED), FILL_RATE);
    c.bench_function("life_simd1_update_4096_16", |b| b.iter(|| life.update(16)));
}

fn bench_life_simd2_update_4096_16(c: &mut Criterion) {
    let mut life = conway::life_simd2::ConwayField::blank(4096, 4096);
    life.randomize(Some(SEED), FILL_RATE);
    c.bench_function("life_simd2_update_4096_16", |b| b.iter(|| life.update(16)));
}

fn bench_life_shader_update_4096_16(c: &mut Criterion) {
    let mut life = conway::life_shader::ConwayField::blank(4096, 4096);
    life.randomize(Some(SEED), FILL_RATE);
    c.bench_function("life_shader_update_4096_16", |b| b.iter(|| life.update(16)));
}

criterion_group!(
    benches,
    bench_life_naive_update_1024_16,
    bench_life_simd1_update_4096_16,
    bench_life_simd2_update_4096_16,
    bench_life_shader_update_4096_16,
);
criterion_main!(benches);
