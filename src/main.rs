extern crate regex;

use human_panic::setup_panic;



pub mod common {
    pub fn fatal_error(message: &str) -> ! {
        eprintln!("{}ERROR: {}", termion::color::Fg(termion::color::Red), message);
        std::process::exit(1);
    }
}

pub mod args;

fn main() {
    // Setup human readable panic (only activates in release mode)
    setup_panic!();

    // Parse the config arguments
    let args = args::init();

    // Parse the given paths
    let paths = args::validate_paths(&args);

    for file in &paths {
        println!("Parsing {}", file);
    }


}

