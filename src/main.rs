use env_logger;
use jdiff;
use log;

use std::env;
use std::process;

fn main() {
    env_logger::init();
    // parse config from arguments
    let args: Vec<String> = env::args().collect();
    let config = jdiff::Config::new(&args).unwrap_or_else(|err| {
        log::error!("Error parsing arguments: {}.", err);
        process::exit(1);
    });
    // compare json files and output deltas
    jdiff::run(config);
}
