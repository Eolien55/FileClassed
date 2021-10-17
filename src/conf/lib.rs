use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::path;

pub use dirs::home_dir as home_directory;

#[derive(Debug)]
pub struct Config {
    pub dest: String,
    pub dirs: HashSet<String>,
    pub once: bool,
    pub sleep: usize,
    pub codes: HashMap<String, String>,
    pub timeinfo: bool,
}

pub type DeclaredType = [bool; 7];

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub dest: Option<String>,
    pub dirs: Option<HashSet<String>>,
    pub once: Option<bool>,
    pub sleep: Option<usize>,
    pub codes: Option<HashMap<String, String>>,
    pub timeinfo: Option<bool>,
}

pub fn exists(the_path: &String) -> bool {
    return path::Path::new(the_path.as_str()).exists();
}

macro_rules! home_dir {
    ($dir:expr) => {
        format!(
            "{}{}{}",
            lib::home_directory().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
            $dir
        )
    };
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
            _ => 7,
        }
    };
}

pub(crate) use home_dir;
pub(crate) use which_declared;
