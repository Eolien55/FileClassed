use serde::{Deserialize, Serialize};
use std::collections::{HashMap};

#[derive(Debug)]
pub struct Config {
    pub dest : String,
    pub dirs : Vec<String>,
    pub once : bool,
    pub sleep : u32,
    pub codes : HashMap<String, String>,
    pub months : HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSerDe {
    pub dest : String,
    pub dirs : Vec<String>,
    pub once : bool,
    pub sleep : u32,
    pub codes : HashMap<String, String>,
    pub months : [String ; 12],
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
            "months" => 5,
            "verbose" => 6,
            _ => 7,
        }
    };
}

macro_rules! which_month {
    ($val:expr) => {
        match $val {
            0 => "???",
            1 => "Jan",
            2 => "Feb",
            3 => "Mar",
            4 => "Apr",
            5 => "May",
            6 => "Jun",
            7 => "Jul",
            8 => "Aug",
            9 => "Sep",
            10 => "Oct",
            11 => "Nov",
            12 => "Dec",
            _ => "?????",
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
    sleep : Option<u32>,

    /// Shortcuts, ie meanings
    #[structopt(
        short = "-c",
        long = "--code",
        value_name = "shortcut=meaning", 
        parse(try_from_str = parse_key_val)
    )]
    codes : Option<Vec<(String, String)>>,

    /// Month codes
    #[structopt(short, long, value_name="months", number_of_values = 12, multiple = false)]
    months : Option<Vec<String>>,

    /*/// Makes the program verbose
    #[structopt(flatten)]
    verbose : clap_verbosity_flag::Verbosity,*/
}

fn get_args() -> (Config, String, [bool ; 6]) {
    // Processing Options
    let args = Cli::from_args();
    let mut declared : [bool ; 6] = [false, false, false, false, false, false];

    let config = args.config.unwrap_or(
        format!(
            "{}{}{}", config_dir().unwrap().to_str().unwrap(), path::MAIN_SEPARATOR, "fcs.yml"
        )
    );

    let dest : String;
    let mut dirs : Vec<String>;
    let once : bool;
    let sleep : u32;
    let mut codes : HashMap<String, String> = HashMap::with_capacity(15);
    let mut months : HashMap<String, String> = HashMap::with_capacity(12);

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

    if !args.months.is_none() {
        let mut counter : u8 = 0;
        months = args.months.unwrap().iter()
                                     .map(|month| {counter+=1; (which_month!(counter).to_string(), month.to_owned())})
                                     .collect();
        declared[which_declared!("months")] = true;
    } else {
        let mut counter : u8 = 0;
        months = [
                    "Janvier",
                    "Février",
                    "Mars",
                    "Avril",
                    "Mai",
                    "Juin",
                    "Juillet",
                    "Août",
                    "Septembre",
                    "Octobre",
                    "Novembre",
                    "Décembre"
                ].iter()
                 .map(|month| {counter+=1; (which_month!(counter).to_string(), month.to_string())})
                 .collect();
    }

    dirs.shrink_to_fit();

    return (Config {dest, dirs, once, sleep, codes, months}, config, declared);
}

/*










BUILDING CONFIG FROM COMMAND-LINE










*/

/*










BUILDING CONFIG FROM CONFIG FILE, WHILE RESPECTING PREVIOUS CONFIG










*/

use std::fs;

macro_rules! replace_value {
    ($conf1:expr, $conf2:expr, $attr:expr, $declared:expr) => {
        if !$declared[which_declared!($attr)] {
            $conf1 = $conf2;
        }
    };
}

/*










BUILDING CONFIG FROM CONFIG FILE, WHILE RESPECTING PREVIOUS CONFIG










*/

pub fn get_config_args() -> (Config, String) {
    let (mut config, config_file, declared) = get_args();

    let from_file : ConfigSerDe;
    match fs::read_to_string(config_file) {
        Ok(reading_file) => {
            match serde_yaml::from_str::<ConfigSerDe>(&reading_file) {
                Ok(from_file) => {
                    replace_value!(config.dirs, from_file.dirs, "dirs", declared);
                    replace_value!(config.dest, from_file.dest, "dest", declared);
                    replace_value!(config.once, from_file.once, "once", declared);
                    replace_value!(config.sleep, from_file.sleep, "sleep", declared);
                    replace_value!(config.codes, from_file.codes, "codes", declared);
                    let mut counter : u8 = 0;
                    replace_value!(config.months, from_file.months.iter()
                                                                  .map(
                                                                      |month| {counter+=1; (which_month!(counter).to_string(), month.to_owned())
                                                                    })
                                                                   .collect(), "months", declared);
                    (config, "".to_string())
                },
                Err(e) => (config, e.to_string())
            }
        },
        Err(_) => (config, "".to_string())
    }
}

pub fn clean(mut config : Config) -> Config {
    config.dest = String::from(shellexpand::env(&config.dest).unwrap());
    config.dirs = config.dirs.iter()
                             .map(|dir| (String::from(shellexpand::env(&dir).unwrap())))
                             .collect();
    
    return config;
}