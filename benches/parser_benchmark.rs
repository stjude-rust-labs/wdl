use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;
use wdl::parser::parse_document;

fn parse_large_file(c: &mut Criterion) {
    let large_file_path = Path::new("benches/fixtures/large_workflow.wdl");
    let content = fs::read_to_string(large_file_path).unwrap();
    
    c.bench_function("parse_large_file", |b| {
        b.iter(|| {
            let _ = parse_document(black_box(&content));
        })
    });
}

fn parse_many_small_files(c: &mut Criterion) {
    let files = [
        "benches/fixtures/small_workflow1.wdl",
        "benches/fixtures/small_workflow2.wdl",
        "benches/fixtures/small_workflow3.wdl",
        // ... more files ...
    ];
    
    let contents: Vec<String> = files
        .iter()
        .map(|path| fs::read_to_string(Path::new(path)).unwrap())
        .collect();
    
    c.bench_function("parse_many_small_files", |b| {
        b.iter(|| {
            for content in &contents {
                let _ = parse_document(black_box(content));
            }
        })
    });
}

criterion_group!(benches, parse_large_file, parse_many_small_files);
criterion_main!(benches);