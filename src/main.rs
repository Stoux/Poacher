extern crate regex;

pub mod common;
pub mod args;
pub mod tui;
pub mod files;
pub mod processing;

use human_panic::setup_panic;
use std::sync::mpsc;
use mpsc::{Receiver, Sender};
use crate::common::{Event, fatal_error};
use std::thread::{sleep, JoinHandle};
use std::time::Duration;
use crate::files::events::{ToMainEvent as FilesToMainEvent, FromMainEvent as MainToFilesEvent, FileEvent as FilesToProcessingEvent, FileEvent, ToMainEvent, FromMainEvent};
use std::sync::mpsc::RecvTimeoutError;

fn main() {
    // Setup human readable panic (only activates in release mode)
    setup_panic!();

    // Parse the config arguments
    let args = args::init();

    // Parse the given paths
    let paths = args::validate_paths(&args);

    // Create the file processor thread
    let (
        file_thread_handle,
        main_to_files_thread_transmitter,
        files_to_processing_processing_thread_receiver,
    ) = create_file_processor(paths.clone(), args.is_present(args::OPTION_ONLY_TAIL));





    // Wait for all threads to finish
    file_thread_handle.join().unwrap();

    println!("Done!");
}

fn create_file_processor(
    paths: Vec<String>,
    only_tail_files: bool,
) -> (
    JoinHandle<()>,
    Sender<FromMainEvent>,
    Receiver<FileEvent>
) {
    // Build communication channels for all threads
    let (files_to_main_thread_transmitter, files_to_main_thread_receiver): (Sender<FilesToMainEvent>, Receiver<FilesToMainEvent>) = mpsc::channel();
    let (main_to_files_thread_transmitter, main_to_files_thread_receiver): (Sender<MainToFilesEvent>, Receiver<MainToFilesEvent>) = mpsc::channel();
    let (files_to_processing_thread_transmitter, files_to_processing_processing_thread_receiver): (Sender<FilesToProcessingEvent>, Receiver<FilesToProcessingEvent>) = mpsc::channel();


    // Start the thread for processing the files
    let files_handle = crate::files::start(
        paths,
        main_to_files_thread_receiver,
        files_to_main_thread_transmitter,
        files_to_processing_thread_transmitter,
        only_tail_files,
    );

    // Wait for files thread to initialize
    let mut initialised_files_thread = false;
    if let Ok(event) = files_to_main_thread_receiver.recv_timeout(Duration::from_secs(3)) {
        if let ToMainEvent::AllOk = event {
            initialised_files_thread = true;
        }
    }
    if ! initialised_files_thread {
        fatal_error("Failed to initialise files thread?")
    }

    return (files_handle, main_to_files_thread_transmitter, files_to_processing_processing_thread_receiver);
}

