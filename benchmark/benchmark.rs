use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fcs::run;
use std::collections::HashMap;
use std::path;
use std::time;
use std::str::FromStr;

pub fn bench_get_new_name(c: &mut Criterion) {
    let timestamp = time::SystemTime::now();

    let keys = "abcdefghijklmnopqrstuvwxyz".to_string();
    let codes: HashMap<String, String> = keys
        .chars()
        .map(|key| (String::from(key), format!("{}zerty", key)))
        .collect();

    let dest_str = "/home/default/Documents";
    let dest = path::PathBuf::from_str(dest_str).unwrap();

    let name: Vec<_> = keys.chars().map(String::from).collect();
    let mut name = name.join(".");
    name += ".ex.txt";
    name = format!("{}/{}", dest_str, name);
    let name = path::Path::new(&name);

    c.bench_function("get_new_name", |b| {
        b.iter(|| {
            run::get_new_name(
                black_box(&name),
                black_box(&dest),
                black_box(&codes),
                black_box(Some(timestamp)),
                black_box(false),
            )
        })
    });
}

criterion_group!(benches, bench_get_new_name);
criterion_main!(benches);
