extern crate clap;

use clap::{Arg, App};
use self::clap::ArgMatches;

use std::path::Path;
use termion::color;
use std::fs;
use regex::Regex;

use crate::common::fatal_error;


pub const ARG_PATHS: &str = "paths";
pub const OPTION_CONFIG: &str = "config";
pub const OPTION_ONLY_TAIL: &str = "only-tail";
pub const OPTION_INCLUDE_ERROR_LOGS: &str = "error-logs";


/// Initialize and match the program arguments
pub fn init<'a>() -> ArgMatches<'a> {
    return App::new("Poacher")
        .version("0.1")
        .author("Leon Stam <leon@melonats.dev>")
        .about("A program that monitors nginx/apache log files and reports on bad-actors.")
        .arg(
            Arg::with_name(OPTION_CONFIG)
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Override the default config file")
        )
        .arg(
            Arg::with_name(OPTION_ONLY_TAIL)
                .short("t")
                .long("only-tail")
                .takes_value(false)
                .help("Ignore all data that's already in the log files, just tail them")
        )
        .arg(
            Arg::with_name(OPTION_INCLUDE_ERROR_LOGS)
                .long("include-error-logs")
                .takes_value(false)
                .help("Should include *.error.log files when searching in directories")
        )
        .arg(
            Arg::with_name(ARG_PATHS)
                .help("Path(s) to the files/folders you want to track")
                .required(true)
                .multiple(true)
                .value_name("PATH")
        )
        .get_matches();
}

/// Parse the given paths into accessible, readable log file paths.
pub fn validate_paths(args: &ArgMatches) -> Vec<String> {
    let paths = args
        .values_of(ARG_PATHS)
        .unwrap();

    let c_reset = color::Fg(color::Reset);
    let c_yellow = color::Fg(color::Yellow);

    // Validate all paths to be readable
    let mut log_files: Vec<String> = vec![];
    // TODO: Make this regex configurable through the arguments
    let log_match_regex = Regex::new(r"\.log$").unwrap();
    let error_log_match_regex = Regex::new(r"\.error\.log$").unwrap();
    let should_include_error_logs = args.is_present(OPTION_INCLUDE_ERROR_LOGS);
    let mut notified_error_logs = false;
    for (_index, file_arg) in paths.enumerate() {

        // TODO: Support * arguments

        // Check if the file/folder exists
        let file_path = Path::new(file_arg);
        if !file_path.exists() {
            fatal_error(&format!("{} doesn't exist!", file_arg));
        }

        // Check the type of file
        if file_path.is_dir() {
            // Attempt to find the files that are in that folder
            let inner_files = match fs::read_dir(file_path) {
                Ok(file) => file,
                Err(error) => {
                    fatal_error(&format!("Unable to open {}: {}", file_arg, error));
                }
            };

            // Loop through those files
            for inner_file in inner_files {
                // Skip everything that isn't a normal file.
                let inner_file = inner_file.unwrap();
                let inner_file_path = inner_file.path();
                if !inner_file_path.is_file() {
                    continue;
                }

                // Check if matches log format
                let inner_filename = inner_file.file_name().to_str().unwrap().to_owned();
                if log_match_regex.is_match(&inner_filename) {
                    if !should_include_error_logs && error_log_match_regex.is_match(&inner_filename) {
                        if !notified_error_logs {
                            eprintln!("{}Skipping *.error.log files in directories (see --include-error-logs){}", c_yellow, c_reset);
                            notified_error_logs = true;
                        }
                    } else {
                        log_files.push(inner_file.path().to_str().unwrap().to_owned());
                    }
                } else {
                    eprintln!("{}Skipping {} that does not match .log pattern{}", c_yellow, inner_file.path().to_str().unwrap(), c_reset);
                }
            }
        } else if file_path.is_file() {
            // Check if it's a file or a folder
            match fs::File::open(file_path) {
                Ok(file) => file,
                Err(error) => {
                    fatal_error(&format!("Unable to open {}: {}", file_arg, error));
                }
            };

            // Add the file to the list of files to track
            log_files.push(file_arg.to_owned());
        } else {
            // TODO: Fatal?
        }


    }

    return log_files;
}

