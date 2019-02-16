use env_logger;
use jdiff;

use std::env;
use std::process;

fn main() {
    env_logger::init();
    // parse config from arguments
    let args: Vec<String> = env::args().collect();
    let config = jdiff::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}.", err);
        process::exit(1);
    });
    // compare json files
    jdiff::run(config);
}
