use conway::CellularAutomaton;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_life_naive(c: &mut Criterion) {
    const N: usize = 1 << 12;
    let mut life = conway::life_naive::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_naive", |b| b.iter(|| life.update(1)));
}

// bench_life_hash is meaningless because it quickly reaches stable configuration and memorizes it

fn bench_life_simd(c: &mut Criterion) {
    const N: usize = 1 << 15;
    let mut life = conway::life_simd::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_simd", |b| b.iter(|| life.update(1)));
}

fn bench_life_shader(c: &mut Criterion) {
    const N: usize = 1 << 15;
    let mut life = conway::life_shader::ConwayField::blank(N, N);
    life.randomize(Some(42), 0.3);
    c.bench_function("life_shader", |b| b.iter(|| life.update(1)));
}

fn bench_dev(c: &mut Criterion) {
    c.bench_function("temp", |b| b.iter(|| ()));
}

criterion_group!(
    benches,
    bench_life_naive,
    bench_life_simd,
    bench_life_shader,
    bench_dev,
);
criterion_main!(benches);
