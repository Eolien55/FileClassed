use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fcs::run;
use std::collections::HashMap;
use std::path;
use std::str::FromStr;
use std::time;

pub fn bench_get_new_name(c: &mut Criterion) {
    let timestamp = time::SystemTime::now();

    let keys = "a".to_string();
    let codes: HashMap<String, String> = keys
        .chars()
        .map(|key| (String::from(key), format!("{}zerty", key)))
        .collect();

    let dest_str = "/home/default/Documents";
    let dest = path::PathBuf::from_str(dest_str).unwrap();

    let name: Vec<_> = keys.chars().map(String::from).collect();
    let mut name = name.join(".");
    name += "..0";
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

pub fn bench_expand(c : &mut Criterion) {
    let keys = "a".to_string();
    let codes: HashMap<String, String> = keys
        .chars()
        .map(|key| (String::from(key), format!("{}zerty", key)))
        .collect();
    
    let name : Vec<_> = keys.chars().map(String::from).collect();
    let name = format!("{{{}}}", name.join("}{"));

    c.bench_function("expand", |b| {
        b.iter(|| {
            run::expand(black_box(&name), black_box(&codes))
        })
    });
}

criterion_group!(benches, bench_get_new_name, bench_expand);
criterion_main!(benches);
