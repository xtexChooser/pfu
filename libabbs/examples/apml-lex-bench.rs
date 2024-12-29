use std::{
    env::{self},
    fs,
    path::Path,
    time::Instant,
};

fn collect_apml(path: &Path, result: &mut Vec<String>) {
    for entry in path.read_dir().unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "spec" || entry.file_name() == "defines" {
            result.push(fs::read_to_string(entry.path()).unwrap());
        } else if entry.file_type().unwrap().is_dir() {
            collect_apml(&entry.path(), result);
        }
    }
}

fn main() {
    let tree = env::var("TREE").expect("TREE env var must be set");
    let mut srcs = Vec::new();
    collect_apml(Path::new(&tree), &mut srcs);

    let start = Instant::now();
    for _ in 0..10 {
        for src in &srcs {
            let _ = libabbs::apml::tree::ApmlParseTree::parse(&src).unwrap();
        }
    }
    let elapsed = start.elapsed();
    println!("analysed {} files in {:?}", srcs.len(), elapsed);
}