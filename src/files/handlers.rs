use std::io::{BufReader, SeekFrom, Error, Seek, Read, BufRead};
use std::fs::File;
use std::path::Path;
use crate::common::fatal_error;

#[derive(Debug)]
pub struct WatchedFile {

    pub reader: BufReader<File>,
    filename: String,
    cursor: SeekFrom,

}

impl WatchedFile {

    pub fn read_to_end(& mut self) -> Vec<String> {
        self.seek_to_cursor();

        // Read the lines and collect them into a vector
        let lines: Vec<String> = self.reader.by_ref().lines().map(|line| {
            line.unwrap()
        }).collect();

        // Update the cursor to the read position
        self.update_cursor();

        return lines;
    }

    pub fn get_path(&self) -> &Path {
        return Path::new(&self.filename);
    }

    pub fn get_filename(&self) -> &str {
        return &self.filename;
    }

    // Update the underlying reader to the set cursor position
    fn seek_to_cursor(&mut self) {
        self.reader.seek(self.cursor).unwrap();
    }

    // Update the cursor to underlying reader position
    fn update_cursor(&mut self) {
        // Get the current position of the reader in the file.
        let current_position = self.reader.seek(SeekFrom::Current(0)).unwrap();

        // Next read should be done from the start + that number of bytes
        self.cursor = SeekFrom::Start(current_position);
    }

    pub fn new(filename: String, set_pointer_to_end: bool) -> Self {
        // Open the file for reading
        let mut file = match File::open(Path::new(&filename)) {
            Ok(file) => file,
            Err(error) => {
                fatal_error(&format!("Failed to open file: {}", error));
            }
        };

        let mut file = WatchedFile {
            filename,
            reader: BufReader::new(file),
            cursor: if set_pointer_to_end {
                SeekFrom::End(0)
            } else {
                SeekFrom::Start(0)
            }
        };

        return file;
    }


}