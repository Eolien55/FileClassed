use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;

use super::cli;
use super::lib as conf;

impl conf::Config {
    // Get config from CLI args and config file
    pub fn from_args_and_file(args: cli::Cli) -> (Self, String, conf::DeclaredType, bool) {
        log::trace!("Getting arguments from CLI");
        let (mut config, mut config_file, declared) = conf::Config::from_args(args);

        config.add_or_update_from_file(&mut config_file, &declared);

        log::trace!("Cleaning a bit configuration");
        let fatal = config.clean(true);

        (config, config_file, declared, fatal)
    }

    // The bool value indicates if the config is so messed
    // up that it is unusable, and if the program should exit
    pub fn clean(&mut self, mutates: bool) -> bool {
        let mut fatal = false;
        let mut true_fatal = false;

        let dest;
        match shellexpand::full(self.dest.to_str().unwrap()) {
            Ok(result) => dest = PathBuf::from_str(&result).unwrap(),
            Err(e) => {
                if mutates {
                    log::error!(
                        "Error while expanding destination : {}. Exiting",
                        e.to_string()
                    );
                } else {
                    log::warn!("Error while expanding destination : {}", e.to_string());
                }
                fatal = true;

                dest = self.dest.clone();
            }
        }
        let dirs: HashSet<_> = self
            .dirs
            .iter()
            .map(|dir| match shellexpand::full(dir.to_str().unwrap()) {
                Ok(result) => PathBuf::from_str(&result).unwrap(),
                Err(e) => {
                    log::warn!("Error while expanding dirs : {}", e.to_string());
                    dir.to_owned()
                }
            })
            .collect();

        let existing_dirs: HashSet<_> = dirs
            .iter()
            .filter(|&dir| conf::test_path!(&dir, "dir"))
            .map(PathBuf::from)
            .collect();

        let non_existing_dirs = dirs.difference(&existing_dirs);
        for dir in non_existing_dirs {
            log::warn!(
                "Watching directory `{:#?}` doesn't exist, can't be expanded or isn't a directory. Not using it",
                dir
            );
        }

        if existing_dirs.is_empty() {
            if mutates {
                log::error!("No directories set up, or none of them exist ! Exiting");
            } else {
                log::warn!("No directories set up, or none of them exist");
            }
            fatal = true;
        }

        if !(conf::test_path!(&dest, "dir")) {
            if mutates {
                log::error!(
                    "Destination `{:#?}` doesn't exist, or isn't a directory ! Exiting",
                    dest
                );
            } else {
                log::warn!(
                    "Destination `{:#?}` doesn't exist, or isn't a directory",
                    dest
                );
            }
            fatal = true;
        }

        let valid_codes: HashMap<String, String> = self
            .codes
            .iter()
            .filter(|entry| {
                !vec![".", ".."].contains(&entry.1.as_str())
                    && entry.0.matches('.').count() < 1
                    && !entry.0.is_empty()
                    && entry.0.matches('/').count() < 1
                    && !entry.1.is_empty()
                    && entry.1.matches('/').count() < 1
            })
            .map(|entry| (entry.0.to_owned(), entry.1.to_owned()))
            .collect();

        let valid_codes_keys: HashSet<String> =
            valid_codes.iter().map(|entry| entry.0.to_owned()).collect();
        let codes_keys: HashSet<String> =
            self.codes.iter().map(|entry| entry.0.to_owned()).collect();

        let invalid_codes_keys = codes_keys.difference(&valid_codes_keys);

        for key in invalid_codes_keys {
            log::warn!(
                "Shortcut `{}={}` isn't valid ! Not using it",
                key,
                self.codes[key]
            );
        }

        if valid_codes.is_empty() {
            log::error!("No shortcut set up, or none of them are valid ! Exiting");
            true_fatal = true;
        }

        if self.begin_var == self.end_var {
            log::error!(
                "The 'begin variable token' ({}) is identical to the 'end variable token' ({})",
                self.begin_var,
                self.end_var
            );
            true_fatal = true;
        }

        if mutates {
            self.dest = dest;
            self.dirs = existing_dirs;
            self.codes = valid_codes;
        }

        log::debug!("Here's the config : {:#?}", self);

        true_fatal || (fatal && mutates)
    }
}
