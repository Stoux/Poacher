use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::sync::mpsc::{channel, Receiver, Sender, RecvError, TryRecvError};
use notify::{Watcher, RecursiveMode, watcher, FsEventWatcher, Error, DebouncedEvent};
use crate::common::{fatal_error};
use std::path::Path;
use std::fmt::Debug;
use crate::files::events::{FromMainEvent, FileEvent, ToMainEvent};

pub mod events;

struct FilesProcessor {
    message_receiver: Receiver<FromMainEvent>,
    to_processing_thread: Sender<FileEvent>,
}

impl FilesProcessor {
    fn run(self,
           to_main_thread: Sender<ToMainEvent>,
           files: Vec<String>,
           only_tail_files: bool,
    ) {
        // TODO: Build current indexes of the files

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
        for file in files {
            match watcher.watch(Path::new(&file), RecursiveMode::NonRecursive) {
                Ok(_) => {}
                Err(error) => {
                    fatal_error(&format!("Failed to watch file '{}': {}", &file, error));
                }
            }
        }

        // Notify the main thread that we're ready to start processing files
        to_main_thread.send(ToMainEvent::AllOk).unwrap();

        // Start listening to events
        loop {
            // Fetch file change events from the watcher
            loop {
                // Keep processing them until we reive a
                match file_rx.try_recv() {
                    Ok(event) => {
                        // We only care about write events
                        if let DebouncedEvent::Write(path) = event {
                            // TODO: Read changed lines
                            // TODO: Send to Processing thread
                            println!("Write executed on {:?}", path.to_str().unwrap())
                        }
                    },
                    Err(watch_error) => {
                        match watch_error {
                            TryRecvError::Empty => {
                                // There is no message waiting, stop this loop.
                                break;
                            }
                            TryRecvError::Disconnected => eprintln!("Watch error {:?}", watch_error)
                        }
                    }
                }
            }

            // Check if there are no messages waiting from the main thread
            match self.message_receiver.try_recv() {
                Ok(event) => {
                    match event {
                        FromMainEvent::Shutdown => {
                            println!("Shutdown command received from main thread.");
                            return;
                        }
                    }
                }
                Err(message_error) => {
                    match message_error {
                        TryRecvError::Empty => {}
                        TryRecvError::Disconnected => {
                            eprintln!("Error communicating with main thread {:?}", message_error);
                            eprintln!("Shutting down thread.");
                            return;
                        }
                    }
                }
            }
        }
    }

}

pub fn start(files: Vec<String>,
             message_receiver: Receiver<FromMainEvent>,
             to_main_thread: Sender<ToMainEvent>,
             to_processing_thread: Sender<FileEvent>,
             only_tail_files: bool,
) -> JoinHandle<()> {
    let processor = FilesProcessor {
        message_receiver,
        to_processing_thread,
    };

    return thread::spawn(move || {
        processor.run(to_main_thread, files, only_tail_files);
    });
}