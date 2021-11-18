use dirs_next::config_dir;
use structopt::clap::Shell;
use structopt::StructOpt;

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::process::exit;
use std::{path, path::PathBuf};

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

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "fcs",
    about = "Efficient yet customizable file organizer",
    author = "Elie Le Vaillant <elielevaillant2007@gmail.com>",
    setting = structopt::clap::AppSettings::ColoredHelp
)]

pub struct Cli {
    /// Sets the configurationg file
    #[structopt(short = "-C", long, value_name = "file")]
    config: Option<PathBuf>,

    /// Sets the watching directories
    #[structopt(short, long = "dir", value_name = "directory")]
    dirs: Option<Vec<PathBuf>>,

    /// Sets destination directory
    #[structopt(short = "-D", long, value_name = "directory")]
    dest: Option<PathBuf>,

    /// Loops only once then exit
    #[structopt(short, long)]
    once: bool,

    /// Sets sleeping time between each loop in ms
    #[structopt(short, long, value_name = "milliseconds")]
    sleep: Option<usize>,

    /// Sets shortcuts
    #[structopt(
        short = "-c",
        long = "--code",
        value_name = "shortcut=meaning", 
        parse(try_from_str = parse_key_val)
    )]
    codes: Option<Vec<(String, String)>>,

    /// Includes year and month in path
    #[structopt(short, long)]
    timeinfo: bool,

    /// Sets verbosity
    #[structopt(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Generates completion script for shell and output it
    #[structopt(long, value_name = "shell", possible_values(&Shell::variants()))]
    completion: Option<Shell>,

    /// Disables configuration reloading on configuration file change
    #[structopt(short = "-S", long = "--static")]
    static_mode: bool,

    /// Generates configuration file from CLI arguments and output it
    ///
    /// Note that the generation isn't deterministic, which means codes won't be ordered the same way.
    /// Note also that the generated config file isn't pretty printed, ie it's quite ugly
    #[structopt(short, long)]
    generate_config: bool,

    /// Set the separator to separate each filename part
    #[structopt(short = "-p", long, value_name="char")]
    separator: Option<char>,

    /// Set the number of characters that are the separator
    /// ('.' by default) in the filename
    #[structopt(short, long, value_name="number")]
    filename_separators: Option<usize>,

    /// Set the character to detect a variable 'lookup'
    /// ('{' by default)
    #[structopt(short, long, value_name="char")]
    begin_var: Option<char>,

    /// Set the character to detect the end of a variable 'lookup'
    /// ('}' by default)
    #[structopt(short, long, value_name="char")]
    end_var: Option<char>,
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

impl lib::Config {
    pub fn from_args(args: Cli) -> (Self, String, lib::DeclaredType) {
        let mut declared: lib::DeclaredType = [false; 12];

        if let Some(shell) = args.completion {
            let mut app = Cli::clap();
            app.gen_completions_to("fcs", shell, &mut io::stdout());
            exit(exitcode::OK);
        }

        let config = match args.config {
            Some(file) => {
                declared[lib::which_declared!("config")] = true;
                file
            }
            None => PathBuf::from(format!(
                "{}{}fcs.yml",
                config_dir().unwrap().to_str().unwrap(),
                path::MAIN_SEPARATOR,
            )),
        };

        let build_default = lib::BuildConfig::default();

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
            codes,
            separator,
            filename_separators,
            begin_var,
            end_var
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

        let result = convert_types(build_result);

        if args.generate_config {
            let mut result = result;

            match result.clean(false) {
                true => {
                    log::error!("Configuration is unusable");
                    exit(exitcode::DATAERR);
                }
                false => (),
            }

            let yaml_result = lib::ConfigSerDe {
                separator: Some(result.separator),
                filename_separators: Some(result.filename_separators),
                begin_var: Some(result.begin_var),
                end_var: Some(result.end_var),
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
        (result, config.to_str().unwrap().to_string(), declared)
    }
}

fn convert_types(build_result: lib::BuildConfig) -> lib::Config {
    let codes: HashMap<String, String> = build_result
        .codes
        .unwrap()
        .iter()
        .map(|x| (x.0.to_owned(), x.1.to_owned()))
        .collect();
    let dirs: HashSet<PathBuf> = build_result
        .dirs
        .unwrap()
        .iter()
        .map(|x| x.to_owned())
        .collect();

    let dest = build_result.dest.unwrap();
    let sleep = build_result.sleep.unwrap();
    let separator = build_result.separator.unwrap();
    let filename_separators = build_result.filename_separators.unwrap();
    let begin_var = build_result.begin_var.unwrap();
    let end_var = build_result.end_var.unwrap();

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
        separator,
        filename_separators,
        begin_var,
        end_var,
    }
}
