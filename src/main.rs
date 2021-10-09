use std::process::exit;
use file_classed::cli;

fn main() {
    let should_end = false;

    human_panic::setup_panic!();

    let config = cli::get_config_args();

    let my_config = config.0;

    exit(exitcode::OK);
}