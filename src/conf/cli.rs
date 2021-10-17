use dirs::config_dir;
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path;

use super::lib;

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
    #[structopt(short = "-C", long, value_name = "file")]
    config: Option<String>,

    /// Overrides the watching directories
    #[structopt(short, long, value_name = "directory")]
    dir: Option<Vec<String>>,

    /// Overrides destination directory
    #[structopt(short = "-D", long, value_name = "directory")]
    dest: Option<String>,

    /// Makes the program loop only once
    #[structopt(short, long)]
    once: bool,

    /// Sets the how much milliseconds the program should sleep between each loop
    #[structopt(short, long, value_name = "milliseconds")]
    sleep: Option<usize>,

    /// Shortcuts, ie meanings
    #[structopt(
        short = "-c",
        long = "--code",
        value_name = "shortcut=meaning", 
        parse(try_from_str = parse_key_val)
    )]
    codes: Option<Vec<(String, String)>>,

    /// Activates time info, ie including months and years in the path
    #[structopt(short, long)]
    timeinfo: bool,

    /// Makes the program verbose
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

pub fn get_verbose() -> Option<log::Level> {
    let mut verbose = Cli::from_args().verbose;
    verbose.set_default(Some(log::Level::Warn));

    return verbose.log_level();
}

pub fn get_args() -> (lib::Config, String, lib::DeclaredType) {
    // Processing Options
    let args = Cli::from_args();
    let mut declared: lib::DeclaredType = [false, false, false, false, false, false, false];

    let config: String;
    if !args.config.is_none() {
        config = args.config.unwrap();
        declared[lib::which_declared!("config")] = true;
    } else {
        config = format!(
            "{}{}{}",
            config_dir().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
            "fcs.yml"
        );
    }

    let dest: String;
    let mut dirs: Vec<String>;
    let once: bool;
    let sleep: usize;
    let codes: HashMap<String, String>;
    let timeinfo: bool;

    if !args.dir.is_none() {
        dirs = args.dir.unwrap();
        declared[lib::which_declared!("dirs")] = true;
    } else {
        dirs = vec![lib::home_dir!("Scolaire"), lib::home_dir!("usb")]
    }

    if !args.dest.is_none() {
        dest = args.dest.unwrap();
        declared[lib::which_declared!("dest")] = true;
    } else {
        dest = lib::home_dir!("Scolaire");
    }

    once = args.once;
    declared[lib::which_declared!("once")] = once;

    if !args.sleep.is_none() {
        sleep = args.sleep.unwrap();
        declared[lib::which_declared!("sleep")] = true;
    } else {
        sleep = 1000;
    }

    if !args.codes.is_none() {
        codes = args
            .codes
            .unwrap()
            .iter()
            .map(|tuple| (tuple.0.to_owned(), tuple.1.to_owned()))
            .collect();
        declared[lib::which_declared!("codes")] = true;
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
            ("vdc", "Vie de Classe"),
        ]
        .iter()
        .map(|tuple| (tuple.0.to_string(), tuple.1.to_string()))
        .collect();
    }

    timeinfo = args.timeinfo;
    declared[lib::which_declared!("timeinfo")] = timeinfo;

    dirs.shrink_to_fit();
    let dirs: HashSet<String> = dirs.into_iter().collect();

    return (
        lib::Config {
            dest,
            dirs,
            once,
            sleep,
            codes,
            timeinfo,
        },
        config,
        declared,
    );
}
