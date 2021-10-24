use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct Config {
    pub once: bool,
    pub sleep: usize,
    pub timeinfo: bool,
    pub static_mode: bool,
    pub dest: String,
    pub dirs: HashSet<String>,
    pub codes: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct BuildConfig {
    pub once: bool,
    pub sleep: Option<usize>,
    pub timeinfo: bool,
    pub static_mode: bool,
    pub dirs: Option<Vec<String>>,
    pub dest: Option<String>,
    pub codes: Option<Vec<(String, String)>>,
}

pub type DeclaredType = [bool; 8];

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub once: Option<bool>,
    pub timeinfo: Option<bool>,
    pub static_mode: Option<bool>,
    pub sleep: Option<usize>,
    pub codes: Option<HashMap<String, String>>,
    pub dest: Option<String>,
    pub dirs: Option<HashSet<String>>,
}

macro_rules! test_path {
    ($the_path:expr, $arg:expr) => {{
        let path = std::path::Path::new($the_path);
        let mut result = path.exists();

        match $arg {
            "file" => result = result && path.is_file(),
            "dir" => result = result && path.is_dir(),
            _ => (),
        };

        result
    }};
}

macro_rules! which_declared {
    ($val:expr) => {
        match $val {
            "dirs" => 0,
            "dest" => 1,
            "once" => 2,
            "sleep" => 3,
            "codes" => 4,
            "config" => 5,
            "timeinfo" => 6,
            "static_mode" => 7,
            _ => 8,
        }
    };
}

pub(crate) use test_path;
pub(crate) use which_declared;
