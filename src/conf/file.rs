use dirs_next::config_dir;

use std::fs;
use std::path;

use super::lib;

macro_rules! replace_value {
    ($conf1:expr, $conf2:expr, $attr:expr, $declared:expr) => {
        if !$declared[lib::which_declared!($attr)] && !$conf2.is_none() {
            $conf1 = $conf2.unwrap();
        }
    };

    ($conf1:expr, $conf2:expr, $attr:expr, $declared:expr, $default:expr) => {
        if !$declared[lib::which_declared!($attr)] && !$conf2.is_none() {
            $conf1 = $conf2.unwrap();
        } else if !$declared[lib::which_declared!($attr)] {
            $conf1 = $default;
        }
    };
}

pub fn get_config(
    config: &mut lib::Config,
    config_file: &mut String,
    declared: &lib::DeclaredType,
) {
    let mut default_config_files = vec![
        format!(
            "{}{}fcs.yml",
            config_dir().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
        ),
        format!(
            "{}{}fcs{}init.yml",
            config_dir().unwrap().to_str().unwrap(),
            path::MAIN_SEPARATOR,
            path::MAIN_SEPARATOR,
        ),
    ];

    if !declared[lib::which_declared!("config")] {
        while !lib::test_path!(&config_file, "file") && !default_config_files.is_empty() {
            *config_file = default_config_files.pop().unwrap();
        }
    }

    log::trace!("Reading `{}` for config", config_file);

    match fs::read_to_string(&config_file) {
        Ok(reading_file) => match serde_yaml::from_str::<lib::ConfigSerDe>(&reading_file) {
            Ok(from_file) => {
                log::debug!("Config from file : {:?}", from_file);
                replace_value!(config.dirs, from_file.dirs, "dirs", declared);
                replace_value!(config.dest, from_file.dest, "dest", declared);
                replace_value!(config.once, from_file.once, "once", declared);
                replace_value!(config.sleep, from_file.sleep, "sleep", declared);
                replace_value!(config.codes, from_file.codes, "codes", declared);
                replace_value!(
                    config.timeinfo,
                    from_file.timeinfo,
                    "timeinfo",
                    declared,
                    false
                );
                replace_value!(
                    config.static_mode,
                    from_file.static_mode,
                    "static_mode",
                    declared,
                    false
                );
            }
            Err(e) => {
                log::error!(
                    "Error happenned while parsing config file `{}`. Falling back to defaults",
                    e.to_string()
                );
            }
        },
        Err(_) => {
            log::error!(
                "Config file `{}` doesn't exist or isn't valid UTF-8. Falling back to defaults",
                config_file
            );
        }
    }
}
