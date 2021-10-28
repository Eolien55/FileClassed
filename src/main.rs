use simple_logger::init_with_level;

use std::process::exit;
use structopt::StructOpt;

use fcs::{conf, run};

fn main() {
    let args = conf::cli::Cli::from_args();
    let verbose = conf::cli::get_verbose();
    match init_with_level(verbose.unwrap_or(log::Level::Error)) {
        Ok(_) => (),
        Err(e) => {
            println!(
                "Error happenned while setting up logger : {}. Exiting",
                e.to_string()
            );
            exit(exitcode::DATAERR);
        }
    }

    log::trace!("Setting up human panic");
    human_panic::setup_panic!();

    log::trace!("Setting up the configuration");
    let (mut my_config, config_file, declared) = conf::lib::Config::from_args_and_file(args);

    log::trace!("Cleaning a bit configuration");
    let fatal = my_config.clean();
    if fatal {
        log::info!("Goodbye");
        exit(exitcode::DATAERR);
    }

    log::trace!("Ready to do the dirty job ! Configuration is ready");
    run::run(my_config, declared, config_file);

    log::info!("Goodbye");
    exit(exitcode::OK);
}
