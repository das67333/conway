use criterion::{criterion_group, criterion_main, Criterion};

fn bench_life_fast(c: &mut Criterion) {
    use conway::trait_grid::Grid;

    let (w, h) = (1600, 900);
    eprintln!("Frame size: {w} x {h}");
    let mut life = conway::life_fast::ConwayField::random(w, h, Some(42), 0.3);
    c.bench_function("life_fast", |b| b.iter(|| life.update(1)));
}

criterion_group!(benches, bench_life_fast);
criterion_main!(benches);
