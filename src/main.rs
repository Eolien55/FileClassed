use std::process::exit;
use fcs::{main_config, conf, run};
use simple_logger::init_with_level;

fn main() {
    let verbose = conf::cli::get_verbose();
    init_with_level(verbose.unwrap_or(log::Level::Error)).unwrap();

    log::trace!("Setting up human panic");
    human_panic::setup_panic!();

    log::trace!("Setting up the configuration");
    let (mut my_config, config_file, declared) = main_config::get_config_args();

    log::trace!("Cleaning a bit configuration");
    let fatal = main_config::clean(&mut my_config);
    if fatal {
        log::info!("Goodbye");
        exit(exitcode::DATAERR);
    }

    log::trace!("Ready to do the dirty job ! Configuration is ready");
    run::run(my_config, declared, config_file);

    log::info!("Goodbye");
    exit(exitcode::OK);
}