use jdiff;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    // parse config from arguments
    let config = jdiff::Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {}.", err);
        process::exit(1);
    });
    // compare json files
    if let Err(err) = jdiff::run(config) {
        eprintln!("Application error: {}.", err);
        process::exit(1);
    };
}
