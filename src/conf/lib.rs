use serde::{Deserialize, Serialize};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::defaults;


#[derive(Clone, Debug)]
pub struct Config {
    pub once: bool,
    pub sleep: usize,
    pub timeinfo: bool,
    pub static_mode: bool,
    pub dest: PathBuf,
    pub dirs: HashSet<PathBuf>,
    pub codes: HashMap<String, String>,
    pub separator: char,
    pub filename_separators: usize,
    pub begin_var: char,
    pub end_var: char,
}

impl Config {
    #[inline]
    pub fn default() -> Self {
        defaults::get_default()
    }
}

#[derive(Clone, Debug)]
pub struct BuildConfig {
    pub once: bool,
    pub sleep: Option<usize>,
    pub timeinfo: bool,
    pub static_mode: bool,
    pub dirs: Option<Vec<PathBuf>>,
    pub dest: Option<PathBuf>,
    pub codes: Option<Vec<(String, String)>>,
    pub separator: Option<char>,
    pub filename_separators: Option<usize>,
    pub begin_var: Option<char>,
    pub end_var: Option<char>,
}

impl BuildConfig {
    #[inline]
    pub fn default() -> Self {
        defaults::get_build_default()
    }
}

pub type DeclaredType = [bool; 12];

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub separator: Option<char>,
    pub filename_separators: Option<usize>,
    pub begin_var: Option<char>,
    pub end_var: Option<char>,
    pub once: Option<bool>,
    pub timeinfo: Option<bool>,
    pub static_mode: Option<bool>,
    pub sleep: Option<usize>,
    pub codes: Option<HashMap<String, String>>,
    pub dest: Option<PathBuf>,
    pub dirs: Option<HashSet<PathBuf>>,
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
            "separator" => 8,
            "filename_separators" => 9,
            _ => 8,
        }
    };
}

pub(crate) use test_path;
pub(crate) use which_declared;
