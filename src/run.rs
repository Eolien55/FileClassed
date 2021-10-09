use std::collections::{HashMap};
use std::fs;
use std::path;

fn handle(name: String, dest: String, codes: &HashMap<String, String>, month_codes: &HashMap<String, String>, verbose: bool) {
    if !path::Path::new(&name).exists() {
        return;
    }

    let timestamp = fs::metadata(name).unwrap().created();
}