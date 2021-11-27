use dirs_next::config_dir;

use std::fs;
use std::path;

use super::lib;

macro_rules! replace_value {
    ($fromfile:expr, $config:expr, $field:ident, $declared:expr, $default:expr) => {
        {if !$declared[lib::which_declared!(quote::quote!($field).to_string().as_str())] && !$fromfile.$field.is_none() {
            $config.$field = $fromfile.$field.unwrap();
        } else if !$declared[lib::which_declared!(quote::quote!($field).to_string().as_str())] {
            $config.$field = $default.$field;
        }}
    };

    ($fromfile:expr, $config:expr, $declared:expr, $default:expr, $($field:ident),+) => {
        ($(replace_value!($fromfile, $config, $field, $declared, $default),)+)
    };
}

impl lib::Config {
    pub fn add_or_update_from_file(
        &mut self,
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

        log::trace!("Reading `{:#}` for config", config_file);

        match fs::read_to_string(&config_file) {
            Ok(reading_file) => match serde_yaml::from_str::<lib::ConfigSerDe>(&reading_file) {
                Ok(from_file) => {
                    log::debug!("Config from file : {:#?}", from_file);
                    let default = lib::Config::default();
                    replace_value!(
                        from_file,
                        self,
                        declared,
                        default,
                        // Values to replace
                        dirs,
                        dest,
                        once,
                        sleep,
                        codes,
                        timeinfo,
                        static_mode,
                        separator,
                        filename_separators,
                        begin_var,
                        end_var
                    );
                }
                Err(e) => {
                    log::error!(
                        "Error happenned while parsing config file `{:#}`. Falling back to defaults",
                        e.to_string()
                    );
                }
            },
            Err(_) => {
                log::error!(
                    "Config file `{:#}` doesn't exist or isn't valid UTF-8. Falling back to defaults",
                    config_file
                );
            }
        }
    }
}
