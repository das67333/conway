use gol_engines::*;

fn detect_format(filename: &str) -> Option<PatternFormat> {
    if filename.ends_with(".rle") {
        Some(PatternFormat::RLE)
    } else if filename.ends_with(".mc") {
        Some(PatternFormat::Macrocell)
    } else if filename.ends_with(".mc.gz") {
        Some(PatternFormat::CompressedMacrocell)
    } else {
        None
    }
}

fn main() {
    set_memory_manager_cap_log2(28);

    let paths = std::fs::read_dir("/home/das/Downloads/very_large_patterns").unwrap();
    for (i, path) in paths.enumerate() {
        let path = path.unwrap().path();
        let name = path.file_name().unwrap().to_str().unwrap();
        println!("i={}\t{}", i, name);
        let format = detect_format(name).unwrap();
        let data = std::fs::read(path).unwrap();
        let pattern = Pattern::from_format(format, &data).unwrap();

        // let timer = std::time::Instant::now();
        // let mut engine = HashLifeEngineAsync::from_pattern(&pattern, Topology::Unbounded).unwrap();
        // let elapsed_build = timer.elapsed();
        // println!("build time: {:?}", elapsed_build);
        // engine.update(10);
        // let elapsed_update = timer.elapsed() - elapsed_build;
        // println!("update time: {:?}", elapsed_update);
    }
}
