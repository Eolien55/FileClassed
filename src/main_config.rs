use std::collections::HashSet;

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

    config.dest = String::from(shellexpand::env(&config.dest).unwrap());
    config.dirs = config
        .dirs
        .iter()
        .map(|dir| (String::from(shellexpand::env(&dir).unwrap())))
        .collect();

    let existing_dirs: HashSet<String> = config
        .dirs
        .iter()
        .filter(|&dir| conf::exists(&dir))
        .map(|dir| String::from(dir))
        .collect();

    let non_existing_dirs = config.dirs.difference(&existing_dirs);
    for dir in non_existing_dirs {
        log::warn!("Watching directory \"{}\" doesn't exist. Not using it", dir);
    }

    config.dirs = existing_dirs;
    if config.dirs.is_empty() {
        log::error!("No directories set up, or none of them exist ! Exiting");
        fatal = true;
    }

    if !conf::exists(&config.dest) {
        log::error!("Destination \"{}\" doesn't exist ! Exiting", config.dest);
        fatal = true;
    }

    return fatal;
}
