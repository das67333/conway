use gol_engines::*;

fn main() {
    let otca_depth = 2;
    let top_pattern = vec![
        vec![0, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 1, 0, 0, 0, 0, 0],
        vec![1, 1, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let timer = std::time::Instant::now();

    let mut engine = HashLifeEngineAsync::from_recursive_otca_metapixel(otca_depth, top_pattern);
    println!("Time on building field: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    let steps_log2 = 23;
    engine.update(steps_log2, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());
    assert_eq!(engine.hash(), 0xf35ef0ba0c9db279);
    assert_eq!(engine.population(), 6_094_494_746_384.0);
}
