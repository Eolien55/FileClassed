use dirs_next::config_dir;
use structopt::clap::Shell;
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::path;
use std::process::exit;
use std::str::FromStr;

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

    /// Generates completion script for specified shell and writing it on stdout
    #[structopt(long, value_name = "SHELL")]
    completion: Option<String>,
}

pub fn get_verbose() -> Option<log::Level> {
    let mut verbose = Cli::from_args().verbose;
    verbose.set_default(Some(log::Level::Warn));

    return verbose.log_level();
}

macro_rules! define {
    ($args_entry:expr, $val:expr, $fallback:expr, $name:expr, $var:expr, $declared:expr) => {
        if !$args_entry.is_none() {
            $var = $val;
            $declared[lib::which_declared!($name)] = true;
        } else {
            $var = $fallback;
        }
    };

    ($args_entry:expr, $fallback:expr, $name:expr, $var:expr, $declared:expr) => {
        if !$args_entry.is_none() {
            $var = $args_entry.unwrap();
            $declared[lib::which_declared!($name)] = true;
        } else {
            $var = $fallback;
        }
    };

    ($args_entry:expr, $name:expr, $var:expr, $declared:expr) => {
        $var = $args_entry;
        $declared[lib::which_declared!($name)] = $var;
    };
}

pub fn get_args() -> (lib::Config, String, lib::DeclaredType) {
    // Processing Options
    let args = Cli::from_args();
    let mut declared: lib::DeclaredType = [false, false, false, false, false, false, false];

    if !args.completion.is_none() {
        match Shell::from_str(&args.completion.unwrap()) {
            Ok(shell) => {
                let mut app = Cli::clap();
                app.gen_completions_to("fcs", shell, &mut io::stdout());
                exit(exitcode::OK);
            }
            Err(e) => {
                log::error!("{}", e.to_string());
                exit(exitcode::DATAERR);
            }
        }
    }

    let config: String;
    define!(
        args.config,
        format!(
            "{}{}{}",
            config_dir().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
            "fcs.yml"
        ),
        "config",
        config,
        declared
    );

    let dest: String;
    let mut dirs: Vec<String>;
    let once: bool;
    let sleep: usize;
    let codes: HashMap<String, String>;
    let timeinfo: bool;

    define!(
        args.dir,
        vec!["~/Scolaire", "~/usb"]
            .iter()
            .map(|x| x.to_string())
            .collect(),
        "dirs",
        dirs,
        declared
    );

    define!(args.dest, "~/Scolaire".to_string(), "dest", dest, declared);

    define!(args.once, "once", once, declared);

    define!(args.sleep, 1000, "sleep", sleep, declared);

    define!(
        args.codes,
        args.codes
            .unwrap()
            .iter()
            .map(|tuple| (tuple.0.to_owned(), tuple.1.to_owned()))
            .collect(),
        [
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
        .collect(),
        "codes",
        codes,
        declared
    );

    define!(args.timeinfo, "timeinfo", timeinfo, declared);

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
