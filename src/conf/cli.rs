use dirs_next::config_dir;
use structopt::clap::Shell;
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::path;
use std::process::exit;
use std::str::FromStr;

use super::defaults::get_build_default;
use super::lib;
use super::super::main_config::clean as clean_custom;

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
    name = "fcs",
    about = "Efficient yet customizable file organizer",
    author = "Elie Le Vaillant <elielevaillant2007@gmail.com>",
    setting = structopt::clap::AppSettings::ColoredHelp
)]

struct Cli {
    /// Sets the configurationg file
    #[structopt(short = "-C", long, value_name = "file")]
    config: Option<String>,

    /// Overrides the watching directories
    #[structopt(short, long = "dir", value_name = "directory")]
    dirs: Option<Vec<String>>,

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
    #[structopt(long, value_name = "shell")]
    completion: Option<String>,

    /// Runs in static mode, ie not reloading configuration file on changes
    #[structopt(short = "-S", long = "--static")]
    static_mode: bool,

    /// Generates configuration file from CLI arguments
    ///
    /// Note that the generation isn't deterministic, which means codes won't be ordered the same way.
    /// Note also that the generated config file isn't pretty printed, ie it's quite ugly
    #[structopt(short, long)]
    generate_config: bool,
}

pub fn get_verbose() -> Option<log::Level> {
    let mut verbose = Cli::from_args().verbose;
    verbose.set_default(Some(log::Level::Warn));

    verbose.log_level()
}

macro_rules! define_option {
    ($args:expr, $config:expr, $field:ident, $declared:expr, $default:expr) => {
        $config.$field = match $args.$field {
            Some(value) => {
                $declared[lib::which_declared!(quote::quote!($field).to_string().as_str())] = true;
                Some(value)
            }
            None => $default.$field,
        }
    };

    ($args:expr, $config:expr, $declared:expr, $default:expr, $($field:ident),+) => {
        ($(define_option!($args, $config, $field, $declared, $default),)+)
    }
}

macro_rules! define_bool {
    ($args:expr, $config:expr, $field:ident, $declared:expr) => {
        {
            $config.$field = $args.$field;
            $declared[lib::which_declared!(quote::quote!($field).to_string().as_str())] =
            $config.$field;
        }
    };

    ($args:expr, $config:expr, $declared:expr, $($field:ident),+) => {
        ($(define_bool!($args, $config, $field, $declared),)+)
    }
}

pub fn get_args() -> (lib::Config, String, lib::DeclaredType) {
    // Processing Options
    let args = Cli::from_args();
    let mut declared: lib::DeclaredType = [false, false, false, false, false, false, false, false];

    if args.completion.is_some() {
        match Shell::from_str(&args.completion.unwrap()) {
            Ok(shell) => {
                let mut app = Cli::clap();
                app.gen_completions_to("fcs", shell, &mut io::stdout());
                exit(exitcode::OK);
            }
            Err(e) => {
                log::error!("{}", e);
                exit(exitcode::DATAERR);
            }
        }
    }

    let config: String;

    config = match args.config {
        Some(file) => {
            declared[lib::which_declared!("config")] = true;
            file
        }
        None => format!(
            "{}{}fcs.yml",
            config_dir().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
        ),
    };

    let build_default = get_build_default();

    let result: lib::Config;
    let mut build_result: lib::BuildConfig = build_default;

    define_option!(
        args,
        build_result,
        declared,
        build_result,
        // Options to define
        dest,
        dirs,
        sleep,
        codes
    );

    define_bool!(
        args,
        build_result,
        declared,
        // Bools to define
        once,
        timeinfo,
        static_mode
    );

    result = convert_types(build_result);

    if args.generate_config {
        let mut result = result;

        match clean_custom(&mut result) {
            true => {
                log::error!("Configuration is unusable");
                exit(exitcode::DATAERR);
            }
            false => (),
        }

        let yaml_result = lib::ConfigSerDe {
            dest: Some(result.dest),
            dirs: Some(result.dirs),
            once: Some(result.once),
            timeinfo: Some(result.timeinfo),
            static_mode: Some(result.static_mode),
            sleep: Some(result.sleep),
            codes: Some(result.codes),
        };

        let deserialized = match serde_yaml::to_string(&yaml_result) {
            Ok(res) => res,
            Err(e) => {
                log::error!(
                    "Failed somehow to parse configuration. Error : {}",
                    e.to_string()
                );
                exit(exitcode::DATAERR);
            }
        };
        print!("{}", deserialized);

        exit(exitcode::OK);
    }

    (result, config, declared)
}

fn convert_types(build_result: lib::BuildConfig) -> lib::Config {
    let codes: HashMap<String, String> = build_result
        .codes
        .unwrap()
        .iter()
        .map(|x| (x.0.to_owned(), x.1.to_owned()))
        .collect();
    let dirs: HashSet<String> = build_result
        .dirs
        .unwrap()
        .iter()
        .map(|x| x.to_owned())
        .collect();

    let dest = build_result.dest.unwrap();
    let sleep = build_result.sleep.unwrap();

    let once = build_result.once;
    let timeinfo = build_result.timeinfo;
    let static_mode = build_result.static_mode;

    lib::Config {
        codes,
        dirs,
        dest,
        sleep,
        once,
        timeinfo,
        static_mode,
    }
}
