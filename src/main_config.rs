use std::collections::{HashMap, HashSet};

use crate::conf::lib as conf;
use crate::conf::{cli, file};

// Get config from CLI args and config file
pub fn get_config_args() -> (conf::Config, String, conf::DeclaredType) {
    log::trace!("Getting arguments from CLI");
    let (mut config, mut config_file, declared) = cli::get_args();

    file::get_config(&mut config, &mut config_file, &declared);

    (config, config_file, declared)
}

// The bool value indicates if the config is so messed
// up that it is unusable, and if the program should exit
pub fn clean(config: &mut conf::Config) -> bool {
    let mut fatal = false;

    match shellexpand::full(&config.dest) {
        Ok(result) => config.dest = String::from(result),
        Err(e) => {
            log::error!(
                "Error while expanding destination : {}. Exiting",
                e.to_string()
            );
            fatal = true;
            return fatal;
        }
    }
    config.dirs = config
        .dirs
        .iter()
        .map(|dir| match shellexpand::full(&dir) {
            Ok(result) => String::from(result),
            Err(e) => {
                log::warn!("Error while expanding dirs : {}", e.to_string());
                String::from(dir)
            }
        })
        .collect();

    if fatal {
        return fatal;
    }

    let existing_dirs: HashSet<String> = config
        .dirs
        .iter()
        .filter(|&dir| conf::test_path!(&dir, "dir"))
        .map(String::from)
        .collect();

    let non_existing_dirs = config.dirs.difference(&existing_dirs);
    for dir in non_existing_dirs {
        log::warn!(
            "Watching directory `{}` doesn't exist, can't be expanded or isn't a directory. Not using it",
            dir
        );
    }

    config.dirs = existing_dirs;
    if config.dirs.is_empty() {
        log::error!("No directories set up, or none of them exist ! Exiting");
        fatal = true;
    }

    if !(conf::test_path!(&config.dest, "dir")) {
        log::error!(
            "Destination `{}` doesn't exist, or isn't a directory ! Exiting",
            config.dest
        );
        fatal = true;
    }

    let valid_codes: HashMap<String, String> = config
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

    fatal
}
