#![feature(portable_simd)]

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_life_fast_stable(c: &mut Criterion) {
    use conway::trait_grid::Grid;

    let (w, h) = (1600, 900);
    eprintln!("Frame size: {w} x {h}");

    let mut life = conway::life_fast_stable::ConwayField::random(w, h, Some(42), 0.3);
    c.bench_function("life_fast_stable", |b| b.iter(|| life.update(1)));
}

fn bench_life_fast_unstable(c: &mut Criterion) {
    use conway::trait_grid::Grid;

    let (w, h) = (1600, 900);
    eprintln!("Frame size: {w} x {h}");

    let mut life = conway::life_fast_unstable::ConwayField::random(w, h, Some(42), 0.3);
    c.bench_function("life_fast_unstable", |b| b.iter(|| life.update(1)));
}

criterion_group!(benches, bench_life_fast_stable, bench_life_fast_unstable);
criterion_main!(benches);
