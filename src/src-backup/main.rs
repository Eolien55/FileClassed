use std::process::exit;
use file_classed::{config, run};

fn main() {
    human_panic::setup_panic!();

    let the_config = config::get_config_args();

    let my_config = the_config.0;
    let my_config = config::clean(my_config);
    println!("{:?}", my_config);

    run::run(my_config);

    exit(exitcode::OK);
}