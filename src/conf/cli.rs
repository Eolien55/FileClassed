use dirs_next::config_dir;
use structopt::clap::Shell;
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::path;
use std::process::exit;
use std::str::FromStr;

use super::defaults::get_defaults;
use super::lib;

// The bool value indicates if the config is so messed
// up that it is unusable, and if the program should exit
pub fn clean_custom(config: &mut lib::Config) -> bool {
    let mut fatal = false;

    let valid_codes: HashMap<String, String> = config
        .codes
        .iter()
        .filter(|entry| {
            !vec![".", ".."].contains(&entry.1.as_str())
                && entry.0.matches('.').count() < 1
                && entry.0.len() > 0
                && entry.0.matches('/').count() < 1
                && entry.1.len() > 0
                && entry.1.matches('/').count() < 1
        })
        .map(|entry| (entry.0.to_owned(), entry.1.to_owned()))
        .collect();

    let valid_codes_keys: HashSet<String> =
        valid_codes.iter().map(|entry| entry.0.to_owned()).collect();
    let codes_keys: HashSet<String> = config
        .codes
        .iter()
        .map(|entry| entry.0.to_owned())
        .collect();

    let invalid_codes_keys = codes_keys.difference(&valid_codes_keys);

    for key in invalid_codes_keys {
        log::warn!(
            "Shortcut `{}={}` isn't valid ! Not using it",
            key,
            config.codes[key]
        );
    }
    config.codes = valid_codes;

    if config.codes.is_empty() {
        log::error!("No shortcut set up, or none of them are valid ! Exiting");
        fatal = true;
    }

    log::debug!("Here's the config : {:?}", config);

    return fatal;
}

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

    return verbose.log_level();
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

    let build_default = get_defaults();

    let result: lib::Config;
    let mut build_result: lib::BuildConfig = build_default.clone();

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
        let mut result = result.clone();

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
            Ok(result) => result,
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

    return (result, config, declared);
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

    let result = lib::Config {
        codes,
        dirs,
        dest,
        sleep,
        once,
        timeinfo,
        static_mode,
    };

    return result;
}
