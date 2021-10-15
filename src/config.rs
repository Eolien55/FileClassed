use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Config {
    pub dest : String,
    pub dirs : HashSet<String>,
    pub once : bool,
    pub sleep :usize,
    pub codes : HashMap<String, String>,
}

pub type DeclaredType = [bool ; 6];

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub dest :Option<String>,
    pub dirs :Option<HashSet<String>>,
    pub once :Option<bool>,
    pub sleep :Option<usize>,
    pub codes :Option<HashMap<String, String>>,
}

macro_rules! home_dir {
    ($dir:expr) => (
        format!("{}{}{}", home_dir().unwrap().to_str().unwrap(), path::MAIN_SEPARATOR, $dir)
    );
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
            _ => 6,
        }
    };
}

/*










BUILDING CONFIG FROM COMMAND-LINE










*/

use dirs::{config_dir, home_dir};
use structopt::StructOpt;
use std::error::Error;
use std::path;

fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error>>
where
    T: std::str::FromStr,
    T::Err: Error + 'static,
    U: std::str::FromStr,
    U::Err: Error + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{}`", s))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "FileClassed",
    about = "Efficient yet customizable file organizer",
    author = "Elie Le Vaillant <elielevaillant2007@gmail.com>",
    setting = structopt::clap::AppSettings::ColoredHelp
)]

struct Cli {
    /// Sets the configurationg file
    #[structopt(short="-C", long, value_name = "file")]
    config : Option<String>,

    /// Overrides the watching directories
    #[structopt(short, long, value_name = "directory")]
    dir : Option<Vec<String>>,

    /// Overrides destination directory
    #[structopt(short="-D", long, value_name = "directory")]
    dest : Option<String>,

    /// Makes the program loop only once
    #[structopt(short, long)]
    once : bool,

    /// Sets the how much milliseconds the program should sleep between each loop
    #[structopt(short, long, value_name = "milliseconds")]
    sleep : Option<usize>,

    /// Shortcuts, ie meanings
    #[structopt(
        short = "-c",
        long = "--code",
        value_name = "shortcut=meaning", 
        parse(try_from_str = parse_key_val)
    )]
    codes : Option<Vec<(String, String)>>,

    /// Makes the program verbose
    #[structopt(flatten)]
    verbose : clap_verbosity_flag::Verbosity,
}

pub fn get_verbose() -> Option<log::Level> {
    let mut verbose = Cli::from_args().verbose;
    verbose.set_default(Some(log::Level::Warn));

    return verbose.log_level();
}

fn get_args() -> (Config, String, DeclaredType) {
    // Processing Options
    let args = Cli::from_args();
    let mut declared : DeclaredType = [false, false, false, false, false, false];

    let config : String;
    if !args.config.is_none() {
        config = args.config.unwrap();
        declared[which_declared!("config")] = true;
    } else {
        config = format!(
            "{}{}{}", config_dir().unwrap().to_str().unwrap(), path::MAIN_SEPARATOR, "fcs.yml"
        );
    }

    let dest : String;
    let mut dirs : Vec<String>;
    let once : bool;
    let sleep : usize;
    let codes : HashMap<String, String>;

    if !args.dir.is_none() {
        dirs = args.dir.unwrap();
        declared[which_declared!("dirs")] = true;
    } else {
        dirs = vec![
            home_dir!("Scolaire"),
            home_dir!("usb")
        ]
    }

    if !args.dest.is_none() {
        dest = args.dest.unwrap();
        declared[which_declared!("dest")] = true;
    } else {
        dest = home_dir!("Scolaire");
    }

    once = args.once;
    declared[which_declared!("once")] = once;

    if !args.sleep.is_none() {
        sleep = args.sleep.unwrap();
        declared[which_declared!("sleep")] = true;
    } else {
        sleep = 1000;
    }

    if !args.codes.is_none() {
        codes = args.codes.unwrap().iter()
                                   .map(|tuple| (tuple.0.to_owned(), tuple.1.to_owned()))
                                   .collect();
        declared[which_declared!("codes")] = true;
    } else {
        codes = [
            ("chin", "Chinois"),
            ("en", "Anglais"),
            ("eps", "EPS"),
            ("fr", "Français"),
            ("glb", "Global"),
            ("gr", "Grec"),
            ("hg", "Histoire-Géographie"),
            ("info", "Informatique"),
            ("mt", "Mathématiques"),
            ("pc", "Physique-Chimie"),
            ("ses", "Sciences Économiques et Sociales"),
            ("svt", "SVT"),
            ("vdc", "Vie de Classe")].iter()
                                     .map(|tuple| (tuple.0.to_string(), tuple.1.to_string()))
                                     .collect();        
    }

    dirs.shrink_to_fit();
    let dirs : HashSet<String> = dirs.into_iter().collect();

    return (Config {dest, dirs, once, sleep, codes}, config, declared);
}

/*










BUILDING CONFIG FROM COMMAND-LINE










*/

use std::fs;

macro_rules! replace_value {
    ($conf1:expr, $conf2:expr, $attr:expr, $declared:expr) => {
        if !$declared[which_declared!($attr)] && !$conf2.is_none() {
            $conf1 = $conf2.unwrap();
        }
    };
}

fn exists(the_path : &String) -> bool {
    return path::Path::new(the_path.as_str()).exists();
}

// Get config from CLI args and config file
pub fn get_config_args() -> (Config, DeclaredType) {
    log::trace!("Getting arguments from CLI");
    let (mut config, mut config_file, declared) = get_args();

    let mut default_configs = vec![
        format!(
            "{}{}{}", config_dir().unwrap().to_str().unwrap(), path::MAIN_SEPARATOR, "fcs.yml"
        ),
        format!(
            "{}{}{}{}{}", config_dir().unwrap().to_str().unwrap(), path::MAIN_SEPARATOR, "fcs",
            path::MAIN_SEPARATOR, "init.yml"
        )
    ];

    while !exists(&config_file) && !declared[which_declared!("config")] && !default_configs.is_empty() {
        config_file = default_configs.pop().unwrap();
    }

    log::trace!("Reading \"{}\" for config", config_file);

    match fs::read_to_string(&config_file) {
        Ok(reading_file) => {
            match serde_yaml::from_str::<ConfigSerDe>(&reading_file) {
                Ok(from_file) => {
                    replace_value!(config.dirs, from_file.dirs, "dirs", declared);
                    replace_value!(config.dest, from_file.dest, "dest", declared);
                    replace_value!(config.once, from_file.once, "once", declared);
                    replace_value!(config.sleep, from_file.sleep, "sleep", declared);
                    replace_value!(config.codes, from_file.codes, "codes", declared);
                    (config, declared)
                },
                Err(e) => {
                    log::error!("Error happenned while parsing config file \"{}\". Falling back to defaults", e.to_string());
                    (config, declared)
                },
            }
        },
        Err(_) => {
            log::error!("Config file \"{}\" doesn't exist or isn't valid UTF-8. Falling back to defaults", config_file);
            (config, declared)
        }
    }
}

// Here, we return our cleaned config
// ie, without non-existing directories,
// The bool value indicates if the config is so messed
// up that it is unusable, and if the program should exit
pub fn clean(mut config : Config) -> (Config, bool) {
    let mut fatal = false;

    config.dest = String::from(shellexpand::env(&config.dest).unwrap());
    config.dirs = config.dirs.iter()
                             .map(|dir| (String::from(shellexpand::env(&dir).unwrap())))
                             .collect();
    
    let existing_dirs : HashSet<String> = 
        config.dirs.iter()
                   .filter(|&dir| exists(&dir))
                   .map(|dir| String::from(dir))
                   .collect();
    
    let non_existing_dirs = config.dirs.difference(&existing_dirs);
    for dir in non_existing_dirs {
        log::warn!("Watching directory \"{}\" doesn't exist. Not using it", dir);
    }

    config.dirs = existing_dirs;
    if config.dirs.is_empty() {
       log::error!("No directories set up, or none of them exist ! Exiting");
       fatal = true;
    }

    if !exists(&config.dest) {
        log::error!("Destination \"{}\" doesn't exist ! Exiting", config.dest);
        fatal = true;
    }
    
    return (config, fatal);
}