use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, TryRecvError};
use notify::{Watcher, RecursiveMode, watcher, FsEventWatcher, Error, DebouncedEvent};
use crate::common::{fatal_error};
use std::path::Path;
use std::fmt::Debug;
use crate::files::events::{FromMainEvent, FileEvent, ToMainEvent, FileLines};
use std::collections::HashMap;
use std::cell::Cell;
use crate::files::handlers::WatchedFile;

pub mod handlers;
pub mod events;

struct FilesProcessor {
    files: HashMap<String, WatchedFile>,
    message_receiver: Receiver<FromMainEvent>,
    to_processing_thread: Sender<FileEvent>,
}

impl FilesProcessor {
    fn run(mut self,
           to_main_thread: Sender<ToMainEvent>,
    ) {
        // Create a communication channel for the watcher
        let (file_tx, file_rx) = channel();

        // Build the watcher
        let mut watcher = match watcher(file_tx, Duration::from_millis(50)) {
            Ok(watcher) => watcher,
            Err(error) => {
                fatal_error(&format!("Failed to create watcher: {}", error));
            }
        };

        // Start adding the files to the watcher
        for file in self.files.values() {
            match watcher.watch(file.get_path(), RecursiveMode::NonRecursive) {
                Ok(_) => {}
                Err(error) => {
                    fatal_error(&format!("Failed to watch file '{}': {}", file.get_filename(), error));
                }
            }
        }

        // Notify the main thread that we're ready to start processing files
        to_main_thread.send(ToMainEvent::AllOk).unwrap();

        // Read all existing files to their end
        for file in self.files.values_mut() {
            let lines = file.read_to_end();
            if ! lines.is_empty() {
                FilesProcessor::send_lines(&self.to_processing_thread, file, lines);
            }
        }


        // Start listening to events
        loop {
            // Fetch file change events from the watcher
            loop {
                // Keep processing them until we reive a
                match file_rx.try_recv() {
                    Ok(event) => {
                        // We only care about write events
                        if let DebouncedEvent::Write(path) = event {
                            let filename = path.to_str().unwrap();
                            let mut file = match self.files.get_mut(filename) {
                                Some(file) => file,
                                None => fatal_error(&format!("Got error for file that we aren't watching? {}", filename)),
                            };

                            let lines = file.read_to_end();
                            if !lines.is_empty() {
                                FilesProcessor::send_lines(&self.to_processing_thread, file, lines);
                            }
                        }
                    }
                    Err(watch_error) => {
                        match watch_error {
                            TryRecvError::Empty => {  /* There is no message waiting, stop this loop. */ },
                            TryRecvError::Disconnected => eprintln!("Watch error {:?}", watch_error)
                        }
                    }
                }
            }

            // Check if there are no messages waiting from the main thread
            let mut shutdown = false;
            match self.message_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        FromMainEvent::Shutdown => {
                            println!("Shutdown command received from main thread.");
                            shutdown = true;
                        }
                    }
                }
                Err(message_error) => {
                    match message_error {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            eprintln!("Error communicating with main thread {:?}", message_error);
                            eprintln!("Shutting down thread.");
                            shutdown = true;
                        }
                    }
                }
            }

            if shutdown {
                return;
            }
        }
    }

    fn send_lines(to_processing_thread: &Sender<FileEvent>, file: &WatchedFile, lines: Vec<String>) {
        println!("Sending {} lines from {}", lines.len(), file.get_filename());
        to_processing_thread.send(FileEvent::Lines(FileLines {
            lines,
            filename: file.get_filename().to_owned(),
        })).unwrap();
    }

    fn new(files: Vec<String>,
               message_receiver: Receiver<FromMainEvent>,
               to_processing_thread: Sender<FileEvent>,
               only_tail_files: bool) -> Self {
        // Map the
        let mut files_map = HashMap::new();
        for filename in files {
            files_map.insert(
                filename.clone(),
                WatchedFile::new(filename, only_tail_files),
            );
        }

        return FilesProcessor {
            files: files_map,
            message_receiver,
            to_processing_thread,
        };
    }
}

pub fn start(files: Vec<String>,
             message_receiver: Receiver<FromMainEvent>,
             to_main_thread: Sender<ToMainEvent>,
             to_processing_thread: Sender<FileEvent>,
             only_tail_files: bool,
) -> JoinHandle<()> {
    return thread::spawn(move || {
        let processor = FilesProcessor::new(
            files,
            message_receiver,
            to_processing_thread,
            only_tail_files
        );

        processor.run(to_main_thread);
    });
}