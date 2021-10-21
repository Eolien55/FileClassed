use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::path;

#[derive(Debug)]
pub struct Config {
    pub dest: String,
    pub dirs: HashSet<String>,
    pub once: bool,
    pub sleep: usize,
    pub codes: HashMap<String, String>,
    pub timeinfo: bool,
    pub static_mode: bool,
}

pub type DeclaredType = [bool; 8];

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub dest: Option<String>,
    pub dirs: Option<HashSet<String>>,
    pub once: Option<bool>,
    pub timeinfo: Option<bool>,
    pub static_mode: Option<bool>,
    pub sleep: Option<usize>,
    pub codes: Option<HashMap<String, String>>,
}

pub fn exists(the_path: &String) -> bool {
    return path::Path::new(the_path.as_str()).exists();
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

pub(crate) use which_declared;
