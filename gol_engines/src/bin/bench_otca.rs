use gol_engines::{DefaultEngine, Engine, Topology};

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

    let mut engine = DefaultEngine::from_recursive_otca_metapixel(otca_depth, top_pattern);
    println!("Time on building field: {:?}", timer.elapsed());

    let timer = std::time::Instant::now();
    engine.update(engine.side_length_log2() - 1, Topology::Unbounded);
    println!("Time on big update: {:?}", timer.elapsed());
}
