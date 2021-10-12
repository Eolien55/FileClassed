use std::process::exit;
use file_classed::{config, run};

fn main() {
    human_panic::setup_panic!();

    let the_config = config::get_config_args();

    let my_config = the_config.0;
    let (my_config, config_errors) = config::clean(my_config);
    if (&config_errors).into_iter().map(|entry| entry.0).any(|entry| entry == true) {
        eprintln!("Error, exiting.");
        exit(exitcode::OK);
    }

    run::run(my_config);

    exit(exitcode::OK);
}