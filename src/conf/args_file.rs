use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::str::FromStr;

use super::cli;
use super::lib as conf;

impl conf::Config {
    // Get config from CLI args and config file
    pub fn from_args_and_file(args: cli::Cli) -> (Self, String, conf::DeclaredType) {
        log::trace!("Getting arguments from CLI");
        let (mut config, mut config_file, declared) = conf::Config::from_args(args);

        config.add_or_update_from_file(&mut config_file, &declared);

        config.clean();

        (config, config_file, declared)
    }

    // The bool value indicates if the config is so messed
    // up that it is unusable, and if the program should exit
    pub fn clean(&mut self) -> bool {
        let mut fatal = false;

        match shellexpand::full(self.dest.to_str().unwrap()) {
            Ok(result) => self.dest = PathBuf::from_str(&result).unwrap(),
            Err(e) => {
                log::error!(
                    "Error while expanding destination : {}. Exiting",
                    e.to_string()
                );
                fatal = true;
                return fatal;
            }
        }
        self.dirs = self
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

        if fatal {
            return fatal;
        }

        let existing_dirs: HashSet<PathBuf> = self
            .dirs
            .iter()
            .filter(|&dir| conf::test_path!(&dir, "dir"))
            .map(PathBuf::from)
            .collect();

        let non_existing_dirs = self.dirs.difference(&existing_dirs);
        for dir in non_existing_dirs {
            log::warn!(
            "Watching directory `{:#?}` doesn't exist, can't be expanded or isn't a directory. Not using it",
            dir
        );
        }

        self.dirs = existing_dirs;
        if self.dirs.is_empty() {
            log::error!("No directories set up, or none of them exist ! Exiting");
            fatal = true;
        }

        if !(conf::test_path!(&self.dest, "dir")) {
            log::error!(
                "Destination `{:#?}` doesn't exist, or isn't a directory ! Exiting",
                self.dest
            );
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
        self.codes = valid_codes;

        if self.codes.is_empty() {
            log::error!("No shortcut set up, or none of them are valid ! Exiting");
            fatal = true;
        }

        if self.begin_var == self.end_var {
            log::error!(
                "The 'begin variable token' ({}) is identical to the 'end variable token' ({})",
                self.begin_var,
                self.end_var
            );
            fatal = true;
        }

        log::debug!("Here's the config : {:#?}", self);

        fatal
    }
}
