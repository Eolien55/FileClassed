use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fcs::run;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path;
use std::str::FromStr;
use std::time;

pub fn bench_get_new_name(c: &mut Criterion) {
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

    c.bench_function("get_new_name", |b| {
        b.iter(|| {
            run::get_new_name(
                black_box(&name),
                black_box(&dest),
                black_box(&codes),
                black_box(None),
                (black_box('.'), black_box(1)),
                (black_box('{'), black_box('}')),
                black_box(','),
            )
        })
    });
}

pub fn bench_expand(c: &mut Criterion) {
    let keys = "a".to_string();
    let codes: HashMap<String, String> = keys
        .chars()
        .map(|key| (String::from(key), format!("{}zerty", key)))
        .collect();

    let name: Vec<_> = keys.chars().map(String::from).collect();
    let name = format!("{{{}}}", name.join("}{"));

    c.bench_function("expand", |b| {
        b.iter(|| {
            run::expand(
                black_box(&name),
                black_box(&codes),
                black_box('{'),
                black_box('}'),
                black_box(None),
                black_box(&mut vec![]),
                black_box(','),
            )
        })
    });
}

pub fn bench_brackets(c: &mut Criterion) {
    let string = "{{}";

    c.bench_function("brackets", |b| {
        b.iter(|| {
            run::find_first_valid_opening_bracket(black_box(string), black_box('{'), black_box('}'))
        })
    });
}

fn get_new_name_without_errors(
    name: &str,
    dest: &path::Path,
    codes: &HashMap<String, String>,
    timestamp: Option<time::SystemTime>,
    separator: (char, usize),
    var: (char, char),
    last_token: char,
) {
    run::get_new_name(name, dest, codes, timestamp, separator, var, last_token).ok();
}

pub fn bench_all(c: &mut Criterion) {
    let destination = path::PathBuf::from("/home/some_user/Documents/");
    let codes: HashMap<String, String> = vec![
        ("pc", "Physique-Chimie"),
        ("gr", "Grec"),
        ("fr", "Français"),
        ("svt", "SVT"),
        ("chin", "Chinois"),
        ("glb", "Global"),
        ("ses", "Sciences Économiques et Sociales"),
        ("info", "Informatique"),
        ("eps", "EPS"),
        ("vdc", "Vie de Classe"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    let mut files: Vec<path::PathBuf> = Vec::new();

    for key in codes.keys() {
        for number in 0..10 {
            for other_number1 in 0..10 {
                for other_number2 in 0..10 {
                    for other_number3 in 0..10 {
                        files.push(path::PathBuf::from(format!(
                            "{}.{}{}{}{}.txt",
                            key, number, other_number1, other_number2, other_number3
                        )));
                    }
                }
            }
        }
    }

    c.bench_function(format!("testing for {} files", files.len()).as_str(), |b| {
        b.iter(|| {
            let _: () = files
                .par_iter()
                .map(|file_name| {
                    get_new_name_without_errors(
                        black_box(file_name.to_str().unwrap()),
                        black_box(&destination),
                        black_box(&codes),
                        black_box(Some(time::SystemTime::now())),
                        black_box(('.', 1)),
                        black_box(('{', '}')),
                        black_box(','),
                    )
                })
                .collect();
        })
    });
}

criterion_group!(
    benches,
    bench_get_new_name,
    bench_expand,
    bench_brackets,
    bench_all,
);
criterion_main!(benches);
