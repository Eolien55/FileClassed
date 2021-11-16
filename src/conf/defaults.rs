use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;

use super::lib;

pub fn get_default() -> lib::Config {
    let default: lib::Config = lib::Config {
        dirs: HashSet::new(),
        dest: PathBuf::from_str("").unwrap(),
        sleep: 1000,
        codes: HashMap::new(),
        timeinfo: false,
        once: false,
        static_mode: false,
        separator: '.',
        filename_separators: 1,
    };

    default
}

pub fn get_build_default() -> lib::BuildConfig {
    let default: lib::Config = get_default();

    let build_default: lib::BuildConfig = lib::BuildConfig {
        dirs: Some(default.dirs.iter().cloned().collect()),
        dest: Some(default.dest),
        sleep: Some(default.sleep),
        codes: Some(
            default
                .codes
                .iter()
                .map(|tuple| (tuple.0.to_owned(), tuple.1.to_owned()))
                .collect(),
        ),
        timeinfo: default.timeinfo,
        once: default.once,
        static_mode: default.static_mode,
        separator: Some(default.separator),
        filename_separators: Some(default.filename_separators),
    };

    build_default
}
