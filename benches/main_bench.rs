use conway::CellularAutomaton;
use criterion::{criterion_group, criterion_main, Criterion};

const N: usize = 1024;
const K: usize = 16;

fn bench_life_naive(c: &mut Criterion) {
    let mut life = conway::life_naive::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_naive", |b| b.iter(|| life.update(K)));
}

// bench_life_hash is meaningless because it quickly reaches stable configuration and memorizes it

fn bench_life_simd(c: &mut Criterion) {
    let mut life = conway::life_simd::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_simd", |b| b.iter(|| life.update(K)));
}

fn bench_life_shader(c: &mut Criterion) {
    let mut life = conway::life_shader::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_shader", |b| b.iter(|| life.update(K)));
}

criterion_group!(
    benches,
    bench_life_naive,
    bench_life_simd,
    bench_life_shader
);
criterion_main!(benches);
